use std::collections::HashMap;

use crate::words::*;
use calamine::{open_workbook, Reader, Xlsx};

pub fn fill_database(filename: &str) -> Database {
    let mut excel: Xlsx<_> = open_workbook(filename).unwrap();
    let r = excel.worksheet_range("Words").unwrap().unwrap();

    

    let mut db = Database {
        groups: vec![],
        words: HashMap::new()
    };
    for row in r.rows().skip(2) {
        let mut map: HashMap<usize, String> = row.iter().map(|dt| dt.to_string()).enumerate().collect();
        let pos = get_part_of_speech(&map);
        let word;

        word =  match pos {
            "n" => Box::new(Noun::new(&mut map, &mut db)) as Box<dyn Word>,
            "v" => Box::new(Verb::new(&mut map, &mut db)),
            _ => continue,
        };
        db.words.insert(word.get_word().to_owned(), word);
    }

    db
}

