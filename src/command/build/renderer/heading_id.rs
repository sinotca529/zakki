use comrak::adapters::{HeadingAdapter, HeadingMeta};
use comrak::nodes::Sourcepos;
use itertools::Itertools as _;
use std::fmt;
use std::sync::Mutex;

#[derive(Default)]
pub struct NumberedHeadings {
    /// 各レベルの採番カウンタ。
    /// `HeadingAdapter` が `&self` かつ `Sync` を要求するため Mutex に入れています。
    numbering: Mutex<HeaderIdGenerator>,
}

/// 見出しに階層番号の id を振ります (`1`, `1.1`, `1.2`, `2` ...)。
/// ただし、 h1 はページタイトル用なので番号に含めません。
impl HeadingAdapter for NumberedHeadings {
    fn enter(
        &self,
        out: &mut dyn fmt::Write,
        heading: &HeadingMeta,
        _sourcepos: Option<Sourcepos>,
    ) -> fmt::Result {
        let mut numbering = self.numbering.lock().unwrap();

        let lv = heading.level;
        let id = numbering.next_id(lv);
        write!(
            out,
            r#"<h{lv} id="{id}""><span class="section-number">{id}. </span>"#
        )
    }

    fn exit(&self, out: &mut dyn fmt::Write, heading: &HeadingMeta) -> fmt::Result {
        write!(out, "</h{}>", heading.level)
    }
}

#[derive(Default)]
struct HeaderIdGenerator {
    /// セクション番号を管理するカウンタ。
    /// `counter[1]` は h1 に相当。
    /// `counter[0]` は番兵。
    counter: [usize; 7],
}

impl HeaderIdGenerator {
    fn next_id(&mut self, level: u8) -> String {
        let level = level as usize;

        // 下位の階層をリセットしてから、自分の階層を 1 つ進める
        self.counter[(level + 1)..].iter_mut().for_each(|c| *c = 0);
        self.counter[level] += 1;

        self.counter[2..=level]
            .iter()
            .map(|c| c.to_string())
            .join(".")
    }
}
