use super::PassAssets;
use anyhow::Context as _;
use pulldown_cmark::Event;

/// 数式を KaTeX でレンダリング済みの HTML に置き換えます。
pub fn convert_math(events: &mut [Event<'_>], pa: &mut PassAssets) -> anyhow::Result<()> {
    let opts_display = katex::Opts::builder()
        .output_type(katex::opts::OutputType::Html)
        .display_mode(true)
        .build()
        .unwrap();
    let opts_inline = katex::Opts::builder()
        .output_type(katex::opts::OutputType::Html)
        .display_mode(false)
        .build()
        .unwrap();

    let mut math_used = false;

    for e in events.iter_mut() {
        let (latex, opts) = match e {
            Event::InlineMath(latex) => (latex, &opts_inline),
            Event::DisplayMath(latex) => (latex, &opts_display),
            _ => continue,
        };

        let html = katex::render_with_opts(latex, opts)
            .with_context(|| format!("Failed to render math: {latex}"))?;

        *e = Event::InlineHtml(html.into());
        math_used = true;
    }

    if math_used {
        pa.css_paths.push("katex/katex.min.css".to_string());
    }

    Ok(())
}
