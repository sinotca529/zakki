use crate::path::dst_dir;
use anyhow::Result;

pub fn clean() -> Result<()> {
    if !dst_dir().exists() {
        return Ok(());
    }
    std::fs::remove_dir_all(dst_dir()).map_err(Into::into)
}
