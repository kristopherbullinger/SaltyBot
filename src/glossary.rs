use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::FromIterator;

static GLOSSARY_RAW: &str = include_str!("../glossary/glossary.json");
static GLOSSARY: OnceCell<HashMap<String, Term>> = OnceCell::new();
pub fn init() -> anyhow::Result<()> {
    let glossary: Vec<GlossaryEntry> = serde_json::from_str(GLOSSARY_RAW)?;
    let _ = GLOSSARY.set(HashMap::from_iter(glossary.into_iter().map(|gl| {
        let (mut name, term) = gl.into_term();
        name.make_ascii_lowercase();
        (name, term)
    })));
    Ok(())
}

pub fn get<A: AsRef<str>>(s: A) -> Option<&'static Term> {
    GLOSSARY.get().and_then(|gl| gl.get(s.as_ref()))
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlossaryEntry {
    pub term: String,
    pub def: String,
    // pub video: Option<Vec<String>>,
    // pub jp: Option<String>,
    #[serde(default)]
    pub altterm: Vec<String>,
    #[serde(default)]
    pub games: Vec<String>,
    // pub image: Option<Vec<String>>,
}

impl GlossaryEntry {
    fn into_term(self) -> (String, Term) {
        (
            self.term,
            Term {
                def: self.def,
                altterm: self.altterm,
                games: self.games,
            },
        )
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Term {
    pub def: String,
    #[serde(default)]
    pub altterm: Vec<String>,
    #[serde(default)]
    pub games: Vec<String>,
}
