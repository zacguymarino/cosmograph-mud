use super::ids::FeatureId;

#[derive(Debug, Clone)]
pub struct RoomFeature {
    pub id: FeatureId,
    pub name: String,
    pub room_description: String,
    pub description: String,
}
