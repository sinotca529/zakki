use super::{end_of, text_of};
use crate::util::PathExt as _;
use anyhow::anyhow;
use pulldown_cmark::{CowStr, Event, LinkType, Tag, TagEnd};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// リンクを調整します。
///
/// - `[[path]]` 形式のウィキリンクを、リンク先の記事のタイトルを表示する通常のリンクにします
/// - リンク文字列が空のローカルリンクも、同じくタイトルで埋めます
/// - ローカルリンクの `.md` 拡張子を `.html` に変換します
pub fn adjust_link(
    events: &mut Vec<Event<'_>>,
    src_path: &Path,
    title_map: &HashMap<PathBuf, String>,
) -> anyhow::Result<()> {
    let src_dir = src_path.parent().unwrap_or(Path::new("")).to_owned();

    let mut out = Vec::with_capacity(events.len());
    let mut i = 0;

    while i < events.len() {
        let Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) = &events[i]
        else {
            out.push(events[i].clone());
            i += 1;
            continue;
        };

        let is_wiki = matches!(link_type, LinkType::WikiLink { .. });
        if !is_wiki && !is_local_md_url(dest_url) {
            out.push(events[i].clone());
            i += 1;
            continue;
        }

        let url = dest_url.to_string();
        let end = end_of(events, i);
        let inner = &events[(i + 1)..end];

        let link_title = title_map
            .get(&src_dir.join(&url).normalized())
            .ok_or_else(|| {
                anyhow!(
                    "リンク先の記事が存在しないか、タイトルが設定されていません : {}",
                    url
                )
            })?;

        // ウィキリンクは表示名を書かないと url がそのままテキストになる
        let shown = text_of(inner);
        let title_is_specified = !shown.is_empty() && shown != url;

        // url の末尾は html に変更する
        let url_stem = url
            .strip_suffix(".md")
            .expect("title_map に対応が存在する url のみが到達するため、末尾は必ず .md である");

        // ウィキリンクも通常のリンクとして描画する
        out.push(Event::Start(Tag::Link {
            link_type: LinkType::Inline,
            dest_url: CowStr::from(format!("{url_stem}.html")),
            title: title.clone(),
            id: id.clone(),
        }));

        if title_is_specified {
            out.extend(inner.iter().cloned());
        } else {
            out.push(Event::Text(link_title.clone().into()));
        }

        out.push(Event::End(TagEnd::Link));
        i = end + 1;
    }

    *events = out;
    Ok(())
}

fn is_local_md_url(url: &str) -> bool {
    !url.starts_with("http://") && !url.starts_with("https://") && url.ends_with(".md")
}
