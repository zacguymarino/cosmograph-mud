use super::character::Character;
use super::command::Command;
use super::direction::Direction;
use super::event::{
    DropFailureReason, ExamineFailureReason, ExaminedItem, GameEvent, ObservedItem,
    TakeFailureReason,
};
use super::ids::ItemId;
use super::item::normalize_item_name;
use super::world::World;

#[derive(Debug)]
pub struct Game {
    pub world: World,
    pub character: Character,
}

impl Game {
    pub fn new(world: World, character: Character) -> Self {
        Self { world, character }
    }

    fn observe_current_room(&self) -> Option<GameEvent> {
        let room = self.world.room(&self.character.current_room)?;

        let items = room
            .items
            .iter()
            .map(|item_id| {
                let item = self.world.item(item_id)?;

                Some(ObservedItem {
                    id: item.id.clone(),
                    name: item.name.clone(),
                })
            })
            .collect::<Option<Vec<_>>>()?;

        Some(GameEvent::RoomObserved {
            room_id: room.id.clone(),
            name: room.name.clone(),
            description: room.description.clone(),
            items,
            exits: room.exits.keys().cloned().collect(),
        })
    }

    fn attempt_move(&mut self, direction: Direction) -> Vec<GameEvent> {
        let current_room = self.character.current_room.clone();
        let Some(room) = self.world.room(&current_room) else {
            return vec![GameEvent::MovementFailed {
                character_id: self.character.id.clone(),
                room_id: current_room,
                direction,
            }];
        };

        let destination = room.exits.get(&direction).cloned();
        let Some(destination) = destination else {
            return vec![GameEvent::MovementFailed {
                character_id: self.character.id.clone(),
                room_id: current_room,
                direction,
            }];
        };

        self.character.current_room = destination.clone();

        vec![GameEvent::CharacterMoved {
            character_id: self.character.id.clone(),
            from: current_room,
            to: destination,
            direction,
        }]
    }

    fn matching_room_items(&self, query: &str) -> Vec<ItemId> {
        let Some(room) = self.world.room(&self.character.current_room) else {
            return vec![];
        };

        let normalized_query = normalize_item_name(query);

        room.items
            .iter()
            .filter(|item_id| {
                self.world
                    .item(item_id)
                    .is_some_and(|item| normalize_item_name(&item.name) == normalized_query)
            })
            .cloned()
            .collect()
    }

    fn attempt_take(&mut self, query: String) -> Vec<GameEvent> {
        let query = normalize_item_name(&query);
        let room_id = self.character.current_room.clone();
        let matches = self.matching_room_items(&query);

        let item_id = match matches.as_slice() {
            [] => {
                return vec![GameEvent::TakeFailed {
                    character_id: self.character.id.clone(),
                    room_id,
                    query,
                    reason: TakeFailureReason::NotFound,
                }];
            }

            [item_id] => item_id.clone(),

            _ => {
                return vec![GameEvent::TakeFailed {
                    character_id: self.character.id.clone(),
                    room_id,
                    query,
                    reason: TakeFailureReason::Ambiguous,
                }];
            }
        };

        let Some(item) = self.world.item(&item_id) else {
            return vec![GameEvent::TakeFailed {
                character_id: self.character.id.clone(),
                room_id,
                query,
                reason: TakeFailureReason::NotFound,
            }];
        };

        let observed_item = ObservedItem {
            id: item.id.clone(),
            name: item.name.clone(),
        };

        let Some(room) = self.world.room_mut(&room_id) else {
            return vec![GameEvent::TakeFailed {
                character_id: self.character.id.clone(),
                room_id,
                query,
                reason: TakeFailureReason::NotFound,
            }];
        };

        let Some(position) = room
            .items
            .iter()
            .position(|candidate| candidate == &item_id)
        else {
            return vec![GameEvent::TakeFailed {
                character_id: self.character.id.clone(),
                room_id,
                query,
                reason: TakeFailureReason::NotFound,
            }];
        };

        room.items.remove(position);
        self.character.inventory.push(item_id);

        vec![GameEvent::ItemTaken {
            character_id: self.character.id.clone(),
            room_id,
            item: observed_item,
        }]
    }

    fn observe_inventory(&self) -> Option<GameEvent> {
        let items = self
            .character
            .inventory
            .iter()
            .map(|item_id| {
                let item = self.world.item(item_id)?;

                Some(ObservedItem {
                    id: item.id.clone(),
                    name: item.name.clone(),
                })
            })
            .collect::<Option<Vec<_>>>()?;

        Some(GameEvent::InventoryObserved { items })
    }

    fn matching_inventory_items(&self, query: &str) -> Vec<ItemId> {
        let normalized_query = normalize_item_name(query);

        self.character
            .inventory
            .iter()
            .filter(|item_id| {
                self.world
                    .item(item_id)
                    .is_some_and(|item| normalize_item_name(&item.name) == normalized_query)
            })
            .cloned()
            .collect()
    }

    fn attempt_drop(&mut self, query: String) -> Vec<GameEvent> {
        let query = normalize_item_name(&query);
        let room_id = self.character.current_room.clone();
        let matches = self.matching_inventory_items(&query);

        let item_id = match matches.as_slice() {
            [] => {
                return vec![GameEvent::DropFailed {
                    character_id: self.character.id.clone(),
                    room_id,
                    query,
                    reason: DropFailureReason::NotFound,
                }];
            }

            [item_id] => item_id.clone(),

            _ => {
                return vec![GameEvent::DropFailed {
                    character_id: self.character.id.clone(),
                    room_id,
                    query,
                    reason: DropFailureReason::Ambiguous,
                }];
            }
        };

        let Some(item) = self.world.item(&item_id) else {
            return vec![GameEvent::DropFailed {
                character_id: self.character.id.clone(),
                room_id,
                query,
                reason: DropFailureReason::NotFound,
            }];
        };

        let observed_item = ObservedItem {
            id: item.id.clone(),
            name: item.name.clone(),
        };

        let Some(position) = self
            .character
            .inventory
            .iter()
            .position(|candidate| candidate == &item_id)
        else {
            return vec![GameEvent::DropFailed {
                character_id: self.character.id.clone(),
                room_id,
                query,
                reason: DropFailureReason::NotFound,
            }];
        };

        let Some(room) = self.world.room_mut(&room_id) else {
            return vec![GameEvent::DropFailed {
                character_id: self.character.id.clone(),
                room_id,
                query,
                reason: DropFailureReason::NotFound,
            }];
        };

        let dropped_item_id = self.character.inventory.remove(position);
        room.items.push(dropped_item_id);

        vec![GameEvent::ItemDropped {
            character_id: self.character.id.clone(),
            room_id,
            item: observed_item,
        }]
    }

    fn matching_accessible_items(&self, query: &str) -> Vec<ItemId> {
        let mut matches = self.matching_room_items(query);
        matches.extend(self.matching_inventory_items(query));
        matches
    }

    fn attempt_examine(&self, query: String) -> Vec<GameEvent> {
        let query = normalize_item_name(&query);
        let matches = self.matching_accessible_items(&query);

        let item_id = match matches.as_slice() {
            [] => {
                return vec![GameEvent::ExamineFailed {
                    character_id: self.character.id.clone(),
                    query,
                    reason: ExamineFailureReason::NotFound,
                }];
            }

            [item_id] => item_id,

            _ => {
                return vec![GameEvent::ExamineFailed {
                    character_id: self.character.id.clone(),
                    query,
                    reason: ExamineFailureReason::Ambiguous,
                }];
            }
        };

        let Some(item) = self.world.item(item_id) else {
            return vec![GameEvent::ExamineFailed {
                character_id: self.character.id.clone(),
                query,
                reason: ExamineFailureReason::NotFound,
            }];
        };

        vec![GameEvent::ItemExamined {
            character_id: self.character.id.clone(),
            item: ExaminedItem {
                id: item.id.clone(),
                name: item.name.clone(),
                description: item.description.clone(),
            },
        }]
    }

    pub fn process(&mut self, command: Command) -> Vec<GameEvent> {
        match command {
            Command::Look => self.observe_current_room().into_iter().collect(),
            Command::Move(direction) => self.attempt_move(direction),
            Command::Take(query) => self.attempt_take(query),
            Command::Inventory => self.observe_inventory().into_iter().collect(),
            Command::Drop(query) => self.attempt_drop(query),
            Command::Examine(query) => self.attempt_examine(query),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::ids::{CharacterId, ItemId, RoomId, WorldId};
    use crate::game::item::Item;
    use crate::game::room::Room;
    use std::collections::HashMap;

    fn game_with_character_in(current_room: &str) -> Game {
        let mut start_exits = HashMap::new();
        start_exits.insert(Direction::North, RoomId("next_room".to_string()));

        let room = Room {
            id: RoomId("start".to_string()),
            name: "Starting Room".to_string(),
            description: "A test room.".to_string(),
            items: vec![ItemId("test_item".to_string())],
            exits: start_exits,
        };

        let next_room = Room {
            id: RoomId("next_room".to_string()),
            name: "Next Room".to_string(),
            description: "Another test room.".to_string(),
            items: vec![],
            exits: HashMap::new(),
        };

        let mut rooms = HashMap::new();
        rooms.insert(room.id.clone(), room);
        rooms.insert(next_room.id.clone(), next_room);

        let item = Item {
            id: ItemId("test_item".to_string()),
            name: "Test Item".to_string(),
            description: "An item used for testing.".to_string(),
        };

        let mut items = HashMap::new();
        items.insert(item.id.clone(), item);

        let world = World {
            id: WorldId("test_world".to_string()),
            name: "Test World".to_string(),
            starting_room: RoomId("start".to_string()),
            items,
            rooms,
        };

        let character = Character {
            id: CharacterId("player".to_string()),
            name: "Player".to_string(),
            current_room: RoomId(current_room.to_string()),
            inventory: vec![],
        };

        Game::new(world, character)
    }

    #[test]
    fn observing_existing_room_returns_event() {
        let game = game_with_character_in("start");

        assert_eq!(
            game.observe_current_room(),
            Some(GameEvent::RoomObserved {
                room_id: RoomId("start".to_string()),
                name: "Starting Room".to_string(),
                description: "A test room.".to_string(),
                items: vec![ObservedItem {
                    id: ItemId("test_item".to_string()),
                    name: "Test Item".to_string(),
                }],
                exits: vec![Direction::North],
            })
        )
    }

    #[test]
    fn observing_missing_room_returns_none() {
        let game = game_with_character_in("missing");

        assert_eq!(game.observe_current_room(), None);
    }

    #[test]
    fn movement_through_existing_exit_succeeds() {
        let mut game = game_with_character_in("start");

        let events = game.attempt_move(Direction::North);

        assert_eq!(
            events,
            vec![GameEvent::CharacterMoved {
                character_id: CharacterId("player".to_string()),
                from: RoomId("start".to_string()),
                to: RoomId("next_room".to_string()),
                direction: Direction::North,
            }]
        );

        assert_eq!(game.character.current_room, RoomId("next_room".to_string()));
    }

    #[test]
    fn movement_without_exit_fails_without_changing_room() {
        let mut game = game_with_character_in("start");

        let events = game.attempt_move(Direction::East);

        assert_eq!(
            events,
            vec![GameEvent::MovementFailed {
                character_id: CharacterId("player".to_string()),
                room_id: RoomId("start".to_string()),
                direction: Direction::East,
            }]
        );

        assert_eq!(game.character.current_room, RoomId("start".to_string()));
    }

    #[test]
    fn look_command_observes_current_room() {
        let mut game = game_with_character_in("start");

        let events = game.process(Command::Look);

        assert!(matches!(
            events.as_slice(),
            [GameEvent::RoomObserved { .. }]
        ));
    }

    #[test]
    fn move_command_attempts_movement() {
        let mut game = game_with_character_in("start");

        let events = game.process(Command::Move(Direction::North));

        assert!(matches!(
            events.as_slice(),
            [GameEvent::CharacterMoved { .. }]
        ));

        assert_eq!(game.character.current_room, RoomId("next_room".to_string()));
    }

    #[test]
    fn room_items_can_be_found_by_normalized_name() {
        let game = game_with_character_in("start");

        assert_eq!(
            game.matching_room_items("  TEST   item "),
            vec![ItemId("test_item".to_string())]
        );
    }

    #[test]
    fn taking_unique_item_moves_it_to_inventory() {
        let mut game = game_with_character_in("start");

        let events = game.attempt_take("  TEST   item ".to_string());

        assert_eq!(
            events,
            vec![GameEvent::ItemTaken {
                character_id: CharacterId("player".to_string()),
                room_id: RoomId("start".to_string()),
                item: ObservedItem {
                    id: ItemId("test_item".to_string()),
                    name: "Test Item".to_string(),
                },
            }]
        );

        assert_eq!(
            game.character.inventory,
            vec![ItemId("test_item".to_string())]
        );

        assert!(
            game.world
                .room(&RoomId("start".to_string()))
                .expect("starting room should exist")
                .items
                .is_empty()
        );
    }

    #[test]
    fn taking_missing_item_does_not_change_state() {
        let mut game = game_with_character_in("start");

        let events = game.attempt_take("missing item".to_string());

        assert_eq!(
            events,
            vec![GameEvent::TakeFailed {
                character_id: CharacterId("player".to_string()),
                room_id: RoomId("start".to_string()),
                query: "missing item".to_string(),
                reason: TakeFailureReason::NotFound,
            }]
        );

        assert!(game.character.inventory.is_empty());

        assert_eq!(
            game.world
                .room(&RoomId("start".to_string()))
                .expect("starting room should exist")
                .items,
            vec![ItemId("test_item".to_string())]
        );
    }

    #[test]
    fn taking_ambiguous_item_does_not_change_state() {
        let mut game = game_with_character_in("start");

        let second_item = Item {
            id: ItemId("second_item".to_string()),
            name: "Test Item".to_string(),
            description: "Another item with the same name.".to_string(),
        };

        game.world.items.insert(second_item.id.clone(), second_item);

        game.world
            .room_mut(&RoomId("start".to_string()))
            .expect("starting room should exist")
            .items
            .push(ItemId("second_item".to_string()));

        let events = game.attempt_take("test item".to_string());

        assert_eq!(
            events,
            vec![GameEvent::TakeFailed {
                character_id: CharacterId("player".to_string()),
                room_id: RoomId("start".to_string()),
                query: "test item".to_string(),
                reason: TakeFailureReason::Ambiguous,
            }]
        );

        assert!(game.character.inventory.is_empty());

        assert_eq!(
            game.world
                .room(&RoomId("start".to_string()))
                .expect("starting room should exist")
                .items
                .len(),
            2
        );
    }

    #[test]
    fn take_command_attempts_to_take_item() {
        let mut game = game_with_character_in("start");

        let events = game.process(Command::Take("test item".to_string()));

        assert!(matches!(events.as_slice(), [GameEvent::ItemTaken { .. }]));

        assert_eq!(
            game.character.inventory,
            vec![ItemId("test_item".to_string())]
        );
    }

    #[test]
    fn inventory_observation_reports_carried_items() {
        let mut game = game_with_character_in("start");

        game.attempt_take("test item".to_string());

        assert_eq!(
            game.observe_inventory(),
            Some(GameEvent::InventoryObserved {
                items: vec![ObservedItem {
                    id: ItemId("test_item".to_string()),
                    name: "Test Item".to_string(),
                }],
            })
        );
    }

    #[test]
    fn inventory_command_observes_carried_items() {
        let mut game = game_with_character_in("start");

        game.attempt_take("test item".to_string());

        let events = game.process(Command::Inventory);

        assert_eq!(
            events,
            vec![GameEvent::InventoryObserved {
                items: vec![ObservedItem {
                    id: ItemId("test_item".to_string()),
                    name: "Test Item".to_string(),
                }],
            }]
        );
    }

    #[test]
    fn inventory_items_can_be_found_by_normalized_name() {
        let mut game = game_with_character_in("start");

        game.attempt_take("test item".to_string());

        assert_eq!(
            game.matching_inventory_items("  TEST   item "),
            vec![ItemId("test_item".to_string())]
        );
    }

    #[test]
    fn dropping_unique_item_moves_it_to_room() {
        let mut game = game_with_character_in("start");
        game.attempt_take("test item".to_string());

        let events = game.attempt_drop("  TEST   item ".to_string());

        assert_eq!(
            events,
            vec![GameEvent::ItemDropped {
                character_id: CharacterId("player".to_string()),
                room_id: RoomId("start".to_string()),
                item: ObservedItem {
                    id: ItemId("test_item".to_string()),
                    name: "Test Item".to_string(),
                },
            }]
        );

        assert!(game.character.inventory.is_empty());

        assert_eq!(
            game.world
                .room(&RoomId("start".to_string()))
                .expect("starting room should exist")
                .items,
            vec![ItemId("test_item".to_string())]
        );
    }

    #[test]
    fn dropping_missing_item_does_not_change_state() {
        let mut game = game_with_character_in("start");

        let events = game.attempt_drop("missing item".to_string());

        assert_eq!(
            events,
            vec![GameEvent::DropFailed {
                character_id: CharacterId("player".to_string()),
                room_id: RoomId("start".to_string()),
                query: "missing item".to_string(),
                reason: DropFailureReason::NotFound,
            }]
        );

        assert!(game.character.inventory.is_empty());

        assert_eq!(
            game.world
                .room(&RoomId("start".to_string()))
                .expect("starting room should exist")
                .items,
            vec![ItemId("test_item".to_string())]
        );
    }

    #[test]
    fn dropping_ambiguous_item_does_not_change_state() {
        let mut game = game_with_character_in("start");
        game.attempt_take("test item".to_string());

        let second_item = Item {
            id: ItemId("second_item".to_string()),
            name: "Test Item".to_string(),
            description: "Another item with the same name.".to_string(),
        };

        game.world.items.insert(second_item.id.clone(), second_item);

        game.character
            .inventory
            .push(ItemId("second_item".to_string()));

        let events = game.attempt_drop("test item".to_string());

        assert_eq!(
            events,
            vec![GameEvent::DropFailed {
                character_id: CharacterId("player".to_string()),
                room_id: RoomId("start".to_string()),
                query: "test item".to_string(),
                reason: DropFailureReason::Ambiguous,
            }]
        );

        assert_eq!(game.character.inventory.len(), 2);

        assert!(
            game.world
                .room(&RoomId("start".to_string()))
                .expect("starting room should exist")
                .items
                .is_empty()
        );
    }

    #[test]
    fn drop_command_attempts_to_drop_item() {
        let mut game = game_with_character_in("start");
        game.attempt_take("test item".to_string());

        let events = game.process(Command::Drop("test item".to_string()));

        assert!(matches!(events.as_slice(), [GameEvent::ItemDropped { .. }]));

        assert!(game.character.inventory.is_empty());

        assert_eq!(
            game.world
                .room(&RoomId("start".to_string()))
                .expect("starting room should exist")
                .items,
            vec![ItemId("test_item".to_string())]
        );
    }

    #[test]
    fn accessible_items_include_room_and_inventory_items() {
        let mut game = game_with_character_in("start");

        assert_eq!(
            game.matching_accessible_items("test item"),
            vec![ItemId("test_item".to_string())]
        );

        game.attempt_take("test item".to_string());

        assert_eq!(
            game.matching_accessible_items("test item"),
            vec![ItemId("test_item".to_string())]
        );
    }

    #[test]
    fn examining_room_item_returns_description() {
        let game = game_with_character_in("start");

        assert_eq!(
            game.attempt_examine("  TEST   item ".to_string()),
            vec![GameEvent::ItemExamined {
                character_id: CharacterId("player".to_string()),
                item: ExaminedItem {
                    id: ItemId("test_item".to_string()),
                    name: "Test Item".to_string(),
                    description: "An item used for testing.".to_string(),
                },
            }]
        );
    }

    #[test]
    fn examining_inventory_item_returns_description() {
        let mut game = game_with_character_in("start");
        game.attempt_take("test item".to_string());

        assert_eq!(
            game.attempt_examine("test item".to_string()),
            vec![GameEvent::ItemExamined {
                character_id: CharacterId("player".to_string()),
                item: ExaminedItem {
                    id: ItemId("test_item".to_string()),
                    name: "Test Item".to_string(),
                    description: "An item used for testing.".to_string(),
                },
            }]
        );
    }

    #[test]
    fn examining_missing_item_fails() {
        let game = game_with_character_in("start");

        assert_eq!(
            game.attempt_examine("missing item".to_string()),
            vec![GameEvent::ExamineFailed {
                character_id: CharacterId("player".to_string()),
                query: "missing item".to_string(),
                reason: ExamineFailureReason::NotFound,
            }]
        );
    }

    #[test]
    fn examining_ambiguous_item_fails() {
        let mut game = game_with_character_in("start");

        let second_item = Item {
            id: ItemId("second_item".to_string()),
            name: "Test Item".to_string(),
            description: "Another item with the same name.".to_string(),
        };

        game.world.items.insert(second_item.id.clone(), second_item);

        game.character
            .inventory
            .push(ItemId("second_item".to_string()));

        assert_eq!(
            game.attempt_examine("test item".to_string()),
            vec![GameEvent::ExamineFailed {
                character_id: CharacterId("player".to_string()),
                query: "test item".to_string(),
                reason: ExamineFailureReason::Ambiguous,
            }]
        );
    }

    #[test]
    fn examine_command_attempts_examination() {
        let mut game = game_with_character_in("start");

        let events = game.process(Command::Examine("test item".to_string()));

        assert!(matches!(
            events.as_slice(),
            [GameEvent::ItemExamined { .. }]
        ));
    }
}
