use std::path::{Path, PathBuf};

use crate::util::PathExt as _;
use anyhow::bail;
use serde::Deserialize;

const fn default_search_fp() -> f64 {
    0.0001f64
}

#[derive(Deserialize)]
pub struct FileConfig {
    /// サイトの名前
    site_name: String,

    /// ページの暗号化に使うパスワード
    #[serde(default)]
    password: Option<String>,

    /// ページの下部に表示する内容 (HTML形式)
    #[serde(default)]
    footer: Option<String>,

    /// サイト内検索の偽陽性率
    /// INFO: デフォルト値の即値による指定は現状できない。
    /// see: <https://github.com/serde-rs/serde/issues/368>
    #[serde(default = "default_search_fp")]
    search_fp: f64,

    /// 追加の JS ファイル
    /// インターネット上へのリンクも扱えるよう、 PathBuf ではなく String で扱う
    #[serde(default)]
    js_list: Vec<String>,

    /// 追加の CSS ファイル
    /// インターネット上へのリンクも扱えるよう、 PathBuf ではなく String で扱う
    #[serde(default)]
    css_list: Vec<String>,
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
    /// サイト内検索の偽陽性率
    search_fp: f64,
    /// 追加の JS ファイル
    /// インターネット上へのリンクも扱えるよう、 PathBuf ではなく String で扱う
    js_list: Vec<String>,
    /// 追加の CSS ファイル
    /// インターネット上へのリンクも扱えるよう、 PathBuf ではなく String で扱う
    css_list: Vec<String>,
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
            search_fp: file_config.search_fp,
            js_list: file_config.js_list,
            css_list: file_config.css_list,
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

    pub fn search_fp(&self) -> f64 {
        self.search_fp
    }

    /// ソースファイルの出力先パスを返します。
    pub fn dst_path_of(&self, src_path: impl AsRef<Path>) -> PathBuf {
        let src_path = src_path.as_ref();
        let rel = src_path.strip_prefix(self.src_dir()).unwrap();

        if rel.extension_is("md") {
            self.dst_dir().join(rel.with_extension("html"))
        } else {
            self.dst_dir().join(rel)
        }
    }

    pub fn js_list(&self) -> &Vec<String> {
        &self.js_list
    }

    pub fn css_list(&self) -> &Vec<String> {
        &self.css_list
    }
}
