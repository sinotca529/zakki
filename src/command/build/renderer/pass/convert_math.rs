use crate::command::build::renderer::context::Context;
use anyhow::Context as _;
use comrak::nodes::{AstNode, NodeValue};

/// 数式を KaTeX でレンダリング済みの HTML に置き換えます。
pub fn convert_math<'a>(root: &'a AstNode<'a>, ctx: &mut Context) -> anyhow::Result<()> {
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

    for node in root.descendants() {
        // data_mut() を呼ぶ前に借用を落とすため、必要な値だけ取り出す
        let math = match &node.data().value {
            NodeValue::Math(m) => Some((m.display_math, m.literal.clone())),
            _ => None,
        };
        let Some((display, latex)) = math else {
            continue;
        };

        let opts = if display { &opts_display } else { &opts_inline };
        let html = katex::render_with_opts(&latex, opts)
            .with_context(|| format!("Failed to render math: {latex}"))?;

        node.data_mut().value = NodeValue::HtmlInline(html);
        math_used = true;
    }

    if math_used {
        ctx.push_css_path("katex/katex.min.css");
    }

    Ok(())
}
