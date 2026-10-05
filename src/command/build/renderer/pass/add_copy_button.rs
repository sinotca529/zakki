use crate::command::build::renderer::html_component::{DIV, escape_html_attr};
use crate::include_asset;
use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};

/// コードブロックを `<div class="code-block">` で囲み、コピーボタンを置きます。
///
/// ボタンの位置の基準は、スクロールしない要素である必要があります。
/// `pre` には `overflow-x: auto` を当てているので、囲んだ `div` を基準にします。
///
/// 押したときの動作は `script.js` に書いています。
///
/// `pre` と `code` の開きタグもここで書きます。pulldown-cmark が書くタグには
/// `tabindex` を足せないためです。横にスクロールする領域は、キーボードでも
/// 送れる必要があります。表に対しては `wrap_table` が同じことをしています。
pub fn add_copy_button(events: &mut Vec<Event<'_>>) {
    let mut out = Vec::with_capacity(events.len());
    let (open, close) = DIV.attr("class", "code-block").pair();
    let button = include_asset!("copy-button.html");

    for e in events.drain(..) {
        match e {
            Event::Start(Tag::CodeBlock(kind)) => {
                out.push(Event::Html(open.clone().into()));
                out.push(Event::Html(code_open(&kind).into()));
            }
            Event::End(TagEnd::CodeBlock) => {
                out.push(Event::Html("</code></pre>".into()));
                out.push(Event::Html(button.into()));
                out.push(Event::Html(close.clone().into()));
            }
            _ => out.push(e),
        }
    }

    *events = out;
}

/// `pre` と `code` の開きタグを書きます。
///
/// 言語名の付け方は pulldown-cmark に合わせています。info string の
/// 最初の空白までを言語名とし、`language-` を前に付けた class にします。
fn code_open(kind: &CodeBlockKind) -> String {
    let lang = match kind {
        CodeBlockKind::Fenced(info) => info.split(' ').next().unwrap_or(""),
        CodeBlockKind::Indented => "",
    };

    if lang.is_empty() {
        r#"<pre tabindex="0"><code>"#.to_owned()
    } else {
        format!(
            r#"<pre tabindex="0"><code class="language-{}">"#,
            escape_html_attr(lang)
        )
    }
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
            html.starts_with("<div class=\"code-block\"><pre tabindex=\"0\"><code>"),
            "{html}"
        );
        assert!(
            html.contains("</code></pre><button type=\"button\" class=\"copy-button\""),
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
