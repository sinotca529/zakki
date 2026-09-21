use crate::command::build::renderer::html_component::SPAN;
use itertools::Itertools as _;
use pulldown_cmark::{Event, Tag};

/// 見出しの id に付ける接頭辞です。
/// 脚注の id は `1`, `2` ... なので、付けないと衝突します。
/// 数字で始まる id は CSS のセレクタや `querySelector` でも扱えません。
pub(super) const SECTION_ID_PREFIX: &str = "s";

/// 見出しに階層番号 (`s1`, `s1.1`, `s1.2`, `s2` ...) の id を振り、番号を画面にも表示します。
/// h1 はページタイトル用なので番号に含めません。
pub fn assign_header_id(events: &mut Vec<Event<'_>>) {
    let mut numbering = HeaderIdGenerator::default();
    let mut out = Vec::with_capacity(events.len());

    for mut e in events.drain(..) {
        let Event::Start(Tag::Heading { level, id, .. }) = &mut e else {
            out.push(e);
            continue;
        };

        let number = numbering.next_id(*level as usize);
        *id = Some(format!("{SECTION_ID_PREFIX}{number}").into());

        let span = SPAN
            .attr("class", "section-number")
            .text(format!("{number}. "));
        out.push(e);
        out.push(Event::InlineHtml(span.into()));
    }

    *events = out;
}

/// 各レベルの採番カウンタ。
#[derive(Default)]
struct HeaderIdGenerator {
    /// セクション番号を管理するカウンタ。
    /// `counter[1]` は h1 に相当。
    /// `counter[0]` は番兵。
    counter: [usize; 7],
}

impl HeaderIdGenerator {
    fn next_id(&mut self, level: usize) -> String {
        // 下位の階層をリセットしてから、自分の階層を 1 つ進める
        self.counter[(level + 1)..].iter_mut().for_each(|c| *c = 0);
        self.counter[level] += 1;

        self.counter[2..=level]
            .iter()
            .map(|c| c.to_string())
            .join(".")
    }
}

#[cfg(test)]
mod test {
    use super::assign_header_id;
    use pulldown_cmark::{Event, Options, Parser, Tag};

    /// 接頭辞がないと、脚注定義の id (`1`, `2` ...) と衝突します。
    #[test]
    fn does_not_collide_with_footnote_ids() {
        let md = "## 見出し\n\n本文です[^1]。\n\n[^1]: 脚注の中身です。\n";
        let mut events: Vec<_> = Parser::new_ext(md, Options::ENABLE_FOOTNOTES).collect();

        assign_header_id(&mut events);

        let ids: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                Event::Start(Tag::Heading { id, .. }) => id.as_deref(),
                _ => None,
            })
            .collect();
        assert_eq!(ids, ["s1"]);
    }
}
