use crate::command::build::renderer::context::Context;
use jotdown::{Container, Event};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// リンクを調整します。
///
/// - リンク文字列が空のローカルリンクは、リンク先の記事のタイトルで埋めます
///   (例: `[](foo.dj)` → `[foo の title](foo.html)`)
/// - ローカルリンクの `.dj` 拡張子を `.html` に変換します
pub fn adjust_link<'a>(
    events: &mut Vec<Event<'a>>,
    ctxt: &mut Context,
    title_map: &HashMap<PathBuf, String>,
) -> anyhow::Result<()> {
    let src_dir = ctxt
        .src_path()?
        .parent()
        .unwrap_or(Path::new(""))
        .to_owned();

    let mut out = Vec::with_capacity(events.len());
    let mut iter = std::mem::take(events).into_iter().peekable();

    while let Some(e) = iter.next() {
        match e {
            Event::Start(Container::Link(url, link_type), attrs) => {
                let is_local = !url.starts_with("http://") && !url.starts_with("https://");

                let is_empty_text =
                    matches!(iter.peek(), Some(Event::End(Container::Link(..))));
                let title = (is_local && is_empty_text)
                    .then(|| title_map.get(&src_dir.join(url.as_ref())))
                    .flatten()
                    .cloned();

                let url: Cow<'a, str> = match url.strip_suffix(".dj") {
                    Some(stem) if is_local => format!("{stem}.html").into(),
                    _ => url,
                };

                out.push(Event::Start(Container::Link(url, link_type), attrs));
                if let Some(title) = title {
                    out.push(Event::Str(title.into()));
                }
            }
            _ => out.push(e),
        }
    }

    *events = out;
    Ok(())
}
