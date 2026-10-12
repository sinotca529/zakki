use crate::command::build::box_drawing;
use crate::config::CodeFontConfig;
use crate::include_asset;
use crate::util;
use anyhow::{Context as _, Result, anyhow, bail};
use fontcull_klippa::{Plan, SubsetFlags, subset_font};
use fontcull_skrifa::charmap::Charmap;
use fontcull_skrifa::instance::{LocationRef, Size};
use fontcull_skrifa::metrics::GlyphMetrics;
use fontcull_skrifa::raw::collections::IntSet;
use fontcull_skrifa::raw::types::NameId;
use fontcull_skrifa::raw::{FileRef, TableProvider as _};
use fontcull_skrifa::string::StringId;
use fontcull_skrifa::{FontRef, GlyphId, MetadataProvider as _, Tag};
use fontcull_write_fonts::tables::cmap::Cmap;
use fontcull_write_fonts::{FontBuilder, dump_table};
use std::collections::BTreeSet;
use std::path::Path;

/// 生成したフォントを置くディレクトリ。ライセンス文書も同じ場所に入れます。
const FONT_DIR: &str = "font";

/// 生成したフォントのファイル名。
const WOFF2_NAME: &str = "zakki-code.woff2";

/// 生成したフォントを読み込む CSS。style.css の末尾に足します。
///
/// `--code-font` は style.css が定義しています。
/// 後ろに足すので、同じ名前を書き直せば先頭に差し込めます。
/// フォントに付ける名前は zakki が決めます。
/// 改変したものに元の名前を使わせない条件を踏まないためです。
/// SIL Open Font License でいう Reserved Font Name が該当します。
const CSS: &str = include_asset!("code-font.css");

/// brotli の圧縮の強さ。
///
/// 11 は 3,415 字で 1.1 秒かかるのに対し、9 は 0.1 秒で済みます。
/// その差でサイズは 3% しか変わらないため、9 を選んでいます。
const BROTLI_QUALITY: u8 = 9;

/// セルの幅を測るのに使う文字。
///
/// CSS の `ch` がこの文字の advance なので、合わせておく必要があります。
const CELL_WIDTH_CHAR: char = '0';

/// 罫線素片に割り当てる字形の文字。
///
/// 等幅のフォントなら advance がセル 1 つ分で、字形も空です。
const BLANK_CHAR: char = ' ';

/// `OS/2` の `fsType` のうち、許可なく埋め込めないことを表すビット。
const FS_TYPE_RESTRICTED: u16 = 0x0002;

/// `OS/2` の `fsType` のうち、サブセットを禁じることを表すビット。
const FS_TYPE_NO_SUBSETTING: u16 = 0x0100;

/// 設定で指定されたコードフォント。
///
/// 記事を描く前に読み、列を数えるのとサブセットを作るのに使い回します。
pub struct CodeFont<'a> {
    cfg: &'a CodeFontConfig,
    /// 読んだファイルの中身。
    bytes: Vec<u8>,
    /// ファイルの何番目のフォントを使うか。コレクションでなければ 0 です。
    font_index: u32,
}

impl<'a> CodeFont<'a> {
    /// フォントを読みます。コレクションであれば、設定の名前と一致するものを選びます。
    pub fn read(cfg: &'a CodeFontConfig) -> Result<Self> {
        let bytes = std::fs::read(&cfg.path)
            .with_context(|| format!("{} を読めません", cfg.path.display()))?;
        let font_index = select_font_index(&bytes, cfg)?;

        Ok(Self {
            cfg,
            bytes,
            font_index,
        })
    }

    fn font(&self) -> Result<FontRef<'_>> {
        FontRef::from_index(&self.bytes, self.font_index)
            .with_context(|| format!("{} をフォントとして読めません", self.cfg.path.display()))
    }

    /// 列を数えるための送り幅を取り出します。
    pub fn advances(&self) -> Result<Advances<'_>> {
        let font = self.font()?;
        let charmap = font.charmap();
        let metrics = font.glyph_metrics(Size::unscaled(), LocationRef::default());

        let cell = charmap
            .map(CELL_WIDTH_CHAR)
            .and_then(|gid| metrics.advance_width(gid))
            .map(|w| w as u32)
            .filter(|w| *w > 0)
            .with_context(|| {
                format!(
                    "{} に '{CELL_WIDTH_CHAR}' がありません",
                    self.cfg.path.display()
                )
            })?;

        let upem = u32::from(font.head()?.units_per_em());

        Ok(Advances {
            charmap,
            metrics,
            cell,
            upem,
        })
    }
}

/// 文字がセルをいくつ占めるかを、フォントの送り幅から答えます。
pub struct Advances<'a> {
    charmap: Charmap<'a>,
    metrics: GlyphMetrics<'a>,
    /// セル 1 つ分の送り幅。CSS の `ch` に合わせて `'0'` のものを使います。
    cell: u32,
    /// em あたりのフォント単位。送り幅を em に直すのに使います。
    upem: u32,
}

impl Advances<'_> {
    /// 文字が占めるセルの数を返します。
    ///
    /// フォントが持っていない文字と、送り幅がセルの整数倍でない文字は `None` です。
    /// 呼び出し側が East Asian Width で数えます。
    pub fn cells(&self, c: char) -> Option<u32> {
        let advance = self.advance(c)?;
        cells_of(advance, self.cell)
    }

    /// セル 1 つ分の幅を em で返します。
    pub fn cell_width_em(&self) -> f32 {
        self.cell as f32 / self.upem as f32
    }

    /// 送り幅がセルの整数倍でない文字を集めます。
    pub fn odd_chars(&self, chars: &BTreeSet<char>) -> BTreeSet<char> {
        chars
            .iter()
            .filter(|c| {
                self.advance(**c)
                    .is_some_and(|a| cells_of(a, self.cell).is_none())
            })
            .copied()
            .collect()
    }

    /// 描かれる字形の送り幅を返します。
    ///
    /// 罫線素片は `blank_box_drawing` が空白の字形に差し替えるので、
    /// 元の字形の送り幅ではなく空白の送り幅になります。
    fn advance(&self, c: char) -> Option<u32> {
        let drawn = match box_drawing::is_covered(c) {
            true => BLANK_CHAR,
            false => c,
        };
        let gid = self.charmap.map(drawn)?;
        self.metrics.advance_width(gid).map(|a| a as u32)
    }
}

/// 送り幅をセルの数に直します。整数倍でなければ `None` を返します。
///
/// 送り幅はフォント単位の整数なので、割り切れるかどうかで判定できます。
fn cells_of(advance: u32, cell: u32) -> Option<u32> {
    advance.is_multiple_of(cell).then(|| advance / cell)
}

/// サブセットしたフォントを出力し、それを読み込む CSS を返します。
/// ライセンス文書の指定があれば、あわせてコピーします。
///
/// 残す文字が 1 つもなければフォントを作らないので、空の CSS を返します。
pub fn output(
    code_font: &CodeFont,
    chars: &BTreeSet<char>,
    build_dir: &Path,
) -> Result<&'static str> {
    if chars.is_empty() {
        return Ok("");
    }

    let cfg = code_font.cfg;
    let font = code_font.font()?;

    check_permissions(&font, cfg)?;
    check_outlines(&font, cfg)?;
    check_coverage(&font, cfg, chars)?;

    // 罫線素片は線を SVG で描くので、字形を残しません。
    let (box_chars, mut chars): (BTreeSet<_>, BTreeSet<_>) =
        chars.iter().partition(|c| box_drawing::is_covered(**c));

    // '0' と空白は記事で使われていなくても残します。
    // CSS の `ch` が '0' の advance で、罫線素片を描く SVG の幅をこれで決めます。
    // 字形がないと、ブラウザは `ch` を 0.5em と見なし、図が文字とずれます。
    // 空白の字形は罫線素片の行き先として使います。
    chars.insert(CELL_WIDTH_CHAR);
    chars.insert(BLANK_CHAR);

    let sfnt = subset(&font, &chars)
        .with_context(|| format!("{} のサブセットに失敗しました", cfg.path.display()))?;

    let sfnt = blank_box_drawing(&sfnt, &box_chars)
        .with_context(|| format!("{} の罫線素片を空にできません", cfg.path.display()))?;

    // 既定では 1 スレッドで圧縮するため、同じ入力からは同じバイト列が出ます。
    let woff2 = ttf2woff2::encode(&sfnt, BROTLI_QUALITY.into())
        .with_context(|| format!("{} を woff2 にできません", cfg.path.display()))?;

    let font_dir = build_dir.join(FONT_DIR);
    util::write_file(font_dir.join(WOFF2_NAME), woff2)?;

    if let Some(license) = &cfg.license {
        let name = license
            .file_name()
            .with_context(|| format!("{} はファイルを指していません", license.display()))?;
        util::copy_file(license, font_dir.join(name))
            .with_context(|| format!("{} をコピーできません", license.display()))?;
    }

    Ok(CSS)
}

/// 罫線素片を空白の字形に向けます。
///
/// 線は `box_drawing` が作る SVG で描きます。
/// 二重に描かれないよう、cmap を書き直して罫線素片の字形を空白に向けます。
/// 字形そのものはサブセットの時点で落とします。
///
/// 日本語のフォントは罫線素片に全角の advance を与えていることがあり、
/// そのままでは図の格子が崩れます。
/// 等幅のフォントなら空白の advance はセル 1 つ分なので、向け先を変えるだけで幅も揃います。
fn blank_box_drawing(sfnt: &[u8], box_chars: &BTreeSet<char>) -> Result<Vec<u8>> {
    if box_chars.is_empty() {
        return Ok(sfnt.to_owned());
    }

    let font = FontRef::new(sfnt)?;
    let charmap = font.charmap();
    let blank = charmap.map(BLANK_CHAR).context("空白の字形がありません")?;

    let mappings = charmap
        .mappings()
        .filter_map(|(code, gid)| char::from_u32(code).map(|c| (c, gid)))
        .chain(box_chars.iter().map(|c| (*c, blank)));
    let cmap = Cmap::from_mappings(mappings).map_err(|e| anyhow!("cmap を作れません : {e}"))?;

    let mut builder = FontBuilder::new();
    builder.add_raw(Tag::new(b"cmap"), dump_table(&cmap)?);
    builder.copy_missing_tables(font);

    Ok(builder.build())
}

/// 使うフォントがファイルの何番目かを返します。
///
/// 1 つのファイルに複数のフォントが入っていることがあります (`.ttc`)。
/// 番号を人が知る手立てはないので、設定の `name` で選びます。
fn select_font_index(bytes: &[u8], cfg: &CodeFontConfig) -> Result<u32> {
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
/// 1 つも持っていないと、字形はすべてフォールバックのフォントから出ます。
/// 指定したフォントが使われないので、設定の誤りとして落とします。
///
/// SVG で描く文字は数えません。
/// `blank_box_drawing` が空白の字形に向けるので、もとのフォントが持っていなくても構いません。
fn check_coverage(font: &FontRef, cfg: &CodeFontConfig, chars: &BTreeSet<char>) -> Result<()> {
    let charmap = font.charmap();
    let mut drawn = chars
        .iter()
        .filter(|c| !box_drawing::is_covered(**c))
        .peekable();

    if drawn.peek().is_some() && !drawn.any(|c| charmap.map(*c).is_some()) {
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

    // 縦組みの表は横組みでは使いません。
    // 加えて、klippa は `vmtx` を縮めず `vhea` の件数も元のまま残すため、
    // 置いておくと字形の数と件数が食い違います。
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
    use super::{CSS, FONT_DIR, WOFF2_NAME, cells_of};

    /// 送り幅がセルの整数倍であれば、その倍数を返します。
    #[test]
    fn counts_cells_from_the_advance() {
        assert_eq!(cells_of(1024, 1024), Some(1));
        assert_eq!(cells_of(2048, 1024), Some(2));
        assert_eq!(cells_of(0, 1024), Some(0));
    }

    /// 等幅でない送り幅は数えません。East Asian Width に任せます。
    #[test]
    fn rejects_an_advance_that_is_not_a_multiple() {
        assert_eq!(cells_of(1000, 1024), None);
        assert_eq!(cells_of(1536, 1024), None);
    }

    /// CSS が読む URL と実際に書き出す位置は同じである必要があります。
    /// CSS は style.css の末尾に入るので、URL はサイトの根からの相対です。
    #[test]
    fn css_points_at_the_generated_font() {
        assert!(
            CSS.contains(&format!("url({FONT_DIR}/{WOFF2_NAME})")),
            "{CSS}"
        );
    }

    /// CSS は @font-face の名前と `--code-font` の先頭を揃えている必要があります。
    /// 片方だけ変えると、指定しただけで使われないフォントができます。
    #[test]
    fn css_uses_the_same_family_name() {
        assert!(CSS.contains("font-family: \"zakki-code\";"));
        assert!(CSS.contains("--code-font: \"zakki-code\","));
    }
}
