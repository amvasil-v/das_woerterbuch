use std::collections::HashMap;

use crate::words::*;
use bincode;
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ExerciseResults {
    correct: usize,
    wrong: usize,
}

impl ExerciseResults {
    pub fn add(&mut self, correct: bool) {
        if correct {
            self.correct += 1;
        } else {
            self.wrong += 1;
        }
    }
}
pub struct Game {
    reader: Editor<()>,
    results: HashMap<String, ExerciseResults>,
    results_filename: String,
}

impl Game {
    pub fn new() -> Self {
        Game {
            reader: Editor::<()>::new().unwrap(),
            results: HashMap::new(),
            results_filename: String::new(),
        }
    }

    pub fn load_results(&mut self, filename: &str) {
        let path = std::path::Path::new(filename);
        self.results_filename = filename.to_owned();
        if path.exists() {
            let f = std::fs::File::open(path).unwrap();
            let freader = std::io::BufReader::new(f);
            self.results = bincode::deserialize_from(freader).unwrap();
            println!("Loaded previous results, {} entries", self.results.len())
        }
    }

    pub fn save_results(&self) {
        let path = std::path::Path::new(&self.results_filename);
        let f = std::fs::File::create(path).unwrap();
        let writer = std::io::BufWriter::new(f);
        bincode::serialize_into(writer, &self.results).unwrap();
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

        self.results
            .entry(word.get_word().to_owned())
            .or_default()
            .add(res);
        Some(res)
    }
}
