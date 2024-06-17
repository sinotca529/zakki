use crate::{copy_asset, util::write_file};
use anyhow::Result;

pub fn init() -> Result<()> {
    copy_asset!("config.toml", "")?;
    Ok(())
}
