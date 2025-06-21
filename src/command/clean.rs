use super::goto_zakki_root;
use anyhow::Result;
use std::fs::remove_dir_all;

pub fn clean() -> Result<()> {
    goto_zakki_root()?;
    let zakki_root = std::env::current_dir()?.join("build");
    if !zakki_root.exists() {
        return Ok(());
    }
    remove_dir_all(zakki_root).map_err(Into::into)
}
