use crate::util::BloomFilter;
use serde::Serialize;
use std::path::PathBuf;

/// JSON として出力するメタデータ
#[derive(Serialize)]
pub struct PageMetadata {
    pub create: String,
    pub update: String,
    pub tags: Vec<String>,
    pub title: String,
    pub path: PathBuf,
    #[serde(skip)]
    pub bloom: BloomFilter,
    #[serde(skip)]
    pub is_sub: bool,
    #[serde(skip)]
    pub is_privte: bool,
}
