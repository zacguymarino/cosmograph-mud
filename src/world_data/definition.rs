use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct WorldDefinition {
    pub id: String,
    pub name: String,
    pub starting_room: String,

    #[serde(default)]
    pub items: Vec<ItemDefinition>,
    #[serde(default)]
    pub features: Vec<FeatureDefinition>,
    #[serde(default)]
    pub npcs: Vec<NpcDefinition>,

    pub rooms: Vec<RoomDefinition>,
}

#[derive(Debug, Deserialize)]
pub struct RoomDefinition {
    pub id: String,
    pub name: String,
    pub description: String,

    #[serde(default)]
    pub items: Vec<String>,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub npcs: Vec<String>,

    pub exits: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct ItemDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct FeatureDefinition {
    pub id: String,
    pub name: String,
    pub room_description: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct NpcDefinition {
    pub id: String,
    pub name: String,
    pub room_description: String,
    pub description: String,
    pub greeting: String,
    #[serde(default)]
    pub topics: Vec<NpcTopicDefinition>,
}

#[derive(Debug, Deserialize)]
pub struct NpcTopicDefinition {
    pub id: String,
    pub name: String,
    pub response: String,
}
