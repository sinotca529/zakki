mod build;
mod clean;
mod init;

use crate::util::PathExt;
use anyhow::{Result, anyhow, bail};
use clap::Subcommand;
use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

#[derive(PartialEq, Eq, Debug, Subcommand)]
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

pub fn zakki_root() -> Result<&'static PathBuf> {
    ZAKKI_ROOT_DIR.as_ref().map_err(|e| anyhow!(e.to_string()))
}
