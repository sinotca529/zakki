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
    let mut tokens = Vec::new();
    let mut chars = text.char_indices().peekable();

    while let Some((start, c)) = chars.next() {
        let cls = class(c);
        let mut end = start + c.len_utf8();

        // 同じ種別が続く間を 1 つの run にする
        while let Some(&(i, next)) = chars.peek() {
            if class(next) != cls {
                break;
            }
            end = i + next.len_utf8();
            chars.next();
        }

        push_tokens(&mut tokens, cls, &text[start..end]);
    }

    tokens
}

fn push_tokens(tokens: &mut Vec<String>, cls: Class, run: &str) {
    match cls {
        Class::Sep => {}
        Class::Ascii => tokens.push(run.to_lowercase()),
        Class::Wide => {
            let mut prev = None;
            for (i, c) in run.char_indices() {
                if let Some(prev) = prev {
                    tokens.push(run[prev..i + c.len_utf8()].to_lowercase());
                }
                prev = Some(i);
            }

            // 1 文字しかない run はバイグラムを作れないので、その文字自体をトークンにする
            if prev == Some(0) {
                tokens.push(run.to_lowercase());
            }
        }
    }
}

/// クライアント側の実装と突き合わせるための表です。
/// Rust と JS の両方がこれを読み、同じ結果になることを確かめます。
/// 期待値を直接書かないので、片方の名前や置き場所が変わっても直す必要がありません。
#[cfg(test)]
mod test_vector {
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

    /// 入力ごとの出力は test_vector の表で見ます。ここに書くのは、
    /// 2 つの入力の関係のように、行ごとの比較で表せないものだけです。
    #[test]
    fn substring_of_a_compound_word_is_searchable() {
        // 分かち書きでは取りこぼしていたケース
        let doc = tokenize("ブルームフィルタ");
        assert!(tokenize("フィルタ").iter().all(|t| doc.contains(t)));
    }
}
