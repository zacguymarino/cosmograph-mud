use std::collections::{HashMap, HashSet};

use super::definition::{
    FeatureDefinition, ItemDefinition, NpcDefinition, NpcTopicDefinition, RoomDefinition,
};
use crate::game::direction::Direction;
use crate::game::exit::{Exit, ExitRequirement};
use crate::game::feature::{RoomFeature, Supporter};
use crate::game::ids::{
    FactId, FeatureId, ItemId, NpcId, NpcTopicId, QuestId, QuestStepId, RoomId, WorldId,
};
use crate::game::item::Item;
use crate::game::item_location::{ItemLocation, ItemPlacement};
use crate::game::npc::{Npc, NpcTopic};
use crate::game::quest::{Quest, QuestObjective, QuestStep};
use crate::game::room::Room;
use crate::game::world::World;
use crate::world_data::definition::WorldDefinition;

pub fn convert_room(definition: RoomDefinition) -> Result<Room, String> {
    let mut exits = HashMap::new();
    for (direction, exit_definition) in definition.exits {
        let direction = Direction::from_str(&direction)
            .ok_or_else(|| format!("unknown direction '{direction}'"))?;

        let exit = match exit_definition {
            super::definition::ExitDefinition::Simple(destination) => {
                Exit::unrestricted(RoomId(destination))
            }
            super::definition::ExitDefinition::Conditional {
                destination,
                requires_item,
                requires_fact,
                failure_message,
            } => Exit {
                destination: RoomId(destination),
                requirements: requires_item
                    .into_iter()
                    .map(|item| ExitRequirement::CarryingItem(ItemId(item)))
                    .chain(
                        requires_fact
                            .into_iter()
                            .map(|fact| ExitRequirement::KnowsFact(FactId(fact))),
                    )
                    .collect(),
                failure_message: Some(failure_message),
            },
        };

        exits.insert(direction, exit);
    }
    let features = definition.features.into_iter().map(FeatureId).collect();
    let npcs = definition.npcs.into_iter().map(NpcId).collect();
    Ok(Room {
        id: RoomId(definition.id),
        name: definition.name,
        description: definition.description,
        features,
        npcs,
        exits,
    })
}

pub fn convert_world(definition: WorldDefinition) -> Result<World, String> {
    let mut item_placements = definition
        .rooms
        .iter()
        .flat_map(|room| {
            room.items.iter().map(|item_id| ItemPlacement {
                item_id: ItemId(item_id.clone()),
                location: ItemLocation::Room(RoomId(room.id.clone())),
            })
        })
        .collect::<Vec<_>>();
    for item in &definition.items {
        if !item_placements
            .iter()
            .any(|placement| placement.item_id.0 == item.id)
        {
            item_placements.push(ItemPlacement {
                item_id: ItemId(item.id.clone()),
                location: ItemLocation::Nowhere,
            });
        }
    }
    let facts = definition
        .facts
        .into_iter()
        .map(|fact| FactId(fact.id))
        .collect::<HashSet<_>>();

    let quests = definition
        .quests
        .into_iter()
        .map(|definition| {
            let quest = Quest {
                id: QuestId(definition.id),
                name: definition.name,
                description: definition.description,
                starting_step: QuestStepId(definition.starting_step),
                steps: definition
                    .steps
                    .into_iter()
                    .map(|step| QuestStep {
                        id: QuestStepId(step.id),
                        description: step.description,
                        objective: match step.objective {
                            super::definition::QuestObjectiveDefinition::ReachRoom { room } => {
                                QuestObjective::ReachRoom(RoomId(room))
                            }
                            super::definition::QuestObjectiveDefinition::PossessItem { item } => {
                                QuestObjective::PossessItem(ItemId(item))
                            }
                            super::definition::QuestObjectiveDefinition::AskTopic {
                                npc,
                                topic,
                            } => QuestObjective::AskTopic {
                                npc_id: NpcId(npc),
                                topic_id: NpcTopicId(topic),
                            },
                        },
                        next_step: step.next_step.map(QuestStepId),
                    })
                    .collect(),
            };
            (quest.id.clone(), quest)
        })
        .collect::<HashMap<_, _>>();
    let mut rooms = HashMap::new();
    for room_definition in definition.rooms {
        let room = convert_room(room_definition)?;
        rooms.insert(room.id.clone(), room);
    }

    let mut items = HashMap::new();
    for item_definition in definition.items {
        let item = convert_item(item_definition);
        items.insert(item.id.clone(), item);
    }

    let mut features = HashMap::new();
    for feature_definition in definition.features {
        let feature = convert_feature(feature_definition);
        features.insert(feature.id.clone(), feature);
    }

    let mut npcs = HashMap::new();
    for npc_definition in definition.npcs {
        let npc = convert_npc(npc_definition);
        npcs.insert(npc.id.clone(), npc);
    }

    Ok(World {
        id: WorldId(definition.id),
        name: definition.name,
        starting_room: RoomId(definition.starting_room),
        facts,
        quests,
        items,
        item_placements,
        features,
        npcs,
        rooms,
    })
}

pub fn convert_npc(definition: NpcDefinition) -> Npc {
    Npc {
        id: NpcId(definition.id),
        name: definition.name,
        room_description: definition.room_description,
        description: definition.description,
        greeting: definition.greeting,
        topics: definition
            .topics
            .into_iter()
            .map(convert_npc_topic)
            .collect(),
    }
}

pub fn convert_npc_topic(definition: NpcTopicDefinition) -> NpcTopic {
    NpcTopic {
        id: NpcTopicId(definition.id),
        name: definition.name,
        response: definition.response,
        requires_facts: definition.requires_facts.into_iter().map(FactId).collect(),
        excludes_facts: definition.excludes_facts.into_iter().map(FactId).collect(),
        grants_facts: definition.grants_facts.into_iter().map(FactId).collect(),
        starts_quests: definition.starts_quests.into_iter().map(QuestId).collect(),
    }
}

pub fn convert_item(definition: ItemDefinition) -> Item {
    Item {
        id: ItemId(definition.id),
        name: definition.name,
        description: definition.description,
    }
}

pub fn convert_feature(definition: FeatureDefinition) -> RoomFeature {
    RoomFeature {
        id: FeatureId(definition.id),
        name: definition.name,
        room_description: definition.room_description,
        description: definition.description,
        supporter: definition
            .placement
            .map(|placement| match placement.relation {
                super::definition::PlacementRelationDefinition::On => Supporter {
                    capacity: placement.capacity,
                    accepts_items: placement.accepts_items.into_iter().map(ItemId).collect(),
                    rejection_message: placement.rejection_message,
                    full_message: placement.full_message,
                },
            }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_data::definition::{
        ExitDefinition, FactDefinition, FeatureDefinition, FeaturePlacementDefinition,
        ItemDefinition, PlacementRelationDefinition, QuestDefinition, QuestObjectiveDefinition,
        QuestStepDefinition, RoomDefinition,
    };
    use std::collections::HashMap;

    fn valid_room_definition() -> RoomDefinition {
        let mut exits = HashMap::new();
        exits.insert(
            "north".to_string(),
            ExitDefinition::Simple("next_room".to_string()),
        );

        RoomDefinition {
            id: "start".to_string(),
            name: "Starting Room".to_string(),
            description: "A test room.".to_string(),
            items: vec!["test_item".to_string()],
            features: vec!["test_feature".to_string()],
            npcs: vec!["test_npc".to_string()],
            exits,
        }
    }

    fn valid_world_definition() -> WorldDefinition {
        WorldDefinition {
            id: "test_world".to_string(),
            name: "Test World".to_string(),
            starting_room: "start".to_string(),
            facts: vec![FactDefinition {
                id: "knows_secret".to_string(),
            }],
            quests: vec![QuestDefinition {
                id: "test_quest".to_string(),
                name: "Test Quest".to_string(),
                description: "A quest used for testing.".to_string(),
                starting_step: "first_step".to_string(),
                steps: vec![QuestStepDefinition {
                    id: "first_step".to_string(),
                    description: "Complete the first objective.".to_string(),
                    objective: QuestObjectiveDefinition::ReachRoom {
                        room: "next_room".to_string(),
                    },
                    next_step: None,
                }],
            }],
            items: vec![ItemDefinition {
                id: "test_item".to_string(),
                name: "Test Item".to_string(),
                description: "An item used for testing.".to_string(),
            }],
            features: vec![FeatureDefinition {
                id: "test_feature".to_string(),
                name: "Test Feature".to_string(),
                room_description: "A test feature stands here.".to_string(),
                description: "A feature used for testing.".to_string(),
                placement: None,
            }],
            npcs: vec![NpcDefinition {
                id: "test_npc".to_string(),
                name: "Test NPC".to_string(),
                room_description: "A test NPC stands here.".to_string(),
                description: "An NPC used for testing.".to_string(),
                greeting: "Hello from the test NPC.".to_string(),
                topics: vec![NpcTopicDefinition {
                    id: "test_topic".to_string(),
                    name: "Test Topic".to_string(),
                    response: "This is the test topic response.".to_string(),
                    requires_facts: vec!["knows_secret".to_string()],
                    excludes_facts: vec![],
                    grants_facts: vec!["knows_secret".to_string()],
                    starts_quests: vec!["test_quest".to_string()],
                }],
            }],
            rooms: vec![
                valid_room_definition(),
                RoomDefinition {
                    id: "next_room".to_string(),
                    name: "Next Room".to_string(),
                    description: "Another test room.".to_string(),
                    items: vec![],
                    features: vec![],
                    npcs: vec![],
                    exits: HashMap::new(),
                },
            ],
        }
    }

    #[test]
    fn valid_world_definition_converts() {
        let definition = valid_world_definition();

        let world = convert_world(definition).expect("valid world should convert");

        let item = world
            .items
            .get(&ItemId("test_item".to_string()))
            .expect("converted item should exist");
        let feature = world
            .features
            .get(&FeatureId("test_feature".to_string()))
            .expect("converted feature should exist");
        let npc = world
            .npcs
            .get(&NpcId("test_npc".to_string()))
            .expect("converted NPC should exist");

        assert_eq!(world.id, WorldId("test_world".to_string()));
        assert_eq!(world.name, "Test World");
        assert_eq!(world.starting_room, RoomId("start".to_string()));
        assert_eq!(world.rooms.len(), 2);
        assert_eq!(world.items.len(), 1);
        assert_eq!(world.features.len(), 1);
        assert_eq!(world.npcs.len(), 1);
        assert_eq!(
            world.item_location(&ItemId("test_item".to_string())),
            Some(&ItemLocation::Room(RoomId("start".to_string())))
        );
        assert!(world.declares_fact(&FactId("knows_secret".to_string())));
        assert_eq!(
            world
                .quest(&QuestId("test_quest".to_string()))
                .map(|quest| quest.name.as_str()),
            Some("Test Quest")
        );
        let quest = world
            .quest(&QuestId("test_quest".to_string()))
            .expect("converted quest should exist");
        assert_eq!(quest.starting_step, QuestStepId("first_step".to_string()));
        assert_eq!(
            quest
                .step(&QuestStepId("first_step".to_string()))
                .map(|step| step.description.as_str()),
            Some("Complete the first objective.")
        );
        assert_eq!(
            quest
                .step(&QuestStepId("first_step".to_string()))
                .map(|step| &step.objective),
            Some(&QuestObjective::ReachRoom(RoomId("next_room".to_string())))
        );
        assert!(
            quest
                .step(&QuestStepId("first_step".to_string()))
                .expect("converted step should exist")
                .next_step
                .is_none()
        );
        assert_eq!(item.name, "Test Item");
        assert_eq!(feature.name, "Test Feature");
        assert_eq!(npc.name, "Test NPC");
        assert!(world.rooms.contains_key(&RoomId("start".to_string())));
        assert!(world.rooms.contains_key(&RoomId("next_room".to_string())));
    }

    #[test]
    fn declared_unplaced_item_converts_with_nowhere_location() {
        let mut definition = valid_world_definition();
        definition.items.push(ItemDefinition {
            id: "unplaced_item".to_string(),
            name: "Unplaced Item".to_string(),
            description: "An item not initially present in a room.".to_string(),
        });

        let world = convert_world(definition).expect("valid world should convert");

        assert_eq!(
            world.item_location(&ItemId("unplaced_item".to_string())),
            Some(&ItemLocation::Nowhere)
        );
    }

    #[test]
    fn invalid_room_prevents_world_conversion() {
        let mut definition = valid_world_definition();

        definition.rooms[0].exits.insert(
            "sideways".to_string(),
            ExitDefinition::Simple("next_room".to_string()),
        );

        let error =
            convert_world(definition).expect_err("world containing an invalid room should fail");

        assert_eq!(error, "unknown direction 'sideways'");
    }

    #[test]
    fn ask_topic_objective_converts_to_typed_npc_and_topic_ids() {
        let mut definition = valid_world_definition();
        definition.quests[0].steps[0].objective = QuestObjectiveDefinition::AskTopic {
            npc: "test_npc".to_string(),
            topic: "test_topic".to_string(),
        };

        let world = convert_world(definition).expect("valid world should convert");
        let objective = &world
            .quest(&QuestId("test_quest".to_string()))
            .unwrap()
            .steps[0]
            .objective;

        assert_eq!(
            objective,
            &QuestObjective::AskTopic {
                npc_id: NpcId("test_npc".to_string()),
                topic_id: NpcTopicId("test_topic".to_string()),
            }
        );
    }

    #[test]
    fn possess_item_objective_converts_to_typed_item_id() {
        let mut definition = valid_world_definition();
        definition.quests[0].steps[0].objective = QuestObjectiveDefinition::PossessItem {
            item: "test_item".to_string(),
        };

        let world = convert_world(definition).expect("valid world should convert");
        let objective = &world
            .quest(&QuestId("test_quest".to_string()))
            .unwrap()
            .steps[0]
            .objective;

        assert_eq!(
            objective,
            &QuestObjective::PossessItem(ItemId("test_item".to_string()))
        );
    }

    #[test]
    fn valid_room_definition_converts() {
        let room_definition = valid_room_definition();
        let room = convert_room(room_definition).expect("valid room should convert");

        assert_eq!(room.id, RoomId("start".to_string()));
        assert_eq!(room.name, "Starting Room");
        assert_eq!(
            room.exits
                .get(&Direction::North)
                .map(|exit| &exit.destination),
            Some(&RoomId("next_room".to_string()))
        );
        assert_eq!(room.features, vec![FeatureId("test_feature".to_string())]);
        assert_eq!(room.npcs, vec![NpcId("test_npc".to_string())]);
    }

    #[test]
    fn combined_exit_requirements_convert_to_typed_runtime_requirements() {
        let mut room_definition = valid_room_definition();
        room_definition.exits.insert(
            "east".to_string(),
            ExitDefinition::Conditional {
                destination: "next_room".to_string(),
                requires_item: Some("test_item".to_string()),
                requires_fact: Some("test_fact".to_string()),
                failure_message: "The door is locked.".to_string(),
            },
        );

        let room = convert_room(room_definition).expect("valid room should convert");
        let exit = room.exits.get(&Direction::East).unwrap();

        assert_eq!(exit.destination, RoomId("next_room".to_string()));
        assert_eq!(
            exit.requirements,
            vec![
                ExitRequirement::CarryingItem(ItemId("test_item".to_string())),
                ExitRequirement::KnowsFact(FactId("test_fact".to_string()))
            ]
        );
        assert_eq!(exit.failure_message.as_deref(), Some("The door is locked."));
    }

    #[test]
    fn invalid_direction_does_not_convert() {
        let mut room_definition = valid_room_definition();

        room_definition.exits.insert(
            "sideways".to_string(),
            ExitDefinition::Simple("start".to_string()),
        );

        let error = convert_room(room_definition).expect_err("unknown direction should fail");

        assert_eq!(error, "unknown direction 'sideways'");
    }

    #[test]
    fn item_definition_converts() {
        let definition = ItemDefinition {
            id: "test_item".to_string(),
            name: "Test Item".to_string(),
            description: "An item used for testing.".to_string(),
        };

        let item = convert_item(definition);

        assert_eq!(item.id, ItemId("test_item".to_string()));
        assert_eq!(item.name, "Test Item");
        assert_eq!(item.description, "An item used for testing.");
    }

    #[test]
    fn feature_definition_converts() {
        let definition = FeatureDefinition {
            id: "test_feature".to_string(),
            name: "Test Feature".to_string(),
            room_description: "A test feature stands here.".to_string(),
            description: "A feature used for testing.".to_string(),
            placement: None,
        };

        let feature = convert_feature(definition);

        assert_eq!(feature.id, FeatureId("test_feature".to_string()));
        assert_eq!(feature.name, "Test Feature");
        assert_eq!(feature.room_description, "A test feature stands here.");
        assert_eq!(feature.description, "A feature used for testing.");
    }

    #[test]
    fn supporter_definition_converts_with_typed_accepted_items() {
        let definition = FeatureDefinition {
            id: "hook".to_string(),
            name: "Hook".to_string(),
            room_description: "A hook is mounted here.".to_string(),
            description: "A sturdy hook.".to_string(),
            placement: Some(FeaturePlacementDefinition {
                relation: PlacementRelationDefinition::On,
                capacity: Some(1),
                accepts_items: vec!["test_item".to_string()],
                rejection_message: Some("That will not hang.".to_string()),
                full_message: Some("The hook is occupied.".to_string()),
            }),
        };

        let feature = convert_feature(definition);
        let supporter = feature.supporter.expect("supporter should convert");

        assert_eq!(supporter.capacity, Some(1));
        assert_eq!(
            supporter.accepts_items,
            vec![ItemId("test_item".to_string())]
        );
        assert_eq!(
            supporter.rejection_message.as_deref(),
            Some("That will not hang.")
        );
    }

    #[test]
    fn npc_definition_converts() {
        let npc = convert_npc(NpcDefinition {
            id: "test_npc".to_string(),
            name: "Test NPC".to_string(),
            room_description: "A test NPC stands here.".to_string(),
            description: "An NPC used for testing.".to_string(),
            greeting: "Hello from the test NPC.".to_string(),
            topics: vec![NpcTopicDefinition {
                id: "test_topic".to_string(),
                name: "Test Topic".to_string(),
                response: "This is the test topic response.".to_string(),
                requires_facts: vec![],
                excludes_facts: vec![],
                grants_facts: vec![],
                starts_quests: vec![],
            }],
        });

        assert_eq!(npc.id, NpcId("test_npc".to_string()));
        assert_eq!(npc.name, "Test NPC");
        assert_eq!(npc.room_description, "A test NPC stands here.");
        assert_eq!(npc.description, "An NPC used for testing.");
        assert_eq!(npc.greeting, "Hello from the test NPC.");
        assert_eq!(npc.topics.len(), 1);
        assert_eq!(npc.topics[0].id, NpcTopicId("test_topic".to_string()));
        assert!(npc.topics[0].requires_facts.is_empty());
        assert!(npc.topics[0].excludes_facts.is_empty());
        assert!(npc.topics[0].grants_facts.is_empty());
        assert!(npc.topics[0].starts_quests.is_empty());
    }
}
