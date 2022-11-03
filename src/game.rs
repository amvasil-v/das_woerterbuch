use crate::{game_reader::GameReader, words::*};
use bincode;
use colored::Colorize;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{cmp::Ordering, vec};
use strum::IntoEnumIterator;

const ANSWER_OPTIONS: usize = 4;
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

    fn select_word_to_learn(&mut self) -> &mut ExerciseResults {
        let mut rng = rand::thread_rng();
        let dist = self.rand_dist.as_ref().unwrap();
        &mut self.results[dist.sample(&mut rng)]
    }

    fn select_word_by_pos(&mut self, db: &Database, pos: PartOfSpeech) -> &mut ExerciseResults {
        let mut rng = rand::thread_rng();
        let mut weights = vec![];
        let mut indices = vec![];
        for (i, &weight) in self.weights.iter().enumerate() {
            let word = &self.results[i].word;
            if db.words[word].get_pos() == pos {
                weights.push(weight);
                indices.push(i)
            }
        }
        let dist = WeightedIndex::new(weights).unwrap();
        let idx = dist.sample(&mut rng);
        &mut self.results[indices[idx]]
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

enum UserInput {
    Answer(usize),
    InvalidAnswer,
    Exit,
}

impl Game {
    pub fn new(db: Database) -> Self {
        Game { db }
    }

    pub fn exercise_translate_to_de(
        &mut self,
        reader: &mut GameReader,
        results: &mut GameResults,
    ) -> Option<bool> {
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
        let mut rng = rand::thread_rng();
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

        let mut options = HashMap::new();
        options.insert(word.get_word(), word);

        const MAX_ATTEMPTS: usize = 1000;
        let mut attempts = 0usize;
        while options.len() < ANSWER_OPTIONS.min(candidates.len()) {
            let cand = *candidates.choose(&mut rng).unwrap();
            if let Some(_) = options.insert(cand.get_word(), cand) {
                attempts += 1;
                if attempts >= MAX_ATTEMPTS {
                    panic!("Cannot choose answer options");
                }
            }
        }

        let mut opt_vec: Vec<&Box<dyn Word>> = options.into_values().collect();
        opt_vec.shuffle(&mut rng);
        opt_vec
    }

    pub fn exercise_select_de(
        &mut self,
        reader: &mut GameReader,
        results: &mut GameResults,
    ) -> Option<bool> {
        let exercise_result = results.select_word_to_learn();
        let word = self.db.words.get(&exercise_result.word).unwrap();
        let options = self.fetch_word_options(word);

        println!(
            "Select translation to Deutsch: {} ({})",
            word.translation(),
            word.pos_str()
        );

        let bullets: Vec<_> = options.iter().map(|w| w.spelling()).collect();
        let result = match print_options_and_guess(&bullets, reader) {
            UserInput::Answer(a) => options[a].get_word() == word.get_word(),
            UserInput::InvalidAnswer => false,
            UserInput::Exit => return None,
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

    pub fn guess_noun_article(
        &mut self,
        reader: &mut GameReader,
        results: &mut GameResults,
    ) -> Option<bool> {
        let exercise_result = results.select_word_by_pos(&self.db, PartOfSpeech::Noun);
        let word = self.db.words.get(&exercise_result.word).unwrap();

        println!(
            "Select the correct article for the noun: {}",
            capitalize_noun(word.get_word())
        );
        let bullets: Vec<_> = NounArticle::iter().map(|a| a.to_answer_buller()).collect();
        let result = match print_options_and_guess(&bullets, reader) {
            UserInput::Answer(a) => NounArticle::iter().nth(a).unwrap() == word.get_article().unwrap(),
            UserInput::InvalidAnswer => false,
            UserInput::Exit => return None,
        };

        if result {
            print!("{}", "Correct! ".bold().green());
            true
        } else {
            print!(
                "{} The word is ",
                "Incorrect!".bold().red()
            );
            false
        };
        println!("{} - {}", word.spelling(), word.translation());
        println!();
        exercise_result.add(result);
        Some(result)
    }
}

fn print_options_and_guess(options: &[String], reader: &mut GameReader) -> UserInput {
    let mut count = 0usize;

    for (i, option) in options.iter().enumerate() {
        println!("{}) {}", i + 1, option);
        count += 1;
    }
    let input_str = match reader.read_line() {
        Some(s) => s,
        None => return UserInput::Exit,
    };
    let select: usize = match input_str.parse() {
        Err(_) => return UserInput::InvalidAnswer,
        Ok(v) => v,
    };

    if select < 1 || select > count {
        UserInput::InvalidAnswer
    } else {
        UserInput::Answer(select - 1)
    }
}
