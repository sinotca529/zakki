mod build;
mod clean;
mod init;

use anyhow::Result;
use clap::Subcommand;

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
        match &self {
            Self::Init => init::init(),
            Self::Build { render_draft } => build::build(*render_draft),
            Self::Clean => clean::clean(),
        }
    }
}
