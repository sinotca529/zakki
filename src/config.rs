use anyhow::Context;
use serde::Deserialize;
use std::path::{Path, PathBuf};

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

    /// コードブロックに使うフォント
    /// 指定がなければ、同梱のフォントだけを使う
    pub code_font: Option<CodeFontConfig>,
}

/// コードブロックに使うフォントの設定
///
/// フォント自体は zakki に同梱しません。日本語フォントはサブセット前の全字形が要るため、
/// 実行ファイルが十数 MB ふくらむためです。利用者が手元のフォントを指す形にしています。
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodeFontConfig {
    /// フォントファイルのパス
    /// TrueType アウトライン (`glyf`) を持つものに限る
    pub path: PathBuf,

    /// 1 つのファイルに複数のフォントが入っている場合 (`.ttc`) に、どれを使うか
    /// フォントが名乗っている名前で指定する。指定を誤ると、入っている名前を挙げて止まる
    pub name: Option<String>,

    /// 出力に添えるライセンス文書のパス
    /// ライセンスはフォントごとに違うため、置き場所を zakki からは決められない
    pub license: Option<PathBuf>,
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
