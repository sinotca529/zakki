use super::build_dir;
use anyhow::Result;

pub fn clean() -> Result<()> {
    std::fs::remove_dir_all(build_dir())?;
    Ok(())
}
