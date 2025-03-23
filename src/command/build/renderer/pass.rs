mod assign_header_id;
mod convert_math_pass;
mod get_title_pass;
mod highlight_code_pass;
mod image_convert_pass;
mod link_adjust_pass;
mod read_header_pass;
mod table_wrapper_pass;
mod toc_pass;

use super::context::Context;
use pulldown_cmark::Event;

pub use assign_header_id::assign_header_id;
pub use convert_math_pass::convert_math_pass;
pub use get_title_pass::get_title_pass;
pub use highlight_code_pass::{HighlightRule, highlight_code_pass};
pub use image_convert_pass::image_convert_pass;
pub use link_adjust_pass::link_adjust_pass;
pub use read_header_pass::read_header_pass;
pub use table_wrapper_pass::table_wrapper_pass;
pub use toc_pass::{toc::Toc, toc_pass};

pub type EventPass<'a> = fn(Vec<Event<'a>>, &mut Context) -> anyhow::Result<Vec<Event<'a>>>;

pub struct PassManager<'a>(Vec<EventPass<'a>>);

impl<'a> PassManager<'a> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn register(&mut self, pass: EventPass<'a>) -> &mut Self {
        self.0.push(pass);
        self
    }

    pub fn run(
        &self,
        mut events: Vec<Event<'a>>,
        ctxt: &mut Context,
    ) -> anyhow::Result<Vec<Event<'a>>> {
        for pass in &self.0 {
            events = pass(events, ctxt)?;
        }
        Ok(events)
    }
}
