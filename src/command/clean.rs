use anyhow::Result;
use std::{fs::remove_dir_all, path::Path};

pub fn clean(dst_dir: impl AsRef<Path>) -> Result<()> {
    let dst_dir = dst_dir.as_ref();
    if !dst_dir.exists() {
        return Ok(());
    }

    remove_dir_all(dst_dir).map_err(Into::into)
}
