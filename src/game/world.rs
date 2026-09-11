use std::collections::{HashMap, HashSet};

use super::feature::RoomFeature;
use super::ids::{FactId, FeatureId, ItemId, NpcId, QuestId, RoomId, WorldId};
use super::item::Item;
use super::item_location::{ItemLocation, ItemPlacement};
use super::npc::Npc;
use super::quest::Quest;
use super::room::Room;

#[derive(Debug)]
pub struct World {
    pub id: WorldId,
    pub name: String,
    pub starting_room: RoomId,
    pub facts: HashSet<FactId>,
    pub quests: HashMap<QuestId, Quest>,
    pub items: HashMap<ItemId, Item>,
    pub item_placements: Vec<ItemPlacement>,
    pub features: HashMap<FeatureId, RoomFeature>,
    pub npcs: HashMap<NpcId, Npc>,
    pub rooms: HashMap<RoomId, Room>,
}

impl World {
    pub fn room(&self, id: &RoomId) -> Option<&Room> {
        self.rooms.get(id)
    }

    pub fn room_mut(&mut self, id: &RoomId) -> Option<&mut Room> {
        self.rooms.get_mut(id)
    }

    pub fn item(&self, id: &ItemId) -> Option<&Item> {
        self.items.get(id)
    }

    pub fn item_location(&self, id: &ItemId) -> Option<&ItemLocation> {
        self.item_placements
            .iter()
            .find(|placement| &placement.item_id == id)
            .map(|placement| &placement.location)
    }

    pub fn item_ids_in_room<'a>(
        &'a self,
        room_id: &'a RoomId,
    ) -> impl Iterator<Item = &'a ItemId> + 'a {
        self.item_placements.iter().filter_map(move |placement| {
            matches!(&placement.location, ItemLocation::Room(id) if id == room_id)
                .then_some(&placement.item_id)
        })
    }

    pub fn item_ids_carried_by<'a>(
        &'a self,
        character_id: &'a super::ids::CharacterId,
    ) -> impl Iterator<Item = &'a ItemId> + 'a {
        self.item_placements.iter().filter_map(move |placement| {
            matches!(&placement.location, ItemLocation::CarriedBy(id) if id == character_id)
                .then_some(&placement.item_id)
        })
    }

    pub fn item_ids_on_feature<'a>(
        &'a self,
        feature_id: &'a FeatureId,
    ) -> impl Iterator<Item = &'a ItemId> + 'a {
        self.item_placements.iter().filter_map(move |placement| {
            matches!(&placement.location, ItemLocation::OnFeature(id) if id == feature_id)
                .then_some(&placement.item_id)
        })
    }

    pub fn item_ids_accessible_in_room<'a>(
        &'a self,
        room: &'a Room,
    ) -> impl Iterator<Item = &'a ItemId> + 'a {
        self.item_placements.iter().filter_map(move |placement| {
            let accessible = match &placement.location {
                ItemLocation::Room(room_id) => room_id == &room.id,
                ItemLocation::OnFeature(feature_id) => room.features.contains(feature_id),
                ItemLocation::Nowhere | ItemLocation::CarriedBy(_) => false,
            };
            accessible.then_some(&placement.item_id)
        })
    }

    pub fn move_item(&mut self, item_id: &ItemId, location: ItemLocation) -> bool {
        let Some(position) = self
            .item_placements
            .iter()
            .position(|placement| &placement.item_id == item_id)
        else {
            return false;
        };
        let mut placement = self.item_placements.remove(position);
        placement.location = location;
        self.item_placements.push(placement);
        true
    }

    pub fn feature(&self, id: &FeatureId) -> Option<&RoomFeature> {
        self.features.get(id)
    }

    pub fn npc(&self, id: &NpcId) -> Option<&Npc> {
        self.npcs.get(id)
    }

    pub fn declares_fact(&self, id: &FactId) -> bool {
        self.facts.contains(id)
    }

    pub fn quest(&self, id: &QuestId) -> Option<&Quest> {
        self.quests.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::ids::QuestStepId;
    use crate::game::quest::{QuestObjective, QuestStep};
    use std::collections::HashMap;

    fn valid_room() -> Room {
        Room {
            id: RoomId("start".to_string()),
            name: "Starting Room".to_string(),
            description: "A test room.".to_string(),
            features: vec![],
            npcs: vec![],
            exits: HashMap::new(),
        }
    }

    fn valid_world() -> World {
        let mut rooms = HashMap::new();
        let room = valid_room();
        rooms.insert(room.id.clone(), room);

        let item = Item {
            id: ItemId("test_item".to_string()),
            name: "Test Item".to_string(),
            description: "An item used for testing.".to_string(),
        };
        let mut items = HashMap::new();
        items.insert(item.id.clone(), item);

        let feature = RoomFeature {
            id: FeatureId("test_feature".to_string()),
            name: "Test Feature".to_string(),
            room_description: "A test feature stands here.".to_string(),
            description: "A feature used for testing.".to_string(),
            supporter: None,
        };

        let mut features = HashMap::new();
        features.insert(feature.id.clone(), feature);

        let npc = Npc {
            id: NpcId("test_npc".to_string()),
            name: "Test NPC".to_string(),
            room_description: "A test NPC stands here.".to_string(),
            description: "An NPC used for testing.".to_string(),
            greeting: "Hello from the test NPC.".to_string(),
            topics: vec![],
        };
        let mut npcs = HashMap::new();
        npcs.insert(npc.id.clone(), npc);

        World {
            id: WorldId("test_world".to_string()),
            name: "Test World".to_string(),
            starting_room: RoomId("start".to_string()),
            facts: HashSet::new(),
            quests: HashMap::new(),
            items,
            item_placements: vec![ItemPlacement {
                item_id: ItemId("test_item".to_string()),
                location: ItemLocation::Nowhere,
            }],
            features,
            npcs,
            rooms,
        }
    }

    #[test]
    fn existing_room_returns_borrowed_room() {
        let world = valid_world();
        let room = world
            .room(&RoomId("start".to_string()))
            .expect("starting room should exist");

        assert_eq!(room.name, "Starting Room");
    }

    #[test]
    fn missing_room_returns_none() {
        let world = valid_world();
        assert!(world.room(&RoomId("missing".to_string())).is_none());
    }

    #[test]
    fn existing_item_returns_borrowed_item() {
        let world = valid_world();

        let item = world
            .item(&ItemId("test_item".to_string()))
            .expect("Item should exist");

        assert_eq!(item.name, "Test Item");
    }

    #[test]
    fn missing_item_returns_none() {
        let world = valid_world();

        assert!(world.item(&ItemId("missing".to_string())).is_none());
    }

    #[test]
    fn item_location_can_be_changed() {
        let mut world = valid_world();
        assert!(world.move_item(
            &ItemId("test_item".to_string()),
            ItemLocation::Room(RoomId("start".to_string()))
        ));
        let character_id = crate::game::ids::CharacterId("player".to_string());
        assert!(world.move_item(
            &ItemId("test_item".to_string()),
            ItemLocation::CarriedBy(character_id.clone())
        ));
        assert_eq!(
            world.item_location(&ItemId("test_item".to_string())),
            Some(&ItemLocation::CarriedBy(character_id))
        );
        assert_eq!(world.item_placements.len(), 1);
    }

    #[test]
    fn existing_feature_returns_borrowed_feature() {
        let world = valid_world();

        let feature = world
            .feature(&FeatureId("test_feature".to_string()))
            .expect("feature should exist");

        assert_eq!(feature.name, "Test Feature");
    }

    #[test]
    fn missing_feature_returns_none() {
        let world = valid_world();

        assert!(world.feature(&FeatureId("missing".to_string())).is_none());
    }

    #[test]
    fn existing_npc_returns_borrowed_npc() {
        let world = valid_world();

        let npc = world
            .npc(&NpcId("test_npc".to_string()))
            .expect("NPC should exist");

        assert_eq!(npc.name, "Test NPC");
    }

    #[test]
    fn missing_npc_returns_none() {
        let world = valid_world();
        assert!(world.npc(&NpcId("missing".to_string())).is_none());
    }

    #[test]
    fn declared_fact_can_be_found() {
        let mut world = valid_world();
        world.facts.insert(FactId("known_fact".to_string()));

        assert!(world.declares_fact(&FactId("known_fact".to_string())));
        assert!(!world.declares_fact(&FactId("missing_fact".to_string())));
    }

    #[test]
    fn existing_quest_returns_borrowed_quest() {
        let mut world = valid_world();
        let quest = Quest {
            id: QuestId("test_quest".to_string()),
            name: "Test Quest".to_string(),
            description: "A quest used for testing.".to_string(),
            starting_step: QuestStepId("first_step".to_string()),
            steps: vec![QuestStep {
                id: QuestStepId("first_step".to_string()),
                description: "Complete the first objective.".to_string(),
                objective: QuestObjective::ReachRoom(RoomId("start".to_string())),
                next_step: None,
            }],
        };
        world.quests.insert(quest.id.clone(), quest);

        assert_eq!(
            world
                .quest(&QuestId("test_quest".to_string()))
                .map(|quest| quest.name.as_str()),
            Some("Test Quest")
        );
        assert!(world.quest(&QuestId("missing".to_string())).is_none());
        assert_eq!(
            world
                .quest(&QuestId("test_quest".to_string()))
                .and_then(|quest| quest.step(&QuestStepId("first_step".to_string())))
                .map(|step| step.description.as_str()),
            Some("Complete the first objective.")
        );
    }
}
