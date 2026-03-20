mod build;
mod clean;
mod init;
mod new;

use anyhow::Result;
use clap::Subcommand;

#[derive(PartialEq, Eq, Debug, Subcommand)]
pub enum Command {
    Init,
    Build {
        #[arg(short = 'd', long)]
        render_draft: bool,
    },
    Clean,
    New {
        /// 作成するファイルのパス (src/ からの相対パス, 拡張子省略可)
        path: String,
    },
}

impl Command {
    pub fn exec(&self) -> Result<()> {
        match &self {
            Self::Init => init::init(),
            Self::Build { render_draft } => build::build(*render_draft),
            Self::Clean => clean::clean(),
            Self::New { path } => new::new(path),
        }
    }
}
