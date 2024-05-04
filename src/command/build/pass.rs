use latex2mathml::{latex_to_mathml, DisplayStyle};
use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};
use std::cell::RefCell;
use syntastica::language_set::SupportedLanguage;
use syntastica::renderer::*;
use syntastica_parsers::{Lang, LanguageSetImpl};

pub fn adjust_link_to_md(mut event: Event) -> Event {
    if let Event::Start(Tag::Link { dest_url, .. }) = &mut event {
        let is_local_file = !dest_url.starts_with("http://") && !dest_url.starts_with("https://");
        let is_md_file = dest_url.ends_with(".md");

        if is_local_file && is_md_file {
            *dest_url = format!("{}.html", &dest_url[..dest_url.len() - ".md".len()]).into();
        }
    }
    event
}

pub fn convert_math(event: Event) -> Event {
    match event {
        Event::InlineMath(latex) => {
            let mathml = latex_to_mathml(&latex, DisplayStyle::Inline).unwrap();
            Event::InlineHtml(mathml.into())
        }
        Event::DisplayMath(latex) => {
            let mathml = latex_to_mathml(&latex, DisplayStyle::Block).unwrap();
            Event::InlineHtml(mathml.into())
        }
        _ => event,
    }
}

pub fn highlight_code(event: Event) -> Event {
    thread_local! {
        pub static CODE_BLOCK: RefCell<Option<String>> = const { RefCell::new(None) };
    };

    match &event {
        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
            CODE_BLOCK.with(|code_block| {
                *code_block.borrow_mut() = Some(lang.to_string());
            });
            event
        }
        Event::End(TagEnd::CodeBlock) => {
            CODE_BLOCK.with(|code_block| {
                *code_block.borrow_mut() = None;
            });
            event
        }
        Event::Text(t) => {
            let lang = CODE_BLOCK.take().and_then(|l| Lang::for_name(l).ok());
            let Some(lang) = lang else {
                return event;
            };

            let highlights =
                syntastica::Processor::process_once(t, lang, &LanguageSetImpl::new()).unwrap();

            let highlighten = syntastica::render(
                &highlights,
                &mut HtmlRenderer::new(),
                syntastica_themes::one::light(),
            );
            Event::Html(highlighten.into())
        }
        _ => event,
    }
}
