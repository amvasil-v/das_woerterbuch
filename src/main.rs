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
    SelectDe
}

const EXERCISE_TYPE: ExerciseType = ExerciseType::SelectDe;

fn main() {
    let db = fill_database("woerterbuch.xlsx");
    let mut game = Game::new(db);
    let mut game_reader = GameReader::new();

    println!("Type \"exit\" or press Ctrl-C to quit game");
    game.load_results("exercises.bin");
    game.update_result_with_db();
    game.update_weights();
    println!();
    loop {
        let result = match EXERCISE_TYPE {
            ExerciseType::TranslateRuDe => game.exercise_translate_to_de(&mut game_reader),
            ExerciseType::SelectDe => game.exercise_select_de(&mut game_reader)
        };

        if let None = result {
            break;
        }

        game.update_weights();
    }
    game.save_results();
    println!("Top words to learn are {:?}", game.get_top_words(5));
    println!("Quit dictionary game");
}
