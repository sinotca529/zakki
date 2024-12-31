mod convert_math_pass;
mod get_title_pass;
mod highlight_code_pass;
mod image_convert_pass;
mod link_adjust_pass;
mod read_header_pass;

use super::context::Context;
use pulldown_cmark::Event;

pub use convert_math_pass::convert_math_pass;
pub use get_title_pass::get_title_pass;
pub use highlight_code_pass::{highlight_code_pass, HighlightRule};
pub use image_convert_pass::image_convert_pass;
pub use link_adjust_pass::link_adjust_pass;
pub use read_header_pass::read_header_pass;

pub type EventPass = fn(events: &mut Vec<Event>, ctxt: &mut Context) -> anyhow::Result<()>;
