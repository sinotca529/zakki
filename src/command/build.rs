mod html;
mod metadata;
mod pass;

use super::{clean::clean, ensure_pwd_is_book_root_dir};
use crate::{
    command::build::html::Page,
    config::{Config, FileConfig},
    copy_asset,
    path::{dst_dir, src_dir, SrcPath},
    read_asset,
    util::{copy_file, write_file},
};
use anyhow::Result;
use metadata::Metadata;
use std::path::{Path, PathBuf};

pub fn build(render_draft: bool) -> Result<()> {
    ensure_pwd_is_book_root_dir()?;
    clean()?;

    let cfg = {
        let file_cfg = FileConfig::load()?;
        Config::new(file_cfg, render_draft)
    };

    render_index(&cfg)?;
    render_tag(&cfg)?;
    copy_asset!("style.css", "build")?;
    copy_asset!("script.js", "build")?;

    let metadata_list = render_pages(&cfg)?;
    save_metadata(&metadata_list)?;

    Ok(())
}

fn render_index(cfg: &Config) -> Result<()> {
    let content = format!(read_asset!("index.html"), site_name = cfg.site_name());
    write_file(dst_dir().join("index.html"), content).map_err(Into::into)
}

fn render_tag(cfg: &Config) -> Result<()> {
    let content = format!(read_asset!("tag.html"), site_name = cfg.site_name());
    write_file(dst_dir().join("tag.html"), content).map_err(Into::into)
}

fn render_pages(cfg: &Config) -> Result<Vec<Metadata>> {
    let mut metadata_list = vec![];
    visit_files_recursively(src_dir(), |p| {
        render_page(SrcPath::new(p).unwrap(), &mut metadata_list, cfg)
    })?;
    Ok(metadata_list)
}

fn save_metadata(metadata_list: &[Metadata]) -> Result<()> {
    let js = serde_json::to_string(metadata_list)?;
    let content = format!("const METADATA={js}");
    write_file(dst_dir().join("metadata.js"), content).map_err(Into::into)
}

fn copy_non_md(src_path: &SrcPath) -> Result<()> {
    assert!(!src_path.is_md());
    copy_file(src_path.get_ref(), src_path.to_dst_path().get_ref())
        .map(|_| ())
        .map_err(Into::into)
}

fn render_md(src_path: &SrcPath, cfg: &Config) -> Result<Metadata> {
    assert!(src_path.is_md());
    let page = Page::from_md_file(src_path)?;

    if cfg.render_draft() || !page.metadata().draft() {
        page.save(cfg)?;
    }

    Ok(page.into_metadata())
}

fn render_page(src_path: SrcPath, metadata_list: &mut Vec<Metadata>, cfg: &Config) -> Result<()> {
    if src_path.is_md() {
        let metadata = render_md(&src_path, cfg)?;
        if cfg.render_draft() || !metadata.draft() {
            metadata_list.push(metadata);
        }
    } else {
        copy_non_md(&src_path)?;
    }
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
