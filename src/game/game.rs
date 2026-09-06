use super::character::Character;
use super::command::{AskQuery, Command, TargetKind, TargetQuery};
use super::direction::Direction;
use super::event::{
    AskFailureReason, DropFailureReason, ExamineFailureReason, ExaminedFeature, ExaminedItem,
    ExaminedNpc, GameEvent, ObservedFeature, ObservedItem, ObservedNpc, ObservedQuest,
    TakeFailureReason, TalkFailureReason,
};
use super::ids::{FactKey, FeatureId, ItemId, NpcId, QuestKey};
use super::naming::normalize_name;
use super::npc::NpcTopic;
use super::quest::{QuestProgress, QuestStatus};
use super::world::World;

#[derive(Debug)]
pub struct Game {
    pub world: World,
    pub character: Character,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExamineTarget {
    Item(ItemId),
    Feature(FeatureId),
    Npc(NpcId),
}

impl Game {
    pub fn new(world: World, character: Character) -> Self {
        Self { world, character }
    }

    fn topic_is_available(&self, topic: &NpcTopic) -> bool {
        let character_has_fact = |fact_id: &super::ids::FactId| {
            self.character.facts.contains(&FactKey {
                world_id: self.world.id.clone(),
                fact_id: fact_id.clone(),
            })
        };

        topic.requires_facts.iter().all(character_has_fact)
            && topic
                .excludes_facts
                .iter()
                .all(|fact_id| !character_has_fact(fact_id))
    }

    fn available_topic_names(&self, topics: &[NpcTopic]) -> Vec<String> {
        topics
            .iter()
            .filter(|topic| self.topic_is_available(topic))
            .map(|topic| topic.name.clone())
            .collect()
    }

    fn observe_current_room(&self) -> Option<GameEvent> {
        let room = self.world.room(&self.character.current_room)?;

        let items = room
            .items
            .iter()
            .map(|item_id| {
                let item = self.world.item(item_id)?;

                Some(ObservedItem {
                    id: item.id.clone(),
                    name: item.name.clone(),
                })
            })
            .collect::<Option<Vec<_>>>()?;

        let features = room
            .features
            .iter()
            .map(|feature_id| {
                let feature = self.world.feature(feature_id)?;

                Some(ObservedFeature {
                    id: feature.id.clone(),
                    name: feature.name.clone(),
                    room_description: feature.room_description.clone(),
                })
            })
            .collect::<Option<Vec<_>>>()?;

        let npcs = room
            .npcs
            .iter()
            .map(|npc_id| {
                let npc = self.world.npc(npc_id)?;
                Some(ObservedNpc {
                    id: npc.id.clone(),
                    name: npc.name.clone(),
                    room_description: npc.room_description.clone(),
                })
            })
            .collect::<Option<Vec<_>>>()?;

        Some(GameEvent::RoomObserved {
            room_id: room.id.clone(),
            name: room.name.clone(),
            description: room.description.clone(),
            items,
            features,
            npcs,
            exits: room.exits.keys().cloned().collect(),
        })
    }

    fn observe_current_exits(&self) -> Option<GameEvent> {
        let room = self.world.room(&self.character.current_room)?;

        Some(GameEvent::ExitsObserved {
            room_id: room.id.clone(),
            exits: room.exits.keys().cloned().collect(),
        })
    }

    fn attempt_move(&mut self, direction: Direction) -> Vec<GameEvent> {
        let current_room = self.character.current_room.clone();
        let Some(room) = self.world.room(&current_room) else {
            return vec![GameEvent::MovementFailed {
                character_id: self.character.id.clone(),
                room_id: current_room,
                direction,
            }];
        };

        let destination = room.exits.get(&direction).cloned();
        let Some(destination) = destination else {
            return vec![GameEvent::MovementFailed {
                character_id: self.character.id.clone(),
                room_id: current_room,
                direction,
            }];
        };

        self.character.current_room = destination.clone();

        let movement_event = GameEvent::CharacterMoved {
            character_id: self.character.id.clone(),
            from: current_room,
            to: destination,
            direction,
        };

        let mut events = vec![movement_event];

        if let Some(room_observed) = self.observe_current_room() {
            events.push(room_observed);
        }

        events
    }

    fn matching_room_items(&self, query: &str) -> Vec<ItemId> {
        let Some(room) = self.world.room(&self.character.current_room) else {
            return vec![];
        };

        let normalized_query = normalize_name(query);

        room.items
            .iter()
            .filter(|item_id| {
                self.world
                    .item(item_id)
                    .is_some_and(|item| normalize_name(&item.name) == normalized_query)
            })
            .cloned()
            .collect()
    }

    fn attempt_take(&mut self, query: impl Into<TargetQuery>) -> Vec<GameEvent> {
        let mut query = query.into();
        query.name = normalize_name(&query.name);
        let room_id = self.character.current_room.clone();
        let matches = self.matching_room_items(&query.name);
        let query_name = query.name;

        let item_id = if let Some(ordinal) = query.ordinal {
            let requested = ordinal.get();
            let Some(item_id) = matches.get(requested - 1) else {
                return vec![GameEvent::TakeFailed {
                    character_id: self.character.id.clone(),
                    room_id,
                    query: query_name,
                    reason: TakeFailureReason::OrdinalOutOfRange {
                        requested,
                        available: matches.len(),
                    },
                }];
            };
            item_id.clone()
        } else {
            match matches.as_slice() {
                [] => {
                    return vec![GameEvent::TakeFailed {
                        character_id: self.character.id.clone(),
                        room_id,
                        query: query_name,
                        reason: TakeFailureReason::NotFound,
                    }];
                }
                [item_id] => item_id.clone(),
                _ => {
                    return vec![GameEvent::TakeFailed {
                        character_id: self.character.id.clone(),
                        room_id,
                        query: query_name,
                        reason: TakeFailureReason::Ambiguous {
                            match_count: matches.len(),
                        },
                    }];
                }
            }
        };

        let Some(item) = self.world.item(&item_id) else {
            return vec![GameEvent::TakeFailed {
                character_id: self.character.id.clone(),
                room_id,
                query: query_name.clone(),
                reason: TakeFailureReason::NotFound,
            }];
        };

        let observed_item = ObservedItem {
            id: item.id.clone(),
            name: item.name.clone(),
        };

        let Some(room) = self.world.room_mut(&room_id) else {
            return vec![GameEvent::TakeFailed {
                character_id: self.character.id.clone(),
                room_id,
                query: query_name.clone(),
                reason: TakeFailureReason::NotFound,
            }];
        };

        let Some(position) = room
            .items
            .iter()
            .position(|candidate| candidate == &item_id)
        else {
            return vec![GameEvent::TakeFailed {
                character_id: self.character.id.clone(),
                room_id,
                query: query_name,
                reason: TakeFailureReason::NotFound,
            }];
        };

        room.items.remove(position);
        self.character.inventory.push(item_id);

        vec![GameEvent::ItemTaken {
            character_id: self.character.id.clone(),
            room_id,
            item: observed_item,
        }]
    }

    fn observe_inventory(&self) -> Option<GameEvent> {
        let items = self
            .character
            .inventory
            .iter()
            .map(|item_id| {
                let item = self.world.item(item_id)?;

                Some(ObservedItem {
                    id: item.id.clone(),
                    name: item.name.clone(),
                })
            })
            .collect::<Option<Vec<_>>>()?;

        Some(GameEvent::InventoryObserved { items })
    }

    fn observe_quests(&self) -> GameEvent {
        let mut active = Vec::new();
        let mut completed = Vec::new();

        for (key, progress) in &self.character.quests {
            if key.world_id != self.world.id {
                continue;
            }
            let Some(quest) = self.world.quest(&key.quest_id) else {
                continue;
            };
            let observed = ObservedQuest {
                key: key.clone(),
                name: quest.name.clone(),
                description: quest.description.clone(),
            };
            match progress.status {
                QuestStatus::Active => active.push(observed),
                QuestStatus::Completed => completed.push(observed),
            }
        }

        active.sort_by(|left, right| left.name.cmp(&right.name));
        completed.sort_by(|left, right| left.name.cmp(&right.name));

        GameEvent::QuestsObserved { active, completed }
    }

    fn matching_inventory_items(&self, query: &str) -> Vec<ItemId> {
        let normalized_query = normalize_name(query);

        self.character
            .inventory
            .iter()
            .filter(|item_id| {
                self.world
                    .item(item_id)
                    .is_some_and(|item| normalize_name(&item.name) == normalized_query)
            })
            .cloned()
            .collect()
    }

    fn attempt_drop(&mut self, query: impl Into<TargetQuery>) -> Vec<GameEvent> {
        let mut query = query.into();
        query.name = normalize_name(&query.name);
        let room_id = self.character.current_room.clone();
        let matches = self.matching_inventory_items(&query.name);
        let query_name = query.name;

        let item_id = if let Some(ordinal) = query.ordinal {
            let requested = ordinal.get();
            let Some(item_id) = matches.get(requested - 1) else {
                return vec![GameEvent::DropFailed {
                    character_id: self.character.id.clone(),
                    room_id,
                    query: query_name,
                    reason: DropFailureReason::OrdinalOutOfRange {
                        requested,
                        available: matches.len(),
                    },
                }];
            };
            item_id.clone()
        } else {
            match matches.as_slice() {
                [] => {
                    return vec![GameEvent::DropFailed {
                        character_id: self.character.id.clone(),
                        room_id,
                        query: query_name,
                        reason: DropFailureReason::NotFound,
                    }];
                }
                [item_id] => item_id.clone(),
                _ => {
                    return vec![GameEvent::DropFailed {
                        character_id: self.character.id.clone(),
                        room_id,
                        query: query_name,
                        reason: DropFailureReason::Ambiguous {
                            match_count: matches.len(),
                        },
                    }];
                }
            }
        };

        let Some(item) = self.world.item(&item_id) else {
            return vec![GameEvent::DropFailed {
                character_id: self.character.id.clone(),
                room_id,
                query: query_name.clone(),
                reason: DropFailureReason::NotFound,
            }];
        };

        let observed_item = ObservedItem {
            id: item.id.clone(),
            name: item.name.clone(),
        };

        let Some(position) = self
            .character
            .inventory
            .iter()
            .position(|candidate| candidate == &item_id)
        else {
            return vec![GameEvent::DropFailed {
                character_id: self.character.id.clone(),
                room_id,
                query: query_name.clone(),
                reason: DropFailureReason::NotFound,
            }];
        };

        let Some(room) = self.world.room_mut(&room_id) else {
            return vec![GameEvent::DropFailed {
                character_id: self.character.id.clone(),
                room_id,
                query: query_name,
                reason: DropFailureReason::NotFound,
            }];
        };

        let dropped_item_id = self.character.inventory.remove(position);
        room.items.push(dropped_item_id);

        vec![GameEvent::ItemDropped {
            character_id: self.character.id.clone(),
            room_id,
            item: observed_item,
        }]
    }

    fn matching_room_features(&self, query: &str) -> Vec<FeatureId> {
        let Some(room) = self.world.room(&self.character.current_room) else {
            return vec![];
        };

        let normalized_query = normalize_name(query);

        room.features
            .iter()
            .filter(|feature_id| {
                self.world
                    .feature(feature_id)
                    .is_some_and(|feature| normalize_name(&feature.name) == normalized_query)
            })
            .cloned()
            .collect()
    }

    fn matching_accessible_items(&self, query: &str) -> Vec<ItemId> {
        let mut matches = self.matching_room_items(query);
        matches.extend(self.matching_inventory_items(query));
        matches
    }

    fn matching_room_npcs(&self, query: &str) -> Vec<NpcId> {
        let Some(room) = self.world.room(&self.character.current_room) else {
            return vec![];
        };
        let normalized_query = normalize_name(query);
        room.npcs
            .iter()
            .filter(|npc_id| {
                self.world
                    .npc(npc_id)
                    .is_some_and(|npc| normalize_name(&npc.name) == normalized_query)
            })
            .cloned()
            .collect()
    }

    fn attempt_talk(&self, query: impl Into<TargetQuery>) -> Vec<GameEvent> {
        let mut query = query.into();
        query.name = normalize_name(&query.name);
        let matches = self.matching_room_npcs(&query.name);
        let query_name = query.name;

        let npc_id = if let Some(ordinal) = query.ordinal {
            let requested = ordinal.get();
            let Some(npc_id) = matches.get(requested - 1) else {
                return vec![GameEvent::TalkFailed {
                    character_id: self.character.id.clone(),
                    query: query_name,
                    reason: TalkFailureReason::OrdinalOutOfRange {
                        requested,
                        available: matches.len(),
                    },
                }];
            };
            npc_id
        } else {
            match matches.as_slice() {
                [] => {
                    return vec![GameEvent::TalkFailed {
                        character_id: self.character.id.clone(),
                        query: query_name,
                        reason: TalkFailureReason::NotFound,
                    }];
                }
                [npc_id] => npc_id,
                _ => {
                    return vec![GameEvent::TalkFailed {
                        character_id: self.character.id.clone(),
                        query: query_name,
                        reason: TalkFailureReason::Ambiguous {
                            match_count: matches.len(),
                        },
                    }];
                }
            }
        };

        let Some(npc) = self.world.npc(npc_id) else {
            return vec![GameEvent::TalkFailed {
                character_id: self.character.id.clone(),
                query: query_name,
                reason: TalkFailureReason::NotFound,
            }];
        };

        vec![GameEvent::NpcSpoke {
            character_id: self.character.id.clone(),
            npc_id: npc.id.clone(),
            name: npc.name.clone(),
            greeting: npc.greeting.clone(),
            topics: self.available_topic_names(&npc.topics),
        }]
    }

    fn attempt_ask(&mut self, mut query: AskQuery) -> Vec<GameEvent> {
        query.npc.name = normalize_name(&query.npc.name);
        query.topic = normalize_name(&query.topic);
        let matches = self.matching_room_npcs(&query.npc.name);
        let npc_query = query.npc.name;
        let topic_query = query.topic;

        let npc_id = if let Some(ordinal) = query.npc.ordinal {
            let requested = ordinal.get();
            let Some(npc_id) = matches.get(requested - 1) else {
                return vec![GameEvent::AskFailed {
                    character_id: self.character.id.clone(),
                    npc_query,
                    topic_query,
                    reason: AskFailureReason::NpcOrdinalOutOfRange {
                        requested,
                        available: matches.len(),
                    },
                }];
            };
            npc_id
        } else {
            match matches.as_slice() {
                [] => {
                    return vec![GameEvent::AskFailed {
                        character_id: self.character.id.clone(),
                        npc_query,
                        topic_query,
                        reason: AskFailureReason::NpcNotFound,
                    }];
                }
                [npc_id] => npc_id,
                _ => {
                    return vec![GameEvent::AskFailed {
                        character_id: self.character.id.clone(),
                        npc_query,
                        topic_query,
                        reason: AskFailureReason::NpcAmbiguous {
                            match_count: matches.len(),
                        },
                    }];
                }
            }
        };

        let Some(npc) = self.world.npc(npc_id) else {
            return vec![GameEvent::AskFailed {
                character_id: self.character.id.clone(),
                npc_query,
                topic_query,
                reason: AskFailureReason::NpcNotFound,
            }];
        };

        let Some(topic) = npc.topics.iter().find(|topic| {
            self.topic_is_available(topic) && normalize_name(&topic.name) == topic_query
        }) else {
            return vec![GameEvent::AskFailed {
                character_id: self.character.id.clone(),
                npc_query,
                topic_query,
                reason: AskFailureReason::TopicNotFound {
                    npc_name: npc.name.clone(),
                },
            }];
        };

        let npc_id = npc.id.clone();
        let npc_name = npc.name.clone();
        let topic = topic.clone();
        let previous_topics = self.available_topic_names(&npc.topics);

        let mut learned_facts = Vec::new();
        for fact_id in &topic.grants_facts {
            let fact = FactKey {
                world_id: self.world.id.clone(),
                fact_id: fact_id.clone(),
            };
            if self.character.facts.insert(fact.clone()) {
                learned_facts.push(fact);
            }
        }

        let current_topics = self
            .world
            .npc(&npc_id)
            .map(|npc| self.available_topic_names(&npc.topics))
            .unwrap_or_default();
        let updated_topics = (previous_topics != current_topics).then_some(current_topics);

        let mut events = vec![GameEvent::NpcAnswered {
            character_id: self.character.id.clone(),
            npc_id,
            npc_name,
            topic_id: topic.id.clone(),
            topic_name: topic.name.clone(),
            response: topic.response.clone(),
            learned_facts,
            updated_topics,
        }];

        for quest_id in &topic.starts_quests {
            let key = QuestKey {
                world_id: self.world.id.clone(),
                quest_id: quest_id.clone(),
            };
            if self.character.quests.contains_key(&key) {
                continue;
            }
            let Some(quest) = self.world.quest(quest_id) else {
                continue;
            };

            self.character.quests.insert(
                key.clone(),
                QuestProgress {
                    status: QuestStatus::Active,
                },
            );
            events.push(GameEvent::QuestStarted {
                character_id: self.character.id.clone(),
                quest: ObservedQuest {
                    key,
                    name: quest.name.clone(),
                    description: quest.description.clone(),
                },
            });
        }

        events
    }

    fn matching_examine_targets(&self, query: &TargetQuery) -> Vec<ExamineTarget> {
        let mut matches = Vec::new();

        if !matches!(query.kind, Some(TargetKind::Feature | TargetKind::Npc)) {
            matches.extend(
                self.matching_accessible_items(&query.name)
                    .into_iter()
                    .map(ExamineTarget::Item),
            );
        }

        if !matches!(query.kind, Some(TargetKind::Item | TargetKind::Npc)) {
            matches.extend(
                self.matching_room_features(&query.name)
                    .into_iter()
                    .map(ExamineTarget::Feature),
            );
        }

        if !matches!(query.kind, Some(TargetKind::Item | TargetKind::Feature)) {
            matches.extend(
                self.matching_room_npcs(&query.name)
                    .into_iter()
                    .map(ExamineTarget::Npc),
            );
        }

        matches
    }

    fn attempt_examine(&self, query: impl Into<TargetQuery>) -> Vec<GameEvent> {
        let mut query = query.into();
        query.name = normalize_name(&query.name);
        let matches = self.matching_examine_targets(&query);
        let query_name = query.name;

        let target = if let Some(ordinal) = query.ordinal {
            let requested = ordinal.get();
            let Some(target) = matches.get(requested - 1) else {
                return vec![GameEvent::ExamineFailed {
                    character_id: self.character.id.clone(),
                    query: query_name,
                    reason: ExamineFailureReason::OrdinalOutOfRange {
                        requested,
                        available: matches.len(),
                    },
                }];
            };
            target
        } else {
            match matches.as_slice() {
                [] => {
                    return vec![GameEvent::ExamineFailed {
                        character_id: self.character.id.clone(),
                        query: query_name,
                        reason: ExamineFailureReason::NotFound,
                    }];
                }
                [target] => target,
                _ => {
                    let kinds = matches
                        .iter()
                        .map(|target| match target {
                            ExamineTarget::Item(_) => TargetKind::Item,
                            ExamineTarget::Feature(_) => TargetKind::Feature,
                            ExamineTarget::Npc(_) => TargetKind::Npc,
                        })
                        .collect();

                    return vec![GameEvent::ExamineFailed {
                        character_id: self.character.id.clone(),
                        query: query_name,
                        reason: ExamineFailureReason::Ambiguous { kinds },
                    }];
                }
            }
        };

        match target {
            ExamineTarget::Item(item_id) => {
                let Some(item) = self.world.item(item_id) else {
                    return vec![GameEvent::ExamineFailed {
                        character_id: self.character.id.clone(),
                        query: query_name,
                        reason: ExamineFailureReason::NotFound,
                    }];
                };

                vec![GameEvent::ItemExamined {
                    character_id: self.character.id.clone(),
                    item: ExaminedItem {
                        id: item.id.clone(),
                        name: item.name.clone(),
                        description: item.description.clone(),
                    },
                }]
            }
            ExamineTarget::Feature(feature_id) => {
                let Some(feature) = self.world.feature(feature_id) else {
                    return vec![GameEvent::ExamineFailed {
                        character_id: self.character.id.clone(),
                        query: query_name,
                        reason: ExamineFailureReason::NotFound,
                    }];
                };

                vec![GameEvent::FeatureExamined {
                    character_id: self.character.id.clone(),
                    feature: ExaminedFeature {
                        id: feature.id.clone(),
                        name: feature.name.clone(),
                        description: feature.description.clone(),
                    },
                }]
            }
            ExamineTarget::Npc(npc_id) => {
                let Some(npc) = self.world.npc(npc_id) else {
                    return vec![GameEvent::ExamineFailed {
                        character_id: self.character.id.clone(),
                        query: query_name,
                        reason: ExamineFailureReason::NotFound,
                    }];
                };
                vec![GameEvent::NpcExamined {
                    character_id: self.character.id.clone(),
                    npc: ExaminedNpc {
                        id: npc.id.clone(),
                        name: npc.name.clone(),
                        description: npc.description.clone(),
                    },
                }]
            }
        }
    }

    pub fn process(&mut self, command: Command) -> Vec<GameEvent> {
        match command {
            Command::Look => self.observe_current_room().into_iter().collect(),
            Command::Exits => self.observe_current_exits().into_iter().collect(),
            Command::Help => vec![GameEvent::HelpRequested],
            Command::Move(direction) => self.attempt_move(direction),
            Command::Take(query) => self.attempt_take(query),
            Command::Inventory => self.observe_inventory().into_iter().collect(),
            Command::Quests => vec![self.observe_quests()],
            Command::Drop(query) => self.attempt_drop(query),
            Command::Examine(query) => self.attempt_examine(query),
            Command::Talk(query) => self.attempt_talk(query),
            Command::Ask(query) => self.attempt_ask(query),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::feature::RoomFeature;
    use crate::game::ids::{
        CharacterId, FactId, FactKey, FeatureId, ItemId, NpcId, NpcTopicId, QuestId, QuestKey,
        RoomId, WorldId,
    };
    use crate::game::item::Item;
    use crate::game::npc::{Npc, NpcTopic};
    use crate::game::quest::{Quest, QuestProgress, QuestStatus};
    use crate::game::room::Room;
    use std::collections::{HashMap, HashSet};
    use std::num::NonZeroUsize;

    fn game_with_character_in(current_room: &str) -> Game {
        let mut start_exits = HashMap::new();
        start_exits.insert(Direction::North, RoomId("next_room".to_string()));

        let room = Room {
            id: RoomId("start".to_string()),
            name: "Starting Room".to_string(),
            description: "A test room.".to_string(),
            items: vec![ItemId("test_item".to_string())],
            features: vec![FeatureId("test_feature".to_string())],
            npcs: vec![NpcId("test_npc".to_string())],
            exits: start_exits,
        };

        let next_room = Room {
            id: RoomId("next_room".to_string()),
            name: "Next Room".to_string(),
            description: "Another test room.".to_string(),
            items: vec![],
            features: vec![],
            npcs: vec![],
            exits: HashMap::new(),
        };

        let mut rooms = HashMap::new();
        rooms.insert(room.id.clone(), room);
        rooms.insert(next_room.id.clone(), next_room);

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
        };

        let mut features = HashMap::new();
        features.insert(feature.id.clone(), feature);

        let npc = Npc {
            id: NpcId("test_npc".to_string()),
            name: "Test NPC".to_string(),
            room_description: "A test NPC stands here.".to_string(),
            description: "An NPC used for testing.".to_string(),
            greeting: "Hello from the test NPC.".to_string(),
            topics: vec![NpcTopic {
                id: NpcTopicId("test_topic".to_string()),
                name: "Test Topic".to_string(),
                response: "This is the test topic response.".to_string(),
                requires_facts: vec![],
                excludes_facts: vec![],
                grants_facts: vec![],
                starts_quests: vec![],
            }],
        };
        let mut npcs = HashMap::new();
        npcs.insert(npc.id.clone(), npc);

        let quest = Quest {
            id: QuestId("test_quest".to_string()),
            name: "Test Quest".to_string(),
            description: "A quest used for testing.".to_string(),
        };
        let mut quests = HashMap::new();
        quests.insert(quest.id.clone(), quest);

        let world = World {
            id: WorldId("test_world".to_string()),
            name: "Test World".to_string(),
            starting_room: RoomId("start".to_string()),
            facts: HashSet::new(),
            quests,
            items,
            features,
            npcs,
            rooms,
        };

        let character = Character {
            id: CharacterId("player".to_string()),
            name: "Player".to_string(),
            current_room: RoomId(current_room.to_string()),
            inventory: vec![],
            facts: HashSet::new(),
            quests: HashMap::new(),
        };

        Game::new(world, character)
    }

    fn add_conditional_topic(
        game: &mut Game,
        requires_facts: Vec<FactId>,
        excludes_facts: Vec<FactId>,
    ) {
        game.world
            .npcs
            .get_mut(&NpcId("test_npc".to_string()))
            .expect("test NPC should exist")
            .topics
            .push(NpcTopic {
                id: NpcTopicId("secret_topic".to_string()),
                name: "Secret Topic".to_string(),
                response: "A conditional response.".to_string(),
                requires_facts,
                excludes_facts,
                grants_facts: vec![],
                starts_quests: vec![],
            });
    }

    fn give_fact(game: &mut Game, world_id: &str, fact_id: &str) {
        game.character.facts.insert(FactKey {
            world_id: WorldId(world_id.to_string()),
            fact_id: FactId(fact_id.to_string()),
        });
    }

    fn set_topic_grants(game: &mut Game, topic_id: &str, grants_facts: Vec<FactId>) {
        game.world
            .npcs
            .get_mut(&NpcId("test_npc".to_string()))
            .expect("test NPC should exist")
            .topics
            .iter_mut()
            .find(|topic| topic.id == NpcTopicId(topic_id.to_string()))
            .expect("test topic should exist")
            .grants_facts = grants_facts;
    }

    fn set_topic_starts_quests(game: &mut Game, topic_id: &str, starts_quests: Vec<QuestId>) {
        game.world
            .npcs
            .get_mut(&NpcId("test_npc".to_string()))
            .expect("test NPC should exist")
            .topics
            .iter_mut()
            .find(|topic| topic.id == NpcTopicId(topic_id.to_string()))
            .expect("test topic should exist")
            .starts_quests = starts_quests;
    }

    #[test]
    fn observing_existing_room_returns_event() {
        let game = game_with_character_in("start");

        assert_eq!(
            game.observe_current_room(),
            Some(GameEvent::RoomObserved {
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
                npcs: vec![ObservedNpc {
                    id: NpcId("test_npc".to_string()),
                    name: "Test NPC".to_string(),
                    room_description: "A test NPC stands here.".to_string(),
                }],
                exits: vec![Direction::North],
            })
        )
    }

    #[test]
    fn observing_missing_room_returns_none() {
        let game = game_with_character_in("missing");

        assert_eq!(game.observe_current_room(), None);
    }

    #[test]
    fn movement_through_existing_exit_succeeds() {
        let mut game = game_with_character_in("start");

        let events = game.attempt_move(Direction::North);

        assert!(matches!(
            events.as_slice(),
            [
                GameEvent::CharacterMoved {
                    to,
                    direction: Direction::North,
                    ..
                },
                GameEvent::RoomObserved { room_id, .. }
            ] if to == &RoomId("next_room".to_string())
                && room_id == &RoomId("next_room".to_string())
        ));

        assert_eq!(game.character.current_room, RoomId("next_room".to_string()));
    }

    #[test]
    fn movement_without_exit_fails_without_changing_room() {
        let mut game = game_with_character_in("start");

        let events = game.attempt_move(Direction::East);

        assert_eq!(
            events,
            vec![GameEvent::MovementFailed {
                character_id: CharacterId("player".to_string()),
                room_id: RoomId("start".to_string()),
                direction: Direction::East,
            }]
        );

        assert_eq!(game.character.current_room, RoomId("start".to_string()));
    }

    #[test]
    fn look_command_observes_current_room() {
        let mut game = game_with_character_in("start");

        let events = game.process(Command::Look);

        assert!(matches!(
            events.as_slice(),
            [GameEvent::RoomObserved { .. }]
        ));
    }

    #[test]
    fn exits_command_observes_current_room_exits() {
        let mut game = game_with_character_in("start");

        assert_eq!(
            game.process(Command::Exits),
            vec![GameEvent::ExitsObserved {
                room_id: RoomId("start".to_string()),
                exits: vec![Direction::North],
            }]
        );
    }

    #[test]
    fn exits_command_with_missing_room_produces_no_event() {
        let mut game = game_with_character_in("missing");
        assert!(game.process(Command::Exits).is_empty());
    }

    #[test]
    fn help_command_requests_help() {
        let mut game = game_with_character_in("start");
        assert_eq!(game.process(Command::Help), vec![GameEvent::HelpRequested]);
    }

    #[test]
    fn move_command_attempts_movement() {
        let mut game = game_with_character_in("start");

        let events = game.process(Command::Move(Direction::North));

        assert!(matches!(
            events.as_slice(),
            [
                GameEvent::CharacterMoved { .. },
                GameEvent::RoomObserved { .. }
            ]
        ));

        assert_eq!(game.character.current_room, RoomId("next_room".to_string()));
    }

    #[test]
    fn room_items_can_be_found_by_normalized_name() {
        let game = game_with_character_in("start");

        assert_eq!(
            game.matching_room_items("  TEST   item "),
            vec![ItemId("test_item".to_string())]
        );
    }

    #[test]
    fn taking_unique_item_moves_it_to_inventory() {
        let mut game = game_with_character_in("start");

        let events = game.attempt_take("  TEST   item ".to_string());

        assert_eq!(
            events,
            vec![GameEvent::ItemTaken {
                character_id: CharacterId("player".to_string()),
                room_id: RoomId("start".to_string()),
                item: ObservedItem {
                    id: ItemId("test_item".to_string()),
                    name: "Test Item".to_string(),
                },
            }]
        );

        assert_eq!(
            game.character.inventory,
            vec![ItemId("test_item".to_string())]
        );

        assert!(
            game.world
                .room(&RoomId("start".to_string()))
                .expect("starting room should exist")
                .items
                .is_empty()
        );
    }

    #[test]
    fn taking_missing_item_does_not_change_state() {
        let mut game = game_with_character_in("start");

        let events = game.attempt_take("missing item".to_string());

        assert_eq!(
            events,
            vec![GameEvent::TakeFailed {
                character_id: CharacterId("player".to_string()),
                room_id: RoomId("start".to_string()),
                query: "missing item".to_string(),
                reason: TakeFailureReason::NotFound,
            }]
        );

        assert!(game.character.inventory.is_empty());

        assert_eq!(
            game.world
                .room(&RoomId("start".to_string()))
                .expect("starting room should exist")
                .items,
            vec![ItemId("test_item".to_string())]
        );
    }

    #[test]
    fn taking_ambiguous_item_does_not_change_state() {
        let mut game = game_with_character_in("start");

        let second_item = Item {
            id: ItemId("second_item".to_string()),
            name: "Test Item".to_string(),
            description: "Another item with the same name.".to_string(),
        };

        game.world.items.insert(second_item.id.clone(), second_item);

        game.world
            .room_mut(&RoomId("start".to_string()))
            .expect("starting room should exist")
            .items
            .push(ItemId("second_item".to_string()));

        let events = game.attempt_take("test item".to_string());

        assert_eq!(
            events,
            vec![GameEvent::TakeFailed {
                character_id: CharacterId("player".to_string()),
                room_id: RoomId("start".to_string()),
                query: "test item".to_string(),
                reason: TakeFailureReason::Ambiguous { match_count: 2 },
            }]
        );

        assert!(game.character.inventory.is_empty());

        assert_eq!(
            game.world
                .room(&RoomId("start".to_string()))
                .expect("starting room should exist")
                .items
                .len(),
            2
        );
    }

    #[test]
    fn take_command_attempts_to_take_item() {
        let mut game = game_with_character_in("start");

        let events = game.process(Command::Take("test item".to_string().into()));

        assert!(matches!(events.as_slice(), [GameEvent::ItemTaken { .. }]));

        assert_eq!(
            game.character.inventory,
            vec![ItemId("test_item".to_string())]
        );
    }

    #[test]
    fn inventory_observation_reports_carried_items() {
        let mut game = game_with_character_in("start");

        game.attempt_take("test item".to_string());

        assert_eq!(
            game.observe_inventory(),
            Some(GameEvent::InventoryObserved {
                items: vec![ObservedItem {
                    id: ItemId("test_item".to_string()),
                    name: "Test Item".to_string(),
                }],
            })
        );
    }

    #[test]
    fn inventory_command_observes_carried_items() {
        let mut game = game_with_character_in("start");

        game.attempt_take("test item".to_string());

        let events = game.process(Command::Inventory);

        assert_eq!(
            events,
            vec![GameEvent::InventoryObserved {
                items: vec![ObservedItem {
                    id: ItemId("test_item".to_string()),
                    name: "Test Item".to_string(),
                }],
            }]
        );
    }

    #[test]
    fn inventory_items_can_be_found_by_normalized_name() {
        let mut game = game_with_character_in("start");

        game.attempt_take("test item".to_string());

        assert_eq!(
            game.matching_inventory_items("  TEST   item "),
            vec![ItemId("test_item".to_string())]
        );
    }

    #[test]
    fn dropping_unique_item_moves_it_to_room() {
        let mut game = game_with_character_in("start");
        game.attempt_take("test item".to_string());

        let events = game.attempt_drop("  TEST   item ".to_string());

        assert_eq!(
            events,
            vec![GameEvent::ItemDropped {
                character_id: CharacterId("player".to_string()),
                room_id: RoomId("start".to_string()),
                item: ObservedItem {
                    id: ItemId("test_item".to_string()),
                    name: "Test Item".to_string(),
                },
            }]
        );

        assert!(game.character.inventory.is_empty());

        assert_eq!(
            game.world
                .room(&RoomId("start".to_string()))
                .expect("starting room should exist")
                .items,
            vec![ItemId("test_item".to_string())]
        );
    }

    #[test]
    fn dropping_missing_item_does_not_change_state() {
        let mut game = game_with_character_in("start");

        let events = game.attempt_drop("missing item".to_string());

        assert_eq!(
            events,
            vec![GameEvent::DropFailed {
                character_id: CharacterId("player".to_string()),
                room_id: RoomId("start".to_string()),
                query: "missing item".to_string(),
                reason: DropFailureReason::NotFound,
            }]
        );

        assert!(game.character.inventory.is_empty());

        assert_eq!(
            game.world
                .room(&RoomId("start".to_string()))
                .expect("starting room should exist")
                .items,
            vec![ItemId("test_item".to_string())]
        );
    }

    #[test]
    fn dropping_ambiguous_item_does_not_change_state() {
        let mut game = game_with_character_in("start");
        game.attempt_take("test item".to_string());

        let second_item = Item {
            id: ItemId("second_item".to_string()),
            name: "Test Item".to_string(),
            description: "Another item with the same name.".to_string(),
        };

        game.world.items.insert(second_item.id.clone(), second_item);

        game.character
            .inventory
            .push(ItemId("second_item".to_string()));

        let events = game.attempt_drop("test item".to_string());

        assert_eq!(
            events,
            vec![GameEvent::DropFailed {
                character_id: CharacterId("player".to_string()),
                room_id: RoomId("start".to_string()),
                query: "test item".to_string(),
                reason: DropFailureReason::Ambiguous { match_count: 2 },
            }]
        );

        assert_eq!(game.character.inventory.len(), 2);

        assert!(
            game.world
                .room(&RoomId("start".to_string()))
                .expect("starting room should exist")
                .items
                .is_empty()
        );
    }

    #[test]
    fn drop_command_attempts_to_drop_item() {
        let mut game = game_with_character_in("start");
        game.attempt_take("test item".to_string());

        let events = game.process(Command::Drop("test item".to_string().into()));

        assert!(matches!(events.as_slice(), [GameEvent::ItemDropped { .. }]));

        assert!(game.character.inventory.is_empty());

        assert_eq!(
            game.world
                .room(&RoomId("start".to_string()))
                .expect("starting room should exist")
                .items,
            vec![ItemId("test_item".to_string())]
        );
    }

    #[test]
    fn accessible_items_include_room_and_inventory_items() {
        let mut game = game_with_character_in("start");

        assert_eq!(
            game.matching_accessible_items("test item"),
            vec![ItemId("test_item".to_string())]
        );

        game.attempt_take("test item".to_string());

        assert_eq!(
            game.matching_accessible_items("test item"),
            vec![ItemId("test_item".to_string())]
        );
    }

    #[test]
    fn examining_room_item_returns_description() {
        let game = game_with_character_in("start");

        assert_eq!(
            game.attempt_examine("  TEST   item ".to_string()),
            vec![GameEvent::ItemExamined {
                character_id: CharacterId("player".to_string()),
                item: ExaminedItem {
                    id: ItemId("test_item".to_string()),
                    name: "Test Item".to_string(),
                    description: "An item used for testing.".to_string(),
                },
            }]
        );
    }

    #[test]
    fn examining_inventory_item_returns_description() {
        let mut game = game_with_character_in("start");
        game.attempt_take("test item".to_string());

        assert_eq!(
            game.attempt_examine("test item".to_string()),
            vec![GameEvent::ItemExamined {
                character_id: CharacterId("player".to_string()),
                item: ExaminedItem {
                    id: ItemId("test_item".to_string()),
                    name: "Test Item".to_string(),
                    description: "An item used for testing.".to_string(),
                },
            }]
        );
    }

    #[test]
    fn examining_missing_item_fails() {
        let game = game_with_character_in("start");

        assert_eq!(
            game.attempt_examine("missing item".to_string()),
            vec![GameEvent::ExamineFailed {
                character_id: CharacterId("player".to_string()),
                query: "missing item".to_string(),
                reason: ExamineFailureReason::NotFound,
            }]
        );
    }

    #[test]
    fn examining_ambiguous_item_fails() {
        let mut game = game_with_character_in("start");

        let second_item = Item {
            id: ItemId("second_item".to_string()),
            name: "Test Item".to_string(),
            description: "Another item with the same name.".to_string(),
        };

        game.world.items.insert(second_item.id.clone(), second_item);

        game.character
            .inventory
            .push(ItemId("second_item".to_string()));

        assert_eq!(
            game.attempt_examine("test item".to_string()),
            vec![GameEvent::ExamineFailed {
                character_id: CharacterId("player".to_string()),
                query: "test item".to_string(),
                reason: ExamineFailureReason::Ambiguous {
                    kinds: vec![TargetKind::Item, TargetKind::Item],
                },
            }]
        );
    }

    #[test]
    fn examine_command_attempts_examination() {
        let mut game = game_with_character_in("start");

        let events = game.process(Command::Examine("test item".to_string().into()));

        assert!(matches!(
            events.as_slice(),
            [GameEvent::ItemExamined { .. }]
        ));
    }

    #[test]
    fn room_features_can_be_found_by_normalized_name() {
        let game = game_with_character_in("start");

        assert_eq!(
            game.matching_room_features("  TEST   feature "),
            vec![FeatureId("test_feature".to_string())]
        );
    }

    #[test]
    fn examining_room_feature_returns_description() {
        let game = game_with_character_in("start");

        assert_eq!(
            game.attempt_examine("test feature".to_string()),
            vec![GameEvent::FeatureExamined {
                character_id: CharacterId("player".to_string()),
                feature: ExaminedFeature {
                    id: FeatureId("test_feature".to_string()),
                    name: "Test Feature".to_string(),
                    description: "A feature used for testing.".to_string(),
                },
            }]
        );
    }

    #[test]
    fn item_and_feature_with_same_name_are_ambiguous_when_examined() {
        let mut game = game_with_character_in("start");
        game.world
            .features
            .get_mut(&FeatureId("test_feature".to_string()))
            .expect("test feature should exist")
            .name = "Test Item".to_string();

        assert_eq!(
            game.attempt_examine("test item".to_string()),
            vec![GameEvent::ExamineFailed {
                character_id: CharacterId("player".to_string()),
                query: "test item".to_string(),
                reason: ExamineFailureReason::Ambiguous {
                    kinds: vec![TargetKind::Item, TargetKind::Feature],
                },
            }]
        );
    }

    #[test]
    fn item_qualifier_resolves_shared_name_to_item() {
        let mut game = game_with_character_in("start");
        game.world
            .features
            .get_mut(&FeatureId("test_feature".to_string()))
            .expect("test feature should exist")
            .name = "Test Item".to_string();

        assert!(matches!(
            game.attempt_examine(TargetQuery::item("test item".to_string()))
                .as_slice(),
            [GameEvent::ItemExamined { .. }]
        ));
    }

    #[test]
    fn feature_qualifier_resolves_shared_name_to_feature() {
        let mut game = game_with_character_in("start");
        game.world
            .features
            .get_mut(&FeatureId("test_feature".to_string()))
            .expect("test feature should exist")
            .name = "Test Item".to_string();

        assert!(matches!(
            game.attempt_examine(TargetQuery::feature("test item".to_string()))
                .as_slice(),
            [GameEvent::FeatureExamined { .. }]
        ));
    }

    #[test]
    fn room_feature_cannot_be_taken() {
        let mut game = game_with_character_in("start");

        assert_eq!(
            game.attempt_take("test feature".to_string()),
            vec![GameEvent::TakeFailed {
                character_id: CharacterId("player".to_string()),
                room_id: RoomId("start".to_string()),
                query: "test feature".to_string(),
                reason: TakeFailureReason::NotFound,
            }]
        );
    }

    #[test]
    fn numbered_take_selects_requested_item_instance() {
        let mut game = game_with_character_in("start");
        let second_item = Item {
            id: ItemId("second_item".to_string()),
            name: "Test Item".to_string(),
            description: "The second matching item.".to_string(),
        };
        game.world.items.insert(second_item.id.clone(), second_item);
        game.world
            .room_mut(&RoomId("start".to_string()))
            .expect("starting room should exist")
            .items
            .push(ItemId("second_item".to_string()));

        game.attempt_take(
            TargetQuery::item("test item".to_string())
                .with_ordinal(NonZeroUsize::new(2).expect("2 is nonzero")),
        );

        assert_eq!(
            game.character.inventory,
            vec![ItemId("second_item".to_string())]
        );
        assert_eq!(
            game.world
                .room(&RoomId("start".to_string()))
                .expect("starting room should exist")
                .items,
            vec![ItemId("test_item".to_string())]
        );
    }

    #[test]
    fn numbered_drop_selects_requested_item_instance() {
        let mut game = game_with_character_in("start");
        let second_item = Item {
            id: ItemId("second_item".to_string()),
            name: "Test Item".to_string(),
            description: "The second matching item.".to_string(),
        };
        game.world.items.insert(second_item.id.clone(), second_item);
        game.character.inventory = vec![
            ItemId("test_item".to_string()),
            ItemId("second_item".to_string()),
        ];

        game.attempt_drop(
            TargetQuery::item("test item".to_string())
                .with_ordinal(NonZeroUsize::new(2).expect("2 is nonzero")),
        );

        assert_eq!(
            game.character.inventory,
            vec![ItemId("test_item".to_string())]
        );
        assert!(
            game.world
                .room(&RoomId("start".to_string()))
                .expect("starting room should exist")
                .items
                .contains(&ItemId("second_item".to_string()))
        );
    }

    #[test]
    fn numbered_examine_selects_across_target_kinds() {
        let mut game = game_with_character_in("start");
        game.world
            .features
            .get_mut(&FeatureId("test_feature".to_string()))
            .expect("test feature should exist")
            .name = "Test Item".to_string();

        assert!(matches!(
            game.attempt_examine(
                TargetQuery::any("test item".to_string())
                    .with_ordinal(NonZeroUsize::new(2).expect("2 is nonzero"))
            )
            .as_slice(),
            [GameEvent::FeatureExamined { .. }]
        ));
    }

    #[test]
    fn npc_can_be_examined_by_qualified_name() {
        let game = game_with_character_in("start");

        assert_eq!(
            game.attempt_examine(TargetQuery::npc("test npc".to_string())),
            vec![GameEvent::NpcExamined {
                character_id: CharacterId("player".to_string()),
                npc: ExaminedNpc {
                    id: NpcId("test_npc".to_string()),
                    name: "Test NPC".to_string(),
                    description: "An NPC used for testing.".to_string(),
                },
            }]
        );
    }

    #[test]
    fn npc_in_another_room_is_not_accessible() {
        let game = game_with_character_in("next_room");

        assert!(matches!(
            game.attempt_examine(TargetQuery::npc("test npc".to_string()))
                .as_slice(),
            [GameEvent::ExamineFailed {
                reason: ExamineFailureReason::NotFound,
                ..
            }]
        ));
    }

    #[test]
    fn unqualified_examine_reports_npc_ambiguity_in_candidate_order() {
        let mut game = game_with_character_in("start");
        game.world
            .npcs
            .get_mut(&NpcId("test_npc".to_string()))
            .expect("test NPC should exist")
            .name = "Test Item".to_string();

        assert_eq!(
            game.attempt_examine(TargetQuery::any("test item".to_string())),
            vec![GameEvent::ExamineFailed {
                character_id: CharacterId("player".to_string()),
                query: "test item".to_string(),
                reason: ExamineFailureReason::Ambiguous {
                    kinds: vec![TargetKind::Item, TargetKind::Npc],
                },
            }]
        );
    }

    #[test]
    fn talking_to_nearby_npc_returns_authored_greeting() {
        let game = game_with_character_in("start");

        assert_eq!(
            game.attempt_talk(TargetQuery::npc("test npc".to_string())),
            vec![GameEvent::NpcSpoke {
                character_id: CharacterId("player".to_string()),
                npc_id: NpcId("test_npc".to_string()),
                name: "Test NPC".to_string(),
                greeting: "Hello from the test NPC.".to_string(),
                topics: vec!["Test Topic".to_string()],
            }]
        );
    }

    #[test]
    fn talking_to_npc_in_another_room_fails() {
        let game = game_with_character_in("next_room");

        assert_eq!(
            game.attempt_talk(TargetQuery::npc("test npc".to_string())),
            vec![GameEvent::TalkFailed {
                character_id: CharacterId("player".to_string()),
                query: "test npc".to_string(),
                reason: TalkFailureReason::NotFound,
            }]
        );
    }

    #[test]
    fn talking_to_duplicate_npc_name_is_ambiguous() {
        let mut game = game_with_character_in("start");
        let second_npc = Npc {
            id: NpcId("second_npc".to_string()),
            name: "Test NPC".to_string(),
            room_description: "Another test NPC stands here.".to_string(),
            description: "Another NPC used for testing.".to_string(),
            greeting: "Hello from the second NPC.".to_string(),
            topics: vec![],
        };
        game.world.npcs.insert(second_npc.id.clone(), second_npc);
        game.world
            .room_mut(&RoomId("start".to_string()))
            .expect("starting room should exist")
            .npcs
            .push(NpcId("second_npc".to_string()));

        assert_eq!(
            game.attempt_talk(TargetQuery::npc("test npc".to_string())),
            vec![GameEvent::TalkFailed {
                character_id: CharacterId("player".to_string()),
                query: "test npc".to_string(),
                reason: TalkFailureReason::Ambiguous { match_count: 2 },
            }]
        );
    }

    #[test]
    fn numbered_talk_selects_requested_npc() {
        let mut game = game_with_character_in("start");
        let second_npc = Npc {
            id: NpcId("second_npc".to_string()),
            name: "Test NPC".to_string(),
            room_description: "Another test NPC stands here.".to_string(),
            description: "Another NPC used for testing.".to_string(),
            greeting: "Hello from the second NPC.".to_string(),
            topics: vec![],
        };
        game.world.npcs.insert(second_npc.id.clone(), second_npc);
        game.world
            .room_mut(&RoomId("start".to_string()))
            .expect("starting room should exist")
            .npcs
            .push(NpcId("second_npc".to_string()));

        assert!(matches!(
            game.attempt_talk(
                TargetQuery::npc("test npc".to_string())
                    .with_ordinal(NonZeroUsize::new(2).expect("2 is nonzero"))
            )
            .as_slice(),
            [GameEvent::NpcSpoke { npc_id, .. }]
                if npc_id == &NpcId("second_npc".to_string())
        ));
    }

    #[test]
    fn talk_command_attempts_conversation() {
        let mut game = game_with_character_in("start");
        assert!(matches!(
            game.process(Command::Talk(TargetQuery::npc("test npc".to_string())))
                .as_slice(),
            [GameEvent::NpcSpoke { .. }]
        ));
    }

    #[test]
    fn out_of_range_talk_reports_available_npcs() {
        let game = game_with_character_in("start");

        assert_eq!(
            game.attempt_talk(
                TargetQuery::npc("test npc".to_string())
                    .with_ordinal(NonZeroUsize::new(2).expect("2 is nonzero"))
            ),
            vec![GameEvent::TalkFailed {
                character_id: CharacterId("player".to_string()),
                query: "test npc".to_string(),
                reason: TalkFailureReason::OrdinalOutOfRange {
                    requested: 2,
                    available: 1,
                },
            }]
        );
    }

    #[test]
    fn asking_nearby_npc_about_known_topic_returns_response() {
        let mut game = game_with_character_in("start");
        assert_eq!(
            game.attempt_ask(AskQuery {
                npc: TargetQuery::npc("test npc".to_string()),
                topic: "  TEST-TOPIC ".to_string()
            }),
            vec![GameEvent::NpcAnswered {
                character_id: CharacterId("player".to_string()),
                npc_id: NpcId("test_npc".to_string()),
                npc_name: "Test NPC".to_string(),
                topic_id: NpcTopicId("test_topic".to_string()),
                topic_name: "Test Topic".to_string(),
                response: "This is the test topic response.".to_string(),
                learned_facts: vec![],
                updated_topics: None,
            }]
        );
    }

    #[test]
    fn topic_without_fact_conditions_remains_available() {
        let mut game = game_with_character_in("start");

        assert!(matches!(
            game.attempt_ask(AskQuery {
                npc: TargetQuery::npc("test npc".to_string()),
                topic: "test topic".to_string(),
            })
            .as_slice(),
            [GameEvent::NpcAnswered { .. }]
        ));
    }

    #[test]
    fn required_fact_controls_topic_discovery_and_resolution() {
        let mut game = game_with_character_in("start");
        add_conditional_topic(&mut game, vec![FactId("knows_secret".to_string())], vec![]);

        assert!(matches!(
            game.attempt_talk(TargetQuery::npc("test npc".to_string())).as_slice(),
            [GameEvent::NpcSpoke { topics, .. }]
                if topics == &["Test Topic".to_string()]
        ));
        assert!(matches!(
            game.attempt_ask(AskQuery {
                npc: TargetQuery::npc("test npc".to_string()),
                topic: "secret topic".to_string(),
            })
            .as_slice(),
            [GameEvent::AskFailed {
                reason: AskFailureReason::TopicNotFound { .. },
                ..
            }]
        ));

        give_fact(&mut game, "test_world", "knows_secret");

        assert!(matches!(
            game.attempt_talk(TargetQuery::npc("test npc".to_string())).as_slice(),
            [GameEvent::NpcSpoke { topics, .. }]
                if topics == &["Test Topic".to_string(), "Secret Topic".to_string()]
        ));
        assert!(matches!(
            game.attempt_ask(AskQuery {
                npc: TargetQuery::npc("test npc".to_string()),
                topic: "secret topic".to_string(),
            })
            .as_slice(),
            [GameEvent::NpcAnswered { topic_id, .. }]
                if topic_id == &NpcTopicId("secret_topic".to_string())
        ));
    }

    #[test]
    fn all_required_facts_must_be_present() {
        let mut game = game_with_character_in("start");
        add_conditional_topic(
            &mut game,
            vec![
                FactId("first_fact".to_string()),
                FactId("second_fact".to_string()),
            ],
            vec![],
        );
        give_fact(&mut game, "test_world", "first_fact");

        assert!(matches!(
            game.attempt_ask(AskQuery {
                npc: TargetQuery::npc("test npc".to_string()),
                topic: "secret topic".to_string(),
            })
            .as_slice(),
            [GameEvent::AskFailed { .. }]
        ));

        give_fact(&mut game, "test_world", "second_fact");
        assert!(matches!(
            game.attempt_ask(AskQuery {
                npc: TargetQuery::npc("test npc".to_string()),
                topic: "secret topic".to_string(),
            })
            .as_slice(),
            [GameEvent::NpcAnswered { .. }]
        ));
    }

    #[test]
    fn excluded_fact_hides_an_otherwise_available_topic() {
        let mut game = game_with_character_in("start");
        add_conditional_topic(
            &mut game,
            vec![],
            vec![FactId("secret_expired".to_string())],
        );

        assert!(matches!(
            game.attempt_ask(AskQuery {
                npc: TargetQuery::npc("test npc".to_string()),
                topic: "secret topic".to_string(),
            })
            .as_slice(),
            [GameEvent::NpcAnswered { .. }]
        ));

        give_fact(&mut game, "test_world", "secret_expired");
        assert!(matches!(
            game.attempt_ask(AskQuery {
                npc: TargetQuery::npc("test npc".to_string()),
                topic: "secret topic".to_string(),
            })
            .as_slice(),
            [GameEvent::AskFailed {
                reason: AskFailureReason::TopicNotFound { .. },
                ..
            }]
        ));
    }

    #[test]
    fn same_local_fact_from_another_world_does_not_unlock_topic() {
        let mut game = game_with_character_in("start");
        add_conditional_topic(&mut game, vec![FactId("knows_secret".to_string())], vec![]);
        give_fact(&mut game, "another_world", "knows_secret");

        assert!(matches!(
            game.attempt_ask(AskQuery {
                npc: TargetQuery::npc("test npc".to_string()),
                topic: "secret topic".to_string(),
            })
            .as_slice(),
            [GameEvent::AskFailed { .. }]
        ));
    }

    #[test]
    fn reading_dialogue_does_not_change_character_facts() {
        let mut game = game_with_character_in("start");
        give_fact(&mut game, "test_world", "existing_fact");
        let original_facts = game.character.facts.clone();

        game.attempt_talk(TargetQuery::npc("test npc".to_string()));
        game.attempt_ask(AskQuery {
            npc: TargetQuery::npc("test npc".to_string()),
            topic: "test topic".to_string(),
        });

        assert_eq!(game.character.facts, original_facts);
    }

    #[test]
    fn answering_topic_grants_world_qualified_facts() {
        let mut game = game_with_character_in("start");
        set_topic_grants(
            &mut game,
            "test_topic",
            vec![
                FactId("first_fact".to_string()),
                FactId("second_fact".to_string()),
            ],
        );

        let events = game.attempt_ask(AskQuery {
            npc: TargetQuery::npc("test npc".to_string()),
            topic: "test topic".to_string(),
        });

        let expected_facts = vec![
            FactKey {
                world_id: WorldId("test_world".to_string()),
                fact_id: FactId("first_fact".to_string()),
            },
            FactKey {
                world_id: WorldId("test_world".to_string()),
                fact_id: FactId("second_fact".to_string()),
            },
        ];
        assert!(
            expected_facts
                .iter()
                .all(|fact| game.character.facts.contains(fact))
        );
        assert!(matches!(
            events.as_slice(),
            [GameEvent::NpcAnswered {
                learned_facts,
                updated_topics: None,
                ..
            }] if learned_facts == &expected_facts
        ));
    }

    #[test]
    fn learned_fact_refreshes_topics_when_availability_changes() {
        let mut game = game_with_character_in("start");
        let fact_id = FactId("knows_secret".to_string());
        set_topic_grants(&mut game, "test_topic", vec![fact_id.clone()]);
        game.world
            .npcs
            .get_mut(&NpcId("test_npc".to_string()))
            .unwrap()
            .topics[0]
            .excludes_facts
            .push(fact_id.clone());
        add_conditional_topic(&mut game, vec![fact_id], vec![]);

        let events = game.attempt_ask(AskQuery {
            npc: TargetQuery::npc("test npc".to_string()),
            topic: "test topic".to_string(),
        });

        assert!(matches!(
            events.as_slice(),
            [GameEvent::NpcAnswered {
                updated_topics: Some(topics),
                ..
            }] if topics == &["Secret Topic".to_string()]
        ));
    }

    #[test]
    fn repeated_fact_grant_reports_only_newly_learned_facts() {
        let mut game = game_with_character_in("start");
        set_topic_grants(
            &mut game,
            "test_topic",
            vec![FactId("known_fact".to_string())],
        );
        give_fact(&mut game, "test_world", "known_fact");

        let events = game.attempt_ask(AskQuery {
            npc: TargetQuery::npc("test npc".to_string()),
            topic: "test topic".to_string(),
        });

        assert!(matches!(
            events.as_slice(),
            [GameEvent::NpcAnswered {
                learned_facts,
                updated_topics: None,
                ..
            }] if learned_facts.is_empty()
        ));
        assert_eq!(game.character.facts.len(), 1);
    }

    #[test]
    fn unavailable_topic_does_not_grant_facts() {
        let mut game = game_with_character_in("start");
        add_conditional_topic(&mut game, vec![FactId("required_fact".to_string())], vec![]);
        set_topic_grants(
            &mut game,
            "secret_topic",
            vec![FactId("forbidden_grant".to_string())],
        );

        let events = game.attempt_ask(AskQuery {
            npc: TargetQuery::npc("test npc".to_string()),
            topic: "secret topic".to_string(),
        });

        assert!(matches!(events.as_slice(), [GameEvent::AskFailed { .. }]));
        assert!(game.character.facts.is_empty());
    }

    #[test]
    fn quests_command_reports_no_started_quests() {
        let mut game = game_with_character_in("start");

        assert_eq!(
            game.process(Command::Quests),
            vec![GameEvent::QuestsObserved {
                active: vec![],
                completed: vec![],
            }]
        );
    }

    #[test]
    fn quest_observation_separates_active_and_completed_progress() {
        let mut game = game_with_character_in("start");
        let key = QuestKey {
            world_id: WorldId("test_world".to_string()),
            quest_id: QuestId("test_quest".to_string()),
        };
        game.character.quests.insert(
            key.clone(),
            QuestProgress {
                status: QuestStatus::Completed,
            },
        );

        assert_eq!(
            game.process(Command::Quests),
            vec![GameEvent::QuestsObserved {
                active: vec![],
                completed: vec![ObservedQuest {
                    key,
                    name: "Test Quest".to_string(),
                    description: "A quest used for testing.".to_string(),
                }],
            }]
        );
    }

    #[test]
    fn dialogue_starts_world_qualified_quest() {
        let mut game = game_with_character_in("start");
        set_topic_starts_quests(
            &mut game,
            "test_topic",
            vec![QuestId("test_quest".to_string())],
        );

        let events = game.attempt_ask(AskQuery {
            npc: TargetQuery::npc("test npc".to_string()),
            topic: "test topic".to_string(),
        });
        let key = QuestKey {
            world_id: WorldId("test_world".to_string()),
            quest_id: QuestId("test_quest".to_string()),
        };

        assert_eq!(
            game.character.quests.get(&key),
            Some(&QuestProgress {
                status: QuestStatus::Active,
            })
        );
        assert!(matches!(
            events.as_slice(),
            [GameEvent::NpcAnswered { .. }, GameEvent::QuestStarted { quest, .. }]
                if quest.key == key && quest.name == "Test Quest"
        ));
    }

    #[test]
    fn repeating_dialogue_does_not_restart_quest() {
        let mut game = game_with_character_in("start");
        set_topic_starts_quests(
            &mut game,
            "test_topic",
            vec![QuestId("test_quest".to_string())],
        );
        let query = || AskQuery {
            npc: TargetQuery::npc("test npc".to_string()),
            topic: "test topic".to_string(),
        };

        game.attempt_ask(query());
        let repeated_events = game.attempt_ask(query());

        assert_eq!(game.character.quests.len(), 1);
        assert!(matches!(
            repeated_events.as_slice(),
            [GameEvent::NpcAnswered { .. }]
        ));
    }

    #[test]
    fn unavailable_dialogue_does_not_start_quest() {
        let mut game = game_with_character_in("start");
        add_conditional_topic(&mut game, vec![FactId("required_fact".to_string())], vec![]);
        set_topic_starts_quests(
            &mut game,
            "secret_topic",
            vec![QuestId("test_quest".to_string())],
        );

        let events = game.attempt_ask(AskQuery {
            npc: TargetQuery::npc("test npc".to_string()),
            topic: "secret topic".to_string(),
        });

        assert!(matches!(events.as_slice(), [GameEvent::AskFailed { .. }]));
        assert!(game.character.quests.is_empty());
    }

    #[test]
    fn asking_about_unknown_topic_reports_selected_npc() {
        let mut game = game_with_character_in("start");
        assert!(matches!(
            game.attempt_ask(AskQuery {
                npc: TargetQuery::npc("test npc".to_string()),
                topic: "missing topic".to_string()
            })
            .as_slice(),
            [GameEvent::AskFailed {
                reason: AskFailureReason::TopicNotFound { .. },
                ..
            }]
        ));
    }

    #[test]
    fn asking_npc_in_another_room_fails_before_topic_lookup() {
        let mut game = game_with_character_in("next_room");
        assert!(matches!(
            game.attempt_ask(AskQuery {
                npc: TargetQuery::npc("test npc".to_string()),
                topic: "test topic".to_string()
            })
            .as_slice(),
            [GameEvent::AskFailed {
                reason: AskFailureReason::NpcNotFound,
                ..
            }]
        ));
    }

    #[test]
    fn ask_command_dispatches_to_topic_resolution() {
        let mut game = game_with_character_in("start");
        assert!(matches!(
            game.process(Command::Ask(AskQuery {
                npc: TargetQuery::npc("test npc".to_string()),
                topic: "test topic".to_string()
            }))
            .as_slice(),
            [GameEvent::NpcAnswered { .. }]
        ));
    }

    #[test]
    fn asking_duplicate_npc_name_is_ambiguous() {
        let mut game = game_with_character_in("start");
        let second = Npc {
            id: NpcId("second_npc".to_string()),
            name: "Test NPC".to_string(),
            room_description: "Another NPC stands here.".to_string(),
            description: "Another NPC.".to_string(),
            greeting: "Hello.".to_string(),
            topics: vec![],
        };
        game.world.npcs.insert(second.id.clone(), second);
        game.world
            .room_mut(&RoomId("start".to_string()))
            .unwrap()
            .npcs
            .push(NpcId("second_npc".to_string()));
        assert!(matches!(
            game.attempt_ask(AskQuery {
                npc: TargetQuery::npc("test npc".to_string()),
                topic: "test topic".to_string()
            })
            .as_slice(),
            [GameEvent::AskFailed {
                reason: AskFailureReason::NpcAmbiguous { match_count: 2 },
                ..
            }]
        ));
    }

    #[test]
    fn numbered_ask_selects_requested_npc_before_topic_lookup() {
        let mut game = game_with_character_in("start");
        let second = Npc {
            id: NpcId("second_npc".to_string()),
            name: "Test NPC".to_string(),
            room_description: "Another NPC stands here.".to_string(),
            description: "Another NPC.".to_string(),
            greeting: "Hello.".to_string(),
            topics: vec![NpcTopic {
                id: NpcTopicId("test_topic".to_string()),
                name: "Test Topic".to_string(),
                response: "The second NPC answers.".to_string(),
                requires_facts: vec![],
                excludes_facts: vec![],
                grants_facts: vec![],
                starts_quests: vec![],
            }],
        };
        game.world.npcs.insert(second.id.clone(), second);
        game.world
            .room_mut(&RoomId("start".to_string()))
            .unwrap()
            .npcs
            .push(NpcId("second_npc".to_string()));
        assert!(
            matches!(game.attempt_ask(AskQuery { npc: TargetQuery::npc("test npc".to_string()).with_ordinal(NonZeroUsize::new(2).unwrap()), topic: "test topic".to_string() }).as_slice(), [GameEvent::NpcAnswered { npc_id, .. }] if npc_id == &NpcId("second_npc".to_string()))
        );
    }

    #[test]
    fn out_of_range_ask_reports_matching_npc_count() {
        let mut game = game_with_character_in("start");
        assert!(matches!(
            game.attempt_ask(AskQuery {
                npc: TargetQuery::npc("test npc".to_string())
                    .with_ordinal(NonZeroUsize::new(2).unwrap()),
                topic: "test topic".to_string()
            })
            .as_slice(),
            [GameEvent::AskFailed {
                reason: AskFailureReason::NpcOrdinalOutOfRange {
                    requested: 2,
                    available: 1
                },
                ..
            }]
        ));
    }

    #[test]
    fn out_of_range_take_does_not_change_state() {
        let mut game = game_with_character_in("start");

        assert_eq!(
            game.attempt_take(
                TargetQuery::item("test item".to_string())
                    .with_ordinal(NonZeroUsize::new(2).expect("2 is nonzero"))
            ),
            vec![GameEvent::TakeFailed {
                character_id: CharacterId("player".to_string()),
                room_id: RoomId("start".to_string()),
                query: "test item".to_string(),
                reason: TakeFailureReason::OrdinalOutOfRange {
                    requested: 2,
                    available: 1,
                },
            }]
        );
        assert!(game.character.inventory.is_empty());
    }
}
