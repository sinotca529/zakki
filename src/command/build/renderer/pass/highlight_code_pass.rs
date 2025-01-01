use crate::command::build::renderer::context::Context;
use anyhow::Result;
use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};
use regex::Regex;
use serde::Deserialize;
use std::borrow::Cow;

#[derive(Clone, Deserialize, Debug)]
pub struct HighlightRule {
    delim: [String; 2],
    style: String,
}

impl HighlightRule {
    pub fn replace_all<'a>(&self, code: &'a str) -> Cow<'a, str> {
        if let Ok(pat) = Regex::new(&format!("{}(.*?){}", &self.delim[0], &self.delim[1])) {
            pat.replace_all(code, format!("<span style=\"{}\">$1</span>", &self.style))
        } else {
            code.into()
        }
    }
}

pub fn highlight_code_pass(events: &mut Vec<Event>, ctxt: &mut Context) -> Result<()> {
    let Ok(macros) = ctxt.highlights() else {
        return Ok(());
    };

    let mut is_code_block = false;
    for e in events {
        match e {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(_))) => {
                is_code_block = true;
            }
            Event::End(TagEnd::CodeBlock) => {
                is_code_block = false;
            }
            Event::Text(t) => {
                if is_code_block {
                    let code = t.to_string();

                    let mut code = code
                        .to_string()
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;");

                    for m in macros {
                        code = m.replace_all(&code).to_string();
                    }

                    *e = Event::InlineHtml(code.into());
                }
            }
            _ => {}
        }
    }
    Ok(())
}
