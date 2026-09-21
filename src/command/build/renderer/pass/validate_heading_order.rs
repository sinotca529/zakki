use anyhow::bail;
use pulldown_cmark::{Event, Tag};

/// ヘッダの順序性を確認します。
/// - `#` がある場合はエラーとします。 フロントマターの title 要素が h1 に相当するためです。
/// - 次のような、上位のヘッダの登場に先だって下位のヘッダが利用された場合はエラーにします。
///   - 階層が飛んでいる場合 (`##` のあとに `####` が来るようなケース)
///   - 浅い階層よりも先に深い階層が現れる場合 (`###` のあとに初めて `##` が来るようなケース)
pub fn validate_heading_order(events: &[Event]) -> anyhow::Result<()> {
    let mut prev_level = 1;

    for e in events {
        let Event::Start(Tag::Heading { level, .. }) = e else {
            continue;
        };
        let level = *level as usize;

        if level == 1 {
            bail!("本文中で '#' (h1) は利用できません。 '##' (h2) から始めてください");
        }

        if level > prev_level + 1 {
            bail!(
                "h{} の直後に h{} が現れました。見出しは 1 段ずつ深くしてください",
                prev_level,
                level
            );
        }

        prev_level = level;
    }

    Ok(())
}
