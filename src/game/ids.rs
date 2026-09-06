#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorldId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoomId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CharacterId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FeatureId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NpcId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NpcTopicId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FactId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FactKey {
    pub world_id: WorldId,
    pub fact_id: FactId,
}
