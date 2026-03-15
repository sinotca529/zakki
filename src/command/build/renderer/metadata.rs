use super::pass::HighlightRule;
use crate::util::BloomFilter;
use anyhow::{Context as _, Result, anyhow};
use paste::paste;
use serde::{
    Serialize,
    ser::{Error as SerError, Serializer},
};
use std::path::PathBuf;

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

#[derive(Default, Serialize)]
pub struct Metadata {
    /// 記事を作成した日付 (yyyy-MM-dd)
    #[serde(serialize_with = "ser_unwrap", rename = "create")]
    create_date: Option<String>,

    /// 記事を最後に更新したした日付 (yyyy-MM-dd)
    #[serde(serialize_with = "ser_unwrap", rename = "update")]
    last_update_date: Option<String>,

    /// 記事につけられたタグ
    #[serde(serialize_with = "ser_unwrap")]
    tags: Option<Vec<String>>,

    /// 記事のタイトル
    #[serde(serialize_with = "ser_unwrap")]
    title: Option<String>,

    /// ルートから記事の出力先への相対パス
    #[serde(serialize_with = "ser_unwrap", rename = "path")]
    dst_rel_path: Option<PathBuf>,

    /// Bloom filter
    #[serde(skip)]
    bloom_filter: Option<BloomFilter>,

    /// コードハイライトの設定
    #[serde(skip)]
    highlights: Option<Vec<HighlightRule>>,

    /// 暗号化時のパスワード
    #[serde(skip)]
    password: Option<String>,

    /// 追加で読み込む JS 一覧
    #[serde(skip)]
    js_paths: Vec<String>,

    /// 追加で読み込む CSS 一覧
    #[serde(skip)]
    css_paths: Vec<String>,

    /// ソースファイルのパス
    #[serde(skip)]
    src_path: Option<PathBuf>,

    /// 下書きか否か
    pub is_draft: bool,

    /// サブページか否か
    pub is_sub: bool,

    /// 暗号化するか否か
    #[serde(skip)]
    pub to_encrypt: bool,
}

impl Metadata {
    try_get!(create_date, &String);
    try_get!(last_update_date, &String);
    try_get!(tags, &Vec<String>);
    try_get!(title, &String);
    try_get!(dst_rel_path, &PathBuf);
    try_get!(src_path, &PathBuf);
    try_get!(highlights, &Vec<HighlightRule>);
    try_get!(password, &String);
    try_get!(bloom_filter, &BloomFilter);

    pub fn css_list(&self) -> &Vec<String> {
        &self.css_paths
    }

    pub fn js_list(&self) -> &Vec<String> {
        &self.js_paths
    }

    setter!(create_date, String);
    setter!(last_update_date, String);
    setter!(tags, Vec<String>);
    setter!(title, String);
    setter!(dst_rel_path, PathBuf);
    setter!(src_path, PathBuf);
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

pub fn ser_unwrap<S, T>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    match value {
        Some(inner) => inner.serialize(serializer),
        None => Err(S::Error::custom("Expected value, found None")),
    }
}
