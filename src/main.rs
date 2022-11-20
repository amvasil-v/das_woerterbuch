mod dictionary;
mod exercise;
mod game;
mod game_reader;
mod words;

use crate::dictionary::*;
use crate::game::*;
use crate::game_reader::GameReader;
use strum::IntoEnumIterator;

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
    let exercise_types = select_excercise_mode(&mut game_reader);
    if exercise_types.is_empty() {
        println!("Quit game");
        return;
    }

    play_game(EXERCISE_MAX_COUNT, db, exercise_types, game_reader);
    println!("Quit dictionary game");
}
