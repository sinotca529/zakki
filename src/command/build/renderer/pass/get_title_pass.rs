use crate::command::build::renderer::context::Context;
use anyhow::bail;
use pulldown_cmark::{Event, HeadingLevel, Tag, TagEnd};

pub fn get_title_pass<'a>(
    events: Vec<Event<'a>>,
    ctxt: &mut Context,
) -> anyhow::Result<Vec<Event<'a>>> {
    let mut h1_events = events
        .iter()
        .skip_while(|e| !matches!(e, Event::Start(Tag::Heading { level, .. }) if level == &HeadingLevel::H1))
        .skip(1)
        .take_while(|e| !matches!(e, Event::End(TagEnd::Heading(HeadingLevel::H1))));

    let Some(Event::Text(t)) = h1_events.next() else {
        bail!("Failed to find title.");
    };

    if h1_events.next().is_some() {
        bail!("Title must be made with just one vanilla text.");
    }

    ctxt.set_title(t.to_string());

    Ok(events)
}
