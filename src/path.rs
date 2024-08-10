use anyhow::{bail, Result};
use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

pub fn src_dir() -> &'static PathBuf {
    static SRC_DIR: OnceLock<PathBuf> = OnceLock::new();
    SRC_DIR.get_or_init(|| {
        std::env::current_dir()
            .expect("Failed to get the current directry")
            .join("src")
    })
}

pub fn dst_dir() -> &'static PathBuf {
    static BUILD_DIR: OnceLock<PathBuf> = OnceLock::new();
    BUILD_DIR.get_or_init(|| {
        std::env::current_dir()
            .expect("Failed to get the current directry")
            .join("build")
    })
}

#[macro_export]
macro_rules! include_asset {
    ($fname:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/asset/", $fname))
    };
}

#[macro_export]
macro_rules! copy_asset {
    ($fname:literal, $to:literal) => {{
        let path = std::env::current_dir()?.join($to).join($fname);
        let result: Result<()> = if path.exists() {
            Ok(())
        } else {
            write_file(
                path,
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/asset/", $fname)),
            )
        }
        .map_err(Into::into);
        result
    }};
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct ContentPath {
    src_path: PathBuf,
    dst_path: PathBuf,
    path_to_dst_dir: PathBuf,
}

impl ContentPath {
    fn src_to_dst(src_path: &Path) -> PathBuf {
        let mut rel_path = src_path.strip_prefix(src_dir()).unwrap().to_owned();
        if matches!(rel_path.extension().and_then(|e| e.to_str()), Some("md")) {
            rel_path = rel_path.with_extension("html");
        }
        dst_dir().join(rel_path)
    }

    pub fn new(src_path: PathBuf) -> Result<Self> {
        if !src_path.starts_with(src_dir()) {
            bail!("path does not starts with src_dir");
        }

        let dst_path = Self::src_to_dst(&src_path);
        let path_to_dst_dir = pathdiff::diff_paths(dst_dir(), dst_path.parent().unwrap()).unwrap();

        Ok(Self {
            src_path,
            dst_path,
            path_to_dst_dir,
        })
    }

    pub fn src_path(&self) -> &PathBuf {
        &self.src_path
    }

    pub fn dst_path(&self) -> &PathBuf {
        &self.dst_path
    }

    pub fn path_to_dst_dir(&self) -> &PathBuf {
        &self.path_to_dst_dir
    }

    pub fn rel_path_from_dst_dir(&self) -> &Path {
        self.dst_path.strip_prefix(dst_dir()).unwrap()
    }
}
