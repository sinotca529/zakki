use crate::command::build::renderer::context::Context;
use pulldown_cmark::{Event, Tag};

fn gen_id(cntr: &[i32]) -> String {
    cntr.iter()
        .take_while(|&&e| e > 0)
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

pub fn assign_header_id<'a>(
    mut events: Vec<Event<'a>>,
    _ctxt: &mut Context,
) -> anyhow::Result<Vec<Event<'a>>> {
    let mut id_counter = [0; 6];

    for e in events.iter_mut() {
        if let Event::Start(Tag::Heading { level, id, .. }) = e {
            let level = *level as usize;
            id_counter.iter_mut().skip(level).for_each(|l| *l = 0);
            id_counter[level - 1] += 1;
            *id = Some(gen_id(&id_counter[1..]).into());
        }
    }

    Ok(events)
}
