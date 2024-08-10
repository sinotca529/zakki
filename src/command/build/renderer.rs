use super::content::{Content, HighlightMacro, Metadata};
use crate::util::PathExt as _;
use crate::{
    config::Config,
    copy_asset, include_asset,
    util::{copy_file, encode_with_password, write_file},
};
use anyhow::Result;
use base64::{prelude::BASE64_STANDARD, Engine};
use latex2mathml::{latex_to_mathml, DisplayStyle};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use serde::Serialize;
use std::path::{Path, PathBuf};

pub struct Renderer<'a> {
    config: &'a Config,
    metadatas: Vec<Metadata>,
}

// Passes
impl<'a> Renderer<'a> {
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

impl<'a> Renderer<'a> {
    pub fn new(config: &'a Config) -> Self {
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
            include_asset!("crypto.html"),
            encoded = encoded,
            path_to_root = path_to_dst_dir.to_str().unwrap(),
        )
    }

    fn make_html(&self, body: &str, meta: &Metadata) -> String {
        let path_to_root = self
            .config
            .dst_dir()
            .relative_path(self.config.dst_path_of(&meta.src_path).parent().unwrap())
            .unwrap();

        let plain_html = format!(
            include_asset!("page.html"),
            tag_elems = Self::tag_elems(&meta.tags, &path_to_root),
            create_date = meta.create_date,
            last_update_date = meta.last_update_date,
            path_to_root = path_to_root.to_str().unwrap(),
            body = body,
            site_name = self.config.site_name(),
            page_title = meta.title,
            footer = self.config.footer(),
        );

        if meta.flags.contains(&"crypto".to_owned()) {
            self.crypto_html(&plain_html, &path_to_root)
        } else {
            plain_html
        }
    }

    pub fn render(&mut self, content: Content) -> Result<()> {
        match content {
            Content::Other { src_path: path } => {
                copy_file(&path, self.config.dst_path_of(&path))?;
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
                write_file(self.config.dst_path_of(&metadata.src_path), html)?;

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
            include_asset!("index.html"),
            site_name = self.config.site_name(),
            footer = self.config.footer(),
        );
        let dst = self.config.dst_dir().join("index.html");
        write_file(dst, content).map_err(Into::into)
    }

    fn render_tag(&self) -> Result<()> {
        let content = format!(
            include_asset!("tag.html"),
            site_name = self.config.site_name(),
            footer = self.config.footer(),
        );
        let dst = self.config.dst_dir().join("tag.html");
        write_file(dst, content).map_err(Into::into)
    }

    pub fn save_metadata(&self) -> Result<()> {
        let metas: Vec<MetadataToDump> = self
            .metadatas
            .iter()
            .map(|m| MetadataToDump::from(m, &self.config))
            .collect();
        let js = serde_json::to_string(&metas)?;
        let content = format!("const METADATA={js}");
        let dst = self.config.dst_dir().join("metadata.js");
        write_file(dst, content).map_err(Into::into)
    }
}

#[derive(Serialize)]
struct MetadataToDump<'a> {
    create: &'a String,
    update: &'a String,
    tags: &'a Vec<String>,
    flags: &'a Vec<String>,
    title: &'a String,
    path: PathBuf,
}

impl<'a> MetadataToDump<'a> {
    fn from(meta: &'a Metadata, cfg: &Config) -> Self {
        Self {
            create: &meta.create_date,
            update: &meta.last_update_date,
            tags: &meta.tags,
            flags: &meta.flags,
            title: &meta.title,
            path: cfg
                .dst_path_of(&meta.src_path)
                .relative_path(cfg.dst_dir())
                .unwrap(),
        }
    }
}
