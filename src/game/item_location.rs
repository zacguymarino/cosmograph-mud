use super::ids::{CharacterId, FeatureId, ItemId, RoomId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemLocation {
    Nowhere,
    Room(RoomId),
    CarriedBy(CharacterId),
    OnFeature(FeatureId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemPlacement {
    pub item_id: ItemId,
    pub location: ItemLocation,
}
