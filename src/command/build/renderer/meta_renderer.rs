use std::collections::BTreeSet;

use crate::{
    command::build::{
        code_font,
        renderer::{self, PageMetadata},
    },
    config::ProjectConfig,
    path::ProjectPaths,
    util,
};

pub struct MetaRenderer<'a> {
    cfg: &'a ProjectConfig,
    pj_paths: &'a ProjectPaths,
}

impl<'a> MetaRenderer<'a> {
    pub fn new(cfg: &'a ProjectConfig, pj_paths: &'a ProjectPaths) -> Self {
        Self { cfg, pj_paths }
    }

    pub fn render(&self, metas: &[PageMetadata]) -> anyhow::Result<()> {
        self.output_monospace_font_file(metas)?;
        renderer::render_index(self.cfg, self.pj_paths.build_dir(), metas)?;
        self.output_sitemap(metas)?;
        self.output_pagemeta(metas)?;
        self.output_bloom_index(metas)?;
        Ok(())
    }

    fn output_monospace_font_file(&self, metas: &[PageMetadata]) -> anyhow::Result<()> {
        let monospace_chars = self.collect_monospace_chars(metas);
        if let Some(font) = &self.cfg.code_font {
            Self::warn_private_only_chars(&monospace_chars);
            code_font::output(font, &monospace_chars.all(), self.pj_paths.build_dir())?;
        }
        Ok(())
    }

    /// 記事ごとの変換結果を、メタデータの一覧と、等幅で描く文字に分けます。
    fn collect_monospace_chars(&self, metas: &[PageMetadata]) -> MonospaceChars {
        let mut chars = MonospaceChars::default();

        for output in metas.iter() {
            let set = match output.is_private {
                true => &mut chars.private,
                false => &mut chars.public,
            };
            set.extend(output.monospace_chars.iter().copied());
        }
        chars
    }

    /// 非公開の記事にしか出てこない文字を伝えます。
    ///
    /// サブセットは全記事を混ぜた集合なので、どの記事に出たかは分かりません。
    /// それでも、公開した記事に出てこない文字が残れば、非公開の記事で使ったことは読み取れます。
    fn warn_private_only_chars(chars: &MonospaceChars) {
        let leaked = chars.private_only();

        if !leaked.is_empty() {
            eprintln!("{}", Self::private_only_warning(&leaked));
        }
    }

    /// 並べて見せる字数の上限。超えたぶんは数だけ伝えます。
    const SHOWN_CHARS: usize = 40;

    fn private_only_warning(leaked: &BTreeSet<char>) -> String {
        let shown = leaked
            .iter()
            .take(Self::SHOWN_CHARS)
            .map(char::to_string)
            .collect::<Vec<_>>()
            .join(" ");

        let rest = match leaked.len().saturating_sub(Self::SHOWN_CHARS) {
            0 => String::new(),
            n => format!(" ほか {n} 字"),
        };

        format!("警告: 公開した記事に出てこない文字が、コード用フォントに残ります : {shown}{rest}")
    }

    fn output_sitemap(&self, metas: &[PageMetadata]) -> anyhow::Result<()> {
        let Some(pub_url) = self.cfg.publish_url.as_ref() else {
            return Ok(());
        };

        let sitemap_path = self.pj_paths.build_dir().join("sitemap.xml");
        util::write_file(sitemap_path, Self::sitemap_xml(pub_url, metas))?;

        Ok(())
    }

    /// 公開する記事から sitemap.xml の中身を作ります。
    fn sitemap_xml(publish_url: &str, metas: &[PageMetadata]) -> String {
        let publish_url = publish_url.trim_end_matches('/');

        let urls: String = metas
            .iter()
            .filter(|m| !m.is_private)
            .map(|m| {
                let loc = Self::escape_xml_text(&format!("{publish_url}/{}", m.path));
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

    // METADATA を書き出します
    // どちらも metas を先頭から順に並べるため、添字が同じ要素が同じ記事を指します。
    fn output_pagemeta(&self, metas: &[PageMetadata]) -> anyhow::Result<()> {
        let json = serde_json::to_string(metas)?;
        let js = format!("const METADATA={json}");
        let metadata_path = self.pj_paths.build_dir().join("metadata.js");
        util::write_file(metadata_path, js)?;
        Ok(())
    }

    // BLOOM_FILTER を書き出します。
    fn output_bloom_index(&self, metas: &[PageMetadata]) -> anyhow::Result<()> {
        let blooms: Vec<_> = metas.iter().map(|o| &o.bloom).collect();
        let json = serde_json::to_string(&blooms)?;
        let js = format!("const BLOOM_FILTER={json}");
        let bloom_filter_path = self.pj_paths.build_dir().join("bloom_filter.js");
        util::write_file(bloom_filter_path, js)?;
        Ok(())
    }

    /// XML の要素内容として使えるようエスケープします。
    ///
    /// ファイル名に `&` が入ると実体参照の開始として読まれ、文書全体が整形式でなくなります。
    /// `Url` は `&` をパーセントエンコードしないため、ここで実体参照に置き換えます。
    fn escape_xml_text(text: &str) -> String {
        text.replace('&', "&amp;").replace('<', "&lt;")
    }
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
