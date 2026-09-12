use std::fs::create_dir_all;

use crate::{copy_asset, path::ProjectPaths};
use anyhow::{Context as _, anyhow};

pub fn init() -> anyhow::Result<()> {
    let pj_paths = ProjectPaths::at_current_dir()?;

    copy_asset!("zakki.toml", pj_paths.root_dir())?;

    create_dir_all(pj_paths.src_public_dir())?;
    create_dir_all(pj_paths.src_private_dir())?;
    create_dir_all(pj_paths.src_draft_dir())?;

    Ok(())
}
