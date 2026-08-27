use super::HighlightRule;
use crate::command::build::renderer::context::Context;
use comrak::nodes::{AstNode, NodeValue};
use serde::Deserialize;

/// YAML フロントマターを読み、メタデータを Context に設定します。
pub fn read_header<'a>(root: &'a AstNode<'a>, meta: &mut Context) -> anyhow::Result<()> {
    let front_matter = root
        .descendants()
        .find_map(|node| match &node.data().value {
            NodeValue::FrontMatter(text) => Some(text.clone()),
            _ => None,
        });

    let Some(front_matter) = front_matter else {
        anyhow::bail!("Yaml header is not existing.")
    };

    let header: YamlHeader = serde_yaml::from_str(&strip_delimiters(&front_matter))?;
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

/// フロントマターのノードは区切りの `---` を含むため、それを取り除きます。
/// 改行は LF と CRLF のどちらでも構いません。
fn strip_delimiters(front_matter: &str) -> String {
    front_matter
        .lines()
        .filter(|line| line.trim_end_matches(['\r', '\n']) != "---")
        .collect::<Vec<_>>()
        .join("\n")
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
