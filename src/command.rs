mod build;
mod clean;
mod init;

use anyhow::Result;
use clap::Subcommand;

use crate::path::ProjectPaths;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Zakki 向けのディレクトリを作成する
    Init,
    /// 文書をビルドする
    Build {
        #[arg(short = 'd', help = "下書きも html に変換する", long)]
        render_draft: bool,
    },
    /// ビルド結果を削除する
    Clean,
}

impl Command {
    pub fn exec(&self) -> Result<()> {
        let pwd = std::env::current_dir()?;
        match &self {
            Self::Init => init::init(pwd),
            Self::Build { render_draft } => build::build(&ProjectPaths::find(&pwd)?, *render_draft),
            Self::Clean => clean::clean(&ProjectPaths::find(&pwd)?),
        }
    }
}
