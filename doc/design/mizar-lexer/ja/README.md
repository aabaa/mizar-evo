# Module Specifications: mizar-lexer

> Canonical language: English. English canonical version: [../en/README.md](../en/README.md).

`mizar-lexer` は、Mizar Evo のソーステキストをトークン化するための基盤機能を担当します。

Mizar の字句分類(lexical classification)は、インポート、アクティブなユーザーシンボル(active user symbol)、パーサーの位置、スコープ付き識別子束縛(scoped identifier binding)に依存します。そのためこのクレートでは、生の字句スキャン(raw lexical scanning)と最終トークンの曖昧性解消(final token disambiguation)を明確に分けて扱います。

## Context

- [doc/spec/en/02.lexical_structure.md](../../../spec/en/02.lexical_structure.md) - lexical structure, identifiers, symbols, layout, literals, annotations
- [doc/spec/en/11.symbol_management.md](../../../spec/en/11.symbol_management.md) - user-defined symbols and active lexicons
- [doc/design/architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md) - frontend pipeline and source/token boundaries
- [doc/design/architecture/en/03.module_and_symbol_resolution.md](../../architecture/en/03.module_and_symbol_resolution.md) - module imports, namespaces, and symbol resolution
- [doc/design/internal/en/07.crate_module_layout.md](../../internal/en/07.crate_module_layout.md) - crate/module ownership map

## Index

| Document | Maps To | Description | Status |
|---|---|---|---|
| [raw_lexer.md](./raw_lexer.md) | `crates/mizar-lexer/src/raw_lexer.rs`（ソース/テーブル境界の補足を含む） | 生スキャン、`LexemeRun`、ソース前処理の受け渡し、予約テーブル、曖昧性解消の境界 | Draft |
| [import_prescan.md](./import_prescan.md) | `crates/mizar-lexer/src/import_prescan.rs` | インポートプレリュードのスキャンと、生トークンからの `ImportStub` 抽出 | Draft |
| [lexical_environment.md](./lexical_environment.md) | `crates/mizar-lexer/src/lexical_environment.rs` | 予約テーブルとモジュール字句サマリーからのアクティブ字句環境の構築 | Draft |
| [scope_skeleton.md](./scope_skeleton.md) | `crates/mizar-lexer/src/scope_skeleton.rs` | 予約キーワードに基づく字句スコープスケルトンと `ScopeLexView` 射影 | Draft |
| [disambiguator.md](./disambiguator.md) | `crates/mizar-lexer/src/disambiguator.rs` | `LexemeRun` から最終トークンへの、文脈依存・最長一致のトークン曖昧性解消 | Draft |
| [test_and_implementation_plan.md](./test_and_implementation_plan.md) | `tests/lexical`, `tests/coverage/spec_trace.toml`, `crates/mizar-lexer` | 順序付けされた字句解析器のテストコーパスと実装チェックリスト | Draft |
| [todo.md](./todo.md) | `crates/mizar-lexer`, `tests/lexical`, review follow-ups | 品質レビューのフォローアップタスク | Living |

## Crate Boundary

`mizar-lexer` が提供するもの:

- ソース読み込みが LF のみに正規化した後の、生の字句スキャン;
- コメント除去、LF/ASCII の診断、モジュールソース名の規約を扱うソース前処理ヘルパー;
- Unicode 正規化を行わない、厳密に ASCII のみのコード領域綴り規則;
- ソーススパンを保持する生の字句単位;
- 字句解析器の診断とテストのための、軽量なソーススパン→行/列変換ヘルパー;
- 識別子・数字・レイアウト・予約語・記号形状の認識を行うヘルパー API;
- 字句環境・パーサーの期待・スコープ付き束縛を受け取る、最終トークン曖昧性解消のサポート.

このクレートが行わないもの:

- ファイルの読み込みやプラットフォーム改行の正規化;
- Unicode ソーステキストの正規化、および正規化による非 ASCII のコード識別子/記号の受理;
- インポートの解決;
- インポートしたファイルの完全なモジュール IR の読み込み;
- 識別子が未定義かどうかの判定;
- 型検査・オーバーロード解決・証明に関わる意味論.
