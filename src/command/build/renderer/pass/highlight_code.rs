use super::end_of;
use pulldown_cmark::{Event, Tag};
use serde::Deserialize;

use crate::command::build::renderer::html_component::SPAN;
use anyhow::{Result, bail};

/// コードブロックの中身に、記事で指定された区切り文字のスタイルを適用します。
///
/// スタイルは `<span>` を生の HTML として差し込み、コードの中身は `Text` のまま残します。
/// こうすると、後続のパスがコードの文字を読めます。区切り文字はここで取り除かれるため、
/// 検索の索引にも入りません。
pub fn highlight_code(events: &mut Vec<Event<'_>>, highlights: &Option<Vec<HighlightRule>>) {
    let Some(rules) = highlights.as_ref() else {
        return;
    };

    let mut out = Vec::with_capacity(events.len());
    let mut i = 0;

    while i < events.len() {
        let Event::Start(Tag::CodeBlock(_)) = &events[i] else {
            out.push(events[i].clone());
            i += 1;
            continue;
        };

        let end = end_of(events, i);

        let code: String = events[(i + 1)..end]
            .iter()
            .filter_map(|e| match e {
                Event::Text(t) => Some(t.as_ref()),
                _ => None,
            })
            .collect();

        out.push(events[i].clone());
        out.extend(highlighted_events(&code, rules));
        out.push(events[end].clone());
        i = end + 1;
    }

    *events = out;
}

/// 分割した結果をイベント列にします。
fn highlighted_events(code: &str, rules: &[HighlightRule]) -> Vec<Event<'static>> {
    let mut out = Vec::new();

    for piece in split(code, rules) {
        match piece {
            Piece::Plain(text) => out.push(Event::Text(text.to_owned().into())),
            Piece::Styled { text, style } => {
                let (open, close) = SPAN.attr("style", style).pair();
                out.push(Event::InlineHtml(open.into()));
                out.push(Event::Text(text.to_owned().into()));
                out.push(Event::InlineHtml(close.into()));
            }
        }
    }

    out
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
            .filter_map(|rule| rule.find(rest).map(|found| (rule, found)))
            .min_by_key(|(_, found)| found.start);

        let Some((rule, found)) = hit else {
            pieces.push(Piece::Plain(rest));
            break;
        };

        if found.start > 0 {
            pieces.push(Piece::Plain(&rest[..found.start]));
        }
        pieces.push(Piece::Styled {
            text: found.inner,
            style: &rule.style,
        });

        rest = &rest[found.end..];
    }

    pieces
}

/// 区切り文字に挟まれた部分の位置です。
struct Found<'a> {
    /// 開きの区切り文字が始まる位置
    start: usize,
    /// 閉じの区切り文字が終わる位置
    end: usize,
    /// 区切り文字の間の文字列
    inner: &'a str,
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
    open: String,
    close: String,
    style: String,
}

impl HighlightRule {
    /// 区切り文字に挟まれた部分のうち、いちばん手前のものを探します。
    /// 開きと閉じが別の行にある組は選びません。
    fn find<'a>(&self, code: &'a str) -> Option<Found<'a>> {
        let mut from = 0;

        while let Some(open) = code[from..].find(&self.open) {
            let start = from + open;
            let inner_start = start + self.open.len();

            let line_end = code[inner_start..]
                .find('\n')
                .map_or(code.len(), |i| inner_start + i);

            if let Some(close) = code[inner_start..line_end].find(&self.close) {
                let inner_end = inner_start + close;
                return Some(Found {
                    start,
                    end: inner_end + self.close.len(),
                    inner: &code[inner_start..inner_end],
                });
            }

            from = inner_start;
        }

        None
    }
}

impl TryFrom<HighlightRuleConfig> for HighlightRule {
    type Error = anyhow::Error;

    fn try_from(value: HighlightRuleConfig) -> Result<Self> {
        if value.delim.iter().any(|d| d.is_empty()) {
            bail!("highlights の delim に空の文字列は書けません");
        }

        let [open, close] = value.delim;

        Ok(Self {
            open,
            close,
            style: value.style,
        })
    }
}

#[cfg(test)]
mod test {
    use super::{HighlightRule, HighlightRuleConfig, highlighted_events};
    use pulldown_cmark::Event;

    fn config(open: &str, close: &str, style: &str) -> HighlightRuleConfig {
        HighlightRuleConfig {
            delim: [open.to_owned(), close.to_owned()],
            style: style.to_owned(),
        }
    }

    fn rule(open: &str, close: &str, style: &str) -> HighlightRule {
        HighlightRule::try_from(config(open, close, style)).unwrap()
    }

    /// 比べやすいよう、イベントの種類と中身を 1 行にまとめる
    fn dump(code: &str, rules: &[HighlightRule]) -> Vec<String> {
        highlighted_events(code, rules)
            .iter()
            .map(|e| match e {
                Event::Text(t) => format!("text: {t}"),
                Event::InlineHtml(h) => format!("html: {h}"),
                _ => unreachable!("この関数が作るのは Text と InlineHtml だけ"),
            })
            .collect()
    }

    #[test]
    fn wraps_each_delimited_part() {
        let rules = [
            rule("[[", "]]", "color: red"),
            rule("<<", ">>", "background: yellow"),
        ];
        assert_eq!(
            dump("let [[a]] = <<b>>;", &rules),
            [
                "text: let ",
                r#"html: <span style="color: red">"#,
                "text: a",
                "html: </span>",
                "text:  = ",
                r#"html: <span style="background: yellow">"#,
                "text: b",
                "html: </span>",
                "text: ;",
            ]
        );
    }

    /// コードの文字はエスケープせずに `Text` のまま渡します。
    /// 描画側が escape_html_body_text を通すためです。
    #[test]
    fn leaves_code_unescaped() {
        let rules = [rule("[[", "]]", "color: red")];
        assert_eq!(
            dump("a < b && [[c > d]]", &rules),
            [
                "text: a < b && ",
                r#"html: <span style="color: red">"#,
                "text: c > d",
                "html: </span>",
            ]
        );
    }

    /// 同じ位置で複数の規則が当たる場合は、yaml ヘッダに先に書いたものを使います。
    #[test]
    fn earlier_rule_wins_at_the_same_position() {
        let rules = [rule("[[", "]]", "first"), rule("[[", "]]", "second")];
        assert_eq!(
            dump("[[x]]", &rules),
            [r#"html: <span style="first">"#, "text: x", "html: </span>"]
        );
    }

    #[test]
    fn keeps_code_as_is_when_no_rule_matches() {
        let rules = [rule("[[", "]]", "color: red")];
        assert_eq!(dump("let a = 0;", &rules), ["text: let a = 0;"]);
    }

    #[test]
    fn rejects_empty_delimiters() {
        assert!(HighlightRule::try_from(config("", "", "color: red")).is_err());
        assert!(HighlightRule::try_from(config("[[", "", "color: red")).is_err());
        assert!(HighlightRule::try_from(config("", "]]", "color: red")).is_err());
        assert!(HighlightRule::try_from(config("[[", "]]", "color: red")).is_ok());
    }
}
