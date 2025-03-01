use crate::command::build::renderer::context::Context;
use anyhow::Context as _;
use pulldown_cmark::Event;

pub fn convert_math_pass<'a>(
    mut input: Vec<Event<'a>>,
    ctxt: &mut Context,
) -> anyhow::Result<Vec<Event<'a>>> {
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
    for e in &mut input {
        match e {
            Event::InlineMath(latex) => {
                let math = katex::render_with_opts(latex, &opts_inline)
                    .with_context(|| format!("Failed to render inline math: {}", latex))?;
                *e = Event::InlineHtml(math.into());
                math_used = true;
            }
            Event::DisplayMath(latex) => {
                let math = katex::render_with_opts(latex, &opts_display)
                    .with_context(|| format!("Failed to render display math: {}", latex))?;
                *e = Event::InlineHtml(math.into());
                math_used = true;
            }
            _ => {}
        }
    }

    if math_used {
        ctxt.push_css_path("katex/katex.min.css");
    }

    Ok(input)
}
