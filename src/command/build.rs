mod renderer;

use crate::config::FileConfig;
use crate::path::ProjectPaths;
use crate::util::PathExt as _;
use crate::{config::Config, util};
use anyhow::{Context as _, Result};
use rayon::prelude::*;
use renderer::Renderer;
use renderer::context::Metadata;
use renderer::extract_title_from_path;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

pub fn build(pj_paths: &ProjectPaths, render_draft: bool) -> Result<()> {
    let file_cfg = FileConfig::load(pj_paths.config_path())?;
    let cfg = Config::new(file_cfg, render_draft);

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
    let renderer = Renderer::new(&cfg, &title_map, pj_paths);

    renderer.render_assets()?;

    let contexts = files
        .par_iter()
        .map(|p| renderer.render(p).with_context(|| p.display().to_string()))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    let mut metadatas: Vec<Metadata> = contexts
        .into_iter()
        .map(|c| c.into_output())
        .collect::<Result<_>>()?;
    metadatas.sort_unstable_by(|a, b| b.update.cmp(&a.update));

    renderer.render_index(&metadatas)?;
    output_sitemap(&cfg, &metadatas, pj_paths.build_dir())?;
    output_metadatas(metadatas, pj_paths.build_dir())?;

    Ok(())
}

fn collect_titles(files: &[PathBuf]) -> Result<HashMap<PathBuf, String>> {
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

fn output_sitemap(cfg: &Config, metas: &[Metadata], build_dir: &Path) -> Result<()> {
    let pub_url = cfg.publish_url().map(|u| u.trim_end_matches("/"));
    let Some(pub_url) = pub_url else {
        return Ok(());
    };

    let mut xml = String::new();
    writeln!(&mut xml, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(
        &mut xml,
        r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#
    )?;

    for m in metas {
        if m.path.starts_with("private") {
            continue;
        }
        let path = m.path.display().to_string();
        writeln!(
            &mut xml,
            r#"  <url><loc>{}{}</loc><lastmod>{}</lastmod></url>"#,
            pub_url, path, m.update
        )?;
    }
    writeln!(&mut xml, "</urlset>")?;

    let sitemap_path = build_dir.join("sitemap.xml");
    util::write_file(sitemap_path, xml)?;

    Ok(())
}

// METADATA と BLOOM_FILTER を書き出します。
// どちらも metas を先頭から順に並べるため、添字が同じ要素が同じ記事を指します。
// クライアント側の検索がこの対応を前提にしているので、片方だけ並べ替えないでください。
fn output_metadatas(metas: Vec<Metadata>, build_dir: &Path) -> Result<()> {
    // メタデータの書き出し
    let json = serde_json::to_string(&metas)?;
    let js = format!("const METADATA={json}");
    let metadata_path = build_dir.join("metadata.js");
    util::write_file(metadata_path, js)?;

    // Bloom filter の書き出し
    let blooms: Vec<_> = metas.iter().map(|o| &o.bloom).collect();
    let json = serde_json::to_string(&blooms)?;
    let js = format!("const BLOOM_FILTER={json}");
    let bloom_filter_path = build_dir.join("bloom_filter.js");
    util::write_file(bloom_filter_path, js)?;

    Ok(())
}
