use crate::words::*;
use std::io;
use colored::Colorize;

pub fn exercise_translate_to_de(word: &impl Word) -> Option<bool>
{
    println!("Translate to German: {} ({})", word.translation(), word.pos_str());
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .expect("Failed to read line");
    answer = answer.trim().to_lowercase();
    if answer == "exit" || answer == "quit" {
        return None;
    }
    let res = word.check_spelling(&answer);

    if res {
        println!("{} {}", "Correct!".bold().green(), word.spelling());
    } else {
        println!("{} The word is {}", "Incorrect!".bold().red(), word.spelling());
    }
    println!();

    Some(res)
}