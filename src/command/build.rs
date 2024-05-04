mod pass;

use super::{clean::clean, init::init};
use crate::{
    command::build::pass::{adjust_link_to_md, convert_math, highlight_code},
    path::{src_dir, DstPath, SrcPath},
    util::{copy_file, write_file},
};
use anyhow::Result;
use indoc::formatdoc;
use pulldown_cmark::Options;
use std::path::{Path, PathBuf};

pub fn build() -> Result<()> {
    clean()?;
    init()?;
    visit_files_recursively(src_dir(), render)
}

fn md_to_html(md_content: &str, dst_path: &DstPath) -> String {
    let mut body = String::new();

    let parser = pulldown_cmark::Parser::new_ext(md_content, Options::all())
        .map(adjust_link_to_md)
        .map(convert_math)
        .map(highlight_code);

    pulldown_cmark::html::push_html(&mut body, parser);

    formatdoc! {r#"
            <!DOCTYPE html>
            <html lang="ja">
            <head>
            <meta charset="UTF-8">
            <link rel="stylesheet" href="{path_to_css}">
            </head>
            <body>
            {body}
            </body>
            </html>
        "#,
        path_to_css = dst_path.path_to_css().to_str().unwrap(),
        body = body,
    }
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
