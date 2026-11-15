# Code actions
正本: [英語版](../en/code_action.md)。英語版と同期した日本語訳。

## Responsibility

LSP は architecture12 の [Fix Suggestions](../../architecture/en/12.diagnostics_and_lsp.md#fix-suggestions) と [Code Actions](../../architecture/en/12.diagnostics_and_lsp.md#code-actions) に従い、既存の構造化 fix 提案を editor の code action に変換する。提案の生成は Diagnostics が所有し、LSP は認証済み producer 登録を通して action を提示し、diagnostic payload schema、証明受理、artifact を変更しない。

## Request sources and conversion

[request source](../../architecture/en/12.diagnostics_and_lsp.md#request-sources) である `FixSuggestion`、`missing_facts.json`、explanation ref を使う。document version と range を持つ要求に対し、[5 段階の flow](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#code-actions) に従い、現在の snapshot の診断とその提案を検索し、既存の [診断公開](diagnostics.md#build-index-publication) に対応付ける。

並行する fix schema を定義せず、producer の既存 `FixSuggestion` payload を消費する。title、applicability、与えられた edit または任意の command が提示する action を表す。[internal03 の Fix Suggestion](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#fix-suggestion) に従い、与えられた `required_snapshot` と `required_text_hash` を含む既存の前提条件を保持する。この変換は fix 種別、field、payload の形を追加しない。

| Action class | Conversion and safety |
|---|---|
| Local text edits | `qua` や annotation の挿入、typo の置換など、提案の明示的 edit を提示する。range と expected text の一致を必要とする。 |
| Artifact-assisted edits | lemma の追加や citation の調整では、提案と `missing_facts` または explanation ref を使う。artifact と source の hash の一致を必要とし、関連 handle を検証する。 |
| Commands | explain、VC panel の表示、再検証など、与えられた command を提示する。snapshot と diagnostic handle を捕捉し、実行時に再検証する。command が後で検証しない限り source edit は行わない。 |

広範囲な書き換えより小さな明示的 edit を優先する。証明提案では backend 由来の candidate と kernel が検証した `used_axioms` を区別し、candidate を検証済みの証明事実としない。

## Applicability and freshness

提案の applicability とその class の検証が現在の snapshot に対して成立する場合だけ action を提示する。source text に依存するすべての edit は target range と expected text を検証し、artifact-assisted edit はさらに artifact hash、source hash、関連する explanation または `missing_facts` handle を検証する。この規則を filtering の基準とし、新たな applicability policy は追加しない。

対応する source snapshot と既存の [range 変換](range.md) を LSP UTF-16 境界で使う。[snapshot association](snapshot.md#source-association) を保持し、古い source の提案 range と現在の buffer の text や map を組み合わせない。

[incremental update flow](../../architecture/en/12.diagnostics_and_lsp.md#incremental-update-flow) に従い、snapshot の置換時には旧 snapshot の stale な code action を無効化する。current action として提示せず、保持された診断や explanation handle によって stale edit を許可しない。

## Explicit application

server は code action を提示し、提案 edit を自ら適用しない。適用可能な range/text と artifact/source の検証が通った後、editor の明示的な code-action 適用経路だけで edit を適用する。提示時に検証済みでも、前提条件が stale になった後の適用は許可されない。command にも実行時の handle 再検証と source-edit 検証の規則を保持する。

## Implementation boundary

承認済みの hash・provenance・command 契約は [validator contracts](code_action_validators.md) が所有し、有効化には認証済み producer 登録と validator が必要です。

`register_deferred_fix` は publication、diagnostic、canonical fix に不変の producer 権威を関連付けます。`DeferredActionProducer` が対応する backing encoding と認証済み missing-facts selector を所有し、LSP が承認済み hash、元の source、manifest 到達可能な artifact dependency を全て検証します。hash 宣言は Session source-text の byte domain を用い、登録の欠落・曖昧さ・未対応・不一致はフィルタします。locator は完全な canonical fix にも結び付き、fix id の再利用では変更内容を認可しません。

`enable_diagnostic_commands` は初期化前に bounded 実行 host を設定し、`mizar.executeDiagnosticCommand` のみを advertise します。`handle_message_with_diagnostic_commands` は metadata と実行を処理します。resolve が単一 locator argument を持つ固定 command を再構築します。実行は再検証後に保持した ExplanationStore を読み、返却前に currentness と cancellation を再確認します。preview-only/query-service backing はこの境界で action 権威を持ちません。VC 表示・reverify は検証済み data と atomic effect admission を提供する操作所有者が現れるまでフィルタします。暗黙の編集や client の操作/path 引数は受け入れません。

`src/diagnostics.rs` は既存の metadata/build host を通して公開済み fix を変換する。publication host のない通常の stdio 経路は action を返さない。要求では生きている文書の version と元の source association を使う。primary 診断 range の半開区間の重なりで fix を選び、空 range は非空区間内の点、または一致する点として扱う。

server は quickfix の resolve を通知する。client は code-action data と提示する edit/command の resolve に対応する必要があり、初回 action は title/kind/data だけを持つ。locator は不透明な process-local publication 識別子に結び付き、build ID、raw text、文書 version が同じでも識別子を再利用しない。`codeAction/resolve` は data を locator として現在の index から action を再構築し、再検証後だけ version 付き `edit.documentChanges` または固定 command を返す。利用不能または置換済み action は ContentModified で失敗する。client が渡す edit、command、診断内容は権限の根拠にならない。

明示された expected text が一致する local/snapshot-bound 提案は、edit を持つ 3 種の applicability で提示できる。informational 提案、未知の class、および必要な認証済み登録または validator がない hash/artifact/command 提案は除外する。所有する validator がない限り、不透明な hash を source hash や artifact hash と解釈しない。新しい action class、公開 diagnostic code/name/alias、explanation query の意味は追加しない。

この規則は、未決の OpenBuffer artifact provenance/storage と Session SourceId encoding から独立して、既存の提案と snapshot association を消費する。それらを前提とも決定ともせず、producer、diagnostics、artifact、snapshot の挙動を変更しない。

## Test design

task17 の test は既存の fix safety/applicability class、遅延 resolve、version 付き複数 source edit、独立した raw UTF-16 edit round-trip、不正/stale な入力の拒否を検証する。既存の production 診断はまだ fix を付けないため、native publication test は currentness を独立して検証する。frame 付き stdio では publication が利用できない経路を検証する。Task38 の typed consumer test が認証済み hash/artifact/backing 登録と command 再構築・再検証を扱い、native payload 配信は Task39 が所有する。未対応 producer と効果を伴う command の所有者は保留し、§23.6/SCA-003 の credit は変わらない。

[Native producer の前提](native_payload_producers.md) は承認済み Q1 A/Q2 A/Q3 B/Q4 A/Q5 A と別の [ranged host adapter](ranged_diagnostic_host.md) に従い、Task39 再 dispatch は producer 実装後です。
