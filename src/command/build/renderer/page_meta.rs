use crate::{command::build::renderer::url::Url, util::BloomFilter};
use serde::Serialize;

/// JSON として出力するメタデータ
#[derive(Serialize)]
pub struct PageMetadata {
    pub create: String,
    pub update: String,
    pub tags: Vec<String>,
    pub title: String,
    pub path: Url,
    #[serde(skip)]
    pub bloom: BloomFilter,
    #[serde(skip)]
    pub is_sub: bool,
    #[serde(skip)]
    pub is_private: bool,
}
