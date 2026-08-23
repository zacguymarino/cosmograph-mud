use super::ids::ItemId;

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
    pub description: String,
}
