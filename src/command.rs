mod build;
mod clean;
mod init;

use anyhow::{bail, Result};
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum Command {
    Init,
    Build {
        #[arg(short = 'd', long)]
        render_draft: bool,
    },
    Clean,
}

impl Command {
    pub fn exec(&self) -> Result<()> {
        match &self {
            Self::Init => init::init(),
            Self::Build { render_draft } => build::build(*render_draft),
            Self::Clean => clean::clean(),
        }
    }
}

pub fn ensure_pwd_is_book_root_dir() -> Result<()> {
    let pwd = std::env::current_dir()?;
    let pwd_contains_cfg = std::fs::read_dir(pwd)?
        .filter_map(|f| f.ok())
        .map(|f| f.file_name())
        .any(|f| &f == "config.toml");

    if !pwd_contains_cfg {
        bail!("Current directory does not have config.toml")
    }

    Ok(())
}
