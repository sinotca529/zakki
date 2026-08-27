use super::escape_html_text;
use crate::command::build::renderer::context::Context;
use anyhow::Result;
use comrak::nodes::{AstNode, NodeHtmlBlock, NodeValue};
use regex::Regex;
use serde::Deserialize;
use std::borrow::Cow;

/// コードブロックの中身に、記事で指定された区切り文字のスタイルを適用します。
///
/// スタイルは `<span>` として埋め込むため、コードブロックごと
/// 生の HTML に置き換えます。
pub fn highlight_code<'a>(root: &'a AstNode<'a>, ctxt: &mut Context) -> Result<()> {
    let Ok(macros) = ctxt.highlights() else {
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
            .map(|lang| format!(r#" class="language-{}""#, escape_html_text(lang)))
            .unwrap_or_default();

        node.data_mut().value = NodeValue::HtmlBlock(NodeHtmlBlock {
            block_type: 0,
            literal: format!("<pre><code{class}>{code}</code></pre>"),
        });
    }

    Ok(())
}

#[derive(Clone, Deserialize, Debug)]
pub struct HighlightRule {
    delim: [String; 2],
    style: String,
}

impl HighlightRule {
    pub fn replace_all<'a>(&self, code: &'a str) -> Cow<'a, str> {
        let Ok(pat) = Regex::new(&format!("{}(.*?){}", self.delim[0], self.delim[1])) else {
            return code.into();
        };

        pat.replace_all(code, format!("<span style=\"{}\">$1</span>", self.style))
    }
}
