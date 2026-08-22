use super::ids::{CharacterId, ItemId, RoomId};

#[derive(Debug)]
pub struct Character {
    pub id: CharacterId,
    pub name: String,
    pub current_room: RoomId,
    pub inventory: Vec<ItemId>,
}
