use strum_macros::EnumIter;

use crate::exercise::*;
use crate::game_reader::GameReader;
use crate::words::Database;

#[allow(unused)]
#[derive(EnumIter)]
pub enum ExerciseType {
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

pub fn play_game(
    exercise_max_cnt: usize,
    db: Database,
    exercise_types: Vec<ExerciseType>,
    mut game_reader: GameReader,
) {
    let mut results = GameResults::new();
    results.load_results("exercises.bin");
    results.update_with_db(&db);
    results.update_weights();
    let mut game = Exercise::new(db);

    println!("Type \"exit\" or press Ctrl-C to quit game");
    println!();
    'outer: for exercise_type in exercise_types.iter().cycle() {
        for _ in 0..exercise_max_cnt {
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
}
