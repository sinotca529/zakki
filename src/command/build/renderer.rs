mod html_template;
pub mod metadata;
mod pass;

use crate::copy_asset;
use crate::path::{dst_path_of, zakki_dst_dir};
use crate::util::{BloomFilter, PathExt as _};
use crate::{config::Config, util};
use anyhow::{Context as _, Result, anyhow};
use base64::{Engine, prelude::BASE64_STANDARD};
use html_template::{crypto_html, index_html, page_html};
use itertools::Itertools;
use metadata::Metadata;
use pass::PassManager;
use pulldown_cmark::{Event, Options, Parser};
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub struct Renderer<'a> {
    config: &'a Config,
}

impl<'a> Renderer<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn render(&self, src: impl AsRef<Path>) -> Result<Option<Metadata>> {
        let src = src.as_ref();
        if !src.extension_is("md") {
            util::copy_file(src, dst_path_of(src)?)?;
            return Ok(None);
        }

        let md = std::fs::read_to_string(src)?;
        let dst_path = dst_path_of(src)?;
        let Some((html, meta)) = self.md_to_html(&md, dst_path.clone())? else {
            return Ok(None);
        };

        util::write_file(dst_path, html)?;

        Ok(Some(meta))
    }

    fn events_to_html(&self, events: Vec<Event>, ctxt: &Metadata) -> Result<String> {
        let body = {
            let mut body = String::new();
            pulldown_cmark::html::push_html(&mut body, events.into_iter());
            body
        };

        let path_to_root = ctxt
            .dst_rel_path()?
            .parent()
            .unwrap()
            .dir_path_to_origin_unchecked();

        let css_list = self
            .config
            .css_list()
            .iter()
            .map(|p| &p[..])
            .chain(ctxt.css_list().iter().map(|p| &p[..]));

        let js_list = self
            .config
            .js_list()
            .iter()
            .map(|p| &p[..])
            .chain(ctxt.js_list().iter().map(|p| &p[..]));

        let html = if ctxt.to_encrypt {
            let password = ctxt.password()?;
            let cypher = util::encode_with_password(password, body.as_bytes());
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
                ctxt.toc()?,
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
            .join(" ");

        // テキストをワードに分割する
        let words: HashSet<_> = crate::util::segment(&text)
            .into_iter()
            // スペースのみの場合は無視する
            .map(|w| w.trim())
            .filter(|w| !w.is_empty())
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
        let mut ctxt = Metadata::default();
        if let Some(password) = self.config.password() {
            ctxt.set_password(password.clone());
        }

        let dst_rel_path = dst_path.strip_prefix(zakki_dst_dir()?).unwrap();
        ctxt.is_draft = dst_rel_path.starts_with("draft/");
        ctxt.to_encrypt = dst_rel_path.starts_with("private/");
        ctxt.is_sub =
            !dst_rel_path.ends_with("index.html") && dst_rel_path.components().count() >= 3;
        ctxt.set_dst_rel_path(dst_rel_path.to_owned());

        // Markdown をイベント列に変換
        let opt = Options::all() ^ Options::ENABLE_OLD_FOOTNOTES ^ Options::ENABLE_FOOTNOTES;
        let mut events: Vec<_> = Parser::new_ext(markdown, opt).collect();

        // イベント列に対してパスを適用
        pass::read_header_pass(&mut events, &mut ctxt)?;

        if !self.config.render_draft() && ctxt.is_draft {
            return Ok(None);
        }

        let mut pass_manager = PassManager::new();
        pass_manager
            .register(pass::get_title_pass)
            .register(pass::adjust_link_pass)
            .register(pass::image_convert_pass)
            .register(pass::highlight_code_pass)
            .register(pass::convert_math_pass)
            .register(pass::assign_header_id)
            .register(pass::table_wrapper_pass)
            .register(pass::toc_pass);

        let events = pass_manager.run(events, &mut ctxt)?;

        // イベント列を HTML に変換
        let html = self.events_to_html(events, &ctxt)?;

        // HTML に対してパスを適用
        let filter = self.make_bloom_filter(&html)?;
        ctxt.set_bloom_filter(filter);

        Ok(Some((html, ctxt)))
    }

    pub fn render_assets(&self) -> Result<()> {
        let dst_dir = zakki_dst_dir()?;

        self.render_index()?;
        copy_asset!("style.css", dst_dir)?;
        copy_asset!("script.js", dst_dir)?;
        copy_asset!("segmenter.js", dst_dir)?;
        copy_asset!("theme.js", dst_dir)?;

        copy_asset!("katex/LICENSE", dst_dir)?;
        copy_asset!("katex/katex.min.css", dst_dir)?;

        macro_rules! copy_katex_fonts {
            ($($font_name:literal),* $(,)?) => {
                $(
                    copy_asset!(concat!("katex/fonts/", $font_name), dst_dir)?;
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

        copy_asset!("font/SourceCodePro/LICENSE.md", dst_dir)?;
        copy_asset!(
            "font/SourceCodePro/SourceCodePro-Regular.otf.woff2",
            dst_dir
        )?;

        Ok(())
    }

    fn render_index(&self) -> Result<()> {
        let css_list = self.config.css_list().iter().map(|p| &p[..]);
        let js_list = self.config.js_list().iter().map(|p| &p[..]);

        let content = index_html(
            self.config.site_name(),
            css_list,
            js_list,
            self.config.footer(),
        );

        let dst = zakki_dst_dir()?.join("index.html");
        util::write_file(dst, content).map_err(Into::into)
    }
}
