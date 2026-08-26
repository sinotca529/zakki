# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.19.0](https://github.com/sinotca529/zakki/compare/v0.18.1...v0.19.0) - 2026-08-26

### Added

- [**breaking**] Markdown から djot へ移行する

### Fixed

- CRLF のフロントマターを受け付ける
- 見出しの閉じイベントの id を開始イベントと揃える
- 画像に書かれた djot の属性を出力に引き継ぐ

### Other

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
