use crate::game::command::{Command, TargetKind, TargetQuery};
use crate::game::direction::Direction;
use crate::game::naming::normalize_name;
use std::num::NonZeroUsize;

fn parse_target_query(
    query: &str,
    default_kind: Option<TargetKind>,
    allow_kind_qualifier: bool,
    missing_target_error: &str,
) -> Result<TargetQuery, String> {
    let mut words: Vec<&str> = query.split_whitespace().collect();
    let mut kind = default_kind;

    if allow_kind_qualifier {
        kind = match words.first().copied() {
            Some("item") => {
                words.remove(0);
                Some(TargetKind::Item)
            }
            Some("feature") => {
                words.remove(0);
                Some(TargetKind::Feature)
            }
            Some("npc") => {
                words.remove(0);
                Some(TargetKind::Npc)
            }
            _ => kind,
        };
    }

    let ordinal = match words.first().and_then(|word| word.parse::<usize>().ok()) {
        Some(0) => return Err("target number must be at least 1".to_string()),
        Some(number) => {
            words.remove(0);
            NonZeroUsize::new(number)
        }
        None => None,
    };

    if words.is_empty() {
        return Err(missing_target_error.to_string());
    }

    let name = words.join(" ");
    let mut target = match kind {
        Some(TargetKind::Item) => TargetQuery::item(name),
        Some(TargetKind::Feature) => TargetQuery::feature(name),
        Some(TargetKind::Npc) => TargetQuery::npc(name),
        None => TargetQuery::any(name),
    };

    if let Some(ordinal) = ordinal {
        target = target.with_ordinal(ordinal);
    }

    Ok(target)
}

fn parse_examine_query(query: &str) -> Result<Command, String> {
    parse_target_query(query, None, true, "examine what?").map(Command::Examine)
}

fn parse_direction(value: &str) -> Option<Direction> {
    match value {
        "north" | "n" => Some(Direction::North),
        "south" | "s" => Some(Direction::South),
        "east" | "e" => Some(Direction::East),
        "west" | "w" => Some(Direction::West),
        _ => None,
    }
}

pub fn parse_command(input: &str) -> Result<Command, String> {
    let normalized = normalize_name(input);

    match normalized.as_str() {
        "look" | "l" => Ok(Command::Look),
        "north" | "n" => Ok(Command::Move(Direction::North)),
        "south" | "s" => Ok(Command::Move(Direction::South)),
        "east" | "e" => Ok(Command::Move(Direction::East)),
        "west" | "w" => Ok(Command::Move(Direction::West)),
        "go" => Err("go where?".to_string()),
        command if command.starts_with("go ") => {
            let direction = command
                .strip_prefix("go ")
                .expect("go prefix was already checked");

            parse_direction(direction)
                .map(Command::Move)
                .ok_or_else(|| format!("unknown direction '{direction}'"))
        }
        "exits" => Ok(Command::Exits),
        "help" | "commands" => Ok(Command::Help),
        "take" => Err("take what?".to_string()),
        command if command.starts_with("take ") => {
            let query = command
                .strip_prefix("take ")
                .expect("take prefix was already checked")
                .to_string();

            parse_target_query(&query, Some(TargetKind::Item), false, "take what?")
                .map(Command::Take)
        }
        command if command.starts_with("get ") => {
            let query = command
                .strip_prefix("get ")
                .expect("get prefix was already checked")
                .to_string();

            parse_target_query(&query, Some(TargetKind::Item), false, "get what?")
                .map(Command::Take)
        }
        "get" => Err("get what?".to_string()),
        "inventory" | "inv" | "i" => Ok(Command::Inventory),
        "drop" => Err("drop what?".to_string()),
        command if command.starts_with("drop ") => {
            let query = command
                .strip_prefix("drop ")
                .expect("drop prefix was already checked")
                .to_string();

            parse_target_query(&query, Some(TargetKind::Item), false, "drop what?")
                .map(Command::Drop)
        }
        "talk" | "talk to" => Err("talk to whom?".to_string()),
        command if command.starts_with("talk to ") => {
            let query = command
                .strip_prefix("talk to ")
                .expect("talk-to prefix was already checked")
                .to_string();

            parse_target_query(&query, Some(TargetKind::Npc), false, "talk to whom?")
                .map(Command::Talk)
        }
        command if command.starts_with("talk ") => {
            let query = command
                .strip_prefix("talk ")
                .expect("talk prefix was already checked")
                .to_string();

            parse_target_query(&query, Some(TargetKind::Npc), false, "talk to whom?")
                .map(Command::Talk)
        }
        "examine" | "x" | "look at" => Err("examine what?".to_string()),
        command if command.starts_with("examine ") => {
            let query = command
                .strip_prefix("examine ")
                .expect("examine prefix was already checked")
                .to_string();

            parse_examine_query(&query)
        }
        command if command.starts_with("x ") => {
            let query = command
                .strip_prefix("x ")
                .expect("x prefix was already checked")
                .to_string();

            parse_examine_query(&query)
        }
        command if command.starts_with("look at ") => {
            let query = command
                .strip_prefix("look at ")
                .expect("look-at prefix was already checked")
                .to_string();

            parse_examine_query(&query)
        }
        command if command.starts_with("look ") => {
            let query = command
                .strip_prefix("look ")
                .expect("look prefix was already checked")
                .to_string();

            parse_examine_query(&query)
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
    fn go_with_full_direction_returns_move_command() {
        assert_eq!(
            parse_command("go north"),
            Ok(Command::Move(Direction::North))
        );
    }

    #[test]
    fn go_with_direction_alias_returns_move_command() {
        assert_eq!(parse_command("go w"), Ok(Command::Move(Direction::West)));
    }

    #[test]
    fn go_without_direction_returns_error() {
        assert_eq!(parse_command("go"), Err("go where?".to_string()));
    }

    #[test]
    fn go_with_unknown_direction_returns_error() {
        assert_eq!(
            parse_command("go sideways"),
            Err("unknown direction 'sideways'".to_string())
        );
    }

    #[test]
    fn exits_input_returns_exits_command() {
        assert_eq!(parse_command("exits"), Ok(Command::Exits));
    }

    #[test]
    fn help_inputs_return_help_command() {
        assert_eq!(parse_command("help"), Ok(Command::Help));
        assert_eq!(parse_command("commands"), Ok(Command::Help));
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
            Ok(Command::Take(TargetQuery::item("rusty key".to_string())))
        );
    }

    #[test]
    fn take_without_target_returns_error() {
        assert_eq!(parse_command("take"), Err("take what?".to_string()));
    }

    #[test]
    fn get_alias_returns_take_command_with_normalized_target() {
        assert_eq!(
            parse_command("  GET   Rusty   Key "),
            Ok(Command::Take(TargetQuery::item("rusty key".to_string())))
        );
    }

    #[test]
    fn get_without_target_returns_error() {
        assert_eq!(parse_command("get"), Err("get what?".to_string()));
    }

    #[test]
    fn inventory_alias_returns_inventory_command() {
        assert_eq!(parse_command(" INV "), Ok(Command::Inventory));
    }

    #[test]
    fn drop_input_returns_drop_command_with_normalized_target() {
        assert_eq!(
            parse_command("  DROP   Rusty   Key "),
            Ok(Command::Drop(TargetQuery::item("rusty key".to_string())))
        );
    }

    #[test]
    fn drop_without_target_returns_error() {
        assert_eq!(parse_command("drop"), Err("drop what?".to_string()));
    }

    #[test]
    fn talk_to_input_returns_npc_query() {
        assert_eq!(
            parse_command("  TALK TO   Mara   Voss "),
            Ok(Command::Talk(TargetQuery::npc("mara voss".to_string())))
        );
    }

    #[test]
    fn talk_without_to_returns_npc_query() {
        assert_eq!(
            parse_command("talk mara voss"),
            Ok(Command::Talk(TargetQuery::npc("mara voss".to_string())))
        );
    }

    #[test]
    fn talk_without_target_returns_error() {
        assert_eq!(parse_command("talk"), Err("talk to whom?".to_string()));
        assert_eq!(parse_command("talk to"), Err("talk to whom?".to_string()));
    }

    #[test]
    fn talk_to_ordinal_returns_numbered_npc_query() {
        assert_eq!(
            parse_command("talk to 2 guard"),
            Ok(Command::Talk(
                TargetQuery::npc("guard".to_string())
                    .with_ordinal(NonZeroUsize::new(2).expect("2 is nonzero"))
            ))
        );
    }

    #[test]
    fn examine_input_returns_examine_command() {
        assert_eq!(
            parse_command(" EXAMINE   Rusty Key "),
            Ok(Command::Examine(TargetQuery::any("rusty key".to_string())))
        );
    }

    #[test]
    fn examine_short_alias_returns_examine_command() {
        assert_eq!(
            parse_command("x rusty key"),
            Ok(Command::Examine(TargetQuery::any("rusty key".to_string())))
        );
    }

    #[test]
    fn look_at_returns_examine_command() {
        assert_eq!(
            parse_command("look   at   rusty key"),
            Ok(Command::Examine(TargetQuery::any("rusty key".to_string())))
        );
    }

    #[test]
    fn look_with_target_returns_examine_command() {
        assert_eq!(
            parse_command("look rusty key"),
            Ok(Command::Examine(TargetQuery::any("rusty key".to_string())))
        );
    }

    #[test]
    fn look_with_qualified_target_returns_typed_examine_command() {
        assert_eq!(
            parse_command("look npc mara voss"),
            Ok(Command::Examine(TargetQuery::npc("mara voss".to_string())))
        );
    }

    #[test]
    fn examine_without_target_returns_error() {
        assert_eq!(parse_command("examine"), Err("examine what?".to_string()));
    }

    #[test]
    fn examine_item_qualifier_returns_typed_query() {
        assert_eq!(
            parse_command("x item rusty key"),
            Ok(Command::Examine(TargetQuery::item("rusty key".to_string())))
        );
    }

    #[test]
    fn examine_feature_qualifier_returns_typed_query() {
        assert_eq!(
            parse_command("look at feature plaque"),
            Ok(Command::Examine(TargetQuery::feature("plaque".to_string())))
        );
    }

    #[test]
    fn examine_npc_qualifier_returns_typed_query() {
        assert_eq!(
            parse_command("x npc mara voss"),
            Ok(Command::Examine(TargetQuery::npc("mara voss".to_string())))
        );
    }

    #[test]
    fn quoted_examine_target_is_normalized() {
        assert_eq!(
            parse_command("x \"Founder's Plaque\""),
            Ok(Command::Examine(TargetQuery::any(
                "founders plaque".to_string()
            )))
        );
    }

    #[test]
    fn take_ordinal_returns_numbered_item_query() {
        assert_eq!(
            parse_command("take 2 rusty key"),
            Ok(Command::Take(
                TargetQuery::item("rusty key".to_string())
                    .with_ordinal(NonZeroUsize::new(2).expect("2 is nonzero"))
            ))
        );
    }

    #[test]
    fn qualified_examine_ordinal_returns_numbered_query() {
        assert_eq!(
            parse_command("x feature 2 rusty key"),
            Ok(Command::Examine(
                TargetQuery::feature("rusty key".to_string())
                    .with_ordinal(NonZeroUsize::new(2).expect("2 is nonzero"))
            ))
        );
    }

    #[test]
    fn zero_ordinal_is_rejected() {
        assert_eq!(
            parse_command("take 0 rusty key"),
            Err("target number must be at least 1".to_string())
        );
    }
}
