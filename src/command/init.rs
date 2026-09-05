use crate::{copy_asset, path::zakki_root};
use anyhow::{Context, Result, anyhow, bail};

pub fn init() -> Result<()> {
    if zakki_root().is_ok() {
        bail!("このディレクトリはすでに zakki 用です");
    }
    copy_asset!("zakki.toml", std::env::current_dir()?)
}
