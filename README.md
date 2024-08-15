# zakki

`zakki` は静的サイトジェネレーターです。<br>
`zakki` は、インターネットのない状況下でも `file://` 経由で完全に動作することを目的としています。

Markdown から HTML への変換には [`pulldown-cmark`](https://docs.rs/pulldown-cmark/latest/pulldown_cmark/) を使っています。<br>
`zakki` は `pulldown-cmark` が扱えない構文を扱えません。

## 主な機能

- サイト生成時の数式描画
- 下書きを HTML に変換しない機能
- コードハイライト
- ページの暗号化
- サイト内検索

## 使い方

- `zakki init` コマンドでひな形を作成します。
- `zakki build` コマンドでサイトを生成します (下書きは変換されません)。
- `zakki build -d` コマンドでサイトを生成します (下書きも変換されます)。

### 設定ファイル

設定は `zakki.toml` に記述します。

```toml
site_name = "(必須) サイト名を指定します。"
password = "(任意) 暗号化用のパスワードを指定します。"
footer = "(任意) フッターの内容を HTML で指定します。"
search_fp = "(任意) サイト内検索の偽陽性率を指定します。デフォルトは 0.0001 (0.01%) です。"
```

### ページのメタデータ

ページのメタデータは yaml ヘッダに記述します。

```md
---
create: 2024-05-13 # 記事の作成日
update: 2024-08-15 # 記事の最終更新日
tag: [数学, tips]  # 記事に付けるタグ
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

正規表現を使用してコードブロックの中身を変更できます。<br>
正規表現は [`regex`](https://docs.rs/regex/latest/regex/) で解釈されます。

````md
---
create: 2024-05-13
update: 2024-08-15
tag: [misc]
highlight:
  - { before: "r@(.*?)@", after: '<span style="color:red">$1</span>' }
  - { before: "g@(.*?)@", after: '<span style="color:green">$1</span>' }
  - { before: "b@(.*?)@", after: '<span style="color:blue">$1</span>' }
---

# ハイライト

```
r@ここは赤@g@ここは緑@b@ここは青@
```
````

### ページの暗号化

ページを暗号化するには、ヘッダで `crypto` フラグをセットします。

```md
---
create: 2024-05-13
update: 2024-08-15
tag: [test]
flag: [crypto]
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
