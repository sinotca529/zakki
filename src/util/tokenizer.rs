use itertools::Itertools as _;

/// 文字の種別。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    /// ASCII の英数字
    Ascii,
    /// それ以外の文字 (日本語など)
    Wide,
    /// 区切り文字 (空白・記号)
    Sep,
}

fn class(c: char) -> Class {
    if !c.is_alphanumeric() {
        Class::Sep
    } else if c.is_ascii() {
        Class::Ascii
    } else {
        Class::Wide
    }
}

/// 検索インデックス用にテキストをトークンへ分割します。
///
/// - ASCII 英数字の連続は、そのまま 1 トークンにします (例: `rust` -> `rust`)
/// - それ以外の文字の連続は、文字バイグラムにします (例: `検索語` -> `検索`, `索語`)
/// - 区切り文字をまたぐバイグラムは作りません
///
/// バイグラムを使うのは、日本語には単語境界がなく、分かち書きに頼ると
/// 文書側とクエリ側で切れ目が食い違って取りこぼすためです。
/// (例: 文書が `ブルームフィルタ` を 1 語と切ると `フィルタ` で引けない)
///
pub fn tokenize(text: &str) -> Vec<String> {
    text.chars()
        .chunk_by(|c| class(*c))
        .into_iter()
        .flat_map(|(cls, run)| tokens_of(cls, &run.collect::<Vec<_>>()))
        .collect()
}

fn tokens_of(cls: Class, run: &[char]) -> Vec<String> {
    let lower = |cs: &[char]| cs.iter().collect::<String>().to_lowercase();
    match cls {
        Class::Sep => vec![],
        Class::Ascii => vec![lower(run)],
        // 1 文字しかない run はバイグラムを作れないので、その文字自体をトークンにする
        Class::Wide if run.len() == 1 => vec![lower(run)],
        Class::Wide => run.windows(2).map(lower).collect(),
    }
}

/// クライアント側の実装と突き合わせるための表です。
/// Rust と JS の両方がこれを読み、同じ結果になることを確かめます。
/// 期待値を直接書かないので、片方の名前や置き場所が変わっても直す必要がありません。
#[cfg(test)]
mod golden {
    #[derive(serde::Deserialize)]
    struct Case {
        r#in: String,
        out: Vec<String>,
    }

    #[test]
    fn matches_table() {
        let src = crate::include_testdata!("tokenize.json");
        let cases: Vec<Case> = serde_json::from_str(src).unwrap();
        assert!(!cases.is_empty());
        for c in cases {
            assert_eq!(super::tokenize(&c.r#in), c.out, "入力: {:?}", c.r#in);
        }
    }
}

#[cfg(test)]
mod test {
    use super::tokenize;

    /// 入力ごとの出力は golden の表で見ます。ここに書くのは、
    /// 2 つの入力の関係のように、行ごとの比較で表せないものだけです。
    #[test]
    fn substring_of_a_compound_word_is_searchable() {
        // 分かち書きでは取りこぼしていたケース
        let doc = tokenize("ブルームフィルタ");
        assert!(tokenize("フィルタ").iter().all(|t| doc.contains(t)));
    }
}
