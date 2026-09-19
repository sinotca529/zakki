use crate::command::build::renderer::pass::HighlightRule;
use anyhow::Context as _;
use pulldown_cmark::{Event, MetadataBlockKind::YamlStyle, Tag, TagEnd};
use serde::Deserialize;

/// YAML フロントマターを読み取ります
pub fn read_front_matter(events: &[Event]) -> anyhow::Result<PageFrontMatter> {
    let front_matter = events
        .iter()
        .skip_while(|e| !matches!(e, Event::Start(Tag::MetadataBlock(YamlStyle))))
        .take_while(|e| !matches!(e, Event::End(TagEnd::MetadataBlock(YamlStyle))))
        .find_map(|e| match e {
            Event::Text(t) => Some(t.as_ref()),
            _ => None,
        });

    let Some(front_matter) = front_matter else {
        anyhow::bail!("記事は yaml ヘッダーで始めてください")
    };

    serde_yaml::from_str::<PageFrontMatter>(front_matter)
        .context("yaml ヘッダーのデコードに失敗しました")
}

/// `Option` のフィールドに `#[serde(default)]` は付けません。
/// serde が省略時に `None` を入れるためです。付けるのは `Vec` のように、
/// 省略時の値を serde が決められない型だけにします。
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct PageFrontMatter {
    /// 記事の作成日
    #[serde(rename = "create")]
    pub create_date: String,

    /// 記事の最終更新日
    #[serde(rename = "update")]
    pub last_update_date: String,

    /// 記事のタイトル
    pub title: String,

    /// 記事につけられたタグ
    #[serde(default)]
    pub tags: Vec<String>,

    /// 暗号化時のパスワード
    pub password: Option<String>,

    /// コードハイライトのルール
    pub highlights: Option<Vec<HighlightRule>>,
}
