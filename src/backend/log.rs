use anyhow::{anyhow, Result};
use bevy::prelude::*;
use regex::{Captures, Match, Regex};

use crate::{app::AppSet, errors, join, screens::Screen, utils::AppExt};

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
    app.register_scoped_type::<CurrentRevset>(Screen::Interface);
    app.add_systems(
        Update,
        (refresh_log, read_logs.pipe(errors::forward))
            .chain()
            .in_set(AppSet::Update)
            .run_if(in_state(Screen::Interface)),
    );

    debug!("Finished loading");
}

const LOG_TEMPLATE: &str = concat!(
    "'[' ++ ",
    join!(
        " ++ '&JJJ&' ++ ",
        [
            "change_id.shortest()",
            "change_id.shortest(8)",
            "commit_id.shortest()",
            "commit_id.shortest(8)",
            "description",
            "divergent",
            "immutable",
            "empty",
            "root"
        ]
    ),
    " ++ ']%JJJ%'"
);

const MATCH_LOG: &str = concat!(
    r#"(?s)\["#,
    join!(
        r#"&JJJ&"#,
        [
            r#"(?P<change_id_shortest>.*)"#,
            r#"(?P<change_id>.*)"#,
            r#"(?P<commit_id_shortest>.*)"#,
            r#"(?P<commit_id>.*)"#,
            r#"(?P<description>.*)"#,
            r#"(?P<divergent>.*)"#,
            r#"(?P<immutable>.*)"#,
            r#"(?P<empty>.*)"#,
            r#"(?P<root>.*)"#
        ]
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
    mut ev_log_response: EventWriter<LogResponseEvent>,
    mut ev_log_revset: EventReader<LogRevsetEvent>,
    mut current_revset: ResMut<CurrentRevset>,
) -> Result<()> {
    for LogRevsetEvent(revset) in ev_log_revset.read() {
        debug!("Reading log for revset: {revset}");

        let log = execute_jj_command(vec!["log", "-r", revset.as_str(), "-T", LOG_TEMPLATE])
            .map_err(|_| anyhow!("Couldn't read log for revset `{revset}`"))?;

        let mut revs = vec![];
        for line in (log.split("%JJJ%"))
            .filter(|l| !l.trim().is_empty())
            .map(parse_line)
            .filter_map(|l| l.transpose())
        {
            revs.push(line?)
        }

        ev_log_response.send(LogResponseEvent(revs));
        current_revset.0 = Some(revset.clone());
    }

    Ok(())
}

fn parse_line(line: &str) -> Result<Option<Revision>> {
    let match_log = Regex::new(MATCH_LOG)?;
    let Some(caps) = match_log.captures(line) else {
        return Ok(None);
    };

    let change_id = require(&caps, "change_id")?.as_str().to_string();
    let change_id_shortest = require(&caps, "change_id_shortest")?.as_str().len();

    let commit_id = require(&caps, "commit_id")?.as_str().to_string();
    let commit_id_shortest = require(&caps, "commit_id_shortest")?.as_str().len();

    let description = (caps.name("description"))
        .map(|d| d.as_str().trim().to_string())
        .filter(|d| !d.is_empty());

    let is_divergent = require(&caps, "divergent")?.as_str() == "true";
    let is_immutable = require(&caps, "immutable")?.as_str() == "true";
    let is_empty = require(&caps, "empty")?.as_str() == "true";
    let is_root = require(&caps, "root")?.as_str() == "true";

    Ok(Some(Revision {
        change_id: ChangeId(change_id, change_id_shortest),
        commit_id: CommitId(commit_id, commit_id_shortest),
        is_divergent,
        is_immutable,
        description,
        is_empty,
        is_root,
    }))
}

fn require<'a>(caps: &Captures<'a>, name: &str) -> Result<Match<'a>> {
    caps.name(name)
        .ok_or(anyhow!("Couldn't find `{name}` in captures: {caps:?}"))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rejects_no_match() {
        let line = "~";

        assert_eq!(parse_line(line).unwrap(), None);
    }

    #[test]
    fn parses_empty_description() {
        let line = format!(
            "[{}]",
            join!(
                "&JJJ&",
                ["a", "abcdefg", "hi", "hijklmn", "", "true", "false", "true", "false"]
            )
        );

        assert_eq!(
            parse_line(&line).unwrap(),
            Some(Revision {
                change_id: ChangeId("abcdefg".into(), 1),
                commit_id: CommitId("hijklmn".into(), 2),
                description: None,
                is_divergent: true,
                is_immutable: false,
                is_empty: true,
                is_root: false,
            })
        );
    }

    #[test]
    fn parses_oneline_description() {
        let line = format!(
            "[{}]",
            join!(
                "&JJJ&",
                ["a", "abcdefg", "hi", "hijklmn", "one line", "true", "false", "true", "false"]
            )
        );

        assert_eq!(
            parse_line(&line).unwrap(),
            Some(Revision {
                change_id: ChangeId("abcdefg".into(), 1),
                commit_id: CommitId("hijklmn".into(), 2),
                description: Some("one line".into()),
                is_divergent: true,
                is_immutable: false,
                is_empty: true,
                is_root: false,
            })
        );
    }

    #[test]
    fn parses_multiline_description() {
        let line = format!(
            "[{}]",
            join!(
                "&JJJ&",
                [
                    "a",
                    "abcdefg",
                    "hi",
                    "hijklmn",
                    "one\nline\n",
                    "true",
                    "false",
                    "true",
                    "false"
                ]
            )
        );

        assert_eq!(
            parse_line(&line).unwrap(),
            Some(Revision {
                change_id: ChangeId("abcdefg".into(), 1),
                commit_id: CommitId("hijklmn".into(), 2),
                description: Some("one\nline".into()),
                is_divergent: true,
                is_immutable: false,
                is_empty: true,
                is_root: false,
            })
        );
    }
}
