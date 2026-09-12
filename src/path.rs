use crate::util::PathExt;
use anyhow::{Result, bail};
use paste::paste;
use std::path::{Path, PathBuf};

macro_rules! getter {
    ($field:ident, $type:ty) => {
        paste! {
            pub fn [<$field>](&self) -> $type {
                &self.$field
            }
        }
    };
}

#[derive(Clone)]
pub struct ProjectPaths {
    root_dir: PathBuf,
    src_dir: PathBuf,
    src_public_dir: PathBuf,
    src_private_dir: PathBuf,
    src_draft_dir: PathBuf,
    build_dir: PathBuf,
    config_path: PathBuf,
}

impl ProjectPaths {
    getter!(root_dir, &PathBuf);
    getter!(src_dir, &PathBuf);
    getter!(src_public_dir, &PathBuf);
    getter!(src_private_dir, &PathBuf);
    getter!(src_draft_dir, &PathBuf);
    getter!(build_dir, &PathBuf);
    getter!(config_path, &PathBuf);

    fn new(root_dir: PathBuf) -> Self {
        let src_dir = root_dir.join("src");

        Self {
            src_public_dir: src_dir.join("public"),
            src_private_dir: src_dir.join("private"),
            src_draft_dir: src_dir.join("draft"),
            src_dir,
            build_dir: root_dir.join("build"),
            config_path: root_dir.join("zakki.toml"),
            root_dir,
        }
    }

    /// 設定ファイルのあるディレクトリをルートとしたパス情報を返します。
    /// 祖先方向に設定ファイルを探索します。
    pub fn find() -> Result<Self> {
        let pwd = std::env::current_dir()?;
        let mut dir: Option<&Path> = Some(pwd.as_ref());

        while let Some(d) = dir {
            let is_zakki_root = d.has_file("zakki.toml")?;
            if is_zakki_root {
                return Ok(Self::new(d.to_owned()));
            }
            dir = d.parent();
        }

        bail!("このディレクトリは zakki 用のものではありません");
    }

    /// CWD を root としたパス情報を返します (設定ファイルの探索・作成は実施しません)
    pub fn at_current_dir() -> anyhow::Result<Self> {
        Ok(Self::new(std::env::current_dir()?))
    }

    pub fn build_path_of(&self, src_path: impl AsRef<Path>) -> Result<PathBuf> {
        let src_path = src_path.as_ref();
        let rel = src_path.strip_prefix(self.src_dir()).unwrap();

        if rel.extension_is("md") {
            Ok(self.build_dir.join(rel.with_extension("html")))
        } else {
            Ok(self.build_dir.join(rel))
        }
    }

    pub fn is_draft(&self, src_path: &Path) -> bool {
        src_path.starts_with(self.src_draft_dir())
    }

    pub fn is_private(&self, src_path: &Path) -> bool {
        src_path.starts_with(self.src_private_dir())
    }
}
