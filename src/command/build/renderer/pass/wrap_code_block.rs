use crate::command::build::renderer::html_component::DIV;
use pulldown_cmark::{Event, Tag, TagEnd};

/// コードブロックを `<div class="code-block">` で囲みます。
///
/// コピーボタンを重ねる相手です。`pre` は `overflow-x: auto` なので、
/// ボタンを `pre` の中に置くと、横にスクロールしたときに一緒に流れます。
pub fn wrap_code_block(events: &mut Vec<Event<'_>>) {
    let mut out = Vec::with_capacity(events.len());
    let (open, close) = DIV.attr("class", "code-block").pair();

    for e in events.drain(..) {
        match e {
            Event::Start(Tag::CodeBlock(_)) => {
                out.push(Event::Html(open.clone().into()));
                out.push(e);
            }
            Event::End(TagEnd::CodeBlock) => {
                out.push(e);
                out.push(Event::Html(close.clone().into()));
            }
            _ => out.push(e),
        }
    }

    *events = out;
}

#[cfg(test)]
mod test {
    use super::wrap_code_block;
    use pulldown_cmark::{Options, Parser};

    fn html_of(md: &str) -> String {
        let mut events: Vec<_> = Parser::new_ext(md, Options::empty()).collect();
        wrap_code_block(&mut events);
        let mut out = String::new();
        pulldown_cmark::html::push_html(&mut out, events.into_iter());
        out
    }

    #[test]
    fn wraps_a_code_block() {
        assert_eq!(
            html_of("```\nlet x = 1;\n```\n"),
            concat!(
                "<div class=\"code-block\">\n",
                "<pre><code>let x = 1;\n",
                "</code></pre>\n",
                "</div>",
            )
        );
    }

    /// インラインコードは `pre` にならないので、囲みません。
    #[test]
    fn leaves_inline_code_alone() {
        let html = html_of("`x` です。\n");
        assert!(!html.contains("code-block"), "{html}");
    }
}
