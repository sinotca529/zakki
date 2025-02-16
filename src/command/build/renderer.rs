pub mod context;
mod html_template;
mod pass;

use crate::copy_asset;
use crate::util::{BloomFilter, PathExt as _};
use crate::{
    config::Config,
    util::{copy_file, encode_with_password, write_file},
};
use anyhow::{anyhow, Context as _, Result};
use base64::{prelude::BASE64_STANDARD, Engine};
use context::{Context, Flag, Metadata};
use html_template::{crypto_html, index_html, page_html};
use pass::{
    assign_header_id, convert_math_pass, get_title_pass, highlight_code_pass, image_convert_pass, link_adjust_pass, read_header_pass
};
use pulldown_cmark::{Event, Options, Parser};
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::fs::File;
use std::io::Read as _;
use std::path::{Path, PathBuf};

pub struct Renderer<'a> {
    config: &'a Config,
}

impl<'a> Renderer<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    fn default_css_list(&self) -> [&'static str; 1] {
        ["style.css"]
    }

    fn default_js_list(&self) -> [&'static str; 3] {
        ["metadata.js", "script.js", "theme.js"]
    }

    fn events_to_html(&self, events: Vec<Event>, ctxt: &Context) -> Result<String> {
        let body = {
            let mut body = String::new();
            pulldown_cmark::html::push_html(&mut body, events.into_iter());
            body
        };

        let path_to_root = ctxt
            .build_root_to_dst()?
            .parent()
            .unwrap()
            .dir_path_to_origin_unchecked();

        let css_list = self
            .default_css_list()
            .into_iter()
            .chain(self.config.css_list().iter().map(|p| &p[..]))
            .chain(ctxt.css_list().iter().map(|p| &p[..]));

        let js_list = self
            .default_js_list()
            .into_iter()
            .chain(self.config.js_list().iter().map(|p| &p[..]))
            .chain(ctxt.js_list().iter().map(|p| &p[..]));

        let crypto = ctxt.flags()?.contains(&Flag::Crypto);
        let html = if crypto {
            let password = ctxt.password()?;
            let cypher = encode_with_password(password, body.as_bytes());
            let encoded = BASE64_STANDARD.encode(cypher);

            crypto_html(
                &path_to_root,
                self.config.site_name(),
                ctxt.title()?,
                ctxt.create_date()?,
                ctxt.last_update_date()?,
                css_list,
                js_list,
                ctxt.tags()?,
                &encoded,
                self.config.footer(),
            )
        } else {
            page_html(
                &path_to_root,
                self.config.site_name(),
                ctxt.title()?,
                ctxt.create_date()?,
                ctxt.last_update_date()?,
                css_list,
                js_list,
                ctxt.tags()?,
                &body,
                self.config.footer(),
            )
        };

        Ok(html)
    }

    fn make_bloom_filter(&self, html: &str) -> Result<BloomFilter> {
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

        // Bloom filter を構築する
        let fp = self.config.search_fp();
        let num_words = words.len();
        let mut filter = BloomFilter::new(num_words, fp);
        words.iter().for_each(|w| filter.insert_word(w));

        Ok(filter)
    }

    /// Markdown を HTML に変換します。
    /// 変換後の HTML とメタデータを返します。
    /// Markdown がドラフト記事であり、ドラフトを描画しない設定の場合は `None` を返します。
    fn md_to_html(&self, markdown: &str, dst_path: PathBuf) -> Result<Option<(String, Metadata)>> {
        let mut ctxt = Context::default();
        if let Some(password) = self.config.password() {
            ctxt.set_password(password.clone());
        }

        let build_root_to_dst = dst_path.strip_prefix(self.config.dst_dir()).unwrap();
        ctxt.set_build_root_to_dst(build_root_to_dst.to_owned());

        // Markdown をイベント列に変換
        let opt = Options::all() ^ Options::ENABLE_OLD_FOOTNOTES ^ Options::ENABLE_FOOTNOTES;
        let mut events: Vec<_> = Parser::new_ext(markdown, opt).collect();

        // イベント列に対してパスを適用
        read_header_pass(&mut events, &mut ctxt)?;

        if !self.config.render_draft() && ctxt.flags()?.contains(&Flag::Draft) {
            return Ok(None);
        }

        get_title_pass(&mut events, &mut ctxt)?;
        link_adjust_pass(&mut events, &mut ctxt)?;
        image_convert_pass(&mut events, &mut ctxt)?;
        highlight_code_pass(&mut events, &mut ctxt)?;
        convert_math_pass(&mut events, &mut ctxt)?;
        assign_header_id(&mut events, &mut ctxt)?;

        // イベント列を HTML に変換
        let html = self.events_to_html(events, &ctxt)?;

        // HTML に対してパスを適用
        let filter = self.make_bloom_filter(&html)?;
        ctxt.set_bloom_filter(filter);

        Ok(Some((html, ctxt.try_into()?)))
    }

    pub fn render(&self, src: impl AsRef<Path>) -> Result<Option<Metadata>> {
        let src = src.as_ref();
        if !src.extension_is("md") {
            copy_file(src, self.config.dst_path_of(src))?;
            return Ok(None);
        }

        let markdown = {
            let mut file = File::open(src)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            content
        };

        let dst_path = self.config.dst_path_of(src);
        let Some((html, meta)) = self.md_to_html(&markdown, dst_path.clone())? else {
            return Ok(None);
        };

        write_file(dst_path, html)?;

        Ok(Some(meta))
    }

    pub fn render_assets(&self) -> Result<()> {
        self.render_index()?;
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

        copy_asset!("font/SourceCodePro/LICENSE.md", self.config.dst_dir())?;
        copy_asset!(
            "font/SourceCodePro/SourceCodePro-Regular.otf.woff2",
            self.config.dst_dir()
        )?;

        Ok(())
    }

    fn render_index(&self) -> Result<()> {
        let css_list = self
            .default_css_list()
            .into_iter()
            .chain(self.config.css_list().iter().map(|p| &p[..]));

        let js_list = self
            .default_js_list()
            .into_iter()
            .chain(self.config.js_list().iter().map(|p| &p[..]));

        let content = index_html(
            self.config.site_name(),
            css_list,
            js_list,
            self.config.footer(),
        );

        let dst = self.config.dst_dir().join("index.html");
        write_file(dst, content).map_err(Into::into)
    }
}
