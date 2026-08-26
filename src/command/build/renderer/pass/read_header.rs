use super::HighlightRule;
use crate::command::build::renderer::context::Context;
use serde::Deserialize;

/// YAML フロントマターを読み、メタデータを Context に設定します。
///
/// djot にはフロントマターの構文がないため、本文をパースする前に
/// 呼び出し側で切り出した YAML を受け取ります。
pub fn read_header(yaml: &str, meta: &mut Context) -> anyhow::Result<()> {
    let header: YamlHeader = serde_yaml::from_str(yaml)?;
    meta.set_create_date(header.create_date);
    meta.set_last_update_date(header.last_update_date);
    meta.set_tags(header.tags);
    if let Some(h) = header.highlights {
        meta.set_highlights(h);
    }
    if let Some(pwd) = header.password {
        meta.set_password(pwd);
    }

    Ok(())
}

#[derive(Deserialize, Debug)]
struct YamlHeader {
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

    /// 暗号化時のパスワード
    pub password: Option<String>,

    /// コードハイライトのルール
    #[serde(alias = "highlight")]
    pub highlights: Option<Vec<HighlightRule>>,
}
