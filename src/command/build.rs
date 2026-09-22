mod assets;
mod code_font;
mod renderer;

use crate::config::ProjectConfig;
use crate::path::ProjectPaths;
use crate::util;
use crate::util::PathExt as _;
use anyhow::{Context as _, Result, bail};
use itertools::{Either, Itertools as _};
use rayon::prelude::*;
use renderer::extract_title_from_path;
use renderer::{PageMetadata, PageOutput, Renderer};
use std::cmp::Reverse;
use std::collections::{BTreeSet, HashMap};
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
    let (outputs, errors): (Vec<_>, Vec<_>) = files
        .par_iter()
        .map(|p| renderer.render(p).with_context(|| p.display().to_string()))
        .collect::<Vec<_>>()
        .into_iter()
        .partition_map(|r| match r {
            Ok(output) => Either::Left(output),
            Err(e) => Either::Right(e),
        });

    if !errors.is_empty() {
        let list = errors.iter().map(|e| format!("{e:#}")).join("\n");
        bail!("記事を変換できませんでした\n{list}");
    }

    let (mut metas, monospace_chars) = split_outputs(outputs);

    // 新しい順に並べる
    metas.sort_unstable_by_key(|m| Reverse(m.update));

    renderer::render_index(&cfg, pj_paths.build_dir(), &metas)?;

    if let Some(font) = &cfg.code_font {
        warn_private_only_chars(&monospace_chars);
        code_font::output(font, &monospace_chars.all(), pj_paths.build_dir())?;
    }

    output_sitemap(&cfg, &metas, pj_paths.build_dir())?;
    output_metadatas(metas, pj_paths.build_dir())?;

    Ok(())
}

/// 等幅で描く文字を、公開する記事と非公開の記事に分けて集めたものです。
#[derive(Default)]
struct MonospaceChars {
    public: BTreeSet<char>,
    private: BTreeSet<char>,
}

impl MonospaceChars {
    /// サブセットに残す文字です。非公開の記事のぶんも要ります。
    /// 外すと、その記事のコードだけ桁が揃わなくなります。
    fn all(&self) -> BTreeSet<char> {
        self.public.union(&self.private).copied().collect()
    }

    /// 非公開の記事にしか出てこない文字です。
    /// 公開した記事にも出る文字は、フォントに残っていても何も示しません。
    fn private_only(&self) -> BTreeSet<char> {
        self.private.difference(&self.public).copied().collect()
    }
}

/// 記事ごとの変換結果を、メタデータの一覧と、等幅で描く文字に分けます。
fn split_outputs(outputs: Vec<Option<PageOutput>>) -> (Vec<PageMetadata>, MonospaceChars) {
    let mut metas = Vec::with_capacity(outputs.len());
    let mut chars = MonospaceChars::default();

    for output in outputs.into_iter().flatten() {
        let set = match output.meta.is_private {
            true => &mut chars.private,
            false => &mut chars.public,
        };
        set.extend(output.monospace_chars);
        metas.push(output.meta);
    }

    (metas, chars)
}

/// 非公開の記事にしか出てこない文字を伝えます。
///
/// サブセットは全記事を混ぜた集合なので、どの記事に出たかは分かりません。
/// それでも、公開した記事に出てこない文字が残れば、非公開の記事で使ったことは読み取れます。
fn warn_private_only_chars(chars: &MonospaceChars) {
    let leaked = chars.private_only();

    if !leaked.is_empty() {
        eprintln!("{}", private_only_warning(&leaked));
    }
}

/// 並べて見せる字数の上限。超えたぶんは数だけ伝えます。
const SHOWN_CHARS: usize = 40;

fn private_only_warning(leaked: &BTreeSet<char>) -> String {
    let shown = leaked
        .iter()
        .take(SHOWN_CHARS)
        .map(char::to_string)
        .collect::<Vec<_>>()
        .join(" ");

    let rest = match leaked.len().saturating_sub(SHOWN_CHARS) {
        0 => String::new(),
        n => format!(" ほか {n} 字"),
    };

    format!("警告: 公開した記事に出てこない文字が、コード用フォントに残ります : {shown}{rest}")
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
    use super::{MonospaceChars, SHOWN_CHARS, escape_xml_text, private_only_warning};
    use std::collections::BTreeSet;

    fn set(s: &str) -> BTreeSet<char> {
        s.chars().collect()
    }

    /// 公開した記事にも出る文字は、フォントに残っていても何も示しません。
    #[test]
    fn private_only_excludes_chars_that_public_pages_share() {
        let chars = MonospaceChars {
            public: set("abc"),
            private: set("abz"),
        };
        assert_eq!(chars.private_only(), set("z"));
        assert_eq!(chars.all(), set("abcz"));
    }

    /// 字数が多いときに全部並べると、警告が読めなくなります。
    #[test]
    fn warning_shows_the_rest_as_a_count() {
        let few = set("ab");
        assert!(private_only_warning(&few).ends_with(": a b"));

        let many: BTreeSet<char> = ('a'..='z').chain('A'..='Z').collect();
        let message = private_only_warning(&many);
        assert!(message.ends_with(&format!(" ほか {} 字", many.len() - SHOWN_CHARS)));
    }

    /// `&` をそのまま置くと、XML のパーサが実体参照の開始として読みます。
    #[test]
    fn escapes_ampersand() {
        assert_eq!(escape_xml_text("a&copy.html"), "a&amp;copy.html");
        assert_eq!(escape_xml_text("a<b"), "a&lt;b");
    }
}
