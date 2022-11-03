mod dictionary;
mod game;
mod words;
mod game_reader;

use crate::dictionary::*;
use crate::game::*;
use crate::game_reader::GameReader;

#[allow(unused)]
enum ExerciseType {
    TranslateRuDe,
    SelectDe,
    GuessNounArticle,
    SelectRu,
}

const EXERCISE_TYPE: ExerciseType = ExerciseType::SelectRu;

fn main() {
    let db = fill_database("woerterbuch.xlsx");
    let mut game_reader = GameReader::new();
    let mut results = GameResults::new();
    results.load_results("exercises.bin");
    results.update_with_db(&db);
    results.update_weights();
    let mut game = Game::new(db);

    println!("Type \"exit\" or press Ctrl-C to quit game");
    println!();
    loop {
        let result = match EXERCISE_TYPE {
            ExerciseType::TranslateRuDe => game.exercise_translate_to_de(&mut game_reader, &mut results),
            ExerciseType::SelectDe => game.exercise_select_de(&mut game_reader, &mut results),
            ExerciseType::GuessNounArticle => game.guess_noun_article(&mut game_reader, &mut results),
            ExerciseType::SelectRu => game.exercise_select_ru(&mut game_reader, &mut results)
        };

        if let None = result {
            break;
        }

        results.update_weights();
    }
    results.save_results();
    println!("Top words to learn are {:?}", results.get_top_words(5));
    println!("Quit dictionary game");
}
