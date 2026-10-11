use crate::{copy_asset, include_asset, util};
use anyhow::{Context, anyhow};
use std::path::Path;

/// 書き写すだけの style.css。末尾に zakki が組み立てた分を足します。
const STYLE_CSS: &str = include_asset!("style.css");

/// 書き出す style.css の先頭に置く断り書き。
///
/// 足す分だけでなく、ファイル全体がビルドのたびに書き直されます。
const HEADER: &str =
    "/* zakki がビルドのたびに書き出します。手を入れても次のビルドで消えます。 */\n";

/// style.css を書き出します。
///
/// `generated` は zakki が組み立てた分で、末尾に足します。
/// 後ろに置くのは、style.css が定義している変数を上書きできるようにするためです。
pub fn write_style_css(build_dir: &Path, generated: &str) -> anyhow::Result<()> {
    let css = match generated.is_empty() {
        true => format!("{HEADER}{STYLE_CSS}"),
        false => format!("{HEADER}{STYLE_CSS}\n{generated}"),
    };
    util::write_file(build_dir.join("style.css"), css).map_err(Into::into)
}

pub fn copy_assets(build_dir: &Path) -> anyhow::Result<()> {
    copy_asset!("script.js", build_dir)?;

    copy_asset!("katex/LICENSE", build_dir)?;
    copy_asset!("katex/katex.min.css", build_dir)?;

    macro_rules! copy_katex_fonts {
            ($($font_name:literal),* $(,)?) => {
                $(
                    copy_asset!(concat!("katex/fonts/", $font_name), build_dir)?;
                )*
            }
        }
    copy_katex_fonts!(
        "KaTeX_AMS-Regular.woff2",
        "KaTeX_Caligraphic-Bold.woff2",
        "KaTeX_Caligraphic-Regular.woff2",
        "KaTeX_Fraktur-Bold.woff2",
        "KaTeX_Fraktur-Regular.woff2",
        "KaTeX_Main-BoldItalic.woff2",
        "KaTeX_Main-Bold.woff2",
        "KaTeX_Main-Italic.woff2",
        "KaTeX_Main-Regular.woff2",
        "KaTeX_Math-BoldItalic.woff2",
        "KaTeX_Math-Italic.woff2",
        "KaTeX_SansSerif-Bold.woff2",
        "KaTeX_SansSerif-Italic.woff2",
        "KaTeX_SansSerif-Regular.woff2",
        "KaTeX_Script-Regular.woff2",
        "KaTeX_Size1-Regular.woff2",
        "KaTeX_Size2-Regular.woff2",
        "KaTeX_Size3-Regular.woff2",
        "KaTeX_Size4-Regular.woff2",
        "KaTeX_Typewriter-Regular.woff2",
    );

    Ok(())
}
