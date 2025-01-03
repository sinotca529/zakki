use crate::command::build::renderer::context::Context;
use pulldown_cmark::{Event, LinkType::Inline, Tag, TagEnd};

pub fn image_convert_pass(events: &mut Vec<Event>, _: &mut Context) -> anyhow::Result<()> {
    let mut url = None;
    let mut title = None;
    let mut alt = None;

    events.iter_mut().for_each(move |e| match e {
        Event::Start(Tag::Image {
            link_type: Inline, ..
        }) => {
            let mut dummy = Event::Html("".into());
            std::mem::swap(e, &mut dummy);

            let Event::Start(Tag::Image {
                link_type: Inline,
                dest_url,
                title: t,
                ..
            }) = dummy
            else {
                unreachable!()
            };

            url = Some(dest_url);
            title = Some(t);
        }
        Event::Text(_) if url.is_some() => {
            let mut dummy = Event::Html("".into());
            std::mem::swap(e, &mut dummy);

            let Event::Text(t) = dummy else {
                unreachable!()
            };
            alt = Some(t);
        }
        Event::End(TagEnd::Image) if url.is_some() => {
            let url = url.take().unwrap();
            let alt_text = alt.take();

            let img_tag = if url.ends_with(".svg") {
                format!(r#"<object type="image/svg+xml" data="{url}">"#)
            } else {
                let alt_attr = alt_text
                    .as_ref()
                    .map(|t| format!(r#" alt="{}""#, t))
                    .unwrap_or_default();

                let title_attr = title
                    .as_ref()
                    .map(|t| format!(r#" title="{}""#, t))
                    .unwrap_or_default();

                format!(r#"<img loading="lazy" src="{url}"{alt_attr}{title_attr}/>"#)
            };

            let figcaption_tag = alt_text
                .as_ref()
                .map(|t| format!(r#"<figcaption>{}</figcaption>"#, t))
                .unwrap_or_default();

            let figure_tag = format!(
                r#"<figure><div class="zakki-scroll">{img_tag}</div>{figcaption_tag}</figure>"#
            );

            *e = Event::Html(figure_tag.into());
        }
        _ => {}
    });

    Ok(())
}
