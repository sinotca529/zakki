use indoc::indoc;
use latex2mathml::{latex_to_mathml, DisplayStyle};
use pulldown_cmark::{CodeBlockKind, Event, Options, Tag, TagEnd};
use syntastica::language_set::SupportedLanguage;
use syntastica::renderer::*;
use syntastica_parsers::{Lang, LanguageSetImpl};

use std::{fs::File, io::Read, path::Path};

fn main() {
    let path = std::env::args().nth(1).unwrap();
    println!("{}", convert(path));
}

fn convert_body(md: &str) -> String {
    let parser = pulldown_cmark::Parser::new_ext(&md, Options::all());

    let mut code_block = None;

    let parser = parser.map(|e| match e {
        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
            code_block = Some(lang.clone());
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang)))
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
            let lang = code_block.take().map(|l| Lang::for_name(&l).ok()).flatten();
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

fn convert(path: impl AsRef<Path>) -> String {
    fn inner(path: &Path) -> String {
        let mut input = String::new();
        File::open(path)
            .unwrap()
            .read_to_string(&mut input)
            .unwrap();

        let html_begin = indoc! {r#"
            <!DOCTYPE html>
            <html lang="ja">
            <head>
            <meta charset="UTF-8">
            </head>
            <body>
        "#};

        let html_end = indoc! {"
            </body>
            </html>
        "};

        let body = convert_body(&input);

        format!("{html_begin}{body}{html_end}")
    }
    inner(path.as_ref())
}
