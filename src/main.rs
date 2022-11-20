mod dictionary;
mod exercise;
mod game_reader;
mod words;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::dictionary::*;
use crate::exercise::*;
use crate::game_reader::GameReader;

#[allow(unused)]
#[derive(EnumIter)]
enum ExerciseType {
    SelectDe,
    TranslateRuDe,
    SelectRu,
    GuessNounArticle,
    VerbFormRandom,
}

impl ExerciseType {
    pub fn to_string(&self) -> &'static str {
        match &self {
            ExerciseType::SelectDe => "1) Select correct word in Deutsch",
            ExerciseType::TranslateRuDe => "2) Type in word in Deutsch",
            ExerciseType::SelectRu => "3) Select correct translation to Russian",
            ExerciseType::GuessNounArticle => "4) Select correct noun atricle",
            ExerciseType::VerbFormRandom => "5) Type in correct verb form",
        }
    }
}

const EXERCISE_MAX_COUNT: usize = 10;

fn select_excercise_mode(reader: &mut GameReader) -> Vec<ExerciseType> {
    println!("Select exercise mode:");
    println!("0) All exercises in series");
    for ex in ExerciseType::iter() {
        println!("{}", ex.to_string())
    }
    println!("other) Quit game");
    let select: usize = match reader.read_line() {
        None => return vec![],
        Some(s) => match s.parse() {
            Err(_) => return vec![],
            Ok(n) => n,
        },
    };
    if select == 0 {
        ExerciseType::iter().collect()
    } else {
        match ExerciseType::iter().nth(select - 1) {
            Some(ex) => vec![ex],
            None => return vec![],
        }
    }
}

fn main() {
    let db = fill_database("woerterbuch.xlsx");
    let mut game_reader = GameReader::new();
    let mut results = GameResults::new();
    results.load_results("exercises.bin");
    results.update_with_db(&db);
    results.update_weights();
    let mut game = Exercise::new(db);

    let exercise_types = select_excercise_mode(&mut game_reader);
    if exercise_types.is_empty() {
        println!("Quit game");
        return;
    }

    println!("Type \"exit\" or press Ctrl-C to quit game");
    println!();
    'outer: for exercise_type in exercise_types.iter().cycle() {
        for _ in 0..EXERCISE_MAX_COUNT {
            let result = match exercise_type {
                ExerciseType::TranslateRuDe => {
                    game.exercise_translate_to_de(&mut game_reader, &mut results)
                }
                ExerciseType::SelectDe => game.exercise_select_de(&mut game_reader, &mut results),
                ExerciseType::GuessNounArticle => {
                    game.guess_noun_article(&mut game_reader, &mut results)
                }
                ExerciseType::SelectRu => game.exercise_select_ru(&mut game_reader, &mut results),
                ExerciseType::VerbFormRandom => {
                    game.exercise_verb_form_random(&mut game_reader, &mut results)
                }
            };

            if let None = result {
                break 'outer;
            }

            results.update_weights();
        }
    }
    results.save_results();
    println!("Top words to learn are {:?}", results.get_top_words(5));
    println!("Quit dictionary game");
}
