use super::{escape_html_text, html_block};
use comrak::Arena;
use comrak::nodes::{AstNode, NodeValue};

/// コードブロックの info string に `:タイトル` が含まれている場合、
/// `<figure class="code-figure">` と `<figcaption>` で囲みます。
///
/// 例: ` ```python:ソートアルゴリズム ` → figcaption 付きの figure に変換
pub fn add_code_caption<'a>(arena: &'a Arena<'a>, root: &'a AstNode<'a>) -> anyhow::Result<()> {
    let targets: Vec<_> = root
        .descendants()
        .filter_map(|node| {
            let NodeValue::CodeBlock(code) = &node.data().value else {
                return None;
            };
            let (lang, title) = code.info.split_once(':')?;
            let title = title.trim();
            (!title.is_empty()).then(|| (node, lang.to_owned(), title.to_owned()))
        })
        .collect();

    for (node, lang, title) in targets {
        // info string からタイトルを取り除き、言語名だけ残す
        if let NodeValue::CodeBlock(code) = &mut node.data_mut().value {
            code.info = lang;
        }

        let caption = escape_html_text(&title);
        node.insert_before(html_block(
            arena,
            format!(r#"<figure class="code-figure"><figcaption>{caption}</figcaption>"#),
        ));
        node.insert_after(html_block(arena, "</figure>"));
    }

    Ok(())
}
