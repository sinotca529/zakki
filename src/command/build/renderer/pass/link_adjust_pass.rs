use crate::command::build::renderer::context::Context;
use pulldown_cmark::{Event, Tag};

pub fn link_adjust_pass<'a>(
    mut events: Vec<Event<'a>>,
    _: &mut Context,
) -> anyhow::Result<Vec<Event<'a>>> {
    events.iter_mut().for_each(|mut e| {
        if let Event::Start(Tag::Link { dest_url: url, .. }) = &mut e {
            let is_local = !url.starts_with("http://") && !url.starts_with("https://");
            let is_md = url.ends_with(".md");
            if is_local && is_md {
                *url = format!("{}.html", &url[..url.len() - ".md".len()]).into();
            }
        }
    });
    Ok(events)
}
