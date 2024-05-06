mod html;
mod pass;

use self::html::PageMetadata;

use super::{clean::clean, init::init};
use crate::{
    command::build::html::Page,
    path::{dst_metadata_path, src_dir, SrcPath},
    util::{copy_file, write_file},
};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn build() -> Result<()> {
    clean()?;
    init()?;

    let metadata_list = render_pages()?;
    save_metadata(&metadata_list)?;

    Ok(())
}

fn render_pages() -> Result<Vec<PageMetadata>> {
    let mut metadata_list = vec![];
    visit_files_recursively(src_dir(), |p| render(p, &mut metadata_list))?;
    Ok(metadata_list)
}

fn save_metadata(metadata_list: &[PageMetadata]) -> Result<()> {
    let js = serde_json::to_string(metadata_list)?;
    let content = format!("const METADATA={js}");
    write_file(dst_metadata_path(), content).map_err(Into::into)
}

fn copy_non_md(src_path: &SrcPath) -> Result<()> {
    assert!(!src_path.is_md());
    copy_file(src_path.get_ref(), src_path.to_dst_path().get_ref())
        .map(|_| ())
        .map_err(Into::into)
}

fn render_md(src_path: &SrcPath) -> Result<PageMetadata> {
    assert!(src_path.is_md());
    let page = Page::from_md_file(src_path)?;
    page.save()?;
    Ok(page.metadata())
}

fn render(src_path: PathBuf, metadata_list: &mut Vec<PageMetadata>) -> Result<()> {
    let src_path = SrcPath::new(src_path)?;
    if src_path.is_md() {
        metadata_list.push(render_md(&src_path)?);
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
