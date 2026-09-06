# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
