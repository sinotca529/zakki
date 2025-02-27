use crate::util::BloomFilter;
use anyhow::{Context as _, Result, anyhow};
use paste::paste;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::pass::HighlightRule;

macro_rules! try_get {
    ($field:ident, $return_type:ty) => {
        pub fn $field(&self) -> Result<$return_type> {
            self.$field
                .as_ref()
                .with_context(|| anyhow!(concat!(stringify!($field), " has not been set yet.")))
        }
    };
}

macro_rules! setter {
    ($field:ident, $type:ty) => {
        paste! {
            pub fn [<set_ $field>](&mut self, $field: $type) {
                self.$field = Some($field);
            }
        }
    };
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Flag {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "crypto")]
    Crypto,
}

#[derive(Default)]
pub struct Context {
    /// 記事を作成した日付 (yyyy-MM-dd)
    create_date: Option<String>,

    /// 記事を最後に更新したした日付 (yyyy-MM-dd)
    last_update_date: Option<String>,

    /// 記事につけられたタグ
    tags: Option<Vec<String>>,

    /// 記事につけられたフラグ
    flags: Option<Vec<Flag>>,

    /// 記事のタイトル
    title: Option<String>,

    /// ルートから記事の出力先への相対パス
    build_root_to_dst: Option<PathBuf>,

    /// Bloom filter
    bloom_filter: Option<BloomFilter>,

    /// コードハイライトの設定
    highlights: Option<Vec<HighlightRule>>,

    /// 暗号化時のパスワード
    password: Option<String>,

    /// 追加で読み込む JS 一覧
    js_paths: Vec<String>,

    /// 追加で読み込む CSS 一覧
    css_paths: Vec<String>,
}

impl Context {
    try_get!(create_date, &String);
    try_get!(last_update_date, &String);
    try_get!(tags, &Vec<String>);
    try_get!(flags, &Vec<Flag>);
    try_get!(title, &String);
    try_get!(build_root_to_dst, &PathBuf);
    try_get!(highlights, &Vec<HighlightRule>);
    try_get!(password, &String);

    pub fn css_list(&self) -> &Vec<String> {
        &self.css_paths
    }

    pub fn js_list(&self) -> &Vec<String> {
        &self.js_paths
    }

    setter!(create_date, String);
    setter!(last_update_date, String);
    setter!(tags, Vec<String>);
    setter!(flags, Vec<Flag>);
    setter!(title, String);
    setter!(build_root_to_dst, PathBuf);
    setter!(bloom_filter, BloomFilter);
    setter!(password, String);
    setter!(highlights, Vec<HighlightRule>);

    pub fn push_js_path(&mut self, path: impl Into<String>) {
        self.js_paths.push(path.into());
    }

    pub fn push_css_path(&mut self, path: impl Into<String>) {
        self.css_paths.push(path.into());
    }
}

impl TryInto<Metadata> for Context {
    type Error = anyhow::Error;

    fn try_into(self) -> std::result::Result<Metadata, Self::Error> {
        macro_rules! try_take {
            ($field:ident) => {
                self.$field
                    .with_context(|| anyhow!(concat!(stringify!($field), " has not been set.")))?
            };
        }

        Ok(Metadata {
            create: try_take!(create_date),
            update: try_take!(last_update_date),
            tags: try_take!(tags),
            flags: try_take!(flags),
            title: try_take!(title),
            path: try_take!(build_root_to_dst),
            bloom_filter: try_take!(bloom_filter),
        })
    }
}

#[derive(Default, Serialize)]
pub struct Metadata {
    /// 記事を作成した日付 (yyyy-MM-dd)
    create: String,

    /// 記事を最後に更新したした日付 (yyyy-MM-dd)
    update: String,

    /// 記事につけられたタグ
    tags: Vec<String>,

    /// 記事につけられたフラグ
    flags: Vec<Flag>,

    /// 記事のタイトル
    title: String,

    /// ルートから記事の出力先への相対パス
    path: PathBuf,

    /// Bloom filter
    #[serde(skip)]
    bloom_filter: BloomFilter,
}

impl Metadata {
    pub fn update(&self) -> &String {
        &self.update
    }

    pub fn bloom_filter(&self) -> &BloomFilter {
        &self.bloom_filter
    }
}
