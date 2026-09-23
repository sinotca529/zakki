# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.20.1](https://github.com/sinotca529/zakki/compare/v0.20.0...v0.20.1) - 2026-09-23

### Added

- サブページの一覧を index に自動で出す
- Source Code Pro の同梱をやめる
- ttc の中のフォントを名前で選ぶ
- private の記事の文字が混ざることを警告する
- タグとキャプションの文字もサブセットに残す
- コードブロックのフォントをサブセットして配る

### Fixed

- group の判定を、まとめるディレクトリの index.md に限る
- css と js のパスの先頭の / をサイトのルートとして読む
- サイトのルートに置くページで css と js の URL が / で始まる問題を修正
- タグを等幅にしない
- 警告を、非公開の記事にしかない文字に絞る
- ASCII を常に含めるのをやめる
- 検索窓を等幅にしない

### Other

- 作業の分担を、設計の力が伸びるかどうかで分ける
- Merge remote-tracking branch 'origin/main' into feat/children-list
- 生成する CSS を asset に出す
- README にコードブロックのフォントの設定を書く

## [0.20.0](https://github.com/sinotca529/zakki/compare/v0.19.0...v0.20.0) - 2026-09-21

### Added

- 日付を型で持つ
- [**breaking**] デフォルトのフッターを廃止
- 暗号を強化
- 復号失敗時にエラーメッセージを表示
- エラーメッセージの拡充
- ヘルプを日本語化
- 画像に tabindex を設定
- table を tab で選択可能に

### Fixed

- 壊れた記事を全件報告する
- SVG の埋め込みに代替と名前を足す
- page_url をたたんで呼び出し側に書く
- page_url の引数を &str にする
- sitemap.xml の loc を直す
- 見出しの id に接頭辞 s を付ける
- 非公開記事のタイトルは索引に残す
- 空の区切り文字を弾く理由を書き直す
- 空の区切り文字を設定できないようにする
- css の path がエスケープされる問題を修正
- Url のセグメントが2重エスケープされる問題の対処
- タグのリンクをエスケープ
- 暗号化ページのタイトルをエスケープ処理
- [**breaking**] front matter と設定の打ち間違いをエラーにする
- 検索欄を空にしても結果が残る問題を直す
- 検索結果とタグリンクのエスケープ漏れを直す
- 非 ascii のパスワードを受け付けるように変更
- 表の折り返しを無効化
- 表横スクロール関連の css を修正
- copy_asset マクロが呼び出し元の処理そのものを return させてしまう問題を修正

### Other

- 文字の種別と小文字化の判定を減らす
- トークンの確保とディレクトリ作成を減らす
- トークン分割とハイライトの探索を軽くする
- 属性の並べ替えを 1 回の走査にする
- なぜ SVG を object で埋め込むのかを書く
- sitemap の組み立てと書き出しを分ける
- KaTeX を Rust 実装に替える
- 閉じタグを手で書く場所をなくす
- 参照されていない id="searchbar" を外す
- 役目を終えた id="main-content" を外す
- スタイル目的の id をクラスに置き換える
- 要素の組み立てを 1 か所にまとめてエスケープを寄せる
- 見出しの id がない場合に expect で落とす
- 目次をイベント列から作る
- パスは人が読む文字を Text として残す
- tokenize の doc が言っていることをコメントで繰り返さない
- 索引の作成で中間の文字列を持たない
- 検索の索引をイベント列から作る
- [**breaking**] パーサを comrak から pulldown-cmark に戻す
- bail と expect が言っていることをコメントで繰り返さない
- ハイライトのエスケープを組み立て時の 1 回にする
- path を str に変換する方法を調整
- エラーメッセージを改善
- current_dir の呼び出しを1箇所にまとめる
- テストを追加
- '.' に対応するパス専用のファクトリを削除
- Url の公開メソッドを調整
- タグリンク作成時の引数を改善
- Url 型の導入
- ページがプライベートかどうかをメタデータに移動
- rename render_assets to copy_assets
- render.rs の分解に向けて
- Context, Metadata の整理
- Config を削除
- テストのモジュール名を test_vector にする
- 表の読み込みを include_testdata にまとめる
- 例示データのサロゲートペアを中立な語に差し替える
- 表と重複する名前付きテストを削除する
- Rust と JS の規則をテストベクタで突き合わせる
- .hidden をやめて hidden 属性を CSS で守る
- 保守性を下げるため、コメント中で他ファイル名への言及をやめる
- コメント追加 + concat 利用
- 未使用の依存を削除
- 造語についての規則を書き直す
- 文体検査に textlint を導入する
- 見出しは名詞句にするという規則を足す
- 会話での見出しと分量の規則を足し、検査を GitHub コメントにも広げる
- 文体と進め方の指摘を反映
- 文体の規則に指摘のあった5件を反映
- git の取り決めを一般化
- AI 向けの指示ファイルを追加
- 設定ファイル名を定数化
- clippy
- find が多重で呼ばれる問題の解消
- サブページ判定をカプセル化
- Renderer のフィールド順を new にあわせる
- dst という用語を排除
- path.rs
- 不要な Result を剥がした
- ProjectPaths::at_current_dir の責務を縮小
- 不要な derive を削除
- パスの扱いをカプセル化
- 未使用 css の削除
- タグ一覧への css を統一
- index.html のタグ一覧のスタイルをカードなどと統一
- fmt
- 不要な空行を削除
- js を追加指定する機構 (未使用) を削除
- push_js_path を復活。ただし、現状は未使用。
- table と figure でスクロール制御の class を共有
- インラインスタイルを css ファイルに移動
- タグ一覧の不要な nbsp を排除し、 css で隙間を確保
- マジックリテラルの利用を排除

## [0.19.0](https://github.com/sinotca529/zakki/compare/v0.18.1...v0.19.0) - 2026-09-06

### Added

- [**breaking**] ヘッダid採番を変更 & ヘッダの順序を厳格化
- ヘッダ番号を画面上に表示するように変更
- ヘッダの id 採番を数字に戻す
- [**breaking**] タイトルが指定されているリンクもリンク切れを検知する
- [**breaking**] リンク切れのウィキリンクがある場合はエラーとする
- svg を囲む object に title 要素を追加
- [**breaking**] djot をやめて Markdown (comrak) に戻す
- [**breaking**] Markdown から djot へ移行する

### Fixed

- 注記 (> [!NOTE] など) のスタイルが当たっていない問題を修正
- site_name をエスケープ
- html 出力時にタグをエスケープ
- html 出力時にタイトルをエスケープ
- 空の記事が常に検索ヒットする問題に対処
- タイトルが検索対象になるように修正
- コードブロックは html エスケープ後にハイライトされるため、ハイライトの正規表現も html エスケープするよう修正
- 独自ハイライト構文の区切りについて、正規表現をエスケープ
- コードブロックの属性値のエスケープ手法を修正
- 目次をエスケープ処理
- 目次の表示名がテキストになるように調整
- 相対パスでのリンク指定の解決
- WikiLink でリンク表示名を指定した場合に無視される問題を解消
- figcaption の中身をエスケープ
- YAML ヘッダの区切り文字の扱いを修正
- CRLF のフロントマターを受け付ける
- 見出しの閉じイベントの id を開始イベントと揃える
- 画像に書かれた djot の属性を出力に引き継ぐ

### Other

- パーセントエンコーディングの判定を改善
- クエリパラメータのタグのエスケープ手法を text から attr 用のものに修正
- clippy
- 検索インデックスの title を html パースではなく直接取得
- コメント追加
- 不要な pub の削除
- 独自ハイライトの正規表現コンパイルを1回に改善
- スペルミス修正 & コメント修正
- コメントの改善
- コメント追加
- 定数の活用
- html のエスケープ処理を修正
- YAML ヘッダの区切り文字を定数化
- header の id 生成を comark に移譲
- コメント改善
- エラーメッセージの改善
- エラーメッセージを改善
- figcaption のエスケープを緩和
- unwrap -> expect
- リンク表示名の指定有無検知方法を改善
- figure タグの作成処理を切り出し
- with_context -> context
- cargo fmt
- エラーメッセージの修正
- title_map の意義をコメントで補足
- エラーメッセージを日本語化
- ctxt -> ctx (rename)
- YAML ヘッダ行の区切り行除去を改善
- リファクタリング
- エスケープ関数の名前を用途がわかるものにする
- raw_html のクロージャを clone に置き換える

## [0.18.1](https://github.com/sinotca529/zakki/compare/v0.18.0...v0.18.1) - 2026-08-22

### Fixed

- fix init subcommand
- make titles searchable

### Other

- add comment
- cargo clippy
- remove dead code
- brush up README.md
- cargo fmt
- simplify tokenize method
