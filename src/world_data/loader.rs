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
