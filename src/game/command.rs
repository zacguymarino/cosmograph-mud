use super::direction::Direction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Look,
    Move(Direction),
    Take(String),
    Inventory,
    Drop(String),
    Examine(String),
}
