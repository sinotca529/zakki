use super::{end_of, text_of};
use crate::command::build::renderer::html_component::{DIV, IMG, OBJECT};
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
        // 子ノード (alt テキスト) は figcaption に移す
        let alt = text_of(&events[(i + 1)..end]);

        out.push(Event::InlineHtml(open_tag(dest_url, &alt, title).into()));
        if !alt.is_empty() {
            out.push(Event::Text(alt.clone().into()));
            out.push(Event::InlineHtml("</figcaption></figure>".into()));
        } else {
            out.push(Event::InlineHtml("</figure>".into()));
        }
        i = end + 1;
    }

    *events = out;
}

/// figure の開きから figcaption の開きまでを組み立てます。
/// 閉じは呼び出し側が書きます。alt は `Text` として間に入れるためです。
fn open_tag(url: &str, alt: &str, title: &str) -> String {
    let img_tag = if url.ends_with(".svg") {
        // 文字列を選択できるようにするため、 SVG は object ノードで囲む
        OBJECT
            .attr("type", "image/svg+xml")
            .attr("data", url)
            .attr("title", title)
            .build()
    } else {
        IMG.attr("loading", "lazy")
            .attr("src", url)
            .attr("alt", alt)
            .attr("title", title)
            .build()
    };

    let scroll = DIV
        .attr("class", "x-scroll")
        .attr("tabindex", "0")
        .html(img_tag);
    let figcaption_open = if alt.is_empty() { "" } else { "<figcaption>" };

    format!("<figure>{scroll}{figcaption_open}")
}
