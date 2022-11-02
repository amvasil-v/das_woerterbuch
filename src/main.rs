mod dictionary;
mod game;
mod words;
mod game_reader;

use crate::dictionary::*;
use crate::game::*;
use crate::game_reader::GameReader;


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
        let result = game.exercise_translate_to_de(&mut game_reader);

        if let None = result {
            break;
        }
    }
    game.save_results();
    println!("Top words to learn are {:?}", game.get_top_words(5));
    println!("Quit dictionary game");
}
