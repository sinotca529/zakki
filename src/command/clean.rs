use crate::command::zakki_root;
use anyhow::Result;
use std::fs::remove_dir_all;

pub fn clean() -> Result<()> {
    let build_dir = zakki_root()?.join("build");
    if !build_dir.exists() {
        return Ok(());
    }
    remove_dir_all(build_dir).map_err(Into::into)
}
