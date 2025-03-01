use crate::command::build::renderer::context::Context;
use pulldown_cmark::{Event, Tag};

pub fn table_wrapper_pass<'a>(
    events: Vec<Event<'a>>,
    _ctxt: &mut Context,
) -> anyhow::Result<Vec<Event<'a>>> {
    let mut out_events = Vec::with_capacity(events.len());

    for e in events.into_iter() {
        match e {
            Event::Start(Tag::Table(_)) => {
                out_events.push(Event::InlineHtml(r#"<div class="table-wrapper">"#.into()));
                out_events.push(e);
            }
            Event::End(pulldown_cmark::TagEnd::Table) => {
                out_events.push(e);
                out_events.push(Event::InlineHtml("</div>".into()));
            }
            _ => out_events.push(e),
        }
    }

    Ok(out_events)
}
