# zakki

`zakki` は [djot](https://djot.net/) を HTML に変換する静的サイトジェネレーターです。<br>
変換には [`jotdown`](https://docs.rs/jotdown/latest/jotdown/) を利用しています。<br>
記事は拡張子 `.dj` のファイルとして書きます。

## 主な機能

- サイト生成時の数式描画
- 下書き機能
- ページの暗号化
- サイト内検索
- コードハイライト
- コードブロックへのキャプション付け
- `file://` プロトコルでの動作

## 使い方

- `zakki init` コマンドでひな形を作成します。
- `zakki build` コマンドでサイトを生成します (下書きは変換されません)。
- `zakki build -d` コマンドでサイトを生成します (下書きも変換されます)。

ヘルプは `zakki --help` で確認できます。

### 設定ファイル

設定は `zakki.toml` に記述します。

```toml
site_name = "(必須) サイト名を指定します。"
password  = "(任意) 暗号化用のパスワードを指定します。"
footer    = "(任意) フッターの内容を HTML で指定します。"
search_fp = "(任意) サイト内検索の偽陽性率を指定します。デフォルトは 0.0001 (0.01%) です。"
js_list   = ["(任意) 追加する javascript ファイルを指定します。"]
css_list  = ["(任意) 追加する css ファイルを指定します。"]
```

Google Analytics などの javascript を追加する場合は、`js_list` に追加してください。

### ディレクトリ構造

Zakki のディレクトリ構造は次のようになっています。

```txt
.
├── src
│  ├── public/       # 公開する記事を配置します
│  ├── private/      # パスワード付きで公開する記事を配置します
│  ├── draft/        # 下書きを配置します (zakki build -d でのみビルドされます)
│  ├── gtag.js
│  └── favicon.ico
├── build/
└── zakki.toml
```

### サブページ

サブページはトップのページ一覧に表示されないページです。
`public`, `private`, `draft` ディレクトリ直下になく、かつ、名前が `index.dj` ではないファイルがサブページとして扱われます。
サブページは検索の対象には含まれます。

```txt
.
└── src/public/
   ├── foo.dj        # ページ一覧に表示される
   └── bar/
       ├── index.dj  # ページ一覧に表示される
       └── sub.dj    # ページ一覧に表示されない
```

### ページのメタデータ

ページのメタデータは yaml ヘッダに記述します。<br>
djot にヘッダの構文はないため、`zakki` が本文をパースする前に切り出しています。

```txt
---
title:  見出し       # 記事のタイトル (必須)
create: 2024-05-13   # 記事の作成日 (必須)
update: 2024-08-15   # 記事の最終更新日 (必須)
tag:    [数学, tips] # 記事に付けるタグ
password: test       # 暗号化のパスワード (指定がない場合は zakki.toml の値が使用されます)
---

こんにちは
```

`password` は記事が `private/` 配下にない場合は無視されます。

### コードのハイライト

指定した区切り文字で囲まれた範囲にスタイルを適用できます。

````txt
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

### コードブロックのキャプション

コードブロックに `caption` 属性を付けると、タイトルを表示できます。

````txt
{caption="main.rs"}
```rust
fn main() {
    println!("Hello, world!");
}
```
````

### 記事へのリンク

リンク文字列を空にしてローカルの `.dj` ファイルを指すと、リンク先の記事のタイトルで埋められます。<br>
リンク先の `.dj` は `.html` に変換されます。

```txt
[](bar/index.dj)  →  <a href="bar/index.html">参照先の記事</a>
```

### 数式

djot の数式は `$` とバックティックで書きます。<br>
中身は verbatim として扱われるため、LaTeX をそのまま書けます。

```txt
インライン: $`e = mc^2`

ディスプレイ: $$`\int_0^1 x^2 dx = \frac{1}{3}`
```

## 暗号化のしくみ

[staticrypt](https://github.com/robinmoisson/staticrypt) と同様の仕組みでページを暗号化しています。<br>
ページの生成時に、内容は aes256cbc で暗号化されます。<br>
ページの表示時に、パスワードが入力されると javascript で復号します。<br>

## サイト内検索

[bloom filter](https://ja.wikipedia.org/wiki/%E3%83%96%E3%83%AB%E3%83%BC%E3%83%A0%E3%83%95%E3%82%A3%E3%83%AB%E3%82%BF) による軽量な全文検索を実現しています。<br>
Bloom filter は確率的な手法であり、正確さとメタデータの大きさの間にトレードオフがあります。<br>
`zakki.toml` の `search_fp` を使うと、偽陽性率の目安を指定できます。<br>
小さい数値を指定するほど、検索が正確になり、メタデータのサイズが大きくなります。
