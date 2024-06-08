use super::{metadata::HighlightMacro, metadata::MetadataBuilder};
use latex2mathml::{latex_to_mathml, DisplayStyle};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, MetadataBlockKind, Tag, TagEnd};

pub fn adjust_link_to_md(event: &mut Vec<Event>) {
    for e in event {
        if let Event::Start(Tag::Link { dest_url, .. }) = e {
            let is_local_file =
                !dest_url.starts_with("http://") && !dest_url.starts_with("https://");
            let is_md_file = dest_url.ends_with(".md");

            if is_local_file && is_md_file {
                *dest_url = format!("{}.html", &dest_url[..dest_url.len() - ".md".len()]).into();
            }
        }
    }
}

pub fn convert_math(events: &mut Vec<Event>) {
    for e in events {
        match e {
            Event::InlineMath(latex) => {
                let mathml = latex_to_mathml(latex, DisplayStyle::Inline).unwrap();
                *e = Event::InlineHtml(mathml.into());
            }
            Event::DisplayMath(latex) => {
                let mathml = latex_to_mathml(latex, DisplayStyle::Block).unwrap();
                *e = Event::InlineHtml(mathml.into());
            }
            _ => {}
        }
    }
}

pub fn read_yaml_header(events: &Vec<Event>, metadata: &mut MetadataBuilder) {
    let mut is_metadata_block = false;
    for e in events {
        match e {
            Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle)) => {
                is_metadata_block = true;
            }
            Event::End(TagEnd::MetadataBlock(MetadataBlockKind::YamlStyle)) => {
                is_metadata_block = false;
            }
            Event::Text(yaml) => {
                if is_metadata_block {
                    metadata.read_yaml(yaml);
                }
            }
            _ => {}
        }
    }
}

pub fn highlight_code(events: &mut Vec<Event>, macros: &[HighlightMacro]) {
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
                if !is_code_block {
                    continue;
                }

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
            _ => {}
        }
    }
}

pub fn get_h1(events: &Vec<Event>, metadata: &mut MetadataBuilder) {
    let mut inner_h1 = false;
    for e in events {
        match e {
            Event::Start(Tag::Heading { level, .. }) if level == &HeadingLevel::H1 => {
                inner_h1 = true;
            }
            Event::End(TagEnd::Heading(level)) if level == &HeadingLevel::H1 => {
                inner_h1 = false;
            }
            Event::Text(t) => {
                if inner_h1 {
                    metadata.title(t.to_string());
                }
            }
            _ => {}
        }
    }
}
