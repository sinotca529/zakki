use super::html_block;
use comrak::Arena;
use comrak::nodes::{AstNode, NodeValue};

/// 表を横スクロールできるよう `<div class="table-wrapper">` で囲みます。
pub fn wrap_table<'a>(arena: &'a Arena<'a>, root: &'a AstNode<'a>) -> anyhow::Result<()> {
    let tables: Vec<_> = root
        .descendants()
        .filter(|n| matches!(n.data().value, NodeValue::Table(_)))
        .collect();

    for table in tables {
        table.insert_before(html_block(arena, r#"<div class="table-wrapper">"#));
        table.insert_after(html_block(arena, "</div>"));
    }

    Ok(())
}
