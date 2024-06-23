mod content;
mod renderer;

use super::{clean::clean, ensure_pwd_is_book_root_dir};
use crate::{
    config::{Config, FileConfig},
    path::src_dir,
};
use anyhow::Result;
use content::Content;
use renderer::Renderer;
use std::path::{Path, PathBuf};

pub fn build(render_draft: bool) -> Result<()> {
    ensure_pwd_is_book_root_dir()?;
    clean()?;

    let cfg = {
        let file_cfg = FileConfig::load()?;
        Config::new(file_cfg, render_draft)
    };

    let mut renderer = Renderer::new(cfg);
    renderer.render_assets()?;

    visit_files_recursively(src_dir(), |p| {
        let content = Content::new(p)?;
        renderer.render(content)?;
        Ok(())
    })?;

    renderer.save_metadata()?;

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
