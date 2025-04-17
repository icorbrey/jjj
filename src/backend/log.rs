use anyhow::{anyhow, Result};
use bevy::{prelude::*, utils::hashbrown::HashMap};
use enum_as_inner::EnumAsInner;
use regex::{Captures, Match, Regex};

use crate::{
    app::AppSet, backend::revisions::Parts, errors, events::prelude::*, join, screens::Screen,
    utils::AppExt,
};

use super::{
    revisions::{ChangeId, CommitId, Revision},
    JujutsuCli,
};

#[derive(Debug, Default, Event)]
pub struct RefreshLogEvent;

#[derive(Clone, Debug, Event, PartialEq)]
pub struct LogRevsetEvent(pub String);

#[derive(Event, Deref, DerefMut)]
pub struct LogResponseEvent(pub Vec<LogOutput>);

#[derive(Clone, Debug, PartialEq, Eq, EnumAsInner)]
pub enum LogOutput {
    Revision(Revision),
    Decoration(String),
}

#[derive(Default, Deref, DerefMut, Reflect, Resource)]
pub struct CurrentRevset(pub Option<String>);

#[mutants::skip]
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
    jj_cli: Res<JujutsuCli>,
) -> Result<()> {
    for LogRevsetEvent(revset) in ev_log_revset.read() {
        debug!("Reading log for revset: {revset}");

        let details = jj_cli.log(revset, DETAILS_TEMPLATE)?;
        let descriptions = jj_cli.log(revset, DESCRIPTION_TEMPLATE)?;

        debug!(details);
        debug!(descriptions);

        let descriptions = parse_descriptions(descriptions)?;

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

#[derive(Debug)]
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

#[tracing::instrument]
fn parse_details(lines: &str) -> Result<Option<Details>> {
    trace!("Attempting match");

    let Some(details_caps) = Regex::new(MATCH_DETAILS)?.captures(lines) else {
        return Ok(None);
    };

    trace!("{:?}", details_caps);

    let change_id = require(&details_caps, "change_id")?.as_str().to_string();
    let change_id_shortest = require(&details_caps, "change_id_shortest")?.as_str().len();

    let commit_id = require(&details_caps, "commit_id")?.as_str().to_string();
    let commit_id_shortest = require(&details_caps, "commit_id_shortest")?.as_str().len();

    let author = require(&details_caps, "author")?.as_str().to_string();
    let timestamp = require(&details_caps, "timestamp")?.as_str().to_string();

    let bookmarks = (details_caps.name("bookmarks"))
        .map(|d| {
            (d.as_str().trim().split(" "))
                .filter(|x| !x.is_empty())
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

fn parse_descriptions(descriptions: String) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for line in descriptions.split("%JJJ%") {
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

#[cfg(test)]
mod tests {

    use bevy::ecs::system::RunSystemOnce;

    use crate::{backend::JujutsuCli, events};

    use super::*;

    #[test]
    fn refresh_log() -> Result<()> {
        let mut app = App::new();

        app.add_plugins(events::plugin);
        app.add_systems(Update, super::refresh_log);

        app.init_resource::<CurrentRevset>();

        app.update();

        // Nothing happens when no refresh event has been sent.
        (app.world_mut()).run_system_once(
            |ev_log_revset: EventReader<LogRevsetEvent>,
             mut ev_refresh_log: EventWriter<RefreshLogEvent>| {
                assert!(ev_log_revset.is_empty());
                ev_refresh_log.send(default());
            },
        )?;

        app.update();

        // Nothing happens when a refresh event has been sent but there's no current revset.
        (app.world_mut()).run_system_once(
            |ev_log_revset: EventReader<LogRevsetEvent>,
             mut current_revset: ResMut<CurrentRevset>,
             mut ev_refresh_log: EventWriter<RefreshLogEvent>| {
                assert!(ev_log_revset.is_empty());
                current_revset.0 = Some("foo".to_string());
                ev_refresh_log.send(default());
            },
        )?;

        app.update();

        // A log revset event is sent when a revset is currently selected and a refresh event is
        // sent.
        (app.world_mut()).run_system_once(|mut ev_log_revset: EventReader<LogRevsetEvent>| {
            assert!(!ev_log_revset.is_empty());
            for ev in ev_log_revset.read() {
                assert_eq!(ev, &LogRevsetEvent("foo".to_string()));
            }
        })?;

        Ok(())
    }

    #[test]
    fn read_logs() -> Result<()> {
        let mut app = App::new();

        app.add_plugins(events::plugin);
        app.add_systems(Update, super::read_logs.pipe(errors::forward));

        app.init_resource::<CurrentRevset>();
        app.insert_resource(JujutsuCli::mock(|_| Ok(String::new())));

        app.update();

        // Nothing happens when no log revset event is sent.
        (app.world_mut()).run_system_once(
            |ev_notification: EventReader<NotificationEvent>,
             mut ev_log_revset: EventWriter<LogRevsetEvent>,
             ev_log_response: EventReader<LogResponseEvent>,
             ev_error: EventReader<ErrorEvent>| {
                assert!(ev_notification.is_empty());
                assert!(ev_log_response.is_empty());
                assert!(ev_error.is_empty());

                ev_log_revset.send(LogRevsetEvent("test".into()));
            },
        )?;

        app.insert_resource(JujutsuCli::mock(|_| Err(anyhow!("foo"))));

        app.update();

        // Jujutus CLI errors are forwarded to the error notifier.
        (app.world_mut()).run_system_once(
            |mut ev_log_revset: EventWriter<LogRevsetEvent>,
             mut ev_error: EventReader<ErrorEvent>| {
                assert_eq!(ev_error.len(), 1);
                for ev in ev_error.read() {
                    assert_eq!(ev.as_str(), "Couldn't read log for revset `test`");
                }

                ev_log_revset.send(LogRevsetEvent("test".into()));
            },
        )?;

        app.insert_resource(JujutsuCli::mock(|_| Ok(String::new())));

        app.update();

        // Empty revsets are sent to the gulag.
        (app.world_mut()).run_system_once(
            |mut ev_notification: EventReader<NotificationEvent>,
             mut ev_log_revset: EventWriter<LogRevsetEvent>,
             ev_log_response: EventReader<LogResponseEvent>,
             ev_error: EventReader<ErrorEvent>| {
                assert_eq!(ev_error.len(), 1);

                assert!(!ev_notification.is_empty());
                assert!(ev_log_response.is_empty());

                assert_eq!(ev_notification.len(), 1);
                for ev in ev_notification.read() {
                    assert!(ev.angry);
                    assert_eq!(ev.message, String::from("No revisions found for `test`"));
                }

                ev_log_revset.send(LogRevsetEvent("test".into()));
            },
        )?;

        app.insert_resource(JujutsuCli::mock(|args| {
            Ok(if args.iter().any(|s| s.contains("description")) {
                [
                    ["abcdefgh", "Change 1\n\nLots of cool things!\n"],
                    ["hgfedcba", ""],
                ]
                .map(|l| format!("[{}]%JJJ%", l.join("&JJJ&")))
                .join("")
            } else {
                [
                    (
                        "foo",
                        [
                            "a",
                            "abcdefgh",
                            "12",
                            "12345678",
                            "John Smith",
                            "2 days ago",
                            "",
                            "true",
                            "false",
                            "true",
                            "false",
                        ],
                        "bar",
                    ),
                    (
                        "kek",
                        [
                            "hgf",
                            "hgfedcba",
                            "8765",
                            "87654321",
                            "Jane Doe",
                            "5 minutes ago",
                            "trunk",
                            "false",
                            "true",
                            "false",
                            "true",
                        ],
                        "lel",
                    ),
                ]
                .map(|(head, l, tail)| format!("{head}[{}]\n{tail}%JJJ%", l.join("&JJJ&")))
                .join("\nDECOR\n")
            })
        }));

        app.update();

        // Empty revsets are sent to the gulag.
        (app.world_mut()).run_system_once(
            |mut ev_log_response: EventReader<LogResponseEvent>,
             ev_notification: EventReader<NotificationEvent>| {
                assert_eq!(ev_notification.len(), 1);

                assert_eq!(ev_log_response.len(), 1);
                for ev in ev_log_response.read() {
                    assert_eq!(
                        ev.0,
                        vec![
                            LogOutput::Revision(Revision {
                                change_id: ChangeId("abcdefgh".into(), 1),
                                commit_id: CommitId("12345678".into(), 2),
                                author: "John Smith".into(),
                                description: Some("Change 1\n\nLots of cool things!".into()),
                                timestamp: "2 days ago".into(),
                                bookmarks: vec![],
                                is_divergent: true,
                                is_immutable: false,
                                is_empty: true,
                                is_root: false,
                                graph: Parts {
                                    head: "foo".into(),
                                    tail: "bar".into()
                                }
                            }),
                            LogOutput::Decoration("DECOR".into()),
                            LogOutput::Revision(Revision {
                                change_id: ChangeId("hgfedcba".into(), 3),
                                commit_id: CommitId("87654321".into(), 4),
                                author: "Jane Doe".into(),
                                description: None,
                                timestamp: "5 minutes ago".into(),
                                bookmarks: vec!["trunk".into()],
                                is_divergent: false,
                                is_immutable: true,
                                is_empty: false,
                                is_root: true,
                                graph: Parts {
                                    head: "kek".into(),
                                    tail: "lel".into()
                                }
                            }),
                        ]
                    );
                }
            },
        )?;

        Ok(())
    }
}
