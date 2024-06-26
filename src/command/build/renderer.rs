use super::content::{Content, HighlightMacro, Metadata};
use crate::{
    config::Config,
    copy_asset,
    path::dst_dir,
    read_asset,
    util::{copy_file, encode_with_password, write_file},
};
use anyhow::Result;
use base64::{prelude::BASE64_STANDARD, Engine};
use latex2mathml::{latex_to_mathml, DisplayStyle};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use serde::Serialize;
use std::path::Path;

pub struct Renderer {
    config: Config,
    metadatas: Vec<Metadata>,
}

// Passes
impl Renderer {
    fn adjust_link_to_md(event: &mut Vec<Event>) {
        for e in event {
            if let Event::Start(Tag::Link { dest_url, .. }) = e {
                let is_local_file =
                    !dest_url.starts_with("http://") && !dest_url.starts_with("https://");
                let is_md_file = dest_url.ends_with(".md");

                if is_local_file && is_md_file {
                    *dest_url =
                        format!("{}.html", &dest_url[..dest_url.len() - ".md".len()]).into();
                }
            }
        }
    }

    fn convert_math(events: &mut Vec<Event>) {
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

    fn highlight_code(events: &mut Vec<Event>, macros: &[HighlightMacro]) {
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
}

impl Renderer {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            metadatas: Vec::new(),
        }
    }

    fn tag_elems(tags: &[String], path_to_dst_dir: &Path) -> String {
        let nsbp = "\u{00a0}";
        tags.iter()
            .map(|n| {
                let path = path_to_dst_dir.join("tag.html");
                let path = path.to_str().unwrap();
                format!(r#"<a class="tag" href="{path}?tag={n}">{n}</a>"#)
            })
            .fold(String::new(), |acc, e| format!("{acc}{nsbp}{e}"))
    }

    fn crypto_html(&self, html: &str, path_to_dst_dir: &Path) -> String {
        let html = html.as_bytes();
        let password = self.config.password();

        let cypher = encode_with_password(&password, html);
        let encoded = BASE64_STANDARD.encode(cypher);
        format!(
            read_asset!("crypto.html"),
            encoded = encoded,
            path_to_root = path_to_dst_dir.to_str().unwrap(),
        )
    }

    fn make_html(&self, body: &str, meta: &Metadata) -> String {
        let dst_path = meta.path.dst_path();
        let path_to_root = meta.path.path_to_dst_dir();

        let plain_html = format!(
            read_asset!("page.html"),
            tag_elems = Self::tag_elems(&meta.tags, dst_path),
            data = meta.date,
            path_to_root = path_to_root.to_str().unwrap(),
            body = body,
            site_name = self.config.site_name(),
            page_title = meta.title,
        );

        if meta.flags.contains(&"crypto".to_owned()) {
            self.crypto_html(&plain_html, meta.path.path_to_dst_dir())
        } else {
            plain_html
        }
    }

    pub fn render(&mut self, content: Content) -> Result<()> {
        match content {
            Content::Other { path } => {
                copy_file(path.src_path(), path.dst_path())?;
            }
            Content::Markdown { metadata, content } => {
                if !self.config.render_draft() && metadata.flags.contains(&"draft".to_owned()) {
                    return Ok(());
                }

                let mut events: Vec<_> = Parser::new_ext(&content, Options::all()).collect();
                Self::adjust_link_to_md(&mut events);
                Self::convert_math(&mut events);
                Self::highlight_code(&mut events, &metadata.highlights);

                let body = {
                    let mut body = String::new();
                    pulldown_cmark::html::push_html(&mut body, events.into_iter());
                    body
                };

                let html = self.make_html(&body, &metadata);
                write_file(metadata.path.dst_path(), html)?;

                self.metadatas.push(metadata);
            }
        }

        Ok(())
    }

    pub fn render_assets(&self) -> Result<()> {
        self.render_index()?;
        self.render_tag()?;
        copy_asset!("style.css", "build")?;
        copy_asset!("script.js", "build")?;
        Ok(())
    }

    fn render_index(&self) -> Result<()> {
        let content = format!(
            read_asset!("index.html"),
            site_name = self.config.site_name()
        );
        write_file(dst_dir().join("index.html"), content).map_err(Into::into)
    }

    fn render_tag(&self) -> Result<()> {
        let content = format!(read_asset!("tag.html"), site_name = self.config.site_name());
        write_file(dst_dir().join("tag.html"), content).map_err(Into::into)
    }

    pub fn save_metadata(&self) -> Result<()> {
        let metas: Vec<MetadataToDump> = self.metadatas.iter().map(Into::into).collect();
        let js = serde_json::to_string(&metas)?;
        let content = format!("const METADATA={js}");
        write_file(dst_dir().join("metadata.js"), content).map_err(Into::into)
    }
}

#[derive(Serialize)]
struct MetadataToDump<'a> {
    date: &'a String,
    tags: &'a Vec<String>,
    flags: &'a Vec<String>,
    title: &'a String,
    path: &'a Path,
}

impl<'a> From<&'a Metadata> for MetadataToDump<'a> {
    fn from(meta: &'a Metadata) -> Self {
        Self {
            date: &meta.date,
            tags: &meta.tags,
            flags: &meta.flags,
            title: &meta.title,
            path: meta.path.rel_path_from_dst_dir(),
        }
    }
}
