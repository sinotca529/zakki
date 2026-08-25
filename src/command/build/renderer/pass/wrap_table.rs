use super::raw_html;
use crate::command::build::renderer::context::Context;
use jotdown::{Container, Event};

pub fn wrap_table<'a>(events: &mut Vec<Event<'a>>, _ctxt: &mut Context) -> anyhow::Result<()> {
    let mut out = Vec::with_capacity(events.len());

    for e in events.drain(..) {
        match e {
            Event::Start(Container::Table, _) => {
                out.extend(raw_html(r#"<div class="table-wrapper">"#));
                out.push(e);
            }
            Event::End(Container::Table) => {
                out.push(e);
                out.extend(raw_html("</div>"));
            }
            _ => out.push(e),
        }
    }

    *events = out;
    Ok(())
}
