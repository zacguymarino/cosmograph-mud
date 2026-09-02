use super::ids::NpcId;

#[derive(Debug, Clone)]
pub struct Npc {
    pub id: NpcId,
    pub name: String,
    pub room_description: String,
    pub description: String,
    pub greeting: String,
}
