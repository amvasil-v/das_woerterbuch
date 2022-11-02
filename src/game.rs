use crate::{game_reader::GameReader, words::*};
use bincode;
use colored::Colorize;
use rand::{rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ExerciseResults {
    word: String,
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

    pub fn score(&self) -> i32 {
        self.correct as i32 - (self.wrong * 2) as i32
    }

    pub fn new(s: &str) -> Self {
        Self {
            correct: 0,
            wrong: 0,
            word: s.to_owned()
        }
    }
}

impl Ord for ExerciseResults {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score().cmp(&other.score())
    }
}

impl PartialOrd for ExerciseResults {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ExerciseResults {
    fn eq(&self, other: &Self) -> bool {
        self.score() == other.score() && self.word == other.word
    }
}

impl Eq for ExerciseResults { }


pub struct Game {
    results: Vec<ExerciseResults>,
    results_filename: String,
    db: Database,
    rng: ThreadRng,
}

impl Game {
    pub fn new(db: Database) -> Self {
        Game {
            results: vec![],
            results_filename: String::new(),
            db,
            rng: rand::thread_rng(),
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

    pub fn save_results(&mut self) {
        let path = std::path::Path::new(&self.results_filename);
        let f = std::fs::File::create(path).unwrap();
        let writer = std::io::BufWriter::new(f);

        self.results.sort_unstable();

        bincode::serialize_into(writer, &self.results).unwrap();
    }

    pub fn exercise_translate_to_de(&mut self, reader: &mut GameReader) -> Option<bool> {
        let idx = self.rng.gen_range(0..self.results.len());
        let exercise_result = &mut self.results[idx];
        let word = self.db.words.get(&exercise_result.word).unwrap().as_ref();
        println!(
            "Translate to German: {} ({})",
            word.translation(),
            word.pos_str()
        );
        let answer = match reader.read_line() {
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

        exercise_result.add(res);
        Some(res)
    }

    pub fn update_result_with_db(&mut self) {
        for word in self.db.words.keys() {
            let new_entry = ExerciseResults::new(word);
            if !self.results.contains(&new_entry) {
                self.results.push(new_entry);
            }
        }
        self.results.sort_unstable()
    }

    pub fn get_top_words(&self, n: usize) -> Vec<String> {
        self.results
            .iter()
            .take(n)
            .map(|r| r.word.to_owned())
            .collect()
    }
}
