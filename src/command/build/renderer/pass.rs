mod add_code_caption;
mod adjust_link;
mod assign_header_id;
mod convert_image;
mod convert_math;
mod highlight_code;
mod read_header;
mod wrap_table;

pub use add_code_caption::add_code_caption;
pub use adjust_link::adjust_link;
pub use assign_header_id::assign_header_id;
pub use convert_image::convert_image;
pub use convert_math::convert_math;
pub use highlight_code::{HighlightRule, highlight_code};
pub use read_header::read_header;
pub use wrap_table::wrap_table;

use jotdown::{AttributeKind, Attributes, Container, Event};
use std::borrow::Cow;

/// 生の HTML をそのまま出力するイベント列を作ります。
///
/// djot は生の HTML を素通ししないため、パスが HTML を差し込むときは
/// `RawInline` で明示する必要があります。
fn raw_html<'a>(html: impl Into<Cow<'a, str>>) -> [Event<'a>; 3] {
    let raw_inline = Container::RawInline {
        format: "html".into(),
    };
    [
        Event::Start(raw_inline.clone(), Attributes::new()),
        Event::Str(html.into()),
        Event::End(raw_inline),
    ]
}

/// HTML の特殊文字をエスケープします。
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// 属性を HTML の属性文字列にします (先頭に空白が付きます)。
///
/// djot では記事側から任意の属性を書けるため、パスが自前で HTML を組み立てる
/// 要素についても、書かれた属性をそのまま引き継ぎます。
fn attrs_to_html(attrs: &Attributes) -> String {
    let mut out = String::new();

    // クラスは複数書けるので、まとめて 1 つの属性にする
    if let Some(class) = attrs.get_value("class") {
        out.push_str(&format!(r#" class="{}""#, escape_attr(&class.to_string())));
    }

    for (kind, value) in attrs {
        match kind {
            AttributeKind::Class | AttributeKind::Comment => continue,
            _ => {}
        }
        let Some(key) = kind.key() else { continue };
        out.push_str(&format!(
            r#" {}="{}""#,
            key,
            escape_attr(&value.to_string())
        ));
    }

    out
}

/// HTML の属性値をエスケープします。
fn escape_attr(value: &str) -> String {
    escape(value).replace('"', "&quot;")
}
