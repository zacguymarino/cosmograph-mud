use super::ids::QuestId;

#[derive(Debug, Clone)]
pub struct Quest {
    pub id: QuestId,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestStatus {
    Active,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestProgress {
    pub status: QuestStatus,
}
