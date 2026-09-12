use crate::path::zakki_dst_dir;
use anyhow::Result;
use std::fs::remove_dir_all;

pub fn clean() -> Result<()> {
    let build_dir = zakki_dst_dir()?;
    if !build_dir.exists() {
        return Ok(());
    }
    remove_dir_all(build_dir).map_err(Into::into)
}
