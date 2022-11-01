use crate::words::*;
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::Editor;

pub struct Game {
    reader: Editor<()>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            reader: Editor::<()>::new().unwrap(),
        }
    }

    pub fn read_line(&mut self) -> Option<String> {
        let readline = self.reader.readline(">> ");
        match readline {
            Ok(s) => {
                let answer = s.trim().to_lowercase();
                if answer == "exit" || answer == "quit" {
                    return None;
                }
                Some(answer)
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                None
            }
            _ => {
                println!("Readline error");
                None
            }
        }
    }

    pub fn exercise_translate_to_de(&mut self, word: &impl Word) -> Option<bool> {
        println!(
            "Translate to German: {} ({})",
            word.translation(),
            word.pos_str()
        );
        let answer = match self.read_line() {
            None => return None,
            Some(s) => s,
        };
        let res = word.check_spelling(&answer);

        if res {
            println!("{} {}", "Correct!".bold().green(), word.spelling());
        } else {
            println!(
                "{} The word is {}",
                "Incorrect!".bold().red(),
                word.spelling()
            );
        }
        println!();

        Some(res)
    }
}
