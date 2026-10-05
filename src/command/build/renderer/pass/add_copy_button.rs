use crate::command::build::renderer::html_component::DIV;
use crate::include_asset;
use pulldown_cmark::{Event, Tag, TagEnd};

/// コードブロックを `<div class="code-block">` で囲み、コピーボタンを置きます。
///
/// `pre` は `overflow-x: auto` なので、ボタンを `pre` の中に置くと、
/// 横にスクロールしたときに一緒に流れます。囲んだ `div` に重ねます。
///
/// ボタンは JS で後から挿さずに、ここで出力します。挿す作りでは、
/// 本文を描いた後にボタンが現れます。
/// 押したときの動作は `script.js` が受け持ちます。
pub fn add_copy_button(events: &mut Vec<Event<'_>>) {
    let mut out = Vec::with_capacity(events.len());
    let (open, close) = DIV.attr("class", "code-block").pair();
    let button = include_asset!("copy-button.html");

    for e in events.drain(..) {
        match e {
            Event::Start(Tag::CodeBlock(_)) => {
                out.push(Event::Html(open.clone().into()));
                out.push(e);
            }
            Event::End(TagEnd::CodeBlock) => {
                out.push(e);
                out.push(Event::Html(button.into()));
                out.push(Event::Html(close.clone().into()));
            }
            _ => out.push(e),
        }
    }

    *events = out;
}

#[cfg(test)]
mod test {
    use super::add_copy_button;
    use pulldown_cmark::{Options, Parser};

    fn html_of(md: &str) -> String {
        let mut events: Vec<_> = Parser::new_ext(md, Options::empty()).collect();
        add_copy_button(&mut events);
        let mut out = String::new();
        pulldown_cmark::html::push_html(&mut out, events.into_iter());
        out
    }

    #[test]
    fn wraps_a_code_block_and_adds_a_button() {
        let html = html_of("```\nlet x = 1;\n```\n");
        assert!(
            html.starts_with("<div class=\"code-block\">\n<pre>"),
            "{html}"
        );
        assert!(
            html.contains("</pre>\n<button type=\"button\" class=\"copy-button\""),
            "{html}"
        );
        assert!(html.trim_end().ends_with("</button></div>"), "{html}");
    }

    /// インラインコードは `pre` にならないので、囲みません。
    #[test]
    fn leaves_inline_code_alone() {
        let html = html_of("`x` です。\n");
        assert!(!html.contains("code-block"), "{html}");
        assert!(!html.contains("copy-button"), "{html}");
    }
}
