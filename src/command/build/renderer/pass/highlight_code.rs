use std::borrow::Cow;

use super::escape_html_text;
use crate::command::build::renderer::{context::Context, pass::escape_html_attr};
use anyhow::Result;
use comrak::nodes::{AstNode, NodeHtmlBlock, NodeValue};
use regex::Regex;
use serde::Deserialize;

/// コードブロックの中身に、記事で指定された区切り文字のスタイルを適用します。
///
/// スタイルは `<span>` として埋め込むため、コードブロックごと
/// 生の HTML に置き換えます。
pub fn highlight_code<'a>(root: &'a AstNode<'a>, ctx: &mut Context) -> Result<()> {
    let Ok(macros) = ctx.highlights() else {
        return Ok(());
    };

    let targets: Vec<_> = root
        .descendants()
        .filter_map(|node| match &node.data().value {
            NodeValue::CodeBlock(code) => Some((node, code.info.clone(), code.literal.clone())),
            _ => None,
        })
        .collect();

    for (node, info, literal) in targets {
        let mut code = escape_html_text(&literal);
        for m in macros {
            code = m.replace_all(&code).to_string();
        }

        let class = info
            .split_whitespace()
            .next()
            .filter(|lang| !lang.is_empty())
            .map(|lang| format!(r#" class="language-{}""#, escape_html_attr(lang)))
            .unwrap_or_default();

        node.data_mut().value = NodeValue::HtmlBlock(NodeHtmlBlock {
            block_type: 0,
            literal: format!("<pre><code{class}>{code}</code></pre>"),
        });
    }

    Ok(())
}

#[derive(Clone, Deserialize, Debug)]
struct HighlightRuleConfig {
    delim: [String; 2],
    style: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "HighlightRuleConfig")]
pub struct HighlightRule {
    pattern: Regex,
    style: String,
}

impl TryFrom<HighlightRuleConfig> for HighlightRule {
    type Error = regex::Error;

    fn try_from(value: HighlightRuleConfig) -> Result<Self, Self::Error> {
        // コード側は escape_html_text を通してから置換される。
        // そのため、パターンの区切り文字も同様に置換をしておく。
        let open = regex::escape(&escape_html_text(&value.delim[0]));
        let close = regex::escape(&escape_html_text(&value.delim[1]));
        let pattern = Regex::new(&format!("{open}(.*?){close}"))?;

        Ok(Self {
            pattern,
            style: value.style,
        })
    }
}

impl HighlightRule {
    pub fn replace_all<'a>(&self, code: &'a str) -> Cow<'a, str> {
        self.pattern
            .replace_all(code, format!("<span style=\"{}\">$1</span>", self.style))
    }
}
