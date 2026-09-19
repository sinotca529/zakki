mod html_component;
mod index;
mod page_locs;
mod page_meta;
mod pass;
mod url;

use crate::command::build::renderer::html_component::{
    escape_html_text, footer, head, header, tag_elems,
};
use crate::command::build::renderer::pass::{PageFrontMatter, PassAssets};
use crate::command::build::renderer::url::Url;
use crate::config::ProjectConfig;
use crate::include_asset;
use crate::path::ProjectPaths;
use crate::util::{self, PathExt as _};
use anyhow::{Context as _, Result};
use base64::{Engine, prelude::BASE64_STANDARD};
use pulldown_cmark::{Event, Options, Parser};
use scraper::{Html, Selector};
use serde::Deserialize;
use std::collections::HashMap;
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

    fn render_page(
        &self,
        events: Vec<Event<'_>>,
        front_matter: &PageFrontMatter,
        page_paths: &PageLocs,
        pass_assets: &PassAssets,
    ) -> Result<String> {
        let body = {
            let mut buf = String::new();
            pulldown_cmark::html::push_html(&mut buf, events.into_iter());
            buf
        };

        let css_list = self
            .config
            .css_list
            .iter()
            .map(String::as_str)
            .chain(pass_assets.css_paths.iter().map(String::as_str));

        let js_list = self.config.js_list.iter().map(String::as_str);

        let toc = extract_toc_html(&body);
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

        // Markdown をイベント列に変換
        let mut events: Vec<_> = Parser::new_ext(content, markdown_options()).collect();

        // イベント列に対してパスを適用
        let front_matter = pass::read_front_matter(&mut events)?;

        let mut pass_assets = PassAssets::default();
        pass::validate_heading_order(&events)?;
        pass::assign_header_id(&mut events);
        pass::adjust_link(&mut events, page_paths.src_path, self.title_map)?;

        // 非公開の記事は本文を索引に載せない。bloom filter は語の有無を問い合わせられるため、
        // 暗号化した本文に対して総当たりができてしまう。
        // タイトルは一覧にも metadata.js にも出ているので、索引に入れても変わらない。
        let body = if self.pj_paths.is_private(page_paths.src_path) {
            &[][..]
        } else {
            &events[..]
        };
        let filter = pass::make_bloom_filter(body, &front_matter.title, self.config.search_fp);

        pass::convert_image(&mut events);
        pass::add_code_caption(&mut events);
        pass::highlight_code(&mut events, &front_matter.highlights);
        pass::convert_math(&mut events, &mut pass_assets)?;
        pass::wrap_table(&mut events);
        pass::convert_alert(&mut events);
        pass::collect_footnotes(&mut events);

        // イベント列を HTML に変換
        let html = self.render_page(events, &front_matter, page_paths, &pass_assets)?;

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
fn markdown_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_DEFINITION_LIST
        | Options::ENABLE_SUPERSCRIPT
        | Options::ENABLE_SUBSCRIPT
        | Options::ENABLE_GFM
        | Options::ENABLE_MATH
        | Options::ENABLE_WIKILINKS
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
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
            // 目次はナビゲーションなので、見出しの <code> や <strong> は落としてテキストだけにする。
            let inner: String = el.text().collect();
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
        (prev_level..*level).for_each(|_| html.push("<ol><li>".to_string()));
        // 階層を上る
        (*level..prev_level).for_each(|_| html.push("</li></ol>".to_string()));
        // 次の要素へ
        if *level <= prev_level {
            html.push("</li><li>".to_string());
        }
        // リンクを追加
        html.push(format!(
            "<a href=\"#{}\">{}</a>",
            id,
            escape_html_text(inner)
        ));
        prev_level = *level;
    }
    // 閉じる
    (0..prev_level).for_each(|_| html.push("</li></ol>".to_string()));

    format!(
        "<details id=\"toc\"><summary>目次</summary>{}</details>",
        html.join("")
    )
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
