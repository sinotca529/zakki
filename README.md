# zakki

`zakki` は静的サイトジェネレーターです。<br>
`zakki` の目的は、クライアント側の処理を減らすことです。

Markdown から HTML への変換には [`pulldown-cmark`](https://docs.rs/pulldown-cmark/latest/pulldown_cmark/) を使っています。<br>
`zakki` は `pulldown-cmark` が扱えない構文を扱えません。

## 主な機能

- サイト生成時の数式描画
- 下書きを HTML に変換しない機能
- コードハイライト
- ページの暗号化

## 使い方

- `zakki init` コマンドでひな形を作成します。
- `zakki build` コマンドでサイトを生成します (下書きは変換されません)。
- `zakki build -d` コマンドでサイトを生成します (下書きも変換されます)。

### 設定ファイル

設定は `config.toml` に記述します。

```toml
site_name = "(必須) サイト名を指摘します。"
password = "(任意) 暗号化用のパスワードを指定します。"
footer = "(任意) フッターの内容を HTML で記述します。"
```

### ページのメタデータ

ページのメタデータは yaml ヘッダに記述します。

```md
---
date: 2024-05-13
tag: [数学]
---

# 見出し

こんにちは
```

### 下書き機能

`flag` に `draft` を指定します。

```md
---
date: 2024-05-12
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
date: 2022-05-15
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
date: 2024-05-12
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
