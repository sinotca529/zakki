use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::Deserialize;
use std::{borrow::Cow, path::PathBuf};

#[derive(Default)]
pub struct RenderingContext {
    /// コードハイライトの設定
    highlights: Option<Vec<HighlightRule>>,

    /// 暗号化時のパスワード
    password: Option<String>,

    /// 追加で読み込む JS 一覧
    js_paths: Vec<PathBuf>,

    /// 追加で読み込む CSS 一覧
    css_paths: Vec<PathBuf>,
}

impl RenderingContext {
    pub fn push_js_path(&mut self, path: impl Into<PathBuf>) {
        self.js_paths.push(path.into());
    }

    pub fn push_css_path(&mut self, path: impl Into<PathBuf>) {
        self.css_paths.push(path.into());
    }

    pub fn password(&self) -> Option<&String> {
        self.password.as_ref()
    }

    pub fn set_password(&mut self, password: Option<String>) {
        self.password = password;
    }

    pub fn set_highlights(&mut self, highlights: Vec<HighlightRule>) {
        self.highlights = Some(highlights);
    }

    pub fn highlights(&self) -> Result<&Vec<HighlightRule>> {
        self.highlights
            .as_ref()
            .with_context(|| anyhow!("highlights has not been set yet."))
    }

    pub fn css_list(&self) -> &Vec<PathBuf> {
        &self.css_paths
    }

    pub fn js_list(&self) -> &Vec<PathBuf> {
        &self.js_paths
    }
}

