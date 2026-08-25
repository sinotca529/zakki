use super::{escape, raw_html};
use crate::command::build::renderer::context::Context;
use anyhow::Result;
use regex::Regex;
use serde::Deserialize;
use std::borrow::Cow;
use jotdown::{Container, Event};

/// コードブロックの中身に、記事で指定された区切り文字のスタイルを適用します。
pub fn highlight_code<'a>(events: &mut Vec<Event<'a>>, ctxt: &mut Context) -> Result<()> {
    let Ok(macros) = ctxt.highlights() else {
        return Ok(());
    };

    let mut out = Vec::with_capacity(events.len());
    let mut is_code_block = false;

    for e in events.drain(..) {
        match e {
            Event::Start(Container::CodeBlock { .. }, _) => {
                is_code_block = true;
                out.push(e);
            }
            Event::End(Container::CodeBlock { .. }) => {
                is_code_block = false;
                out.push(e);
            }
            Event::Str(ref s) if is_code_block => {
                let mut code = escape(s);
                for m in macros {
                    code = m.replace_all(&code).to_string();
                }
                out.extend(raw_html(code));
            }
            _ => out.push(e),
        }
    }

    *events = out;
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
