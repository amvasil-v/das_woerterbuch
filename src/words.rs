use std::collections::HashMap;

use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, EnumIter, PartialEq)]
pub enum PartOfSpeech {
    Noun,
    Verb,
}

fn umlaut_normalize(word: &str) -> String {
    word.replace("ü", "ue")
        .replace("ä", "ae")
        .replace("ö", "oe")
        .replace("ß", "ss")
}

pub trait Word {
    fn pos_str(&self) -> &'static str {
        unimplemented!()
    }

    fn translation(&self) -> &str;

    fn spelling(&self) -> String {
        self.get_word().to_owned()
    }

    fn check_spelling(&self, answer: &str) -> bool {
        let low_ans = answer.to_lowercase();
        let spelling = self.spelling().to_lowercase();
        if low_ans == spelling {
            true
        } else if low_ans == umlaut_normalize(&spelling) {
            true
        } else {
            false
        }
    }

    fn get_word(&self) -> &str;

    fn new(map: &mut HashMap<usize, String>, db: &mut Database) -> Self where Self:Sized;

    fn get_help(&self) -> &str;
}

#[derive(Debug)]
pub enum NounArticle {
    Der,
    Das,
    Die,
    Plural,
}

impl NounArticle {
    pub fn to_string(&self) -> String {
        match self {
            Self::Der => "der",
            Self::Die => "die",
            Self::Das => "das",
            Self::Plural => "die",
        }
        .to_owned()
    }
}

#[derive(Debug)]
pub struct WordCommon {
    pub word: String,
    pub group_id: usize,
    pub translation: String,
    pub help: String,
}

const WORD_IDX: usize = 0;
const POS_IDX: usize = 1;
const TRANSLATION_IDX: usize = 2;
const GROUP_IDX: usize = 3;
const ARTICLE_IDX: usize = 4;
const HELP_IDX: usize = 7;

pub fn get_part_of_speech(map: &HashMap<usize, String>) -> &str {
    return &map[&POS_IDX];
}

impl Word for WordCommon {
    fn new(map: &mut HashMap<usize, String>, db: &mut Database) -> Self {
        Self {
            word: map.remove(&WORD_IDX).unwrap(),
            group_id: db.get_group_id(&map.remove(&GROUP_IDX).unwrap()),
            translation: map.remove(&TRANSLATION_IDX).unwrap(),
            help: map.remove(&HELP_IDX).unwrap(),
        }
    }

    fn translation(&self) -> &str {
        &self.translation
    }

    fn get_word(&self) -> &str {
        &self.word
    }

    fn get_help(&self) -> &str {
        &self.help
    }
}

fn get_article(s: &str) -> Result<NounArticle, String> {
    Ok(match s {
        "der" => NounArticle::Der,
        "das" => NounArticle::Das,
        "die" => NounArticle::Die,
        "pl" => NounArticle::Plural,
        _ => return Err(format!("Unknown article {:?}", s)),
    })
}

#[derive(Debug)]
pub struct Noun {
    pub common: WordCommon,
    pub article: NounArticle,
}

impl Word for Noun {
    fn pos_str(&self) -> &'static str {
        "noun"
    }

    fn spelling(&self) -> String {
        self.article.to_string()
            + " "
            + &self.common.word[0..1].to_uppercase().to_string()
            + &self.common.word[1..]
    }

    fn translation(&self) -> &str {
        self.common.translation()
    }

    fn get_word(&self) -> &str {
        self.common.get_word()
    }

    fn get_help(&self) -> &str {
        self.common.get_help()
    }

    fn new(map: &mut HashMap<usize, String>, db: &mut Database) -> Self {
        Self {
            common: WordCommon::new(map, db),
            article: get_article(&map.remove(&ARTICLE_IDX).unwrap()).unwrap()
        }
    }

    
}

#[derive(Debug)]
pub struct Verb {
    pub common: WordCommon,
}

impl Word for Verb {
    fn pos_str(&self) -> &'static str {
        "verb"
    }

    fn new(map: &mut HashMap<usize, String>, db: &mut Database) -> Self {
        Self {
            common: WordCommon::new(map, db)
        }
    }

    fn translation(&self) -> &str {
        self.common.translation()
    }

    fn get_word(&self) -> &str {
        self.common.get_word()
    }
    
    fn get_help(&self) -> &str {
        self.common.get_help()
    }

}

#[derive(Debug)]
pub struct Adjective {
    pub common: WordCommon,
}

impl Word for Adjective {
    fn pos_str(&self) -> &'static str {
        "adj"
    }

    fn new(map: &mut HashMap<usize, String>, db: &mut Database) -> Self {
        Self {
            common: WordCommon::new(map, db)
        }
    }

    fn translation(&self) -> &str {
        self.common.translation()
    }

    fn get_word(&self) -> &str {
        self.common.get_word()
    }
    
    fn get_help(&self) -> &str {
        self.common.get_help()
    }
}

#[derive(Debug)]
pub struct Adverb {
    pub common: WordCommon,
}

impl Word for Adverb {
    fn pos_str(&self) -> &'static str {
        "adv"
    }

    fn new(map: &mut HashMap<usize, String>, db: &mut Database) -> Self {
        Self {
            common: WordCommon::new(map, db)
        }
    }

    fn translation(&self) -> &str {
        self.common.translation()
    }

    fn get_word(&self) -> &str {
        self.common.get_word()
    }
    
    fn get_help(&self) -> &str {
        self.common.get_help()
    }
}

#[derive(Debug)]
pub struct Preposition {
    pub common: WordCommon,
}

impl Word for Preposition {
    fn pos_str(&self) -> &'static str {
        "preposition"
    }

    fn new(map: &mut HashMap<usize, String>, db: &mut Database) -> Self {
        Self {
            common: WordCommon::new(map, db)
        }
    }

    fn translation(&self) -> &str {
        self.common.translation()
    }

    fn get_word(&self) -> &str {
        self.common.get_word()
    }
    
    fn get_help(&self) -> &str {
        self.common.get_help()
    }
}
pub struct Database {
    pub groups: Vec<String>,
    pub words: HashMap<String, Box<dyn Word>>,
}

impl Database {
    pub fn get_group_id(&mut self, name: &str) -> usize {
        match self.groups.iter().position(|g| g == name) {
            None => {
                self.groups.push(name.to_owned());
                self.groups.len() - 1
            }
            Some(i) => i,
        }
    }
}