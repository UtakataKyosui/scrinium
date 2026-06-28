# Changelog

All notable changes to scrinium are documented in this file.

## [1.0.0] - 2026-06-28

### Bug Fixes
- **archivum**: YAML Frontmatterがない場合のFallbackと操作方法を常時表示を実装

- **archivum**: 引数の取り方がユーザからは使いづらそうだったので、--typeのような ハイフン2つとプロパティ名で明示指定するように修正

- **archivum**: .claude配下がvalidateの走査対象に入らないように修正


### Build System
- **nix**: Nixを使った環境構築（Rust-Overlayを使用）

- **devcontainer**: DevContainerの修正

- **nix-flake**: Claude Codeを追加する

- **devcontainer**: Claude Code SonnetにDevContainerの構成を改善してもらった

- **devcontainer,nix-flake**: Ghとatuinの追加

- **devcotainer**: Claude Code SonnetにModernなコマンドに自動的に書き直すように実装してもらった

- **devcotainer**: Claude Code Sonnetに直してもらった


### CI/CD
- **check & release**: CHANGELOG作成のためのCIを作成

- **release**: リリースの失敗があったので修正 + archivumという名前がすでに使用されていたので修正


### Documentation
- **gh-wheel**: Gh-wheelの追加とClaude CodeのAgent SKillの追加

- **readme**: READMEを更新


### Features
- DevContainerを作成

- **parse**: YAMLフロントマターをパースするプログラムの作成開始、まずは、依存関係のパッケージを（fronma）を導入、サンプルをコピー

- **archivum**: CLI部分を作成

- **archivum**: TUIとしての実装を追加

- **archivum**: Open Knowledge Formatの仕様に合わせて、YAML Frontmatterのプロパティ拡張への対応と、更新時に自動的にtimestampを更新するように実装


### Miscellaneous
- **hooks**: Rmの改善版、rm-improvedを導入したので、それの変換プログラムも仕込む

- **agent-skill**: Open Knowledge FormatのAgent SkillをClaude Sonnetに作成してもらった


### Testing
- **archivum**: テストを追加

