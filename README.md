# zakki

`zakki` は静的サイトジェネレーターです。<br>

Markdown から HTML への変換は [`pulldown-cmark`](https://docs.rs/pulldown-cmark/latest/pulldown_cmark/) に依存しています。<br>
そのため、 `zakki` は `pulldown-cmark` が扱えない構文を扱えません。

## 主な機能

- サイト生成時の数式描画
- 下書き機能
- ページの暗号化
- サイト内検索
- コードハイライト
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

### ディレクトリ構造

Zakki のディレクトリ構造は次のようになっています。

```txt
.
├── src
│  ├── public/
│  ├── private/
│  ├── draft/
│  ├── gtag.js
│  └── favicon.ico
├── build/
└── zakki.toml
```

- Markdown ファイルは `src/` 下に配置します。
  - パスワードなしで公開する記事は `public/` 下に配置します。
  - パスワード付きで公開する記事は `private/` 下に配置します。
  - 下書きは `draft/` 下に配置します。
- ビルドの結果は `build/` 下に配置されます。

### 記事の追加

Markdown ファイルは `public/`, `private/`, `/draft` 下に直接配置します。

```txt
.
└── src
   └── public/
      └── foo.md
```

画像ファイルなどがある場合は、ファイル名と同名のディレクトリを作成し、そこに配置します。

```txt
.
└── src
   └── public
      ├── foo.md
      └── foo
        └── img.png
```

### ページのメタデータ

ページのメタデータは yaml ヘッダに記述します。

```md
---
create: 2024-05-13 # 記事の作成日 (必須)
update: 2024-08-15 # 記事の最終更新日 (必須)
tag: [数学, tips]  # 記事に付けるタグ
password: test     # 暗号化の際のパスワード (指定がない場合、 zakki.toml の値を使用)
---


# 見出し

こんにちは
```

`password` は記事が `private/` 配下にない場合無視されます。

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


## 暗号化のしくみ

[staticrypt](https://github.com/robinmoisson/staticrypt) と同様の仕組みでページを暗号化しています。<br>
ページの生成時に、内容は aes256cbc で暗号化されます。<br>
ページの表示時に、パスワードが入力されると javascript で復号します。<br>

## サイト内検索

サイト内検索には [bloom fileter](https://ja.wikipedia.org/wiki/%E3%83%96%E3%83%AB%E3%83%BC%E3%83%A0%E3%83%95%E3%82%A3%E3%83%AB%E3%82%BF) を用いています。
Bloom filter はメタデータの小ささと引き換えに、偽陽性を許すアルゴリズムです。
`zakki.toml` の `search_fp` を使うと、この偽陽性率の目安を指定できます。
小さい数値を指定するほど、メタデータのサイズが大きくなります。
