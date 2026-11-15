# Code actions
正本: [英語版](../en/code_action.md)。英語版と同期した日本語訳。

## Responsibility

LSP は architecture12 の [Fix Suggestions](../../architecture/en/12.diagnostics_and_lsp.md#fix-suggestions) と [Code Actions](../../architecture/en/12.diagnostics_and_lsp.md#code-actions) に従い、既存の構造化 fix 提案を editor の code action に変換する。提案の生成は Diagnostics が所有し、LSP は producer、証明受理、artifact を変更せず action を提示する。

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

task16 は変換と安全性だけを定める。task9 の診断公開、task17 の code-action 実装、tasks18/19 の explanation query は別単位のままである。command の wire encoding、explanation query の挙動、新しい action class、公開 diagnostic code/name/alias はここでは定義しない。

この規則は、保留中の OpenBuffer artifact provenance/storage、Session SourceId encoding、registry activation の判断から独立して、既存の提案と snapshot association を消費する。それらを前提とも決定ともせず、producer、diagnostics、artifact、snapshot の挙動を変更しない。

## Test design

この設計単位では code も test も変更しない。fix 種別ごとの action fixture と edit の range 変換 round-trip の検証義務は task17 に残り、task9 と `mizar-diagnostics` task13 に依存する。task17 と module は未完了のままとし、§23.6/SCA-003 の credit は変わらない。
