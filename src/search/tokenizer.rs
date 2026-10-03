use std::borrow::Cow;

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
pub fn tokenize(text: &str) -> Vec<Cow<'_, str>> {
    let mut tokens = Vec::new();
    let mut chars = text.char_indices();

    let Some((_, first)) = chars.next() else {
        return tokens;
    };
    let mut start = 0;
    let mut cls = class(first);

    // 同じ種別が続く間を 1 つの run にする。種別は 1 文字につき 1 度だけ調べる
    for (i, c) in chars {
        let next = class(c);
        if next != cls {
            push_tokens(&mut tokens, cls, &text[start..i]);
            start = i;
            cls = next;
        }
    }
    push_tokens(&mut tokens, cls, &text[start..]);

    tokens
}

fn push_tokens<'a>(tokens: &mut Vec<Cow<'a, str>>, cls: Class, run: &'a str) {
    match cls {
        Class::Sep => {}
        Class::Ascii => tokens.push(lower(run)),
        Class::Wide => push_bigrams(tokens, run),
    }
}

/// 文字バイグラムを積みます。
///
/// 小文字にする必要があるかは run 単位で調べます。日本語のように変わらない場合は、
/// 部分文字列を借りたまま積めます。
fn push_bigrams<'a>(tokens: &mut Vec<Cow<'a, str>>, run: &'a str) {
    match lower(run) {
        Cow::Borrowed(run) => each_bigram(run, |b| tokens.push(Cow::Borrowed(b))),
        Cow::Owned(lowered) => each_bigram(&lowered, |b| tokens.push(Cow::Owned(b.to_owned()))),
    }
}

/// 連続する 2 文字を順に渡します。1 文字しかない場合はその文字を渡します。
fn each_bigram<'a>(run: &'a str, mut f: impl FnMut(&'a str)) {
    let mut prev = None;

    for (i, c) in run.char_indices() {
        if let Some(prev) = prev {
            f(&run[prev..i + c.len_utf8()]);
        }
        prev = Some(i);
    }

    if prev == Some(0) {
        f(run);
    }
}

/// 小文字にします。変える文字がなければ、借りたまま返します。
fn lower(s: &str) -> Cow<'_, str> {
    match s.chars().any(|c| c.to_lowercase().next() != Some(c)) {
        true => Cow::Owned(s.to_lowercase()),
        false => Cow::Borrowed(s),
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
