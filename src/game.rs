use crate::{game_reader::GameReader, words::*};
use bincode;
use colored::Colorize;
use rand::prelude::*;
use rand::{distributions::WeightedIndex};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, vec};

const ANSWER_OPTIONS: usize = 3;
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
            word: s.to_owned(),
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
        self.word == other.word
    }
}

impl Eq for ExerciseResults {}

pub struct Game {
    db: Database,
}

pub struct GameResults {
    results: Vec<ExerciseResults>,
    results_filename: String,
    weights: Vec<f32>,
    rand_dist: Option<WeightedIndex<f32>>,
}

impl GameResults {
    pub fn new() -> Self {
        GameResults {
            results: vec![],
            results_filename: String::new(),
            weights: vec![],
            rand_dist: None,
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

    pub fn update_with_db(&mut self, db: &Database) {
        for word in db.words.keys() {
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

    pub fn select_word_to_learn(&mut self) -> &mut ExerciseResults {
        let mut rng = rand::thread_rng();
        let dist = self.rand_dist.as_ref().unwrap();
        &mut self.results[dist.sample(&mut rng)]
    }

    pub fn update_weights(&mut self) {
        self.weights.clear();
        self.results.sort_unstable();
        let max_score = self.results.last().unwrap().score();
        let min_score = self.results.first().unwrap().score();
        self.weights.extend(
            self.results
                .iter()
                .map(|ex| (2 * max_score - min_score - ex.score() + 1) as f32),
        );
        self.rand_dist = Some(WeightedIndex::new(&self.weights).unwrap());
    }
}

impl Game {
    pub fn new(db: Database) -> Self {
        Game {
            db
        }
    }   

    pub fn exercise_translate_to_de(&mut self, reader: &mut GameReader, results: &mut GameResults) -> Option<bool> {
        let exercise_result = results.select_word_to_learn();
        let word = self.db.words.get(&exercise_result.word).unwrap().as_ref();
        print!(
            "Translate to German: {} ({})",
            word.translation(),
            word.pos_str()
        );
        let help = word.get_help();
        if !help.is_empty() {
            print!(" Help: {}", help);
        }
        println!();
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

    fn fetch_word_options<'a>(&'a self, word: &'a Box<dyn Word>) -> Vec<&'a Box<dyn Word>> {
        let group_id = word.get_group_id();
        let pos = word.get_pos();
        let candidates: Vec<_> = self
            .db
            .words
            .iter()
            .filter_map(|(_, w)| {
                if w.get_group_id() == group_id && w.get_pos() == pos {
                    Some(w)
                } else {
                    None
                }
            })
            .collect();
        let mut options: Vec<_> = candidates
            .into_iter()
            .choose_multiple(&mut rand::thread_rng(), ANSWER_OPTIONS);

        options.push(&word);
        options
    }

    pub fn exercise_select_de(&mut self, reader: &mut GameReader, results: &mut GameResults) -> Option<bool> {
        let exercise_result = results.select_word_to_learn();
        let word = self.db.words.get(&exercise_result.word).unwrap();
        let options = self.fetch_word_options(word);

        println!("Select translation to deutsch: {} ({})", word.translation(), word.pos_str());
        for (i, &option) in options.iter().enumerate() {
            println!("{}) {}", i + 1, self.db.words[option.get_word()].spelling());
        }
        let select: usize = match reader.read_line()?.parse() {
            Err(_) => return Some(false),
            Ok(v) => v
        };

        let result = if select < 1 || select > options.len() {
            false
        } else if options[select - 1].get_word() == word.get_word() {
            true
        } else {
            false
        };

        if result {
            println!("{}", "Correct!".bold().green());
            true
        } else {
            println!(
                "{} The word is {}",
                "Incorrect!".bold().red(),
                word.spelling()
            );
            false
        };
        println!();
        exercise_result.add(result);
        Some(result)
    }
}
