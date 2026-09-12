use crate::path::ProjectPaths;
use anyhow::Result;
use std::fs::remove_dir_all;

pub fn clean(pj_paths: &ProjectPaths) -> Result<()> {
    let build_dir = pj_paths.build_dir();

    if !build_dir.exists() {
        return Ok(());
    }

    remove_dir_all(build_dir).map_err(Into::into)
}
