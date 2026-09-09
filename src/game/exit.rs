use super::ids::{ItemId, RoomId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExitRequirement {
    CarryingItem(ItemId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exit {
    pub destination: RoomId,
    pub requirement: Option<ExitRequirement>,
    pub failure_message: Option<String>,
}

impl Exit {
    pub fn unrestricted(destination: RoomId) -> Self {
        Self {
            destination,
            requirement: None,
            failure_message: None,
        }
    }
}
