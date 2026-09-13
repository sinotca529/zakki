use anyhow::Context;
use serde::Deserialize;
use std::path::Path;

const fn default_search_fp() -> f64 {
    0.0001f64
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    /// サイトの名前
    pub site_name: String,

    /// サイトの公開先 URL
    pub publish_url: Option<String>,

    /// ページの暗号化に使用するパスワード
    pub password: Option<String>,

    /// ページの下部に表示する内容 (HTML 形式)
    pub footer: Option<String>,

    /// サイト内検索の偽陽性率
    /// INFO: デフォルト値の即値による指定は現状できない。
    /// see: <https://github.com/serde-rs/serde/issues/368>
    #[serde(default = "default_search_fp")]
    pub search_fp: f64,

    /// 追加の JS ファイル
    /// インターネット上へのリンクも扱えるよう、 PathBuf ではなく String で扱う
    #[serde(default)]
    pub js_list: Vec<String>,

    /// 追加の CSS ファイル
    /// インターネット上へのリンクも扱えるよう、 PathBuf ではなく String で扱う
    #[serde(default)]
    pub css_list: Vec<String>,
}

impl ProjectConfig {
    pub fn load(config_path: &Path) -> anyhow::Result<Self> {
        let cfg = std::fs::read(config_path)
            .with_context(|| format!("{} を読めません", config_path.display()))?;

        let cfg = std::str::from_utf8(&cfg)
            .with_context(|| format!("{} が utf-8 ではありません", config_path.display()))?;

        toml::from_str(cfg).with_context(|| format!("{} の中身が不正です", config_path.display()))
    }
}
