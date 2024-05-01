mod build;
mod clean;
mod init;

use anyhow::Result;
use clap::Subcommand;
use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

#[derive(Debug, Subcommand)]
pub enum Command {
    Init,
    Build,
    Clean,
}

impl Command {
    pub fn exec(&self) -> Result<()> {
        match &self {
            Self::Init => init::init(),
            Self::Build => build::build(),
            Self::Clean => clean::clean(),
        }
    }
}

fn src_dir() -> &'static PathBuf {
    static SRC_DIR: OnceLock<PathBuf> = OnceLock::new();
    SRC_DIR.get_or_init(|| {
        std::env::current_dir()
            .expect("Failed to get the current directry")
            .join("src")
    })
}

fn dst_dir() -> &'static PathBuf {
    static BUILD_DIR: OnceLock<PathBuf> = OnceLock::new();
    BUILD_DIR.get_or_init(|| {
        std::env::current_dir()
            .expect("Failed to get the current directry")
            .join("build")
    })
}

fn css_path() -> &'static PathBuf {
    static CSS_DIR: OnceLock<PathBuf> = OnceLock::new();
    CSS_DIR.get_or_init(|| dst_dir().join("style.css"))
}

fn relative_path_to_css(html_path: impl AsRef<Path>) -> Result<PathBuf> {
    Ok(pathdiff::diff_paths(css_path(), html_path.as_ref().parent().unwrap()).unwrap())
}

fn html_path(md_path: impl AsRef<Path>) -> Result<PathBuf> {
    fn inner(md_path: &Path) -> Result<PathBuf> {
        let rel_path = md_path.strip_prefix(src_dir())?;
        let html_path = dst_dir().join(rel_path).with_extension("html");
        Ok(html_path)
    }
    inner(md_path.as_ref())
}
