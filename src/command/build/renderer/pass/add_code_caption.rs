use super::{escape_html_text, raw_html};
use crate::command::build::renderer::context::Context;
use jotdown::{Container, Event};

/// コードブロックに `caption` 属性がある場合、
/// `<figure class="code-figure">` と `<figcaption>` で囲みます。
///
/// 例:
/// ```txt
/// {caption="main.rs"}
/// ```rust
/// fn main() {}
/// ```
/// ```
pub fn add_code_caption<'a>(
    events: &mut Vec<Event<'a>>,
    _ctxt: &mut Context,
) -> anyhow::Result<()> {
    let mut out = Vec::with_capacity(events.len());
    let mut in_captioned = false;

    for e in events.drain(..) {
        match e {
            Event::Start(Container::CodeBlock { language }, mut attrs) => {
                let caption = attrs.get_value("caption").map(|v| v.to_string());
                if let Some(caption) = caption {
                    // caption 属性が <pre> に出力されないよう取り除く
                    attrs.retain(|(k, _)| k.key() != Some("caption"));
                    in_captioned = true;
                    out.extend(raw_html(format!(
                        r#"<figure class="code-figure"><figcaption>{}</figcaption>"#,
                        escape_html_text(&caption)
                    )));
                }
                out.push(Event::Start(Container::CodeBlock { language }, attrs));
            }
            Event::End(Container::CodeBlock { language }) => {
                out.push(Event::End(Container::CodeBlock { language }));
                if in_captioned {
                    in_captioned = false;
                    out.extend(raw_html("</figure>"));
                }
            }
            _ => out.push(e),
        }
    }

    *events = out;
    Ok(())
}
