use crate::game::direction::Direction;
use crate::game::event::{DropFailureReason, ExamineFailureReason, GameEvent, TakeFailureReason};

fn direction_name(direction: &Direction) -> &'static str {
    match direction {
        Direction::North => "north",
        Direction::South => "south",
        Direction::East => "east",
        Direction::West => "west",
    }
}

pub fn render_event(event: &GameEvent) -> String {
    match event {
        GameEvent::RoomObserved {
            name,
            description,
            items,
            exits,
            ..
        } => {
            let mut exit_names: Vec<&str> = exits.iter().map(direction_name).collect();
            exit_names.sort();

            let exits_text = if exit_names.is_empty() {
                "none".to_string()
            } else {
                exit_names.join(", ")
            };

            let mut item_names: Vec<&str> = items.iter().map(|item| item.name.as_str()).collect();

            item_names.sort();

            let items_text = if item_names.is_empty() {
                "none".to_string()
            } else {
                item_names.join(", ")
            };

            format!("{name}\n{description}\nItems: {items_text}\nExits: {exits_text}")
        }

        GameEvent::CharacterMoved { direction, .. } => {
            format!("You move {}.", direction_name(direction))
        }

        GameEvent::MovementFailed { direction, .. } => {
            format!("You cannot go {}.", direction_name(direction))
        }

        GameEvent::ItemTaken { item, .. } => {
            format!("You take {}.", item.name)
        }

        GameEvent::TakeFailed {
            query,
            reason: TakeFailureReason::NotFound,
            ..
        } => {
            format!("You do not see '{query}' here.")
        }

        GameEvent::TakeFailed {
            query,
            reason: TakeFailureReason::Ambiguous,
            ..
        } => {
            format!("More than one item matches '{query}'.")
        }

        GameEvent::InventoryObserved { items } => {
            let mut item_names: Vec<&str> = items.iter().map(|item| item.name.as_str()).collect();

            item_names.sort();

            let items_text = if item_names.is_empty() {
                "none".to_string()
            } else {
                item_names.join(", ")
            };

            format!("Inventory: {items_text}")
        }

        GameEvent::ItemDropped { item, .. } => {
            format!("You drop {}.", item.name)
        }

        GameEvent::DropFailed {
            query,
            reason: DropFailureReason::NotFound,
            ..
        } => {
            format!("You are not carrying '{query}'.")
        }

        GameEvent::DropFailed {
            query,
            reason: DropFailureReason::Ambiguous,
            ..
        } => {
            format!("More than one carried item matches '{query}'.")
        }

        GameEvent::ItemExamined { item, .. } => {
            format!("{}\n{}", item.name, item.description)
        }

        GameEvent::ExamineFailed {
            query,
            reason: ExamineFailureReason::NotFound,
            ..
        } => {
            format!("You cannot find '{query}' to examine.")
        }

        GameEvent::ExamineFailed {
            query,
            reason: ExamineFailureReason::Ambiguous,
            ..
        } => {
            format!("More than one accessible item matches '{query}'.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::event::{ExaminedItem, ObservedItem};
    use crate::game::ids::{CharacterId, ItemId, RoomId};

    #[test]
    fn room_observation_renders_room_details() {
        let event = GameEvent::RoomObserved {
            room_id: RoomId("start".to_string()),
            name: "Starting Room".to_string(),
            description: "A test room.".to_string(),
            items: vec![ObservedItem {
                id: ItemId("test_item".to_string()),
                name: "Test Item".to_string(),
            }],
            exits: vec![Direction::West, Direction::North, Direction::East],
        };

        assert_eq!(
            render_event(&event),
            "Starting Room\nA test room.\nItems: Test Item\nExits: east, north, west"
        );
    }

    #[test]
    fn successful_movement_renders_message() {
        let event = GameEvent::CharacterMoved {
            character_id: CharacterId("player".to_string()),
            from: RoomId("start".to_string()),
            to: RoomId("next_room".to_string()),
            direction: Direction::North,
        };

        assert_eq!(render_event(&event), "You move north.");
    }

    #[test]
    fn failed_movement_renders_message() {
        let event = GameEvent::MovementFailed {
            character_id: CharacterId("player".to_string()),
            room_id: RoomId("start".to_string()),
            direction: Direction::East,
        };

        assert_eq!(render_event(&event), "You cannot go east.");
    }

    #[test]
    fn taken_item_renders_message() {
        let event = GameEvent::ItemTaken {
            character_id: CharacterId("player".to_string()),
            room_id: RoomId("start".to_string()),
            item: ObservedItem {
                id: ItemId("rusty_key".to_string()),
                name: "Rusty Key".to_string(),
            },
        };

        assert_eq!(render_event(&event), "You take Rusty Key.");
    }

    #[test]
    fn missing_take_target_renders_message() {
        let event = GameEvent::TakeFailed {
            character_id: CharacterId("player".to_string()),
            room_id: RoomId("start".to_string()),
            query: "silver key".to_string(),
            reason: TakeFailureReason::NotFound,
        };

        assert_eq!(render_event(&event), "You do not see 'silver key' here.");
    }

    #[test]
    fn ambiguous_take_target_renders_message() {
        let event = GameEvent::TakeFailed {
            character_id: CharacterId("player".to_string()),
            room_id: RoomId("start".to_string()),
            query: "key".to_string(),
            reason: TakeFailureReason::Ambiguous,
        };

        assert_eq!(render_event(&event), "More than one item matches 'key'.");
    }

    #[test]
    fn inventory_renders_item_names() {
        let event = GameEvent::InventoryObserved {
            items: vec![ObservedItem {
                id: ItemId("rusty_key".to_string()),
                name: "Rusty Key".to_string(),
            }],
        };

        assert_eq!(render_event(&event), "Inventory: Rusty Key");
    }

    #[test]
    fn dropped_item_renders_message() {
        let event = GameEvent::ItemDropped {
            character_id: CharacterId("player".to_string()),
            room_id: RoomId("start".to_string()),
            item: ObservedItem {
                id: ItemId("rusty_key".to_string()),
                name: "Rusty Key".to_string(),
            },
        };

        assert_eq!(render_event(&event), "You drop Rusty Key.");
    }

    #[test]
    fn missing_drop_target_renders_message() {
        let event = GameEvent::DropFailed {
            character_id: CharacterId("player".to_string()),
            room_id: RoomId("start".to_string()),
            query: "silver key".to_string(),
            reason: DropFailureReason::NotFound,
        };

        assert_eq!(render_event(&event), "You are not carrying 'silver key'.");
    }

    #[test]
    fn ambiguous_drop_target_renders_message() {
        let event = GameEvent::DropFailed {
            character_id: CharacterId("player".to_string()),
            room_id: RoomId("start".to_string()),
            query: "key".to_string(),
            reason: DropFailureReason::Ambiguous,
        };

        assert_eq!(
            render_event(&event),
            "More than one carried item matches 'key'."
        );
    }

    #[test]
    fn examined_item_renders_description() {
        let event = GameEvent::ItemExamined {
            character_id: CharacterId("player".to_string()),
            item: ExaminedItem {
                id: ItemId("rusty_key".to_string()),
                name: "Rusty Key".to_string(),
                description: "A small iron key covered with rust.".to_string(),
            },
        };

        assert_eq!(
            render_event(&event),
            "Rusty Key\nA small iron key covered with rust."
        );
    }

    #[test]
    fn missing_examine_target_renders_message() {
        let event = GameEvent::ExamineFailed {
            character_id: CharacterId("player".to_string()),
            query: "silver key".to_string(),
            reason: ExamineFailureReason::NotFound,
        };

        assert_eq!(
            render_event(&event),
            "You cannot find 'silver key' to examine."
        );
    }

    #[test]
    fn ambiguous_examine_target_renders_message() {
        let event = GameEvent::ExamineFailed {
            character_id: CharacterId("player".to_string()),
            query: "key".to_string(),
            reason: ExamineFailureReason::Ambiguous,
        };

        assert_eq!(
            render_event(&event),
            "More than one accessible item matches 'key'."
        );
    }
}
