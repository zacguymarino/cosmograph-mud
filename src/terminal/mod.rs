pub mod parser;
pub mod renderer;

use std::io::{self, Write};

use crate::game::command::Command;
use crate::game::game::Game;

use parser::parse_command;
use renderer::render_event;

pub fn run(game: &mut Game) -> io::Result<()> {
    for event in game.process(Command::Look) {
        println!("{}", render_event(&event));
    }

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        let bytes_read = io::stdin().read_line(&mut input)?;

        if bytes_read == 0 {
            println!();
            break;
        }

        let input = input.trim();

        if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
            break;
        }

        match parse_command(input) {
            Ok(command) => {
                for event in game.process(command) {
                    println!("{}", render_event(&event));
                }
            }
            Err(error) => println!("{error}"),
        }
    }

    Ok(())
}
