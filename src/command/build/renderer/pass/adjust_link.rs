use super::text_of;
use crate::command::build::renderer::context::Context;
use comrak::Arena;
use comrak::nodes::{AstNode, NodeLink, NodeValue};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// リンクを調整します。
///
/// - `[[path]]` 形式のウィキリンクを、リンク先の記事のタイトルを表示する通常のリンクにします
/// - リンク文字列が空のローカルリンクも、同じくタイトルで埋めます
/// - ローカルリンクの `.md` 拡張子を `.html` に変換します
pub fn adjust_link<'a>(
    arena: &'a Arena<'a>,
    root: &'a AstNode<'a>,
    ctx: &mut Context,
    title_map: &HashMap<PathBuf, String>,
) -> anyhow::Result<()> {
    let src_dir = ctx.src_path()?.parent().unwrap_or(Path::new("")).to_owned();

    let links: Vec<_> = root
        .descendants()
        .filter_map(|node| match &node.data().value {
            NodeValue::Link(link) => Some((node, link.url.clone(), false)),
            NodeValue::WikiLink(link) => Some((node, link.url.clone(), true)),
            _ => None,
        })
        .collect();

    for (node, url, is_wiki) in links {
        // ウィキリンクとリンク文字列が空のリンクは、リンク先の記事のタイトルで埋める
        let fill_title = is_local_url(&url) && (is_wiki || text_of(node).is_empty());
        if let Some(title) = fill_title
            .then(|| title_map.get(&src_dir.join(&url)))
            .flatten()
        {
            node.children().for_each(|child| child.detach());
            let text = NodeValue::Text(title.clone().into());
            node.append(arena.alloc(AstNode::from(text)));
        }

        let html_url = match url.strip_suffix(".md") {
            Some(stem) if is_local_url(&url) => format!("{stem}.html"),
            _ => url,
        };

        // ウィキリンクも通常のリンクとして描画する
        let mut data = node.data_mut();
        match &mut data.value {
            NodeValue::Link(link) => link.url = html_url,
            _ => {
                data.value = NodeValue::Link(Box::new(NodeLink {
                    url: html_url,
                    title: String::new(),
                }))
            }
        }
    }

    Ok(())
}

fn is_local_url(url: &str) -> bool {
    !url.starts_with("http://") && !url.starts_with("https://")
}
