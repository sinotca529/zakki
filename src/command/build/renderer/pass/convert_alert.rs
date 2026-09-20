use crate::command::build::renderer::html_component::P;
use pulldown_cmark::{BlockQuoteKind, Event, Tag, TagEnd};

/// 注記 (`> [!NOTE]` など) を `<aside>` に変換します。
///
/// pulldown-cmark は注記を `<blockquote>` として描画しますが、
/// HTML の `blockquote` は別の出典からの引用を表すため、注記には合いません。
/// 本文から外れた補足という意味を持つ `aside` を使います。
pub fn convert_alert(events: &mut Vec<Event<'_>>) {
    let mut out = Vec::with_capacity(events.len());

    // 引用の入れ子ごとに、注記かどうかを覚えておく
    let mut is_alert = Vec::new();

    for e in events.drain(..) {
        match e {
            Event::Start(Tag::BlockQuote(kind)) => {
                is_alert.push(kind.is_some());
                match kind {
                    Some(kind) => out.push(Event::Html(open_tag(kind).into())),
                    None => out.push(Event::Start(Tag::BlockQuote(None))),
                }
            }
            Event::End(TagEnd::BlockQuote(_)) => match is_alert.pop() {
                Some(true) => out.push(Event::Html("</aside>".into())),
                _ => out.push(e),
            },
            _ => out.push(e),
        }
    }

    *events = out;
}

fn open_tag(kind: BlockQuoteKind) -> String {
    let (name, title) = match kind {
        BlockQuoteKind::Note => ("note", "Note"),
        BlockQuoteKind::Tip => ("tip", "Tip"),
        BlockQuoteKind::Important => ("important", "Important"),
        BlockQuoteKind::Warning => ("warning", "Warning"),
        BlockQuoteKind::Caution => ("caution", "Caution"),
    };

    let title = P.attr("class", "markdown-alert-title").text(title);
    format!(r#"<aside class="markdown-alert markdown-alert-{name}">{title}"#)
}
