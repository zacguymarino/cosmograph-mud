use super::ids::{FactId, ItemId, RoomId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExitRequirement {
    CarryingItem(ItemId),
    KnowsFact(FactId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exit {
    pub destination: RoomId,
    pub requirements: Vec<ExitRequirement>,
    pub failure_message: Option<String>,
}

impl Exit {
    pub fn unrestricted(destination: RoomId) -> Self {
        Self {
            destination,
            requirements: vec![],
            failure_message: None,
        }
    }
}
