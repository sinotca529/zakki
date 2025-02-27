mod renderer;

use super::clean::clean;
use crate::util::PathExt as _;
use crate::{config::Config, util::write_file};
use anyhow::{Context, Result};
use rayon::prelude::*;
use renderer::Renderer;
use renderer::context::Metadata;
use std::path::PathBuf;

fn render_pages(cfg: &Config) -> Result<Vec<Metadata>> {
    let renderer = Renderer::new(cfg);
    renderer.render_assets()?;

    let files = cfg.src_dir().descendants_file_paths()?;
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

fn output_metadatas(cfg: &Config, mut metas: Vec<Metadata>) -> Result<()> {
    // メタデータの書き出し
    metas.sort_unstable_by(|a, b| b.update().cmp(a.update()));
    let js = serde_json::to_string(&metas)?;
    let content = format!("const METADATA={js}");
    let dst = cfg.dst_dir().join("metadata.js");
    write_file(dst, content)?;

    // Bloom filter の書き出し
    let bloom: Vec<_> = metas.iter_mut().map(|e| e.bloom_filter()).collect();
    let js = serde_json::to_string(&bloom)?;
    let content = format!("const BLOOM_FILTER={js}");
    let dst = cfg.dst_dir().join("bloom_filter.js");
    write_file(dst, content)?;

    Ok(())
}

pub fn build(cfg: &Config) -> Result<()> {
    clean(cfg.dst_dir())?;

    let metadatas = render_pages(cfg)?;
    output_metadatas(cfg, metadatas)?;

    Ok(())
}
