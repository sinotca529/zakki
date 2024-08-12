use super::{Flag, HighlightMacro, Metadata};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct YamlHeader {
    /// 記事の作成日
    #[serde(rename = "create")]
    pub create_date: String,

    /// 記事の最終更新日
    #[serde(rename = "update")]
    pub last_update_date: String,
    #[serde(default)]
    #[serde(alias = "tag")]

    /// 記事につけられたタグ
    pub tags: Vec<String>,
    #[serde(default)]
    #[serde(alias = "flag")]

    /// 記事を HTML に変換する際に使用するフラグ
    pub flags: Vec<Flag>,
    #[serde(alias = "highlight")]
    #[serde(default)]

    /// コードハイライトのルール
    pub highlights: Vec<HighlightMacro>,
}

impl YamlHeader {
    pub fn merge_into(self, md: &mut Metadata) {
        md.set_create_date(self.create_date);
        md.set_last_update_date(self.last_update_date);
        md.set_tags(self.tags);
        md.set_flags(self.flags);
        md.set_highlights(self.highlights);
    }
}
