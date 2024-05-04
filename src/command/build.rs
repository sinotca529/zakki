mod html;
mod pass;

use super::{clean::clean, init::init};
use crate::{
    command::build::html::Page,
    path::{src_dir, SrcPath},
    util::copy_file,
};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn build() -> Result<()> {
    clean()?;
    init()?;
    visit_files_recursively(src_dir(), render)
}

fn copy_non_md(src_path: &SrcPath) -> Result<()> {
    assert!(!src_path.is_md());
    copy_file(src_path.get_ref(), src_path.to_dst_path().get_ref())
        .map(|_| ())
        .map_err(Into::into)
}

fn render_md(src_path: &SrcPath) -> Result<()> {
    assert!(src_path.is_md());
    let page = Page::from_md_file(src_path)?;
    page.save()
}

fn render(src_path: PathBuf) -> Result<()> {
    let src_path = SrcPath::new(src_path)?;
    if src_path.is_md() {
        render_md(&src_path)
    } else {
        copy_non_md(&src_path)
    }
}

fn visit_files_recursively(
    dir: impl AsRef<Path>,
    operator: impl Fn(PathBuf) -> Result<()>,
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
