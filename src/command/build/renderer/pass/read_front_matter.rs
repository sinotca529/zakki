use crate::command::build::renderer::pass::HighlightRule;
use crate::util::Date;
use anyhow::Context as _;
use pulldown_cmark::{Event, MetadataBlockKind::YamlStyle, Tag, TagEnd};
use serde::Deserialize;

/// YAML フロントマターを読み取り、イベント列から取り除きます。
///
/// 取り除くのは、後続のパスがヘッダの中身を本文として扱わないようにするためです。
/// パスワードが検索の索引に載るような間違いを、構造として防ぎます。
pub fn read_front_matter(events: &mut Vec<Event>) -> anyhow::Result<PageFrontMatter> {
    let start = events
        .iter()
        .position(|e| matches!(e, Event::Start(Tag::MetadataBlock(YamlStyle))));

    let Some(start) = start else {
        anyhow::bail!("記事は yaml ヘッダーで始めてください")
    };

    let end = events[start..]
        .iter()
        .position(|e| matches!(e, Event::End(TagEnd::MetadataBlock(YamlStyle))))
        .expect("開始イベントには必ず終了イベントが対応する")
        + start;

    let body: String = events[start..=end]
        .iter()
        .filter_map(|e| match e {
            Event::Text(t) => Some(t.as_ref()),
            _ => None,
        })
        .collect();

    let front_matter = serde_yaml::from_str::<PageFrontMatter>(&body)
        .context("yaml ヘッダーのデコードに失敗しました")?;

    events.drain(start..=end);

    Ok(front_matter)
}

/// `Option` のフィールドに `#[serde(default)]` は付けません。
/// serde が省略時に `None` を入れるためです。付けるのは `Vec` のように、
/// 省略時の値を serde が決められない型だけにします。
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct PageFrontMatter {
    /// 記事の作成日
    #[serde(rename = "create")]
    pub create_date: Date,

    /// 記事の最終更新日
    /// 省略できる。読むときは `last_update_date` を使う
    #[serde(rename = "update")]
    update_date: Option<Date>,

    /// 記事のタイトル
    pub title: String,

    /// 記事につけられたタグ
    #[serde(default)]
    pub tags: Vec<String>,

    /// 暗号化時のパスワード
    pub password: Option<String>,

    /// コードハイライトのルール
    pub highlights: Option<Vec<HighlightRule>>,
}

impl PageFrontMatter {
    /// 記事の最終更新日です。
    ///
    /// `update` を省略した記事では作成日を返します。書いたその日に出す記事では
    /// 2 つが同じ日付になるため、改訂したときだけ書けば済むようにしています。
    pub fn last_update_date(&self) -> Date {
        self.update_date.unwrap_or(self.create_date)
    }
}

#[cfg(test)]
mod test {
    use super::read_front_matter;
    use pulldown_cmark::{Event, Options, Parser, Tag};

    /// ヘッダを残すと、パスワードが検索の索引に載ります。
    #[test]
    fn removes_the_header_from_the_events() {
        let md = "---\ntitle: 題\ncreate: 2025-01-01\nupdate: 2025-01-01\npassword: ひみつ\n---\n\n本文です。\n";
        let options = Options::ENABLE_YAML_STYLE_METADATA_BLOCKS;
        let mut events: Vec<_> = Parser::new_ext(md, options).collect();

        let front_matter = read_front_matter(&mut events).unwrap();
        assert_eq!(front_matter.title, "題");

        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::Start(Tag::MetadataBlock(_))))
        );
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::Text(t) if t.contains("ひみつ")))
        );
    }

    fn front_matter_of(md: &str) -> super::PageFrontMatter {
        let options = Options::ENABLE_YAML_STYLE_METADATA_BLOCKS;
        let mut events: Vec<_> = Parser::new_ext(md, options).collect();
        read_front_matter(&mut events).unwrap()
    }

    /// 書いたその日に出す記事で、同じ日付を 2 回書かずに済むようにしています。
    #[test]
    fn update_falls_back_to_create() {
        let md = "---\ntitle: 題\ncreate: 2025-01-01\n---\n\n本文です。\n";
        let front_matter = front_matter_of(md);
        assert_eq!(front_matter.last_update_date(), front_matter.create_date);
    }

    /// 書いてあれば、そちらを使います。
    #[test]
    fn update_wins_when_written() {
        let md = "---\ntitle: 題\ncreate: 2025-01-01\nupdate: 2025-03-04\n---\n\n本文です。\n";
        let front_matter = front_matter_of(md);
        assert_eq!(front_matter.last_update_date().to_string(), "2025-03-04");
    }
}
