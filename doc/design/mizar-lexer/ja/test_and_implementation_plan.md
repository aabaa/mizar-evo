# Module: test_and_implementation_plan

> Canonical language: English. English canonical version: [../en/test_and_implementation_plan.md](../en/test_and_implementation_plan.md).

## Purpose

この文書は、`mizar-lexer` のテスト追加と実装の推奨順序を定義します。

目的は、テストコーパスを実装より少し先行させつつ、ブートストラップ実装の一時的な制約を恒久的な言語仕様として固定しないことです。コミットするコーパステストは安定した言語契約を表し、クレートローカルのユニットテストは機能の実装中の一時的な実装境界を表してよい、という分担にします。

## Guiding Rules

- 段階的なテストモデルに従う。字句解析器のテストは `stage = "lexical"`、`expected_phase = "lex"` に属する。
- `tests/lexical/` 以下にコミットする実行可能なフィクスチャは、隣接する `.expect.toml` を必ず持つ。
- テストが主張するカバレッジは、テスト追加の前または同時に `tests/coverage/spec_trace.toml` へ追加する。
- 現在のブートストラップ字句解析器が拒否するという理由だけで、将来有効になる言語形式をコミット済みの失敗フィクスチャにしない。
- 一時的に未対応の入力動作は、クレートローカルのユニットテストに置く。
- 1 つのフィクスチャは、1 つの字句規則をできるだけ小さく切り出す。
- `scan_raw` の導入後は、生スキャンのテストと最終トークンの曖昧性解消のテストを混ぜない。

## Traceability-First Requirement Inventory

字句解析器のフィクスチャをまとめて追加する前に、まず `doc/spec/en/02.lexical_structure.md` を、確認可能な要件レコードに分解します。

これは計画の手順であり、実装が章全体をすでに満たしているという主張ではありません。マニフェストは、テストが存在する前から既知の欠落を見えるようにするために使います。

推奨ワークフロー:

1. 第 2 章を安定した要件 ID に分解し、`tests/coverage/spec_trace.toml` に追加する。
2. 実装済みかつテスト済みの動作は `status = "covered"` にする。
3. 既知だが未テストの動作は `status = "planned"` にする。
4. カバレッジを意図的に延期する場合だけ `status = "deferred"` を使い、`deferred_reason` を含める。
5. 各レコードに `pass`, `fail`, `pass_and_fail`, `diagnostic`, `snapshot`, `property`, `manual_review` のいずれかのカバレッジ形態を選ぶ。
6. コミットするフィクスチャを後で追加するときに、そのレコードの `tests = [...]` を埋め、各 `.expect.toml` の `spec_refs` に対応する ID を追加する。

最初の棚卸しには、少なくとも次を含めます。

- 文字集合とソースの事前条件;
- レイアウトと LF のみの境界;
- 予約語;
- 予約特殊記号;
- ユーザー定義の記号名;
- 最長一致の動作;
- ドットの曖昧性解消;
- 識別子;
- 数字;
- 文字列リテラルの文脈依存性;
- ファイルとモジュールの命名に関する字句制約;
- コメントと注釈マーカー;
- 字句の前処理の境界.

コミット済みのテストを持つ要件だけを `covered` にします。たとえば現在のブートストラップ状態では、`spec.en.02.lexical.identifiers.basic` は `pass_lexical_identifier_basic_001` により covered にできる一方、予約語・数字・記号名・注釈・文脈依存の曖昧性解消は `planned` または `deferred` のままにします。

## Phase 0: Bootstrap Identifier Lexer -> Done

Status: 歴史的な bootstrap surface。後続の phase で、現在の実装はこの API を超えて拡張されている。

実装の対象範囲:

- `Token`
- `TokenKind::Identifier`
- `LexError`
- `lex(&str)`

コミットするコーパステストは、現在の API で表現できる安定した動作だけをカバーします。

- ASCII 識別子の開始: `A-Z`, `a-z`, `_`
- ASCII 識別子の継続: `A-Z`, `a-z`, `0-9`, `_`, `'`
- 空白・タブ・LF のレイアウトのスキップ
- レイアウトで区切られた複数の識別子
- 大文字小文字を区別した綴りの保持

推奨フィクスチャ:

```text
tests/lexical/pass/pass_lexical_identifier_basic_001.src
tests/lexical/pass/pass_lexical_identifier_start_ascii_001.src
tests/lexical/pass/pass_lexical_identifier_continue_ascii_001.src
tests/lexical/pass/pass_lexical_identifier_apostrophe_001.src
tests/lexical/pass/pass_lexical_layout_space_tab_lf_001.src
tests/lexical/pass/pass_lexical_identifier_case_sensitive_001.src
```

推奨要件 ID:

```text
spec.en.02.lexical.identifiers.basic
spec.en.02.lexical.identifiers.start_ascii
spec.en.02.lexical.identifiers.continue_ascii
spec.en.02.lexical.identifiers.apostrophe
spec.en.02.lexical.layout.space_tab_lf
spec.en.02.lexical.identifiers.case_sensitive
```

クレートローカルのユニットテストでは、一時的なブートストラップ上の拒否をカバーしてよいです。

- 数字トークンが存在するまでは、数字で始まる入力は未対応;
- 生の `LexemeRun` スキャンが存在するまでは、句読点は未対応;
- キャリッジリターンは字句境界ではレイアウトではない;
- 非 ASCII のコード領域テキストは、不正なフィクスチャが意図的にこの層に到達する場合を除き、字句解析器の事前条件の外側にある.

これらの一時的なケースは、`tests/coverage/spec_trace.toml` で恒久的な失敗カバレッジとして主張しないでください。

完了条件:

- 字句の合格フィクスチャがトークンの期待値を持つ;
- トレーサビリティマニフェストがすべての新しいサイドカーを参照する;
- `cargo test -p mizar-lexer` とメタデータ検証が通る。

## Phase 1: Raw Scanner API -> Done

目標とする API の方向性:

```rust
pub fn scan_raw(input: &str) -> Result<RawTokenStream, LexError>;
```

最終トークンの種別を増やす前に、ソーススパンを保持する生の単位を導入します。

生スキャンのテストは以下をカバーします。

- 空の入力;
- 選んだ生ストリームの形に応じた、レイアウトのランまたはレイアウトのスキップ;
- 識別子の形をした `LexemeRun` の値;
- `+`, `*+`, `|.`, `.|` のような、記号の形をした `LexemeRun` の値;
- `x*+y` のような混在したラン;
- `0`, `42` のような数字に似た生の単位;
- `@latex`, `@[` のような注釈マーカーの形;
- 安定した診断を持つ、未対応または不正な生の文字.

重要な境界:

- 生スキャンのテストは、インポートの解決・アクティブなユーザーシンボル・パーサーの位置・スコープ付き束縛を要求しない。
- コメントは生トークンではない。コメント除去とトリビアの取り込みは、ソース読み込みが正規化済みのソーステキストを提供した後の `mizar-lexer` の前処理の責務。

推奨要件 ID:

```text
spec.en.02.lexical.raw.lexeme_runs
spec.en.02.lexical.raw.numeral_like
spec.en.02.lexical.annotations.markers
spec.en.02.lexical.preprocessing.comments_not_raw_tokens
```

完了条件:

- 現在の `lex(&str)` は、ブートストラップの部分集合について生スキャンに委譲するか、互換ラッパーとして明示されている;
- 生スパンがテストまたはスナップショットでカバーされている;
- 句読点が恒久的な字句解析器のエラーとしてテストされていない。

## Phase 2: Reserved Tables And Final Token Shell -> Done

目標とする実装:

- 予約語テーブル;
- 予約特殊記号テーブル;
- 必要に応じた、予約語・予約記号・数字・ユーザーシンボル・文字列リテラル・エラー回復のための最終 `TokenKind` バリアント;
- 識別子・数字・レイアウト・予約語・記号形状の認識を行うヘルパー API.

テストでは以下を確認します。

- 予約語テーブルの各項目が綴りで認識される;
- 予約語が大文字小文字を区別する;
- 予約特殊記号が `:=`, `<>`, `.=` , `.*`, `.{`, `@[`, `...` などの複合項目を含む;
- 予約複合記号が最長の綴りを優先する;
- 予約語を内部に含むだけの通常の識別子が、完全一致でない限り識別子のままであること。

推奨要件 ID:

```text
spec.en.02.lexical.reserved_words.table
spec.en.02.lexical.reserved_words.case_sensitive
spec.en.02.lexical.reserved_symbols.table
spec.en.02.lexical.reserved_symbols.longest_compound
```

完了条件:

- 予約テーブルがデータ駆動、または仕様と照合しやすい形で管理されている;
- テストが、生の綴り認識と文脈依存の曖昧性解消を区別している。

## Phase 3: Import Pre-Scan -> Done

目標とする API の方向性:

```rust
pub fn scan_import_prelude(raw: &RawTokenStream) -> ImportPrelude;
```

テストでは以下を確認します。

- 空のプレリュード;
- 単一のインポート;
- カンマ区切りのインポート;
- 分岐インポート;
- `as` による別名;
- `.` と `..` による相対インポート;
- `export`, `definition`, `registration`、定理に類する項目、またはその他の非インポートのトップレベルテキストでのプレリュードの終了;
- 不正なインポートからの回復;
- プレリュード終了後にインポートをスキャンしないこと。

推奨要件 ID:

```text
spec.en.12.modules.import_prelude.basic
spec.en.12.modules.import_prelude.alias
spec.en.12.modules.import_prelude.relative
spec.en.12.modules.import_prelude.branch
spec.en.12.modules.import_prelude.termination
spec.en.12.modules.import_prelude.malformed_recovery
```

完了条件:

- 事前スキャンが、生のパスの綴りとソーススパンだけを返す;
- 事前スキャンが、モジュールの存在・可視性・インポートの循環・エクスポートされたシンボルを解決しない。

## Phase 4: Active Lexical Environment -> Done

目標とする API の方向性:

```rust
pub fn build_lexical_environment(
    imports: &[ResolvedImport],
    summaries: &[ModuleLexicalSummary],
) -> Result<ActiveLexicalEnvironment, LexicalEnvironmentError>;
```

テストでは以下を確認します。

- 予約テーブルが常に存在する;
- 記号の形をしたインポート済みシンボルが可視である;
- 識別子の形をしたインポート済みシンボルが可視である;
- `.` を含むシンボルをインデックス化できる;
- 異なるインポートから来た同綴りのユーザーシンボルが決定的に拒否される;
- 予約語・予約記号との不正な衝突が拒否される;
- 環境のフィンガープリントが決定的な入力順序に対して安定する;
- 最長一致の探索が、識別子の形・記号の形をしたシンボルの両方で機能する。

推奨要件 ID:

```text
spec.en.11.symbol_management.active_lexicon.imported_symbols
spec.en.11.symbol_management.active_lexicon.import_conflicts
spec.en.11.symbol_management.active_lexicon.reserved_collisions
spec.en.11.symbol_management.active_lexicon.fingerprint
```

完了条件:

- 環境のテストが、完全なモジュール IR ではなく軽量なモジュール字句サマリーを使う;
- 探索の動作が繰り返し実行で決定的である;
- 生スキャンのテスト・最終トークンシェルのテスト・インポート事前スキャンのテストとは分離し、クレートローカルの字句環境ユニットテストとトレーサビリティマニフェストでカバレッジを記録する。

## Phase 5: Scope Skeleton -> Done

目標とする API の方向性:

```rust
pub fn build_scope_skeleton(raw: &RawTokenStream) -> ScopeSkeleton;
```

テストでは以下を確認します。

- 空のスケルトン;
- 単純な `let x` 形式の束縛;
- カンマ区切りの束縛子;
- 対応が進むにつれての `for`, `reserve`, `given` の束縛子の形;
- `definition`, `proof`, `now`, `end` の入れ子ブロック範囲;
- 不正な束縛子では名前を捏造せず過小近似すること;
- `ScopeLexView` が束縛範囲の内側でのみ真を返すこと;
- 繰り返し実行で決定的な出力。

推奨要件 ID:

```text
spec.en.02.lexical.scope_override.skeleton
spec.en.04.variables_and_constants.binders.lexical_shapes
spec.en.16.theorems_and_proofs.blocks.lexical_ranges
```

完了条件:

- スケルトンの構築が、式の全体をパースしない;
- スケルトンの診断が構造的かつ回復可能である;
- スケルトンの出力が、字句上の上書きの問いだけに答えられる。

## Phase 6: Disambiguator -> Done

目標とする API の方向性:

```rust
pub fn disambiguate(
    raw: &RawTokenStream,
    lexical_env: &ActiveLexicalEnvironment,
    parser_context: &ParserLexContext,
    scope_view: &dyn ScopeLexView,
) -> TokenStream;
```

テストでは以下を確認します。

- 記号の形をしたユーザーシンボルの最長一致;
- 識別子の形をしたユーザーシンボルと、通常の識別子の区別;
- アクティブな識別子の形をしたシンボルに対する、スコープ付き識別子束縛の上書き;
- 予約語の出力;
- 予約複合記号の出力;
- 名前空間パスのコンテキスト;
- 複合予約トークン・ユーザーシンボル・セレクタアクセス・名前空間パスに関するドットの曖昧性解消;
- 文字列リテラルが文字列必須のパーサーコンテキストでのみ認識されること;
- 字句環境を経由したインポート衝突の報告;
- 回復処理が、安定した `ErrorRecovery` トークンと診断を出力すること;
- 生トークンと 1 対 1 に対応する場合、`LexemeRun` を分割する場合、文字列リテラル、回復トークンのいずれでも、最終 `Token` がソーススパンを保持すること;
- トークンに行/列を保存せず、スパンが指すテキストと同じテキストから構築した軽量な行インデックスを通じて、ソーススパンから行/列に変換できること。

推奨要件 ID:

```text
spec.en.02.lexical.longest_match
spec.en.02.lexical.user_symbols.identifier_shaped
spec.en.02.lexical.user_symbols.punctuation_shaped
spec.en.02.lexical.dot_disambiguation
spec.en.02.lexical.string_literals.context_sensitive
spec.en.02.lexical.error_recovery
```

完了条件:

- 曖昧性解消器が、環境・パーサーコンテキスト・スコープビューを消費するが構築しない;
- 未定義の識別子が字句の `Identifier` トークンのままで、名前解決が後続フェーズで拒否する;
- 診断が順序と同一性において安定している;
- すべての最終トークンがソーススパンを持つこと。連続したソーステキストに由来するトークンでは、`token.lexeme` がそのスパンの指すソース列と一致すること;
- 行/列ヘルパーが、有効な `SourceSpan` から 0 始まりのバイト列の位置を導出し、範囲外のオフセットでは `None` を返す。人間向けの 1 始まりの位置と LSP の UTF-16 位置は、整形/アダプターの責務とする。

## Phase 7: Regression, Property, And Fuzz Handoff -> Done

段階的な字句解析器の API が安定した後、より広いリグレッションカバレッジを追加します。

推奨するテスト群:

- 生スキャンのパニックや非決定性に対する、コミット済みの最小化されたファズリグレッション;
- スパンのカバレッジと、連結/再トークン化の不変条件のためのプロパティテスト;
- 生成されたユーザーシンボルの重なりのケース;
- 生成されたインポート衝突のケース;
- 形式が安定したときの、生ストリームと最終トークンストリームのスナップショットテスト.

昇格の規則:

- 生成またはファズで発見された失敗は、コミット済みコーパスに入れる前に、最小化し、安定した人間に読める名前を付け、`.expect.toml` と対にし、`tests/coverage/spec_trace.toml` から参照する。

完了条件:

1. 仕様書 `doc/spec/en/02.lexical_structure.md` を満たしていることをテストできているか?
2. リファクタリングは行ったか?
3. エラー処理を含めているか、また負例をテストに含めたか?
4. 複雑な複合ケースをテストに含めているか?
5. アルゴリズムをレビューしたか

アルゴリズムレビューの内容は、この計画書に重複して書かず、各モジュール設計文書に記録します。

- [raw_lexer.md](./raw_lexer.md): ソース前処理、生スキャン、予約シェルの曖昧性解消の流れ。
- [import_prescan.md](./import_prescan.md): インポート専用のトークン分割器、文パーサー、回復戦略。
- [lexical_environment.md](./lexical_environment.md): アクティブ環境の構築、検証、探索、フィンガープリント。
- [scope_skeleton.md](./scope_skeleton.md): フレーム構築、束縛子の有効範囲の割り当て、回復、上書きの意味論。
- [disambiguator.md](./disambiguator.md): 生トークンの処理、候補選択、パーサーコンテキストによる絞り込み、文字列の扱い、回復。

## Review Checklist For New Lexer Tests

新しい字句解析器のテストをコミットする前に確認すること。

- そのテストは、健全に確認できる最も早いパイプライン段階を対象にしているか。
- その動作は、一時的な実装の欠落ではなく、安定した言語契約か。
- そのモジュールが明示的な対象でない限り、フィクスチャはパーサー・リゾルバ・型検査器・インポートしたライブラリの意味論に依存していないか。
- サイドカーは `schema_version`, `id`, `kind`, `stage`, `domain`, `source`, `expected_outcome`, `expected_phase`, `diagnostic_codes`, `spec_refs` を含むか。
- すべての `spec_refs` の項目が `tests/coverage/spec_trace.toml` に存在するか。
- マニフェストはサイドカーへ逆参照しているか。
- フィクスチャは十分に小さく、安定したスネークケースの名前を持つか。
- 診断とトークンの期待値は決定的か。
