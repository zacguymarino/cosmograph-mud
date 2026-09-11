use super::ids::{FeatureId, ItemId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Supporter {
    pub capacity: Option<usize>,
    pub accepts_items: Vec<ItemId>,
    pub rejection_message: Option<String>,
    pub full_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RoomFeature {
    pub id: FeatureId,
    pub name: String,
    pub room_description: String,
    pub description: String,
    pub supporter: Option<Supporter>,
}
