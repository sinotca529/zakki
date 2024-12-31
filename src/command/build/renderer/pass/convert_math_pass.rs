use crate::command::build::renderer::context::Context;
use pulldown_cmark::Event;

pub fn convert_math_pass(events: &mut Vec<Event>, ctxt: &mut Context) -> anyhow::Result<()> {
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
    for e in events {
        match e {
            Event::InlineMath(latex) => {
                let math = katex::render_with_opts(latex, &opts_inline).unwrap();
                *e = Event::InlineHtml(math.into());
                math_used = true;
            }
            Event::DisplayMath(latex) => {
                let math = katex::render_with_opts(latex, &opts_display).unwrap();
                *e = Event::InlineHtml(math.into());
                math_used = true;
            }
            _ => {}
        }
    }

    if math_used {
        ctxt.push_css_path("katex/katex.min.css");
    }

    Ok(())
}
