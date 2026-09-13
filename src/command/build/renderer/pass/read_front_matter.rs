use crate::command::build::renderer::{FRONT_MATTER_DELIMITER, pass::HighlightRule};
use anyhow::Context as _;
use comrak::nodes::{AstNode, NodeValue};
use serde::Deserialize;

/// YAML フロントマターを読み、メタデータを Context に設定します。
pub fn read_front_matter<'a>(root: &'a AstNode<'a>) -> anyhow::Result<PageFrontMatter> {
    // 区切り ('---') を含むヘッダ文字列
    let front_matter = root
        .descendants()
        .find_map(|node| match &node.data().value {
            NodeValue::FrontMatter(text) => Some(text.clone()),
            _ => None,
        });

    let Some(front_matter) = front_matter else {
        anyhow::bail!("記事は yaml ヘッダーで始めてください")
    };

    let front_matter_body = front_matter
        .trim_end()
        .strip_prefix(FRONT_MATTER_DELIMITER)
        .and_then(|s| s.strip_suffix(FRONT_MATTER_DELIMITER))
        .context("yaml ヘッダーは --- で開始・終了する必要があります")?;

    serde_yaml::from_str::<PageFrontMatter>(front_matter_body)
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
