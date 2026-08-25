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

use jotdown::{Attributes, Container, Event};
use std::borrow::Cow;

/// 生の HTML をそのまま出力するイベント列を作ります。
///
/// djot は生の HTML を素通ししないため、パスが HTML を差し込むときは
/// `RawInline` で明示する必要があります。
fn raw_html<'a>(html: impl Into<Cow<'a, str>>) -> [Event<'a>; 3] {
    let format = || Container::RawInline {
        format: "html".into(),
    };
    [
        Event::Start(format(), Attributes::new()),
        Event::Str(html.into()),
        Event::End(format()),
    ]
}

/// HTML の特殊文字をエスケープします。
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
