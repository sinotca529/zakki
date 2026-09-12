use anyhow::Context;
use serde::Deserialize;
use std::path::Path;

const fn default_search_fp() -> f64 {
    0.0001f64
}

#[derive(Deserialize)]
pub struct FileConfig {
    /// サイトの名前
    site_name: String,

    // サイトの公開先 URL
    publish_url: Option<String>,

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
    pub fn load(config_path: &Path) -> anyhow::Result<Self> {
        let cfg = std::fs::read(config_path)
            .with_context(|| format!("{} を読めません", config_path.display()))?;

        let cfg = std::str::from_utf8(&cfg)
            .with_context(|| format!("{} が utf-8 ではありません", config_path.display()))?;

        toml::from_str(cfg).with_context(|| format!("{} の中身が不正です", config_path.display()))
    }
}

pub struct Config {
    /// サイト名
    site_name: String,
    /// 公開先 URL
    publish_url: Option<String>,
    /// 下書き記事を html に変換するかどうか
    render_draft: bool,
    /// 記事の暗号化に使うデフォルトのパスワード
    password: Option<String>,
    /// フッタの内容
    footer: String,
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
    pub fn new(file_config: FileConfig, render_draft: bool) -> Self {
        Self {
            footer: file_config.footer.unwrap_or(format!(
                "&copy; {}. All rights reserved.",
                file_config.site_name
            )),
            site_name: file_config.site_name,
            publish_url: file_config.publish_url,
            render_draft,
            password: file_config.password,
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

    pub fn publish_url(&self) -> Option<&String> {
        self.publish_url.as_ref()
    }

    pub fn footer(&self) -> &str {
        &self.footer
    }

    pub fn search_fp(&self) -> f64 {
        self.search_fp
    }

    pub fn js_list(&self) -> &Vec<String> {
        &self.js_list
    }

    pub fn css_list(&self) -> &Vec<String> {
        &self.css_list
    }
}
