use std::fs::create_dir_all;

use crate::{config_file_name, copy_asset, path::ProjectPaths};
use anyhow::{Context as _, anyhow, bail};

pub fn init() -> anyhow::Result<()> {
    if ProjectPaths::find().is_ok() {
        bail!("このプロジェクトはすでに zakki 用です")
    }

    let pj_paths = ProjectPaths::at_current_dir()?;

    copy_asset!(config_file_name!(), pj_paths.root_dir())?;

    create_dir_all(pj_paths.src_public_dir())?;
    create_dir_all(pj_paths.src_private_dir())?;
    create_dir_all(pj_paths.src_draft_dir())?;

    Ok(())
}
