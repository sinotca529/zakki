use super::dst_dir;
use anyhow::Result;

pub fn clean() -> Result<()> {
    std::fs::remove_dir_all(dst_dir())?;
    Ok(())
}
