use super::HighlightRule;
use crate::command::build::renderer::metadata::Metadata;
use MetadataBlockKind::YamlStyle;
use anyhow::bail;
use pulldown_cmark::{Event, Tag};
use pulldown_cmark::{MetadataBlockKind, TagEnd};
use serde::Deserialize;

pub fn read_header_pass(events: &mut Vec<Event>, meta: &mut Metadata) -> anyhow::Result<()> {
    let header = events
        .iter()
        .skip_while(|e| !matches!(e, Event::Start(Tag::MetadataBlock(YamlStyle))))
        .take_while(|e| !matches!(e, Event::End(TagEnd::MetadataBlock(YamlStyle))))
        .filter_map(|e| match e {
            Event::Text(t) => Some(t),
            _ => None,
        })
        .next();

    let Some(header) = header else {
        bail!("Yaml header is not existing.")
    };

    let header: YamlHeader = serde_yaml::from_str(header)?;
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
