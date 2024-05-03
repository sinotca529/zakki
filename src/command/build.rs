use super::{clean::clean, init::init};
use crate::{
    convert::md_to_html,
    path::{src_dir, SrcPath},
    util::{copy_file, write_file},
};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn build() -> Result<()> {
    clean()?;
    init()?;
    visit_files_recursively(src_dir(), render)
}

fn copy_non_md(src_path: SrcPath) -> Result<()> {
    assert!(!src_path.is_md());
    copy_file(src_path.get_ref(), src_path.to_dst_path().get_ref())
        .map(|_| ())
        .map_err(Into::into)
}

fn render_md(src_path: SrcPath) -> Result<()> {
    assert!(src_path.is_md());
    let md_content = std::fs::read(src_path.get_ref())?;
    let md_content = std::str::from_utf8(&md_content)?;

    let dst_path = src_path.to_dst_path();
    let html_content = md_to_html(md_content, &dst_path);

    write_file(dst_path.get_ref(), html_content).map_err(Into::into)
}

fn render(src_path: PathBuf) -> Result<()> {
    let src_path = SrcPath::new(src_path)?;
    if src_path.is_md() {
        render_md(src_path)
    } else {
        copy_non_md(src_path)
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
