use std::borrow::Cow;

struct TocItem {
    title: String,
    id: String,
    level: usize,
}

pub struct Toc {
    items: Vec<TocItem>,
}

impl Toc {
    pub fn to_html(&self) -> String {
        let mut html = Vec::<Cow<'_, str>>::new();

        //let mut html = String::new();
        let mut prev_level = 0;

        for item in &self.items {
            // 階層を下る
            (prev_level..item.level).for_each(|_| html.push("<ul><li>".into()));

            // 階層を上る
            (item.level..prev_level).for_each(|_| html.push("</li></ul>".into()));

            // 次の要素へ
            if item.level <= prev_level {
                html.push("</li><li>".into());
            }

            // リンクを追加
            html.push(format!("<a href=\"#{}\">{}</a>", item.id, item.title).into());

            prev_level = item.level;
        }

        // 閉じる
        (0..prev_level).for_each(|_| html.push("</li></ul>".into()));

        html.join("")
    }
}

pub struct TocBuilder {
    items: Vec<TocItem>,
}

impl TocBuilder {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add_item(&mut self, title: String, id: String, level: usize) {
        self.items.push(TocItem { title, id, level });
    }

    pub fn build(self) -> Toc {
        Toc { items: self.items }
    }
}
