use super::command::TargetKind;
use super::direction::Direction;
use super::ids::{CharacterId, FeatureId, ItemId, NpcId, RoomId};

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
    },

    TalkFailed {
        character_id: CharacterId,
        query: String,
        reason: TalkFailureReason,
    },

    ExamineFailed {
        character_id: CharacterId,
        query: String,
        reason: ExamineFailureReason,
    },
}
