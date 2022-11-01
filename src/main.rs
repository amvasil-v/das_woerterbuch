mod dictionary;
mod game;
mod words;

use crate::dictionary::*;
use crate::game::*;
use crate::words::*;
use rand::Rng;
use strum::IntoEnumIterator;

fn main() {
    let db = fill_database("woerterbuch.xlsx");
    let mut rng = rand::thread_rng();
    let mut game = Game::new();

    println!("Type \"exit\" or press Ctrl-C to quit game");
    game.load_results("exercises.bin");
    let mut pos = PartOfSpeech::iter();
    println!();
    loop {
        let part_of_speech = match pos.next() {
            Some(p) => p,
            None => {
                pos = PartOfSpeech::iter();
                continue;
            }
        };
        let result = match part_of_speech {
            PartOfSpeech::Noun => {
                let idx = rng.gen_range(0..db.nouns.len());
                let noun = &db.nouns[idx];
                game.exercise_translate_to_de(noun)
            }
            PartOfSpeech::Verb => {
                let idx = rng.gen_range(0..db.verbs.len());
                let verb = &db.verbs[idx];
                game.exercise_translate_to_de(verb)
            }
        };

        if let None = result {
            break;
        }
    }
    game.save_results();
    println!("Quit dictionary game");
}
