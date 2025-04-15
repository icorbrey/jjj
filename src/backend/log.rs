use anyhow::{anyhow, Result};
use bevy::prelude::*;
use regex::{Captures, Match, Regex};

use crate::{
    app::AppSet, backend::revisions::Parts, errors, events::prelude::*, join, screens::Screen,
    utils::AppExt,
};

use super::{
    execute_jj_command,
    revisions::{ChangeId, CommitId, Revision},
};

#[derive(Default, Event)]
pub struct RefreshLogEvent;

#[derive(Event)]
pub struct LogRevsetEvent(pub String);

#[derive(Event, Deref, DerefMut)]
pub struct LogResponseEvent(pub Vec<Revision>);

#[derive(Default, Deref, DerefMut, Reflect, Resource)]
pub struct CurrentRevset(pub Option<String>);

#[tracing::instrument(skip_all)]
pub fn plugin(app: &mut App) {
    trace!("Initializing plugin...");

    app.register_scoped_type::<CurrentRevset>(Screen::Interface);
    app.add_systems(
        Update,
        (refresh_log, read_logs.pipe(errors::forward))
            .chain()
            .in_set(AppSet::Update)
            .run_if(in_state(Screen::Interface)),
    );

    trace!("Plugin initialized.");
}

const DETAILS_TEMPLATE: &str = concat!(
    "'[' ++ ",
    join!(
        " ++ '&JJJ&' ++ ",
        [
            "change_id.shortest()",
            "change_id.shortest(8)",
            "commit_id.shortest()",
            "commit_id.shortest(8)",
            "author",
            "format_timestamp(author.timestamp())",
            "bookmarks",
            "divergent",
            "immutable",
            "empty",
            "root"
        ]
    ),
    " ++ ']' ++ \"\n\" ++ '%JJJ%'"
);

const MATCH_DETAILS: &str = concat!(
    r#"(?s)^(?P<graph_head>.*?)\["#,
    join!(
        r#"&JJJ&"#,
        [
            r#"(?P<change_id_shortest>.*)"#,
            r#"(?P<change_id>.*)"#,
            r#"(?P<commit_id_shortest>.*)"#,
            r#"(?P<commit_id>.*)"#,
            r#"(?P<author>.*)"#,
            r#"(?P<timestamp>.*)"#,
            r#"(?P<bookmarks>.*)"#,
            r#"(?P<divergent>.*)"#,
            r#"(?P<immutable>.*)"#,
            r#"(?P<empty>.*)"#,
            r#"(?P<root>.*)"#
        ]
    ),
    r#"\]\n(?P<graph_tail>.*?)$"#,
);

const DESCRIPTION_TEMPLATE: &str = concat!(
    "'[' ++ ",
    join!(" ++ '&JJJ&' ++ ", ["change_id.shortest(8)", "description"]),
    " ++ ']%JJJ%'"
);

const MATCH_DESCRIPTION: &str = concat!(
    r#"(?s)\["#,
    join!(
        r#"&JJJ&"#,
        [r#"(?P<change_id>.*)"#, r#"(?P<description>.*)"#]
    ),
    r#"\]"#,
);

#[tracing::instrument(skip_all)]
fn refresh_log(
    mut ev_refresh_log: EventReader<RefreshLogEvent>,
    mut ev_log_revset: EventWriter<LogRevsetEvent>,
    current_revset: Res<CurrentRevset>,
) {
    for _ in ev_refresh_log.read() {
        if let Some(revset) = current_revset.0.clone() {
            ev_log_revset.send(LogRevsetEvent(revset));
        }
    }
}

#[tracing::instrument(skip_all)]
fn read_logs(
    mut ev_notification: EventWriter<NotificationEvent>,
    mut ev_log_response: EventWriter<LogResponseEvent>,
    mut ev_log_revset: EventReader<LogRevsetEvent>,
    mut current_revset: ResMut<CurrentRevset>,
) -> Result<()> {
    for LogRevsetEvent(revset) in ev_log_revset.read() {
        debug!("Reading log for revset: {revset}");

        let details =
            execute_jj_command(vec!["log", "-r", revset.as_str(), "-T", DETAILS_TEMPLATE])
                .map_err(|_| anyhow!("Couldn't read log for revset `{revset}`"))?;

        let descriptions = execute_jj_command(vec![
            "log",
            "--no-graph",
            "-r",
            revset.as_str(),
            "-T",
            DESCRIPTION_TEMPLATE,
        ])
        .map_err(|_| anyhow!("Couldn't read log for revset `{revset}`"))?;

        let mut revs = vec![];
        for line in (details.split("%JJJ%\n").zip(descriptions.split("%JJJ%")))
            .filter(|(a, b)| !a.trim().is_empty() && !b.trim().is_empty())
            .map(parse_line)
            .filter_map(|l| l.transpose())
        {
            revs.push(line?);
        }

        if !revs.is_empty() {
            ev_log_response.send(LogResponseEvent(revs));
            current_revset.0 = Some(revset.clone());
        } else {
            ev_notification.send(NotificationEvent::angry(format!(
                "No revisions found for `{revset}`"
            )));
        }
    }

    Ok(())
}

fn parse_line((details_line, description_line): (&str, &str)) -> Result<Option<Revision>> {
    let Some(details_caps) = Regex::new(MATCH_DETAILS)?.captures(details_line) else {
        return Ok(None);
    };

    let Some(description_caps) = Regex::new(MATCH_DESCRIPTION)?.captures(description_line) else {
        return Ok(None);
    };

    let change_id = require(&details_caps, "change_id")?.as_str().to_string();
    let change_id_shortest = require(&details_caps, "change_id_shortest")?.as_str().len();

    // These lines should always match for the same revset.
    assert_eq!(
        change_id,
        require(&description_caps, "change_id")?
            .as_str()
            .to_string()
    );

    let commit_id = require(&details_caps, "commit_id")?.as_str().to_string();
    let commit_id_shortest = require(&details_caps, "commit_id_shortest")?.as_str().len();

    let author = require(&details_caps, "author")?.as_str().to_string();
    let timestamp = require(&details_caps, "timestamp")?.as_str().to_string();

    let bookmarks = (details_caps.name("bookmarks"))
        .map(|d| {
            (d.as_str().trim().split(" "))
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let description = (description_caps.name("description"))
        .map(|d| d.as_str().trim().to_string())
        .filter(|d| !d.is_empty());

    let is_divergent = require(&details_caps, "divergent")?.as_str() == "true";
    let is_immutable = require(&details_caps, "immutable")?.as_str() == "true";
    let is_empty = require(&details_caps, "empty")?.as_str() == "true";
    let is_root = require(&details_caps, "root")?.as_str() == "true";

    let graph_head = require(&details_caps, "graph_head")?.as_str().to_string();
    let graph_tail = require(&details_caps, "graph_tail")?.as_str().to_string();

    Ok(Some(Revision {
        change_id: ChangeId(change_id, change_id_shortest),
        commit_id: CommitId(commit_id, commit_id_shortest),
        graph: Parts {
            head: graph_head,
            tail: graph_tail,
        },
        is_divergent,
        is_immutable,
        description,
        is_empty,
        is_root,
        author,
        timestamp,
        bookmarks,
    }))
}

fn require<'a>(caps: &Captures<'a>, name: &str) -> Result<Match<'a>> {
    caps.name(name)
        .ok_or(anyhow!("Couldn't find `{name}` in captures: {caps:?}"))
}
