use super::heading_id::HeaderIdGenerator;
use super::html_component::escape_html_text;
use comrak::nodes::{AstNode, NodeValue};

/// 目次に載せる最も深い見出しのレベル。
const DEEPEST_LEVEL: u8 = 4;

/// AST から目次の HTML を作ります。見出しがない場合は空文字を返します。
///
/// 見出しの番号は描画時と同じ `HeaderIdGenerator` で採番します。
/// 数式を含む見出しがあるため、`convert_math` より前に呼ぶ必要があります。
pub fn toc_html<'a>(root: &'a AstNode<'a>) -> String {
    let mut numbering = HeaderIdGenerator::default();

    // (階層の深さ, id, 見出しの文字列)
    let mut items: Vec<(u8, String, String)> = Vec::new();

    for node in root.descendants() {
        let level = match &node.data().value {
            NodeValue::Heading(h) => h.level,
            _ => continue,
        };

        // 目次に載せない見出しでも、描画時と採番をそろえるためカウンタは進めます。
        let id = numbering.next_id(level);
        if level > DEEPEST_LEVEL {
            continue;
        }

        // h2 を第 1 階層とします。h1 はページタイトル用で、本文では使えません。
        items.push((level - 1, id, heading_text(node)));
    }

    if items.is_empty() {
        return String::new();
    }

    let mut html = Vec::<String>::new();
    let mut prev_level = 0;

    for (level, id, text) in &items {
        // 階層を下る
        (prev_level..*level).for_each(|_| html.push("<ol><li>".to_string()));
        // 階層を上る
        (*level..prev_level).for_each(|_| html.push("</li></ol>".to_string()));
        // 次の要素へ
        if *level <= prev_level {
            html.push("</li><li>".to_string());
        }
        // リンクを追加
        html.push(format!(
            "<a href=\"#{}\">{}</a>",
            id,
            escape_html_text(&format!("{id}. {text}"))
        ));
        prev_level = *level;
    }
    // 閉じる
    (0..prev_level).for_each(|_| html.push("</li></ol>".to_string()));

    format!(
        "<details id=\"toc\"><summary>目次</summary>{}</details>",
        html.join("")
    )
}

/// 見出しの文字列を組み立てます。
///
/// 目次はナビゲーションなので、`<code>` や `<strong>` の装飾は落として文字だけにします。
/// 数式は描画前なので、LaTeX の元の文字列を使います。
fn heading_text<'a>(heading: &'a AstNode<'a>) -> String {
    heading
        .descendants()
        .filter_map(|n| match &n.data().value {
            NodeValue::Text(t) => Some(t.to_string()),
            NodeValue::Code(c) => Some(c.literal.clone()),
            NodeValue::Math(m) => Some(m.literal.clone()),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::toc_html;
    use comrak::{Arena, Options, parse_document};

    fn toc_of(md: &str) -> String {
        let arena = Arena::new();
        let root = parse_document(&arena, md, &Options::default());
        toc_html(root)
    }

    #[test]
    fn nests_by_level_and_numbers_headings() {
        let toc = toc_of("## あ\n\n### い\n\n## う\n");
        assert_eq!(
            toc,
            concat!(
                r##"<details id="toc"><summary>目次</summary>"##,
                r##"<ol><li><a href="#1">1. あ</a>"##,
                r##"<ol><li><a href="#1.1">1.1. い</a></li></ol>"##,
                r##"</li><li><a href="#2">2. う</a></li></ol></details>"##,
            )
        );
    }

    /// h5 以下は目次に載せませんが、採番は描画時と同じ順に進める必要があります。
    #[test]
    fn deeper_headings_are_skipped_but_still_counted() {
        let toc = toc_of("## あ\n\n### い\n\n#### う\n\n##### え\n\n## お\n");
        assert!(!toc.contains("え"));
        assert!(toc.contains(r##"<a href="#2">2. お</a>"##));
    }

    #[test]
    fn no_heading_makes_no_toc() {
        assert_eq!(toc_of("本文だけです。\n"), "");
    }
}
