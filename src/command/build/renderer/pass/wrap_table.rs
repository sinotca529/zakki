use crate::command::build::renderer::html_component::DIV;
use pulldown_cmark::{Event, Tag, TagEnd};

/// 表を横スクロールできるよう `<div class="x-scroll">` で囲みます。
pub fn wrap_table(events: &mut Vec<Event<'_>>) {
    let mut out = Vec::with_capacity(events.len());
    let (open, close) = DIV.attr("class", "x-scroll").attr("tabindex", "0").pair();

    for e in events.drain(..) {
        match e {
            Event::Start(Tag::Table(_)) => {
                out.push(Event::Html(open.clone().into()));
                out.push(e);
            }
            Event::End(TagEnd::Table) => {
                out.push(e);
                out.push(Event::Html(close.clone().into()));
            }
            _ => out.push(e),
        }
    }

    *events = out;
}
