use crate::words::*;
use std::io;
use colored::Colorize;

pub fn exercise_translate_to_de(noun: &Noun) -> Option<bool>
{
    println!("Translate to German: {} (noun)", noun.translation);
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .expect("Failed to read line");
    answer = answer.trim().to_lowercase();
    if answer == "exit" || answer == "quit" {
        return None;
    }
    let res = noun.check_spelling(&answer);

    if res {
        println!("{}", "Correct!".bold().green());
    } else {
        println!("{} The word is {}", "Incorrect!".bold().red(), noun.to_string());
    }
    println!();

    Some(res)
}