use crate::command::build::renderer::context::Context;
use pulldown_cmark::{CowStr, Event, LinkType::Inline, Tag, TagEnd};

fn make_image_tag(
    url: &CowStr<'_>,
    alt: &Option<CowStr<'_>>,
    title: &Option<CowStr<'_>>,
) -> String {
    if url.ends_with(".svg") {
        return format!(r#"<object type="image/svg+xml" data="{url}"></object>"#);
    }

    let alt_attr = alt
        .as_ref()
        .map(|t| format!(r#" alt="{}""#, t))
        .unwrap_or_default();

    let title_attr = title
        .as_ref()
        .map(|t| format!(r#" title="{}""#, t))
        .unwrap_or_default();

    format!(r#"<img loading="lazy" src="{url}"{alt_attr}{title_attr}/>"#)
}

pub fn image_convert_pass<'a>(
    events: Vec<Event<'a>>,
    _: &mut Context,
) -> anyhow::Result<Vec<Event<'a>>> {
    let mut url = None;
    let mut title = None;
    let mut alt = None;

    let mut out = Vec::with_capacity(events.len());

    events.into_iter().for_each(|e| match e {
        Event::Start(Tag::Image {
            link_type: Inline,
            dest_url,
            title: t,
            ..
        }) => {
            url = Some(dest_url);
            title = Some(t);
        }
        Event::Text(t) if url.is_some() => {
            alt = Some(t);
        }
        Event::End(TagEnd::Image) if url.is_some() => {
            let (url, title, alt) = (url.take().unwrap(), title.take(), alt.take());
            let img_tag = make_image_tag(&url, &alt, &title);

            let figcaption_tag = alt
                .as_ref()
                .map(|alt| format!(r#"<figcaption>{}</figcaption>"#, alt))
                .unwrap_or_default();

            let figure_tag = format!(
                r#"<figure><div class="zakki-scroll">{img_tag}</div>{figcaption_tag}</figure>"#
            );

            out.push(Event::Html(figure_tag.into()))
        }
        _ => out.push(e),
    });

    Ok(out)
}
