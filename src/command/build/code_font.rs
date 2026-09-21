use crate::config::CodeFontConfig;
use crate::include_asset;
use crate::util;
use anyhow::{Context as _, Result, bail};
use fontcull_klippa::{Plan, SubsetFlags, subset_font};
use fontcull_skrifa::raw::collections::IntSet;
use fontcull_skrifa::raw::types::NameId;
use fontcull_skrifa::raw::{FileRef, TableProvider as _};
use fontcull_skrifa::string::StringId;
use fontcull_skrifa::{FontRef, GlyphId, MetadataProvider as _, Tag};
use std::collections::BTreeSet;
use std::path::Path;

/// 生成した CSS の位置。サイト全体に読み込みます。
pub const CSS_PATH: &str = "font/zakki-code.css";

/// 生成したフォントの位置。CSS と同じディレクトリに置き、相対で参照します。
const WOFF2_NAME: &str = "zakki-code.woff2";

/// 生成したフォントを読み込む CSS。
///
/// `--code-font` は style.css が定義しています。この CSS は後から読み込まれるので、
/// 同じ名前を書き直せば先頭に差し込めます。
/// フォントに付ける名前を元フォントから取らないのは、改変したものに元の名前を
/// 使わせない条件 (SIL Open Font License でいう Reserved Font Name) を踏まないためです。
const CSS: &str = include_asset!("code-font.css");

/// 等幅で描く文字が記事に 1 つもないときに出す CSS。
///
/// 参照だけ残して woff2 を置かないと、ブラウザが 404 を引きます。
const NO_FONT_CSS: &str = include_asset!("code-font-none.css");

/// brotli の圧縮の強さ。
///
/// 11 は 3,415 字で 1.1 秒かかるのに対し、9 は 0.1 秒で済みます。
/// その差でサイズは 3% しか変わらないため、9 を選んでいます。
const BROTLI_QUALITY: u8 = 9;

/// `OS/2` の `fsType` のうち、許可なく埋め込めないことを表すビット。
const FS_TYPE_RESTRICTED: u16 = 0x0002;

/// `OS/2` の `fsType` のうち、サブセットを禁じることを表すビット。
const FS_TYPE_NO_SUBSETTING: u16 = 0x0100;

/// サブセットしたフォントと、それを読み込む CSS を出力します。
/// ライセンス文書の指定があれば、あわせてコピーします。
pub fn output(cfg: &CodeFontConfig, chars: &BTreeSet<char>, build_dir: &Path) -> Result<()> {
    // 残す文字がなければ、フォントを読む必要もありません。CSS だけは出します。
    // 記事の HTML には参照が入っているため、置かないとブラウザが 404 を引きます。
    if chars.is_empty() {
        return util::write_file(build_dir.join(CSS_PATH), NO_FONT_CSS).map_err(Into::into);
    }

    let bytes =
        std::fs::read(&cfg.path).with_context(|| format!("{} を読めません", cfg.path.display()))?;

    let index = select_index(&bytes, cfg)?;
    let font = FontRef::from_index(&bytes, index)
        .with_context(|| format!("{} をフォントとして読めません", cfg.path.display()))?;

    check_permissions(&font, cfg)?;
    check_outlines(&font, cfg)?;
    check_coverage(&font, cfg, chars)?;

    let sfnt = subset(&font, chars)
        .with_context(|| format!("{} のサブセットに失敗しました", cfg.path.display()))?;

    // 既定では 1 スレッドで圧縮するため、同じ入力からは同じバイト列が出ます。
    let woff2 = ttf2woff2::encode(&sfnt, BROTLI_QUALITY.into())
        .with_context(|| format!("{} を woff2 にできません", cfg.path.display()))?;

    let font_dir = build_dir.join("font");
    util::write_file(font_dir.join(WOFF2_NAME), woff2)?;
    util::write_file(build_dir.join(CSS_PATH), CSS)?;

    if let Some(license) = &cfg.license {
        let name = license
            .file_name()
            .with_context(|| format!("{} はファイルを指していません", license.display()))?;
        util::copy_file(license, font_dir.join(name))
            .with_context(|| format!("{} をコピーできません", license.display()))?;
    }

    Ok(())
}

/// 使うフォントがファイルの何番目かを決めます。
///
/// 1 つのファイルに複数のフォントが入っていることがあります (`.ttc`)。
/// 番号を人が知る手立てはないので、フォントが名乗っている名前で選びます。
fn select_index(bytes: &[u8], cfg: &CodeFontConfig) -> Result<u32> {
    let count = match FileRef::new(bytes) {
        Ok(FileRef::Collection(collection)) => collection.len(),
        _ => 1,
    };

    let Some(name) = cfg.name.as_deref() else {
        if count > 1 {
            bail!(
                "{} には {count} 個のフォントが入っています。name でどれを使うか指定してください\n{}",
                cfg.path.display(),
                font_name_list(bytes, count)
            );
        }
        return Ok(0);
    };

    (0..count)
        .find(|i| font_names(bytes, *i).iter().any(|n| n == name))
        .with_context(|| {
            format!(
                "{} に {name} は入っていません\n{}",
                cfg.path.display(),
                font_name_list(bytes, count)
            )
        })
}

/// 1 つのフォントが名乗っている名前を集めます。
///
/// `fc-list` はファミリ名を、フォントを選ぶ画面はフルネームを見せます。
/// 利用者がどちらを書き写しても通るよう、両方と突き合わせます。
fn font_names(bytes: &[u8], index: u32) -> Vec<String> {
    let Ok(font) = FontRef::from_index(bytes, index) else {
        return Vec::new();
    };

    [StringId::FAMILY_NAME, StringId::FULL_NAME]
        .into_iter()
        .flat_map(|id| font.localized_strings(id).map(|s| s.to_string()))
        .collect()
}

/// エラーに添える、ファイルに入っているフォント名の一覧を作ります。
fn font_name_list(bytes: &[u8], count: u32) -> String {
    (0..count)
        .filter_map(|i| FontRef::from_index(bytes, i).ok())
        .filter_map(|font| {
            font.localized_strings(StringId::FAMILY_NAME)
                .english_or_first()
        })
        .map(|name| format!("  {name}\n"))
        .collect()
}

/// フォントが埋め込みとサブセットを許しているかを確かめます。
///
/// `fsType` はフォント自身が申告している値なので、機械的に判断できます。
/// ライセンス文書の側の条件までは分からないため、そちらは利用者に委ねます。
fn check_permissions(font: &FontRef, cfg: &CodeFontConfig) -> Result<()> {
    let fs_type = font.os2().map(|os2| os2.fs_type()).unwrap_or(0);
    let path = cfg.path.display();

    if fs_type & FS_TYPE_RESTRICTED != 0 {
        bail!("{path} は埋め込みを許可していません (fsType = {fs_type:#06x})");
    }

    if fs_type & FS_TYPE_NO_SUBSETTING != 0 {
        bail!("{path} はサブセットを許可していません (fsType = {fs_type:#06x})");
    }

    Ok(())
}

/// 字形が `glyf` で入っているかを確かめます。
///
/// CFF アウトラインのフォントを渡しても、`CFF ` 表がそのまま通るだけで小さくなりません。
/// 数 MB のフォントがそのまま出力に乗るので、ここで止めます。
fn check_outlines(font: &FontRef, cfg: &CodeFontConfig) -> Result<()> {
    if font.glyf().is_err() {
        bail!(
            "{} は CFF アウトラインのフォントです。TrueType アウトラインのフォント (.ttf) を指定してください",
            cfg.path.display()
        );
    }

    Ok(())
}

/// 記事で使われた文字をフォントが持っているかを確かめます。
///
/// 1 つも持っていないと、字形が 1 つも残らないサブセットができます。
/// その woff2 はブラウザの検査 (OTS) で `glyf: zero-length table` として弾かれます。
fn check_coverage(font: &FontRef, cfg: &CodeFontConfig, chars: &BTreeSet<char>) -> Result<()> {
    let charmap = font.charmap();

    if !chars.iter().any(|c| charmap.map(*c).is_some()) {
        bail!(
            "{} は、記事で等幅に使われた文字を 1 つも持っていません",
            cfg.path.display()
        );
    }

    Ok(())
}

fn subset(font: &FontRef, chars: &BTreeSet<char>) -> Result<Vec<u8>> {
    let mut unicodes = IntSet::empty();
    for c in chars {
        unicodes.insert(*c as u32);
    }

    // 縦組みの表は横組みでは使いません。加えて、klippa は `vmtx` を縮めず
    // `vhea` の件数も元のまま残すため、置いておくと字形の数と件数が食い違います。
    // その状態のフォントはブラウザの検査 (OTS) で弾かれ、読み込まれません。
    let mut drop_tables = IntSet::empty();
    drop_tables.insert(Tag::new(b"vhea"));
    drop_tables.insert(Tag::new(b"vmtx"));

    // 著作権表示 (0) とライセンス (13, 14) は残し、名前にあたるものは落とします。
    let mut name_ids = IntSet::empty();
    for id in [0u16, 13, 14] {
        name_ids.insert(NameId::new(id));
    }

    let plan = Plan::new(
        &IntSet::<GlyphId>::empty(),
        &unicodes,
        font,
        SubsetFlags::default(),
        &drop_tables,
        &IntSet::<Tag>::all(),
        &IntSet::<Tag>::all(),
        &name_ids,
        &IntSet::<u16>::all(),
    );

    subset_font(font, &plan).map_err(Into::into)
}

#[cfg(test)]
mod test {
    use super::{CSS, WOFF2_NAME};

    /// CSS が読むファイル名と、実際に書き出すファイル名は同じである必要があります。
    #[test]
    fn css_points_at_the_generated_font() {
        assert!(CSS.contains(WOFF2_NAME));
    }

    /// CSS は @font-face の名前と `--code-font` の先頭を揃えている必要があります。
    /// 片方だけ変えると、指定しただけで使われないフォントができます。
    #[test]
    fn css_uses_the_same_family_name() {
        assert!(CSS.contains("font-family: \"zakki-code\";"));
        assert!(CSS.contains("--code-font: \"zakki-code\","));
    }
}
