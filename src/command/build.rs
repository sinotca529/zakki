mod renderer;

use super::clean::clean;
use crate::config::FileConfig;
use crate::path::{zakki_dst_dir, zakki_src_dir};
use crate::util::PathExt as _;
use crate::{config::Config, util::write_file};
use anyhow::{Context, Result};
use rayon::prelude::*;
use renderer::Renderer;
use renderer::metadata::Metadata;
use std::path::PathBuf;

fn render_pages(cfg: &Config) -> Result<Vec<Metadata>> {
    let renderer = Renderer::new(cfg);
    renderer.render_assets()?;

    let files = zakki_src_dir()?.descendants_file_paths()?;
    let metadatas: Vec<Metadata> = files
        .par_iter()
        .map(|p: &PathBuf| -> Result<Option<Metadata>> {
            renderer
                .render(p)
                .with_context(|| p.to_string_lossy().to_string())
        })
        .collect::<Result<Vec<Option<Metadata>>>>()?
        .into_iter()
        .flatten()
        .collect();

    Ok(metadatas)
}

fn output_sitemap(cfg: &Config, metas: &[Metadata]) -> Result<()> {
    let Some(publish_url) = cfg.publish_url() else {
        return Ok(());
    };
    let slash = if publish_url.ends_with('/') { "" } else { "/" };

    let mut content = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n".to_owned();
    content += "<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n";

    let is_plane = |m: &Metadata| -> bool { !m.dst_rel_path().unwrap().starts_with("private") };
    metas.iter().filter(|m| is_plane(m)).for_each(|m| {
        content += &format!(
            "  <url><loc>{publish_url}{slash}{}</loc><lastmod>{}</lastmod></url>\n",
            &m.dst_rel_path().unwrap().to_str().unwrap(),
            m.last_update_date().unwrap(),
        );
    });
    content += "</urlset>\n";

    let dst = zakki_dst_dir()?.join("sitemap.xml");
    write_file(dst, content)?;

    Ok(())
}

fn output_metadatas(mut metas: Vec<Metadata>) -> Result<()> {
    // メタデータの書き出し
    metas.sort_unstable_by(|a, b| {
        b.last_update_date()
            .unwrap()
            .cmp(a.last_update_date().unwrap())
    });
    let js = serde_json::to_string(&metas)?;
    let content = format!("const METADATA={js}");
    let dst = zakki_dst_dir()?.join("metadata.js");
    write_file(dst, content)?;

    // Bloom filter の書き出し
    let bloom: Vec<_> = metas.iter().map(|e| e.bloom_filter().unwrap()).collect();
    let js = serde_json::to_string(&bloom)?;
    let content = format!("const BLOOM_FILTER={js}");
    let dst = zakki_dst_dir()?.join("bloom_filter.js");
    write_file(dst, content)?;

    Ok(())
}

pub fn build(render_draft: bool) -> Result<()> {
    let file_cfg = FileConfig::load()?;
    let cfg = Config::new(file_cfg, render_draft);

    clean()?;

    let metadatas = render_pages(&cfg)?;
    output_sitemap(&cfg, &metadatas)?;
    output_metadatas(metadatas)?;

    Ok(())
}
