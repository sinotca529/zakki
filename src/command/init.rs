use anyhow::Result;

use crate::asset_path;
use crate::util::write_file;

macro_rules! copy_asset {
    ($fname:literal) => {{
        let path = crate::path::src_dir().join($fname);
        let result: Result<()> = if path.exists() {
            Ok(())
        } else {
            write_file(path, include_str!(asset_path!($fname)))
        }
        .map_err(Into::into);
        result
    }};
}

pub fn init() -> Result<()> {
    let sr = copy_asset!("style.css");
    let jr = copy_asset!("script.js");
    let ir = copy_asset!("index.html");
    let tr = copy_asset!("tag.html");
    sr.or(jr).or(ir).or(tr)
}
