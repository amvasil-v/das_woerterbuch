use std::collections::HashMap;

use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, EnumIter, PartialEq)]
pub enum PartOfSpeech {
    Noun,
    Verb,
    Adjective,
    Adverb,
    Preposition,
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

    fn new(map: &mut HashMap<usize, String>, db: &mut Database) -> Self
    where
        Self: Sized;

    fn get_help(&self) -> &str;

    fn get_group_id(&self) -> usize;

    fn get_pos(&self) -> PartOfSpeech;

    fn get_article(&self) -> Option<NounArticle> {
        None
    }
}

#[derive(Debug, EnumIter, PartialEq, Eq, Clone, Copy)]
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

    pub fn to_answer_buller(&self) -> String {
        match self {
            Self::Plural => "die (plural)".to_string(),
            _ => self.to_string(),
        }
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

    fn get_group_id(&self) -> usize {
        self.group_id
    }

    fn get_pos(&self) -> PartOfSpeech {
        unimplemented!()
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

pub fn capitalize_noun(noun: &str) -> String {
    noun[0..1].to_uppercase().to_string()
    + &noun[1..]
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
            + &capitalize_noun(&self.common.word)
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
            article: get_article(&map.remove(&ARTICLE_IDX).unwrap()).unwrap(),
        }
    }

    fn get_group_id(&self) -> usize {
        self.common.get_group_id()
    }

    fn get_pos(&self) -> PartOfSpeech {
        PartOfSpeech::Noun
    }

    fn get_article(&self) -> Option<NounArticle> {
        Some(self.article.clone())
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
            common: WordCommon::new(map, db),
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

    fn get_group_id(&self) -> usize {
        self.common.get_group_id()
    }

    fn get_pos(&self) -> PartOfSpeech {
        PartOfSpeech::Verb
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
            common: WordCommon::new(map, db),
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

    fn get_group_id(&self) -> usize {
        self.common.get_group_id()
    }

    fn get_pos(&self) -> PartOfSpeech {
        PartOfSpeech::Adjective
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
            common: WordCommon::new(map, db),
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

    fn get_group_id(&self) -> usize {
        self.common.get_group_id()
    }

    fn get_pos(&self) -> PartOfSpeech {
        PartOfSpeech::Adverb
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
            common: WordCommon::new(map, db),
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

    fn get_group_id(&self) -> usize {
        self.common.get_group_id()
    }

    fn get_pos(&self) -> PartOfSpeech {
        PartOfSpeech::Preposition
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
