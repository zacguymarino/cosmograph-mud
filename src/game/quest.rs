use super::ids::{QuestId, QuestStepId, RoomId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestObjective {
    ReachRoom(RoomId),
}

#[derive(Debug, Clone)]
pub struct QuestStep {
    pub id: QuestStepId,
    pub description: String,
    pub objective: QuestObjective,
    pub next_step: Option<QuestStepId>,
}

#[derive(Debug, Clone)]
pub struct Quest {
    pub id: QuestId,
    pub name: String,
    pub description: String,
    pub starting_step: QuestStepId,
    pub steps: Vec<QuestStep>,
}

impl Quest {
    pub fn step(&self, id: &QuestStepId) -> Option<&QuestStep> {
        self.steps.iter().find(|step| &step.id == id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestStatus {
    Active,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestProgress {
    pub status: QuestStatus,
    pub current_step: QuestStepId,
}
