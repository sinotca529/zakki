mod content;
mod renderer;

use super::clean::clean;
use crate::util::PathExt as _;
use crate::{config::Config, util::write_file};
use anyhow::{Context, Result};
use content::{Content, Metadata};
use renderer::Renderer;
use serde::Serialize;
use std::path::PathBuf;

pub fn build(cfg: &Config) -> Result<()> {
    clean(&cfg.dst_dir())?;

    let mut renderer = Renderer::new(cfg);
    renderer.render_assets()?;

    let mut metadatas = Vec::new();
    let files = cfg.src_dir().descendants_file_paths()?;

    for p in &files {
        let content = Content::new(p.clone()).with_context(|| p.to_str().unwrap().to_owned())?;
        let metadata = renderer.render(content)?;
        if let Some(metadata) = metadata {
            metadatas.push(metadata);
        }
    }

    // メタデータの書き出し
    let metas: Vec<_> = metadatas
        .iter()
        .map(|m| MetadataToDump::from(m, &cfg))
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
    flags: &'a Vec<String>,
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
