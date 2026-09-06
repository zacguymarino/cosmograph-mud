use std::fs;

use crate::game::world::World;

use super::conversion::convert_world;
use super::definition::WorldDefinition;
use super::validation::validate_world;

pub fn load_world(path: &str) -> Result<World, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("failed to read world '{path}': {error}"))?;

    let definition: WorldDefinition = toml::from_str(&content)
        .map_err(|error| format!("failed to parse world '{path}': {error}"))?;

    validate_world(&definition)?;
    convert_world(definition)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::ids::{FactId, NpcId, QuestId};

    #[test]
    fn origin_world_loads_declared_fact_and_conditional_topic() {
        let world = load_world("worlds/origin/world.toml").expect("Origin world should load");

        assert!(world.declares_fact(&FactId("heard_about_old_observatory".to_string())));
        let mara = world
            .npc(&NpcId("mara_voss".to_string()))
            .expect("Mara should exist");
        let topic = mara
            .topics
            .iter()
            .find(|topic| topic.id.0 == "old_observatory")
            .expect("conditional topic should exist");

        assert_eq!(
            topic.requires_facts,
            vec![FactId("heard_about_old_observatory".to_string())]
        );
        assert!(topic.excludes_facts.is_empty());

        let lights = mara
            .topics
            .iter()
            .find(|topic| topic.id.0 == "strange_lights")
            .expect("fact-granting topic should exist");
        assert_eq!(
            lights.grants_facts,
            vec![FactId("heard_about_old_observatory".to_string())]
        );
        assert_eq!(lights.excludes_facts, lights.grants_facts);

        let quest = world
            .quest(&QuestId("lights_in_the_old_observatory".to_string()))
            .expect("observatory quest should exist");
        assert_eq!(quest.name, "Lights in the Old Observatory");
        assert_eq!(
            topic.starts_quests,
            vec![QuestId("lights_in_the_old_observatory".to_string())]
        );
    }
}
