use super::ids::{CharacterId, ItemId, RoomId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemLocation {
    Nowhere,
    Room(RoomId),
    CarriedBy(CharacterId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemPlacement {
    pub item_id: ItemId,
    pub location: ItemLocation,
}
