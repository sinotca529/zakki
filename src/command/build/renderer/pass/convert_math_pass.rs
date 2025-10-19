use crate::command::build::renderer::metadata::Metadata;
use anyhow::Context as _;
use pulldown_cmark::Event;

pub fn convert_math_pass<'a>(
    mut input: Vec<Event<'a>>,
    ctxt: &mut Metadata,
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
        let (latex, opts) = match e {
            Event::InlineMath(latex) => (latex, &opts_inline),
            Event::DisplayMath(latex) => (latex, &opts_display),
            _ => continue,
        };

        let math = katex::render_with_opts(latex, opts)
            .with_context(|| format!("Failed to render math: {}", latex))?;
        *e = Event::InlineHtml(math.into());
        math_used = true;
    }

    if math_used {
        ctxt.push_css_path("katex/katex.min.css");
    }

    Ok(input)
}
