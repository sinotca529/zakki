use std::{fs::create_dir_all, path::PathBuf};

use crate::{config_file_name, copy_asset, path::ProjectPaths};
use anyhow::{Context as _, anyhow, bail};

pub fn init(at: PathBuf) -> anyhow::Result<()> {
    if ProjectPaths::find(&at).is_ok() {
        bail!("このプロジェクトはすでに zakki 用です")
    }

    let pj_paths = ProjectPaths::new(at);

    copy_asset!(config_file_name!(), pj_paths.root_dir())?;

    create_dir_all(pj_paths.src_public_dir())?;
    create_dir_all(pj_paths.src_private_dir())?;
    create_dir_all(pj_paths.src_draft_dir())?;

    Ok(())
}
