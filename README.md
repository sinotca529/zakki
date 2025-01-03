# zakki

`zakki` は静的サイトジェネレーターです。<br>

Markdown から HTML への変換には [`pulldown-cmark`](https://docs.rs/pulldown-cmark/latest/pulldown_cmark/) を使っています。<br>
そのため、 `zakki` は `pulldown-cmark` が扱えない構文を扱えません。

## 主な機能

- サイト生成時の数式描画
- 下書きを HTML に変換しない機能
- コードハイライト
- ページの暗号化
- サイト内検索
- `file://` プロトコルでの動作

## 使い方

- `zakki init` コマンドでひな形を作成します。
- `zakki build` コマンドでサイトを生成します (下書きは変換されません)。
- `zakki build -d` コマンドでサイトを生成します (下書きも変換されます)。

その他のコマンドは `zakki --help` で確認できます。

### 設定ファイル

設定は `zakki.toml` に記述します。

```toml
site_name = "(必須) サイト名を指定します。"
password = "(任意) 暗号化用のパスワードを指定します。"
footer = "(任意) フッターの内容を HTML で指定します。"
search_fp = "(任意) サイト内検索の偽陽性率を指定します。デフォルトは 0.0001 (0.01%) です。"
js_list = ["(任意) 追加する javascript ファイルを指定します。"]
css_list = ["(任意) 追加する css ファイルを指定します。"]
```

Google Analytics などの javascript を追加する場合は、`js_list` に追加してください。

### ページのメタデータ

ページのメタデータは yaml ヘッダに記述します。

```md
---
create: 2024-05-13 # 記事の作成日
update: 2024-08-15 # 記事の最終更新日
tag: [数学, tips] # 記事に付けるタグ
---

# 見出し

こんにちは
```

### 下書き機能

`flag` に `draft` を指定します。

```md
---
create: 2024-05-13
update: 2024-08-15
tag: [test]
flag: [draft]
---

# 下書き

このページは `zakki build -d` されたときのみ HTML に変換されます。
```

### コードのハイライト

指定した区切り文字で囲まれた範囲にスタイルを適用できます。

````md
---
create: 2024-05-13
update: 2024-08-15
tag: [misc]
highlight:
  [
    { delim: ["r@", "@"], style: font-weight:bold;color:red },
    { delim: ["g@", "@"], style: font-weight:bold;color:green },
    { delim: ["b@", "@"], style: font-weight:bold;color:blue },
  ]
---

# ハイライト

```
r@ここは赤@g@ここは緑@b@ここは青@
```
````

### ページの暗号化

ページを暗号化するには、ヘッダで `crypto` フラグをセットします。<br>
パスワードは `password` で指定します。
指定がない場合、 `zakki.toml` で指定したパスワードが使用されます。

```md
---
create: 2024-05-13
update: 2024-08-15
tag: [test]
flag: [crypto]
password: test
---

# 暗号化テスト

このページは暗号化されています。
```

#### 暗号化のしくみ

[staticrypt](https://github.com/robinmoisson/staticrypt) と同様の仕組みでページを暗号化しています。<br>
ページの生成時に、内容は aes256cbc で暗号化されます。<br>
ページの表示時に、パスワードが入力されると javascript で復号します。<br>

### サイト内検索

サイト内検索には [bloom fileter](https://ja.wikipedia.org/wiki/%E3%83%96%E3%83%AB%E3%83%BC%E3%83%A0%E3%83%95%E3%82%A3%E3%83%AB%E3%82%BF) を用いています。
Bloom filter はメタデータの小ささと引き換えに、偽陽性を許すアルゴリズムです。
`zakki.toml` の `search_fp` を使うと、この偽陽性率の目安を指定できます。
小さい数値を指定するほど、メタデータのサイズが大きくなります。
