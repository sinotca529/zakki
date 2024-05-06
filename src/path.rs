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

pub fn dst_css_path() -> &'static PathBuf {
    static CSS_DIR: OnceLock<PathBuf> = OnceLock::new();
    CSS_DIR.get_or_init(|| dst_dir().join("style.css"))
}

pub fn dst_metadata_path() -> &'static PathBuf {
    static METADATA_PATH: OnceLock<PathBuf> = OnceLock::new();
    METADATA_PATH.get_or_init(|| dst_dir().join("metadata.js"))
}

pub fn src_css_path() -> &'static PathBuf {
    static CSS_DIR: OnceLock<PathBuf> = OnceLock::new();
    CSS_DIR.get_or_init(|| src_dir().join("style.css"))
}

pub fn src_index_path() -> &'static PathBuf {
    static CSS_DIR: OnceLock<PathBuf> = OnceLock::new();
    CSS_DIR.get_or_init(|| src_dir().join("index.html"))
}

#[macro_export]
macro_rules! default_css_path {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/asset/style.css")
    };
}

#[macro_export]
macro_rules! default_index_path {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/asset/index.html")
    };
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
    pub fn path_to_css(&self) -> PathBuf {
        pathdiff::diff_paths(dst_css_path(), self.0.parent().unwrap()).unwrap()
    }

    pub fn get_ref(&self) -> &PathBuf {
        &self.0
    }

    pub fn rel_path(&self) -> &Path {
        self.0.strip_prefix(dst_dir()).unwrap()
    }
}
