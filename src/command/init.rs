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
    let sr: Result<()> = copy_asset!("style.css").map_err(Into::into);
    let ir: Result<()> = copy_asset!("index.html").map_err(Into::into);
    let tr: Result<()> = copy_asset!("tag.html").map_err(Into::into);
    sr.or(ir).or(tr)
}
