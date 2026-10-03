use std::collections::BTreeSet;

use crate::{command::build::renderer::url::Url, search::BloomFilter, util::Date};
use serde::Serialize;

/// JSON として出力するメタデータ
#[derive(Serialize)]
pub struct PageMetadata {
    pub create: Date,
    pub update: Date,
    pub tags: Vec<String>,
    pub title: String,
    pub path: Url,
    #[serde(skip)]
    pub bloom: BloomFilter,
    #[serde(skip)]
    pub is_sub: bool,

    /// 記事をまとめるディレクトリの `index.md` かどうか
    /// 一覧のカードの見た目を変えるために使います
    #[serde(skip)]
    pub is_group: bool,
    #[serde(skip)]
    pub is_private: bool,

    /// 等幅フォントで描かれる文字
    /// 全記事ぶんを集めてからサブセットを作るため、ここでは記事ごとに返します。
    #[serde(skip)]
    pub monospace_chars: BTreeSet<char>,
}
