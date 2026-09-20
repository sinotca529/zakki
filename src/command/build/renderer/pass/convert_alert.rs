use crate::command::build::renderer::html_component::{ASIDE, P};
use pulldown_cmark::{BlockQuoteKind, Event, Tag, TagEnd};

/// 注記 (`> [!NOTE]` など) を `<aside>` に変換します。
///
/// pulldown-cmark は注記を `<blockquote>` として描画しますが、
/// HTML の `blockquote` は別の出典からの引用を表すため、注記には合いません。
/// 本文から外れた補足という意味を持つ `aside` を使います。
pub fn convert_alert(events: &mut Vec<Event<'_>>) {
    let mut out = Vec::with_capacity(events.len());

    // 引用の入れ子ごとに、注記なら閉じタグを覚えておく
    let mut close_tags: Vec<Option<String>> = Vec::new();

    for e in events.drain(..) {
        match e {
            Event::Start(Tag::BlockQuote(kind)) => match kind {
                Some(kind) => {
                    let (open, close) = alert_tag(kind);
                    close_tags.push(Some(close));
                    out.push(Event::Html(open.into()));
                }
                None => {
                    close_tags.push(None);
                    out.push(Event::Start(Tag::BlockQuote(None)));
                }
            },
            Event::End(TagEnd::BlockQuote(_)) => match close_tags.pop().flatten() {
                Some(close) => out.push(Event::Html(close.into())),
                None => out.push(e),
            },
            _ => out.push(e),
        }
    }

    *events = out;
}

/// 注記の開きタグと閉じタグを作ります。開きタグには見出しの `<p>` を含みます。
fn alert_tag(kind: BlockQuoteKind) -> (String, String) {
    let (name, title) = match kind {
        BlockQuoteKind::Note => ("note", "Note"),
        BlockQuoteKind::Tip => ("tip", "Tip"),
        BlockQuoteKind::Important => ("important", "Important"),
        BlockQuoteKind::Warning => ("warning", "Warning"),
        BlockQuoteKind::Caution => ("caution", "Caution"),
    };

    let class = format!("markdown-alert markdown-alert-{name}");
    let (open, close) = ASIDE.attr("class", class).pair();
    let title = P.attr("class", "markdown-alert-title").text(title);

    (format!("{open}{title}"), close)
}
