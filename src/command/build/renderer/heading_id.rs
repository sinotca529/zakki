use comrak::adapters::{HeadingAdapter, HeadingMeta};
use comrak::nodes::Sourcepos;
use itertools::Itertools as _;
use std::fmt;
use std::sync::Mutex;

/// 見出しに階層番号の id を振ります (`1`, `1.1`, `1.2`, `2` ...)。
/// ただし、 h1 はページタイトル用なので番号に含めません。
#[derive(Default)]
pub struct NumberedHeadings {
    /// 各レベルの採番カウンタ。
    /// `HeadingAdapter` が `&self` かつ `Sync` を要求するため Mutex に入れています。
    counter: Mutex<[usize; 6]>,
}

impl HeadingAdapter for NumberedHeadings {
    fn enter(
        &self,
        out: &mut dyn fmt::Write,
        heading: &HeadingMeta,
        _sourcepos: Option<Sourcepos>,
    ) -> fmt::Result {
        let level = heading.level as usize;
        let mut counter = self.counter.lock().unwrap();

        // 下位の階層をリセットしてから、自分の階層を 1 つ進める
        counter.iter_mut().skip(level).for_each(|c| *c = 0);
        counter[level - 1] += 1;

        let id = counter[1..]
            .iter()
            .take(level - 1)
            .map(|c| c.to_string())
            .join(".");

        write!(out, "<h{level} id=\"{id}\">")
    }

    fn exit(&self, out: &mut dyn fmt::Write, heading: &HeadingMeta) -> fmt::Result {
        write!(out, "</h{}>", heading.level)
    }
}
