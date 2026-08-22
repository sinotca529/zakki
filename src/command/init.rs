use crate::{copy_asset, path::zakki_root};
use anyhow::{Context, Result, anyhow, bail};

pub fn init() -> Result<()> {
    if zakki_root().is_ok() {
        bail!("Current directory is already a zakki project.");
    }
    copy_asset!("zakki.toml", std::env::current_dir()?)
}
