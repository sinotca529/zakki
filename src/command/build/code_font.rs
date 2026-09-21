use crate::config::CodeFontConfig;
use crate::util;
use anyhow::{Context as _, Result, bail};
use fontcull_klippa::{Plan, SubsetFlags, subset_font};
use fontcull_skrifa::raw::TableProvider as _;
use fontcull_skrifa::raw::collections::IntSet;
use fontcull_skrifa::raw::types::NameId;
use fontcull_skrifa::{FontRef, GlyphId, Tag};
use std::collections::BTreeSet;
use std::path::Path;

/// 出力するフォントに付ける名前。
///
/// 元フォントの名前は残しません。改変したものに元の名前を使わせない条件
/// (SIL Open Font License でいう Reserved Font Name) が多くのフォントに付いているためです。
const FAMILY: &str = "zakki-code";

/// 生成した CSS の位置。サイト全体に読み込みます。
pub const CSS_PATH: &str = "font/zakki-code.css";

/// 生成したフォントの位置。CSS と同じディレクトリに置き、相対で参照します。
const WOFF2_NAME: &str = "zakki-code.woff2";

/// brotli の圧縮の強さ。
///
/// 11 は 3,415 字で 1.1 秒かかるのに対し、9 は 0.1 秒で済みます。
/// その差でサイズは 3% しか変わらないため、9 を選んでいます。
const BROTLI_QUALITY: u8 = 9;

/// サブセットに必ず含める文字。
///
/// タグやキャプションも `--code-font` で描かれます。ここに出る文字まで拾い切るのは
/// 難しいので、少なくとも ASCII は揃えておきます。
const ALWAYS_INCLUDED: std::ops::RangeInclusive<char> = ' '..='~';

/// `OS/2` の `fsType` のうち、許可なく埋め込めないことを表すビット。
const FS_TYPE_RESTRICTED: u16 = 0x0002;

/// `OS/2` の `fsType` のうち、サブセットを禁じることを表すビット。
const FS_TYPE_NO_SUBSETTING: u16 = 0x0100;

/// サブセットしたフォントと、それを読み込む CSS を出力します。
/// ライセンス文書の指定があれば、あわせてコピーします。
pub fn output(cfg: &CodeFontConfig, chars: &BTreeSet<char>, build_dir: &Path) -> Result<()> {
    let bytes =
        std::fs::read(&cfg.path).with_context(|| format!("{} を読めません", cfg.path.display()))?;

    let font = FontRef::from_index(&bytes, cfg.index)
        .with_context(|| format!("{} をフォントとして読めません", cfg.path.display()))?;

    check_permissions(&font, cfg)?;
    check_outlines(&font, cfg)?;

    let sfnt = subset(&font, chars)
        .with_context(|| format!("{} のサブセットに失敗しました", cfg.path.display()))?;

    // 既定では 1 スレッドで圧縮するため、同じ入力からは同じバイト列が出ます。
    let woff2 = ttf2woff2::encode(&sfnt, BROTLI_QUALITY.into())
        .with_context(|| format!("{} を woff2 にできません", cfg.path.display()))?;

    let font_dir = build_dir.join("font");
    util::write_file(font_dir.join(WOFF2_NAME), woff2)?;
    util::write_file(build_dir.join(CSS_PATH), css())?;

    if let Some(license) = &cfg.license {
        let name = license
            .file_name()
            .with_context(|| format!("{} はファイルを指していません", license.display()))?;
        util::copy_file(license, font_dir.join(name))
            .with_context(|| format!("{} をコピーできません", license.display()))?;
    }

    Ok(())
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

fn subset(font: &FontRef, chars: &BTreeSet<char>) -> Result<Vec<u8>> {
    let mut unicodes = IntSet::empty();
    for c in chars.iter().copied().chain(ALWAYS_INCLUDED) {
        unicodes.insert(c as u32);
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

/// 生成したフォントを読み込む CSS を作ります。
///
/// `--code-font` は style.css が定義しています。この CSS は後から読み込まれるので、
/// 同じ名前を書き直せば先頭に差し込めます。
fn css() -> String {
    format!(
        "\
/* zakki が生成したファイルです。編集しても次のビルドで上書きされます。 */
@font-face {{
  font-family: \"{FAMILY}\";
  font-style: normal;
  font-weight: normal;
  src: url({WOFF2_NAME}) format(\"woff2\");
}}

:root {{
  --code-font: \"{FAMILY}\", var(--code-font-fallback);
}}
"
    )
}

#[cfg(test)]
mod test {
    use super::css;

    /// CSS は @font-face の名前と `--code-font` の先頭を揃えている必要があります。
    /// 片方だけ変えると、指定しただけで使われないフォントができます。
    #[test]
    fn css_uses_the_same_family_name() {
        let css = css();
        assert!(css.contains("font-family: \"zakki-code\";"));
        assert!(css.contains("--code-font: \"zakki-code\", var(--code-font-fallback);"));
        assert!(css.contains("src: url(zakki-code.woff2) format(\"woff2\");"));
    }
}
