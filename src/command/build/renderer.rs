mod metadata;
mod yaml_header;

use crate::util::PathExt as _;
use crate::{
    config::Config,
    util::{copy_file, encode_with_password, write_file},
};
use crate::{copy_asset, include_asset};
use anyhow::{anyhow, bail, Context, Result};
use base64::{prelude::BASE64_STANDARD, Engine};
use latex2mathml::{latex_to_mathml, DisplayStyle};
pub use metadata::{Flag, HighlightMacro, Metadata};
use pulldown_cmark::{
    CodeBlockKind, Event, HeadingLevel, MetadataBlockKind, Options, Parser, Tag, TagEnd,
};
use std::fs::File;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use yaml_header::YamlHeader;

pub struct Renderer<'a> {
    config: &'a Config,
}

// Passes
impl<'a> Renderer<'a> {
    /// イベント列から yaml ヘッダを取得して YamlHeader に変換する
    fn read_header(events: &[Event], meta: &mut Metadata) -> Result<()> {
        use MetadataBlockKind::YamlStyle;

        let header = events
            .iter()
            .skip_while(|e| !matches!(e, Event::Start(Tag::MetadataBlock(YamlStyle))))
            .take_while(|e| !matches!(e, Event::End(TagEnd::MetadataBlock(YamlStyle))))
            .filter_map(|e| match e {
                Event::Text(t) => Some(t),
                _ => None,
            })
            .next();

        let Some(header) = header else {
            bail!("Yaml header is not existing.")
        };

        let header: YamlHeader = serde_yaml::from_str(header)?;
        header.merge_into(meta);

        Ok(())
    }

    /// イベント列からページのタイトルを取得する
    fn get_page_title(events: &[Event], meta: &mut Metadata) {
        let h1 = events
            .iter()
            .skip_while(|e| !matches!(e, Event::Start(Tag::Heading { level, .. }) if level == &HeadingLevel::H1))
            .take_while(|e| !matches!(e, Event::End(TagEnd::Heading(HeadingLevel::H1))))
            .filter_map(|e| match e {
                Event::Text(t) => Some(t.to_string()),
                _ => None,
            })
            .next();

        meta.set_title(h1.unwrap_or("No Title".to_owned()));
    }

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
        Self { config }
    }

    fn tag_elems(tags: &[String], dst_root_dir: &Path) -> String {
        let nsbp = "\u{00a0}";
        tags.iter()
            .map(|n| {
                let path = dst_root_dir.join("tag.html");
                let path = path.to_str().unwrap();
                format!(r#"<a class="tag" href="{path}?tag={n}">{n}</a>"#)
            })
            .fold(String::new(), |acc, e| format!("{acc}{nsbp}{e}"))
    }

    fn crypto_html(&self, html: &str, path_to_dst_dir: &Path) -> Result<String> {
        let html = html.as_bytes();
        let password = self
            .config
            .password()
            .ok_or_else(|| anyhow!("Password has not been found at zakki.toml"))?;

        let cypher = encode_with_password(&password, html);
        let encoded = BASE64_STANDARD.encode(cypher);
        Ok(format!(
            include_asset!("crypto.html"),
            encoded = encoded,
            path_to_root = path_to_dst_dir.to_str().unwrap(),
        ))
    }

    fn make_html(&self, body: &str, meta: &Metadata) -> Result<String> {
        let path_to_root = self
            .config
            .dst_dir()
            .relative_path(self.config.dst_path_of(&meta.src_path()?).parent().unwrap())
            .unwrap();

        let plain_html = format!(
            include_asset!("page.html"),
            tag_elems = Self::tag_elems(&meta.tags()?, &path_to_root),
            create_date = meta.create_date()?,
            last_update_date = meta.last_update_date()?,
            path_to_root = path_to_root.to_str().unwrap(),
            body = body,
            site_name = self.config.site_name(),
            page_title = meta.title()?,
            footer = self.config.footer(),
        );

        if meta.flags()?.contains(&Flag::Crypto) {
            self.crypto_html(&plain_html, &path_to_root)
        } else {
            Ok(plain_html)
        }
    }

    fn md_to_html(&self, markdown: &str, meta: &mut Metadata) -> Result<Option<String>> {
        let mut events: Vec<_> = Parser::new_ext(&markdown, Options::all()).collect();

        Self::read_header(&events, meta)?;

        if !self.config.render_draft() && meta.flags()?.contains(&Flag::Draft) {
            return Ok(None);
        }

        Self::adjust_link_to_md(&mut events);
        Self::convert_math(&mut events);
        Self::highlight_code(&mut events, meta.highlights()?);
        Self::get_page_title(&events, meta);

        let body = {
            let mut body = String::new();
            pulldown_cmark::html::push_html(&mut body, events.into_iter());
            body
        };

        let html = self.make_html(&body, &meta)?;

        // TODO: Create a bloom filter.

        Ok(Some(html))
    }

    pub fn render(&self, src: PathBuf) -> Result<Option<Metadata>> {
        if src.extension_is("md") {
            let mut meta = Metadata::default();
            let markdown = {
                let mut file = File::open(&src)?;
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                content
            };

            meta.set_src_path(src);
            let Some(html) = self.md_to_html(&markdown, &mut meta)? else {
                return Ok(None);
            };

            write_file(self.config.dst_path_of(&meta.src_path()?), html)?;

            Ok(Some(meta))
        } else {
            copy_file(&src, self.config.dst_path_of(&src))?;
            Ok(None)
        }
    }

    pub fn render_assets(&self) -> Result<()> {
        self.render_index()?;
        self.render_tag()?;
        copy_asset!("style.css", self.config.dst_dir())?;
        copy_asset!("script.js", self.config.dst_dir())?;
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
}
