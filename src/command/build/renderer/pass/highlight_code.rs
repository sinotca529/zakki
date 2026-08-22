use crate::command::build::renderer::context::Context;
use anyhow::Result;
use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};
use regex::Regex;
use serde::Deserialize;
use std::borrow::Cow;

pub fn highlight_code<'a>(events: &mut Vec<Event<'a>>, ctxt: &mut Context) -> Result<()> {
    let Ok(macros) = ctxt.highlights() else {
        return Ok(());
    };

    let mut is_code_block = false;
    for e in events.iter_mut() {
        match e {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(_))) => {
                is_code_block = true;
            }
            Event::End(TagEnd::CodeBlock) => {
                is_code_block = false;
            }
            Event::Text(t) if is_code_block => {
                let mut code = t
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");

                for m in macros {
                    code = m.replace_all(&code).to_string();
                }

                *e = Event::InlineHtml(code.into());
            }
            _ => {}
        }
    }
    Ok(())
}

#[derive(Clone, Deserialize, Debug)]
pub struct HighlightRule {
    delim: [String; 2],
    style: String,
}

impl HighlightRule {
    pub fn replace_all<'a>(&self, code: &'a str) -> Cow<'a, str> {
        let Ok(pat) = Regex::new(&format!("{}(.*?){}", self.delim[0], self.delim[1])) else {
            return code.into();
        };

        pat.replace_all(code, format!("<span style=\"{}\">$1</span>", self.style))
    }
}
