use super::html::PageMetadataBuilder;
use latex2mathml::{latex_to_mathml, DisplayStyle};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, MetadataBlockKind, Tag, TagEnd};
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
            CODE_BLOCK.with(|code_block| *code_block.borrow_mut() = Some(lang.to_string()));
            event
        }
        Event::End(TagEnd::CodeBlock) => {
            CODE_BLOCK.with(|code_block| *code_block.borrow_mut() = None);
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

pub fn get_h1<'a>(event: Event<'a>, title: &mut String) -> Event<'a> {
    thread_local! {
        pub static H1: RefCell<bool> = const { RefCell::new(false) };
    };

    match &event {
        Event::Start(Tag::Heading { level, .. }) if level == &HeadingLevel::H1 => {
            H1.with(|code_block| *code_block.borrow_mut() = true);
            title.clear();
        }
        Event::End(TagEnd::CodeBlock) => {
            H1.with(|code_block| *code_block.borrow_mut() = false);
        }
        Event::Text(t) => {
            if H1.take() {
                title.push_str(t.as_ref());
            }
        }
        _ => {}
    }

    event
}

pub fn read_yaml_header<'a>(event: Event<'a>, metadata: &mut PageMetadataBuilder) -> Event<'a> {
    thread_local! {
        pub static METADATA_BLOCK: RefCell<bool> = const { RefCell::new(false) };
    };

    match &event {
        Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle)) => {
            METADATA_BLOCK.with(|mb| *mb.borrow_mut() = true);
        }
        Event::End(TagEnd::MetadataBlock(MetadataBlockKind::YamlStyle)) => {
            METADATA_BLOCK.with(|mb| *mb.borrow_mut() = false);
        }
        Event::Text(yaml) => {
            if METADATA_BLOCK.take() {
                metadata.read_yaml(yaml);
            }
        }
        _ => {}
    }

    event
}
