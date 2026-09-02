use std::collections::HashMap;

use super::definition::{FeatureDefinition, ItemDefinition, NpcDefinition, RoomDefinition};
use crate::game::direction::Direction;
use crate::game::feature::RoomFeature;
use crate::game::ids::{FeatureId, ItemId, NpcId, RoomId, WorldId};
use crate::game::item::Item;
use crate::game::npc::Npc;
use crate::game::room::Room;
use crate::game::world::World;
use crate::world_data::definition::WorldDefinition;

pub fn convert_room(definition: RoomDefinition) -> Result<Room, String> {
    let mut exits = HashMap::new();
    for (direction, destination) in definition.exits {
        let direction = Direction::from_str(&direction)
            .ok_or_else(|| format!("unknown direction '{direction}'"))?;

        exits.insert(direction, RoomId(destination));
    }
    let items = definition.items.into_iter().map(ItemId).collect();
    let features = definition.features.into_iter().map(FeatureId).collect();
    let npcs = definition.npcs.into_iter().map(NpcId).collect();
    Ok(Room {
        id: RoomId(definition.id),
        name: definition.name,
        description: definition.description,
        items,
        features,
        npcs,
        exits,
    })
}

pub fn convert_world(definition: WorldDefinition) -> Result<World, String> {
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
        items,
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_data::definition::{FeatureDefinition, ItemDefinition, RoomDefinition};
    use std::collections::HashMap;

    fn valid_room_definition() -> RoomDefinition {
        let mut exits = HashMap::new();
        exits.insert("north".to_string(), "next_room".to_string());

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
            }],
            npcs: vec![NpcDefinition {
                id: "test_npc".to_string(),
                name: "Test NPC".to_string(),
                room_description: "A test NPC stands here.".to_string(),
                description: "An NPC used for testing.".to_string(),
                greeting: "Hello from the test NPC.".to_string(),
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
        assert_eq!(item.name, "Test Item");
        assert_eq!(feature.name, "Test Feature");
        assert_eq!(npc.name, "Test NPC");
        assert!(world.rooms.contains_key(&RoomId("start".to_string())));
        assert!(world.rooms.contains_key(&RoomId("next_room".to_string())));
    }

    #[test]
    fn invalid_room_prevents_world_conversion() {
        let mut definition = valid_world_definition();

        definition.rooms[0]
            .exits
            .insert("sideways".to_string(), "next_room".to_string());

        let error =
            convert_world(definition).expect_err("world containing an invalid room should fail");

        assert_eq!(error, "unknown direction 'sideways'");
    }

    #[test]
    fn valid_room_definition_converts() {
        let room_definition = valid_room_definition();
        let room = convert_room(room_definition).expect("valid room should convert");

        assert_eq!(room.id, RoomId("start".to_string()));
        assert_eq!(room.name, "Starting Room");
        assert_eq!(room.items, vec![ItemId("test_item".to_string())]);
        assert_eq!(
            room.exits.get(&Direction::North),
            Some(&RoomId("next_room".to_string()))
        );
        assert_eq!(room.features, vec![FeatureId("test_feature".to_string())]);
        assert_eq!(room.npcs, vec![NpcId("test_npc".to_string())]);
    }

    #[test]
    fn invalid_direction_does_not_convert() {
        let mut room_definition = valid_room_definition();

        room_definition
            .exits
            .insert("sideways".to_string(), "start".to_string());

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
        };

        let feature = convert_feature(definition);

        assert_eq!(feature.id, FeatureId("test_feature".to_string()));
        assert_eq!(feature.name, "Test Feature");
        assert_eq!(feature.room_description, "A test feature stands here.");
        assert_eq!(feature.description, "A feature used for testing.");
    }

    #[test]
    fn npc_definition_converts() {
        let npc = convert_npc(NpcDefinition {
            id: "test_npc".to_string(),
            name: "Test NPC".to_string(),
            room_description: "A test NPC stands here.".to_string(),
            description: "An NPC used for testing.".to_string(),
            greeting: "Hello from the test NPC.".to_string(),
        });

        assert_eq!(npc.id, NpcId("test_npc".to_string()));
        assert_eq!(npc.name, "Test NPC");
        assert_eq!(npc.room_description, "A test NPC stands here.");
        assert_eq!(npc.description, "An NPC used for testing.");
        assert_eq!(npc.greeting, "Hello from the test NPC.");
    }
}
