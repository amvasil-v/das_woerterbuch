#[derive(Debug)]
pub enum NounArticle {
    Der,
    Das,
    Die,
    Plural
}

impl NounArticle {
    pub fn to_string(&self) -> String {
        match self {
            Self::Der => "der",
            Self::Die => "die",
            Self::Das => "das",
            Self::Plural => "die"
        }.to_owned()
    }
}

#[derive(Debug)]
pub struct Noun {
    pub word: String,
    pub article: NounArticle,
    pub group_id: usize,
    pub translation: String
}

impl Noun {
    pub fn to_string(&self) -> String {
        return self.article.to_string() + " " + &self.word;
    }

    pub fn check_spelling(&self, answer: &str) -> bool {
        let expected = self.to_string();
        return expected.to_lowercase() == answer.to_lowercase();
    }
}

pub struct Database {
    pub groups: Vec<String>,
    pub nouns: Vec<Noun>,
}