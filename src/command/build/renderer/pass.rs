mod add_code_caption;
mod adjust_link;
mod convert_image;
mod convert_math;
mod highlight_code;
mod read_header;
mod wrap_table;

pub use add_code_caption::add_code_caption;
pub use adjust_link::adjust_link;
pub use convert_image::convert_image;
pub use convert_math::convert_math;
pub use highlight_code::{HighlightRule, highlight_code};
pub use read_header::read_header;
pub use wrap_table::wrap_table;

use comrak::Arena;
use comrak::nodes::{AstNode, NodeHtmlBlock, NodeValue};

/// 生の HTML を出力するブロックノードを作ります。
fn html_block<'a>(arena: &'a Arena<'a>, literal: impl Into<String>) -> &'a AstNode<'a> {
    let value = NodeValue::HtmlBlock(NodeHtmlBlock {
        block_type: 0,
        literal: literal.into(),
    });
    arena.alloc(AstNode::from(value))
}

/// ノードの子孫のテキストを連結します。
fn text_of<'a>(node: &'a AstNode<'a>) -> String {
    node.descendants()
        .filter_map(|n| match &n.data().value {
            NodeValue::Text(t) => Some(t.to_string()),
            NodeValue::Code(c) => Some(c.literal.clone()),
            _ => None,
        })
        .collect()
}

/// HTML のテキスト内容として使えるようエスケープします。
pub(super) fn escape_html_text(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;")
}

/// HTML の属性値として使えるようエスケープします。
fn escape_html_attr(attr: &str) -> String {
    attr.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\'', "&apos;")
        .replace('"', "&quot;")
}
