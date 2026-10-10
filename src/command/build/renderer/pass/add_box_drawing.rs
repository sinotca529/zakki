use super::end_of;
use crate::command::build::box_drawing;
use pulldown_cmark::{Event, Tag};

/// コードブロックに罫線素片があれば、線を描く SVG を差し込みます。
///
/// SVG は `code` の中、コードの後ろに置きます。文字はそのまま残るので、
/// コピーも検索も変わりません。位置は CSS が `pre` を基準に決めます。
///
/// 字形を空にするのは `code_font` です。フォントを作らないサイトでは字形が残り、
/// 線が二重になるので、呼び出し元がこのパスを飛ばします。
pub fn add_box_drawing(events: &mut Vec<Event<'_>>) {
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

        out.extend_from_slice(&events[i..end]);
        if let Some(svg) = box_drawing::svg(&code) {
            out.push(Event::InlineHtml(svg.into()));
        }
        out.push(events[end].clone());
        i = end + 1;
    }

    *events = out;
}

#[cfg(test)]
mod test {
    use super::add_box_drawing;
    use pulldown_cmark::{Options, Parser};

    fn html_of(md: &str) -> String {
        let mut events: Vec<_> = Parser::new_ext(md, Options::empty()).collect();
        add_box_drawing(&mut events);
        let mut out = String::new();
        pulldown_cmark::html::push_html(&mut out, events.into_iter());
        out
    }

    #[test]
    fn puts_the_svg_after_the_code() {
        let html = html_of("```\n┌─┐\n└─┘\n```\n");
        assert!(html.contains("└─┘\n<svg class=\"box-drawing\""), "{html}");
        assert!(html.contains("</svg></code></pre>"), "{html}");
    }

    #[test]
    fn leaves_other_code_blocks_alone() {
        let html = html_of("```\nlet x = 1;\n```\n");
        assert!(!html.contains("svg"), "{html}");
    }
}
