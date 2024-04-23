use super::{build_dir, src_dir};
use crate::util::write_file;
use anyhow::Result;
use std::path::PathBuf;

pub fn init() -> Result<()> {
    let src_dir: PathBuf = src_dir()?;
    let demo_md_path = src_dir.join("index.md");
    std::fs::create_dir(src_dir)?;

    let build_dir = build_dir()?;
    let demo_css_path = build_dir.join("style.css");
    std::fs::create_dir(build_dir)?;

    std::fs::File::create(demo_md_path)?;

    std::fs::File::create(&demo_css_path)?;
    write_file(demo_css_path, include_str!("../../asset/style.css"))?;
    Ok(())
}
