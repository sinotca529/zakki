use super::{dst_dir, src_dir};
use anyhow::Result;

pub fn init() -> Result<()> {
    let demo_css_path = dst_dir().join("style.css");
    std::fs::create_dir(dst_dir())?;
    crate::util::write_file(demo_css_path, include_str!("../../asset/style.css"))?;

    let demo_md_path = src_dir().join("index.md");
    std::fs::File::create(demo_md_path)?;

    Ok(())
}
