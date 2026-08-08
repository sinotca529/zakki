use crate::path::zakki_src_dir;
use anyhow::{Result, bail};
use std::path::PathBuf;

pub fn new(path: &str) -> Result<()> {
    let mut rel: PathBuf = path.into();
    if rel.extension().is_none() {
        rel.set_extension("md");
    }

    let dest = zakki_src_dir()?.join(&rel);

    if dest.exists() {
        bail!("{} already exists", dest.display());
    }

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let title = dest.file_stem().unwrap_or_default().to_string_lossy();
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let content = format!("---\ntitle: {title}\ncreate: {today}\nupdate: {today}\ntag: []\n---\n");

    std::fs::write(&dest, content)?;
    println!("Created: {}", dest.display());
    Ok(())
}
