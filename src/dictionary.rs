use std::collections::HashMap;

use crate::words::*;
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

pub fn fill_database(filename: &str) -> Database {
    let mut excel: Xlsx<_> = open_workbook(filename).unwrap();
    let r = excel.worksheet_range("Words").unwrap().unwrap();

    let word_idx = 0;
    let pos_idx = 1;
    let trans_idx = 2;
    let group_idx = 3;
    let article_idx = 4;

    let mut db = Database {
        groups: vec![],
        words: HashMap::new()
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
            db.words.insert(word.to_owned(), Box::new(noun));
        } else if pos == "v" {
            let verb = Verb {
                word: word.to_owned(),
                group_id,
                translation: trans.to_owned(),
            };
            db.words.insert(word.to_owned(), Box::new(verb));
        }
    }

    db
}

