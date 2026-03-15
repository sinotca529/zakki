mod renderer;

use crate::config::FileConfig;
use crate::path::{zakki_dst_dir, zakki_src_dir};
use crate::util::PathExt as _;
use crate::{config::Config, util};
use anyhow::{Context, Result};
use rayon::prelude::*;
use renderer::Renderer;
use renderer::extract_title;
use renderer::metadata::Metadata;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::PathBuf;

pub fn build(render_draft: bool) -> Result<()> {
    let file_cfg = FileConfig::load()?;
    let cfg = Config::new(file_cfg, render_draft);

    super::clean::clean()?;

    let metadatas = render_pages(&cfg)?;
    output_sitemap(&cfg, &metadatas)?;
    output_metadatas(metadatas)?;

    Ok(())
}

fn render_pages(cfg: &Config) -> Result<Vec<Metadata>> {
    let files = zakki_src_dir()?.descendants_file_paths()?;

    // フェーズ1: 全ファイルのタイトルを収集する
    let title_map = collect_titles(&files)?;

    // フェーズ2: 本レンダリング
    let renderer = Renderer::new(cfg, &title_map);
    renderer.render_assets()?;

    let render_page = |p: &PathBuf| -> Result<Option<Metadata>> {
        renderer.render(p).with_context(|| p.display().to_string())
    };

    let metadatas: Vec<Metadata> = files
        .par_iter()
        .map(render_page)
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();

    Ok(metadatas)
}

fn collect_titles(files: &[PathBuf]) -> Result<HashMap<PathBuf, String>> {
    let mut map = HashMap::new();
    for path in files {
        if !path.extension_is("md") {
            continue;
        }
        let md = std::fs::read_to_string(path)?;
        let title = extract_title(&md)
            .with_context(|| path.display().to_string())?;
        if let Some(title) = title {
            map.insert(path.clone(), title);
        }
    }
    Ok(map)
}

fn output_sitemap(cfg: &Config, metas: &[Metadata]) -> Result<()> {
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
        let Ok(rel) = m.dst_rel_path() else { continue };
        if rel.starts_with("private") {
            continue;
        }

        let Ok(lastmod) = m.last_update_date() else {
            continue;
        };
        let path = rel.display().to_string();

        writeln!(
            &mut xml,
            r#"  <url><loc>{}{}</loc><lastmod>{}</lastmod></url>"#,
            pub_url, path, lastmod
        )?;
    }
    writeln!(&mut xml, "</urlset>")?;

    let dst = zakki_dst_dir()?.join("sitemap.xml");
    util::write_file(dst, xml)?;

    Ok(())
}

fn output_metadatas(mut metas: Vec<Metadata>) -> Result<()> {
    let dst_dir = zakki_dst_dir()?;

    // メタデータの書き出し
    metas.sort_unstable_by(|a, b| {
        b.last_update_date()
            .unwrap()
            .cmp(a.last_update_date().unwrap())
    });
    let json = serde_json::to_string(&metas)?;
    let js = format!("const METADATA={json}");
    let dst = dst_dir.join("metadata.js");
    util::write_file(dst, js)?;

    // Bloom filter の書き出し
    let bloom: Vec<_> = metas.iter().map(|e| e.bloom_filter().unwrap()).collect();
    let json = serde_json::to_string(&bloom)?;
    let js = format!("const BLOOM_FILTER={json}");
    let dst = dst_dir.join("bloom_filter.js");
    util::write_file(dst, js)?;

    Ok(())
}
