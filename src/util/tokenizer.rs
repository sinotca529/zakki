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
/// - ASCII 英数字の連続は、そのまま 1 つのトークンにします (例: `rust` → `rust`)
/// - それ以外の文字の連続は、文字バイグラムにします (例: `検索語` → `検索`, `索語`)
/// - 区切り文字をまたぐバイグラムは作りません
///
/// バイグラムを使うのは、日本語には単語境界がなく、分かち書きに頼ると
/// 文書側とクエリ側で切れ目が食い違って取りこぼすためです。
/// (例: 文書が `ブルームフィルタ` を 1 語と切ると `フィルタ` で引けない)
///
/// # 注意
/// クライアント側の `asset/script.js` の `tokenize()` と同じ規則である必要があります。
/// 片方だけを変更すると検索がヒットしなくなります。
pub fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut run: Vec<char> = Vec::new();
    let mut cur = Class::Sep;

    // 末尾の区切り文字は、最後の run を掃き出すための番兵
    for c in text.chars().chain(std::iter::once(' ')) {
        let cls = class(c);
        if cls != cur {
            push_run(&mut tokens, &run, cur);
            run.clear();
            cur = cls;
        }
        if cls != Class::Sep {
            run.push(c);
        }
    }

    tokens
}

fn push_run(tokens: &mut Vec<String>, run: &[char], cls: Class) {
    let lower = |cs: &[char]| cs.iter().collect::<String>().to_lowercase();
    match cls {
        Class::Sep => {}
        Class::Ascii => tokens.push(lower(run)),
        // 1 文字しかない run はバイグラムを作れないので、その文字自体をトークンにする
        Class::Wide if run.len() == 1 => tokens.push(lower(run)),
        Class::Wide => run.windows(2).for_each(|w| tokens.push(lower(w))),
    }
}

#[cfg(test)]
mod test {
    use super::tokenize;

    #[test]
    fn ascii_is_kept_as_a_word() {
        assert_eq!(tokenize("Rust BM25"), ["rust", "bm25"]);
    }

    #[test]
    fn japanese_is_split_into_bigrams() {
        assert_eq!(tokenize("検索語"), ["検索", "索語"]);
    }

    #[test]
    fn scripts_are_split_apart() {
        assert_eq!(tokenize("Rust製"), ["rust", "製"]);
    }

    #[test]
    fn bigrams_do_not_cross_separators() {
        // 「た。」「。次」のような無意味なトークンを作らない
        assert_eq!(tokenize("あい。うえ"), ["あい", "うえ"]);
    }

    #[test]
    fn substring_of_a_compound_word_is_searchable() {
        // 分かち書きでは取りこぼしていたケース
        let doc = tokenize("ブルームフィルタ");
        assert!(tokenize("フィルタ").iter().all(|t| doc.contains(t)));
    }

    #[test]
    fn empty_input() {
        assert!(tokenize("").is_empty());
        assert!(tokenize("   、。 ").is_empty());
    }
}
