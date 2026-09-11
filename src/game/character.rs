use std::collections::{HashMap, HashSet};

use super::ids::{CharacterId, FactKey, QuestKey, RoomId};
use super::quest::QuestProgress;

#[derive(Debug)]
pub struct Character {
    pub id: CharacterId,
    pub name: String,
    pub current_room: RoomId,
    pub facts: HashSet<FactKey>,
    pub quests: HashMap<QuestKey, QuestProgress>,
}
