use super::{end_of, text_of};
use crate::command::build::renderer::html_component::{escape_html_attr, escape_html_text};
use pulldown_cmark::{Event, Tag};

/// 画像を `<figure>` で囲み、alt テキストを `<figcaption>` にします。
pub fn convert_image(events: &mut Vec<Event<'_>>) {
    let mut out = Vec::with_capacity(events.len());
    let mut i = 0;

    while i < events.len() {
        let Event::Start(Tag::Image {
            dest_url, title, ..
        }) = &events[i]
        else {
            out.push(events[i].clone());
            i += 1;
            continue;
        };

        let end = end_of(events, i);
        // 子ノード (alt テキスト) は figure に取り込むので取り除く
        let alt = text_of(&events[(i + 1)..end]);

        out.push(Event::InlineHtml(
            make_figure_tag(dest_url, &alt, title).into(),
        ));
        i = end + 1;
    }

    *events = out;
}

fn make_figure_tag(url: &str, alt: &str, title: &str) -> String {
    let alt = (!alt.is_empty()).then_some(alt);
    let title = (!title.is_empty()).then_some(title);

    let title_attr = title
        .map(|title| format!(r#" title="{}""#, escape_html_attr(title)))
        .unwrap_or_default();

    let img_tag = if url.ends_with(".svg") {
        // 文字列を選択できるようにするため、 SVG は object ノードで囲む
        format!(r#"<object type="image/svg+xml" data="{url}"{title_attr}></object>"#)
    } else {
        let alt_attr = alt
            .as_ref()
            .map(|t| format!(r#" alt="{}""#, escape_html_attr(t)))
            .unwrap_or_default();

        format!(r#"<img loading="lazy" src="{url}"{alt_attr}{title_attr}/>"#)
    };

    let figcaption_tag = alt
        .as_ref()
        .map(|alt| format!(r#"<figcaption>{}</figcaption>"#, escape_html_text(alt)))
        .unwrap_or_default();

    format!(
        r#"<figure><div class="x-scroll" tabindex="0">{img_tag}</div>{figcaption_tag}</figure>"#
    )
}
