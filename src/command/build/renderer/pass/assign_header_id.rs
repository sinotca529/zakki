use itertools::Itertools as _;
use pulldown_cmark::{Event, Tag};

/// 見出しに階層番号 (`1`, `1.1`, `1.2`, `2` ...) の id を振り、番号を画面にも表示します。
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
        *id = Some(number.clone().into());

        let span = format!(r#"<span class="section-number">{number}. </span>"#);
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
