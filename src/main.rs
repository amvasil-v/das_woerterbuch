mod game;
mod words;

use crate::game::*;
use crate::words::*;
use rand::Rng;
use strum::IntoEnumIterator;

use calamine::{open_workbook, Reader, Xlsx};

fn get_article(s: &str) -> Result<NounArticle, String> {
    Ok(match s {
        "der" => NounArticle::Der,
        "das" => NounArticle::Das,
        "die" => NounArticle::Die,
        "pl" => NounArticle::Plural,
        _ => return Err(format!("Unknown article {:?}", s)),
    })
}

fn fill_database(filename: &str) -> Database {
    let mut excel: Xlsx<_> = open_workbook(filename).unwrap();
    let r = excel.worksheet_range("Words").unwrap().unwrap();

    let word_idx = 0;
    let pos_idx = 1;
    let trans_idx = 2;
    let group_idx = 3;
    let article_idx = 4;

    let mut db = Database {
        groups: vec![],
        nouns: vec![],
        verbs: vec![],
    };
    for row in r.rows().skip(2) {
        let word = row[word_idx].get_string().unwrap();
        let pos = row[pos_idx].get_string().unwrap();
        let trans = row[trans_idx].get_string().unwrap();
        let group = row[group_idx].get_string().unwrap();

        let group_id = match db.groups.iter().position(|g| g == group) {
            None => {
                db.groups.push(group.to_owned());
                db.groups.len() - 1
            }
            Some(i) => i,
        };

        if pos == "n" {
            let noun = Noun {
                word: word.to_owned(),
                article: get_article(row[article_idx].get_string().unwrap()).unwrap(),
                group_id,
                translation: trans.to_owned(),
            };
            db.nouns.push(noun);
        } else if pos == "v" {
            let verb = Verb {
                word: word.to_owned(),
                group_id,
                translation: trans.to_owned(),
            };
            db.verbs.push(verb);
        }
    }

    db
}

fn main() {
    let db = fill_database("woerterbuch.xlsx");
    let mut rng = rand::thread_rng();

    println!("Type \"exit\" to quit game\n");
    let mut pos = PartOfSpeech::iter();
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
                exercise_translate_to_de(noun)
            },
            PartOfSpeech::Verb => {
                let idx = rng.gen_range(0..db.verbs.len());
                let verb = &db.verbs[idx];
                exercise_translate_to_de(verb)
            }
        };

        if let None = result {
            break;
        }
    }
}
