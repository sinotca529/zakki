use crate::util::PathExt;
use anyhow::{Result, anyhow, bail};
use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

static ZAKKI_ROOT_DIR: LazyLock<Result<PathBuf>> = LazyLock::new(|| {
    let pwd = std::env::current_dir()?;
    let mut dir: Option<&Path> = Some(pwd.as_ref());

    while let Some(d) = dir {
        let is_zakki_root = d.has_file("zakki.toml")?;
        if is_zakki_root {
            return Ok(d.to_owned());
        }
        dir = d.parent();
    }

    bail!("Failed to detect zakki root.");
});

static ZAKKI_SRC_DIR: LazyLock<Result<PathBuf>> =
    LazyLock::new(|| zakki_root().map(|p| p.join("src")));

static ZAKKI_DST_DIR: LazyLock<Result<PathBuf>> =
    LazyLock::new(|| zakki_root().map(|p| p.join("build")));

pub fn zakki_root() -> Result<&'static PathBuf> {
    ZAKKI_ROOT_DIR.as_ref().map_err(|e| anyhow!(e.to_string()))
}

pub fn zakki_dst_dir() -> Result<&'static PathBuf> {
    ZAKKI_DST_DIR.as_ref().map_err(|e| anyhow!(e.to_string()))
}

pub fn zakki_src_dir() -> Result<&'static PathBuf> {
    ZAKKI_SRC_DIR.as_ref().map_err(|e| anyhow!(e.to_string()))
}

pub fn dst_path_of(src_path: impl AsRef<Path>) -> Result<PathBuf> {
    let src_path = src_path.as_ref();
    let rel = src_path.strip_prefix(zakki_src_dir()?).unwrap();

    if rel.extension_is("dj") {
        Ok(zakki_dst_dir()?.join(rel.with_extension("html")))
    } else {
        Ok(zakki_dst_dir()?.join(rel))
    }
}
