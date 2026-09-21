use super::{end_of, text_of};
use crate::command::build::renderer::html_component::{DIV, FIGCAPTION, FIGURE, IMG, OBJECT};
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

        let (figure_open, figure_close) = FIGURE.pair();
        let scroll = DIV
            .attr("class", "x-scroll")
            .attr("tabindex", "0")
            .html(img_tag(dest_url, &alt, title));

        out.push(Event::InlineHtml(format!("{figure_open}{scroll}").into()));

        // alt は figcaption に Text として置く。後続のパスが読めるようにするため
        match alt.is_empty() {
            true => out.push(Event::InlineHtml(figure_close.into())),
            false => {
                let (caption_open, caption_close) = FIGCAPTION.pair();
                out.push(Event::InlineHtml(caption_open.into()));
                out.push(Event::Text(alt.into()));
                out.push(Event::InlineHtml(
                    format!("{caption_close}{figure_close}").into(),
                ));
            }
        }
        i = end + 1;
    }

    *events = out;
}

/// 画像そのものを組み立てます。SVG かどうかで要素が変わります。
fn img_tag(url: &str, alt: &str, title: &str) -> String {
    if url.ends_with(".svg") {
        // SVG は object で埋め込む。img で読むと制限モードで描画され、中の文字を
        // 選択できず、Ctrl+F でも引っかからず、支援技術からも alt しか見えない。
        // object なら独立した文書として読まれるので、いずれも働く。
        // なお loading="lazy" は object には効かない (img と iframe だけ)。
        let mut attrs = vec![("type", "image/svg+xml"), ("data", url)];
        if !title.is_empty() {
            attrs.push(("title", title));
        }
        return OBJECT.attrs(&attrs).build();
    }

    let mut attrs = vec![("loading", "lazy"), ("src", url)];
    if !alt.is_empty() {
        attrs.push(("alt", alt));
    }
    if !title.is_empty() {
        attrs.push(("title", title));
    }
    IMG.attrs(&attrs).build()
}
