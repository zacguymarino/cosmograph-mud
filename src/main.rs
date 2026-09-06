mod game;
mod terminal;
mod world_data;

use game::character::Character;
use game::game::Game;
use game::ids::CharacterId;
use std::collections::HashMap;
use std::collections::HashSet;
use world_data::loader::load_world;
fn main() {
    let world = load_world("worlds/origin/world.toml").expect("failed to load Origin world");

    let starting_room = world.starting_room.clone();

    let character = Character {
        id: CharacterId("player".to_string()),
        name: "Player".to_string(),
        current_room: starting_room,
        inventory: vec![],
        facts: HashSet::new(),
        quests: HashMap::new(),
    };

    let mut game = Game::new(world, character);

    terminal::run(&mut game).expect("terminal input/output failed");
}
