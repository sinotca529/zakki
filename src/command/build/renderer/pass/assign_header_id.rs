use crate::command::build::renderer::context::Context;
use pulldown_cmark::{Event, Tag};

fn gen_id(cntr: &[i32]) -> String {
    return cntr
        .iter()
        .take_while(|&&e| e > 0)
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(".");
}

pub fn assign_header_id<'a>(mut events: Vec<Event<'a>>, _ctxt: &mut Context) -> anyhow::Result<Vec<Event<'a>>> {
    let mut id_counter = [0; 6];

    for e in events.iter_mut() {
        match e {
            Event::Start(Tag::Heading { level, id, .. }) => {
                let level = *level as usize;
                for i in level..id_counter.len() {
                    id_counter[i] = 0;
                }
                id_counter[level - 1] += 1;
                *id = Some(gen_id(&id_counter[1..]).into());
            }
            _ => {}
        }
    }

    Ok(events)
}
