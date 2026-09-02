use super::direction::Direction;
use std::num::NonZeroUsize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetKind {
    Item,
    Feature,
    Npc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetQuery {
    pub kind: Option<TargetKind>,
    pub ordinal: Option<NonZeroUsize>,
    pub name: String,
}

impl TargetQuery {
    pub fn any(name: String) -> Self {
        Self {
            kind: None,
            ordinal: None,
            name,
        }
    }

    pub fn item(name: String) -> Self {
        Self {
            kind: Some(TargetKind::Item),
            ordinal: None,
            name,
        }
    }

    pub fn feature(name: String) -> Self {
        Self {
            kind: Some(TargetKind::Feature),
            ordinal: None,
            name,
        }
    }

    pub fn npc(name: String) -> Self {
        Self {
            kind: Some(TargetKind::Npc),
            ordinal: None,
            name,
        }
    }

    pub fn with_ordinal(mut self, ordinal: NonZeroUsize) -> Self {
        self.ordinal = Some(ordinal);
        self
    }
}

impl From<String> for TargetQuery {
    fn from(name: String) -> Self {
        Self::any(name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Look,
    Exits,
    Help,
    Move(Direction),
    Take(TargetQuery),
    Inventory,
    Drop(TargetQuery),
    Examine(TargetQuery),
    Talk(TargetQuery),
}
