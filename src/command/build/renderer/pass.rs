mod add_code_caption;
mod adjust_link;
mod assign_header_id;
mod collect_footnotes;
mod collect_monospace_chars;
mod convert_alert;
mod convert_image;
mod convert_math;
mod highlight_code;
mod make_bloom_filter;
mod make_toc;
mod read_front_matter;
mod validate_heading_order;
mod wrap_table;

pub use add_code_caption::add_code_caption;
pub use adjust_link::adjust_link;
pub use assign_header_id::assign_header_id;
pub use collect_footnotes::collect_footnotes;
pub use collect_monospace_chars::collect_monospace_chars;
pub use convert_alert::convert_alert;
pub use convert_image::convert_image;
pub use convert_math::convert_math;
pub use highlight_code::{HighlightRule, highlight_code};
pub use make_bloom_filter::make_bloom_filter;
pub use make_toc::make_toc;
pub use read_front_matter::PageFrontMatter;
pub use read_front_matter::read_front_matter;
pub use validate_heading_order::validate_heading_order;
pub use wrap_table::wrap_table;

use pulldown_cmark::Event;

/// パス実行により配置が必要なアセットの一覧
#[derive(Default)]
pub struct PassAssets {
    /// 追加で読み込む CSS 一覧
    pub css_paths: Vec<String>,
}

/// `events[start]` の開始イベントに対応する終了イベントの位置を返します。
/// 同じ種類の入れ子を数えるため、開始と終了の数で釣り合いを取ります。
fn end_of(events: &[Event], start: usize) -> usize {
    let mut depth = 0usize;

    for (i, e) in events.iter().enumerate().skip(start) {
        match e {
            Event::Start(_) => depth += 1,
            Event::End(_) => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
    }

    unreachable!("開始イベントには必ず終了イベントが対応する")
}

/// 範囲に含まれるテキストを連結します。装飾は落とします。
fn text_of(events: &[Event]) -> String {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Text(t) | Event::Code(t) => Some(t.as_ref()),
            _ => None,
        })
        .collect()
}
