mod dictionary;
mod game;
mod game_reader;
mod words;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::dictionary::*;
use crate::game::*;
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

//const EXERCISE_TYPE: ExerciseType = ExerciseType::SelectRu;
const EXERCISE_MAX_COUNT: usize = 10;

fn main() {
    let db = fill_database("woerterbuch.xlsx");
    let mut game_reader = GameReader::new();
    let mut results = GameResults::new();
    results.load_results("exercises.bin");
    results.update_with_db(&db);
    results.update_weights();
    let mut game = Game::new(db);
    //let exercise_types = [ExerciseType::VerbFormRandom];
    let exercise_types: Vec<_> = ExerciseType::iter().collect();

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
