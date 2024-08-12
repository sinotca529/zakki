use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, path::PathBuf};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Flag {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "crypto")]
    Crypto,
}

#[derive(Default)]
pub struct Metadata {
    create_date: Option<String>,
    last_update_date: Option<String>,
    tags: Option<Vec<String>>,
    flags: Option<Vec<Flag>>,
    title: Option<String>,
    src_path: Option<PathBuf>,
    highlights: Option<Vec<HighlightMacro>>,
}

impl Metadata {
    pub fn create_date(&self) -> Result<&String> {
        self.create_date
            .as_ref()
            .with_context(|| anyhow!("create_date has not been set yet."))
    }

    pub fn last_update_date(&self) -> Result<&String> {
        self.last_update_date
            .as_ref()
            .with_context(|| anyhow!("last_update_date has not been set yet."))
    }

    pub fn tags(&self) -> Result<&Vec<String>> {
        self.tags
            .as_ref()
            .with_context(|| anyhow!("tags has not been set yet."))
    }

    pub fn flags(&self) -> Result<&Vec<Flag>> {
        self.flags
            .as_ref()
            .with_context(|| anyhow!("flags has not been set yet."))
    }

    pub fn title(&self) -> Result<&String> {
        self.title
            .as_ref()
            .with_context(|| anyhow!("title has not been set yet."))
    }

    pub fn src_path(&self) -> Result<&PathBuf> {
        self.src_path
            .as_ref()
            .with_context(|| anyhow!("src_path has not been set yet."))
    }

    pub fn highlights(&self) -> Result<&Vec<HighlightMacro>> {
        self.highlights
            .as_ref()
            .with_context(|| anyhow!("highlights has not been set yet."))
    }

    pub fn set_create_date(&mut self, create_date: String) {
        self.create_date = Some(create_date);
    }

    pub fn set_last_update_date(&mut self, last_update_date: String) {
        self.last_update_date = Some(last_update_date);
    }

    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.tags = Some(tags);
    }

    pub fn set_flags(&mut self, flags: Vec<Flag>) {
        self.flags = Some(flags);
    }

    pub fn set_title(&mut self, title: String) {
        self.title = Some(title);
    }

    pub fn set_src_path(&mut self, src_path: PathBuf) {
        self.src_path = Some(src_path);
    }

    pub fn set_highlights(&mut self, highlights: Vec<HighlightMacro>) {
        self.highlights = Some(highlights);
    }
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
