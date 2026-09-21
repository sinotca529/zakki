use super::PassAssets;
use anyhow::Context as _;
use katex::{KatexContext, OutputFormat, Settings};
use pulldown_cmark::Event;
use regex::Regex;
use std::sync::LazyLock;

/// 数式を KaTeX でレンダリング済みの HTML に置き換えます。
pub fn convert_math(events: &mut [Event<'_>], pa: &mut PassAssets) -> anyhow::Result<()> {
    let settings = |display_mode| {
        Settings::builder()
            .output(OutputFormat::Html)
            .display_mode(display_mode)
            .build()
    };
    let display = settings(true);
    let inline = settings(false);

    // 記号表と関数表の構築は重いので、1 度だけ作って使い回す
    static CTX: LazyLock<KatexContext> = LazyLock::new(KatexContext::default);
    let mut math_used = false;

    for e in events.iter_mut() {
        let (latex, settings) = match e {
            Event::InlineMath(latex) => (latex, &inline),
            Event::DisplayMath(latex) => (latex, &display),
            _ => continue,
        };

        let html = katex::render_to_string(&CTX, latex, settings)
            .with_context(|| format!("数式の変換に失敗しました : {latex}"))?;

        *e = Event::InlineHtml(sort_attributes(&html).into());
        math_used = true;
    }

    if math_used {
        pa.css_paths.push("katex/katex.min.css".to_string());
    }

    Ok(())
}

/// タグの属性を名前順に並べ替えます。
///
/// katex-rs は属性を乱数で種を決めるハッシュ表 (`RapidHashMap`) に入れるため、
/// 書き出す順序が実行のたびに変わります。クレート側もテストで並べ替えてから
/// 比べる作りなので、順序は保証されません。描画は変わりませんが、同じ入力から
/// 同じサイトが出るように、ここで揃えます。
fn sort_attributes(html: &str) -> String {
    static TAG: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"<([a-zA-Z0-9]+)((?:\s+[^\s=]+="[^"]*")+)(\s*/?)>"#).unwrap()
    });
    static ATTR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"([^\s=]+)="([^"]*)""#).unwrap());

    TAG.replace_all(html, |caps: &regex::Captures| {
        let mut attrs: Vec<_> = ATTR
            .captures_iter(&caps[2])
            .map(|a| (a[1].to_owned(), a[2].to_owned()))
            .collect();
        attrs.sort_by(|a, b| a.0.cmp(&b.0));

        let attrs: String = attrs
            .iter()
            .map(|(name, value)| format!(r#" {name}="{value}""#))
            .collect();

        format!("<{}{attrs}{}>", &caps[1], caps[3].trim_end())
    })
    .into_owned()
}
