use super::text_of;
use crate::command::build::renderer::context::Context;
use crate::util::PathExt as _;
use anyhow::anyhow;
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

    let md_links: Vec<_> = root
        .descendants()
        .filter_map(|node| match &node.data().value {
            NodeValue::Link(link) if is_local_md_url(&link.url) => {
                Some((node, link.url.clone(), node.first_child().is_some()))
            }
            NodeValue::WikiLink(link) => Some((node, link.url.clone(), text_of(node) != link.url)),
            _ => None,
        })
        .collect();

    for (node, url, title_is_specified) in md_links {
        let link_title = title_map
            .get(&src_dir.join(&url).normalized())
            .ok_or_else(|| {
                anyhow!(
                    "リンク先の記事が存在しないか、タイトルが設定されていません : {}",
                    url
                )
            })?;

        // タイトル未指定の場合 (ウィキリンクを含む) は、リンク先の記事のタイトルで埋める
        if !title_is_specified {
            node.children().for_each(|child| child.detach());
            let text = NodeValue::Text(link_title.clone().into());
            node.append(arena.alloc(AstNode::from(text)));
        }

        // url の末尾は html に変更する
        let url_stem = url
            .strip_suffix(".md")
            .expect("title_map に対応が存在する url のみが到達するため、末尾は必ず .md である");
        let html_url = format!("{url_stem}.html");

        // ウィキリンクも通常のリンクとして描画する
        let mut data = node.data_mut();
        match &mut data.value {
            NodeValue::Link(link) => link.url = html_url,
            NodeValue::WikiLink(_) => {
                data.value = NodeValue::Link(Box::new(NodeLink {
                    url: html_url,
                    title: String::new(),
                }))
            }
            _ => unreachable!("filter_map で Link と WikiLink だけを集めている"),
        }
    }

    Ok(())
}

fn is_local_md_url(url: &str) -> bool {
    !url.starts_with("http://") && !url.starts_with("https://") && url.ends_with(".md")
}
