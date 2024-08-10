use super::goto_zakki_root;
use crate::path::dst_dir;
use anyhow::Result;
use std::fs::remove_dir_all;

pub fn clean() -> Result<()> {
    goto_zakki_root()?;
    if dst_dir().exists() {
        remove_dir_all(dst_dir()).map_err(Into::into)
    } else {
        Ok(())
    }
}
