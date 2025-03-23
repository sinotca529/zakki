use crate::command::build::renderer::context::Context;
use pulldown_cmark::{Event, Tag};
use toc::TocBuilder;

pub mod toc;

pub fn toc_pass<'a>(events: Vec<Event<'a>>, ctxt: &mut Context) -> anyhow::Result<Vec<Event<'a>>> {
    let mut level = None;
    let mut id = None;

    let mut toc_builder = TocBuilder::new();
    for e in &events {
        match e {
            Event::Start(Tag::Heading {
                level: l, id: i, ..
            }) if *l as usize >= 2 => {
                // h1 はタイトルなので無視する。その他は level から 1 引く。
                level = Some(*l as usize - 1);
                id = Some(i.as_ref().unwrap());
            }
            Event::Text(title) if level.is_some() => {
                let (level, id) = (level.take().unwrap(), id.take().unwrap());
                toc_builder.add_item(title.to_string(), id.to_string(), level);
            }
            _ => {}
        }
    }

    let toc = toc_builder.build();
    ctxt.set_toc(toc);

    Ok(events)
}
