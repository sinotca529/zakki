use super::end_of;
use pulldown_cmark::{Event, Tag};

/// 脚注の定義を本文の末尾へ移します。
///
/// pulldown-cmark は定義を書かれた位置にそのまま描画します。
/// 読むときは本文の流れを切るので、まとめて最後に置きます。
pub fn collect_footnotes(events: &mut Vec<Event<'_>>) {
    let mut body = Vec::with_capacity(events.len());
    let mut definitions = Vec::new();
    let mut i = 0;

    while i < events.len() {
        if matches!(events[i], Event::Start(Tag::FootnoteDefinition(_))) {
            let end = end_of(events, i);
            definitions.extend(events[i..=end].iter().cloned());
            i = end + 1;
            continue;
        }

        body.push(events[i].clone());
        i += 1;
    }

    body.extend(definitions);
    *events = body;
}
