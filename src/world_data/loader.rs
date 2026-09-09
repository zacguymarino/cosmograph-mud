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
    use crate::game::direction::Direction;
    use crate::game::exit::ExitRequirement;
    use crate::game::ids::ItemId;
    use crate::game::ids::RoomId;
    use crate::game::ids::{FactId, NpcId, QuestId, QuestStepId};
    use crate::game::quest::QuestObjective;

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
            quest.starting_step,
            QuestStepId("reach_old_observatory".to_string())
        );
        assert_eq!(
            quest
                .step(&QuestStepId("reach_old_observatory".to_string()))
                .map(|step| step.description.as_str()),
            Some("Find the old observatory.")
        );
        assert_eq!(
            quest
                .step(&QuestStepId("reach_old_observatory".to_string()))
                .map(|step| &step.objective),
            Some(&QuestObjective::ReachRoom(RoomId(
                "old_observatory".to_string()
            )))
        );
        let report_step = quest
            .step(&QuestStepId("report_to_mara".to_string()))
            .expect("report step should exist");
        assert_eq!(
            report_step.objective,
            QuestObjective::AskTopic {
                npc_id: NpcId("mara_voss".to_string()),
                topic_id: crate::game::ids::NpcTopicId("old_observatory".to_string()),
            }
        );
        assert_eq!(
            topic.starts_quests,
            vec![QuestId("lights_in_the_old_observatory".to_string())]
        );

        let forest = world
            .room(&RoomId("forest_path".to_string()))
            .expect("Forest Path should exist");
        let observatory_exit = forest
            .exits
            .get(&Direction::East)
            .expect("Forest Path should lead east to the observatory");
        assert_eq!(
            observatory_exit.destination,
            RoomId("old_observatory".to_string())
        );
        assert_eq!(
            observatory_exit.requirement,
            Some(ExitRequirement::CarryingItem(ItemId(
                "rusty_key".to_string()
            )))
        );
        assert_eq!(
            observatory_exit.failure_message.as_deref(),
            Some("The observatory door is locked. Its corroded keyhole looks oddly familiar.")
        );
    }
}
