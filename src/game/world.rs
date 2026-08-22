use std::collections::HashMap;

use super::ids::{ItemId, RoomId, WorldId};
use super::item::Item;
use super::room::Room;

#[derive(Debug)]
pub struct World {
    pub id: WorldId,
    pub name: String,
    pub starting_room: RoomId,
    pub items: HashMap<ItemId, Item>,
    pub rooms: HashMap<RoomId, Room>,
}

impl World {
    pub fn room(&self, id: &RoomId) -> Option<&Room> {
        self.rooms.get(id)
    }

    pub fn room_mut(&mut self, id: &RoomId) -> Option<&mut Room> {
        self.rooms.get_mut(id)
    }

    pub fn item(&self, id: &ItemId) -> Option<&Item> {
        self.items.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn valid_room() -> Room {
        Room {
            id: RoomId("start".to_string()),
            name: "Starting Room".to_string(),
            description: "A test room.".to_string(),
            items: vec![],
            exits: HashMap::new(),
        }
    }

    fn valid_world() -> World {
        let mut rooms = HashMap::new();
        let room = valid_room();
        rooms.insert(room.id.clone(), room);

        let item = Item {
            id: ItemId("test_item".to_string()),
            name: "Test Item".to_string(),
            description: "An item used for testing.".to_string(),
        };
        let mut items = HashMap::new();
        items.insert(item.id.clone(), item);

        World {
            id: WorldId("test_world".to_string()),
            name: "Test World".to_string(),
            starting_room: RoomId("start".to_string()),
            items,
            rooms,
        }
    }

    #[test]
    fn existing_room_returns_borrowed_room() {
        let world = valid_world();
        let room = world
            .room(&RoomId("start".to_string()))
            .expect("starting room should exist");

        assert_eq!(room.name, "Starting Room");
    }

    #[test]
    fn missing_room_returns_none() {
        let world = valid_world();
        assert!(world.room(&RoomId("missing".to_string())).is_none());
    }

    #[test]
    fn existing_item_returns_borrowed_item() {
        let world = valid_world();

        let item = world
            .item(&ItemId("test_item".to_string()))
            .expect("Item should exist");

        assert_eq!(item.name, "Test Item");
    }

    #[test]
    fn missing_item_returns_none() {
        let world = valid_world();

        assert!(world.item(&ItemId("missing".to_string())).is_none());
    }

    #[test]
    fn existing_room_can_be_mutated() {
        let mut world = valid_world();

        let room = world
            .room_mut(&RoomId("start".to_string()))
            .expect("starting room should exist");

        room.items.push(ItemId("test_item".to_string()));

        assert_eq!(
            world
                .room(&RoomId("start".to_string()))
                .expect("starting room should still exist")
                .items,
            vec![ItemId("test_item".to_string())]
        );
    }
}
