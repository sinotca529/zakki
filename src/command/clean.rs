use super::ensure_pwd_is_book_root_dir;
use crate::path::dst_dir;
use anyhow::Result;
use std::fs::remove_dir_all;

pub fn clean() -> Result<()> {
    ensure_pwd_is_book_root_dir()?;
    if dst_dir().exists() {
        remove_dir_all(dst_dir()).map_err(Into::into)
    } else {
        Ok(())
    }
}
