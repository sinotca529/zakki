use super::{attrs_to_html, raw_html};
use crate::command::build::renderer::context::Context;
use jotdown::{Container, Event};

/// 画像を `<figure>` で囲み、alt テキストを `<figcaption>` にします。
pub fn convert_image<'a>(
    events: &mut Vec<Event<'a>>,
    _ctxt: &mut Context,
) -> anyhow::Result<()> {
    let mut out = Vec::with_capacity(events.len());
    let mut url: Option<String> = None;
    let mut attrs_html = String::new();
    let mut alt = String::new();

    for e in events.drain(..) {
        match e {
            Event::Start(Container::Image(u, _), attrs) => {
                url = Some(u.into_owned());
                attrs_html = attrs_to_html(&attrs);
                alt.clear();
            }
            Event::Str(ref s) if url.is_some() => alt.push_str(s),
            Event::End(Container::Image(..)) => {
                let url = url.take().unwrap();
                let attrs_html = std::mem::take(&mut attrs_html);
                let alt = (!alt.is_empty()).then(|| std::mem::take(&mut alt));

                let img_tag = make_image_tag(&url, &alt, &attrs_html);
                let figcaption_tag = alt
                    .as_ref()
                    .map(|alt| format!(r#"<figcaption>{}</figcaption>"#, alt))
                    .unwrap_or_default();

                out.extend(raw_html(format!(
                    r#"<figure><div class="zakki-scroll">{img_tag}</div>{figcaption_tag}</figure>"#
                )));
            }
            _ => out.push(e),
        }
    }

    *events = out;
    Ok(())
}

fn make_image_tag(url: &str, alt: &Option<String>, attrs_html: &str) -> String {
    if url.ends_with(".svg") {
        return format!(r#"<object type="image/svg+xml" data="{url}"{attrs_html}></object>"#);
    }

    let alt_attr = alt
        .as_ref()
        .map(|t| format!(r#" alt="{}""#, t))
        .unwrap_or_default();

    format!(r#"<img loading="lazy" src="{url}"{alt_attr}{attrs_html}/>"#)
}
