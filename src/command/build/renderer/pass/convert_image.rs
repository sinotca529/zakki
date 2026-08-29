use super::{escape_html_attr, text_of};
use comrak::nodes::{AstNode, NodeValue};

/// 画像を `<figure>` で囲み、alt テキストを `<figcaption>` にします。
pub fn convert_image<'a>(root: &'a AstNode<'a>) -> anyhow::Result<()> {
    let img_nodes: Vec<_> = root
        .descendants()
        .filter_map(|node| match &node.data().value {
            NodeValue::Image(link) => {
                Some((node, link.url.clone(), link.title.clone(), text_of(node)))
            }
            _ => None,
        })
        .collect();

    for (node, url, title, alt) in img_nodes {
        // 子ノード (alt テキスト) は figure に取り込むので取り除く
        node.children().for_each(|child| child.detach());

        // figure タグの追加
        let figure_tag = make_figure_tag(&url, &alt, &title);
        node.data_mut().value = NodeValue::HtmlInline(figure_tag);
    }

    Ok(())
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
        .map(|alt| format!(r#"<figcaption>{}</figcaption>"#, escape_html_attr(alt)))
        .unwrap_or_default();

    format!(r#"<figure><div class="zakki-scroll">{img_tag}</div>{figcaption_tag}</figure>"#)
}
