mod html_template;
pub mod context;
mod pass;

use crate::copy_asset;
use crate::path::{dst_path_of, zakki_dst_dir};
use crate::util::{BloomFilter, PathExt as _};
use crate::{config::Config, util};
use anyhow::{Context as _, Result, anyhow};
use base64::{Engine, prelude::BASE64_STANDARD};
use html_template::{all_tags_html, cards_html, crypto_html, index_html, page_html};
use context::Metadata;
use itertools::Itertools;
use context::Context;
use pulldown_cmark::{Event, Options, Parser};
use scraper::{Html, Selector};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub struct Renderer<'a> {
    config: &'a Config,
    title_map: &'a HashMap<PathBuf, String>,
}

impl<'a> Renderer<'a> {
    pub fn new(config: &'a Config, title_map: &'a HashMap<PathBuf, String>) -> Self {
        Self { config, title_map }
    }

    pub fn render(&self, src: impl AsRef<Path>) -> Result<Option<Context>> {
        let src = src.as_ref();
        if !src.extension_is("md") {
            util::copy_file(src, dst_path_of(src)?)?;
            return Ok(None);
        }

        let raw_md = std::fs::read_to_string(src)?;
        let dst_path = dst_path_of(src)?;
        let Some((html, meta)) = self.md_to_html(&raw_md, src, dst_path.clone())? else {
            return Ok(None);
        };

        util::write_file(dst_path, html)?;

        Ok(Some(meta))
    }

    fn events_to_html(&self, events: Vec<Event>, ctxt: &Context) -> Result<String> {
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
            .map(String::as_str)
            .chain(ctxt.css_list().iter().map(String::as_str));

        let js_list = self
            .config
            .js_list()
            .iter()
            .map(String::as_str)
            .chain(ctxt.js_list().iter().map(String::as_str));

        let toc = extract_toc_html(&body);
        let article = format!("{}<div id=\"main-content\">{}</div>", toc, body);

        let html = if ctxt.to_encrypt {
            let password = ctxt.password()?;
            let cypher = util::encode_with_password(password, article.as_bytes());
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
                &article,
                self.config.footer(),
            )
        };

        Ok(html)
    }

    fn make_bloom_filter(&self, html: &str) -> Result<BloomFilter> {
        // HTML からテキストを抜き出す
        let text = Html::parse_document(html)
            .select(&Selector::parse("#article, #title").unwrap())
            .next()
            .ok_or_else(|| anyhow!("No body element"))?
            .text()
            .join(" ");

        // テキストをトークンに分割する
        let words: HashSet<_> = crate::util::tokenize(&text).into_iter().collect();

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
    fn md_to_html(
        &self,
        markdown: &str,
        src_path: &Path,
        dst_path: PathBuf,
    ) -> Result<Option<(String, Context)>> {
        let mut ctxt = Context::default();
        if let Some(password) = self.config.password() {
            ctxt.set_password(password.clone());
        }

        let dst_rel_path = dst_path.strip_prefix(zakki_dst_dir()?).unwrap();
        ctxt.is_draft = dst_rel_path.starts_with("draft/");
        ctxt.to_encrypt = dst_rel_path.starts_with("private/");
        ctxt.is_sub =
            !dst_rel_path.ends_with("index.html") && dst_rel_path.components().count() >= 3;
        ctxt.set_dst_rel_path(dst_rel_path.to_owned());
        ctxt.set_src_path(src_path.to_owned());

        // タイトルを title_map から設定する
        let title = self
            .title_map
            .get(src_path)
            .with_context(|| anyhow!("タイトルが見つかりません: {}", src_path.display()))?
            .clone();
        ctxt.set_title(title);

        // Markdown をイベント列に変換
        let opt = Options::all() ^ Options::ENABLE_OLD_FOOTNOTES ^ Options::ENABLE_FOOTNOTES;
        let mut events: Vec<_> = Parser::new_ext(markdown, opt).collect();

        // イベント列に対してパスを適用
        pass::read_header(&mut events, &mut ctxt)?;

        if !self.config.render_draft() && ctxt.is_draft {
            return Ok(None);
        }

        pass::adjust_link(&mut events, &mut ctxt, self.title_map)?;
        pass::convert_image(&mut events, &mut ctxt)?;
        pass::add_code_caption(&mut events, &mut ctxt)?;
        pass::highlight_code(&mut events, &mut ctxt)?;
        pass::convert_math(&mut events, &mut ctxt)?;
        pass::assign_header_id(&mut events, &mut ctxt)?;
        pass::wrap_table(&mut events, &mut ctxt)?;

        // イベント列を HTML に変換
        let html = self.events_to_html(events, &ctxt)?;

        // HTML に対してパスを適用
        let filter = self.make_bloom_filter(&html)?;
        ctxt.set_bloom_filter(filter);

        Ok(Some((html, ctxt)))
    }

    pub fn render_index(&self, metadatas: &[Metadata]) -> Result<()> {
        let cards = cards_html(metadatas);
        let tags = all_tags_html(metadatas);

        let content = index_html(
            self.config.site_name(),
            self.config.css_list().iter().map(|p| p.as_str()),
            self.config.js_list().iter().map(|p| p.as_str()),
            self.config.footer(),
            &cards,
            &tags,
        );

        let dst = zakki_dst_dir()?.join("index.html");
        util::write_file(dst, content).map_err(Into::into)
    }

    pub fn render_assets(&self) -> Result<()> {
        let dst_dir = zakki_dst_dir()?;
        copy_asset!("style.css", dst_dir)?;
        copy_asset!("script.js", dst_dir)?;

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

}

/// ファイルを BufReader で読み、YAML フロントマター部分だけ取り出して title を返します。
/// フロントマター終端の `---` に達した時点で読み込みを止めるため、大きなファイルでも効率的です。
pub fn extract_title_from_path(path: &std::path::Path) -> Result<Option<String>> {
    use std::io::{BufRead as _, BufReader};

    let file = std::fs::File::open(path)?;
    let mut lines = BufReader::new(file).lines();

    match lines.next() {
        Some(Ok(line)) if line == "---" => {}
        _ => return Ok(None),
    }

    let mut yaml = String::new();
    for line in lines {
        let line = line?;
        if line == "---" {
            break;
        }
        yaml.push_str(&line);
        yaml.push('\n');
    }

    if yaml.is_empty() {
        return Ok(None);
    }

    parse_title_from_yaml(&yaml)
}

fn parse_title_from_yaml(yaml: &str) -> Result<Option<String>> {
    #[derive(Deserialize)]
    struct TitleOnly {
        title: Option<String>,
    }

    let parsed: TitleOnly = serde_yaml::from_str(yaml)?;
    Ok(parsed.title)
}

/// レンダリング済みの body HTML から目次 HTML を生成します。
/// 見出しがない場合は空文字を返します。
fn extract_toc_html(body: &str) -> String {
    let doc = Html::parse_fragment(body);
    let selector = Selector::parse("h2[id], h3[id], h4[id]").unwrap();

    let items: Vec<(usize, String, String)> = doc
        .select(&selector)
        .map(|el| {
            let level = match el.value().name() {
                "h2" => 1,
                "h3" => 2,
                "h4" => 3,
                _ => 4,
            };
            let id = el.value().attr("id").unwrap_or("").to_string();
            let inner = el.inner_html();
            (level, id, inner)
        })
        .collect();

    if items.is_empty() {
        return String::new();
    }

    let mut html = Vec::<String>::new();
    let mut prev_level = 0;

    for (level, id, inner) in &items {
        // 階層を下る
        (prev_level..*level).for_each(|_| html.push("<ul><li>".to_string()));
        // 階層を上る
        (*level..prev_level).for_each(|_| html.push("</li></ul>".to_string()));
        // 次の要素へ
        if *level <= prev_level {
            html.push("</li><li>".to_string());
        }
        // リンクを追加
        html.push(format!("<a href=\"#{}\">{}</a>", id, inner));
        prev_level = *level;
    }
    // 閉じる
    (0..prev_level).for_each(|_| html.push("</li></ul>".to_string()));

    format!(
        "<details id=\"toc\"><summary>目次</summary>{}</details>",
        html.join("")
    )
}
