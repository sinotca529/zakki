use super::HighlightRule;
use crate::command::build::renderer::context::Context;
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

    let header: YamlHeader = serde_yaml::from_str(strip_delimiters(&front_matter))?;
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

/// 先頭行と末尾行を除いた行を返します。
/// Yaml ヘッダの区切り行 (`---`) の除去に利用します。
fn strip_delimiters(front_matter: &str) -> &str {
    let start = front_matter.find('\n').unwrap() + 1;
    let end = front_matter
        .rfind('\r')
        .or(front_matter.rfind('\n'))
        .unwrap();

    &front_matter[start..end]
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

#[cfg(test)]
mod test {
    use super::strip_delimiters;

    #[test]
    fn strips_lf_delimiters() {
        assert_eq!(strip_delimiters("---\ntitle: a\n---"), "title: a");
    }

    #[test]
    fn strips_crlf_delimiters() {
        // str::lines() が \r を落とすため、LF と同じ結果になる
        assert_eq!(strip_delimiters("---\r\ntitle: a\r\n---"), "title: a");
    }

    #[test]
    fn keeps_multiple_lines() {
        let yaml = strip_delimiters("---\ntitle: a\ntag: [x]\n---");
        assert_eq!(yaml, "title: a\ntag: [x]");
    }
}
