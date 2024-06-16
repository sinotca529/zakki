use crate::{copy_asset, util::write_file};
use anyhow::Result;

pub fn init() -> Result<()> {
    // let pwd = std::env::current_dir()?;
    // write_file(pwd.join("config.toml"), read_asset!("config.toml"))?;
    // Ok(())

    // copy_asset!("style.css")?;
    // copy_asset!("script.js")?;
    // copy_asset!("index.html")?;
    // copy_asset!("tag.html")?;
    copy_asset!("config.toml", "")?;
    Ok(())
}
