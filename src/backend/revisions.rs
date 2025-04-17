use std::fmt::Debug;

use bevy::prelude::*;

#[derive(Clone, Component, Debug, Default, PartialEq, Eq, Reflect)]
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
    pub bookmarks: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Reflect)]
pub struct ChangeId(pub String, pub usize);

impl ChangeId {
    pub fn into_parts(&self) -> Parts<&str> {
        let (head, tail) = self.0.split_at(self.1);
        Parts { head, tail }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Reflect)]
pub struct CommitId(pub String, pub usize);

impl CommitId {
    pub fn into_parts(&self) -> Parts<&str> {
        let (head, tail) = self.0.split_at(self.1);
        Parts { head, tail }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Reflect)]
pub struct Parts<T: Debug + Default> {
    pub head: T,
    pub tail: T,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_id_into_parts() {
        let change_id = ChangeId("abcdef123456".to_string(), 6);
        let parts = change_id.into_parts();
        assert_eq!(parts.head, "abcdef");
        assert_eq!(parts.tail, "123456");
    }

    #[test]
    fn test_commit_id_into_parts() {
        let commit_id = CommitId("123456abcdef".to_string(), 6);
        let parts = commit_id.into_parts();
        assert_eq!(parts.head, "123456");
        assert_eq!(parts.tail, "abcdef");
    }

    #[test]
    fn test_parts_default() {
        let parts: Parts<String> = Parts::default();
        assert_eq!(parts.head, String::default());
        assert_eq!(parts.tail, String::default());
    }
}
