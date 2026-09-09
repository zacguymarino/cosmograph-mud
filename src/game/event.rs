use super::command::TargetKind;
use super::direction::Direction;
use super::ids::{CharacterId, FactKey, FeatureId, ItemId, NpcId, NpcTopicId, QuestKey, RoomId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TakeFailureReason {
    NotFound,
    Ambiguous { match_count: usize },
    OrdinalOutOfRange { requested: usize, available: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DropFailureReason {
    NotFound,
    Ambiguous { match_count: usize },
    OrdinalOutOfRange { requested: usize, available: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExamineFailureReason {
    NotFound,
    Ambiguous { kinds: Vec<TargetKind> },
    OrdinalOutOfRange { requested: usize, available: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TalkFailureReason {
    NotFound,
    Ambiguous { match_count: usize },
    OrdinalOutOfRange { requested: usize, available: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AskFailureReason {
    NpcNotFound,
    NpcAmbiguous { match_count: usize },
    NpcOrdinalOutOfRange { requested: usize, available: usize },
    TopicNotFound { npc_name: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedItem {
    pub id: ItemId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedFeature {
    pub id: FeatureId,
    pub name: String,
    pub room_description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedNpc {
    pub id: NpcId,
    pub name: String,
    pub room_description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExaminedItem {
    pub id: ItemId,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExaminedFeature {
    pub id: FeatureId,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExaminedNpc {
    pub id: NpcId,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedQuest {
    pub key: QuestKey,
    pub name: String,
    pub description: String,
    pub current_objective: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEvent {
    RoomObserved {
        room_id: RoomId,
        name: String,
        description: String,
        items: Vec<ObservedItem>,
        features: Vec<ObservedFeature>,
        npcs: Vec<ObservedNpc>,
        exits: Vec<Direction>,
    },

    ExitsObserved {
        room_id: RoomId,
        exits: Vec<Direction>,
    },

    HelpRequested,

    CharacterMoved {
        character_id: CharacterId,
        from: RoomId,
        to: RoomId,
        direction: Direction,
    },

    MovementFailed {
        character_id: CharacterId,
        room_id: RoomId,
        direction: Direction,
    },

    ItemTaken {
        character_id: CharacterId,
        room_id: RoomId,
        item: ObservedItem,
    },

    TakeFailed {
        character_id: CharacterId,
        room_id: RoomId,
        query: String,
        reason: TakeFailureReason,
    },

    InventoryObserved {
        items: Vec<ObservedItem>,
    },

    QuestsObserved {
        active: Vec<ObservedQuest>,
        completed: Vec<ObservedQuest>,
    },

    QuestStarted {
        character_id: CharacterId,
        quest: ObservedQuest,
    },

    QuestAdvanced {
        character_id: CharacterId,
        quest: ObservedQuest,
    },

    QuestCompleted {
        character_id: CharacterId,
        quest: ObservedQuest,
    },

    ItemDropped {
        character_id: CharacterId,
        room_id: RoomId,
        item: ObservedItem,
    },

    DropFailed {
        character_id: CharacterId,
        room_id: RoomId,
        query: String,
        reason: DropFailureReason,
    },

    ItemExamined {
        character_id: CharacterId,
        item: ExaminedItem,
    },

    FeatureExamined {
        character_id: CharacterId,
        feature: ExaminedFeature,
    },

    NpcExamined {
        character_id: CharacterId,
        npc: ExaminedNpc,
    },

    NpcSpoke {
        character_id: CharacterId,
        npc_id: NpcId,
        name: String,
        greeting: String,
        topics: Vec<String>,
    },

    TalkFailed {
        character_id: CharacterId,
        query: String,
        reason: TalkFailureReason,
    },

    NpcAnswered {
        character_id: CharacterId,
        npc_id: NpcId,
        npc_name: String,
        topic_id: NpcTopicId,
        topic_name: String,
        response: String,
        learned_facts: Vec<FactKey>,
        updated_topics: Option<Vec<String>>,
    },

    AskFailed {
        character_id: CharacterId,
        npc_query: String,
        topic_query: String,
        reason: AskFailureReason,
    },

    ExamineFailed {
        character_id: CharacterId,
        query: String,
        reason: ExamineFailureReason,
    },
}
