use std::path::{Path, PathBuf};

use crate::util::PathExt as _;
use anyhow::bail;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FileConfig {
    site_name: String,
    #[serde(default)]
    password: Option<String>,
    #[serde(default)]
    footer: Option<String>,
}

impl FileConfig {
    pub fn load() -> anyhow::Result<Self> {
        let pwd = std::env::current_dir()?;
        let cfg = std::fs::read_dir(pwd)?
            .filter_map(|f| f.ok())
            .map(|f| f.file_name())
            .find(|f| f == "zakki.toml");

        let Some(cfg) = cfg else {
            bail!("zakki.toml is not found.");
        };

        let cfg = std::fs::read(cfg)?;
        let cfg = std::str::from_utf8(&cfg)?;

        toml::from_str(cfg).map_err(Into::into)
    }
}

pub struct Config {
    site_name: String,
    render_draft: bool,
    password: Option<String>,
    footer: String,
    /// Markdown が配置されているディレクトリ
    src_dir: PathBuf,
    /// HTML を出力するディレクトリ
    dst_dir: PathBuf,
}

impl Config {
    pub fn new(
        file_config: FileConfig,
        render_draft: bool,
        src_dir: PathBuf,
        dst_dir: PathBuf,
    ) -> Self {
        Self {
            footer: file_config.footer.unwrap_or(format!(
                "&copy; {}. All rights reserved.",
                &file_config.site_name
            )),
            site_name: file_config.site_name,
            render_draft,
            password: file_config.password,
            src_dir,
            dst_dir,
        }
    }

    pub fn render_draft(&self) -> bool {
        self.render_draft
    }

    pub fn site_name(&self) -> &str {
        &self.site_name
    }

    pub fn password(&self) -> Option<&String> {
        self.password.as_ref()
    }

    pub fn footer(&self) -> &str {
        &self.footer
    }

    pub fn src_dir(&self) -> &PathBuf {
        &self.src_dir
    }

    pub fn dst_dir(&self) -> &PathBuf {
        &self.dst_dir
    }

    /// ソースファイルの出力先のをパスを返します。
    pub fn dst_path_of(&self, src_path: impl AsRef<Path>) -> PathBuf {
        let src_path = src_path.as_ref();
        let mut rel = src_path.path_from(&self.src_dir()).unwrap();
        if rel.extension_is("md") {
            rel = rel.with_extension("html");
        }
        self.dst_dir().join(rel)
    }
}
