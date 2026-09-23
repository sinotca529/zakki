use crate::command::build::code_font;
use crate::command::build::renderer::html_component::footer;
use crate::command::build::renderer::pass::{self, PageFrontMatter, PassAssets};
use crate::command::build::renderer::{PageLocs, PageMetadata, crypto_html, page_html};
use crate::config::ProjectConfig;
use crate::crypt::encode_with_password;
use crate::path::ProjectPaths;
use crate::util::{self, PathExt as _};
use anyhow::{Context as _, Result};
use base64::{Engine, prelude::BASE64_STANDARD};
use pulldown_cmark::{Event, Options, Parser};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

pub struct PageRenderer<'a> {
    config: &'a ProjectConfig,
    title_map: &'a HashMap<PathBuf, String>,
    pj_paths: &'a ProjectPaths,
    render_draft: bool,
}

impl<'a> PageRenderer<'a> {
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
        let Some((html, output)) = self.md_to_html(&content, &page_paths)? else {
            return Ok(None);
        };

        util::write_file(page_paths.build_path, html)?;

        Ok(Some(output))
    }

    fn render_page(
        &self,
        events: Vec<Event<'_>>,
        toc: &str,
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
            .code_font
            .iter()
            .map(|_| code_font::CSS_PATH)
            .chain(self.config.css_list.iter().map(String::as_str))
            .chain(pass_assets.css_paths.iter().map(String::as_str));

        let js_list = self.config.js_list.iter().map(String::as_str);

        let article = format!("{toc}{body}");

        let is_private = self.pj_paths.is_private(page_paths.src_path);
        let html = if is_private {
            let password = front_matter
                .password
                .as_ref()
                .or(self.config.password.as_ref())
                .context(
                    "private フォルダ内の md ファイルを暗号化するにはパスワード設定が必要です",
                )?;

            let cypher = encode_with_password(password, article.as_bytes());
            let encoded = BASE64_STANDARD.encode(cypher);

            crypto_html(
                &page_paths.url_to_root,
                &self.config.site_name,
                &front_matter.title,
                front_matter.create_date,
                front_matter.last_update_date(),
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
                front_matter.create_date,
                front_matter.last_update_date(),
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

        // ここから下はイベント列を順に書き換えます。並びには次の決まりがあります。
        // 入れ替えてもコンパイルは通り、出力だけが変わります。
        //
        // - read_front_matter が最初。ヘッダのイベントを取り除くので、後のパスは本文だけを見ます。
        //   パスワードが検索の索引に載らないのも、この順序によります
        // - collect_monospace_chars は add_code_caption より前。キャプションは info string にあり、
        //   add_code_caption がそこから取り出して消します
        // - assign_header_id は make_toc より前。make_toc は見出しの id を読み、無ければ落ちます
        // - highlight_code は make_bloom_filter より前。区切り文字はここで取り除かれるので、
        //   索引に載りません
        //
        // ほかの組み合わせでは、触るイベントが重なりません。並びを変えるときは、
        // 生成したサイトを変更前と比べてください。
        let front_matter = pass::read_front_matter(&mut events)?;

        // フォントの指定がなければサブセットを作らないので、文字も集めません。
        let monospace_chars = match self.config.code_font {
            Some(_) => pass::collect_monospace_chars(&events),
            None => BTreeSet::new(),
        };

        let mut pass_assets = PassAssets::default();
        pass::validate_heading_order(&events)?;
        pass::insert_child_list(&mut events, page_paths.src_path, self.title_map);
        pass::assign_header_id(&mut events);
        pass::adjust_link(&mut events, page_paths.src_path, self.title_map)?;
        pass::convert_image(&mut events);
        pass::add_code_caption(&mut events);
        pass::highlight_code(&mut events, &front_matter.highlights);
        pass::convert_math(&mut events, &mut pass_assets)?;
        pass::wrap_table(&mut events);
        pass::convert_alert(&mut events);
        pass::collect_footnotes(&mut events);

        // 目次と索引は本文が確定してから作る。
        let toc = pass::make_toc(&events);

        // 非公開の記事は本文を渡さない。bloom filter は語の有無を問い合わせられるため、
        // 暗号化した本文に対して総当たりができてしまう。
        // タイトルは一覧にも metadata.js にも出ているので、索引に入れても変わらない。
        let body = if self.pj_paths.is_private(page_paths.src_path) {
            &[][..]
        } else {
            &events[..]
        };
        let filter = pass::make_bloom_filter(body, &front_matter.title, self.config.search_fp);

        // イベント列を HTML に変換
        let html = self.render_page(events, &toc, &front_matter, page_paths, &pass_assets)?;

        let metadata = PageMetadata {
            create: front_matter.create_date,
            update: front_matter.last_update_date(),
            tags: front_matter.tags,
            title: front_matter.title,
            path: page_paths.url_path.clone(),
            bloom: filter,
            is_sub: self.pj_paths.is_subpage(page_paths.src_path),
            is_group: self.pj_paths.is_group(page_paths.src_path),
            is_private: self.pj_paths.is_private(page_paths.src_path),
            monospace_chars,
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
