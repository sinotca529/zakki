use pulldown_cmark::{Event, Tag, TagEnd};

/// 表を横スクロールできるよう `<div class="x-scroll">` で囲みます。
pub fn wrap_table(events: &mut Vec<Event<'_>>) {
    let mut out = Vec::with_capacity(events.len());

    for e in events.drain(..) {
        match e {
            Event::Start(Tag::Table(_)) => {
                out.push(Event::Html(r#"<div class="x-scroll" tabindex="0">"#.into()));
                out.push(e);
            }
            Event::End(TagEnd::Table) => {
                out.push(e);
                out.push(Event::Html("</div>".into()));
            }
            _ => out.push(e),
        }
    }

    *events = out;
}
