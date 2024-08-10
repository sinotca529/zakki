mod content;
mod renderer;

use super::clean::clean;
use crate::config::Config;
use anyhow::{Context, Result};
use content::Content;
use renderer::Renderer;
use std::path::{Path, PathBuf};

pub fn build(cfg: &Config) -> Result<()> {
    clean(&cfg.dst_dir())?;

    let mut renderer = Renderer::new(cfg);
    renderer.render_assets()?;

    visit_files_recursively(cfg.src_dir(), |p| {
        let content = Content::new(p.clone()).with_context(|| p.to_str().unwrap().to_owned())?;
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
