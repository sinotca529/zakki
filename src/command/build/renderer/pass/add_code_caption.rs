use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};

/// コードブロックの info string に `:タイトル` が含まれている場合、
/// `<figure class="code-figure">` と `<figcaption>` で囲みます。
///
/// 例: ` ```python:ソートアルゴリズム ` → figcaption 付きの figure に変換
pub fn add_code_caption(events: &mut Vec<Event<'_>>) {
    let mut out = Vec::with_capacity(events.len());
    let mut captioned = false;

    for e in events.drain(..) {
        match e {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref info))) => {
                let caption = info
                    .split_once(':')
                    .map(|(lang, title)| (lang, title.trim()))
                    .filter(|(_, title)| !title.is_empty());

                let Some((lang, title)) = caption else {
                    out.push(e);
                    continue;
                };

                // キャプションは Text のまま置く。後続のパスが読めるようにするため
                out.push(Event::Html(
                    r#"<figure class="code-figure"><figcaption>"#.into(),
                ));
                out.push(Event::Text(title.to_owned().into()));
                out.push(Event::Html("</figcaption>".into()));

                // info string からタイトルを取り除き、言語名だけ残す
                let lang = CowStr::from(lang.to_owned());
                out.push(Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))));
                captioned = true;
            }
            Event::End(TagEnd::CodeBlock) => {
                out.push(e);
                if captioned {
                    out.push(Event::Html("</figure>".into()));
                    captioned = false;
                }
            }
            _ => out.push(e),
        }
    }

    *events = out;
}
