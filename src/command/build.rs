mod renderer;

use crate::config::FileConfig;
use crate::path::{zakki_dst_dir, zakki_src_dir};
use crate::util::PathExt as _;
use crate::{config::Config, util};
use anyhow::{Context as _, Result};
use rayon::prelude::*;
use renderer::Renderer;
use renderer::context::Metadata;
use renderer::extract_title_from_path;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::PathBuf;

pub fn build(render_draft: bool) -> Result<()> {
    let file_cfg = FileConfig::load()?;
    let cfg = Config::new(file_cfg, render_draft);

    super::clean::clean()?;

    let files = zakki_src_dir()?.descendants_file_paths()?;
    let title_map = collect_titles(&files)?;
    let renderer = Renderer::new(&cfg, &title_map);

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
    output_sitemap(&cfg, &metadatas)?;
    output_metadatas(metadatas)?;

    Ok(())
}

fn collect_titles(files: &[PathBuf]) -> Result<HashMap<PathBuf, String>> {
    let mut map = HashMap::new();
    for path in files {
        if !path.extension_is("dj") {
            continue;
        }
        let title = extract_title_from_path(path).with_context(|| path.display().to_string())?;
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

    let dst = zakki_dst_dir()?.join("sitemap.xml");
    util::write_file(dst, xml)?;

    Ok(())
}

fn output_metadatas(metas: Vec<Metadata>) -> Result<()> {
    let dst_dir = zakki_dst_dir()?;

    // メタデータの書き出し
    let json = serde_json::to_string(&metas)?;
    let js = format!("const METADATA={json}");
    let dst = dst_dir.join("metadata.js");
    util::write_file(dst, js)?;

    // Bloom filter の書き出し
    let blooms: Vec<_> = metas.iter().map(|o| &o.bloom).collect();
    let json = serde_json::to_string(&blooms)?;
    let js = format!("const BLOOM_FILTER={json}");
    let dst = dst_dir.join("bloom_filter.js");
    util::write_file(dst, js)?;

    Ok(())
}
