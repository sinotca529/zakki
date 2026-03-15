use crate::command::build::renderer::metadata::Metadata;
use pulldown_cmark::{Event, LinkType, Tag};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn adjust_link_pass<'a>(
    events: &mut Vec<Event<'a>>,
    ctxt: &mut Metadata,
    title_map: &HashMap<PathBuf, String>,
) -> anyhow::Result<()> {
    wiki_link_sub_pass(events, ctxt, title_map)?;
    md_to_html_sub_pass(events);
    Ok(())
}

/// `[[path]]` 形式のウィキリンクについて、タイトルマップからタイトルを取得して差し替えます。
fn wiki_link_sub_pass<'a>(
    events: &mut Vec<Event<'a>>,
    ctxt: &mut Metadata,
    title_map: &HashMap<PathBuf, String>,
) -> anyhow::Result<()> {
    let src_path = ctxt.src_path()?;
    let src_dir = src_path.parent().unwrap_or(Path::new("")).to_owned();

    // [[path]] 形式のとき、次の Text イベントでタイトルを差し替えるために保持する
    let mut pending_title: Option<String> = None;

    for e in events.iter_mut() {
        match e {
            Event::Start(Tag::Link {
                link_type: LinkType::WikiLink { has_pothole: false },
                dest_url,
                ..
            }) => {
                pending_title = title_map.get(&src_dir.join(dest_url.as_ref())).cloned();
            }
            Event::Text(text) if pending_title.is_some() => {
                *text = pending_title.take().unwrap().into();
            }
            _ => {
                pending_title = None;
            }
        }
    }

    Ok(())
}

/// ローカルリンクの .md 拡張子を .html に変換します。
fn md_to_html_sub_pass<'a>(events: &mut Vec<Event<'a>>) {
    events.iter_mut().for_each(|mut e| {
        if let Event::Start(Tag::Link { dest_url: url, .. }) = &mut e {
            let is_local = !url.starts_with("http://") && !url.starts_with("https://");
            let is_md = url.ends_with(".md");
            if is_local && is_md {
                *url = format!("{}.html", &url[..url.len() - ".md".len()]).into();
            }
        }
    });
}
