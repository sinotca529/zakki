use crate::{copy_asset, util::write_file};
use anyhow::{Context, Result, anyhow};

pub fn init() -> Result<()> {
    copy_asset!("zakki.toml", std::env::current_dir()?)?;
    Ok(())
}
