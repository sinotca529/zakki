use crate::copy_asset;
use anyhow::{Context, anyhow};
use std::path::Path;

pub fn copy_assets(build_dir: &Path) -> anyhow::Result<()> {
    copy_asset!("style.css", build_dir)?;
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

    copy_asset!("font/SourceCodePro/LICENSE.md", build_dir)?;
    copy_asset!(
        "font/SourceCodePro/SourceCodePro-Regular.otf.woff2",
        build_dir
    )?;

    Ok(())
}
