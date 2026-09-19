mod heading_id;
mod html_component;
mod index;
mod page_locs;
mod page_meta;
mod pass;
mod toc;
mod url;

use crate::command::build::renderer::heading_id::NumberedHeadings;
use crate::command::build::renderer::html_component::{
    escape_html_text, footer, head, header, tag_elems,
};
use crate::command::build::renderer::pass::{PageFrontMatter, PassAssets};
use crate::command::build::renderer::url::Url;
use crate::config::ProjectConfig;
use crate::include_asset;
use crate::path::ProjectPaths;
use crate::util::{self, BloomFilter, PathExt as _};
use anyhow::{Context as _, Result};
use base64::{Engine, prelude::BASE64_STANDARD};
use comrak::nodes::AstNode;
use comrak::options::Plugins;
use comrak::{Arena, Options, format_html_with_plugins, parse_document};
use itertools::Itertools;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub use index::render_index;
pub use page_locs::PageLocs;
pub use page_meta::PageMetadata;

const FRONT_MATTER_DELIMITER: &str = "---";

pub struct Renderer<'a> {
    config: &'a ProjectConfig,
    title_map: &'a HashMap<PathBuf, String>,
    pj_paths: &'a ProjectPaths,
    render_draft: bool,
}

impl<'a> Renderer<'a> {
    pub fn new(
        config: &'a ProjectConfig,
        title_map: &'a HashMap<PathBuf, String>,
        pj_paths: &'a ProjectPaths,
        render_draft: bool,
    ) -> Self {
        Self {
            config,
            title_map,
            pj_paths,
            render_draft,
        }
    }

    pub fn render(&self, src_path: &Path) -> Result<Option<PageMetadata>> {
        let page_paths = PageLocs::new(src_path, self.pj_paths)?;

        if !src_path.extension_is("md") {
            util::copy_file(src_path, &page_paths.build_path)?;
            return Ok(None);
        }

        let content = std::fs::read_to_string(src_path)?;
        let Some((html, meta)) = self.md_to_html(&content, &page_paths)? else {
            return Ok(None);
        };

        util::write_file(page_paths.build_path, html)?;

        Ok(Some(meta))
    }

    fn render_page<'n>(
        &self,
        root: &'n AstNode<'n>,
        options: &Options,
        front_matter: &PageFrontMatter,
        page_paths: &PageLocs,
        pass_assets: &PassAssets,
        toc: &str,
    ) -> Result<String> {
        let body = {
            let heading_adapter = NumberedHeadings::default();
            let mut plugins = Plugins::default();
            plugins.render.heading_adapter = Some(&heading_adapter);

            let mut buf = String::new();
            format_html_with_plugins(root, options, &mut buf, &plugins)?;
            buf
        };

        let css_list = self
            .config
            .css_list
            .iter()
            .map(String::as_str)
            .chain(pass_assets.css_paths.iter().map(String::as_str));

        let js_list = self.config.js_list.iter().map(String::as_str);

        let article = format!("{}<div id=\"main-content\">{}</div>", toc, body);

        let is_private = self.pj_paths.is_private(page_paths.src_path);
        let html = if is_private {
            let password = front_matter
                .password
                .as_ref()
                .or(self.config.password.as_ref())
                .context(
                    "private フォルダ内の md ファイルを暗号化するにはパスワード設定が必要です",
                )?;

            let cypher = util::encode_with_password(password, article.as_bytes());
            let encoded = BASE64_STANDARD.encode(cypher);

            crypto_html(
                &page_paths.url_to_root,
                &self.config.site_name,
                &front_matter.title,
                &front_matter.create_date,
                &front_matter.last_update_date,
                css_list,
                js_list,
                &front_matter.tags,
                &encoded,
                &footer(&self.config.footer),
            )
        } else {
            page_html(
                &page_paths.url_to_root,
                &self.config.site_name,
                &front_matter.title,
                &front_matter.create_date,
                &front_matter.last_update_date,
                css_list,
                js_list,
                &front_matter.tags,
                &article,
                &footer(&self.config.footer),
            )
        };

        Ok(html)
    }

    fn make_bloom_filter(&self, title: &str, html: &str) -> Result<BloomFilter> {
        // HTML からテキストを抜き出す
        let body = Html::parse_document(html)
            .select(&Selector::parse("#article").unwrap())
            .flat_map(|e| e.text())
            .join(" ");

        let text = format!("{title} {body}");

        // テキストをトークンに分割する
        let words: HashSet<_> = util::tokenize(&text).into_iter().collect();

        // Bloom filter を構築する
        let fp = self.config.search_fp;
        let num_words = words.len();
        let mut filter = BloomFilter::new(num_words, fp);
        words.iter().for_each(|w| filter.insert_word(w));

        Ok(filter)
    }

    /// Markdown を HTML に変換します。
    /// 変換後の HTML とメタデータを返します。
    /// ドラフト記事であり、ドラフトを描画しない設定の場合は `None` を返します。
    fn md_to_html(
        &self,
        content: &str,
        page_paths: &PageLocs,
    ) -> Result<Option<(String, PageMetadata)>> {
        if !self.render_draft && self.pj_paths.is_draft(page_paths.src_path) {
            return Ok(None);
        }

        // Markdown を AST に変換
        let arena = Arena::new();
        let options = markdown_options();
        let root = parse_document(&arena, content, &options);

        // AST に対してパスを適用
        let front_matter = pass::read_front_matter(root)?;

        let mut pass_assets = PassAssets::default();
        pass::validate_heading_order(root)?;

        // 目次は見出しの文字列を使うため、数式を HTML に置き換える前に作る
        let toc = toc::toc_html(root);

        pass::adjust_link(&arena, root, page_paths.src_path, self.title_map)?;
        pass::convert_image(root)?;
        pass::add_code_caption(&arena, root)?;
        pass::highlight_code(root, &front_matter.highlights)?;
        pass::convert_math(root, &mut pass_assets)?;
        pass::wrap_table(&arena, root)?;

        // AST を HTML に変換
        let html = self.render_page(
            root,
            &options,
            &front_matter,
            page_paths,
            &pass_assets,
            &toc,
        )?;

        // HTML に対してパスを適用
        let filter = self.make_bloom_filter(&front_matter.title, &html)?;

        let metadata = PageMetadata {
            create: front_matter.create_date,
            update: front_matter.last_update_date,
            tags: front_matter.tags,
            title: front_matter.title,
            path: page_paths.url_path.clone(),
            bloom: filter,
            is_sub: self.pj_paths.is_subpage(page_paths.src_path),
            is_private: self.pj_paths.is_private(page_paths.src_path),
        };

        Ok(Some((html, metadata)))
    }
}

/// Markdown のパース設定です。
fn markdown_options() -> Options<'static> {
    let mut options = Options::default();

    let ext = &mut options.extension;
    ext.front_matter_delimiter = Some(FRONT_MATTER_DELIMITER.to_owned());
    ext.table = true;
    ext.strikethrough = true;
    ext.tasklist = true;
    ext.autolink = true;
    ext.footnotes = true;
    ext.description_lists = true;
    ext.superscript = true;
    ext.subscript = true;
    ext.alerts = true;
    ext.math_dollars = true;
    ext.wikilinks_title_after_pipe = true;

    // パスが差し込む HTML を出力するために必要
    options.render.r#unsafe = true;

    options
}

/// ファイルを BufReader で読み、YAML フロントマター部分だけ取り出して title を返します。
/// フロントマター終端の `---` に達した時点で読み込みを止めるため、大きなファイルでも効率的です。
pub fn extract_title_from_path(path: &std::path::Path) -> Result<Option<String>> {
    use std::io::{BufRead as _, BufReader};

    let file = std::fs::File::open(path)?;
    let mut lines = BufReader::new(file).lines();

    match lines.next() {
        Some(Ok(line)) if line == FRONT_MATTER_DELIMITER => {}
        _ => return Ok(None),
    }

    let mut yaml = String::new();
    for line in lines {
        let line = line?;
        if line == FRONT_MATTER_DELIMITER {
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

#[allow(clippy::too_many_arguments)]
pub fn page_html<'a>(
    url_to_root: &Url,
    site_name: &str,
    title: &str,
    create_date: &str,
    last_update_date: &str,
    css_list: impl Iterator<Item = &'a str>,
    js_list: impl Iterator<Item = &'a str>,
    tags: &[String],
    article: &str,
    footer: &str,
) -> String {
    let head = head(url_to_root, css_list, js_list, title);
    let header = header(url_to_root, site_name);
    let tag_elems = tag_elems(tags, url_to_root);
    format!(
        include_asset!("page.html"),
        head = head,
        header = header,
        title = escape_html_text(title),
        tag_elems = tag_elems,
        create_date = create_date,
        last_update_date = last_update_date,
        article = article,
        footer = footer,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn crypto_html<'a>(
    url_to_root: &Url,
    site_name: &str,
    title: &str,
    create_date: &str,
    last_update_date: &str,
    css_list: impl Iterator<Item = &'a str>,
    js_list: impl Iterator<Item = &'a str>,
    tags: &[String],
    encoded_body: &str,
    footer: &str,
) -> String {
    let head = head(url_to_root, css_list, js_list, title);
    let header = header(url_to_root, site_name);
    let tag_elems = tag_elems(tags, url_to_root);
    format!(
        include_asset!("crypto.html"),
        head = head,
        header = header,
        title = escape_html_text(title),
        tag_elems = tag_elems,
        create_date = create_date,
        last_update_date = last_update_date,
        encoded = encoded_body,
        footer = footer,
    )
}
