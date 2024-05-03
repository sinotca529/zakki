use anyhow::Result;

use crate::default_css_path;
use crate::path::{src_css_path, src_dir};

fn copy_default_css() -> Result<()> {
    crate::util::write_file(src_css_path(), include_str!(default_css_path!())).map_err(Into::into)
}

fn make_index_page() -> Result<()> {
    let demo_md_path = src_dir().join("index.md");
    if !demo_md_path.exists() {
        crate::util::write_file(demo_md_path, "# Index\n")?;
    }
    Ok(())
}

pub fn init() -> Result<()> {
    copy_default_css()?;
    make_index_page()?;
    Ok(())
}
