use super::pass::HighlightRule;
use crate::util::BloomFilter;
use anyhow::{Context as _, Result, anyhow};
use paste::paste;
use serde::Serialize;
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

#[derive(Default)]
pub struct Context {
    /// 記事を作成した日付 (yyyy-MM-dd)
    create_date: Option<String>,

    /// 記事を最後に更新した日付 (yyyy-MM-dd)
    last_update_date: Option<String>,

    /// 記事につけられたタグ
    tags: Option<Vec<String>>,

    /// 記事のタイトル
    title: Option<String>,

    /// ルートから記事の出力先への相対パス
    dst_rel_path: Option<PathBuf>,

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

    /// ソースファイルのパス
    src_path: Option<PathBuf>,

    /// 下書きか否か
    pub is_draft: bool,

    /// サブページか否か
    pub is_sub: bool,

    /// 暗号化するか否か
    pub to_encrypt: bool,
}

impl Context {
    try_get!(create_date, &String);
    try_get!(last_update_date, &String);
    try_get!(tags, &Vec<String>);
    try_get!(title, &String);
    try_get!(dst_rel_path, &PathBuf);
    try_get!(src_path, &PathBuf);
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

    /// 出力用メタデータに変換します。
    pub fn into_output(self) -> Result<Metadata> {
        Ok(Metadata {
            create: self.create_date.context("create_date has not been set")?,
            update: self
                .last_update_date
                .context("last_update_date has not been set")?,
            tags: self.tags.context("tags has not been set")?,
            title: self.title.context("title has not been set")?,
            path: self.dst_rel_path.context("dst_rel_path has not been set")?,
            bloom: self.bloom_filter.context("bloom_filter has not been set")?,
            is_sub: self.is_sub,
        })
    }
}

/// JSON として出力するメタデータ
#[derive(Serialize)]
pub struct Metadata {
    pub create: String,
    pub update: String,
    pub tags: Vec<String>,
    pub title: String,
    pub path: PathBuf,
    #[serde(skip)]
    pub bloom: BloomFilter,
    #[serde(skip)]
    pub is_sub: bool,
}
