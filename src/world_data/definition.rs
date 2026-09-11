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
    #[serde(default)]
    pub facts: Vec<FactDefinition>,
    #[serde(default)]
    pub quests: Vec<QuestDefinition>,

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

    pub exits: HashMap<String, ExitDefinition>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ExitDefinition {
    Simple(String),
    Conditional {
        destination: String,
        #[serde(default)]
        requires_item: Option<String>,
        #[serde(default)]
        requires_fact: Option<String>,
        failure_message: String,
    },
}

impl ExitDefinition {
    pub fn destination(&self) -> &str {
        match self {
            Self::Simple(destination) | Self::Conditional { destination, .. } => destination,
        }
    }
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
    #[serde(default)]
    pub placement: Option<FeaturePlacementDefinition>,
}

#[derive(Debug, Deserialize)]
pub struct FeaturePlacementDefinition {
    pub relation: PlacementRelationDefinition,
    #[serde(default)]
    pub capacity: Option<usize>,
    #[serde(default)]
    pub accepts_items: Vec<String>,
    #[serde(default)]
    pub rejection_message: Option<String>,
    #[serde(default)]
    pub full_message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlacementRelationDefinition {
    On,
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
pub struct FactDefinition {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct QuestDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub starting_step: String,
    pub steps: Vec<QuestStepDefinition>,
}

#[derive(Debug, Deserialize)]
pub struct QuestStepDefinition {
    pub id: String,
    pub description: String,
    pub objective: QuestObjectiveDefinition,
    #[serde(default)]
    pub next_step: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QuestObjectiveDefinition {
    ReachRoom { room: String },
    PossessItem { item: String },
    AskTopic { npc: String, topic: String },
}

#[derive(Debug, Deserialize)]
pub struct NpcTopicDefinition {
    pub id: String,
    pub name: String,
    pub response: String,
    #[serde(default)]
    pub requires_facts: Vec<String>,
    #[serde(default)]
    pub excludes_facts: Vec<String>,
    #[serde(default)]
    pub grants_facts: Vec<String>,
    #[serde(default)]
    pub starts_quests: Vec<String>,
}
