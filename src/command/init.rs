use crate::{copy_asset, path::zakki_root, util::write_file};
use anyhow::{Context, Result, anyhow};

pub fn init() -> Result<()> {
    copy_asset!("zakki.toml", zakki_root()?)
}
