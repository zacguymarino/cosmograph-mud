use std::collections::HashMap;

use super::direction::Direction;
use super::ids::{ItemId, RoomId};

#[derive(Debug, Clone)]
pub struct Room {
    pub id: RoomId,
    pub name: String,
    pub description: String,
    pub items: Vec<ItemId>,
    pub exits: HashMap<Direction, RoomId>,
}
