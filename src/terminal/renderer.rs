use crate::game::command::TargetKind;
use crate::game::direction::Direction;
use crate::game::event::{
    AskFailureReason, DropFailureReason, ExamineFailureReason, GameEvent, ObservedQuest,
    PutOnFailureReason, TakeFailureReason, TalkFailureReason,
};

fn direction_name(direction: &Direction) -> &'static str {
    match direction {
        Direction::North => "north",
        Direction::South => "south",
        Direction::East => "east",
        Direction::West => "west",
    }
}

fn render_quest_entry(quest: &ObservedQuest) -> String {
    match &quest.current_objective {
        Some(objective) => format!(
            "- {}\n  {}\n  Current objective: {objective}",
            quest.name, quest.description
        ),
        None => format!("- {}\n  {}", quest.name, quest.description),
    }
}

pub fn render_event(event: &GameEvent) -> String {
    match event {
        GameEvent::RoomObserved {
            name,
            description,
            items,
            features,
            npcs,
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

            let feature_descriptions: Vec<String> = features
                .iter()
                .flat_map(|feature| {
                    let mut lines = vec![feature.room_description.clone()];
                    if !feature.items.is_empty() {
                        lines.push(format!(
                            "On {}: {}",
                            feature.name,
                            feature
                                .items
                                .iter()
                                .map(|item| item.name.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                    }
                    lines
                })
                .collect();

            let feature_paragraph = if feature_descriptions.is_empty() {
                String::new()
            } else {
                format!("\n{}", feature_descriptions.join("\n"))
            };

            let npc_descriptions: Vec<&str> = npcs
                .iter()
                .map(|npc| npc.room_description.as_str())
                .collect();
            let npc_paragraph = if npc_descriptions.is_empty() {
                String::new()
            } else {
                format!("\n{}", npc_descriptions.join("\n"))
            };

            format!(
                "{name}\n{description}{feature_paragraph}{npc_paragraph}\nItems: {items_text}\nExits: {exits_text}"
            )
        }

        GameEvent::ExitsObserved { exits, .. } => {
            let mut exit_names: Vec<&str> = exits.iter().map(direction_name).collect();
            exit_names.sort();

            if exit_names.is_empty() {
                "Exits: none".to_string()
            } else {
                format!("Exits: {}", exit_names.join(", "))
            }
        }

        GameEvent::HelpRequested => [
            "Commands:",
            "  look (l)",
            "  look [item|feature|npc] <target>",
            "  examine (x) [item|feature|npc] <target>",
            "  north (n), south (s), east (e), west (w)",
            "  go <direction>",
            "  exits",
            "  take (get) <item>",
            "  drop <item>",
            "  put <item> on <feature>",
            "  inventory (inv, i)",
            "  quests (journal)",
            "  talk to <npc>",
            "  ask <npc> about <topic>",
            "  help (commands)",
            "  quit (exit)",
        ]
        .join("\n"),

        GameEvent::CharacterMoved { direction, .. } => {
            format!("You move {}.", direction_name(direction))
        }

        GameEvent::MovementFailed { direction, .. } => {
            format!("You cannot go {}.", direction_name(direction))
        }

        GameEvent::MovementBlocked { message, .. } => message.clone(),

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

        GameEvent::QuestsObserved { active, completed } => {
            if active.is_empty() && completed.is_empty() {
                return "Quests: none".to_string();
            }

            let mut sections = Vec::new();
            if !active.is_empty() {
                let quests = active
                    .iter()
                    .map(render_quest_entry)
                    .collect::<Vec<_>>()
                    .join("\n");
                sections.push(format!("Active quests:\n{quests}"));
            }
            if !completed.is_empty() {
                let quests = completed
                    .iter()
                    .map(render_quest_entry)
                    .collect::<Vec<_>>()
                    .join("\n");
                sections.push(format!("Completed quests:\n{quests}"));
            }

            sections.join("\n")
        }

        GameEvent::QuestStarted { quest, .. } => {
            let mut rendered = format!("Quest started: {}\n{}", quest.name, quest.description);
            if let Some(objective) = &quest.current_objective {
                rendered.push_str(&format!("\nCurrent objective: {objective}"));
            }
            rendered
        }

        GameEvent::QuestAdvanced { quest, .. } => {
            let mut rendered = format!("Quest updated: {}", quest.name);
            if let Some(objective) = &quest.current_objective {
                rendered.push_str(&format!("\nCurrent objective: {objective}"));
            }
            rendered
        }

        GameEvent::QuestCompleted { quest, .. } => {
            format!("Quest completed: {}", quest.name)
        }

        GameEvent::ItemDropped { item, .. } => {
            format!("You drop {}.", item.name)
        }

        GameEvent::ItemPlacedOnFeature {
            item, feature_name, ..
        } => format!("You put {} on {}.", item.name, feature_name),

        GameEvent::PutOnFailed {
            item_query,
            reason: PutOnFailureReason::ItemNotFound,
            ..
        } => format!("You are not carrying '{item_query}'."),

        GameEvent::PutOnFailed {
            item_query,
            feature_query,
            reason: PutOnFailureReason::ItemAmbiguous { match_count },
            ..
        } => {
            let choices = (1..=*match_count)
                .map(|number| format!("{number}. {item_query} (item)"))
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "More than one carried item matches '{item_query}':\n{choices}\nTry 'put 1 {item_query} on {feature_query}' or another listed number."
            )
        }

        GameEvent::PutOnFailed {
            item_query,
            reason:
                PutOnFailureReason::ItemOrdinalOutOfRange {
                    requested,
                    available,
                },
            ..
        } => format!(
            "There is no #{requested} '{item_query}' in your inventory; {available} matched."
        ),

        GameEvent::PutOnFailed {
            feature_query,
            reason: PutOnFailureReason::FeatureNotFound,
            ..
        } => format!("You do not see '{feature_query}' here."),

        GameEvent::PutOnFailed {
            item_query,
            feature_query,
            reason: PutOnFailureReason::FeatureAmbiguous { match_count },
            ..
        } => {
            let choices = (1..=*match_count)
                .map(|number| format!("{number}. {feature_query} (feature)"))
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "More than one feature matches '{feature_query}':\n{choices}\nTry 'put {item_query} on 1 {feature_query}' or another listed number."
            )
        }

        GameEvent::PutOnFailed {
            feature_query,
            reason:
                PutOnFailureReason::FeatureOrdinalOutOfRange {
                    requested,
                    available,
                },
            ..
        } => format!("There is no #{requested} '{feature_query}' here; {available} matched."),

        GameEvent::PutOnFailed {
            reason: PutOnFailureReason::NotSupporter { feature_name },
            ..
        } => format!("You cannot put things on {feature_name}."),

        GameEvent::PutOnFailed {
            reason: PutOnFailureReason::Rejected { message } | PutOnFailureReason::Full { message },
            ..
        } => message.clone(),

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
            let mut rendered = format!("{}\n{}", feature.name, feature.description);
            if !feature.items.is_empty() {
                rendered.push_str(&format!(
                    "\nOn it: {}",
                    feature
                        .items
                        .iter()
                        .map(|item| item.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            rendered
        }

        GameEvent::NpcExamined { npc, .. } => {
            format!("{}\n{}", npc.name, npc.description)
        }

        GameEvent::NpcSpoke {
            name,
            greeting,
            topics,
            ..
        } => {
            let speech = format!("{name} says, \"{greeting}\"");
            if topics.is_empty() {
                speech
            } else {
                format!(
                    "{speech}\nYou could ask {name} about: {}.",
                    topics.join(", ")
                )
            }
        }

        GameEvent::TalkFailed {
            query,
            reason: TalkFailureReason::NotFound,
            ..
        } => format!("You do not see '{query}' here to talk to."),

        GameEvent::TalkFailed {
            query,
            reason: TalkFailureReason::Ambiguous { match_count },
            ..
        } => {
            let choices = (1..=*match_count)
                .map(|number| format!("{number}. {query} (npc)"))
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "More than one NPC matches '{query}':\n{choices}\nTry 'talk to 1 {query}' or another listed number."
            )
        }

        GameEvent::TalkFailed {
            query,
            reason:
                TalkFailureReason::OrdinalOutOfRange {
                    requested,
                    available,
                },
            ..
        } => format!("There is no #{requested} '{query}' here; {available} NPCs matched."),

        GameEvent::NpcAnswered {
            npc_name,
            response,
            updated_topics,
            ..
        } => {
            let answer = format!("{npc_name} says, \"{response}\"");
            match updated_topics {
                None => answer,
                Some(topics) if topics.is_empty() => {
                    format!("{answer}\nThere are no remaining discussion topics.")
                }
                Some(topics) => format!(
                    "{answer}\nYou could now ask {npc_name} about: {}.",
                    topics.join(", ")
                ),
            }
        }

        GameEvent::AskFailed {
            npc_query,
            reason: AskFailureReason::NpcNotFound,
            ..
        } => format!("You do not see '{npc_query}' here to ask."),

        GameEvent::AskFailed {
            npc_query,
            topic_query,
            reason: AskFailureReason::NpcAmbiguous { match_count },
            ..
        } => {
            let choices = (1..=*match_count)
                .map(|number| format!("{number}. {npc_query} (npc)"))
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "More than one NPC matches '{npc_query}':\n{choices}\nTry 'ask 1 {npc_query} about {topic_query}' or another listed number."
            )
        }

        GameEvent::AskFailed {
            npc_query,
            topic_query,
            reason:
                AskFailureReason::NpcOrdinalOutOfRange {
                    requested,
                    available,
                },
            ..
        } => format!(
            "There is no #{requested} '{npc_query}' here; {available} NPCs matched for the topic '{topic_query}'."
        ),

        GameEvent::AskFailed {
            topic_query,
            reason: AskFailureReason::TopicNotFound { npc_name },
            ..
        } => format!("{npc_name} has nothing to say about '{topic_query}'."),

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
                        TargetKind::Npc => "npc",
                    };
                    format!("{}. {query} ({kind_name})", index + 1)
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "More than one accessible target matches '{query}':\n{choices}\nTry 'examine 1 {query}', 'examine item {query}', 'examine feature {query}', or 'examine npc {query}'."
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
    use crate::game::event::{
        ExaminedFeature, ExaminedItem, ExaminedNpc, ObservedFeature, ObservedItem, ObservedNpc,
        ObservedQuest,
    };
    use crate::game::ids::{
        CharacterId, FeatureId, ItemId, NpcId, NpcTopicId, QuestId, QuestKey, RoomId, WorldId,
    };

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
                items: vec![],
            }],
            npcs: vec![ObservedNpc {
                id: NpcId("test_npc".to_string()),
                name: "Test NPC".to_string(),
                room_description: "A test NPC stands here.".to_string(),
            }],
            exits: vec![Direction::West, Direction::North, Direction::East],
        };

        assert_eq!(
            render_event(&event),
            "Starting Room\nA test room.\nA test feature stands here.\nA test NPC stands here.\nItems: Test Item\nExits: east, north, west"
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
            npcs: vec![],
            exits: vec![],
        };

        assert_eq!(
            render_event(&event),
            "Starting Room\nA test room.\nItems: none\nExits: none"
        );
    }

    #[test]
    fn room_observation_identifies_items_on_features() {
        let event = GameEvent::RoomObserved {
            room_id: RoomId("start".to_string()),
            name: "Starting Room".to_string(),
            description: "A test room.".to_string(),
            items: vec![],
            features: vec![ObservedFeature {
                id: FeatureId("hook".to_string()),
                name: "Brass Hook".to_string(),
                room_description: "A brass hook is mounted here.".to_string(),
                items: vec![ObservedItem {
                    id: ItemId("cloak".to_string()),
                    name: "Traveler's Cloak".to_string(),
                }],
            }],
            npcs: vec![],
            exits: vec![],
        };

        assert!(render_event(&event).contains("On Brass Hook: Traveler's Cloak"));
        assert!(render_event(&event).contains("Items: none"));
    }

    #[test]
    fn placed_item_renders_supporter_name() {
        let event = GameEvent::ItemPlacedOnFeature {
            character_id: CharacterId("player".to_string()),
            item: ObservedItem {
                id: ItemId("cloak".to_string()),
                name: "Traveler's Cloak".to_string(),
            },
            feature_id: FeatureId("hook".to_string()),
            feature_name: "Brass Hook".to_string(),
        };

        assert_eq!(
            render_event(&event),
            "You put Traveler's Cloak on Brass Hook."
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
    fn exits_are_rendered_in_sorted_order() {
        let event = GameEvent::ExitsObserved {
            room_id: RoomId("start".to_string()),
            exits: vec![Direction::West, Direction::North, Direction::East],
        };

        assert_eq!(render_event(&event), "Exits: east, north, west");
    }

    #[test]
    fn room_without_exits_renders_none() {
        let event = GameEvent::ExitsObserved {
            room_id: RoomId("start".to_string()),
            exits: vec![],
        };

        assert_eq!(render_event(&event), "Exits: none");
    }

    #[test]
    fn help_lists_core_commands() {
        let rendered = render_event(&GameEvent::HelpRequested);

        assert!(rendered.starts_with("Commands:\n"));
        assert!(rendered.contains("go <direction>"));
        assert!(rendered.contains("look [item|feature|npc] <target>"));
        assert!(rendered.contains("help (commands)"));
        assert!(rendered.contains("talk to <npc>"));
        assert!(rendered.contains("ask <npc> about <topic>"));
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
    fn blocked_movement_renders_authored_message() {
        let event = GameEvent::MovementBlocked {
            character_id: CharacterId("player".to_string()),
            room_id: RoomId("start".to_string()),
            direction: Direction::East,
            message: "The iron gate is locked.".to_string(),
        };

        assert_eq!(render_event(&event), "The iron gate is locked.");
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
    fn empty_quest_journal_renders_none() {
        let event = GameEvent::QuestsObserved {
            active: vec![],
            completed: vec![],
        };

        assert_eq!(render_event(&event), "Quests: none");
    }

    #[test]
    fn quest_journal_renders_active_and_completed_sections() {
        let quest = |id: &str, name: &str| ObservedQuest {
            key: QuestKey {
                world_id: WorldId("origin".to_string()),
                quest_id: QuestId(id.to_string()),
            },
            name: name.to_string(),
            description: format!("The description for {name}."),
            current_objective: (id == "active").then(|| "Do the active thing.".to_string()),
        };
        let event = GameEvent::QuestsObserved {
            active: vec![quest("active", "Active Quest")],
            completed: vec![quest("complete", "Completed Quest")],
        };

        assert_eq!(
            render_event(&event),
            "Active quests:\n- Active Quest\n  The description for Active Quest.\n  Current objective: Do the active thing.\nCompleted quests:\n- Completed Quest\n  The description for Completed Quest."
        );
    }

    #[test]
    fn started_quest_renders_name_and_description() {
        let event = GameEvent::QuestStarted {
            character_id: CharacterId("player".to_string()),
            quest: ObservedQuest {
                key: QuestKey {
                    world_id: WorldId("origin".to_string()),
                    quest_id: QuestId("observatory".to_string()),
                },
                name: "Lights in the Old Observatory".to_string(),
                description: "Investigate the strange lights.".to_string(),
                current_objective: Some("Find the old observatory.".to_string()),
            },
        };

        assert_eq!(
            render_event(&event),
            "Quest started: Lights in the Old Observatory\nInvestigate the strange lights.\nCurrent objective: Find the old observatory."
        );
    }

    #[test]
    fn advanced_quest_renders_new_objective() {
        let event = GameEvent::QuestAdvanced {
            character_id: CharacterId("player".to_string()),
            quest: ObservedQuest {
                key: QuestKey {
                    world_id: WorldId("origin".to_string()),
                    quest_id: QuestId("observatory".to_string()),
                },
                name: "Lights in the Old Observatory".to_string(),
                description: "Investigate the strange lights.".to_string(),
                current_objective: Some("Return to Mara Voss.".to_string()),
            },
        };

        assert_eq!(
            render_event(&event),
            "Quest updated: Lights in the Old Observatory\nCurrent objective: Return to Mara Voss."
        );
    }

    #[test]
    fn completed_quest_renders_name() {
        let event = GameEvent::QuestCompleted {
            character_id: CharacterId("player".to_string()),
            quest: ObservedQuest {
                key: QuestKey {
                    world_id: WorldId("origin".to_string()),
                    quest_id: QuestId("observatory".to_string()),
                },
                name: "Lights in the Old Observatory".to_string(),
                description: "Investigate the strange lights.".to_string(),
                current_objective: None,
            },
        };

        assert_eq!(
            render_event(&event),
            "Quest completed: Lights in the Old Observatory"
        );
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
                items: vec![],
            },
        };

        assert_eq!(
            render_event(&event),
            "Founder's Plaque\nA weathered bronze plaque."
        );
    }

    #[test]
    fn examined_npc_renders_description() {
        let event = GameEvent::NpcExamined {
            character_id: CharacterId("player".to_string()),
            npc: ExaminedNpc {
                id: NpcId("mara_voss".to_string()),
                name: "Mara Voss".to_string(),
                description: "A sharp-eyed innkeeper.".to_string(),
            },
        };

        assert_eq!(render_event(&event), "Mara Voss\nA sharp-eyed innkeeper.");
    }

    #[test]
    fn npc_greeting_renders_as_speech() {
        let event = GameEvent::NpcSpoke {
            character_id: CharacterId("player".to_string()),
            npc_id: NpcId("mara_voss".to_string()),
            name: "Mara Voss".to_string(),
            greeting: "Welcome to the Rusty Tavern.".to_string(),
            topics: vec![],
        };

        assert_eq!(
            render_event(&event),
            "Mara Voss says, \"Welcome to the Rusty Tavern.\""
        );
    }

    #[test]
    fn npc_greeting_lists_public_topics() {
        let event = GameEvent::NpcSpoke {
            character_id: CharacterId("player".to_string()),
            npc_id: NpcId("mara_voss".to_string()),
            name: "Mara Voss".to_string(),
            greeting: "Welcome.".to_string(),
            topics: vec!["Origin Plaza".to_string()],
        };
        assert_eq!(
            render_event(&event),
            "Mara Voss says, \"Welcome.\"\nYou could ask Mara Voss about: Origin Plaza."
        );
    }

    #[test]
    fn npc_topic_response_renders_as_speech() {
        let event = GameEvent::NpcAnswered {
            character_id: CharacterId("player".to_string()),
            npc_id: NpcId("mara_voss".to_string()),
            npc_name: "Mara Voss".to_string(),
            topic_id: NpcTopicId("origin_plaza".to_string()),
            topic_name: "Origin Plaza".to_string(),
            response: "It is west.".to_string(),
            learned_facts: vec![],
            updated_topics: None,
        };
        assert_eq!(render_event(&event), "Mara Voss says, \"It is west.\"");
    }

    #[test]
    fn npc_answer_renders_topics_when_availability_changes() {
        let event = GameEvent::NpcAnswered {
            character_id: CharacterId("player".to_string()),
            npc_id: NpcId("mara_voss".to_string()),
            npc_name: "Mara Voss".to_string(),
            topic_id: NpcTopicId("strange_lights".to_string()),
            topic_name: "the strange lights".to_string(),
            response: "Look toward the old dome.".to_string(),
            learned_facts: vec![],
            updated_topics: Some(vec!["the old observatory".to_string()]),
        };

        assert_eq!(
            render_event(&event),
            "Mara Voss says, \"Look toward the old dome.\"\nYou could now ask Mara Voss about: the old observatory."
        );
    }

    #[test]
    fn npc_answer_reports_when_no_discussion_topics_remain() {
        let event = GameEvent::NpcAnswered {
            character_id: CharacterId("player".to_string()),
            npc_id: NpcId("mara_voss".to_string()),
            npc_name: "Mara Voss".to_string(),
            topic_id: NpcTopicId("final_topic".to_string()),
            topic_name: "the final topic".to_string(),
            response: "That is all I know.".to_string(),
            learned_facts: vec![],
            updated_topics: Some(vec![]),
        };

        assert_eq!(
            render_event(&event),
            "Mara Voss says, \"That is all I know.\"\nThere are no remaining discussion topics."
        );
    }

    #[test]
    fn unknown_npc_topic_renders_message() {
        let event = GameEvent::AskFailed {
            character_id: CharacterId("player".to_string()),
            npc_query: "mara voss".to_string(),
            topic_query: "dragon".to_string(),
            reason: AskFailureReason::TopicNotFound {
                npc_name: "Mara Voss".to_string(),
            },
        };
        assert_eq!(
            render_event(&event),
            "Mara Voss has nothing to say about 'dragon'."
        );
    }

    #[test]
    fn missing_ask_npc_renders_message() {
        let event = GameEvent::AskFailed {
            character_id: CharacterId("player".to_string()),
            npc_query: "missing npc".to_string(),
            topic_query: "plaza".to_string(),
            reason: AskFailureReason::NpcNotFound,
        };
        assert_eq!(
            render_event(&event),
            "You do not see 'missing npc' here to ask."
        );
    }

    #[test]
    fn ambiguous_ask_npc_renders_numbered_command() {
        let event = GameEvent::AskFailed {
            character_id: CharacterId("player".to_string()),
            npc_query: "guard".to_string(),
            topic_query: "old gate".to_string(),
            reason: AskFailureReason::NpcAmbiguous { match_count: 2 },
        };
        assert_eq!(
            render_event(&event),
            "More than one NPC matches 'guard':\n1. guard (npc)\n2. guard (npc)\nTry 'ask 1 guard about old gate' or another listed number."
        );
    }

    #[test]
    fn out_of_range_ask_npc_renders_counts() {
        let event = GameEvent::AskFailed {
            character_id: CharacterId("player".to_string()),
            npc_query: "guard".to_string(),
            topic_query: "old gate".to_string(),
            reason: AskFailureReason::NpcOrdinalOutOfRange {
                requested: 3,
                available: 2,
            },
        };
        assert_eq!(
            render_event(&event),
            "There is no #3 'guard' here; 2 NPCs matched for the topic 'old gate'."
        );
    }

    #[test]
    fn missing_talk_target_renders_message() {
        let event = GameEvent::TalkFailed {
            character_id: CharacterId("player".to_string()),
            query: "missing npc".to_string(),
            reason: TalkFailureReason::NotFound,
        };

        assert_eq!(
            render_event(&event),
            "You do not see 'missing npc' here to talk to."
        );
    }

    #[test]
    fn ambiguous_talk_target_renders_numbered_choices() {
        let event = GameEvent::TalkFailed {
            character_id: CharacterId("player".to_string()),
            query: "guard".to_string(),
            reason: TalkFailureReason::Ambiguous { match_count: 2 },
        };

        assert_eq!(
            render_event(&event),
            "More than one NPC matches 'guard':\n1. guard (npc)\n2. guard (npc)\nTry 'talk to 1 guard' or another listed number."
        );
    }

    #[test]
    fn out_of_range_talk_target_renders_counts() {
        let event = GameEvent::TalkFailed {
            character_id: CharacterId("player".to_string()),
            query: "guard".to_string(),
            reason: TalkFailureReason::OrdinalOutOfRange {
                requested: 3,
                available: 2,
            },
        };

        assert_eq!(
            render_event(&event),
            "There is no #3 'guard' here; 2 NPCs matched."
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
            "More than one accessible target matches 'key':\n1. key (item)\n2. key (feature)\nTry 'examine 1 key', 'examine item key', 'examine feature key', or 'examine npc key'."
        );
    }
}
