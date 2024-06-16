use derive_builder::Builder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, path::PathBuf};

#[derive(Deserialize)]
struct PartialMetadata {
    date: String,
    #[serde(rename(deserialize = "tag"))]
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    crypto: bool,
    #[serde(rename(deserialize = "highlight"))]
    #[serde(default)]
    highlights: Vec<HighlightMacro>,
}

#[derive(Deserialize, Serialize, Builder)]
pub struct Metadata {
    date: String,
    tags: Vec<String>,
    crypto: bool,
    #[serde(skip_serializing)]
    highlights: Vec<HighlightMacro>,
    title: String,
    path: PathBuf,
}

impl Metadata {
    pub fn highlights(&self) -> &Vec<HighlightMacro> {
        &self.highlights
    }

    pub fn tags(&self) -> &Vec<String> {
        &self.tags
    }

    pub fn date(&self) -> &String {
        &self.date
    }

    pub fn crypto(&self) -> bool {
        self.crypto
    }
}

impl MetadataBuilder {
    pub fn read_yaml(&mut self, yaml: &str) -> &mut Self {
        if let Ok(m) = serde_yaml::from_str::<PartialMetadata>(yaml) {
            self.date(m.date)
                .tags(m.tags)
                .crypto(m.crypto)
                .highlights(m.highlights);
        }
        self
    }
}

#[derive(Clone, Deserialize)]
pub struct HighlightMacro {
    before: String,
    after: String,
}

impl HighlightMacro {
    pub fn replace_all<'a>(&self, code: &'a str) -> Cow<'a, str> {
        if let Ok(pat) = Regex::new(&self.before) {
            pat.replace_all(code, &self.after)
        } else {
            code.into()
        }
    }
}
