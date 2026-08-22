use crate::command::build::renderer::context::Context;
use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};

/// コードブロックの info string に `:タイトル` が含まれている場合、
/// `<figure class="code-figure">` と `<figcaption>` で囲む。
///
/// 例: ` ```python:ソートアルゴリズム ` → figcaption 付きの figure に変換
pub fn add_code_caption<'a>(events: &mut Vec<Event<'a>>, _: &mut Context) -> anyhow::Result<()> {
    let mut out = Vec::with_capacity(events.len());
    let mut in_captioned = false;

    for e in events.drain(..) {
        match e {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                match info.split_once(':') {
                    Some((lang, title)) if !title.trim().is_empty() => {
                        let title = title
                            .trim()
                            .replace('&', "&amp;")
                            .replace('<', "&lt;")
                            .replace('>', "&gt;");
                        in_captioned = true;
                        out.push(Event::Html(
                            format!(
                                r#"<figure class="code-figure"><figcaption>{title}</figcaption>"#
                            )
                            .into(),
                        ));
                        out.push(Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(
                            lang.to_owned().into(),
                        ))));
                    }
                    _ => {
                        out.push(Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))));
                    }
                }
            }
            Event::End(TagEnd::CodeBlock) if in_captioned => {
                in_captioned = false;
                out.push(e);
                out.push(Event::Html("</figure>".into()));
            }
            _ => out.push(e),
        }
    }

    *events = out;
    Ok(())
}
