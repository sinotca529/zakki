use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, path::PathBuf};

use crate::util::BloomFilter;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Flag {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "crypto")]
    Crypto,
}

#[derive(Default, Serialize)]
pub struct Metadata {
    /// 記事を作成した日付 (yyyy-MM-dd)
    #[serde(rename = "create", serialize_with = "serialize_option")]
    create_date: Option<String>,

    /// 記事を最後に更新したした日付 (yyyy-MM-dd)
    #[serde(rename = "update", serialize_with = "serialize_option")]
    last_update_date: Option<String>,

    /// 記事につけられたタグ
    #[serde(serialize_with = "serialize_option")]
    tags: Option<Vec<String>>,

    /// 記事につけられたフラグ
    /// HTML への変換時に利用する
    #[serde(serialize_with = "serialize_option")]
    flags: Option<Vec<Flag>>,

    /// 記事のタイトル
    #[serde(serialize_with = "serialize_option")]
    title: Option<String>,

    /// 雑記の出力先ルートディレクトリから、記事の出力先ディレクトリへの相対パス
    #[serde(rename = "path", serialize_with = "serialize_option")]
    dst_path_from_root: Option<PathBuf>,

    /// Bloom filter
    #[serde(skip)]
    bloom_filter: Option<BloomFilter>,

    /// 記事の出力先ディレクトリへ
    #[serde(skip)]
    dst_path: Option<PathBuf>,

    /// コードハイライトの設定
    /// HTML への変換時に利用する
    #[serde(skip)]
    highlights: Option<Vec<HighlightMacro>>,

    /// 暗号化時のパスワード
    #[serde(skip)]
    password: Option<String>,
}

fn serialize_option<T: Serialize, S: serde::Serializer>(
    v: &Option<T>,
    s: S,
) -> Result<S::Ok, S::Error> {
    match v.as_ref() {
        Some(v) => v.serialize(s),
        None => Err(serde::ser::Error::custom("Expected some, but found None")),
    }
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

    pub fn dst_path(&self) -> Result<&PathBuf> {
        self.dst_path
            .as_ref()
            .with_context(|| anyhow!("dst_path has not been set yet."))
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

    pub fn set_dst_path(&mut self, dst_path: PathBuf) {
        self.dst_path = Some(dst_path);
    }

    pub fn set_bloom_filter(&mut self, bloom_filter: BloomFilter) {
        self.bloom_filter = Some(bloom_filter);
    }

    pub fn set_dst_path_from_root(&mut self, dst_path_from_root: PathBuf) {
        self.dst_path_from_root = Some(dst_path_from_root);
    }

    pub fn set_highlights(&mut self, highlights: Vec<HighlightMacro>) {
        self.highlights = Some(highlights);
    }

    pub fn password(&self) -> Option<&String> {
        self.password.as_ref()
    }

    pub fn set_password(&mut self, password: Option<String>) {
        self.password = password;
    }

    pub fn take_bloom_filter(&mut self) -> Option<BloomFilter> {
        self.bloom_filter.take()
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
