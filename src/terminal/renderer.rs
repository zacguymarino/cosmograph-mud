use crate::game::command::TargetKind;
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
            features,
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

            let feature_descriptions: Vec<&str> = features
                .iter()
                .map(|feature| feature.room_description.as_str())
                .collect();

            let feature_paragraph = if feature_descriptions.is_empty() {
                String::new()
            } else {
                format!("\n{}", feature_descriptions.join("\n"))
            };

            format!(
                "{name}\n{description}{feature_paragraph}\nItems: {items_text}\nExits: {exits_text}"
            )
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
            reason: TakeFailureReason::Ambiguous { match_count },
            ..
        } => {
            let choices = (1..=*match_count)
                .map(|number| format!("{number}. {query} (item)"))
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "More than one item matches '{query}':\n{choices}\nTry 'take 1 {query}' or another listed number."
            )
        }

        GameEvent::TakeFailed {
            query,
            reason:
                TakeFailureReason::OrdinalOutOfRange {
                    requested,
                    available,
                },
            ..
        } => {
            format!("There is no #{requested} '{query}' here; {available} matched.")
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
            reason: DropFailureReason::Ambiguous { match_count },
            ..
        } => {
            let choices = (1..=*match_count)
                .map(|number| format!("{number}. {query} (item)"))
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "More than one carried item matches '{query}':\n{choices}\nTry 'drop 1 {query}' or another listed number."
            )
        }

        GameEvent::DropFailed {
            query,
            reason:
                DropFailureReason::OrdinalOutOfRange {
                    requested,
                    available,
                },
            ..
        } => {
            format!("You are not carrying a #{requested} '{query}'; {available} matched.")
        }

        GameEvent::ItemExamined { item, .. } => {
            format!("{}\n{}", item.name, item.description)
        }

        GameEvent::FeatureExamined { feature, .. } => {
            format!("{}\n{}", feature.name, feature.description)
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
            reason: ExamineFailureReason::Ambiguous { kinds },
            ..
        } => {
            let choices = kinds
                .iter()
                .enumerate()
                .map(|(index, kind)| {
                    let kind_name = match kind {
                        TargetKind::Item => "item",
                        TargetKind::Feature => "feature",
                    };
                    format!("{}. {query} ({kind_name})", index + 1)
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "More than one accessible target matches '{query}':\n{choices}\nTry 'examine 1 {query}', 'examine item {query}', or 'examine feature {query}'."
            )
        }

        GameEvent::ExamineFailed {
            query,
            reason:
                ExamineFailureReason::OrdinalOutOfRange {
                    requested,
                    available,
                },
            ..
        } => {
            format!("There is no accessible #{requested} '{query}'; {available} matched.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::event::{ExaminedFeature, ExaminedItem, ObservedFeature, ObservedItem};
    use crate::game::ids::{CharacterId, FeatureId, ItemId, RoomId};

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
            features: vec![ObservedFeature {
                id: FeatureId("test_feature".to_string()),
                name: "Test Feature".to_string(),
                room_description: "A test feature stands here.".to_string(),
            }],
            exits: vec![Direction::West, Direction::North, Direction::East],
        };

        assert_eq!(
            render_event(&event),
            "Starting Room\nA test room.\nA test feature stands here.\nItems: Test Item\nExits: east, north, west"
        );
    }

    #[test]
    fn room_without_features_omits_feature_paragraph() {
        let event = GameEvent::RoomObserved {
            room_id: RoomId("start".to_string()),
            name: "Starting Room".to_string(),
            description: "A test room.".to_string(),
            items: vec![],
            features: vec![],
            exits: vec![],
        };

        assert_eq!(
            render_event(&event),
            "Starting Room\nA test room.\nItems: none\nExits: none"
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
            reason: TakeFailureReason::Ambiguous { match_count: 2 },
        };

        assert_eq!(
            render_event(&event),
            "More than one item matches 'key':\n1. key (item)\n2. key (item)\nTry 'take 1 key' or another listed number."
        );
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
            reason: DropFailureReason::Ambiguous { match_count: 2 },
        };

        assert_eq!(
            render_event(&event),
            "More than one carried item matches 'key':\n1. key (item)\n2. key (item)\nTry 'drop 1 key' or another listed number."
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
    fn examined_feature_renders_description() {
        let event = GameEvent::FeatureExamined {
            character_id: CharacterId("player".to_string()),
            feature: ExaminedFeature {
                id: FeatureId("founders_plaque".to_string()),
                name: "Founder's Plaque".to_string(),
                description: "A weathered bronze plaque.".to_string(),
            },
        };

        assert_eq!(
            render_event(&event),
            "Founder's Plaque\nA weathered bronze plaque."
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
            reason: ExamineFailureReason::Ambiguous {
                kinds: vec![TargetKind::Item, TargetKind::Feature],
            },
        };

        assert_eq!(
            render_event(&event),
            "More than one accessible target matches 'key':\n1. key (item)\n2. key (feature)\nTry 'examine 1 key', 'examine item key', or 'examine feature key'."
        );
    }
}
