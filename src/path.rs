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

pub fn dst_metadata_path() -> &'static PathBuf {
    static METADATA_PATH: OnceLock<PathBuf> = OnceLock::new();
    METADATA_PATH.get_or_init(|| dst_dir().join("metadata.js"))
}

#[macro_export]
macro_rules! read_asset {
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
                include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/asset/", $fname)),
            )
        }
        .map_err(Into::into);
        result
    }};
}

#[derive(PartialEq, Eq, Hash)]
pub struct SrcPath(PathBuf);

impl SrcPath {
    pub fn new(path: PathBuf) -> Result<Self> {
        if !path.starts_with(src_dir()) {
            bail!("path does not starts with src_dir");
        }
        Ok(Self(path))
    }

    pub fn get_ref(&self) -> &PathBuf {
        &self.0
    }

    pub fn is_md(&self) -> bool {
        self.0.extension().is_some_and(|e| e == "md")
    }

    pub fn to_dst_path(&self) -> DstPath {
        let mut rel_path = self.0.strip_prefix(src_dir()).unwrap().to_owned();

        if matches!(rel_path.extension().and_then(|e| e.to_str()), Some("md")) {
            rel_path = rel_path.with_extension("html");
        }

        DstPath(dst_dir().join(rel_path))
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct DstPath(PathBuf);

impl DstPath {
    pub fn path_to_dst(&self) -> PathBuf {
        pathdiff::diff_paths(dst_dir(), self.0.parent().unwrap()).unwrap()
    }

    pub fn get_ref(&self) -> &PathBuf {
        &self.0
    }

    pub fn rel_path(&self) -> &Path {
        self.0.strip_prefix(dst_dir()).unwrap()
    }
}
