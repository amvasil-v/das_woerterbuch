use crate::{game_reader::GameReader, words::*};
use bincode;
use colored::Colorize;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{cmp::Ordering, vec};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

const ANSWER_OPTIONS: usize = 4;

#[allow(unused)]
#[derive(EnumIter)]
pub enum ExerciseType {
    SelectDe,
    TranslateRuDe,
    SelectRu,
    GuessNounArticle,
    VerbFormRandom,
}
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

pub struct Exercise {
    db: Database,
}

#[derive(Debug, EnumIter)]
enum VerbFormExercise {
    PresentThird,
    Praeteritum,
    Perfect,
}

pub struct GameResults {
    results: Vec<ExerciseResults>,
    results_filename: String,
    weights: Vec<f32>,
    rand_dist: Option<WeightedIndex<f32>>,
    training: Vec<String>,
}

impl GameResults {
    pub fn new() -> Self {
        GameResults {
            results: vec![],
            results_filename: String::new(),
            weights: vec![],
            rand_dist: None,
            training: vec![],
        }
    }

    pub fn get_training_words(&self) -> &Vec<String> {
        &self.training
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

    fn select_word_by_cmp<T>(
        &mut self,
        db: &Database,
        cmp: impl Fn(&dyn Word, &T) -> bool,
        prop: &T,
    ) -> &mut ExerciseResults {
        let mut rng = rand::thread_rng();
        let mut weights = vec![];
        let mut indices = vec![];
        for (i, &weight) in self.weights.iter().enumerate() {
            let word = &self.results[i].word;
            if let Some(w) = db.words.get(word) {
                if cmp(w.as_ref(), prop) {
                    weights.push(weight);
                    indices.push(i);
                }
            }
        }
        let dist = WeightedIndex::new(weights).unwrap();
        let idx = dist.sample(&mut rng);
        &mut self.results[indices[idx]]
    }

    fn select_word_by_pos(&mut self, db: &Database, pos: PartOfSpeech) -> &mut ExerciseResults {
        let cmp = |word: &dyn Word, prop: &PartOfSpeech| &word.get_pos() == prop;
        return self.select_word_by_cmp(db, cmp, &pos);
    }

    fn select_word_with_verb_form(
        &mut self,
        db: &Database,
        form: &VerbFormExercise,
    ) -> &mut ExerciseResults {
        let cmp = |word: &dyn Word, form: &VerbFormExercise| {
            if word.get_pos() != PartOfSpeech::Verb {
                return false;
            }
            let opt = match form {
                &VerbFormExercise::Praeteritum => word.get_verb_praeteritum(),
                &VerbFormExercise::PresentThird => word.get_verb_present_third(),
                &VerbFormExercise::Perfect => {
                    if let None = word.get_verb_perfect_verb() {
                        return false;
                    }
                    word.get_verb_perfect()
                }
            };
            match opt {
                None => false,
                Some(s) if s.is_empty() => false,
                _ => true,
            }
        };
        return self.select_word_by_cmp(db, cmp, form);
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

impl Exercise {
    pub fn new(db: Database) -> Self {
        Exercise { db }
    }

    pub fn exercise_translate_to_de(
        &self,
        reader: &mut GameReader,
        word: &Box<dyn Word>,
    ) -> Option<bool> {
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
        Some(res)
    }

    fn exercise_verb_form(
        &self,
        reader: &mut GameReader,
        word: &Box<dyn Word>,
        form: &VerbFormExercise,
    ) -> Option<bool> {
        println!(
            "{} [ {} - {} ]",
            match form {
                VerbFormExercise::PresentThird => "Add verb in present tense: Er ... jetzt",
                VerbFormExercise::Praeteritum => "Add verb in Präteritum : Er ... einst",
                VerbFormExercise::Perfect => "Add verb in Perfekt : Er ... ... gestern",
            },
            word.get_word(),
            word.translation()
        );
        let answer = match reader.read_line() {
            None => return None,
            Some(s) => s,
        };

        let correct = match form {
            VerbFormExercise::PresentThird => word.get_verb_present_third().unwrap().to_owned(),
            VerbFormExercise::Praeteritum => word.get_verb_praeteritum().unwrap().to_owned(),
            VerbFormExercise::Perfect => word.get_verb_perfect_full().unwrap(),
        };
        let res = match form {
            VerbFormExercise::PresentThird | VerbFormExercise::Praeteritum => {
                check_spelling_simple(&answer, &correct)
            }
            VerbFormExercise::Perfect => check_spelling_perfect(&answer, word.as_ref()),
        };
        if res {
            println!("{} {}", "Correct!".bold().green(), correct);
        } else {
            println!("{} The word is {}", "Incorrect!".bold().red(), correct);
        }
        println!();
        Some(res)
    }

    pub fn exercise_verb_form_random(
        &self,
        reader: &mut GameReader,
        word: &Box<dyn Word>,
    ) -> Option<bool> {
        let mut rng = rand::thread_rng();
        let form = VerbFormExercise::iter().choose(&mut rng).unwrap();
        self.exercise_verb_form(reader, word, &form)
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
        &self,
        reader: &mut GameReader,
        word: &Box<dyn Word>,
    ) -> Option<bool> {
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
        Some(result)
    }

    pub fn guess_noun_article(
        &self,
        reader: &mut GameReader,
        word: &Box<dyn Word>,
    ) -> Option<bool> {
        println!(
            "Select the correct article for the noun: {}",
            capitalize_noun(word.get_word())
        );
        let bullets: Vec<_> = NounArticle::iter().map(|a| a.to_answer_buller()).collect();
        let result = match print_options_and_guess(&bullets, reader) {
            UserInput::Answer(a) => {
                NounArticle::iter().nth(a).unwrap() == word.get_article().unwrap()
            }
            UserInput::InvalidAnswer => false,
            UserInput::Exit => return None,
        };

        if result {
            print!("{}", "Correct! ".bold().green());
            true
        } else {
            print!("{} The word is ", "Incorrect!".bold().red());
            false
        };
        println!("{} - {}", word.spelling(), word.translation());
        println!();
        Some(result)
    }

    pub fn exercise_select_ru(
        &self,
        reader: &mut GameReader,
        word: &Box<dyn Word>,
    ) -> Option<bool> {
        let options = self.fetch_word_options(word);

        println!(
            "Select translation to Russian: {} ({})",
            word.spelling(),
            word.pos_str()
        );

        let bullets: Vec<_> = options.iter().map(|w| w.translation().to_owned()).collect();
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
                "{} The tranlation is {}",
                "Incorrect!".bold().red(),
                word.translation()
            );
            false
        };
        println!();
        Some(result)
    }

    pub fn exercise(
        &self,
        reader: &mut GameReader,
        results: &mut GameResults,
        ex_type: &ExerciseType,
    ) -> Option<bool> {
        let exercise_result = match ex_type {
            ExerciseType::VerbFormRandom => {
                let mut rng = rand::thread_rng();
                let form = VerbFormExercise::iter().choose(&mut rng).unwrap();
                results.select_word_with_verb_form(&self.db, &form)
            }
            ExerciseType::GuessNounArticle => {
                results.select_word_by_pos(&self.db, PartOfSpeech::Noun)
            }
            _ => results.select_word_to_learn(),
        };
        let word = match self.db.words.get(&exercise_result.word) {
            Some(w) => w,
            None => {
                return Some(false);
            }
        };

        let result = self.exercise_with_type(reader, word, ex_type)?;

        exercise_result.add(result);
        if !result {
            results.training.push(word.get_word().to_owned());
        }
        Some(result)
    }

    pub fn exercise_with_type(
        &self,
        reader: &mut GameReader,
        word: &Box<dyn Word>,
        ex_type: &ExerciseType,
    ) -> Option<bool> {
        match ex_type {
            ExerciseType::TranslateRuDe => self.exercise_translate_to_de(reader, word),
            ExerciseType::SelectDe => self.exercise_select_de(reader, word),
            ExerciseType::GuessNounArticle => self.guess_noun_article(reader, word),
            ExerciseType::SelectRu => self.exercise_select_ru(reader, word),
            ExerciseType::VerbFormRandom => self.exercise_verb_form_random(reader, word),
        }
    }

    #[allow(unused)]
    pub fn exercise_with_random_type(
        &self,
        reader: &mut GameReader,
        word: &Box<dyn Word>,
    ) -> Option<bool> {
        self.exercise_with_type(reader, word, &self.get_random_exercise_type(word))
    }

    pub fn get_word_from_database(&self, word: &str) -> &Box<dyn Word> {
        self.db.words.get(word).unwrap()
    }

    pub fn get_random_exercise_type(&self, word: &Box<dyn Word>) -> ExerciseType {
        if let PartOfSpeech::Verb = word.get_pos() {
            if word.get_verb_praeteritum().is_some() {
                return ExerciseType::VerbFormRandom;
            }            
        }
        let mut rng = rand::thread_rng();
        let ex_type_prelim = ExerciseType::iter().choose(&mut rng).unwrap();
        match (ex_type_prelim, word.get_pos()) {
            (ExerciseType::VerbFormRandom, _) => ExerciseType::TranslateRuDe,
            (ExerciseType::GuessNounArticle, PartOfSpeech::Noun) => ExerciseType::GuessNounArticle,
            (ExerciseType::GuessNounArticle, _) => ExerciseType::TranslateRuDe,
            (t, _) => t,
        }
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
