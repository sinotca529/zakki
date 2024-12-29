use super::{page_metadata::{Flag, PageMetadata}, rendering_context::{HighlightMacro, RenderingContext}};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct YamlHeader {
    /// 記事の作成日
    #[serde(rename = "create")]
    pub create_date: String,

    /// 記事の最終更新日
    #[serde(rename = "update")]
    pub last_update_date: String,

    /// 記事につけられたタグ
    #[serde(default)]
    #[serde(alias = "tag")]
    pub tags: Vec<String>,

    /// 記事を HTML に変換する際に使用するフラグ
    #[serde(default)]
    #[serde(alias = "flag")]
    pub flags: Vec<Flag>,

    /// 暗号化時のパスワード
    pub password: Option<String>,

    /// コードハイライトのルール
    #[serde(alias = "highlight")]
    #[serde(default)]
    pub highlights: Vec<HighlightMacro>,
}

impl YamlHeader {
    pub fn merge_into(self, md: &mut PageMetadata, ctxt: &mut RenderingContext) {
        md.set_create_date(self.create_date);
        md.set_last_update_date(self.last_update_date);
        md.set_tags(self.tags);
        md.set_flags(self.flags);
        ctxt.set_password(self.password);
        ctxt.set_highlights(self.highlights);
    }
}
