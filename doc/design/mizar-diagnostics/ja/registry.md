# Diagnostic Registry

> 正本は英語です。英語版:
> [../en/registry.md](../en/registry.md)。

## 目的

`mizar-diagnostics` は `DiagnosticCode` registry を所有する。registry は、
共有 diagnostics model を通じて公開される各 diagnostic に安定した機械可読
identity を与え、コードが別の意味に再利用されていないことを検証し、
rendering、aggregation、fix、explanation、外部 consumer のための小さな
metadata を公開する。

この task は
`doc/spec/en/22.error_handling_and_diagnostics.md` に列挙された diagnostic の
ための初期 stable code space を定義する。build failure、cache/artifact
compatibility、以下の割当以外の Documentation、Extraction、editor-only overlay など、diagnostic に
言及する他の architecture surface は、normative range と descriptor がここに追加
されるまで code-space gap として残る。`mizar-diagnostics` はそれらの将来の割り
当てについても owner であり続けるが、task 3 は placeholder range や descriptor
を発明してはならない。

registry は phase semantics、proof acceptance、trusted status、kernel
acceptance、artifact mutation、LSP protocol conversion、driver session
orchestration を所有しない。これらの service は registry metadata を消費して
よいが、message text、localized text、ordering、semantic name ではなく
`DiagnosticCode` を key にしなければならない。

## コードの形

`DiagnosticCode` は `Xnnnn` の形を持つ。

- `X` は severity prefix であり、`E`、`W`、`I` のいずれかである。
- `nnnn` は 4 桁の 10 進数である。
- prefix と数値は stable identity の一部である。
- 番号は恒久的に割り当てる。retired になった番号を再発行してはならない。

コード範囲は言語仕様に従う。この表の `PhaseFamily` label が descriptor と
test で用いる canonical registry vocabulary である。

| 範囲 | `PhaseFamily` | Scope | 既定 severity |
|---|---|---|---|
| `E0001`-`E0099` | `Syntax` | Lexical and syntax diagnostics | Error |
| `E0100`-`E0199` | `Type` | Type diagnostics | Error |
| `E0200`-`E0299` | `Resolution` | Resolution, overload, and template diagnostics | Error |
| `E0300`-`E0399` | `Proof` | Proof and ATP diagnostics | Error |
| `E0400`-`E0499` | `Logic` | Logical consistency and verification-condition diagnostics | Error |
| `E0500`-`E0599` | `Algorithm` | Algorithm verification diagnostics | Error |
| `E0600`-`E0699` | `SourceLoad` | ソース読込みエラー | Error |
| `W0001`-`W0099` | `StructuralWarning` | Structural warnings | Warning |
| `W0100`-`W0199` | `ProofWarning` | Proof and ATP warnings | Warning |
| `W0200`-`W0299` | `AlgorithmWarning` | Algorithm and contract warnings | Warning |
| `W0300`-`W0399` | `CompatibilityWarning` | Compatibility and packaging warnings | Warning |
| `W0400`-`W0499` | `Documentation` | Documentation attachment and Markdown/tag warnings | Warning |
| `I0001`-`I0099` | `Info` | Informational display diagnostics | Info |

これらの範囲外の実装 diagnostic は registry error であり、通常の user
diagnostic ではない。

## Descriptor Metadata

active または retired の各 code は descriptor を 1 つ持つ。

| Field | 必須 | 互換性上の役割 |
|---|---:|---|
| `code` | yes | 公開 stable identity。割り当て後は immutable。 |
| `meaning_key` | yes | 互換性検証用の安定した非 localized registry key。user-facing identity として表示しない。 |
| `semantic_name` | yes | dot-separated の人間向け名。`meaning_key` が同じで、古い名前を alias として記録するなら rename できる。 |
| `default_severity` | yes | code prefix と一致しなければならない。descriptor の default severity を変更するには、言語仕様が code range を変えない限り新しい code が必要である。policy layer は registry identity の外で warning を build failure に昇格したり consumer-specific presentation を選んだりしてよい。 |
| `phase_family` | yes | 割り当て範囲と一致しなければならない。別 family へ移すには新しい code が必要である。 |
| `summary` | yes | registry audit 用の短い英語 summary。identity を変えずに明確化できる。 |
| `doc_url` | yes | canonical documentation URL または repository-relative documentation target。stable redirect または replacement link を保つなら移動できる。 |
| `status` | yes | `active` または `retired`。 |
| `since` | yes | code を初めて割り当てた version または design task。 |
| `retired_since` | retired の場合 | code を retired にした version または design task。 |
| `replacement_codes` | optional | retired diagnostic を置き換える code。これは提案であり identity は移転しない。 |
| `aliases` | optional | 同じ `meaning_key` の過去の semantic name。 |

message text、localized text、renderer layout、CLI ordering、LSP severity
mapping、fix suggestion wording、explanation text は diagnostic record からの
projection である。これらは registry identity ではない。

初期 registry では、row が明示的に override しない限り次を適用する。

- `meaning_key` は `semantic_name` と同じである。
- `status` は `active` である。
- `since` は `spec-22.7-v1` である。
- `doc_url` は
  `doc/spec/en/22.error_handling_and_diagnostics.md#227-error-code-reference`
  である。
- `aliases`、`retired_since`、`replacement_codes` は空である。
- `summary` は initial-allocation table の `Summary` 値である。

## Documentation attachment allocation

[spec22 §22.6.8](../../../spec/ja/22.error_handling_and_diagnostics.md#2268-documentation-attachment-warnings) は添付 W0400–W0404、[§22.6.9](../../../spec/ja/22.error_handling_and_diagnostics.md#2269-documentation-markdown-and-tag-warnings) は Markdown/tag W0405–W0411 の name・meaning・summary を所有する。両者は `Documentation`、`Warning`、`meaning_key = semantic_name`、active と空の alias/retirement/replacement を使う。添付は `since = spec-22-documentation-v1` と §22.6.8 URL、Markdown/tag は `since = spec-22-documentation-markdown-tags-v1` と §22.6.9 URL。W0416–W0499 は未割当。profile 昇格は descriptor identity を変えない。両者は [record 位置と詳細](./failure_record.md#documentation-primary-locations) を使い、parser/producer 実装と Extraction 割当は別作業。

[Link 割当 §22.6.10](../../../spec/ja/22.error_handling_and_diagnostics.md#22610-documentation-link-warnings) は W0412–W0414、`since = DOC10-SHARED-DIAGNOSTICS`、正本 spec24.2 `doc_url` を追加する。[共有契約](./documentation_links.md) が詳細と表示規則を所有する。

## Deferred Code-Space Gaps

次の surface は task-2 initial allocation の外に明示的に置く。

| Surface | Classification | task-2 disposition |
|---|---|---|
| build、cache、artifact、Extraction と以下の割当以外の Documentation 診断 | `external_dependency_gap` / `spec_gap` | 後続の仕様更新が canonical range と descriptor を追加するまで、code を割り当てず、current record として公開しない。 |
| この crate より前からある lexer、frontend、parser、resolver diagnostics | `external_dependency_gap` / `deferred` | task 16 が実 adoption trigger を記録するまで local diagnostics を触らない。message text から code を推測しない。 |
| `info.thesis`、`info.resolution`、`info.type` などの spec-21 display annotation name | `spec_gap` | `I0001`-`I0099` を reserve する。言語仕様がそれらの semantic name を数値 `DiagnosticCode` に map するまで concrete info code を割り当てない。 |

定義済み initial range の外にある unknown code や descriptor は、現在の registry
では validation failure である。この failure は code space がまだ仕様化されて
いないことを意味し、ad hoc な local range を認めるものではない。

## 割り当て規則

1. diagnostic を生む normative な言語仕様または architecture requirement から、
   かつこの document で既に定義済みの range から phase family を選ぶ。range が
   定義されていない場合は、placeholder を割り当てず code-space gap を記録する。
2. その範囲内で未使用の番号を割り当てる。gap は意図的であり、関連する将来の
   diagnostic のために残してよい。
3. 新しい `meaning_key`、semantic name、default severity、summary、
   documentation target を持つ descriptor を追加する。
4. 割り当て済み descriptor set を固定する registry test を追加または更新する。
5. 外部 adoption placeholder、将来の LSP conversion shape、実 producer/consumer
   seam を持たない driver event のために code を割り当ててはならない。

既存の lexer、frontend、parser、resolver diagnostic を後で移行する場合、その
移行は実 producer output を割り当て済み code に map しなければならない。古い
message string から identity を推測してはならない。

## Retirement Rules

verifier がその diagnostic meaning を emit しなくなったとき、code は retired に
なる。retirement は `status = retired`、`retired_since`、任意の
`replacement_codes` とともに descriptor を保持する。

retired code は historical artifact、cache、log、explanation handle のために
valid なまま残る。新しい current diagnostic は、compatibility mode が historical
data を明示的に読む場合を除き retired code を emit してはならない。その場合でも
record は consuming layer によって historical または stale と mark されなければ
ならない。

retirement は番号を allocator に戻さない。

## Compatibility Validation

registry implementation は locked built-in registry に対して descriptor
compatibility を検証しなければならない。

- code は消えてはならない。削除ではなく `retired` にする。
- retired code を active に戻してはならない。同じ diagnostic meaning が
  retirement 後に復活する場合は、新しい code を割り当て、それを
  `replacement_codes` に列挙する。
- code は prefix、numeric value、phase family、default severity、
  `meaning_key` を変えてはならない。
- semantic-name rename は、古い名前を `aliases` に保持する場合にだけ許される。
- documentation target の移動は、descriptor が同じ diagnostic meaning を指し
  続ける場合にだけ許される。
- `summary` text は identity を変えずに明確化できる。
- 2 つの active descriptor が同じ current `(phase_family, semantic_name)` を共有
  してはならない。`aliases` は同じ code の過去の名前を保存するだけであり、重複
  active semantic name を許可せず、consumer identity path でもない。
- 1 つの `phase_family` の中では、すべての active `semantic_name` とすべての
  active alias 文字列は、combined lookup domain 全体で一意でなければならない。
  current `semantic_name` は別 descriptor の alias を再利用してはならず、alias は
  別 descriptor の current name や alias を再利用してはならない。

invalid または reused diagnostic code は compiler implementation error である。
registry validation test または developer-mode internal error として報告され、
通常の user diagnostic にはならない。

## Initial Allocations

初期 active registry は、実装済み部分について
`doc/spec/en/22.error_handling_and_diagnostics.md#227-error-code-reference`
の canonical code reference を反映する。Info code は display diagnostic が数値
`DiagnosticCode` とともに normative に列挙されるまで reserved のままとする。
producer/consumer 実装をまだ持たない spec-reserved row は active table の後に
記録し、Rust registry と consumer を更新する所有 diagnostics adoption task が
着地するまで emit してはならない。

| Code | Semantic name | `PhaseFamily` | 既定 severity | Summary |
|---|---|---|---|---|
| `E0001` | `syntax.unexpected_token` | `Syntax` | Error | Unexpected token in current syntactic context |
| `E0002` | `syntax.malformed_literal` | `Syntax` | Error | Numeric or string literal does not conform to lexical rules |
| `E0003` | `syntax.unexpected_end_of_file` | `Syntax` | Error | File ended with open construct pending |
| `E0010` | `syntax.missing_end` | `Syntax` | Error | Block opened without matching `end` |
| `E0011` | `syntax.unmatched_delimiter` | `Syntax` | Error | Parenthesis, bracket, or `do` without matching close |
| `E0012` | `syntax.reserved_keyword_as_identifier` | `Syntax` | Error | Reserved keyword used as identifier |
| `E0101` | `type.mismatch` | `Type` | Error | Expression type incompatible with required type |
| `E0102` | `type.narrowing_requires_proof` | `Type` | Error | Narrowing coercion without justification |
| `E0103` | `type.sethood.missing` | `Type` | Error | Fraenkel comprehension for type without `sethood` |
| `E0110` | `type.inference_conflict` | `Type` | Error | Conflicting type constraints within an expression |
| `E0120` | `type.mode_mismatch` | `Type` | Error | Mode incompatibility without registered widening |
| `E0121` | `type.attribute_required` | `Type` | Error | Required attribute not registered for the type |
| `E0122` | `type.attribute_contradiction` | `Type` | Error | Attribute combination rejected: mutually exclusive attributes or missing existential cluster |
| `E0201` | `resolve.ambiguous_symbol` | `Resolution` | Error | Two or more equally-ranked overload candidates |
| `E0202` | `resolve.no_viable_overload` | `Resolution` | Error | No candidate survives type-checking |
| `E0203` | `template.argument_omitted_not_inferable` | `Resolution` | Error | Template schema parameter cannot be inferred |
| `E0204` | `resolve.incompatible_refinement_join` | `Resolution` | Error | Same-root redefinitions expose incompatible joined facts |
| `E0301` | `proof.by.search_exhausted` | `Proof` | Error | `by` step: ATP exhausted resource budget |
| `E0302` | `proof.by.missing_fact` | `Proof` | Error | Goal likely provable but required lemma not in scope |
| `E0303` | `proof.obligation.open` | `Proof` | Error | Proof block closed with goals remaining |
| `E0310` | `proof.counterexample.found` | `Proof` | Error | Counterexample model found for the goal |
| `E0320` | `proof.atp.timeout` | `Proof` | Error | All ATP backends timed out |
| `E0321` | `proof.atp.axiom_budget_exceeded` | `Proof` | Error | Axiom set exceeds `max_axioms` limit for the obligation |
| `E0350` | `proof.kernel.unsupported_evidence` | `Proof` | Error | Legacy or backend proof material is unsupported under normal proof policy |
| `E0351` | `proof.kernel.missing_provenance` | `Proof` | Error | Kernel evidence is missing required provenance or context binding |
| `E0352` | `proof.kernel.invalid_substitution` | `Proof` | Error | Explicit substitution evidence failed kernel side conditions |
| `E0353` | `proof.kernel.invalid_sat_refutation` | `Proof` | Error | Kernel-derived SAT refutation check failed |
| `E0401` | `logic.contradictory_axioms` | `Logic` | Error | ATP derived `False` from declared axioms |
| `E0410` | `logic.circular_definition` | `Logic` | Error | Non-recursive definition refers to itself outside an algorithm block |
| `E0411` | `logic.circular_cluster` | `Logic` | Error | Cluster registration creates attribute inheritance cycle |
| `E0420` | `vc.postcondition.return` | `Logic` | Error | `ensures` not provable at `return` site |
| `E0421` | `vc.assert.failed` | `Logic` | Error | `assert` in algorithm body not provable |
| `E0422` | `vc.precondition.call_site` | `Logic` | Error | Callee `requires` clause not provable at call site |
| `E0423` | `vc.loop.establish` | `Logic` | Error | Loop invariant not provable before first iteration |
| `E0424` | `vc.loop.maintain` | `Logic` | Error | Loop invariant not provable to be preserved |
| `E0425` | `vc.loop.decrease` | `Logic` | Error | Termination measure not provably decreasing |
| `E0426` | `vc.recursion.decrease` | `Logic` | Error | Termination measure not provably decreasing at recursive call |
| `E0430` | `logic.cluster.inconsistency` | `Logic` | Error | Cluster registration creates contradiction |
| `W0001` | `warn.unused_variable` | `StructuralWarning` | Warning | Variable declared but never read |
| `W0002` | `warn.unused_definition` | `StructuralWarning` | Warning | Definition never referenced in package |
| `W0003` | `warn.unused_hypothesis` | `StructuralWarning` | Warning | Proof hypothesis never referenced in subsequent steps |
| `W0010` | `warn.deprecated_syntax` | `StructuralWarning` | Warning | Deprecated construct; replacement provided |
| `W0101` | `warn.redundant_hypothesis` | `ProofWarning` | Warning | `by` clause contains fact not used by accepted evidence |
| `W0102` | `warn.externally_attested_proof` | `ProofWarning` | Warning | External backend success without kernel-accepted evidence |
| `W0103` | `proof.citation.unused` | `ProofWarning` | Warning | Explicit citation absent from kernel-accepted `used_axioms` |
| `W0201` | `warn.unreachable_code` | `AlgorithmWarning` | Warning | Statement unreachable under static control-flow analysis |
| `W0202` | `warn.loop_may_not_terminate` | `AlgorithmWarning` | Warning | `terminating` algorithm with unverified loop measure |
| `W0210` | `warn.weakened_postcondition` | `AlgorithmWarning` | Warning | `ensures` weaker than what the verifier can prove |
| `W0301` | `compat.breaking_change` | `CompatibilityWarning` | Warning | Public API change requires a MAJOR bump |
| `W0302` | `compat.feature_addition` | `CompatibilityWarning` | Warning | Backward-compatible API addition requires a MINOR bump |
| `W0303` | `compat.overload_resolution_shift` | `CompatibilityWarning` | Warning | Registration, redefinition, or conditional-cluster change may shift overload/refinement resolution (heuristic MAJOR) |
| `W0304` | `compat.version_bump_insufficient` | `CompatibilityWarning` | Warning | Declared version bump smaller than required |
| `W0305` | `compat.edition_increase` | `CompatibilityWarning` | Warning | Package edition raised; MAJOR by default, review recommended |

Deferred spec-reserved rows:

| Code | Semantic name | Owner condition |
|---|---|---|
| `E0205` | `resolve.ambiguous_redefinition_target` | checker task 44 で Chapter 22 に追加。checker/resolver の redefinition-target diagnostic producer と source-derived payload が存在するときだけ、active Rust registry に採用する。 |

## Lookup Contract

`DiagnosticCode` による registry lookup は active code と retired code の
descriptor metadata を返す。current semantic name による lookup は人間向け
tooling と test のためにだけ許され、build tool や consumer の identity path に
してはならない。alias lookup は audit aid だけであり、二つ目の descriptor では
なく、一意な owning `DiagnosticCode` を返す。compatibility validation は
alias/current-name collision を拒否するため、alias lookup は deterministic でなければ
ならない。validation を通っていない registry を caller が渡した場合、alias lookup
は code を選ぶのではなく失敗しなければならない。

unknown code、malformed code string、compatibility validation に失敗した
descriptor は registry failure である。aggregation と rendering は unknown code
の descriptor を捏造してはならない。

## Public Enum Compatibility

Task 18 は registry-owned public enums を downstream forward compatibility のため
`#[non_exhaustive]` として mark する。

- `DiagnosticSeverity`;
- `PhaseFamily`;
- `DiagnosticCodeError`;
- `DiagnosticStatus`;
- `RegistryValidationError`。

`DiagnosticSeverity` は architecture 19 に従う。`Error`、`Warning`、`Info` の既存の
machine-readable meaning と sort order は compatibility surface であり、variant の
追加または reclassification には compatibility review と test update が必要である。
consumer は severity を message text、localized rendering、または phase success と混同しては
ならない。

`PhaseFamily` は registry ownership metadata であり、phase success、driver orchestration、
proof acceptance、または kernel acceptance を決定しない。`DiagnosticStatus` は registry
lifecycle metadata であり、artifact mutation や publication を決定しない。
`#[non_exhaustive]` marker は external matching compatibility のためだけのものであり、
internal validation は deliberate review のため exhaustive match を続けてよい。

ソース読込みコード E0600–E0603 は仕様 22.1.2/22.7 に従い、`since` は `spec-22-source-load-v1` とする。既存コードの意味とメタデータは維持する。

## Frontend allocations

E0013–E0052は [仕様22.7](../../../spec/ja/22.error_handling_and_diagnostics.md#227-エラーコードのリファレンス) の名前・意味に従い、summaryは英語正本を用いる。`meaning_key = semantic_name`、Syntax family、Error severity、Active descriptor status、`since = spec-22-frontend-v1` とする。既存メタデータは変更せず、上記の初期割当表を補う。
registry参照と共有draft/record構築を可能にするが、登録だけではfrontend生成元を接続しない。診断発行には仕様22.2.3に従うfrontend所有の分類対応・ソース結合済み変換・生成元テストが引き続き必要である。registryテストは40行を英語仕様と照合し、共有record構築を検査する。

## Import allocations

E0220–E0225 は仕様22.7の名前と英語正本のsummaryを用い、`meaning_key = semantic_name`、Resolution family、Error severity、Active status、`since = spec-22-import-v1` とする。他のメタデータは初期既定値に従う。登録は共有draft/record構築のみを可能にする。E0220–E0224 のソース結合済み発行は[driver の継続境界](../../mizar-driver/ja/frontend_adapter.md#診断専用-import-継続の設計)に従う。E0225の発行は、完全なグラフ全体の入力と、認証済みソースに結合した循環報告が利用可能になるまで延期する。

## Reflection Diagnostic Adoption

仕様予約 E0104/E0111/E0112/E0206/E0332/E0333 の descriptor を、§22.7 の厳密な名前/summary と
上記の初期メタデータ既定値、範囲に対応する Type/Resolution/Proof family で準備する。B1-B8 は分類記録でありコードではない。
共有 registry lookup と draft/record 構築のみを可能にする。以下の source 由来の採用テストは保留する。
採用はソース付きの所有段階記録を消費する。descriptor だけで発行や corpus coverage を有効化しない。
宣言/使用、欠落する局所条件/モード/C、候補/残存位置、最初の未適用 rule または未完了段階を報告する。
テストは source 宣言/使用位置、ghost の runtime 流入、Binder/C と valuation 形成、candidate/残留位置、runtime/import/inspection の limit stage を検査する。未証明/偽、fail/未完、guard 付き valid_at、import 拒否後の返却保持を区別する。
既存の syntax/runtime/verification コードの役割を保ち、推測/新規 ID と未許可の期待値変更を行わない。

[URL 出力診断](./documentation_urls.md) が厳密な descriptor の W0415 を追加する。残る render 診断は gate 内。

## Missing Packaged Source Warning Proposal
[英語正本](../en/registry.md#missing-packaged-source-warning-proposal)に対応する定義のみ。architecture13 の既存の「source 未同梱で source link が利用不可なら warning」という意図を限定する。割当・予約・採用・発行は行わず、W0416–W0499 は未割当のまま。[Gate 4](../../mizar-doc/en/render_prerequisites.md#ordered-readiness-gates) に正式な割当・location 採用・producer 由来の発行/profile witness を残す。

### Trigger Evidence
source/publication owner が、選択された module の元 source が未同梱で、その documentation source link が利用不可であることを明示的に確立する必要がある。選択 publication 内の当該 module source を網羅する証拠を、既存 selected-plan/source owner を通じて厳密な publication hash、完全な module/package/version/lock/edition identity、現在の request/snapshot/options に認証して結合する。caller の申告、表示 path、digest 単独、source-free artifact read は証拠にならず、source fetch・filesystem/network 検索・packaging 推測を行わない。
availability 不明は未同梱ではない。optional source bundle の省略、null/legacy original-text projection、original digest 欠落、comment projection 利用不可、coordinate map 欠落/部分状態だけでは警告しない。認証済み text があっても source location がない場合は location gap のまま。[Q3 source-slot 表示](../../mizar-doc/en/render.md#q3-source-view-and-line-routing-definition)は変えない。証拠の欠落・他主体・古さ・矛盾・不完全さから draft を作らず、既存 lower-reader refusal は error を保つ。現在の unavailable-source API は明示的な未同梱証拠を供給せず、その producer 定義・認証は別途残る。

### Candidate Descriptor And Identity
候補は W0416、semantic name/不変 meaning key は `documentation.render.missing_packaged_source`、summary は `Documentation source is not packaged`、family `Documentation`、default `Warning`、phase `Documentation`、category `DocumentationWarning`。候補 lifecycle は active、aliases/retirement/replacements は空。`doc_url = doc/design/mizar-diagnostics/en/registry.md#missing-packaged-source-warning-proposal`。`since` は後続の実割当 revision/task を示し、この定義 task にはしない。registry entry や W0416 発行許可ではない。
`stable_detail_key = semantic_name` と既存 String/List detail 一つ `documentation.render.source = [publication, module, "not_packaged"]` を用いる。publication/module は[共有 identity encoding](../en/documentation_links.md#structured-identity-and-details)の小文字 publication hash と、optional 値の区別を保持した完全 module tuple。item target・occurrence・block ordinal・推測 offset・caller path・message は subject に含めない。同じ publication/module の利用不可 source link 全体で一警告とし、item/block/worker/route の反復で増やさない。別 publication/完全 module identity は区別する。

### Proposed Locations, Precedence And Profiles
当該 descriptor に既存 [DocumentationScope location](../en/failure_record.md#shared-link-provenance) の適用を提案し、認証済み publication/完全 module identity と既存 module-scope primary 表示を保つ。block・SourceId・span/EOF/挿入点・行列・underline・excerpt を捏造しない。現行 [spec22 applicability](../../../spec/en/22.error_handling_and_diagnostics.md#2212-source-span-and-context-display) は link completeness を対象とし、missing-source 適用は後続の正式採用を要する。record variant/API は増やさない。共有 module source key、scope-before-block 順序、descriptor/detail comparator、deduplication、snapshot freshness を変えず、message を identity にしない。
lower integrity/schema/binding/currentness refusal は依存する missing-source draft を抑止する。source 欠落だけで W0411 conversion failure、W0412 unresolved、W0413 ambiguous、W0414 incomplete-link-input、W0415 rejected-URL 条件を作らない。独立に確立した link/URL・Doc8 警告は意味を保つ。Doc8 failed-view recovery は block 内に留まり、packaging 証拠を作らず、独立に確立した module source 警告も抑止しない。包括的 precedence や新しい build-refusal outcome は導入しない。
別途採用後に既存 warning control と `Phase16Profile.fail_on_doc_warnings` を適用する。default Warning/code identity は固定し、promotion は documentation quality のみを失敗させ得る。proof validity、published verification input、extraction acceptance と独立 warning policy は変えず、新 profile/suppression syntax は定義しない。
general readiness 診断の trigger/outcome と package-level provenance は未定義で対象外。完全 rendering/search、interface adoption、条件付き package-overview producer readiness、schemas/publication、Doc23/24、G4/G5、completion credit は別 gate のまま。
