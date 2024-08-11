mod content;
mod renderer;

use super::clean::clean;
use crate::util::PathExt as _;
use crate::{config::Config, util::write_file};
use anyhow::{Context, Result};
use content::{Content, Flag, Metadata};
use rayon::prelude::*;
use renderer::Renderer;
use serde::Serialize;
use std::path::PathBuf;

pub fn build(cfg: &Config) -> Result<()> {
    clean(&cfg.dst_dir())?;

    let renderer = Renderer::new(cfg);
    renderer.render_assets()?;

    let files = cfg.src_dir().descendants_file_paths()?;

    // 並列レンダリング
    let metadatas: Vec<Option<Metadata>> = files
        .par_iter()
        .map(|p: &PathBuf| -> Result<Option<Metadata>> {
            let content =
                Content::new(p.clone()).with_context(|| p.to_string_lossy().to_string())?;
            renderer.render(content)
        })
        .collect::<Result<Vec<Option<Metadata>>>>()?;

    // メタデータの書き出し
    let metas: Vec<_> = metadatas
        .iter()
        .filter_map(|x| x.as_ref())
        .map(|m| MetadataToDump::from(&m, &cfg))
        .collect();
    let js = serde_json::to_string(&metas)?;
    let content = format!("const METADATA={js}");
    let dst = cfg.dst_dir().join("metadata.js");
    write_file(dst, content)?;

    Ok(())
}

#[derive(Serialize)]
struct MetadataToDump<'a> {
    create: &'a String,
    update: &'a String,
    tags: &'a Vec<String>,
    flags: &'a Vec<Flag>,
    title: &'a String,
    path: PathBuf,
}

impl<'a> MetadataToDump<'a> {
    fn from(meta: &'a Metadata, cfg: &Config) -> Self {
        Self {
            create: &meta.create_date,
            update: &meta.last_update_date,
            tags: &meta.tags,
            flags: &meta.flags,
            title: &meta.title,
            path: cfg
                .dst_path_of(&meta.src_path)
                .relative_path(cfg.dst_dir())
                .unwrap(),
        }
    }
}
