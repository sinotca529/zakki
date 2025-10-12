use crate::{command::zakki_root, copy_asset, util::write_file};
use anyhow::{Context, Result, anyhow};

pub fn init() -> Result<()> {
    copy_asset!("zakki.toml", zakki_root()?)
}
