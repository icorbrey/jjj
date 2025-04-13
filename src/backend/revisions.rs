use std::fmt::Debug;

use bevy::prelude::*;

#[derive(Clone, Component, Debug, Default, PartialEq, Reflect)]
pub struct Revision {
    pub change_id: ChangeId,
    pub commit_id: CommitId,
    pub description: Option<String>,
    pub is_divergent: bool,
    pub is_immutable: bool,
    pub is_empty: bool,
    pub is_root: bool,
    pub graph: Parts<String>,
    pub author: String,
    pub timestamp: String,
}

#[derive(Clone, Debug, Default, PartialEq, Reflect)]
pub struct ChangeId(pub String, pub usize);

impl ChangeId {
    pub fn into_parts(&self) -> Parts<&str> {
        let (head, tail) = self.0.split_at(self.1);
        Parts { head, tail }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Reflect)]
pub struct CommitId(pub String, pub usize);

impl CommitId {
    pub fn into_parts(&self) -> Parts<&str> {
        let (head, tail) = self.0.split_at(self.1);
        Parts { head, tail }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Reflect)]
pub struct Parts<T: Debug + Default> {
    pub head: T,
    pub tail: T,
}
