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
}

#[derive(Clone, Debug, Default, PartialEq, Reflect)]
pub struct ChangeId(pub String, pub usize);

impl ChangeId {
    pub fn into_parts(&self) -> Parts {
        let (head, tail) = self.0.split_at(self.1);
        Parts { head, tail }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Reflect)]
pub struct CommitId(pub String, pub usize);

impl CommitId {
    pub fn into_parts(&self) -> Parts {
        let (head, tail) = self.0.split_at(self.1);
        Parts { head, tail }
    }
}

pub struct Parts<'a> {
    pub head: &'a str,
    pub tail: &'a str,
}
