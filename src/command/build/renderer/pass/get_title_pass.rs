use crate::command::build::renderer::context::Context;
use anyhow::bail;
use pulldown_cmark::{Event, HeadingLevel, Tag, TagEnd};

pub fn get_title_pass<'a>(
    events: Vec<Event<'a>>,
    ctxt: &mut Context,
) -> anyhow::Result<Vec<Event<'a>>> {
    let h1 = events
        .iter()
        .skip_while(|e| !matches!(e, Event::Start(Tag::Heading { level, .. }) if level == &HeadingLevel::H1))
        .take_while(|e| !matches!(e, Event::End(TagEnd::Heading(HeadingLevel::H1))))
        .filter_map(|e| match e {
            Event::Text(t) => Some(t.to_string()),
            _ => None,
        })
        .next();

    let Some(h1) = h1 else {
        bail!("h1 is not existing.")
    };

    ctxt.set_title(h1);

    Ok(events)
}
