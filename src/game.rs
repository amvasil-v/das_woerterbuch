use rand::Rng;

use crate::exercise::*;
use crate::game_reader::GameReader;
use crate::words::Database;

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

fn play_game_round(
    exercise_max_cnt: usize,
    exercise: &Exercise,
    exercise_types: &Vec<ExerciseType>,
    game_reader: &mut GameReader,
    results: &mut GameResults,
) -> Option<()> {
    for exercise_type in exercise_types {
        for _ in 0..exercise_max_cnt {
            exercise.exercise(game_reader, results, exercise_type)?;
            results.update_weights();
        }
    }
    repeat_words(results.get_training_words(), game_reader, &exercise)
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
    let ex = Exercise::new(db);

    println!("Type \"exit\" or press Ctrl-C to quit game");
    println!();
    loop {
        if let None = play_game_round(
            exercise_max_cnt,
            &ex,
            &exercise_types,
            &mut game_reader,
            &mut results,
        ) {
            break;
        }
    }
    results.save_results();
    println!("Top words to learn are {:?}", results.get_top_words(5));
}

fn repeat_words(words: &Vec<String>, reader: &mut GameReader, exercise: &Exercise) -> Option<()> {
    if words.is_empty() {
        println!("Congratulations, all answers are correct!");
        return Some(());
    }
    println!("Words to repeat are: {:?}", words);
    println!();
    let mut repeat: Vec<_> = words
        .iter()
        .map(|w| {
            let word_ref = exercise.get_word_from_database(w);
            (word_ref, exercise.get_random_exercise_type(word_ref))
        })
        .collect();
    let mut rng = rand::thread_rng();

    while !repeat.is_empty() {
        let elem = repeat.remove(rng.gen_range(0..repeat.len()));
        let result = exercise.exercise_with_type(reader, elem.0, &elem.1)?;
        if result {
            match elem.1 {
                ExerciseType::TranslateRuDe | ExerciseType::VerbFormRandom => continue,
                _ => repeat.push((elem.0, ExerciseType::TranslateRuDe)),
            }
        } else {
            repeat.push((elem.0, ExerciseType::TranslateRuDe));
        }
    }
    println!("All words repeated!");
    println!();
    Some(())
}
