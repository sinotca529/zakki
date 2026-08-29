use super::HighlightRule;
use crate::command::build::renderer::context::Context;
use anyhow::Context as _;
use comrak::nodes::{AstNode, NodeValue};
use serde::Deserialize;

/// YAML フロントマターを読み、メタデータを Context に設定します。
pub fn read_header<'a>(root: &'a AstNode<'a>, ctx: &mut Context) -> anyhow::Result<()> {
    // 区切り ('---') を含むヘッダ文字列
    let front_matter = root
        .descendants()
        .find_map(|node| match &node.data().value {
            NodeValue::FrontMatter(text) => Some(text.clone()),
            _ => None,
        });

    let Some(front_matter) = front_matter else {
        anyhow::bail!("yaml ヘッダーがありません。記事の先頭を '---' で初めてください。")
    };

    let front_matter_body = front_matter
        .trim_end()
        .strip_prefix("---")
        .and_then(|s| s.strip_suffix("---"))
        .with_context(|| "yaml ヘッダーは --- で開始・終了する必要があります")?;

    let header: YamlHeader = serde_yaml::from_str(front_matter_body)?;

    ctx.set_create_date(header.create_date);
    ctx.set_last_update_date(header.last_update_date);
    ctx.set_title(header.title);
    ctx.set_tags(header.tags);
    if let Some(h) = header.highlights {
        ctx.set_highlights(h);
    }
    if let Some(pwd) = header.password {
        ctx.set_password(pwd);
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

    /// 記事のタイトル
    pub title: String,

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
