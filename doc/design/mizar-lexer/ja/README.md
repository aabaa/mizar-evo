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
| [00.crate_plan.md](./00.crate_plan.md) | `doc/design/autonomous_crate_development.md`, `tests/coverage/spec_trace.toml`, `crates/mizar-lexer` | 後追いの autonomous crate plan、責務境界、gap classification、task decomposition | Complete |
| [raw_lexer.md](./raw_lexer.md) | `crates/mizar-lexer/src/raw_lexer.rs`（ソース/テーブル境界の補足を含む） | 生スキャン、`LexemeRun`、ソース前処理の受け渡し、予約テーブル、曖昧性解消の境界 | Draft |
| [import_prescan.md](./import_prescan.md) | `crates/mizar-lexer/src/import_prescan.rs` | インポートプレリュードのスキャンと、生トークンからの `ImportStub` 抽出 | Draft |
| [lexical_environment.md](./lexical_environment.md) | `crates/mizar-lexer/src/lexical_environment.rs` | 予約テーブルとモジュール字句サマリーからのアクティブ字句環境の構築 | Draft |
| [scope_skeleton.md](./scope_skeleton.md) | `crates/mizar-lexer/src/scope_skeleton.rs` | 予約キーワードに基づく字句スコープスケルトンと `ScopeLexView` 射影 | Draft |
| [disambiguator.md](./disambiguator.md) | `crates/mizar-lexer/src/disambiguator.rs` | `LexemeRun` から最終トークンへの、文脈依存・最長一致のトークン曖昧性解消 | Draft |
| [test_and_implementation_plan.md](./test_and_implementation_plan.md) | `tests/lexical`, `tests/coverage/spec_trace.toml`, `crates/mizar-lexer` | 順序付けされた字句解析器のテストコーパスと実装チェックリスト | Draft |
| [crate_exit_report.md](./crate_exit_report.md) | `doc/design/autonomous_crate_development.md`, verification results, handoff | 後追いの crate exit report、quality score、hard gates、deferred items、next-task handoff | Complete |
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

## Responsibility Boundaries

字句解析器クレートは、完全なパーサーなしで最終的な字句トークン列を構築するために必要な、浅いプレパーサーヘルパー(pre-parser helpers)を保持してよいものとします。これらのヘルパーは、綴りに基づき、ソーススパンを保持し、決定的でなければなりません。字句解析器への受け渡しに必要な構造は復元してよいものの、権威ある構文・ソース・意味論のサービスになってはなりません。

| Capability | Long-term owner | Current `mizar-lexer` role |
|---|---|---|
| ファイル I/O、ソース探索、パッケージルートの強制、シンボリックリンク/大文字小文字ポリシー、ソース同一性、スナップショット | `mizar-session` またはフロントエンド/ソースサービス | ファイル I/O は行いません。クレートローカルなバイト読み込みヘルパーとモジュール名ヘルパーは、テストと初期統合のための実行可能な境界契約であり、ファイルシステムの所有ではありません。 |
| UTF-8 検証、先頭 BOM の除去、CRLF から LF への正規化、元バイトの読み込みマップ | ソース/セッション層 | `load_source_text_from_bytes` は、テストと単純な呼び出し側のために境界をミラーします。本番の読み込みマップはセッション/ソース側のコードが所有し、同じ挙動を再利用またはミラーしてかまいません。 |
| コメント除去、ドキュメンテーションコメントのトリビア、字句テキストのソースマップ、字句解析器境界での不正テキスト診断 | 字句の受け渡しは字句解析器、保持するソースマップサービスはセッション/ソース | `preprocess_source_for_lexing` は `mizar-lexer` に残します。生スキャン、インポートのプレスキャン、スコープスケルトンがコメント除去後の字句テキストを消費するためです。豊富な保持マップとエディタ向けスナップショットは字句解析器の外に置きます。 |
| 生スキャン、予約テーブル、識別子・数字・記号の綴りヘルパー、最終トークンスパン | `mizar-lexer` | `mizar-lexer` が直接所有します。スパンは、スキャナーへの厳密な入力に対するバイトオフセットのままにします。 |
| インポートプレリュードの形状抽出 | プレパーサーによる抽出は `mizar-lexer`、解決はモジュールリゾルバ/ビルドプランナー | `scan_import_prelude` は、最終的な曖昧性解消の前に生のインポートスタブを抽出してよいものとします。モジュール解決、サマリーの読み込み、可視性チェック、プレリュード終端を超えたインポート配置の検証は行いません。 |
| 解決済みインポートと字句サマリーからのアクティブ字句環境 | モジュールリゾルバと字句解析器の境界 | `build_lexical_environment` は、解決済みのインポートとモジュール字句サマリーを消費します。サマリーの生成とインポートグラフの決定は字句解析器の外で行います。 |
| 字句オーバーライドに必要なスコープスケルトン | パーサーが同等のトークン化前の受け渡しを提供するまでは `mizar-lexer` | `build_scope_skeleton` は、`ScopeLexView` が必要とする束縛範囲を保守的に復元してよいものとします。権威ある AST、構文の受理、名前解決、意味論的ライフタイムはパーサー/リゾルバが所有します。 |
| パーサー字句コンテキスト | パーサー | `ParserLexContext` は、曖昧性解消が消費するパーサー向けの要求オブジェクトです。字句解析器は、渡されたコンテキストを尊重する以上に文法の進行を判定しません。 |
| 人間向け診断、レンダリング、タブ展開、1 始まりの列、LSP の UTF-16 位置 | 診断/フロントエンド/LSP アダプタークレート | 字句解析器の診断は、安定したコードとバイトスパンを保持します。人間向け/プロトコル向けの座標変換は、トークンおよび字句解析器のコア状態の外で明示します。 |
| 型検査、オーバーロード解決、証明の意味論、未定義名の診断 | リゾルバ/エラボレータ/カーネル側のフェーズ | 字句解析器は所有しません。ユーザーシンボルのトークンは綴りとスパンを保持し、後続フェーズが候補を復元して意味を選びます。 |

意図する依存方向は次の通り:

```text
session/source/frontend
  -> raw/preprocessed source text
  -> mizar-lexer
  -> raw tokens, import stubs, lexical summaries/environment handoff, final tokens
  -> parser
  -> resolver/elaborator/proof phases
  -> diagnostics/LSP adapters for rendering and protocol conversion
```

アダプターは、座標空間を橋渡しするために `mizar-lexer` とセッション/ソースの各クレートの両方に依存してかまいません。ただし `mizar-lexer` は、パーサー、リゾルバ、セッションスナップショット、診断レンダリング、LSP プロトコルの各クレートに依存すべきではありません。将来、フロントエンドクレートがソースからトークンまでの完全な受け渡しを所有する場合は、実行可能なソース読み込みヘルパーをラップまたは移動してよいものの、字句解析器はトークンスパンをバイト指向に保ち、プレパーサーヘルパーを字句の受け渡しデータに限定しなければなりません。
