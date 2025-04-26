use std::fmt::format;

use bevy::prelude::*;
use serde::{
    de::{self, Visitor},
    Deserialize,
};

pub fn plugin(app: &mut App) {}

#[derive(Deserialize, Event)]
#[serde(untagged)]
pub enum Command {
    Static(StaticCommand),
    Typeable(TypeableCommand),
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StaticCommand {
    /// Enter normal mode. This focuses the change buffer.
    NormalMode,

    /// Enter command mode. This focuses the command line and enables using
    /// typable commands.
    CommandMode,

    /// Selects the line below the current selection.
    MoveVisualLineDown,

    /// Selects the line above the current selection.
    MoveVisualLineUp,

    /// Extends the current selection to include the next line down.
    ExtendLineDown,

    /// Extends the current selection to include the next line up.
    ExtendLineUp,

    /// Opens the revset picker.
    RevsetPicker,
}

#[derive(Debug)]
pub enum TypeableCommand {
    OpenRevset(String),
}

impl<'de> Deserialize<'de> for TypeableCommand {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CommandVisitor;

        impl<'de> Visitor<'de> for CommandVisitor {
            type Value = TypeableCommand;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(r#"a string like ":command-name <args>""#)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let v = v.trim();
                let body = v
                    .strip_prefix(':')
                    .ok_or_else(|| E::custom("command must start with ':'"))?;

                let (tag, rest) = body
                    .split_once(char::is_whitespace)
                    .map(|(t, r)| (t, r.trim_start()))
                    .unwrap_or((body, ""));

                match (tag, rest.split(char::is_whitespace)) {
                    ("open-revset", mut args) => {
                        let revset = args.next().ok_or_else(|| E::custom("missing revset"))?;
                        Ok(TypeableCommand::OpenRevset(revset.to_string()))
                    }
                    _ => Err(E::custom(format!("unknown command :{tag}"))),
                }
            }
        }

        deserializer.deserialize_str(CommandVisitor)
    }
}
