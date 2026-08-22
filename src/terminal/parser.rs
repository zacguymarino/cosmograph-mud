use crate::game::command::Command;
use crate::game::direction::Direction;
use crate::game::item::normalize_item_name;

pub fn parse_command(input: &str) -> Result<Command, String> {
    let normalized = normalize_item_name(input);

    match normalized.as_str() {
        "look" | "l" => Ok(Command::Look),
        "north" | "n" => Ok(Command::Move(Direction::North)),
        "south" | "s" => Ok(Command::Move(Direction::South)),
        "east" | "e" => Ok(Command::Move(Direction::East)),
        "west" | "w" => Ok(Command::Move(Direction::West)),
        "take" => Err("take what?".to_string()),
        command if command.starts_with("take ") => {
            let query = command
                .strip_prefix("take ")
                .expect("take prefix was already checked")
                .to_string();

            Ok(Command::Take(query))
        }
        "inventory" | "inv" | "i" => Ok(Command::Inventory),
        "drop" => Err("drop what?".to_string()),
        command if command.starts_with("drop ") => {
            let query = command
                .strip_prefix("drop ")
                .expect("drop prefix was already checked")
                .to_string();

            Ok(Command::Drop(query))
        }
        "examine" | "x" | "look at" => Err("examine what?".to_string()),
        command if command.starts_with("examine ") => {
            let query = command
                .strip_prefix("examine ")
                .expect("examine prefix was already checked")
                .to_string();

            Ok(Command::Examine(query))
        }
        command if command.starts_with("x ") => {
            let query = command
                .strip_prefix("x ")
                .expect("x prefix was already checked")
                .to_string();

            Ok(Command::Examine(query))
        }
        command if command.starts_with("look at ") => {
            let query = command
                .strip_prefix("look at ")
                .expect("look-at prefix was already checked")
                .to_string();

            Ok(Command::Examine(query))
        }
        _ => Err(format!("unknown command '{}'", input.trim())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn look_input_returns_look_command() {
        assert_eq!(parse_command("look"), Ok(Command::Look));
    }

    #[test]
    fn direction_input_returns_move_command() {
        assert_eq!(parse_command(" N "), Ok(Command::Move(Direction::North)));
    }

    #[test]
    fn unknown_input_returns_error() {
        assert_eq!(
            parse_command("unknown"),
            Err("unknown command 'unknown'".to_string())
        );
    }

    #[test]
    fn take_input_returns_take_command_with_normalized_target() {
        assert_eq!(
            parse_command("  TAKE   Rusty   Key "),
            Ok(Command::Take("rusty key".to_string()))
        );
    }

    #[test]
    fn take_without_target_returns_error() {
        assert_eq!(parse_command("take"), Err("take what?".to_string()));
    }

    #[test]
    fn inventory_alias_returns_inventory_command() {
        assert_eq!(parse_command(" INV "), Ok(Command::Inventory));
    }

    #[test]
    fn drop_input_returns_drop_command_with_normalized_target() {
        assert_eq!(
            parse_command("  DROP   Rusty   Key "),
            Ok(Command::Drop("rusty key".to_string()))
        );
    }

    #[test]
    fn drop_without_target_returns_error() {
        assert_eq!(parse_command("drop"), Err("drop what?".to_string()));
    }

    #[test]
    fn examine_input_returns_examine_command() {
        assert_eq!(
            parse_command(" EXAMINE   Rusty Key "),
            Ok(Command::Examine("rusty key".to_string()))
        );
    }

    #[test]
    fn examine_short_alias_returns_examine_command() {
        assert_eq!(
            parse_command("x rusty key"),
            Ok(Command::Examine("rusty key".to_string()))
        );
    }

    #[test]
    fn look_at_returns_examine_command() {
        assert_eq!(
            parse_command("look   at   rusty key"),
            Ok(Command::Examine("rusty key".to_string()))
        );
    }

    #[test]
    fn examine_without_target_returns_error() {
        assert_eq!(parse_command("examine"), Err("examine what?".to_string()));
    }
}
