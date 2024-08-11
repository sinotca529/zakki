mod content;
mod renderer;

use super::clean::clean;
use crate::util::PathExt as _;
use crate::{config::Config, util::write_file};
use anyhow::{Context, Result};
use content::{Content, Metadata};
use renderer::Renderer;
use serde::Serialize;
use std::path::{Path, PathBuf};

pub fn build(cfg: &Config) -> Result<()> {
    clean(&cfg.dst_dir())?;

    let mut renderer = Renderer::new(cfg);
    renderer.render_assets()?;

    let mut metadatas = Vec::new();

    visit_files_recursively(cfg.src_dir(), |p| {
        let content = Content::new(p.clone()).with_context(|| p.to_str().unwrap().to_owned())?;
        let metadata = renderer.render(content)?;
        if let Some(metadata) = metadata {
            metadatas.push(metadata);
        }
        Ok(())
    })?;

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

fn visit_files_recursively(
    dir: impl AsRef<Path>,
    mut operator: impl FnMut(PathBuf) -> Result<()>,
) -> Result<()> {
    let dir = dir.as_ref();
    let mut work_list: Vec<PathBuf> = vec![dir.into()];
    while let Some(dir) = work_list.pop() {
        for e in std::fs::read_dir(&dir)? {
            let path = e?.path();
            if path.is_dir() {
                work_list.push(path);
            } else {
                operator(path)?;
            }
        }
    }
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
