use super::assign_header_id::SECTION_ID_PREFIX;
use super::{end_of, text_of};
use crate::command::build::renderer::html_component::{A, DETAILS, SUMMARY};
use pulldown_cmark::{Event, HeadingLevel, Tag};

/// 記事から目次 HTML を作ります。見出しがない場合は空文字を返します。
///
/// 本文が確定した後に呼びます。見出しの中身はテキストだけを拾うので、
/// 見出しに書いたコードや数式は目次には出ません。
pub fn make_toc(events: &[Event]) -> String {
    let mut items = Vec::new();

    for (i, e) in events.iter().enumerate() {
        let Event::Start(Tag::Heading { level, id, .. }) = e else {
            continue;
        };

        // h1 はページタイトル、h5 以下は目次に載せるには細かすぎる
        let depth = match level {
            HeadingLevel::H2 => 1,
            HeadingLevel::H3 => 2,
            HeadingLevel::H4 => 3,
            _ => continue,
        };

        let id = id
            .as_deref()
            .expect("見出しの id は assign_header_id が振る");

        // id は接頭辞と階層番号でできているので、番号の部分を目次の表示にも使う
        let number = id
            .strip_prefix(SECTION_ID_PREFIX)
            .expect("見出しの id には接頭辞が付く");
        let text = text_of(&events[(i + 1)..end_of(events, i)]);

        items.push((depth, id, number, text));
    }

    if items.is_empty() {
        return String::new();
    }

    let mut html = Vec::<String>::new();
    let mut prev_depth = 0;

    for (depth, id, number, text) in &items {
        // 階層を下る
        (prev_depth..*depth).for_each(|_| html.push("<ol><li>".to_string()));
        // 階層を上る
        (*depth..prev_depth).for_each(|_| html.push("</li></ol>".to_string()));
        // 次の要素へ
        if *depth <= prev_depth {
            html.push("</li><li>".to_string());
        }
        // リンクを追加
        html.push(
            A.attr("href", format!("#{id}"))
                .text(format!("{number}. {text}")),
        );
        prev_depth = *depth;
    }
    // 閉じる
    (0..prev_depth).for_each(|_| html.push("</li></ol>".to_string()));

    let summary = SUMMARY.text("目次");
    DETAILS
        .attr("class", "toc")
        .html(format!("{summary}{}", html.join("")))
}

#[cfg(test)]
mod test {
    use super::super::assign_header_id;
    use super::make_toc;
    use pulldown_cmark::{Options, Parser};

    fn toc_of(md: &str) -> String {
        let mut events: Vec<_> = Parser::new_ext(md, Options::empty()).collect();
        assign_header_id(&mut events);
        make_toc(&events)
    }

    #[test]
    fn nests_by_heading_level() {
        let toc = toc_of("## あ\n\n### い\n\n## う\n");
        assert_eq!(
            toc,
            concat!(
                r#"<details class="toc"><summary>目次</summary>"#,
                r##"<ol><li><a href="#s1">1. あ</a>"##,
                r##"<ol><li><a href="#s1.1">1.1. い</a></li></ol>"##,
                r##"</li><li><a href="#s2">2. う</a></li></ol>"##,
                "</details>",
            )
        );
    }

    #[test]
    fn returns_empty_string_without_headings() {
        assert_eq!(toc_of("本文です。\n"), "");
    }

    /// 見出しに書いた修飾は未定義の扱いです。目次にはテキストだけが出ます。
    #[test]
    fn takes_only_text_from_headings() {
        let toc = toc_of("## `Vec<Event>` を*渡す*\n");
        assert!(
            toc.contains(r##"<a href="#s1">1. Vec&lt;Event> を渡す</a>"##),
            "{toc}"
        );
    }
}
