mod assets;
mod code_font;
mod renderer;

use crate::command::build::renderer::meta_renderer::MetaRenderer;
use crate::config::ProjectConfig;
use crate::path::ProjectPaths;
use crate::util::PathExt as _;
use anyhow::{self, Context as _, bail};
use itertools::{Either, Itertools as _};
use rayon::prelude::*;
use renderer::extract_title_from_path;
use renderer::page_renderer::PageRenderer;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::path::PathBuf;

pub fn build(pj_paths: &ProjectPaths, render_draft: bool) -> anyhow::Result<()> {
    let cfg = ProjectConfig::load(pj_paths.config_path())?;

    super::clean::clean(pj_paths)?;

    let files = pj_paths
        .src_dir()
        .descendants_file_paths()
        .with_context(|| {
            format!(
                "{} またはその配下のファイルを読めません",
                pj_paths.src_dir().display()
            )
        })?;

    // Wikilink のタイトルを書くため、全記事のタイトルを先んじて取得する。
    let title_map = collect_titles(&files)?;
    let renderer = PageRenderer::new(&cfg, &title_map, pj_paths, render_draft);

    assets::copy_assets(pj_paths.build_dir())?;

    // Result のまま集める。1 件目のエラーで打ち切ると、残りの記事の問題が分からない。
    let (mut metas, errors): (Vec<_>, Vec<_>) = files
        .par_iter()
        .map(|p| renderer.render(p).with_context(|| p.display().to_string()))
        .filter(|r| !matches!(r, Ok(None)))
        .partition_map(|r| match r {
            // Ok(None) は filter で除外済み
            Ok(output) => Either::Left(output.expect("Ok(None) は filter で除外済み")),
            Err(e) => Either::Right(e),
        });

    metas.sort_unstable_by_key(|m| Reverse(m.update));
    let metas = metas; // freeze

    if !errors.is_empty() {
        let list = errors.iter().map(|e| format!("{e:#}")).join("\n");
        bail!("記事を変換できませんでした\n{list}");
    }

    let mr = MetaRenderer::new(&cfg, pj_paths);
    mr.render(&metas)?;

    Ok(())
}

fn collect_titles(files: &[PathBuf]) -> anyhow::Result<HashMap<PathBuf, String>> {
    let mut map = HashMap::new();
    for path in files {
        if !path.extension_is("md") {
            continue;
        }
        let title = extract_title_from_path(path).with_context(|| path.display().to_string())?;
        if let Some(title) = title {
            map.insert(path.normalized(), title);
        }
    }
    Ok(map)
}
