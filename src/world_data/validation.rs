use super::definition::WorldDefinition;
use crate::game::naming::normalize_name;
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

fn validate_room_feature_references(world: &WorldDefinition) -> Result<(), String> {
    for room in &world.rooms {
        for feature_id in &room.features {
            let feature_exists = world
                .features
                .iter()
                .any(|feature| &feature.id == feature_id);

            if !feature_exists {
                return Err(format!(
                    "Room '{}' references missing feature '{}'",
                    room.id, feature_id
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

fn validate_feature_ids(world: &WorldDefinition) -> Result<(), String> {
    for feature in &world.features {
        if feature.id.trim().is_empty() {
            return Err(format!("Feature '{}' has no id", feature.name));
        }

        if !is_valid_id(&feature.id) {
            return Err(format!(
                "Id for feature '{}' has invalid characters: {}",
                feature.name, feature.id
            ));
        }
    }

    Ok(())
}

fn validate_unique_feature_ids(world: &WorldDefinition) -> Result<(), String> {
    let mut seen = HashSet::new();

    for feature in &world.features {
        if !seen.insert(&feature.id) {
            return Err(format!(
                "The feature named {} has the duplicate id: {}",
                feature.name, feature.id
            ));
        }
    }

    Ok(())
}

fn validate_unique_feature_placements(world: &WorldDefinition) -> Result<(), String> {
    let mut placed_features = HashSet::new();

    for room in &world.rooms {
        for feature_id in &room.features {
            if !placed_features.insert(feature_id) {
                return Err(format!("Feature '{}' is placed more than once", feature_id));
            }
        }
    }

    Ok(())
}

fn validate_npc_ids(world: &WorldDefinition) -> Result<(), String> {
    for npc in &world.npcs {
        if npc.id.trim().is_empty() {
            return Err(format!("NPC '{}' has no id", npc.name));
        }
        if !is_valid_id(&npc.id) {
            return Err(format!(
                "Id for NPC '{}' has invalid characters: {}",
                npc.name, npc.id
            ));
        }
    }
    Ok(())
}

fn validate_unique_npc_ids(world: &WorldDefinition) -> Result<(), String> {
    let mut seen = HashSet::new();
    for npc in &world.npcs {
        if !seen.insert(&npc.id) {
            return Err(format!(
                "The NPC named {} has the duplicate id: {}",
                npc.name, npc.id
            ));
        }
    }
    Ok(())
}

fn validate_room_npc_references(world: &WorldDefinition) -> Result<(), String> {
    for room in &world.rooms {
        for npc_id in &room.npcs {
            if !world.npcs.iter().any(|npc| &npc.id == npc_id) {
                return Err(format!(
                    "Room '{}' references missing NPC '{}'",
                    room.id, npc_id
                ));
            }
        }
    }
    Ok(())
}

fn validate_unique_npc_placements(world: &WorldDefinition) -> Result<(), String> {
    let mut placed_npcs = HashSet::new();
    for room in &world.rooms {
        for npc_id in &room.npcs {
            if !placed_npcs.insert(npc_id) {
                return Err(format!("NPC '{}' is placed more than once", npc_id));
            }
        }
    }
    Ok(())
}

fn validate_fact_ids(world: &WorldDefinition) -> Result<(), String> {
    let mut seen = HashSet::new();

    for fact in &world.facts {
        if fact.id.trim().is_empty() {
            return Err("Fact has no id".to_string());
        }
        if !is_valid_id(&fact.id) {
            return Err(format!("Fact id has invalid characters: {}", fact.id));
        }
        if !seen.insert(&fact.id) {
            return Err(format!("World has duplicate fact id '{}'", fact.id));
        }
    }

    Ok(())
}

fn validate_quests(world: &WorldDefinition) -> Result<(), String> {
    let mut seen = HashSet::new();

    for quest in &world.quests {
        if quest.id.trim().is_empty() {
            return Err(format!("Quest '{}' has no id", quest.name));
        }
        if !is_valid_id(&quest.id) {
            return Err(format!(
                "Id for quest '{}' has invalid characters: {}",
                quest.name, quest.id
            ));
        }
        if !seen.insert(&quest.id) {
            return Err(format!(
                "The quest named {} has the duplicate id: {}",
                quest.name, quest.id
            ));
        }
        if quest.name.trim().is_empty() {
            return Err(format!("Quest '{}' has no name", quest.id));
        }
        if quest.description.trim().is_empty() {
            return Err(format!("Quest '{}' has no description", quest.id));
        }

        if quest.starting_step.trim().is_empty() {
            return Err(format!("Quest '{}' has no starting step", quest.id));
        }
        if !is_valid_id(&quest.starting_step) {
            return Err(format!(
                "Starting step for quest '{}' has invalid characters: {}",
                quest.id, quest.starting_step
            ));
        }

        let mut step_ids = HashSet::new();
        for step in &quest.steps {
            if step.id.trim().is_empty() {
                return Err(format!("A step on quest '{}' has no id", quest.id));
            }
            if !is_valid_id(&step.id) {
                return Err(format!(
                    "Step id on quest '{}' has invalid characters: {}",
                    quest.id, step.id
                ));
            }
            if !step_ids.insert(&step.id) {
                return Err(format!(
                    "Quest '{}' has duplicate step id '{}'",
                    quest.id, step.id
                ));
            }
            if step.description.trim().is_empty() {
                return Err(format!(
                    "Step '{}' on quest '{}' has no description",
                    step.id, quest.id
                ));
            }
            match &step.objective {
                super::definition::QuestObjectiveDefinition::ReachRoom { room } => {
                    if !world.rooms.iter().any(|candidate| &candidate.id == room) {
                        return Err(format!(
                            "Step '{}' on quest '{}' references missing room '{}'",
                            step.id, quest.id, room
                        ));
                    }
                }
            }
            if let Some(next_step) = &step.next_step {
                if next_step.trim().is_empty() {
                    return Err(format!(
                        "Step '{}' on quest '{}' has an empty next step",
                        step.id, quest.id
                    ));
                }
                if !is_valid_id(next_step) {
                    return Err(format!(
                        "Next step for '{}' on quest '{}' has invalid characters: {}",
                        step.id, quest.id, next_step
                    ));
                }
            }
        }

        if !step_ids.contains(&quest.starting_step) {
            return Err(format!(
                "Quest '{}' references missing starting step '{}'",
                quest.id, quest.starting_step
            ));
        }

        for step in &quest.steps {
            if let Some(next_step) = &step.next_step {
                if !step_ids.contains(next_step) {
                    return Err(format!(
                        "Step '{}' on quest '{}' references missing next step '{}'",
                        step.id, quest.id, next_step
                    ));
                }
            }
        }

        let mut reachable = HashSet::new();
        let mut current = Some(quest.starting_step.as_str());
        while let Some(step_id) = current {
            if !reachable.insert(step_id) {
                return Err(format!(
                    "Quest '{}' contains a cycle at step '{}'",
                    quest.id, step_id
                ));
            }
            current = quest
                .steps
                .iter()
                .find(|step| step.id == step_id)
                .and_then(|step| step.next_step.as_deref());
        }

        if let Some(unreachable) = quest
            .steps
            .iter()
            .find(|step| !reachable.contains(step.id.as_str()))
        {
            return Err(format!(
                "Step '{}' on quest '{}' is unreachable from the starting step",
                unreachable.id, quest.id
            ));
        }
    }

    Ok(())
}

fn validate_topic_quest_references(world: &WorldDefinition) -> Result<(), String> {
    let declared_quests = world
        .quests
        .iter()
        .map(|quest| quest.id.as_str())
        .collect::<HashSet<_>>();

    for npc in &world.npcs {
        for topic in &npc.topics {
            let mut started = HashSet::new();
            for quest_id in &topic.starts_quests {
                if !declared_quests.contains(quest_id.as_str()) {
                    return Err(format!(
                        "Topic '{}' on NPC '{}' starts missing quest '{}'",
                        topic.id, npc.id, quest_id
                    ));
                }
                if !started.insert(quest_id) {
                    return Err(format!(
                        "Topic '{}' on NPC '{}' starts quest '{}' more than once",
                        topic.id, npc.id, quest_id
                    ));
                }
            }
        }
    }

    Ok(())
}

fn validate_topic_fact_references(world: &WorldDefinition) -> Result<(), String> {
    let declared_facts = world
        .facts
        .iter()
        .map(|fact| fact.id.as_str())
        .collect::<HashSet<_>>();

    for npc in &world.npcs {
        for topic in &npc.topics {
            let mut required = HashSet::new();
            for fact_id in &topic.requires_facts {
                if !declared_facts.contains(fact_id.as_str()) {
                    return Err(format!(
                        "Topic '{}' on NPC '{}' requires missing fact '{}'",
                        topic.id, npc.id, fact_id
                    ));
                }
                if !required.insert(fact_id) {
                    return Err(format!(
                        "Topic '{}' on NPC '{}' requires fact '{}' more than once",
                        topic.id, npc.id, fact_id
                    ));
                }
            }

            let mut excluded = HashSet::new();
            for fact_id in &topic.excludes_facts {
                if !declared_facts.contains(fact_id.as_str()) {
                    return Err(format!(
                        "Topic '{}' on NPC '{}' excludes missing fact '{}'",
                        topic.id, npc.id, fact_id
                    ));
                }
                if !excluded.insert(fact_id) {
                    return Err(format!(
                        "Topic '{}' on NPC '{}' excludes fact '{}' more than once",
                        topic.id, npc.id, fact_id
                    ));
                }
                if required.contains(fact_id) {
                    return Err(format!(
                        "Topic '{}' on NPC '{}' both requires and excludes fact '{}'",
                        topic.id, npc.id, fact_id
                    ));
                }
            }

            let mut granted = HashSet::new();
            for fact_id in &topic.grants_facts {
                if !declared_facts.contains(fact_id.as_str()) {
                    return Err(format!(
                        "Topic '{}' on NPC '{}' grants missing fact '{}'",
                        topic.id, npc.id, fact_id
                    ));
                }
                if !granted.insert(fact_id) {
                    return Err(format!(
                        "Topic '{}' on NPC '{}' grants fact '{}' more than once",
                        topic.id, npc.id, fact_id
                    ));
                }
            }
        }
    }

    Ok(())
}

fn validate_npc_topics(world: &WorldDefinition) -> Result<(), String> {
    for npc in &world.npcs {
        let mut topic_ids = HashSet::new();
        let mut topic_names = HashSet::new();
        for topic in &npc.topics {
            if topic.id.trim().is_empty() {
                return Err(format!(
                    "Topic '{}' on NPC '{}' has no id",
                    topic.name, npc.id
                ));
            }
            if !is_valid_id(&topic.id) {
                return Err(format!(
                    "Id for topic '{}' on NPC '{}' has invalid characters: {}",
                    topic.name, npc.id, topic.id
                ));
            }
            if !topic_ids.insert(&topic.id) {
                return Err(format!(
                    "NPC '{}' has duplicate topic id '{}'",
                    npc.id, topic.id
                ));
            }
            let normalized_name = normalize_name(&topic.name);
            if normalized_name.is_empty() {
                return Err(format!(
                    "Topic '{}' on NPC '{}' has no targetable name",
                    topic.id, npc.id
                ));
            }
            if !topic_names.insert(normalized_name.clone()) {
                return Err(format!(
                    "NPC '{}' has duplicate normalized topic name '{}'",
                    npc.id, normalized_name
                ));
            }
        }
    }
    Ok(())
}

pub fn validate_world(world: &WorldDefinition) -> Result<(), String> {
    validate_world_id(world)?;
    validate_item_ids(world)?;
    validate_unique_item_ids(world)?;
    validate_feature_ids(world)?;
    validate_unique_feature_ids(world)?;
    validate_room_feature_references(world)?;
    validate_unique_feature_placements(world)?;
    validate_npc_ids(world)?;
    validate_unique_npc_ids(world)?;
    validate_npc_topics(world)?;
    validate_fact_ids(world)?;
    validate_quests(world)?;
    validate_topic_fact_references(world)?;
    validate_topic_quest_references(world)?;
    validate_room_npc_references(world)?;
    validate_unique_npc_placements(world)?;
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
    use crate::world_data::definition::{
        FactDefinition, FeatureDefinition, ItemDefinition, NpcDefinition, NpcTopicDefinition,
        QuestDefinition, QuestObjectiveDefinition, QuestStepDefinition, RoomDefinition,
    };
    use std::collections::HashMap;

    fn valid_world() -> WorldDefinition {
        WorldDefinition {
            id: "test_world".to_string(),
            name: "Test World".to_string(),
            starting_room: "start".to_string(),
            facts: vec![],
            quests: vec![],
            items: vec![],
            features: vec![],
            npcs: vec![],
            rooms: vec![RoomDefinition {
                id: "start".to_string(),
                name: "Starting Room".to_string(),
                description: "A test room.".to_string(),
                items: vec![],
                features: vec![],
                npcs: vec![],
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
            features: vec![],
            npcs: vec![],
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
            features: vec![],
            npcs: vec![],
            exits: HashMap::new(),
        });

        let result = validate_world(&world);

        assert_eq!(
            result,
            Err("Item 'test_item' is placed more than once".to_string())
        );
    }

    #[test]
    fn empty_feature_id_fails_validation() {
        let mut world = valid_world();

        world.features.push(FeatureDefinition {
            id: "  ".to_string(),
            name: "Test Feature".to_string(),
            room_description: "A test feature stands here.".to_string(),
            description: "A feature used for testing.".to_string(),
        });

        let result = validate_world(&world);

        assert_eq!(result, Err("Feature 'Test Feature' has no id".to_string()));
    }

    #[test]
    fn invalid_feature_id_fails_validation() {
        let mut world = valid_world();

        world.features.push(FeatureDefinition {
            id: "bad!".to_string(),
            name: "Test Feature".to_string(),
            room_description: "A test feature stands here.".to_string(),
            description: "A feature used for testing.".to_string(),
        });

        let result = validate_world(&world);

        assert_eq!(
            result,
            Err("Id for feature 'Test Feature' has invalid characters: bad!".to_string())
        );
    }

    #[test]
    fn duplicate_feature_ids_fail_validation() {
        let mut world = valid_world();

        world.features.push(FeatureDefinition {
            id: "test_feature".to_string(),
            name: "First Feature".to_string(),
            room_description: "The first feature stands here.".to_string(),
            description: "The first feature.".to_string(),
        });

        world.features.push(FeatureDefinition {
            id: "test_feature".to_string(),
            name: "Second Feature".to_string(),
            room_description: "The second feature stands here.".to_string(),
            description: "The second feature.".to_string(),
        });

        let result = validate_world(&world);

        assert_eq!(
            result,
            Err("The feature named Second Feature has the duplicate id: test_feature".to_string())
        );
    }

    #[test]
    fn room_feature_reference_must_exist() {
        let mut world = valid_world();

        world.rooms[0].features.push("missing_feature".to_string());

        let result = validate_world(&world);

        assert_eq!(
            result,
            Err("Room 'start' references missing feature 'missing_feature'".to_string())
        );
    }

    #[test]
    fn feature_cannot_be_placed_in_multiple_rooms() {
        let mut world = valid_world();

        world.features.push(FeatureDefinition {
            id: "test_feature".to_string(),
            name: "Test Feature".to_string(),
            room_description: "A test feature stands here.".to_string(),
            description: "A feature used for testing.".to_string(),
        });

        world.rooms[0].features.push("test_feature".to_string());

        world.rooms.push(RoomDefinition {
            id: "second_room".to_string(),
            name: "Second Room".to_string(),
            description: "Another room.".to_string(),
            items: vec![],
            features: vec!["test_feature".to_string()],
            npcs: vec![],
            exits: HashMap::new(),
        });

        let result = validate_world(&world);

        assert_eq!(
            result,
            Err("Feature 'test_feature' is placed more than once".to_string())
        );
    }

    fn test_npc(id: &str, name: &str) -> NpcDefinition {
        NpcDefinition {
            id: id.to_string(),
            name: name.to_string(),
            room_description: "A test NPC stands here.".to_string(),
            description: "An NPC used for testing.".to_string(),
            greeting: "Hello from the test NPC.".to_string(),
            topics: vec![],
        }
    }

    #[test]
    fn empty_npc_id_fails_validation() {
        let mut world = valid_world();
        world.npcs.push(test_npc("  ", "Test NPC"));

        assert_eq!(
            validate_world(&world),
            Err("NPC 'Test NPC' has no id".to_string())
        );
    }

    #[test]
    fn invalid_npc_id_fails_validation() {
        let mut world = valid_world();
        world.npcs.push(test_npc("bad!", "Test NPC"));

        assert_eq!(
            validate_world(&world),
            Err("Id for NPC 'Test NPC' has invalid characters: bad!".to_string())
        );
    }

    #[test]
    fn duplicate_npc_ids_fail_validation() {
        let mut world = valid_world();
        world.npcs.push(test_npc("test_npc", "First NPC"));
        world.npcs.push(test_npc("test_npc", "Second NPC"));

        assert_eq!(
            validate_world(&world),
            Err("The NPC named Second NPC has the duplicate id: test_npc".to_string())
        );
    }

    #[test]
    fn room_npc_reference_must_exist() {
        let mut world = valid_world();
        world.rooms[0].npcs.push("missing_npc".to_string());

        assert_eq!(
            validate_world(&world),
            Err("Room 'start' references missing NPC 'missing_npc'".to_string())
        );
    }

    #[test]
    fn npc_cannot_start_in_multiple_rooms() {
        let mut world = valid_world();
        world.npcs.push(test_npc("test_npc", "Test NPC"));
        world.rooms[0].npcs.push("test_npc".to_string());
        world.rooms.push(RoomDefinition {
            id: "second_room".to_string(),
            name: "Second Room".to_string(),
            description: "Another room.".to_string(),
            items: vec![],
            features: vec![],
            npcs: vec!["test_npc".to_string()],
            exits: HashMap::new(),
        });

        assert_eq!(
            validate_world(&world),
            Err("NPC 'test_npc' is placed more than once".to_string())
        );
    }

    fn test_topic(id: &str, name: &str) -> NpcTopicDefinition {
        NpcTopicDefinition {
            id: id.to_string(),
            name: name.to_string(),
            response: "A test response.".to_string(),
            requires_facts: vec![],
            excludes_facts: vec![],
            grants_facts: vec![],
            starts_quests: vec![],
        }
    }

    #[test]
    fn empty_npc_topic_id_fails_validation() {
        let mut world = valid_world();
        let mut npc = test_npc("test_npc", "Test NPC");
        npc.topics.push(test_topic("  ", "Test Topic"));
        world.npcs.push(npc);
        assert_eq!(
            validate_world(&world),
            Err("Topic 'Test Topic' on NPC 'test_npc' has no id".to_string())
        );
    }

    #[test]
    fn invalid_npc_topic_id_fails_validation() {
        let mut world = valid_world();
        let mut npc = test_npc("test_npc", "Test NPC");
        npc.topics.push(test_topic("bad!", "Test Topic"));
        world.npcs.push(npc);
        assert_eq!(
            validate_world(&world),
            Err(
                "Id for topic 'Test Topic' on NPC 'test_npc' has invalid characters: bad!"
                    .to_string()
            )
        );
    }

    #[test]
    fn duplicate_npc_topic_ids_fail_validation() {
        let mut world = valid_world();
        let mut npc = test_npc("test_npc", "Test NPC");
        npc.topics.push(test_topic("shared", "First Topic"));
        npc.topics.push(test_topic("shared", "Second Topic"));
        world.npcs.push(npc);
        assert_eq!(
            validate_world(&world),
            Err("NPC 'test_npc' has duplicate topic id 'shared'".to_string())
        );
    }

    #[test]
    fn duplicate_normalized_npc_topic_names_fail_validation() {
        let mut world = valid_world();
        let mut npc = test_npc("test_npc", "Test NPC");
        npc.topics.push(test_topic("first", "Old-Gate"));
        npc.topics.push(test_topic("second", "old gate"));
        world.npcs.push(npc);
        assert_eq!(
            validate_world(&world),
            Err("NPC 'test_npc' has duplicate normalized topic name 'old gate'".to_string())
        );
    }

    #[test]
    fn untargetable_npc_topic_name_fails_validation() {
        let mut world = valid_world();
        let mut npc = test_npc("test_npc", "Test NPC");
        npc.topics.push(test_topic("punctuation", "---"));
        world.npcs.push(npc);
        assert_eq!(
            validate_world(&world),
            Err("Topic 'punctuation' on NPC 'test_npc' has no targetable name".to_string())
        );
    }

    #[test]
    fn valid_declared_fact_passes_validation() {
        let mut world = valid_world();
        world.facts.push(FactDefinition {
            id: "knows_secret".to_string(),
        });

        assert!(validate_world(&world).is_ok());
    }

    #[test]
    fn empty_fact_id_fails_validation() {
        let mut world = valid_world();
        world.facts.push(FactDefinition {
            id: "  ".to_string(),
        });

        assert_eq!(validate_world(&world), Err("Fact has no id".to_string()));
    }

    #[test]
    fn invalid_fact_id_fails_validation() {
        let mut world = valid_world();
        world.facts.push(FactDefinition {
            id: "bad!".to_string(),
        });

        assert_eq!(
            validate_world(&world),
            Err("Fact id has invalid characters: bad!".to_string())
        );
    }

    #[test]
    fn duplicate_fact_ids_fail_validation() {
        let mut world = valid_world();
        world.facts.push(FactDefinition {
            id: "shared_fact".to_string(),
        });
        world.facts.push(FactDefinition {
            id: "shared_fact".to_string(),
        });

        assert_eq!(
            validate_world(&world),
            Err("World has duplicate fact id 'shared_fact'".to_string())
        );
    }

    #[test]
    fn topic_required_fact_must_be_declared() {
        let mut world = valid_world();
        let mut npc = test_npc("test_npc", "Test NPC");
        let mut topic = test_topic("secret", "Secret");
        topic.requires_facts.push("missing_fact".to_string());
        npc.topics.push(topic);
        world.npcs.push(npc);

        assert_eq!(
            validate_world(&world),
            Err(
                "Topic 'secret' on NPC 'test_npc' requires missing fact 'missing_fact'".to_string()
            )
        );
    }

    #[test]
    fn topic_excluded_fact_must_be_declared() {
        let mut world = valid_world();
        let mut npc = test_npc("test_npc", "Test NPC");
        let mut topic = test_topic("secret", "Secret");
        topic.excludes_facts.push("missing_fact".to_string());
        npc.topics.push(topic);
        world.npcs.push(npc);

        assert_eq!(
            validate_world(&world),
            Err(
                "Topic 'secret' on NPC 'test_npc' excludes missing fact 'missing_fact'".to_string()
            )
        );
    }

    #[test]
    fn duplicate_topic_fact_condition_fails_validation() {
        let mut world = valid_world();
        world.facts.push(FactDefinition {
            id: "known_fact".to_string(),
        });
        let mut npc = test_npc("test_npc", "Test NPC");
        let mut topic = test_topic("secret", "Secret");
        topic.requires_facts = vec!["known_fact".to_string(), "known_fact".to_string()];
        npc.topics.push(topic);
        world.npcs.push(npc);

        assert_eq!(
            validate_world(&world),
            Err(
                "Topic 'secret' on NPC 'test_npc' requires fact 'known_fact' more than once"
                    .to_string()
            )
        );
    }

    #[test]
    fn duplicate_excluded_topic_fact_condition_fails_validation() {
        let mut world = valid_world();
        world.facts.push(FactDefinition {
            id: "known_fact".to_string(),
        });
        let mut npc = test_npc("test_npc", "Test NPC");
        let mut topic = test_topic("secret", "Secret");
        topic.excludes_facts = vec!["known_fact".to_string(), "known_fact".to_string()];
        npc.topics.push(topic);
        world.npcs.push(npc);

        assert_eq!(
            validate_world(&world),
            Err(
                "Topic 'secret' on NPC 'test_npc' excludes fact 'known_fact' more than once"
                    .to_string()
            )
        );
    }

    #[test]
    fn topic_granted_fact_must_be_declared() {
        let mut world = valid_world();
        let mut npc = test_npc("test_npc", "Test NPC");
        let mut topic = test_topic("secret", "Secret");
        topic.grants_facts.push("missing_fact".to_string());
        npc.topics.push(topic);
        world.npcs.push(npc);

        assert_eq!(
            validate_world(&world),
            Err("Topic 'secret' on NPC 'test_npc' grants missing fact 'missing_fact'".to_string())
        );
    }

    #[test]
    fn duplicate_granted_topic_fact_fails_validation() {
        let mut world = valid_world();
        world.facts.push(FactDefinition {
            id: "known_fact".to_string(),
        });
        let mut npc = test_npc("test_npc", "Test NPC");
        let mut topic = test_topic("secret", "Secret");
        topic.grants_facts = vec!["known_fact".to_string(), "known_fact".to_string()];
        npc.topics.push(topic);
        world.npcs.push(npc);

        assert_eq!(
            validate_world(&world),
            Err(
                "Topic 'secret' on NPC 'test_npc' grants fact 'known_fact' more than once"
                    .to_string()
            )
        );
    }

    #[test]
    fn topic_may_exclude_and_grant_same_fact() {
        let mut world = valid_world();
        world.facts.push(FactDefinition {
            id: "known_fact".to_string(),
        });
        let mut npc = test_npc("test_npc", "Test NPC");
        let mut topic = test_topic("secret", "Secret");
        topic.excludes_facts.push("known_fact".to_string());
        topic.grants_facts.push("known_fact".to_string());
        npc.topics.push(topic);
        world.npcs.push(npc);

        assert!(validate_world(&world).is_ok());
    }

    fn test_quest(id: &str, name: &str) -> QuestDefinition {
        QuestDefinition {
            id: id.to_string(),
            name: name.to_string(),
            description: "A quest used for testing.".to_string(),
            starting_step: "first_step".to_string(),
            steps: vec![QuestStepDefinition {
                id: "first_step".to_string(),
                description: "Complete the first objective.".to_string(),
                objective: QuestObjectiveDefinition::ReachRoom {
                    room: "start".to_string(),
                },
                next_step: None,
            }],
        }
    }

    #[test]
    fn valid_quest_passes_validation() {
        let mut world = valid_world();
        world.quests.push(test_quest("test_quest", "Test Quest"));

        assert!(validate_world(&world).is_ok());
    }

    #[test]
    fn empty_quest_id_fails_validation() {
        let mut world = valid_world();
        world.quests.push(test_quest("  ", "Test Quest"));

        assert_eq!(
            validate_world(&world),
            Err("Quest 'Test Quest' has no id".to_string())
        );
    }

    #[test]
    fn invalid_quest_id_fails_validation() {
        let mut world = valid_world();
        world.quests.push(test_quest("bad!", "Test Quest"));

        assert_eq!(
            validate_world(&world),
            Err("Id for quest 'Test Quest' has invalid characters: bad!".to_string())
        );
    }

    #[test]
    fn duplicate_quest_ids_fail_validation() {
        let mut world = valid_world();
        world.quests.push(test_quest("test_quest", "First Quest"));
        world.quests.push(test_quest("test_quest", "Second Quest"));

        assert_eq!(
            validate_world(&world),
            Err("The quest named Second Quest has the duplicate id: test_quest".to_string())
        );
    }

    #[test]
    fn quest_name_and_description_must_not_be_empty() {
        let mut unnamed_world = valid_world();
        unnamed_world.quests.push(test_quest("test_quest", "  "));
        assert_eq!(
            validate_world(&unnamed_world),
            Err("Quest 'test_quest' has no name".to_string())
        );

        let mut undescribed_world = valid_world();
        let mut quest = test_quest("test_quest", "Test Quest");
        quest.description = "  ".to_string();
        undescribed_world.quests.push(quest);
        assert_eq!(
            validate_world(&undescribed_world),
            Err("Quest 'test_quest' has no description".to_string())
        );
    }

    #[test]
    fn quest_starting_step_must_not_be_empty() {
        let mut world = valid_world();
        let mut quest = test_quest("test_quest", "Test Quest");
        quest.starting_step = "  ".to_string();
        world.quests.push(quest);

        assert_eq!(
            validate_world(&world),
            Err("Quest 'test_quest' has no starting step".to_string())
        );
    }

    #[test]
    fn invalid_quest_starting_step_id_fails_validation() {
        let mut world = valid_world();
        let mut quest = test_quest("test_quest", "Test Quest");
        quest.starting_step = "bad!".to_string();
        world.quests.push(quest);

        assert_eq!(
            validate_world(&world),
            Err("Starting step for quest 'test_quest' has invalid characters: bad!".to_string())
        );
    }

    #[test]
    fn empty_quest_step_id_fails_validation() {
        let mut world = valid_world();
        let mut quest = test_quest("test_quest", "Test Quest");
        quest.steps[0].id = "  ".to_string();
        world.quests.push(quest);

        assert_eq!(
            validate_world(&world),
            Err("A step on quest 'test_quest' has no id".to_string())
        );
    }

    #[test]
    fn invalid_quest_step_id_fails_validation() {
        let mut world = valid_world();
        let mut quest = test_quest("test_quest", "Test Quest");
        quest.steps[0].id = "bad!".to_string();
        world.quests.push(quest);

        assert_eq!(
            validate_world(&world),
            Err("Step id on quest 'test_quest' has invalid characters: bad!".to_string())
        );
    }

    #[test]
    fn duplicate_quest_step_ids_fail_validation() {
        let mut world = valid_world();
        let mut quest = test_quest("test_quest", "Test Quest");
        quest.steps.push(QuestStepDefinition {
            id: "first_step".to_string(),
            description: "A duplicate step.".to_string(),
            objective: QuestObjectiveDefinition::ReachRoom {
                room: "start".to_string(),
            },
            next_step: None,
        });
        world.quests.push(quest);

        assert_eq!(
            validate_world(&world),
            Err("Quest 'test_quest' has duplicate step id 'first_step'".to_string())
        );
    }

    #[test]
    fn quest_step_description_must_not_be_empty() {
        let mut world = valid_world();
        let mut quest = test_quest("test_quest", "Test Quest");
        quest.steps[0].description = "  ".to_string();
        world.quests.push(quest);

        assert_eq!(
            validate_world(&world),
            Err("Step 'first_step' on quest 'test_quest' has no description".to_string())
        );
    }

    #[test]
    fn quest_starting_step_must_reference_declared_step() {
        let mut world = valid_world();
        let mut quest = test_quest("test_quest", "Test Quest");
        quest.starting_step = "missing_step".to_string();
        world.quests.push(quest);

        assert_eq!(
            validate_world(&world),
            Err("Quest 'test_quest' references missing starting step 'missing_step'".to_string())
        );
    }

    #[test]
    fn reach_room_objective_must_reference_declared_room() {
        let mut world = valid_world();
        let mut quest = test_quest("test_quest", "Test Quest");
        quest.steps[0].objective = QuestObjectiveDefinition::ReachRoom {
            room: "missing_room".to_string(),
        };
        world.quests.push(quest);

        assert_eq!(
            validate_world(&world),
            Err(
                "Step 'first_step' on quest 'test_quest' references missing room 'missing_room'"
                    .to_string()
            )
        );
    }

    #[test]
    fn quest_next_step_must_not_be_empty() {
        let mut world = valid_world();
        let mut quest = test_quest("test_quest", "Test Quest");
        quest.steps[0].next_step = Some("  ".to_string());
        world.quests.push(quest);

        assert_eq!(
            validate_world(&world),
            Err("Step 'first_step' on quest 'test_quest' has an empty next step".to_string())
        );
    }

    #[test]
    fn invalid_quest_next_step_id_fails_validation() {
        let mut world = valid_world();
        let mut quest = test_quest("test_quest", "Test Quest");
        quest.steps[0].next_step = Some("bad!".to_string());
        world.quests.push(quest);

        assert_eq!(
            validate_world(&world),
            Err(
                "Next step for 'first_step' on quest 'test_quest' has invalid characters: bad!"
                    .to_string()
            )
        );
    }

    #[test]
    fn quest_next_step_must_reference_declared_step() {
        let mut world = valid_world();
        let mut quest = test_quest("test_quest", "Test Quest");
        quest.steps[0].next_step = Some("missing_step".to_string());
        world.quests.push(quest);

        assert_eq!(
            validate_world(&world),
            Err("Step 'first_step' on quest 'test_quest' references missing next step 'missing_step'".to_string())
        );
    }

    #[test]
    fn cyclic_quest_steps_fail_validation() {
        let mut world = valid_world();
        let mut quest = test_quest("test_quest", "Test Quest");
        quest.steps[0].next_step = Some("second_step".to_string());
        quest.steps.push(QuestStepDefinition {
            id: "second_step".to_string(),
            description: "Complete the second objective.".to_string(),
            objective: QuestObjectiveDefinition::ReachRoom {
                room: "start".to_string(),
            },
            next_step: Some("first_step".to_string()),
        });
        world.quests.push(quest);

        assert_eq!(
            validate_world(&world),
            Err("Quest 'test_quest' contains a cycle at step 'first_step'".to_string())
        );
    }

    #[test]
    fn unreachable_quest_steps_fail_validation() {
        let mut world = valid_world();
        let mut quest = test_quest("test_quest", "Test Quest");
        quest.steps.push(QuestStepDefinition {
            id: "orphaned_step".to_string(),
            description: "An unreachable objective.".to_string(),
            objective: QuestObjectiveDefinition::ReachRoom {
                room: "start".to_string(),
            },
            next_step: None,
        });
        world.quests.push(quest);

        assert_eq!(
            validate_world(&world),
            Err(
                "Step 'orphaned_step' on quest 'test_quest' is unreachable from the starting step"
                    .to_string()
            )
        );
    }

    #[test]
    fn topic_started_quest_must_be_declared() {
        let mut world = valid_world();
        let mut npc = test_npc("test_npc", "Test NPC");
        let mut topic = test_topic("quest_offer", "Quest Offer");
        topic.starts_quests.push("missing_quest".to_string());
        npc.topics.push(topic);
        world.npcs.push(npc);

        assert_eq!(
            validate_world(&world),
            Err(
                "Topic 'quest_offer' on NPC 'test_npc' starts missing quest 'missing_quest'"
                    .to_string()
            )
        );
    }

    #[test]
    fn duplicate_started_quest_fails_validation() {
        let mut world = valid_world();
        world.quests.push(test_quest("test_quest", "Test Quest"));
        let mut npc = test_npc("test_npc", "Test NPC");
        let mut topic = test_topic("quest_offer", "Quest Offer");
        topic.starts_quests = vec!["test_quest".to_string(), "test_quest".to_string()];
        npc.topics.push(topic);
        world.npcs.push(npc);

        assert_eq!(
            validate_world(&world),
            Err(
                "Topic 'quest_offer' on NPC 'test_npc' starts quest 'test_quest' more than once"
                    .to_string()
            )
        );
    }

    #[test]
    fn contradictory_topic_fact_condition_fails_validation() {
        let mut world = valid_world();
        world.facts.push(FactDefinition {
            id: "known_fact".to_string(),
        });
        let mut npc = test_npc("test_npc", "Test NPC");
        let mut topic = test_topic("secret", "Secret");
        topic.requires_facts.push("known_fact".to_string());
        topic.excludes_facts.push("known_fact".to_string());
        npc.topics.push(topic);
        world.npcs.push(npc);

        assert_eq!(
            validate_world(&world),
            Err(
                "Topic 'secret' on NPC 'test_npc' both requires and excludes fact 'known_fact'"
                    .to_string()
            )
        );
    }
}
