#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "north" => Some(Self::North),
            "south" => Some(Self::South),
            "east" => Some(Self::East),
            "west" => Some(Self::West),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_known_direction() {
        assert_eq!(Direction::from_str("north"), Some(Direction::North));
    }

    #[test]
    fn rejects_unknown_direction() {
        assert_eq!(Direction::from_str("sideways"), None);
    }
}
