use super::ids::{NpcId, NpcTopicId};

#[derive(Debug, Clone)]
pub struct NpcTopic {
    pub id: NpcTopicId,
    pub name: String,
    pub response: String,
}

#[derive(Debug, Clone)]
pub struct Npc {
    pub id: NpcId,
    pub name: String,
    pub room_description: String,
    pub description: String,
    pub greeting: String,
    pub topics: Vec<NpcTopic>,
}
