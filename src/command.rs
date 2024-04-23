mod build;
mod clean;
mod init;

use anyhow::Result;
use clap::Subcommand;
use std::path::{Path, PathBuf};

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

fn src_dir() -> Result<PathBuf> {
    Ok(std::env::current_dir()?.join("src"))
}

fn build_dir() -> Result<PathBuf> {
    Ok(std::env::current_dir()?.join("build"))
}

fn css_path() -> Result<PathBuf> {
    Ok(build_dir()?.join("style.css"))
}

fn relative_path_to_css(html_path: impl AsRef<Path>) -> Result<PathBuf> {
    Ok(pathdiff::diff_paths(css_path()?, html_path.as_ref().parent().unwrap()).unwrap())
}

fn html_path(md_path: impl AsRef<Path>) -> Result<PathBuf> {
    fn inner(md_path: &Path) -> Result<PathBuf> {
        let rel_path = md_path.strip_prefix(src_dir()?)?;
        let html_path = build_dir()?.join(rel_path).with_extension("html");
        Ok(html_path)
    }
    inner(md_path.as_ref())
}
