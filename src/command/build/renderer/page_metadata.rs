use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::util::BloomFilter;
use paste::paste;

macro_rules! getter {
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
    build_root_to_dst: Option<PathBuf>,

    /// Bloom filter
    #[serde(skip)]
    bloom_filter: Option<BloomFilter>,
}

impl PageMetadata {
    getter!(create_date, &String);
    getter!(last_update_date, &String);
    getter!(tags, &Vec<String>);
    getter!(flags, &Vec<Flag>);
    getter!(title, &String);
    getter!(build_root_to_dst, &PathBuf);

    setter!(create_date, String);
    setter!(last_update_date, String);
    setter!(tags, Vec<String>);
    setter!(flags, Vec<Flag>);
    setter!(title, String);
    setter!(build_root_to_dst, PathBuf);
    setter!(bloom_filter, BloomFilter);

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
