use anyhow::{Result, bail};
use comrak::nodes::{AstNode, NodeHtmlBlock, NodeValue};
use regex::Regex;
use serde::Deserialize;

use crate::command::build::renderer::html_component::{escape_html_attr, escape_html_text};

/// コードブロックの中身に、記事で指定された区切り文字のスタイルを適用します。
///
/// スタイルは `<span>` として埋め込むため、コードブロックごと
/// 生の HTML に置き換えます。
pub fn highlight_code<'a>(
    root: &'a AstNode<'a>,
    highlights: &Option<Vec<HighlightRule>>,
) -> Result<()> {
    let Some(rules) = highlights.as_ref() else {
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
        let code = highlighted_html(&literal, rules);

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

/// コードの一部分。区切り文字で囲まれていたかどうかで分かれます。
enum Piece<'a> {
    Plain(&'a str),
    Styled { text: &'a str, style: &'a str },
}

/// コードを区切り文字で分割します。
///
/// 左から順に、いちばん手前で当たる規則を選びます。同じ位置で複数の規則が当たる場合は、
/// yaml ヘッダに先に書いたものを使います。区切り文字の入れ子は扱いません。
fn split<'a>(code: &'a str, rules: &'a [HighlightRule]) -> Vec<Piece<'a>> {
    let mut pieces = Vec::new();
    let mut rest = code;

    while !rest.is_empty() {
        let hit = rules
            .iter()
            .filter_map(|rule| rule.pattern.captures(rest).map(|caps| (rule, caps)))
            .min_by_key(|(_, caps)| caps.get(0).map_or(usize::MAX, |m| m.start()));

        let Some((rule, caps)) = hit else {
            pieces.push(Piece::Plain(rest));
            break;
        };

        let whole = caps.get(0).expect("正規表現全体の一致は必ず取れる");
        let inner = caps
            .get(1)
            .expect("規則は中身をキャプチャする括弧を必ず持つ");

        if whole.start() > 0 {
            pieces.push(Piece::Plain(&rest[..whole.start()]));
        }
        pieces.push(Piece::Styled {
            text: inner.as_str(),
            style: &rule.style,
        });

        rest = &rest[whole.end()..];
    }

    pieces
}

/// 分割した結果を HTML に組み立てます。
///
/// エスケープはここでだけ行います。先にコードをエスケープすると、
/// 埋め込んだ `<span>` まで後続の規則の対象になるためです。
fn highlighted_html(code: &str, rules: &[HighlightRule]) -> String {
    let mut out = String::with_capacity(code.len());

    for piece in split(code, rules) {
        match piece {
            Piece::Plain(text) => out.push_str(&escape_html_text(text)),
            Piece::Styled { text, style } => out.push_str(&format!(
                r#"<span style="{}">{}</span>"#,
                escape_html_attr(style),
                escape_html_text(text)
            )),
        }
    }

    out
}

/// yaml ヘッダに書かれる形。
/// 実行時の形は `HighlightRule`。
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
    type Error = anyhow::Error;

    fn try_from(value: HighlightRuleConfig) -> Result<Self> {
        if value.delim.iter().any(|d| d.is_empty()) {
            bail!("highlights の delim に空の文字列は書けません");
        }

        let open = regex::escape(&value.delim[0]);
        let close = regex::escape(&value.delim[1]);
        let pattern = Regex::new(&format!("{open}(.*?){close}"))?;

        Ok(Self {
            pattern,
            style: value.style,
        })
    }
}

#[cfg(test)]
mod test {
    use super::{HighlightRule, HighlightRuleConfig, highlighted_html};

    fn config(open: &str, close: &str, style: &str) -> HighlightRuleConfig {
        HighlightRuleConfig {
            delim: [open.to_owned(), close.to_owned()],
            style: style.to_owned(),
        }
    }

    fn rule(open: &str, close: &str, style: &str) -> HighlightRule {
        HighlightRule::try_from(config(open, close, style)).unwrap()
    }

    #[test]
    fn rejects_empty_delimiters() {
        assert!(HighlightRule::try_from(config("", "", "color: red")).is_err());
        assert!(HighlightRule::try_from(config("[[", "", "color: red")).is_err());
        assert!(HighlightRule::try_from(config("", "]]", "color: red")).is_err());
        assert!(HighlightRule::try_from(config("[[", "]]", "color: red")).is_ok());
    }

    #[test]
    fn wraps_each_delimited_part() {
        let rules = [
            rule("[[", "]]", "color: red"),
            rule("<<", ">>", "color: blue"),
        ];
        assert_eq!(
            highlighted_html("let [[a]] = <<b>>;", &rules),
            concat!(
                r#"let <span style="color: red">a</span> = "#,
                r#"<span style="color: blue">b</span>;"#,
            )
        );
    }

    /// 先にコードをエスケープすると、埋め込んだ `<span>` まで後続の規則の対象になります。
    /// 組み立てのときだけエスケープするので、コード中の `<` は 1 度しか置き換わりません。
    #[test]
    fn escapes_code_once() {
        let rules = [rule("[[", "]]", "color: red")];
        assert_eq!(
            highlighted_html("a < b && [[c < d]]", &rules),
            r#"a &lt; b &amp;&amp; <span style="color: red">c &lt; d</span>"#
        );
    }

    /// 同じ位置で複数の規則が当たる場合は、yaml ヘッダに先に書いたものを使います。
    #[test]
    fn earlier_rule_wins_at_the_same_position() {
        let rules = [rule("[[", "]]", "first"), rule("[[", "]]", "second")];
        assert_eq!(
            highlighted_html("[[x]]", &rules),
            r#"<span style="first">x</span>"#
        );
    }

    #[test]
    fn keeps_code_as_is_when_no_rule_matches() {
        let rules = [rule("[[", "]]", "color: red")];
        assert_eq!(highlighted_html("let a = 0;", &rules), "let a = 0;");
    }
}
