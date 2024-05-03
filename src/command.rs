mod build;
mod clean;
mod init;

use anyhow::Result;
use clap::Subcommand;

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
