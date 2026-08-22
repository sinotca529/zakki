mod build;
mod clean;
mod init;

use anyhow::Result;
use clap::Subcommand;

#[derive(PartialEq, Eq, Debug, Subcommand)]
pub enum Command {
    /// Initialize the current directory as a zakki project.
    Init,
    /// Build the document.
    Build {
        #[arg(short = 'd', long)]
        render_draft: bool,
    },
    /// Clean build directory.
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
