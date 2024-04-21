use indoc::{formatdoc, indoc};
use latex2mathml::{latex_to_mathml, DisplayStyle};
use pulldown_cmark::{CodeBlockKind, Event, Options, Tag, TagEnd};
use syntastica::language_set::SupportedLanguage;
use syntastica::renderer::*;
use syntastica_parsers::{Lang, LanguageSetImpl};

fn convert_body(md: &str) -> String {
    let parser = pulldown_cmark::Parser::new_ext(md, Options::all());

    let mut code_block = None;

    let parser = parser.map(|e| match e {
        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
            code_block = Some(lang.clone());
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang)))
        }
        Event::Start(Tag::Link {
            link_type,
            mut dest_url,
            title,
            id,
        }) => {
            if !dest_url.starts_with("http://")
                && !dest_url.starts_with("https://")
                && dest_url.ends_with(".md")
            {
                dest_url = format!("{}.html", &dest_url[..dest_url.len() - 3]).into();
            }
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            })
        }
        Event::End(TagEnd::CodeBlock) => {
            code_block = None;
            Event::End(TagEnd::CodeBlock)
        }
        Event::InlineMath(latex) => {
            let mathml = latex_to_mathml(&latex, DisplayStyle::Inline).unwrap();
            Event::InlineHtml(mathml.into())
        }
        Event::DisplayMath(latex) => {
            let mathml = latex_to_mathml(&latex, DisplayStyle::Block).unwrap();
            Event::InlineHtml(mathml.into())
        }
        Event::Text(t) => {
            let lang = code_block.take().and_then(|l| Lang::for_name(&l).ok());
            let Some(lang) = lang else {
                return Event::Text(t);
            };

            let highlights =
                syntastica::Processor::process_once(&t, lang, &LanguageSetImpl::new()).unwrap();

            let highlighten = syntastica::render(
                &highlights,
                &mut HtmlRenderer::new(),
                syntastica_themes::one::light(),
            );
            Event::Html(highlighten.into())
        }
        _ => e,
    });

    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    html
}

pub fn md_to_html(md_content: &str, rel_path_to_css: &str) -> String {
    let html_begin = formatdoc! {r#"
            <!DOCTYPE html>
            <html lang="ja">
            <head>
            <meta charset="UTF-8">
            <link rel="stylesheet" href="{path_to_css}">
            </head>
            <body>
        "#,
        path_to_css = rel_path_to_css
    };

    let html_end = indoc! {"
        </body>
        </html>
    "};

    let body = convert_body(md_content);

    format!("{html_begin}{body}{html_end}")
}
