use super::{dst_dir, src_dir};
use anyhow::Result;

pub fn init() -> Result<()> {
    let demo_css_path = dst_dir().join("style.css");
    if !demo_css_path.exists() {
        crate::util::write_file(
            demo_css_path,
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/asset/style.css")),
        )?;
    }

    let demo_md_path = src_dir().join("index.md");
    if !demo_md_path.exists() {
        crate::util::write_file(demo_md_path, "# Index\n")?;
    }

    Ok(())
}
