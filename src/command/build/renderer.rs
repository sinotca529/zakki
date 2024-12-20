mod metadata;
mod yaml_header;

use crate::util::{BloomFilter, PathExt as _};
use crate::{
    config::Config,
    util::{copy_file, encode_with_password, write_file},
};
use crate::{copy_asset, include_asset};
use anyhow::{anyhow, bail, Context, Result};
use base64::{prelude::BASE64_STANDARD, Engine};
pub use metadata::{Flag, HighlightMacro, Metadata};
use pulldown_cmark::{
    CodeBlockKind, Event, HeadingLevel, LinkType, MetadataBlockKind, Options, Parser, Tag, TagEnd,
};
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::fs::File;
use std::io::Read as _;
use std::path::Path;
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

    fn adjust_link_to_md(event: &mut [Event]) {
        for e in event {
            if let Event::Start(Tag::Link { dest_url: url, .. }) = e {
                let is_local = !url.starts_with("http://") && !url.starts_with("https://");
                let is_md = url.ends_with(".md");
                if is_local && is_md {
                    *url = format!("{}.html", &url[..url.len() - ".md".len()]).into();
                }
            }
        }
    }

    fn convert_image(event: &mut [Event]) {
        for i in 1..event.len() {
            let (a, b) = event.split_at_mut(i);
            let first = a.last_mut().unwrap();
            let second = b.first_mut().unwrap();
            if let Event::Start(Tag::Image {
                link_type: LinkType::Inline,
                dest_url,
                title,
                id,
            }) = first
            {
                let alt_text = if let Event::Text(a) = second {
                    Some(&*a) // convert from &mut to &
                } else {
                    None
                };
                let alt_attr = alt_text
                    .map(|a| format!(r#"alt="{a}""#))
                    .unwrap_or_default();

                let img = if dest_url.ends_with(".svg") {
                    format!(
                        r#"<object type="image/svg+xml" data="{dest_url}" title="{title}" id="{id}"></object>"#
                    )
                } else {
                    format!(r#"<img loading="lazy" src="{dest_url}" {alt_attr} id="{id}" />"#)
                };

                let title = alt_text
                    .map(|a| format!(r#"<div>{a}</div>"#))
                    .unwrap_or_default();
                let html = format!(r#"<div class="zakki-img">{img}{title}</div>"#);

                *first = Event::InlineHtml(html.into());
                if alt_text.is_some() {
                    *second = Event::InlineHtml("".into());
                }
            }
        }
    }

    fn convert_math(events: &mut [Event]) {
        let opts_display = katex::Opts::builder()
            .output_type(katex::opts::OutputType::Html)
            .display_mode(true)
            .build()
            .unwrap();
        let opts_inline = katex::Opts::builder()
            .output_type(katex::opts::OutputType::Html)
            .display_mode(false)
            .build()
            .unwrap();

        for e in events {
            match e {
                Event::InlineMath(latex) => {
                    let math = katex::render_with_opts(latex, &opts_inline).unwrap();
                    *e = Event::InlineHtml(math.into());
                }
                Event::DisplayMath(latex) => {
                    let math = katex::render_with_opts(latex, &opts_display).unwrap();
                    *e = Event::InlineHtml(math.into());
                }
                _ => {}
            }
        }
    }

    fn highlight_code(events: &mut [Event], macros: &[HighlightMacro]) {
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

    fn events_to_html(&self, events: Vec<Event>, meta: &Metadata) -> Result<String> {
        let body = {
            let mut body = String::new();
            pulldown_cmark::html::push_html(&mut body, events.into_iter());
            body
        };

        let path_to_root = self
            .config
            .dst_dir()
            .path_from(meta.dst_path()?.parent().unwrap())
            .unwrap();

        let header = format!(
            include_asset!("header.html"),
            path_to_root = path_to_root.to_str().unwrap(),
            site_name = self.config.site_name(),
        );

        let crypto = meta.flags()?.contains(&Flag::Crypto);
        let html = if crypto {
            let password = meta
                .password()
                .or_else(|| self.config.password())
                .ok_or_else(|| anyhow!("Password has not been found at zakki.toml"))?;
            let cypher = encode_with_password(password, body.as_bytes());
            let encoded = BASE64_STANDARD.encode(cypher);

            format!(
                include_asset!("crypto.html"),
                create_date = meta.create_date()?,
                last_update_date = meta.last_update_date().unwrap(),
                tag_elems = Self::tag_elems(meta.tags()?, &path_to_root),
                header = header,
                page_title = meta.title().unwrap(),
                encoded = encoded,
                path_to_root = path_to_root.to_str().unwrap(),
            )
        } else {
            format!(
                include_asset!("page.html"),
                path_to_root = path_to_root.to_str().unwrap(),
                header = header,
                tag_elems = Self::tag_elems(meta.tags()?, &path_to_root),
                create_date = meta.create_date()?,
                last_update_date = meta.last_update_date()?,
                body = body,
                page_title = meta.title()?,
                footer_text = self.config.footer(),
            )
        };

        Ok(html)
    }

    fn make_bloom_filter(&self, html: &str, meta: &mut Metadata) -> Result<()> {
        if meta.flags()?.contains(&Flag::Crypto) {
            meta.set_bloom_filter(BloomFilter::default());
            return Ok(());
        }

        // HTML からテキストを抜き出す
        let text = Html::parse_document(html)
            .select(&Selector::parse("#main-content").unwrap())
            .next()
            .ok_or_else(|| anyhow!("No body element"))?
            .text()
            .collect::<Vec<_>>()
            .join(" ");

        // テキストをワードに分割する
        let words: HashSet<_> = crate::util::segment(&text)
            .into_iter()
            // スペースのみの場合は無視する
            .filter(|w| !w.trim().is_empty())
            // 小文字に統一する
            .map(|w| w.to_lowercase())
            .collect();

        // Bloom filter 用のパラメタを計算する
        let fp = self.config.search_fp();
        let num_words = words.len() as f64;
        let num_bit = -num_words * fp.ln() / 2.0f64.ln().powi(2);
        let num_byte = num_bit / 8.0;

        // Bloom filter を構築する
        let num_byte = num_byte.ceil() as u32;
        let num_hash = (num_bit * 2.0f64.ln() / num_words).ceil() as u8;
        let mut filter = BloomFilter::new(num_byte, num_hash);
        words.iter().for_each(|w| filter.insert_word(w));

        // 構築したフィルタをメタデータに登録する
        meta.set_bloom_filter(filter);

        Ok(())
    }

    fn md_to_html(&self, markdown: &str, meta: &mut Metadata) -> Result<Option<String>> {
        // Markdown を AST に変換
        let mut events: Vec<_> = Parser::new_ext(markdown, Options::all()).collect();

        // AST に対してパスを適用
        Self::read_header(&events, meta)?;
        if !self.config.render_draft() && meta.flags()?.contains(&Flag::Draft) {
            return Ok(None);
        }
        Self::adjust_link_to_md(&mut events);
        Self::convert_math(&mut events);
        Self::convert_image(&mut events);
        Self::highlight_code(&mut events, meta.highlights()?);
        Self::get_page_title(&events, meta);

        // AST を HTML に変換
        let html = self.events_to_html(events, meta)?;

        // HTML に対してパスを適用
        // self.encrypt(&mut html, meta)?;
        self.make_bloom_filter(&html, meta)?;

        Ok(Some(html))
    }

    pub fn render(&self, src: impl AsRef<Path>) -> Result<Option<Metadata>> {
        let src = src.as_ref();
        if src.extension_is("md") {
            let mut meta = Metadata::default();
            let markdown = {
                let mut file = File::open(src)?;
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                content
            };

            let dst_path = self.config.dst_path_of(src);
            let dst_path_from_root = dst_path.path_from(self.config.dst_dir()).unwrap();

            meta.set_dst_path(dst_path);
            meta.set_dst_path_from_root(dst_path_from_root);

            let Some(html) = self.md_to_html(&markdown, &mut meta)? else {
                return Ok(None);
            };

            write_file(meta.dst_path()?, html)?;

            Ok(Some(meta))
        } else {
            copy_file(src, self.config.dst_path_of(src))?;
            Ok(None)
        }
    }

    pub fn render_assets(&self) -> Result<()> {
        self.render_index()?;
        self.render_tag()?;
        copy_asset!("style.css", self.config.dst_dir())?;
        copy_asset!("script.js", self.config.dst_dir())?;
        copy_asset!("segmenter.js", self.config.dst_dir())?;
        copy_asset!("theme.js", self.config.dst_dir())?;

        copy_asset!("katex/LICENSE", self.config.dst_dir())?;
        copy_asset!("katex/katex.min.css", self.config.dst_dir())?;

        macro_rules! copy_katex_fonts {
            ($($font_name:literal),* $(,)?) => {
                $(
                    copy_asset!(concat!("katex/fonts/", $font_name), self.config.dst_dir())?;
                )*
            }
        }

        copy_katex_fonts!(
            "KaTeX_AMS-Regular.woff2",
            "KaTeX_Caligraphic-Bold.woff2",
            "KaTeX_Caligraphic-Regular.woff2",
            "KaTeX_Fraktur-Bold.woff2",
            "KaTeX_Fraktur-Regular.woff2",
            "KaTeX_Main-BoldItalic.woff2",
            "KaTeX_Main-Bold.woff2",
            "KaTeX_Main-Italic.woff2",
            "KaTeX_Main-Regular.woff2",
            "KaTeX_Math-BoldItalic.woff2",
            "KaTeX_Math-Italic.woff2",
            "KaTeX_SansSerif-Bold.woff2",
            "KaTeX_SansSerif-Italic.woff2",
            "KaTeX_SansSerif-Regular.woff2",
            "KaTeX_Script-Regular.woff2",
            "KaTeX_Size1-Regular.woff2",
            "KaTeX_Size2-Regular.woff2",
            "KaTeX_Size3-Regular.woff2",
            "KaTeX_Size4-Regular.woff2",
            "KaTeX_Typewriter-Regular.woff2",
        );

        Ok(())
    }

    fn render_index(&self) -> Result<()> {
        let header = format!(
            include_asset!("header.html"),
            path_to_root = ".",
            site_name = self.config.site_name(),
        );

        let content = format!(
            include_asset!("index.html"),
            header = header,
            site_name = self.config.site_name(),
            footer = self.config.footer(),
        );

        let dst = self.config.dst_dir().join("index.html");
        write_file(dst, content).map_err(Into::into)
    }

    fn render_tag(&self) -> Result<()> {
        let header = format!(
            include_asset!("header.html"),
            path_to_root = ".",
            site_name = self.config.site_name(),
        );

        let content = format!(
            include_asset!("tag.html"),
            header = header,
            footer = self.config.footer(),
        );

        let dst = self.config.dst_dir().join("tag.html");
        write_file(dst, content).map_err(Into::into)
    }
}
