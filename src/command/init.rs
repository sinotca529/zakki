use crate::{copy_asset, util::write_file};
use anyhow::{anyhow, Context, Result};

pub fn init() -> Result<()> {
    copy_asset!("zakki.toml", std::env::current_dir()?)?;
    Ok(())
}
