use super::{dst_dir, html_path, relative_path_to_css, src_dir};
use crate::convert::md_to_html;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn build() -> Result<()> {
    visit_files_recursively(src_dir(), convert_file)
}

fn convert_file(src_path: PathBuf) -> Result<()> {
    match src_path.extension().and_then(|e| e.to_str()) {
        Some("md") => {
            let md_content = std::fs::read(&src_path)?;
            let md_content = std::str::from_utf8(&md_content)?;

            let html_path = html_path(&src_path)?;

            let path_to_css = relative_path_to_css(&html_path)?;
            let html_content = md_to_html(md_content, path_to_css.to_str().unwrap());

            std::fs::write(html_path, html_content)?;
        }
        _ => {
            let dst_path = dst_dir().join(src_path.strip_prefix(src_dir())?);
            std::fs::create_dir_all(dst_path.parent().unwrap())?;
            std::fs::copy(src_path, dst_path)?;
        }
    }
    Ok(())
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
