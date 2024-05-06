use anyhow::Result;

use crate::asset_path;
use crate::util::write_file;

macro_rules! copy_asset {
    ($fname:literal) => {{
        let path = crate::path::src_dir().join($fname);
        if path.exists() {
            return Ok(());
        }
        write_file(path, include_str!(asset_path!($fname)))
    }};
}

pub fn init() -> Result<()> {
    copy_asset!("style.css")?;
    copy_asset!("index.html")?;
    copy_asset!("tag.html")?;
    Ok(())
}
