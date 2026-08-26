use super::raw_html;
use crate::command::build::renderer::context::Context;
use anyhow::Context as _;
use jotdown::{Container, Event};

/// 数式を KaTeX でレンダリング済みの HTML に置き換えます。
pub fn convert_math<'a>(events: &mut Vec<Event<'a>>, ctxt: &mut Context) -> anyhow::Result<()> {
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

    let mut out = Vec::with_capacity(events.len());
    let mut display = None;
    let mut latex = String::new();
    let mut math_used = false;

    for e in events.drain(..) {
        match e {
            Event::Start(Container::Math { display: d }, _) => {
                display = Some(d);
                latex.clear();
            }
            Event::Str(ref s) if display.is_some() => latex.push_str(s),
            Event::End(Container::Math { .. }) => {
                let opts = if display.take() == Some(true) {
                    &opts_display
                } else {
                    &opts_inline
                };
                let math = katex::render_with_opts(&latex, opts)
                    .with_context(|| format!("Failed to render math: {}", latex))?;
                out.extend(raw_html(math));
                math_used = true;
            }
            _ => out.push(e),
        }
    }

    *events = out;

    if math_used {
        ctxt.push_css_path("katex/katex.min.css");
    }

    Ok(())
}
