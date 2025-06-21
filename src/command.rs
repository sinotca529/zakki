mod build;
mod clean;
mod init;

use crate::util::PathExt;
use anyhow::{Result, bail};
use clap::Subcommand;
use std::path::Path;

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

/// Goto the root directory, which has the zakki.toml file.
pub fn goto_zakki_root() -> Result<()> {
    let pwd = std::env::current_dir()?;
    let mut dir: Option<&Path> = Some(pwd.as_ref());

    while let Some(d) = dir {
        let is_zakki_root = d.has_file("zakki.toml")?;
        if is_zakki_root {
            return std::env::set_current_dir(d).map_err(Into::into);
        }
        dir = d.parent();
    }

    bail!("Failed to detect zakki root.");
}
