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

pub fn check_spelling_simple(answer: &str, expected: &str) -> bool {
    let low_ans = answer.to_lowercase();
    let spelling = expected.to_lowercase();
    if low_ans == spelling {
        true
    } else if low_ans == umlaut_normalize(&spelling) {
        true
    } else {
        false
    }
}

pub fn check_spelling_perfect(answer: &str, expected: &dyn Word) -> bool {
    let mut frags = answer.split_whitespace();
    let first = match frags.next() {
        None => {
            return false;
        }
        Some(s) => s,
    };
    let second = match frags.next() {
        None => {
            return false;
        }
        Some(s) => s,
    };
    if first != "hat" && first != "ist" {
        return false;
    }
    if !expected.get_verb_perfect_verb().unwrap().to_string().contains(first) {
        return false;
    }
    check_spelling_simple(second, expected.get_verb_perfect().unwrap())
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
        check_spelling_simple(answer, &self.spelling())
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

    fn get_verb_praeteritum(&self) -> Option<&str> {
        None
    }

    fn get_verb_perfect(&self) -> Option<&str> {
        None
    }

    fn get_verb_perfect_verb(&self) -> Option<&PerfectVerb> {
        None
    }

    fn get_verb_perfect_full(&self) -> Option<String> {
        None
    }

    fn get_verb_present_third(&self) -> Option<&str> {
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
const PERFECT_IDX: usize = 5;
const PRAETERITUM_IDX: usize = 6;
const PERFECT_VERB_IDX: usize = 8;
const PRESENT_THIRD_IDX: usize = 9;

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
    noun[0..1].to_uppercase().to_string() + &noun[1..]
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
        self.article.to_string() + " " + &capitalize_noun(&self.common.word)
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
pub enum PerfectVerb {
    Haben,
    Sein,
    Both,
}

impl PerfectVerb {
    pub fn from(s: &str) -> Option<Self> {
        Some(match s.trim() {
            "hat" => PerfectVerb::Haben,
            "ist" => PerfectVerb::Sein,
            "hat/ist" => PerfectVerb::Both,
            s if s.is_empty() => {
                return None;
            }
            _ => {
                panic!("Unknown perfect verb {}", s);
            }
        })
    }

    pub fn from_option(s: Option<String>) -> Option<Self> {
        Self::from(&s?)
    }

    pub fn to_string(&self) -> String {
        match &self {
            Self::Haben => "hat",
            Self::Sein => "ist",
            Self::Both => "hat/ist",
        }
        .to_owned()
    }
}

#[derive(Debug)]
pub struct Verb {
    pub common: WordCommon,
    pub praeteritum: String,
    pub perfect: String,
    pub perfect_verb: Option<PerfectVerb>,
    pub present_third: String,
}

impl Word for Verb {
    fn pos_str(&self) -> &'static str {
        "verb"
    }

    fn new(map: &mut HashMap<usize, String>, db: &mut Database) -> Self {
        Self {
            common: WordCommon::new(map, db),
            praeteritum: map.remove(&PRAETERITUM_IDX).unwrap_or_default(),
            perfect: map.remove(&PERFECT_IDX).unwrap_or_default(),
            perfect_verb: PerfectVerb::from_option(map.remove(&PERFECT_VERB_IDX)),
            present_third: map.remove(&PRESENT_THIRD_IDX).unwrap_or_default(),
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

    fn get_verb_praeteritum(&self) -> Option<&str> {
        Some(&self.praeteritum)
    }

    fn get_verb_present_third(&self) -> Option<&str> {
        Some(&self.present_third)
    }

    fn get_verb_perfect_verb(&self) -> Option<&PerfectVerb> {
        self.perfect_verb.as_ref()
    }

    fn get_verb_perfect(&self) -> Option<&str> {
        Some(&self.perfect)
    }

    fn get_verb_perfect_full(&self) -> Option<String> {
        Some(format!(
            "{} {}",
            self.get_verb_perfect_verb()?.to_string(),
            self.get_verb_perfect()?
        ))
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
