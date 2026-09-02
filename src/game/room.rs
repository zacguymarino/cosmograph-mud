use std::collections::HashMap;

use super::direction::Direction;
use super::ids::{FeatureId, ItemId, NpcId, RoomId};

#[derive(Debug, Clone)]
pub struct Room {
    pub id: RoomId,
    pub name: String,
    pub description: String,
    pub items: Vec<ItemId>,
    pub features: Vec<FeatureId>,
    pub npcs: Vec<NpcId>,
    pub exits: HashMap<Direction, RoomId>,
}
