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
    fn pos_str(&self) -> &'static str;

    fn translation(&self) -> &str;

    fn spelling(&self) -> String;

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
pub struct Noun {
    pub word: String,
    pub article: NounArticle,
    pub group_id: usize,
    pub translation: String,
}

impl Word for Noun {
    fn pos_str(&self) -> &'static str {
        "noun"
    }

    fn spelling(&self) -> String {
        self.article.to_string()
            + " "
            + &self.word[0..1].to_uppercase().to_string()
            + &self.word[1..]
    }

    fn translation(&self) -> &str {
        &self.translation
    }

    fn get_word(&self) -> &str {
        &self.word
    }
}

#[derive(Debug)]
pub struct Verb {
    pub word: String,
    pub group_id: usize,
    pub translation: String,
}

impl Word for Verb {
    fn pos_str(&self) -> &'static str {
        "verb"
    }

    fn spelling(&self) -> String {
        self.word.clone()
    }

    fn translation(&self) -> &str {
        &self.translation
    }

    fn get_word(&self) -> &str {
        &self.word
    }
}

pub struct Database {
    pub groups: Vec<String>,
    pub nouns: Vec<Noun>,
    pub verbs: Vec<Verb>,
}
