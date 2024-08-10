mod build;
mod clean;
mod init;

use std::path::Path;

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

/// Goto the root directory, which has the zakki.toml file.
pub fn goto_zakki_root() -> Result<()> {
    let pwd = std::env::current_dir()?;
    let mut dir: Option<&Path> = Some(pwd.as_ref());

    while let Some(d) = dir {
        let dir_contains_cfg = std::fs::read_dir(d)?
            .filter_map(|f| f.ok())
            .map(|f| f.file_name())
            .any(|f| &f == "zakki.toml");

        if dir_contains_cfg {
            std::env::set_current_dir(d)?;
            return Ok(());
        } else {
            dir = d.parent();
        }
    }

    bail!("Failed to detect zakki root.");
}
