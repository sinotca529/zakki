mod renderer;

use super::clean::clean;
use crate::util::PathExt as _;
use crate::{config::Config, util::write_file};
use anyhow::{Context, Result};
use rayon::prelude::*;
use renderer::Metadata;
use renderer::Renderer;
use std::path::PathBuf;

pub fn build(cfg: &Config) -> Result<()> {
    clean(cfg.dst_dir())?;

    let renderer = Renderer::new(cfg);
    renderer.render_assets()?;

    let files = cfg.src_dir().descendants_file_paths()?;

    // 並列レンダリング
    let metadatas: Vec<Option<Metadata>> = files
        .par_iter()
        .map(|p: &PathBuf| -> Result<Option<Metadata>> {
            renderer
                .render(p.clone())
                .with_context(|| p.to_string_lossy().to_string())
        })
        .collect::<Result<Vec<Option<Metadata>>>>()?;

    // メタデータの書き出し
    let mut metas: Vec<_> = metadatas.into_iter().flatten().collect();
    metas.sort_unstable_by(|a, b| match (a.last_update_date(), b.last_update_date()) {
        (Ok(a), Ok(b)) => b.cmp(a),
        _ => std::cmp::Ordering::Equal,
    });

    let js = serde_json::to_string(&metas)?;
    let content = format!("const METADATA={js}");
    let dst = cfg.dst_dir().join("metadata.js");
    write_file(dst, content)?;

    Ok(())
}
