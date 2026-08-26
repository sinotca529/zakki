use crate::command::build::renderer::context::Context;
use itertools::Itertools;
use jotdown::{Container, Event};

fn gen_id(cntr: &[i32]) -> String {
    cntr.iter().take_while(|&&e| e > 0).join(".")
}

/// 見出しに階層番号の id を振ります。
///
/// djot は既定で見出しを `<section>` で包み、id をそちらに付けますが、
/// 既存の HTML 構造と目次生成に合わせて section は出力せず、
/// id は見出し自体に付けます。
pub fn assign_header_id<'a>(
    events: &mut Vec<Event<'a>>,
    _ctxt: &mut Context,
) -> anyhow::Result<()> {
    let mut id_counter = [0; 6];
    // 見出しは入れ子にならないので、Start で採番した id を End でも使う
    let mut current_id = None;
    let mut out = Vec::with_capacity(events.len());

    for e in events.drain(..) {
        match e {
            Event::Start(Container::Section { .. }, _) | Event::End(Container::Section { .. }) => {}
            Event::Start(Container::Heading { level, .. }, attrs) => {
                let l = level as usize;
                id_counter.iter_mut().skip(l).for_each(|c| *c = 0);
                id_counter[l - 1] += 1;
                let id: std::borrow::Cow<str> = gen_id(&id_counter[1..]).into();
                current_id = Some(id.clone());
                let heading = Container::Heading {
                    level,
                    has_section: false,
                    id,
                };
                out.push(Event::Start(heading, attrs));
            }
            Event::End(Container::Heading { level, .. }) => {
                out.push(Event::End(Container::Heading {
                    level,
                    has_section: false,
                    id: current_id.take().unwrap_or_default(),
                }));
            }
            _ => out.push(e),
        }
    }

    *events = out;
    Ok(())
}
