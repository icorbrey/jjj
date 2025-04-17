use anyhow::{anyhow, Result};
use bevy::{prelude::*, utils::hashbrown::HashMap};
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
pub struct LogResponseEvent(pub Vec<LogOutput>);

#[derive(Clone)]
pub enum LogOutput {
    Revision(Revision),
    Decoration(String),
}

impl LogOutput {
    pub fn revision(&self) -> Option<Revision> {
        match self {
            Self::Revision(revision) => Some(revision.clone()),
            _ => None,
        }
    }
}

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

        let descriptions = get_descriptions(revset)?;

        let mut detail_lines = details.lines().peekable();
        let mut results = vec![];

        'parse: loop {
            // Stop at the end of the iterator
            let Some(line_one) = detail_lines.next() else {
                break 'parse;
            };

            // We shouldn't encounter empty lines in Jujutsu's output, but this should prevent any
            // catastrophic failure.
            if line_one.trim().is_empty() {
                continue;
            }

            debug!("{line_one}");

            match detail_lines.peek() {
                Some(line_two) => {
                    let lines = format!("{line_one}\n{line_two}");
                    match parse_details(lines.as_str())? {
                        // This is a real revision, merge the details with the optional description
                        // and push it.
                        Some(details) => {
                            let description = descriptions.get(&details.change_id.0).cloned();
                            results.push(LogOutput::Revision(Revision {
                                is_divergent: details.is_divergent,
                                is_immutable: details.is_immutable,
                                bookmarks: details.bookmarks,
                                change_id: details.change_id,
                                commit_id: details.commit_id,
                                timestamp: details.timestamp,
                                is_empty: details.is_empty,
                                is_root: details.is_root,
                                author: details.author,
                                graph: details.graph,
                                description,
                            }));

                            // We've verified the next line is part of the revision, so consume it.
                            let _ = detail_lines.next();
                        }

                        // No match means this isn't a revision and we can
                        // treat it as a decoration.
                        None => results.push(LogOutput::Decoration(line_one.to_string())),
                    }
                }

                // No extra line means we're at the end of the graph and this is a decoration.
                None => results.push(LogOutput::Decoration(line_one.to_string())),
            }
        }

        if !results.is_empty() {
            ev_log_response.send(LogResponseEvent(results));
            current_revset.0 = Some(revset.clone());
        } else {
            ev_notification.send(NotificationEvent::angry(format!(
                "No revisions found for `{revset}`"
            )));
        }
    }

    Ok(())
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
    r#"\]\n(?P<graph_tail>.*?)%JJJ%$"#,
);

struct Details {
    change_id: ChangeId,
    commit_id: CommitId,
    is_divergent: bool,
    is_immutable: bool,
    is_empty: bool,
    is_root: bool,
    graph: Parts<String>,
    author: String,
    timestamp: String,
    bookmarks: Vec<String>,
}

fn parse_details(lines: &str) -> Result<Option<Details>> {
    let Some(details_caps) = Regex::new(MATCH_DETAILS)?.captures(lines) else {
        return Ok(None);
    };

    let change_id = require(&details_caps, "change_id")?.as_str().to_string();
    let change_id_shortest = require(&details_caps, "change_id_shortest")?.as_str().len();

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

    let is_divergent = require(&details_caps, "divergent")?.as_str() == "true";
    let is_immutable = require(&details_caps, "immutable")?.as_str() == "true";
    let is_empty = require(&details_caps, "empty")?.as_str() == "true";
    let is_root = require(&details_caps, "root")?.as_str() == "true";

    let graph_head = require(&details_caps, "graph_head")?.as_str().to_string();
    let graph_tail = require(&details_caps, "graph_tail")?.as_str().to_string();

    Ok(Some(Details {
        change_id: ChangeId(change_id, change_id_shortest),
        commit_id: CommitId(commit_id, commit_id_shortest),
        graph: Parts {
            head: graph_head,
            tail: graph_tail,
        },
        is_divergent,
        is_immutable,
        is_empty,
        is_root,
        author,
        timestamp,
        bookmarks,
    }))
}

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

fn get_descriptions(revset: &str) -> Result<HashMap<String, String>> {
    let response = execute_jj_command(vec![
        "log",
        "--no-graph",
        "-r",
        revset,
        "-T",
        DESCRIPTION_TEMPLATE,
    ])
    .map_err(|_| anyhow!("Couldn't read log for revset `{revset}`"))?;

    let mut map = HashMap::new();
    for line in response.split("%JJJ%") {
        let Some((change_id, description)) = parse_description(line)? else {
            continue;
        };
        map.insert(change_id, description);
    }

    Ok(map)
}

#[tracing::instrument]
fn parse_description(line: &str) -> Result<Option<(String, String)>> {
    let Some(caps) = Regex::new(MATCH_DESCRIPTION)?.captures(line) else {
        return Ok(None);
    };

    let change_id = require(&caps, "change_id")?.as_str().to_string();
    let description = (caps.name("description"))
        .map(|c| c.as_str().trim().to_string())
        .filter(|d| !d.is_empty());

    trace!(change_id, description);

    Ok(Some(change_id).zip(description))
}

fn require<'a>(caps: &Captures<'a>, name: &str) -> Result<Match<'a>> {
    caps.name(name)
        .ok_or(anyhow!("Couldn't find `{name}` in captures: {caps:?}"))
}
