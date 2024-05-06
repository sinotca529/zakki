use anyhow::Result;

use crate::path::{src_css_path, src_index_path};
use crate::util::write_file;
use crate::{default_css_path, default_index_path};

fn copy_default_css() -> Result<()> {
    if !src_css_path().exists() {
        write_file(src_css_path(), include_str!(default_css_path!()))?;
    }
    Ok(())
}

fn copy_default_index() -> Result<()> {
    if !src_index_path().exists() {
        write_file(src_index_path(), include_str!(default_index_path!()))?;
    }
    Ok(())
}

pub fn init() -> Result<()> {
    copy_default_css()?;
    copy_default_index()?;
    Ok(())
}
