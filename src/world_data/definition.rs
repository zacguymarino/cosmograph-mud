use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct WorldDefinition {
    pub id: String,
    pub name: String,
    pub starting_room: String,

    #[serde(default)]
    pub items: Vec<ItemDefinition>,

    pub rooms: Vec<RoomDefinition>,
}

#[derive(Debug, Deserialize)]
pub struct RoomDefinition {
    pub id: String,
    pub name: String,
    pub description: String,

    #[serde(default)]
    pub items: Vec<String>,

    pub exits: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct ItemDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
}
