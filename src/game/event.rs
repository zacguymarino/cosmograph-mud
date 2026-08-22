use super::direction::Direction;
use super::ids::{CharacterId, ItemId, RoomId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TakeFailureReason {
    NotFound,
    Ambiguous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropFailureReason {
    NotFound,
    Ambiguous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExamineFailureReason {
    NotFound,
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedItem {
    pub id: ItemId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExaminedItem {
    pub id: ItemId,
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
        exits: Vec<Direction>,
    },

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

    ExamineFailed {
        character_id: CharacterId,
        query: String,
        reason: ExamineFailureReason,
    },
}
