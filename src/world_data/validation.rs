use super::definition::WorldDefinition;
use std::collections::HashSet;

fn validate_starting_room(world: &WorldDefinition) -> Result<(), String> {
    let target_exists = world
        .rooms
        .iter()
        .any(|room| room.id == world.starting_room);

    if target_exists {
        Ok(())
    } else {
        Err(format!(
            "Starting room '{}' does not exist",
            world.starting_room
        ))
    }
}

fn validate_unique_room_ids(world: &WorldDefinition) -> Result<(), String> {
    let mut seen = HashSet::new();
    for room in &world.rooms {
        if !seen.insert(&room.id) {
            return Err(format!(
                "The room named {} has the duplicate id: {}",
                &room.name, &room.id
            ));
        }
    }
    Ok(())
}

fn validate_exit_destinations(world: &WorldDefinition) -> Result<(), String> {
    for room in &world.rooms {
        for (direction, destination) in &room.exits {
            let target_exists = world
                .rooms
                .iter()
                .any(|candidate| &candidate.id == destination);
            if !target_exists {
                return Err(format!(
                    "Room with id '{}' has exit '{}' pointing to missing room '{}'",
                    room.id, direction, destination
                ));
            }
        }
    }
    Ok(())
}

fn validate_non_empty_room_ids(world: &WorldDefinition) -> Result<(), String> {
    for room in &world.rooms {
        if room.id.trim().is_empty() {
            return Err(format!("Room '{}' has no id", room.name));
        }
    }
    Ok(())
}

fn is_valid_id(id: &str) -> bool {
    id.chars().all(|character| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
    })
}

fn validate_room_id_formats(world: &WorldDefinition) -> Result<(), String> {
    for room in &world.rooms {
        if !is_valid_id(&room.id) {
            return Err(format!(
                "Id for room '{}' has invalid characters: {}",
                room.name, room.id
            ));
        }
    }
    Ok(())
}

fn validate_world_id(world: &WorldDefinition) -> Result<(), String> {
    if world.id.trim().is_empty() {
        return Err("World id is empty".to_string());
    }
    if !is_valid_id(&world.id) {
        return Err(format!("World id has invalid characters: {}", world.id));
    }
    Ok(())
}

fn is_valid_direction(direction: &str) -> bool {
    matches!(direction, "north" | "south" | "east" | "west")
}

fn validate_exit_directions(world: &WorldDefinition) -> Result<(), String> {
    for room in &world.rooms {
        for (direction, _destination) in &room.exits {
            if !is_valid_direction(direction) {
                return Err(format!(
                    "Room with id '{}' has invalid exit direction '{}'",
                    room.id, direction
                ));
            }
        }
    }
    Ok(())
}

fn validate_item_ids(world: &WorldDefinition) -> Result<(), String> {
    for item in &world.items {
        if item.id.trim().is_empty() {
            return Err(format!("Item '{}' has no id", item.name));
        }

        if !is_valid_id(&item.id) {
            return Err(format!(
                "Id for item '{}' has invalid characters: {}",
                item.name, item.id
            ));
        }
    }

    Ok(())
}

fn validate_unique_item_ids(world: &WorldDefinition) -> Result<(), String> {
    let mut seen = HashSet::new();

    for item in &world.items {
        if !seen.insert(&item.id) {
            return Err(format!(
                "The item named {} has the duplicate id: {}",
                item.name, item.id
            ));
        }
    }

    Ok(())
}

fn validate_room_item_references(world: &WorldDefinition) -> Result<(), String> {
    for room in &world.rooms {
        for item_id in &room.items {
            let item_exists = world.items.iter().any(|item| &item.id == item_id);

            if !item_exists {
                return Err(format!(
                    "Room '{}' references missing item '{}'",
                    room.id, item_id
                ));
            }
        }
    }

    Ok(())
}

fn validate_unique_item_placements(world: &WorldDefinition) -> Result<(), String> {
    let mut placed_items = HashSet::new();

    for room in &world.rooms {
        for item_id in &room.items {
            if !placed_items.insert(item_id) {
                return Err(format!("Item '{}' is placed more than once", item_id));
            }
        }
    }

    Ok(())
}

pub fn validate_world(world: &WorldDefinition) -> Result<(), String> {
    validate_world_id(world)?;
    validate_item_ids(world)?;
    validate_unique_item_ids(world)?;
    validate_room_item_references(world)?;
    validate_unique_item_placements(world)?;
    validate_non_empty_room_ids(world)?;
    validate_room_id_formats(world)?;
    validate_unique_room_ids(world)?;
    validate_exit_directions(world)?;
    validate_exit_destinations(world)?;
    validate_starting_room(world)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_data::definition::{ItemDefinition, RoomDefinition};
    use std::collections::HashMap;

    fn valid_world() -> WorldDefinition {
        WorldDefinition {
            id: "test_world".to_string(),
            name: "Test World".to_string(),
            starting_room: "start".to_string(),
            items: vec![],
            rooms: vec![RoomDefinition {
                id: "start".to_string(),
                name: "Starting Room".to_string(),
                description: "A test room.".to_string(),
                items: vec![],
                exits: HashMap::new(),
            }],
        }
    }

    #[test]
    fn valid_world_passes_validation() {
        let world = valid_world();

        assert!(validate_world(&world).is_ok());
    }

    #[test]
    fn empty_world_id_fails_validation() {
        let mut world = valid_world();
        world.id = "  ".to_string();

        let result = validate_world(&world);

        assert_eq!(result, Err("World id is empty".to_string()));
    }

    #[test]
    fn invalid_world_id_fails_validation() {
        let mut world = valid_world();
        world.id = "test!".to_string();

        let result = validate_world(&world);

        assert_eq!(
            result,
            Err("World id has invalid characters: test!".to_string())
        );
    }

    #[test]
    fn invalid_room_id_fails_validation() {
        let mut world = valid_world();
        world.rooms[0].id = "test!".to_string();

        let result = validate_world(&world);

        assert_eq!(
            result,
            Err("Id for room 'Starting Room' has invalid characters: test!".to_string())
        );
    }

    #[test]
    fn empty_room_id_fails_validation() {
        let mut world = valid_world();
        world.rooms[0].id = "  ".to_string();

        let result = validate_world(&world);

        assert_eq!(result, Err("Room 'Starting Room' has no id".to_string()));
    }

    #[test]
    fn missing_exit_destination_fails_validation() {
        let mut world = valid_world();
        world.rooms[0]
            .exits
            .insert("north".to_string(), "missing_room".to_string());

        let result = validate_world(&world);

        assert_eq!(
            result,
            Err(
                "Room with id 'start' has exit 'north' pointing to missing room 'missing_room'"
                    .to_string()
            )
        );
    }

    #[test]
    fn duplicate_room_ids_fail_validation() {
        let mut world = valid_world();
        world.rooms.push(RoomDefinition {
            id: "start".to_string(),
            name: "New Room".to_string(),
            description: "A second test room.".to_string(),
            items: vec![],
            exits: HashMap::new(),
        });

        let result = validate_world(&world);

        assert_eq!(
            result,
            Err("The room named New Room has the duplicate id: start".to_string())
        );
    }

    #[test]
    fn missing_starting_room_fails_validation() {
        let mut world = valid_world();
        world.starting_room = "end".to_string();

        let result = validate_world(&world);

        assert_eq!(
            result,
            Err("Starting room 'end' does not exist".to_string())
        );
    }

    #[test]
    fn unknown_direction_fails_validation() {
        let mut world = valid_world();
        world.rooms[0]
            .exits
            .insert("sideways".to_string(), "start".to_string());

        let result = validate_world(&world);

        assert_eq!(
            result,
            Err("Room with id 'start' has invalid exit direction 'sideways'".to_string())
        );
    }

    #[test]
    fn empty_item_id_fails_validation() {
        let mut world = valid_world();

        world.items.push(ItemDefinition {
            id: "  ".to_string(),
            name: "Test Item".to_string(),
            description: "An item used for testing.".to_string(),
        });

        let result = validate_world(&world);

        assert_eq!(result, Err("Item 'Test Item' has no id".to_string()));
    }

    #[test]
    fn invalid_item_id_fails_validation() {
        let mut world = valid_world();

        world.items.push(ItemDefinition {
            id: "bad!".to_string(),
            name: "Test Item".to_string(),
            description: "An item used for testing.".to_string(),
        });

        let result = validate_world(&world);

        assert_eq!(
            result,
            Err("Id for item 'Test Item' has invalid characters: bad!".to_string())
        );
    }

    #[test]
    fn duplicate_item_ids_fail_validation() {
        let mut world = valid_world();

        world.items.push(ItemDefinition {
            id: "test_item".to_string(),
            name: "First Item".to_string(),
            description: "The first item.".to_string(),
        });

        world.items.push(ItemDefinition {
            id: "test_item".to_string(),
            name: "Second Item".to_string(),
            description: "The second item".to_string(),
        });

        let result = validate_world(&world);

        assert_eq!(
            result,
            Err("The item named Second Item has the duplicate id: test_item".to_string())
        );
    }

    #[test]
    fn room_item_reference_must_exist() {
        let mut world = valid_world();

        world.rooms[0].items.push("missing_item".to_string());

        let result = validate_world(&world);

        assert_eq!(
            result,
            Err("Room 'start' references missing item 'missing_item'".to_string())
        );
    }

    #[test]
    fn item_cannot_start_in_multiple_rooms() {
        let mut world = valid_world();

        world.items.push(ItemDefinition {
            id: "test_item".to_string(),
            name: "Test Item".to_string(),
            description: "An item used for testing.".to_string(),
        });

        world.rooms[0].items.push("test_item".to_string());

        world.rooms.push(RoomDefinition {
            id: "second_room".to_string(),
            name: "Second Room".to_string(),
            description: "Another room".to_string(),
            items: vec!["test_item".to_string()],
            exits: HashMap::new(),
        });

        let result = validate_world(&world);

        assert_eq!(
            result,
            Err("Item 'test_item' is placed more than once".to_string())
        );
    }
}
