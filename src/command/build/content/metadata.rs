use crate::path::ContentPath;
use regex::Regex;
use serde::Deserialize;
use std::borrow::Cow;

#[derive(Deserialize, Debug)]
pub struct YamlHeader {
    pub date: String,
    #[serde(default)]
    #[serde(alias = "tag")]
    pub tags: Vec<String>,
    #[serde(default)]
    #[serde(alias = "flag")]
    pub flags: Vec<String>,
    #[serde(alias = "highlight")]
    #[serde(default)]
    pub highlights: Vec<HighlightMacro>,
}

pub struct Metadata {
    pub date: String,
    pub tags: Vec<String>,
    pub flags: Vec<String>,
    pub title: String,
    pub path: ContentPath,
    pub highlights: Vec<HighlightMacro>,
}

#[derive(Clone, Deserialize, Debug)]
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
