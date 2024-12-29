use std::{borrow::Cow, path::PathBuf};
use regex::Regex;
use serde::Deserialize;
use anyhow::{anyhow, Context, Result};

#[derive(Default)]
pub struct RenderingContext {
    /// ルートディレクトリから記事の出力先ディレクトリへの相対パス
    dst_path: Option<PathBuf>,

    /// コードハイライトの設定
    highlights: Option<Vec<HighlightMacro>>,

    /// 暗号化時のパスワード
    password: Option<String>,

    /// 読み込む JS 一覧
    js_paths: Vec<PathBuf>,

    /// 読み込む CSS 一覧
    css_paths: Vec<PathBuf>,
}

impl RenderingContext {
    pub fn push_js_path(&mut self, path: impl Into<PathBuf>) {
        self.js_paths.push(path.into());
    }

    pub fn push_css_path(&mut self, path: impl Into<PathBuf>) {
        self.css_paths.push(path.into());
    }

    pub fn set_dst_path(&mut self, dst_path: PathBuf) {
        self.dst_path = Some(dst_path);
    }

    pub fn dst_path(&self) -> Result<&PathBuf> {
        self.dst_path
            .as_ref()
            .with_context(|| anyhow!("dst_path has not been set yet."))
    }

    pub fn password(&self) -> Option<&String> {
        self.password.as_ref()
    }

    pub fn set_password(&mut self, password: Option<String>) {
        self.password = password;
    }

    pub fn set_highlights(&mut self, highlights: Vec<HighlightMacro>) {
        self.highlights = Some(highlights);
    }

    pub fn highlights(&self) -> Result<&Vec<HighlightMacro>> {
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

#[derive(Clone, Deserialize, Debug)]
pub struct HighlightMacro {
    delim: [String; 2],
    style: String,
}

impl HighlightMacro {
    pub fn replace_all<'a>(&self, code: &'a str) -> Cow<'a, str> {
        if let Ok(pat) = Regex::new(&format!("{}(.*?){}", &self.delim[0], &self.delim[1])) {
            pat.replace_all(code, format!("<span style=\"{}\">$1</span>", &self.style))
        } else {
            code.into()
        }
    }
}
