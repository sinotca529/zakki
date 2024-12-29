use std::path::PathBuf;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::util::BloomFilter;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Flag {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "crypto")]
    Crypto,
}

#[derive(Default, Serialize)]
pub struct PageMetadata {
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

    /// ルートから記事の出力先への相対パス
    #[serde(rename = "path", serialize_with = "serialize_option")]
    dst_path_from_root: Option<PathBuf>,

    /// Bloom filter
    #[serde(skip)]
    bloom_filter: Option<BloomFilter>,
}

impl PageMetadata {
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

    pub fn set_dst_path_from_root(&mut self, dst_path_from_root: PathBuf) {
        self.dst_path_from_root = Some(dst_path_from_root);
    }

    pub fn set_bloom_filter(&mut self, bloom_filter: BloomFilter) {
        self.bloom_filter = Some(bloom_filter);
    }

    pub fn take_bloom_filter(&mut self) -> Option<BloomFilter> {
        self.bloom_filter.take()
    }
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
