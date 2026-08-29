use super::{escape_html_attr, text_of};
use comrak::nodes::{AstNode, NodeValue};

/// 画像を `<figure>` で囲み、alt テキストを `<figcaption>` にします。
pub fn convert_image<'a>(root: &'a AstNode<'a>) -> anyhow::Result<()> {
    let targets: Vec<_> = root
        .descendants()
        .filter_map(|node| match &node.data().value {
            NodeValue::Image(link) => {
                Some((node, link.url.clone(), link.title.clone(), text_of(node)))
            }
            _ => None,
        })
        .collect();

    for (node, url, title, alt) in targets {
        let alt = (!alt.is_empty()).then_some(alt);

        let img_tag = make_image_tag(&url, &alt, &title);
        let figcaption_tag = alt
            .as_ref()
            .map(|alt| format!(r#"<figcaption>{}</figcaption>"#, escape_html_attr(alt)))
            .unwrap_or_default();

        // 子ノード (alt テキスト) は figure に取り込んだので取り除く
        node.children().for_each(|child| child.detach());
        node.data_mut().value = NodeValue::HtmlInline(format!(
            r#"<figure><div class="zakki-scroll">{img_tag}</div>{figcaption_tag}</figure>"#
        ));
    }

    Ok(())
}

fn make_image_tag(url: &str, alt: &Option<String>, title: &str) -> String {
    // 文字列を選択できるようにするため、 SVG は object ノードで囲む
    if url.ends_with(".svg") {
        return format!(r#"<object type="image/svg+xml" data="{url}"></object>"#);
    }

    let alt_attr = alt
        .as_ref()
        .map(|t| format!(r#" alt="{}""#, escape_html_attr(t)))
        .unwrap_or_default();

    let title_attr = if title.is_empty() {
        String::new()
    } else {
        format!(r#" title="{}""#, escape_html_attr(title))
    };

    format!(r#"<img loading="lazy" src="{url}"{alt_attr}{title_attr}/>"#)
}
