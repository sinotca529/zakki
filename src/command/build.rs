mod assets;
mod renderer;

use crate::config::ProjectConfig;
use crate::path::ProjectPaths;
use crate::util;
use crate::util::PathExt as _;
use anyhow::{Context as _, Result, bail};
use itertools::{Either, Itertools as _};
use rayon::prelude::*;
use renderer::extract_title_from_path;
use renderer::{PageMetadata, Renderer};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn build(pj_paths: &ProjectPaths, render_draft: bool) -> Result<()> {
    let cfg = ProjectConfig::load(pj_paths.config_path())?;

    super::clean::clean(pj_paths)?;

    let files = pj_paths
        .src_dir()
        .descendants_file_paths()
        .with_context(|| {
            format!(
                "{} またはその配下のファイルを読めません",
                pj_paths.src_dir().display()
            )
        })?;

    // Wikilink のタイトルを書くため、全記事のタイトルを先んじて取得する。
    let title_map = collect_titles(&files)?;
    let renderer = Renderer::new(&cfg, &title_map, pj_paths, render_draft);

    assets::copy_assets(pj_paths.build_dir())?;

    // Result のまま集める。1 件目のエラーで打ち切ると、残りの記事の問題が分からない。
    let (metas, errors): (Vec<_>, Vec<_>) = files
        .par_iter()
        .map(|p| renderer.render(p).with_context(|| p.display().to_string()))
        .collect::<Vec<_>>()
        .into_iter()
        .partition_map(|r| match r {
            Ok(meta) => Either::Left(meta),
            Err(e) => Either::Right(e),
        });

    if !errors.is_empty() {
        let list = errors.iter().map(|e| format!("{e:#}")).join("\n");
        bail!("記事を変換できませんでした\n{list}");
    }

    let mut metas: Vec<_> = metas.into_iter().flatten().collect();

    // 新しい順に並べる
    metas.sort_unstable_by_key(|m| Reverse(m.update));

    renderer::render_index(&cfg, pj_paths.build_dir(), &metas)?;

    output_sitemap(&cfg, &metas, pj_paths.build_dir())?;
    output_metadatas(metas, pj_paths.build_dir())?;

    Ok(())
}

fn collect_titles(files: &[PathBuf]) -> Result<HashMap<PathBuf, String>> {
    let mut map = HashMap::new();
    for path in files {
        if !path.extension_is("md") {
            continue;
        }
        let title = extract_title_from_path(path).with_context(|| path.display().to_string())?;
        if let Some(title) = title {
            map.insert(path.normalized(), title);
        }
    }
    Ok(map)
}

/// XML の要素内容として使えるようエスケープします。
///
/// ファイル名に `&` が入ると実体参照の開始として読まれ、文書全体が整形式でなくなります。
/// `Url` は `&` をパーセントエンコードしないため、ここで実体参照に置き換えます。
fn escape_xml_text(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;")
}

fn output_sitemap(cfg: &ProjectConfig, metas: &[PageMetadata], build_dir: &Path) -> Result<()> {
    let Some(pub_url) = cfg.publish_url.as_ref() else {
        return Ok(());
    };

    let sitemap_path = build_dir.join("sitemap.xml");
    util::write_file(sitemap_path, sitemap_xml(pub_url, metas))?;

    Ok(())
}

/// 公開する記事から sitemap.xml の中身を作ります。
fn sitemap_xml(publish_url: &str, metas: &[PageMetadata]) -> String {
    let publish_url = publish_url.trim_end_matches('/');

    let urls: String = metas
        .iter()
        .filter(|m| !m.is_private)
        .map(|m| {
            let loc = escape_xml_text(&format!("{publish_url}/{}", m.path));
            format!(
                "  <url><loc>{loc}</loc><lastmod>{}</lastmod></url>\n",
                m.update
            )
        })
        .collect();

    format!(
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            "<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
            "{}",
            "</urlset>\n",
        ),
        urls
    )
}

// METADATA と BLOOM_FILTER を書き出します。
// どちらも metas を先頭から順に並べるため、添字が同じ要素が同じ記事を指します。
fn output_metadatas(metas: Vec<PageMetadata>, build_dir: &Path) -> Result<()> {
    // メタデータの書き出し
    let json = serde_json::to_string(&metas)?;
    let js = format!("const METADATA={json}");
    let metadata_path = build_dir.join("metadata.js");
    util::write_file(metadata_path, js)?;

    // Bloom filter の書き出し
    let blooms: Vec<_> = metas.iter().map(|o| &o.bloom).collect();
    let json = serde_json::to_string(&blooms)?;
    let js = format!("const BLOOM_FILTER={json}");
    let bloom_filter_path = build_dir.join("bloom_filter.js");
    util::write_file(bloom_filter_path, js)?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::escape_xml_text;

    /// `&` をそのまま置くと、XML のパーサが実体参照の開始として読みます。
    #[test]
    fn escapes_ampersand() {
        assert_eq!(escape_xml_text("a&copy.html"), "a&amp;copy.html");
        assert_eq!(escape_xml_text("a<b"), "a&lt;b");
    }
}
