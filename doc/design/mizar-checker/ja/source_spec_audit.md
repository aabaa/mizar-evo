# Source/Spec 対応監査: mizar-checker

> Canonical language: English:
> [../en/source_spec_audit.md](../en/source_spec_audit.md).

task 32 は task 31 後の checker public surface と仕様上の約束を監査する。
source behavior、`.miz` fixture、expectation、public API は変更しない。
未接続の挙動は現在の実装都合を正本化せず、明示的な
`external_dependency_gap`、`test_gap`、または `deferred` として分類する。

## 範囲と方法

inventory は `crates/mizar-checker/src/lib.rs` の現在の `pub mod` export、
`crates/mizar-checker/src/*.rs` の top-level public item、そして crate-local
`dense_id!` / `string_key!` macro で生成される public newtype をすべて含む。
public method は、module spec が table、builder、output API として記述している
ため、所有する public type の下にまとめる。

監査対象の module specification:

- [typed_ast.md](./typed_ast.md)
- [binding_env.md](./binding_env.md)
- [source_context.md](./source_context.md)
- [source_attribute.md](./source_attribute.md)
- [source_evidence.md](./source_evidence.md)
- [source_term.md](./source_term.md)
- [source_type.md](./source_type.md)
- [type_checker.md](./type_checker.md)
- [registration_resolution.md](./registration_resolution.md)
- [cluster_trace.md](./cluster_trace.md)
- [overload_resolution.md](./overload_resolution.md)
- [resolved_typed_ast.md](./resolved_typed_ast.md)

結果: 実装済み explicit-payload API について、blocking な `source_drift`、
`design_drift`、`source_undocumented_behavior` は観測していない。残る
source-derived behavior の未 coverage は、下記の gap row により意図的に
defer されている。

Post-audit source-derived bridge note: `mizar-test` は対応済み `.miz` source から
explicit checker-owned `BindingEnv`、`DeclarationInput`、`TypedAst`、
`ResolvedTypedAst` payload を構築する bounded reserve-only declaration bridge を
実行するようになった。successful pass execution は bare builtin reserve head に限られ、
task 55 の same-module no-argument local mode head のうち、unique / preceding /
unrecovered な mode definition が bare builtin RHS を持ち definition-local context を
持たないもの、および dependency mode がその accepted bare builtin RHS expansion を持つ
task 56 の one-edge same-module local mode chain、builtin `set` / `object` で終端する
task 74 の AST-bounded structural bare local-mode chain も successful pass execution に含まれる。
active fail slice は same-module
attributed builtin head を missing-evidence diagnostic へ、task 55/56/57/58/59/60/61/62/63/64/65/66/74 外の same-module local
mode head（mixed attributed/bare local mode source、attributed chain
dependency、task 74 の structural guard を満たさない chain を含む）を missing mode-expansion diagnostic へ、same-module local structure head と attributed local
structure head を missing evidence-query diagnostic へ運ぶ。task 57 はさらに、
RHS が local structure head である real same-module local-mode expansion を、missing
mode-expansion diagnostic ではなく同じ missing evidence-query diagnostic へ運ぶ。
task 58 はさらに、RHS が attributed builtin head である real same-module
local-mode expansion を、missing mode-expansion diagnostic ではなく同じ missing
evidence-query diagnostic へ運ぶ。
task 59 はさらに、同じ mode が bare reserve use と mixed でない場合に、real direct
bare-builtin RHS expansion を持つ attributed same-module local-mode reserve head を同じ
missing evidence-query diagnostic へ運ぶ。task 60 はさらに、同じ mode が bare reserve use と
mixed でない場合に、real direct local-structure RHS expansion を持つ attributed
same-module local-mode reserve head を同じ missing evidence-query diagnostic へ運ぶ。
task 61 はさらに、同じ mode が bare reserve use と mixed でない場合に、real direct
attributed-builtin RHS expansion を持つ attributed same-module local-mode reserve head を
同じ missing evidence-query diagnostic へ運ぶ。task 62 はさらに、same-module local
structure RHS で終端する one-edge bare local-mode chain を、同じ `SurfaceAst` から両方の
real mode-expansion payload を抽出したうえで同じ missing evidence-query diagnostic へ運ぶ。
task 63 も同様に、attributed builtin RHS で終端する one-edge bare local-mode chain を、
同じ `SurfaceAst` から両方の real mode-expansion payload を抽出したうえで missing
attributed-type evidence-query diagnostic へ運ぶ。task 64 は one-edge dependency chain が
bare builtin RHS に終端する attributed same-module local-mode reserve head を、同じ
`SurfaceAst` から両方の real mode-expansion payload と reserve-head attribute を抽出した
うえで同じ missing attributed-type evidence-query diagnostic へ運ぶ。task 65 は one-edge
dependency chain が same-module local structure RHS に終端する attributed same-module
local-mode reserve head を、同じ `SurfaceAst` から両方の real mode-expansion payload と
reserve-head attribute を抽出したうえで missing base-shape / constructor-witness と full
attributed-type evidence-query diagnostic へ運ぶ。task 66 は one-edge dependency chain が
attributed builtin RHS に終端する attributed same-module local-mode reserve head を、同じ
`SurfaceAst` から両方の real mode-expansion payload、reserve-head attribute、terminal
RHS attribute を抽出したうえで missing full attributed-type evidence-query diagnostic へ運ぶ。
Task 250はexact Task-67 structure-qualified extraction gapを、written qualifierと
authenticated structure/attribute provenanceをraw source-attribute handoffへ保持して
supersedeする。owner compatibility、admissibility、evidence、truthはdeferredのまま。
task 68 は checker payload が real type-argument と term-argument provenance をまだ
持たないため、argument-bearing same-module local mode reserve head を external
extraction-gap diagnostic へ運ぶ。
task 69 は checker payload が real type-argument と term-argument provenance をまだ
持たないため、argument-bearing same-module local structure reserve head を external
extraction-gap diagnostic へ運ぶ。
task 70 は checker payload が real bracket type-argument と `qua`-argument provenance を
まだ持たないため、bracket type-argument payload extraction や mode-head resolution の前に
bracket-form same-module local mode reserve head を external extraction-gap diagnostic へ運ぶ。
task 71 は checker payload が real bracket type-argument と `qua`-argument provenance を
まだ持たないため、bracket type-argument payload extraction や structure-head resolution の前に
bracket-form same-module local structure reserve head を external extraction-gap diagnostic へ運ぶ。
task 72 はさらに、builtin `set` / `object` で終端する real AST-derived two-edge bare
local-mode chain を既存の pass readiness path へ通す。task 73 は同じ source-derived
seam を builtin `set` / `object` で終端する three-edge bare local-mode chain へ昇格する。
task 74 は temporary depth cap を AST-bounded structural bare local-mode chain rule に置き換える。
task 75 は、後続 same-module local mode declaration を reserve head が先に名前参照する
case について lower-stage active-range boundary を記録する。frontend/resolver
processing は checker handoff 前に unresolved type expression を拒否するため、
future mode declaration は `ModeExpansion` payload へ変換されない。
task 76 は、後続 same-module local structure declaration を reserve head が先に
名前参照する case について対応する lower-stage active-range boundary を記録する。
frontend/resolver processing は checker handoff 前に unresolved type expression を
拒否するため、future structure declaration は checker structure type-head payload や
base-shape evidence query へ変換されない。
これは checker の新しい raw-syntax dependency ではなく、non-builtin declaration、
imported symbol、attribute / mode / structure argument、qualified attribute provenance、
bracket `type_arg_list` と `qua`-argument provenance、type-argument / term-argument
provenance、structure base-shape / full attributed-type
existential evidence、broader / imported / attributed /
argument-bearing / parameterized / contextual / ambiguous / cyclic
mode expansion、term、formula、overload、CoreIr、
ControlFlowIr、VC payload、proof evidence の AST-wide source-to-checker gap を閉じるものでもない。

## Crate Module Exports

`src/lib.rs` は次の checker-owned module だけを export する:

- `binding_env`
- `cluster_trace`
- `overload_resolution`
- `registration_resolution`
- `resolved_typed_ast`
- `source_context`
- `source_attribute`
- `source_evidence`
- `source_term`
- `source_type`
- `type_checker`
- `typed_ast`

根拠: `tests/lint_policy.rs` の
`checker_public_semantic_api_matches_documented_modules` がこの list を検査し、
crate が direct `mizar-syntax` import を持たないこと、resolver/session dependency
boundary を保つことも guard している。

## Public Surface Inventory

### `typed_ast`

生成 public newtype:

- `TypedNodeId`, `LocalTypeContextId`, `TypeEntryId`, `NormalizedTypeId`,
  `OpenCandidateSetId`, `TypeFactId`, `CoercionId`, `InitialObligationId`,
  `TypeDiagnosticId`
- `TypedNodeKind`, `TypeRole`, `TypePredicateRef`, `TypeRuleId`,
  `TypeAssumptionId`, `BuiltinRuleId`, `ResolutionStepId`,
  `InitialObligationGoal`, `InitialObligationProvenance`

literal top-level public item:

- `TypedAst`, `TypedAstParts`, `TypedNode`, `TypedNodeLinks`, `TypingState`,
  `NodeRecoveryState`, `TypedArena`, `TypedArenaBuilder`,
  `TypedArenaError`, `TypedSiteRef`, `TypedSubjectRef`, `LocalTypeContext`,
  `LocalTypeContextDraft`, `LocalTypeContextTable`, `TypeContextLayer`,
  `BindingTypeRef`, `ContextRecoveryState`, `TypeEntry`, `TypeEntryDraft`,
  `TypeTable`, `TypeStatus`, `TypeEntryActual`, `TypeProvenance`,
  `TypeFact`, `TypeFactDraft`, `TypeFactTable`, `Polarity`,
  `FactProvenance`, `FactStatus`, `CoercionEntry`, `CoercionDraft`,
  `CoercionTable`, `CoercionKind`, `CoercionStatus`, `CoercionProvenance`,
  `InitialObligation`, `InitialObligationDraft`, `InitialObligationTable`,
  `InitialObligationKind`, `InitialObligationStatus`, `TypeDiagnostic`,
  `TypeDiagnosticDraft`, `TypeDiagnosticTable`, `TypeDiagnosticClass`,
  `TypeDiagnosticSeverity`, `DiagnosticRecoveryState`, `SourceRangeKey`,
  `TypedAstError`

対応:

| 仕様上の約束 | source 根拠 | test 根拠 | 状態 |
|---|---|---|---|
| source-shaped typed AST root が node、context、type、fact、coercion、obligation、diagnostic side table を保持する。 | `TypedAst`, `TypedAstParts`, accessor/debug method in `src/typed_ast.rs`. | `arena_ids_are_dense_and_debug_rendering_is_stable`, `public_data_shapes_do_not_expose_proof_or_final_overload_fields`. | checker-owned data shape として実装済み。 |
| dense local id、arena validation、parent/child integrity。 | `TypedArena`, `TypedArenaBuilder`, `TypedArenaError`, generated dense ids. | `arena_validation_rejects_invalid_references_and_cycles`. | 実装済み。 |
| immutable local context snapshot と visible fact discipline。 | `LocalTypeContextTable`, `LocalTypeContextDraft`, `BindingTypeRef`, context recovery/status enum. | `local_context_snapshots_validate_parent_chain_and_visibility`, `context_validation_enforces_assumed_fact_visibility`. | 実装済み。 |
| type/fact/coercion/initial-obligation/diagnostic table は id と deterministic order を保持する。 | `TypeTable`, `TypeFactTable`, `CoercionTable`, `InitialObligationTable`, `TypeDiagnosticTable`. | `tables_round_trip_ids_and_deduplicate_facts`, `canonical_queries_are_deterministic`, `coercion_ordering_and_provenance_are_stable`. | 実装済み。 |
| partial typing と handoff boundary が明示される。 | `TypeStatus`, `FactStatus`, `CoercionStatus`, `InitialObligationStatus`, recovery enum. | `status_variants_preserve_partial_typing_and_handoff_boundaries`, validation rejection tests. | 実装済み。 |
| public enum は forward-compatible。 | public enum の `#[non_exhaustive]`。 | `checker_public_enums_are_forward_compatible_and_documented`. | task 31 で guard 済み。 |

### `binding_env`

生成 public newtype:

- `BindingContextId`, `BindingId`, `BindingDiagnosticId`

literal top-level public item:

- `BindingEnv`, `BindingEnvParts`, `BindingContextTable`,
  `BindingContext`, `BindingContextDraft`, `BindingContextOwner`,
  `BindingContextLayer`, `BindingContextRecovery`, `BindingTable`,
  `BindingEntry`, `BindingDraft`, `BindingKind`, `BinderIdentity`,
  `BindingTypeSite`, `BindingStatus`, `BindingRecoveryState`,
  `CapturedFreeVariables`, `BindingDiagnosticTable`, `BindingDiagnostic`,
  `BindingDiagnosticDraft`, `BindingDiagnosticClass`,
  `BindingDiagnosticSeverity`, `BindingDiagnosticRecovery`,
  `BindingLookupSite`, `BindingLookupResult`, `BindingEnvError`

対応:

| 仕様上の約束 | source 根拠 | test 根拠 | 状態 |
|---|---|---|---|
| Binding environment は resolver-boundary data layer であり、raw syntax walker ではない。 | `BindingEnv::try_new`, `BindingEnvParts`, resolver `SymbolEnv` inputs. | `public_module_shell_signature_stays_on_resolver_shell_boundary`, `module_shell_records_external_gaps_and_debug_rendering_is_stable`. | 実装済み。missing extraction は MC-G011。 |
| Context graph は parent、layer、ownership、recovery を検証する。 | `BindingContextTable`, `BindingContextDraft`, owner/layer/recovery enum. | `context_layers_and_validation_cover_parent_chain_and_recovery`, `validation_rejects_invalid_ranges_and_diagnostic_links`. | 実装済み。 |
| Lookup は deterministic、visibility-scoped、forward-reference aware。 | `BindingLookupSite`, `BindingLookupResult`, binding lookup methods. | lookup shadowing/ambiguity/tie-break/resolver-fallback tests. | explicit payload について実装済み。 |
| reserved variable、binder identity、closure metadata、diagnostic、rendering は stable。 | `BindingKind`, `BinderIdentity`, `CapturedFreeVariables`, diagnostic table/classes. | reserved/binder identity/diagnostic/canonical rendering tests. | 実装済み。full closure replay は deferred。 |
| public enum は forward-compatible。 | public enum の `#[non_exhaustive]`。 | `checker_public_enums_are_forward_compatible_and_documented`. | task 31 で guard 済み。 |

### `source_context`

生成 public newtype:

- `SourceItemId`, `SourceDeclarationId`

literal top-level public item:

- `SourceBindingContextInput`, `SourceItemInput`, `SourceItemRole`,
  `SourceItemVisibility`, `SourceItemRecovery`, `SourceBindingContextOwner`,
  `SourceBindingSiteInput`, `SourceBindingSiteRole`,
  `SourceBindingContextBuild`, `SourceBindingContextProjection`,
  `SourceBindingContextIncomplete`, `SourceBindingContextHandoff`,
  `SourceItemTable`, `SourceItem`, `SourceDeclarationTable`,
  `SourceDeclaration`, `SourceContextLinkTable`, `SourceContextLink`,
  `SourceBindingContextProducer`, `SourceContextError`

対応:

| 仕様上の約束 | source 根拠 | test 根拠 | 状態 |
|---|---|---|---|
| syntax-free source-item/binding-site projectionが実opaque resolver shellとsource orderを保持する。 | `SourceBindingContextInput`, `SourceItemInput`, `SourceBindingSiteInput`. | real Task 248 fixture、route isolation、corruption matrix assertion。 | exact Task 248 transactionについて実装済み。 |
| complete constructionは単一immutable checker-owned binding/context handoffをpublishする。 | `SourceBindingContextProducer`, `SourceBindingContextHandoff`, source/declaration/context-link table。 | projection equality、lookup、shadow link、`TypedAst`/`ResolvedTypedAst` preservation assertion。 | atomicに実装済み。 |
| recovered-emptyはincomplete、corrupt/partial/cross-linked inputはpublishせずrejectする。 | `SourceBindingContextBuild`, `SourceBindingContextIncomplete`, `SourceContextError`; `TypedAst` exact handoff validation。 | real-shell corruption/recovery/atomicity matrix。 | frozen branchについて実装済み。 |
| public enum は forward-compatible。 | public enum の `#[non_exhaustive]`。 | `checker_public_enums_are_forward_compatible_and_documented`. | exhaustive exceptionなしでguard済み。 |

bounded gap: broader canonical item/binder familyはexisting MC-G011/MC-G016
follow-up ownerのdeferred `test_gap`/`source_drift`に残る。same-identifier
re-reservationのreplacement/duplicate semanticsだけがnonblocking `spec_gap`である。
source term use-site lookup、Tasks 249+/269+ payloadもTask 248 authority外に残る。

### `source_attribute`

生成 public newtype:

- `SourceAttributeChainId`, `SourceAttributeId`,
  `SourceAttributeQualifierId`, `SourceAttributeArgumentGroupId`,
  `SourceAttributeArgumentId`

literal top-level public item:

- `SourceAttributeHandoffInput`, `SourceAttributeChainInput`,
  `SourceAttributeInput`, `SourceAttributePolarityInput`,
  `SourceAttributeQualifierInput`, `SourceAttributeArgumentGroupKind`,
  `SourceAttributePrefixForm`, `SourceAttributeArgumentGroupInput`,
  `SourceAttributeActualKind`, `SourceAttributeArgumentInput`,
  `SourceAttributeHandoff`, `SourceAttributeChainTable`,
  `SourceAttributeTable`, `SourceAttributeQualifierTable`,
  `SourceAttributeArgumentGroupTable`, `SourceAttributeArgumentTable`,
  `SourceAttributeChain`, `SourceAttribute`, `SourceAttributeQualifier`,
  `SourceAttributeArgumentGroup`, `SourceAttributeArgument`,
  `SourceAttributeProducer`, `SourceAttributeError`

対応:

| 仕様上の約束 | source根拠 | test根拠 | 状態 |
|---|---|---|---|
| syntax-free flat tableがnonempty source attribute chain、written polarity/`non`、qualifier、argument-group punctuation、actual order、semantic provenanceを保持する。 | `SourceAttributeHandoffInput`、dense id 5件、immutable table 5件、`src/source_attribute.rs`のrow accessor。 | exact Task-81/67/84/85 real-route cardinality/field assertionとsynthetic multi-attribute/prefix extractor probe。 | frozen Task 250 transactionについて実装済み。 |
| Task-249 source-type expression ownershipとresolver binding/symbol/contribution identityをpublish前にauthenticateする。 | `SourceAttributeProducer::build`が`SourceTypeApplicationHandoff`、`BindingEnv`、`SymbolEnv`、`TypedArena`をconsumeする。 | producer environment/ownership/symbol-kind/visibility/contribution/site-range corruption assertion。 | transactionalに実装済み。 |
| parent link、dense order、punctuation independence、source containmentはfail closedで、partial handoffをpublishしない。 | `SourceAttributeError`とproducerのchain/attribute/qualifier/group/actual validator。 | dangling/forward/order/punctuation/range/recovery corruptionとatomic-failure assertion。 | sort/repairなしで実装済み。 |
| `TypedAst`がresultを所有し、`ResolvedTypedAst`はclone-preserveだけを行う。 | optional `SourceAttributeHandoff` fieldとborrowed getter。 | immutable final-preservation/deterministic debug assertion。 | 実装済み。legacy empty debug byteはconditionalに維持。 |
| public enumはforward-compatible。 | public enum 5件すべての`#[non_exhaustive]`。 | `checker_public_enums_are_forward_compatible_and_documented`。 | exhaustive exceptionなしでguard。 |

bounded gap: このhandoffがpublishするのはraw source-written payloadだけである。
attribute arity/admissibility/owner compatibility、term binding/type/result、
normalized instance、evidence request/result、cluster fact/truth/closure、
accepted declaration/fact/proof、downstream IRはTasks 251+または既存explicit
ownerに残る。legacy `type_checker::AttributeInput`は変更せず別のままである。

### `source_type`

生成 public newtype:

- `SourceTypeApplicationId`, `SourceTypeExpressionId`,
  `SourceTypeArgumentId`

literal top-level public item:

- `SourceTypeHandoffInput`, `SourceTypeApplicationInput`,
  `SourceTypeExpressionInput`, `SourceTypeArgumentInput`,
  `SourceTypeApplicationForm`, `SourceTypeHead`, `SourceTypeArgument`,
  `SourceTypeApplicationHandoff`, `SourceTypeApplicationTable`,
  `SourceTypeApplication`, `SourceTypeExpressionTable`,
  `SourceTypeExpression`, `SourceTypeArgumentTable`,
  `SourceTypeArgumentRow`, `SourceTypeProducer`, `SourceTypeError`

対応:

| 仕様上の約束 | source根拠 | test根拠 | 状態 |
|---|---|---|---|
| syntax-free flat tableがouter binding link、recursive written type expression/head、ordered term/type/`qua` argumentを保持する。 | `SourceTypeHandoffInput`、dense id、`src/source_type.rs`のimmutable table 3件。 | exact broad 10/13/6 real runner oracle、Task-248 2/2/0 co-consumer。 | Task 249について実装済み。 |
| bindingとreal `DeclarationShell` ownership、symbol/contribution import closure/visibility、arena site/range/recoveryをpublish前にauthenticateする。 | `SourceTypeProducer::build`とinstallation validation。 | producer corruption matrix、real local/imported head、import-target mismatch。 | transactionalに実装済み。 |
| graph order/ownership/containment/non-overlapとdeterministic provenanceをfail closedにする。 | `SourceTypeError`、iterative graph/range/provenance validation、deterministic debug。 | dangling/cycle/multiple-parent/forward/duplicate/wrong-form/range/provenance/deep-chain test。 | sort/recursion/repairなしで実装済み。 |
| `TypedAst`がresultを所有し、`ResolvedTypedAst`はcloneだけする。 | optional `SourceTypeApplicationHandoff` fieldとborrowed getter。 | immutable final preservation/repeated-run assertion。 | 実装済み。legacy empty debug byteはconditionalに維持。 |
| public enumはforward-compatible。 | public enumの`#[non_exhaustive]`。 | `checker_public_enums_are_forward_compatible_and_documented`。 | exhaustive exceptionなしでguard。 |

bounded gap: Task 249がpublishするのはsource-type inputだけである。expansion、
normalization、evidence、term/`qua` selection、accepted fact/declaration/proof、
downstream IRはexplicit later ownerに残る。

### `source_evidence`

生成 public newtype:

- `SourceEvidenceRequestId`, `SourceEvidenceResponseId`

literal top-level public item:

- `SourceEvidenceResponseKey`, `SourceEvidenceHandoffInput`,
  `SourceEvidenceRequestInput`, `SourceEvidenceResponseInput`,
  `SourceEvidenceRequestKind`, `SourceEvidenceInputState`,
  `SourceEvidenceRequestOrigin`, `SourceEvidenceResponseDisposition`,
  `SourceEvidenceResponsePayload`, `SourceEvidenceResponseProvenance`,
  `SourceEvidenceRecovery`, `SourceEvidenceDependencyRecord`,
  `SourceEvidenceDependencyCatalog`, `SourceEvidenceHandoff`,
  `SourceEvidenceRequestTable`, `SourceEvidenceRequest`,
  `SourceEvidenceResponseTable`, `SourceEvidenceResponse`,
  `SourceEvidenceError`, `SourceEvidenceProducer`

対応:

| 仕様上の約束 | source根拠 | test根拠 | 状態 |
|---|---|---|---|
| syntax-free dense transactionがsemantic acceptanceをclaimせずsource-derived requestとauthenticated dependency referenceを保持する。 | `src/source_evidence.rs`の`SourceEvidenceHandoffInput`、request/response table、transport-state/dependency DTO。 | exact 3-route 10-request runner oracleとchecker four-state/table test。 | Task 251について実装済み。 |
| Task-249 application/expression、optional Task-250 chain、symbol kind、source/module、owner/site/range/application ordinal/recoveryをpublish前にauthenticateする。 | `SourceEvidenceProducer::build`がupstream handoff 2件、`SymbolEnv`、`TypeFactTable`、dependency catalogをconsumeする。 | association、distinct application/chain ordinal、symbol/source/module、missing/duplicate、field corruption test。 | transactionalに実装済み。 |
| response cardinality、catalog association、disposition/payload compatibility、fact existence、existential-gate owner/range/recovery/guard factをfail closedにする。 | `SourceEvidenceError`とrequest/response/catalog/payload validator。 | state/cardinality、key reuse/cross/stale、fact、gate、atomic-failure test。 | fallback/repairなしで実装済み。 |
| `TypedAst`がimmutable handoffをownし、`ResolvedTypedAst`はclone-preserveだけを行う。 | optional `SourceEvidenceHandoff` field、validated installer、borrowed getter。 | production-runner ownership、replacement rejection、clone equality、deterministic debug assertion。 | 実装済み。legacy empty outputは不変。 |
| public enumはforward-compatible。 | public Task-251 enumの`#[non_exhaustive]`。 | `checker_public_enums_are_forward_compatible_and_documented`。 | exhaustive exceptionなしでguard。 |

bounded gap: Task 251がtransportするのはexact mode/structure/attributed
evidence requestとauthenticated dependency inputだけである。evidence
interpretation、fact creation/acceptance、gate evaluation、inheritance/coercion
selection、accepted registration/artifact publication、downstream IRは実装しない。
それらはTasks 252+とexplicit ownerに残る。

### `source_term`

生成 public newtype:

- `SourcePrimaryTermId`, `SourcePrimaryTermReferenceId`,
  `SourceNumericTypeRequestId`

literal top-level public item:

- `SourcePrimaryTermHandoffInput`, `SourcePrimaryTermInput`,
  `SourcePrimaryTermReferenceInput`, `SourceNumericTypeRequestInput`,
  `SourcePrimaryTermKind`, `SourcePrimaryTermRole`,
  `SourcePrimaryTermReferenceRole`, `SourcePrimaryTermRecovery`,
  `SourcePrimaryTermHandoff`, `SourcePrimaryTermTable`, `SourcePrimaryTerm`,
  `SourcePrimaryTermReferenceTable`, `SourcePrimaryTermReference`,
  `SourceNumericTypeRequestTable`, `SourceNumericTypeRequest`,
  `SourcePrimaryTermError`, `SourcePrimaryTermProducer`

対応:

| 仕様上の約束 | source根拠 | test根拠 | 状態 |
|---|---|---|---|
| syntax-free three-table transactionがsemantic resultを作らずprimary-term occurrence、authenticated binding reference、unresolved numeric-type requestを保持する。 | `src/source_term.rs`の`SourcePrimaryTermHandoffInput`とimmutable term/reference/request table。 | exact 3-route 7/4/2 runner oracleとevery-kind checker test。 | Task 252について実装済み。 |
| typed site/range/kind/recovery、canonical lexer-identifier vocabulary/spelling、context、dense pre-order、parent closure、reference/request cardinality、numeric associationをfail closedにする。 | `SourcePrimaryTermProducer::build`がraw syntaxをimportせず`mizar_lexer::is_identifier`をreuseし、complete transactionを`TypedArena`と`BindingEnv`に対してvalidateする。 | site/range/kind/recovery/context、identifier shape/reserved-word rejection、graph、cardinality、request、corruption test。 | sort/repairなしでtransactionalに実装済み。 |
| scopeとbinding-event ordinalはproducer-derivedであり、exact `BindingEnv::lookup` local winnerを要求する。 | reference constructionがcontext scopeをcloneし、preceding completed binding rowをcountし、exact duplicate-priority groupを保持し、すべてのnon-local resultをrejectする。 | shadow-winner、forward、ambiguous、missing-scope、unresolved、wrong-winner、ordinal test。 | `Resolver`をstructurally unreachableとして実装済み。 |
| `TypedAst`がimmutable handoffをownし、`ResolvedTypedAst`はclone-preserveだけを行う。 | optional `SourcePrimaryTermHandoff` field、validated installer、borrowed getter。 | production-runner ownership、replacement rejection、clone equality、deterministic replay assertion。 | 実装済み。semantic typed/fact/downstream tableは不変。 |
| public enumはforward-compatible。 | public Task-252 enumすべての`#[non_exhaustive]`。 | `checker_public_enums_are_forward_compatible_and_documented`。 | exhaustive exceptionなしでguard。 |

bounded gap: Task 252がtransportするのはfrozen five-kind primary-term sourceと
numeric requestだけである。application/other term family、cross-family parent
edge、numeric result、real current-definition-result ownership、real
local-constant binding production、formula graph、accepted fact/declaration/
proof、downstream IRはTasks 253+、260、264、269とexplicit ownerに残る。

### `type_checker`

literal top-level public item:

- `TypeNormalizationOutput`, `TypeNormalizer`, `DeclarationCheckingOutput`,
  `DeclarationChecker`, `TermFormulaInferenceOutput`, `TermFormulaChecker`,
  `TermFormulaInferenceError`, `CheckedStatementOwner`, `StatementOwnerError`,
  `CoercionCheckingOutput`, `CoercionObligationChecker`, `CoercionInput`,
  `CoercionRequestKind`, `CoercionJustification`, `CoercionEvidence`,
  `CoercionDeferredReason`, `InitialObligationInput`, `InitialRequirementKind`,
  `TypeFactQueryEngine`, `TypeFactQuery`, `TypeFactQueryOutput`,
  `TypeFactQueryStatus`, `TermInput`, `TermKind`, `TermReference`,
  `TermDeferredReason`, `FormulaInput`, `FormulaKind`,
  `FormulaDeferredReason`, `ExpectedTypeInput`, `FormulaFactInput`,
  `OpenCandidateInput`, `CandidateIdentity`, `OpenCandidateSetTable`,
  `OpenCandidateSet`, `OpenCandidate`, `CandidateSetKind`,
  `CandidateSetStatus`, `CandidateStatus`, `CheckedTermTable`,
  `CheckedTermId`, `CheckedTerm`, `TermStatus`, `CheckedFormulaTable`,
  `CheckedFormulaId`, `CheckedFormula`, `FormulaStatus`,
  `ExpectedTypeConstraint`, `DeclarationContextInput`, `DeclarationInput`,
  `DeclarationKind`, `ReservedDefaultPayload`,
  `DeclarationDeferredReason`, `DeclarationAssumptionInput`,
  `CheckedDeclarationTable`, `CheckedDeclarationId`,
  `CheckedDeclaration`, `DeclarationStatus`,
  `SourceReserveDeclarationBridge`, `SourceReserveBindingInput`,
  `SourceReserveDeclarationHandoff`, `TypeExpressionInput`,
  `TypeHeadInput`, `AttributeInput`, `AttributePolarity`, `ModeExpansion`,
  `NormalizedTypeTable`, `NormalizedType`, `TypeHeadRef`,
  `TypeHeadErrorKind`, `AttributeSet`, `AttributeInstance`, `TypeSource`,
  `NormalizedTypeStatus`, `SourceRangeKey`

対応:

| 仕様上の約束 | source 根拠 | test 根拠 | 状態 |
|---|---|---|---|
| Type-expression normalization は mode、attribute、arity、unsupported input の degraded state を canonicalize し、cluster repair を行わない。 | `TypeNormalizer`, `TypeNormalizationOutput`, `TypeExpressionInput`, `ModeExpansion`, normalized type tables. | attribute/order/builtin/mode-expansion/degraded-head tests. | explicit payload について実装済み。MC-G014 は残る。 |
| Declaration checking は explicit declaration と binding context を消費し、partial output を保持する。 | `DeclarationChecker`, `DeclarationCheckingOutput`, declaration input/status tables. | declaration deterministic/invalid/constrained/set/attributed/reconsider tests. | explicit payload について実装済み。MC-G016 は残る。 |
| reserve-only source-derived producer seam は syntax-free な reserve payload を消費し、raw syntax を import せず checker-owned binding/declaration handoff data を構築する。 | `SourceReserveDeclarationBridge`, `SourceReserveBindingInput`, `SourceReserveDeclarationHandoff`. | `source_reserve_declaration_bridge_builds_checker_owned_handoff`, `source_reserve_declaration_bridge_validates_local_symbol_heads_and_mismatched_inputs`, active `mizar-test` type-elaboration runner regressions. | successful bare builtin `set` / `object` reserve declaration、builtin `set` / `object` に終端する accepted same-module bare local-mode chain、および MC-G020 に記録された active diagnostic reserve/type-head boundary について実装済み。task 82 は imported-mode slice を拡張し、documented `parser.type_fixtures` の imported `TypeCaseMode` head が `ImportedSource` provenance 付き checker type-head payload になり、その後 missing imported `ModeExpansion` payload だけで fail closed する。task 83 は imported-structure slice を documented structure `R` に限って拡張し、imported `SymbolKind::Structure` head が checker type-head payload になり、その後 missing structure-evidence query だけで fail closed する。task 97 は同じ slice を documented structure `TypeCaseStruct` に適用する。task 84 は imported-attribute slice を documented `TypeCaseAttr` に限って拡張し、imported `SymbolKind::Attribute` が builtin `set` 上の checker `AttributeInput` payload になり、その後 missing attributed-type evidence query だけで fail closed する。task 85 は imported-attribute slice を documented negative `empty` over builtin `set` に限って拡張し、imported `SymbolKind::Attribute` が negative checker `AttributeInput` payload になり、その後 missing attributed-type evidence query だけで fail closed する。task 116 は documented positive `empty` over builtin `set` に同じ slice を適用し、positive checker `AttributeInput` payload として同じ missing attributed-type evidence query で fail closed する。`R` / `TypeCaseStruct` を超える imported structure、`TypeCaseAttr` と task-85/task-116 `empty`/builtin-`set` を超える imported attribute、imported mode expansion、imported module AST extraction、より広い AST-wide extraction は MC-G020 のまま。 |
| Term/formula inference は checked table、expected constraint、open candidate、fact、recovery を記録し、type assertion は normalized identity を受理して non-reflexive reachability payload 欠落で fail closed する。 | `TermFormulaChecker`, term/formula input and checked tables, `FormulaDeferredReason::MissingTypeAssertionReachabilityPayload`. | term/formula/recovery tests、`type_assertion_requires_reflexive_or_external_reachability_payload`。 | explicit payload は reflexive type-assertion admissibility まで実装済み。general widening/`qua` reachability は MC-G017/MC-G019/external のまま。 |
| Coercion と initial obligation は `VcId` や fabricated evidence なしで記録される。 | `CoercionObligationChecker`, `CoercionInput`, `InitialObligationInput`, justification/evidence/deferred enum. | coercion deterministic/missing evidence/alternate candidate tests; task 47 omitted-`reconsider` proof-free/requires-proof tests。 | explicit payload について実装済み。source-derived coercion/reconsider extraction について MC-G018/MC-G020 は残る。 |
| Fact query は deterministic、visibility-scoped、non-mutating。 | `TypeFactQueryEngine`, `TypeFactQueryOutput`, `TypeFactQueryStatus`. | deterministic/provenance/visibility/contradiction tests. | 実装済み。 |
| public enum は forward-compatible。 | public enum の `#[non_exhaustive]`。 | `checker_public_enums_are_forward_compatible_and_documented`. | task 31 で guard 済み。 |

Task 75 addendum: active `mizar-test` type-elaboration runner は、後続
same-module local mode declaration を earlier reserve head から参照する
source-derived case について、
`type_elaboration.lower_stage.frontend:malformed_type_expression` を
観測する。これは Chapter 2/11 の active-range/no-forward-reference 境界だけを
credit し、future declaration から checker `ModeExpansion` payload を作ること、
CoreIr/ControlFlowIr/VC/proof payload coverage を主張することはしない。
forward-reference acceptance は active-range rule で forbidden として
lower-stage rejection だけで cover する。

Task 76 addendum: active `mizar-test` type-elaboration runner は、後続
same-module local structure declaration を earlier reserve head から参照する
source-derived case について、同じ
`type_elaboration.lower_stage.frontend:malformed_type_expression` を観測する。
これは Chapter 2/5/11 の active-range/no-forward-reference 境界と structure
syntax/type-head surface だけを credit し、future declaration から checker
structure type-head payload、base-shape evidence query、constructor-witness
evidence query、CoreIr/ControlFlowIr/VC/proof payload coverage を主張しない。

Task 77 addendum: active `mizar-test` type-elaboration runner は、後続
same-module local attribute declaration を earlier reserve type expression から
使う source-derived case について、同じ
`type_elaboration.lower_stage.frontend:malformed_type_expression` を観測する。
これは Chapter 2/6/11 の active-range/no-forward-reference 境界と attribute
syntax/use surface だけを credit し、future declaration から checker
`AttributeInput` extraction、attributed-type evidence query、CoreIr、
ControlFlowIr、VC、proof payload coverage を主張しない。

Task 78 addendum: task 83 より前の active `mizar-test` type-elaboration runner
は、documented `parser.type_fixtures` import summary 由来の structure symbol を
持つ source-derived reserve head について
`type_elaboration.external_dependency.ast_payload_extraction` を観測した。task 83
は `R` imported provenance/type-head coverage についてこの boundary を上書きし、
task 97 は `TypeCaseStruct` imported provenance/type-head coverage について同じ boundary
を上書きする。task-83 / task-97 bridge 外の imported structure は deferred とする。これは
import summary を real imported module AST extraction と扱わず、base-shape /
constructor-witness evidence、positive structure type elaboration、CoreIr、
ControlFlowIr、VC、proof payload coverage を主張しない。

Task 83/task 97 addendum: active `mizar-test` type-elaboration runner は
documented `parser.type_fixtures` imported structure symbol `R` と
`TypeCaseStruct` を real `ImportedSource` checker type head として観測し、
`type_elaboration.checker.checker.declaration.deferred.evidence_query` を報告する。
これは imported structure provenance と type-head payload extraction だけを
credit し、imported module AST extraction、base-shape / constructor-witness
evidence、positive imported structure elaboration、CoreIr、ControlFlowIr、VC、
proof payload coverage は credit しない。

Task 79 addendum: task 82 より前の active `mizar-test` type-elaboration runner
は、documented `parser.type_fixtures` import summary 由来の mode symbol を持つ
source-derived reserve head について
`type_elaboration.external_dependency.ast_payload_extraction` を観測した。task 82
は `TypeCaseMode` imported provenance/type-head coverage だけについてこの
boundary を上書きする。task-82 bridge 外の imported mode は引き続き Chapter
3/7/11/12 の imported-mode reserve-head diagnostic boundary だけを credit する。
これは import summary を real imported module AST extraction と扱わず、
`ModeExpansion` payload、positive mode elaboration、CoreIr、ControlFlowIr、VC、
proof payload coverage を主張しない。

Task 80 addendum: task 84 / task 85 / task 116 / task 171 より前の active `mizar-test`
type-elaboration runner は、documented `parser.type_fixtures` import summary
由来の attribute symbol を持つ source-derived reserve type について
`type_elaboration.external_dependency.ast_payload_extraction` を観測した。task 84
は `TypeCaseAttr` imported provenance と `AttributeInput` payload coverage だけ、
task 85 は negative `empty`/builtin-`set` fixture だけについてこの boundary を
上書きし、task 116 は positive `empty`/builtin-`set` fixture だけについてこの
boundary を上書きし、task 171 は negative `empty`/builtin-`object` fixture だけに
ついてこの boundary を上書きする。task-84 / task-85 / task-116 / task-171 bridge 外の imported attribute は、source-derived
fixture と payload producer が存在するまで deferred とする。これは
import summary を real imported module AST extraction と扱わず、attributed-type
evidence、positive attributed type elaboration、CoreIr、ControlFlowIr、VC、
proof payload coverage を主張しない。positive `empty object` と symbol head 上の
imported attribute は active fixture credit のない deferred extraction gap に残る。

Task 84 addendum: active `mizar-test` type-elaboration runner は documented
`parser.type_fixtures` imported attribute symbol `TypeCaseAttr` を builtin `set`
上の real `ImportedSource` checker `AttributeInput` として観測し、
`type_elaboration.checker.checker.declaration.deferred.evidence_query` を報告する。
これは imported attribute provenance と argument-free `AttributeInput` payload
extraction だけを credit し、imported module AST extraction、attributed-type
existential/evidence payload、positive imported attributed type elaboration、
`empty` のような generic imported attribute、structure-qualified attribute
owner provenance、attribute argument、CoreIr、ControlFlowIr、VC、proof payload
coverage は credit しない。

Task 85 addendum: active `mizar-test` type-elaboration runner は documented
`parser.type_fixtures` imported attribute symbol `empty` を、既存 `non empty set`
fixture について builtin `set` 上の real `ImportedSource` negative checker
`AttributeInput` として観測し、
`type_elaboration.checker.checker.declaration.deferred.evidence_query` を報告する。
これはその fixture の imported attribute provenance と argument-free negative
`AttributeInput` payload extraction だけを credit し、imported module AST extraction、
attributed-type existential/evidence payload、positive `empty object`、symbol head
上の imported attribute、broader imported attribute、structure-qualified attribute
owner provenance、attribute argument、CoreIr、ControlFlowIr、VC、proof payload
coverage は credit しない。
Task 116 addendum: active `mizar-test` type-elaboration runner は documented
`parser.type_fixtures` imported attribute symbol `empty` を、既存 `empty set`
fixture について builtin `set` 上の real `ImportedSource` positive checker
`AttributeInput` として観測し、
`type_elaboration.checker.checker.declaration.deferred.evidence_query` を報告する。
これはその fixture の imported attribute provenance と argument-free positive
`AttributeInput` payload extraction だけを credit し、imported module AST extraction、
attributed-type existential/evidence payload、positive attributed-type acceptance、
positive `empty object`、symbol head 上の imported attribute、broader imported attribute、
structure-qualified attribute owner provenance、attribute argument、CoreIr、
ControlFlowIr、VC、proof payload coverage は credit しない。
Task 171 addendum: active runner は同じ imported `empty` symbol を、既存
`non empty object` fixture について builtin `object` 上の real
`ImportedSource` negative checker `AttributeInput` として観測し、同じ evidence-
query diagnostic を報告する。credit するのは exact imported provenance、negative
polarity、argument-free checker handoff だけである。positive `empty object`、
symbol head 上の imported attribute、imported module AST extraction、attributed-
type admissibility/evidence または acceptance、downstream payload は credit しない。

Task 181 current-state repair: tasks 84、85、116、171 が共有する source route は、
credit 済み source shape 5 件、すなわち exact single-binding fixture 4 件と exact
ordered mixed reserve fixture だけを受理する。real `parser.type_fixtures`
imported attribute は exact import/item layout、spelling、head、polarity、argument
なしを要求し、missing/wrong/duplicate import、duplicate/mixed attribute、broader
binding/item/order shape、definition、recovery node は extraction gap に残す。
既存 expectation と runner count 129 は変えない。この repair は
`source_undocumented_behavior` を閉じるが、positive `empty object`、attribute
evidence/acceptance、imported module AST extraction、downstream payload は昇格しない。

Task 86 / task 115 / task 117 addendum: task 86 は
`theorem FormulaPayloadBoundary: thesis;` という formula-only theorem source を
parser / resolver 実行後の source-derived theorem/formula boundary として記録する。
task 115 はこの exact source だけを supersede し、source-derived `thesis` formula
constant site/range を checker recovery `FormulaInput` として渡す。task 117 は
同じ exact source を real `FormulaKind::Thesis` payload に進め、missing formula
payload で fail closed する。これは exact formula-constant kind handoff だけを
credit し、formula constant semantic checking、child-formula graph payload、
local proof context、recorded fact、theorem acceptance、dedicated
`formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload coverage は
主張しない。

Task 119 MC-G020 current-state override: exact active source
`reserve x for set; theorem ReservedVariableEqualityPayloadBoundary: x = x;`
は real reserve declaration handoff と source-derived identifier-term / equality
payload を組み合わせる。2 つの use ordinal は reserve binding range と両
identifier range の順序から独立に導出し、checker-owned `BindingEnv` は各 use を
別々に解決する。記述された builtin `set` type は distinct source-anchored role
owner を通じて 2 つの term result type と 2 つの equality expected-type
constraint を供給する。production runner は 2 つの `Inferred` variable term、
type/well-formedness だけを表す 1 つの `Checked` equality、exact normalized type
source range/spelling/head、empty candidate/fact/deferred/diagnostic output を要求し、
payload drift は stable
`type_elaboration.checker.reserved_variable_equality.invalid_payload` key を報告する。
unit test は active sidecar の real frontend/resolver AST に対して同じ assertion を
反復する。これは MC-G020 の generic term/formula gap wording をその exact slice
だけについて override する。implicit universal-closure node、equality truth/fact、
theorem acceptance、`formula_statement` runner、proof skeleton、CoreIr、
ControlFlowIr、VC、broader source extraction は credit しない。

Task 120 MC-G020 current-state override: exact active membership source
`reserve x for set; theorem ReservedVariableMembershipPayloadBoundary: x in x;`
は task 119 の real binding/use event ordinal と independent lookup を再利用する。
既存 API が要求する checker-owned membership payload、すなわち 2 つの known
source-derived `set` variable result、右 operand の 1 つの expected-`set` role、
1 つの no-fact `Checked` membership formula だけを追加する。production
invariant、task-specific invalid-payload key、real frontend/resolver sidecar test が
slice を guard する。membership truth/fact、implicit closure、theorem acceptance、
proof、CoreIr、ControlFlowIr、VC、broader source extraction は credit しない。

Task 121 MC-G020 current-state override: exact reserved-variable inequality
source は real binding/use producer を共有し、2 linked result role と 2 linked
expected role を fact-free pre-desugaring `Checked` inequality に渡す。production
invariant と real sidecar payload test が guard する。expected type shape の根拠は
checker-owned API coverage と task 119 であり、task 107 の partial numeral bridge
ではない。desugaring/truth/fact、implicit closure、theorem acceptance、proof、
CoreIr、ControlFlowIr、VC、broader extraction は open のままである。

Task 122 MC-G020 current-state override: exact reserved-variable type assertion
source は task 119 の real reserve lookup/result producer と task 109 の
formula-side asserted builtin-`set` AST producer を結合する。checker は exactly
one ready subject と one asserted type を要求し、normalized reflexive identity
だけで `Checked` を維持し、known non-identical type は widening を捏造せず
`checker.formula.external.type_assertion_reachability_payload` で fail closed する。
source runner は normalization 前の 2 input と distinct source anchor を独立検証し、
1 `Inferred` variable と 1 fact-free `Checked` type assertion を要求する。general
reachability/widening/`qua`、attribute、truth/fact、implicit closure、theorem
acceptance、proof、CoreIr、ControlFlowIr、VC、broader extraction は open のままである。

Task 123 MC-G020 current-state override: exact distinct reserved-variable
equality source は、既存の real multi-reserve declaration producer と task 119 の
equality consumer を結合する。記述された builtin `set` range 1 個が 2 個の
source binding を裏付けるが、source-order lookup は operand ごとの
result/expected role が fact-free `Checked` equality に到達する前に `x` と `y`
を distinct checker binding id に解決する。production invariant、near-miss
matrix、real frontend/resolver sidecar が exact bridge を guard する。これは
classified `test_gap` / `source_drift` / `design_drift` を exact distinct-binding
type/well-formedness についてだけ閉じる。implicit closure/order node、equality
truth/fact、theorem acceptance、broader source shape、proof、CoreIr、
ControlFlowIr、VC は open のままである。

Task 124 MC-G020 current-state override: exact multiple-reserve-declaration
equality source は同じ real declaration、binding、lookup、term、formula consumer
を再利用しつつ、2 個の記述上の builtin `set` range を operand ごとの 4 個すべての
pre-normalization result/expected input に distinct に保持する。checker はこれらの
semantically equal input を、deterministic な最初の source representative を持つ
1 `NormalizedTypeId` に intern するが、その representative は original input の
provenance を置き換えない。production invariant、exact near-miss matrix、real
frontend/resolver sidecar がこの `test_gap` / `source_drift` / `design_drift` repair
を guard する。implicit closure/order、truth/fact、theorem acceptance、broader
source、proof、CoreIr、ControlFlowIr、VC は open のままである。

Task 125 MC-G020 current-state override: exact heterogeneous-reserve membership
source は既存の real mixed-builtin reserve producer と task 120 の right-expected-type
membership consumer を結合する。runner は 2 distinct declaration range と binding
id にわたり、左 `object` result input と右 `set` result/expected input を保持する。
checker は 2 semantic identity を生成し、右の両 role は `set` を共有する。
production invariant は identity partition と deterministic な type ごとの source
representative を検証する。これは classified `test_gap` / `source_drift` /
`design_drift` を exact type/well-formedness slice についてだけ閉じる。membership
truth/fact、object/set coercion evidence、closure/order、theorem acceptance、broader
source、proof、CoreIr、ControlFlowIr、VC は open のままである。

Task 126 MC-G020 current-state override: exact direct-local-mode equality source
は task 55 の real AST-derived bare-set expansion と task 119 の equality consumer
を結合する。4 raw input は local-mode provenance を保持し、checker は expansion を
消費して 1 builtin-set identity、2 `Inferred` term、1 fact-free `Checked` equality
を記録する。mode declaration checking/acceptance、inhabitation evidence、broader
mode、closure/order、truth/fact、theorem acceptance、proof、CoreIr、ControlFlowIr、
VC は open のままである。

Task 127 MC-G020 current-state override: exact one-edge local-mode-chain equality
source は task 56 の real AST-derived expansion 2 個と task 126 の equality consumer
を結合する。4 raw input は outer-mode provenance を保持し、checker は両 link を
recursively 消費して real terminal `set` RHS に anchor された 1 builtin-set
identity、2 `Inferred` term、1 fact-free `Checked` equality を記録する。mode
declaration checking/acceptance、inhabitation evidence、object terminal、
longer-chain formula、closure/order、truth/fact、theorem acceptance、proof、CoreIr、
ControlFlowIr、VC は open のままである。

Task 128 MC-G020 current-state override: exact direct local-object-mode equality
source は task 55 の real AST-derived `LocalObjectMode -> object` expansion と task
126 の equality consumer を結合する。4 raw input は object-mode provenance を保持し、
checker は real expansion を消費して real `object` RHS に anchor された 1
builtin-object identity、2 `Inferred` term、1 fact-free `Checked` equality を記録する。
exact production guard、invalid-expansion corruption、withheld-family near miss、
real sidecar が route を guard する。mode declaration checking/acceptance、
inhabitation evidence、broader object-mode formula、closure/order、truth/fact、
theorem acceptance、proof、CoreIr、ControlFlowIr、VC は open のままである。

Task 130 MC-G020 current-state override: exact direct local-mode inequality は
task 55 の real `LocalModeInequality -> set` expansion と task 121 の
pre-desugaring consumer を結合する。4 raw input は mode provenance を保持し、
real RHS 起点の builtin-set identity 1 個、2 `Inferred` term、1 fact-free
`Checked` inequality を得る。declaration acceptance/inhabitation、desugaring、
closure/order、truth/fact、theorem acceptance、proof/Core/VC は open のままである。

Task 131 MC-G020 current-state override: exact direct local-object-mode
inequality は task 55 の real `LocalObjectModeInequality -> object` expansion と
task 121/130 の pre-desugaring consumer を結合する。4 raw input は object-mode
provenance を保持し、real RHS 起点の builtin-object identity 1 個、2
`Inferred` term、1 fact-free `Checked` inequality を得る。mode declaration
acceptance/inhabitation、desugaring、closure/order、truth/fact、theorem
acceptance、proof/Core/VC は open のままである。

Task 132 MC-G020 current-state override: exact one-edge set-terminal mode-chain
inequality は task 56/127 の real AST-derived expansion 2 個と task 121/130 の
pre-desugaring inequality consumer を結合する。4 raw input は outer-mode
provenance を保持し、recursive normalization は terminal RHS に 1 builtin-set
identity を anchor して 2 `Inferred` term と 1 fact-free `Checked` inequality を
生成する。declaration acceptance/inhabitation、object terminal、direct/longer
chain、desugaring、closure/order、truth/fact、theorem acceptance、proof、CoreIr、
ControlFlowIr、VC は open のままである。

Task 133 MC-G020 current-state override: exact one-edge object-terminal
mode-chain inequality は task 129 の real AST-derived expansion 2 個と task 131
の pre-desugaring builtin-object inequality consumer を結合する。4 raw input は
outer-mode provenance を保持し、recursive normalization は terminal RHS に 1
builtin-object identity を anchor して 2 `Inferred` term と 1 fact-free
`Checked` inequality を生成する。declaration acceptance/inhabitation、
set-terminal、direct/longer chain、desugaring、closure/order、truth/fact、theorem
acceptance、proof、CoreIr、ControlFlowIr、VC は open のままである。

Task 134 MC-G020 current-state override: exact two-edge set-terminal mode-chain
equality は task 72 の real AST-derived expansion 3 個と task 127 の equality
consumer を結合する。4 raw input は outer-mode provenance を保持し、recursive
normalization は terminal RHS に 1 builtin-set identity を anchor して 2
`Inferred` term と 1 fact-free `Checked` equality を生成する。declaration
acceptance/inhabitation、object terminal、direct/one-edge/longer chain、implicit
closure/order、truth/fact、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は
open のままである。

Task 135 MC-G020 current-state override: exact two-edge object-terminal
mode-chain equality は task 72 の real AST-derived expansion 3 個と task 134 の
equality consumer および builtin-object terminal support を結合する。4 raw input
は outer-mode provenance を保持し、recursive normalization は terminal RHS に 1
builtin-object identity を anchor して 2 `Inferred` term と 1 fact-free `Checked`
equality を生成する。declaration acceptance/inhabitation、task 134 を超える
set-terminal sibling semantics、direct/one-edge/longer chain、implicit
closure/order、truth/fact、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は
open のままである。

Task 136 MC-G020 current-state override: exact two-edge set-terminal
mode-chain inequality は task 72 の real AST-derived expansion 3 個と task 132 の
pre-desugaring inequality consumer を結合する。4 raw input は outer-mode
provenance を保持し、recursive normalization は terminal RHS に 1 builtin-set
identity を anchor して 2 `Inferred` term と 1 fact-free pre-desugaring `Checked`
inequality を生成する。mode declaration acceptance/inhabitation、object
terminal、direct/one-edge/longer chain、inequality desugaring、implicit
closure/order、truth/fact、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は
open のままである。

Task 137 MC-G020 current-state override: exact two-edge object-terminal
mode-chain inequality は task 72 の real AST-derived expansion 3 個と task 133 の
builtin-object pre-desugaring inequality consumer を結合する。4 raw input は
outer-mode provenance を保持し、recursive normalization は terminal RHS に 1
builtin-object identity を anchor して 2 `Inferred` term と 1 fact-free
pre-desugaring `Checked` inequality を生成する。declaration
acceptance/inhabitation、set terminal、direct/one-edge/longer chain、inequality
desugaring、implicit closure/order、truth/fact、theorem acceptance、proof、CoreIr、
ControlFlowIr、VC は open のままである。

Task 138 MC-G020 current-state override: exact direct set-terminal local-mode
reserved-variable type assertion は task 55 の real AST-derived
`LocalModeTypeAssertion -> set` expansion と task 122 の normalized-reflexive
type-assertion consumer を結合する。raw subject は local-mode provenance、asserted
builtin `set` は独立した formula source を保持し、real expansion 1 本が両 type を
terminal-RHS builtin-set identity 1 個へ normalize してから、1 `Inferred` term と
1 fact-free `Checked` type assertion を記録する。mode declaration
acceptance/inhabitation、formula-side local-mode asserted head、general
reachability/widening/`qua`、truth/fact、theorem acceptance、proof、CoreIr、
ControlFlowIr、VC は open のままである。

Task 139 MC-G020 current-state override: exact direct set-terminal local-mode
left membership は task 55 の real AST-derived
`LocalModeMembership -> set` expansion、task 120 の right-only expected-set
membership consumer、task 125 の two-binding distinct-source form を結合する。
raw left result は local-mode provenance、独立した right result と sole
expected-set input は explicit reserve provenance を保持する。real expansion 1 本が
left role を normalize し、right builtin-set role は直接 normalize され、3 role
すべてが terminal-RHS builtin-set identity 1 個へ intern してから、2 `Inferred`
term と 1 fact-free `Checked` membership を記録する。mode declaration
acceptance/inhabitation、membership truth/fact、implicit closure/order、theorem
acceptance、proof、CoreIr、ControlFlowIr、VC は open のままである。

Task 140 MC-G020 current-state override: exact direct object-terminal
local-mode left membership は task 55 の real AST-derived
`LocalObjectModeMembership -> object` expansion と task 125 の right-only
expected-set / two-binding distinct-source membership consumer を結合する。raw
left result は local object-mode provenance、独立した right result と sole
expected-set input は explicit reserve provenance を保持する。real expansion 1
本が left role を terminal-RHS builtin-object identity へ normalize し、right
builtin-set role は distinct explicit-reserve-anchored identity へ直接 normalize
する。2 `Inferred` term と 1 fact-free `Checked` membership を、right-owned
constraint 1 個だけ、left expected type なしで記録する。mode declaration
acceptance/inhabitation、membership truth/fact、object/set coercion、implicit
closure/order、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は open の
ままである。

Task 141 MC-G020 current-state override: exact one-edge set-terminal
local-mode-chain left membership は task 56 の real AST-derived
`ChainModeMembership -> BaseModeMembership -> set` expansion 2 本と task 139
の right-only expected-set / two-binding membership consumer を結合する。raw
left result は outer-mode provenance、独立した right result と sole expected-set
input は explicit reserve provenance を保持する。real expansion 2 本が left role
を recursive に normalize し、right builtin-set role は直接 normalize され、3
role すべてが terminal-RHS builtin-set identity 1 個へ intern する。right-owned
constraint 1 個だけ、left expected type なしで 2 `Inferred` term と 1 fact-free
`Checked` membership を記録する。mode declaration acceptance/inhabitation、
membership truth/fact、implicit closure/order、theorem acceptance、proof、
CoreIr、ControlFlowIr、VC は open のままである。

Task 142 MC-G020 current-state override: exact one-edge object-terminal
local-mode-chain left membership は task 56 の real AST-derived
`ChainObjectModeMembership -> BaseObjectModeMembership -> object` expansion 2
本と task 125 / 140 / 141 の right-only expected-set / two-binding membership
consumer を結合する。raw left result は outer-mode provenance、独立した right
result と sole expected-set input は explicit reserve provenance を保持する。real
expansion 2 本が left role を terminal-RHS builtin-object identity 1 個へ recursive
に normalize し、right builtin-set role は distinct explicit-reserve-anchored
identity 1 個へ直接 normalize される。right-owned constraint 1 個だけ、left
expected type なしで 2 `Inferred` term と 1 fact-free `Checked` membership を
記録する。mode declaration acceptance/inhabitation、membership truth/fact、
object/set coercion、implicit closure/order、theorem acceptance、proof、CoreIr、
ControlFlowIr、VC は open のままである。

Task 129 MC-G020 current-state override: exact one-edge object-terminal
mode-chain equality は task 56 の real AST-derived expansion 2 個と task 127/128 の
equality / builtin-object consumer を結合する。4 raw input は outer-mode provenance
を保持し、recursive normalization は terminal RHS に 1 builtin-object identity を
anchor して 2 `Inferred` term と 1 fact-free `Checked` equality を生成する。
declaration acceptance/inhabitation、longer chain、closure/order、truth/fact、
theorem acceptance、proof、CoreIr、ControlFlowIr、VC は open のままである。

Task 143 MC-G020 current-state override: exact two-edge set-terminal
local-mode-chain left membership は task 72 の real AST-derived expansion 3 本
`OuterTwoEdgeModeMembership -> MiddleTwoEdgeModeMembership -> BaseTwoEdgeModeMembership -> set`
と、task 125 / 139 / 141 の right-only expected-set two-binding membership
consumer を compose する。raw left result は outer-mode provenance、独立した
right result と sole expected-set input は explicit reserve provenance を保持する。
real expansion 3 本が left を再帰的に normalize し、right builtin-set role は
直接 normalize され、3 role すべてが terminal-RHS builtin-set identity 1 個へ
intern する。right-owned constraint 1 個だけ、left expected type なしで 2
`Inferred` term と 1 fact-free `Checked` membership を記録する。mode
declaration acceptance/inhabitation、membership truth/fact、implicit
closure/order、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は open の
ままである。

Task 144 MC-G020 current-state override: exact two-edge object-terminal
local-mode-chain left membership は task 72 の real AST-derived expansion 3 本
`OuterTwoEdgeObjectModeMembership -> MiddleTwoEdgeObjectModeMembership -> BaseTwoEdgeObjectModeMembership -> object`
と、tasks 125 / 140 / 142 / 143 の right-only expected-set two-binding
membership consumer を compose する。raw left result は outer-mode provenance、
独立した right result と sole expected-set input は explicit reserve provenance
を保持する。real expansion 3 本が left を terminal-RHS builtin-object identity
へ再帰的に normalize し、right role は distinct explicit-reserve builtin-set
identity へ直接 normalize される。right-owned constraint 1 個だけ、left
expected type なし、object/set coercion なしで 2 `Inferred` term と 1
fact-free `Checked` membership を記録する。mode declaration acceptance/
inhabitation、membership truth/fact、implicit closure/order、theorem
acceptance、proof、CoreIr、ControlFlowIr、VC は open のままである。

Task 145 MC-G020 current-state override: exact direct object-terminal
local-mode reserved-variable type assertion は task 55 の real AST-derived
`LocalObjectModeTypeAssertion -> object` expansion と tasks 122 / 138 の
identifier-result / 独立した formula-anchored asserted-type consumer を compose
する。raw subject result は written local-mode provenance、asserted builtin
`object` は独立した formula source node を保持する。real expansion 1 本が両
input を definition RHS を canonical source とする builtin-object identity 1
個へ normalize してから、1 `Inferred` term と 1 fact-free `Checked` type
assertion を記録する。この exact slice は `BindingId(0)` と source-order use
ordinal 1 を要求し、non-exact definition、reserve、formula、expansion payload
は fail closed する。mode declaration acceptance/inhabitation、formula-side
local-mode asserted-head extraction、general reachability/widening/`qua`、
object/set coercion、truth/fact、implicit closure/order、theorem acceptance、
proof、CoreIr、ControlFlowIr、VC は open のままである。

Task 146 MC-G020 current-state override: exact one-edge set-terminal
local-mode-chain reserved-variable type assertion は task 56 の real
AST-derived `ChainModeTypeAssertion -> BaseModeTypeAssertion -> set` expansion
2 本と、tasks 122 / 138 の identifier-result / 独立した formula-anchored
asserted-type consumer を compose する。raw subject result は written outer-mode
provenance、asserted builtin `set` は独立した formula source node を保持する。
real expansion 2 本が subject と asserted input を terminal definition RHS を
canonical source とする builtin-set identity 1 個へ再帰的に normalize して
から、1 `Inferred` term と 1 fact-free `Checked` type assertion を記録する。
この exact slice は `BindingId(0)` と source-order use ordinal 1 を要求し、
non-exact definition、reserve、formula、各 expansion payload は fail closed
する。mode declaration acceptance/inhabitation、formula-side local-mode
asserted-head extraction、general reachability/widening/`qua`、truth/fact、
implicit closure/order、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は
open のままである。

Task 147 MC-G020 current-state override: exact one-edge object-terminal local-
mode-chain reserved-variable type assertion は task 56 の real AST-derived
`ChainObjectModeTypeAssertion -> BaseObjectModeTypeAssertion -> object`
expansion 2 本と、tasks 122 / 145 / 146 の identifier-result / 独立した formula-
anchored asserted-type consumer を compose する。raw subject result は written
outer-mode provenance、asserted builtin `object` は独立した formula source node
を保持する。real expansion 2 本が subject と asserted input を terminal
definition RHS を canonical source とする builtin-object identity 1 個へ再帰的に
normalize してから、1 `Inferred` term と 1 fact-free `Checked` type assertion
を記録する。この exact slice は `BindingId(0)` と source-order use ordinal 1
を要求し、non-exact definition、reserve、formula、各 expansion payload は fail
closed する。mode declaration acceptance/inhabitation、formula-side local-mode
asserted-head extraction、general reachability/widening/`qua`、object/set
coercion、truth/fact、implicit closure/order、theorem acceptance、proof、CoreIr、
ControlFlowIr、VC は open のままである。

Task 148 MC-G020 current-state override: exact two-edge set-terminal local-mode-chain
reserved-variable type assertion は task 72 の real AST-derived
`OuterTwoEdgeModeTypeAssertion -> MiddleTwoEdgeModeTypeAssertion -> BaseTwoEdgeModeTypeAssertion -> set`
expansion 3 本と、tasks 122 / 146 / 147 の identifier-result / 独立した formula-
anchored asserted-type consumer を compose する。raw subject result は written
outer-mode provenance、asserted builtin `set` は独立した formula source node
を保持する。real expansion 3 本が subject と asserted input
を terminal definition RHS を canonical source とする builtin-set identity 1
個へ再帰的に normalize してから、1 `Inferred` term と 1 fact-free `Checked`
type assertion を記録する。この exact slice は `BindingId(0)` と source-order
use ordinal 1 を要求し、non-exact definition、reserve、formula、各 expansion
payload は fail closed する。mode declaration acceptance/inhabitation、formula-
side local-mode asserted-head extraction、general reachability/widening/`qua`、
truth/fact、implicit closure/order、theorem acceptance、proof、CoreIr、
ControlFlowIr、VC は open のままである。

Task 149 MC-G020 current-state override: exact two-edge object-terminal
local-mode-chain reserved-variable type assertion は task 72 の real AST-derived
`OuterTwoEdgeObjectModeTypeAssertion -> MiddleTwoEdgeObjectModeTypeAssertion -> BaseTwoEdgeObjectModeTypeAssertion -> object`
expansion 3 本と、tasks 122 / 145 / 147 / 148 の identifier-result / 独立した
formula-anchored asserted-type consumer を compose する。raw subject
result は written outer-mode provenance、asserted builtin `object` は独立した
formula source node を保持する。real expansion 3 本が両 input を terminal
definition RHS を canonical source とする builtin-object identity 1 個へ再帰的に
normalize してから、1 `Inferred` term と 1 fact-free `Checked` type assertion
を記録する。この exact slice は `BindingId(0)` と source-order use ordinal 1 を
要求し、non-exact definition、reserve、formula、各 expansion payload は fail
closed する。分類は `test_gap`、`source_drift`、`design_drift` であり、
`spec_gap` ではない。mode declaration acceptance/inhabitation、formula-side
local-mode asserted-head extraction、general reachability/widening/`qua`、
object/set coercion、truth/fact、implicit closure/order、theorem acceptance、
proof、CoreIr、ControlFlowIr、VC は open のままである。production route と
real frontend/resolver sidecar が exact active slice を guard する。

Task 150 MC-G020 current-state override: exact three-edge set-terminal local-mode-
chain reserved-variable type assertion は task 73 の real AST-derived
`OuterThreeEdgeModeTypeAssertion -> MiddleThreeEdgeModeTypeAssertion -> InnerThreeEdgeModeTypeAssertion -> BaseThreeEdgeModeTypeAssertion -> set`
expansion 4 本と、tasks 122 / 148 / 149 の identifier-result / 独立した formula-
anchored asserted-type consumer を compose する。raw subject result は written
outer-mode provenance、asserted builtin `set` は独立した formula source node
を保持しなければならない。real expansion 4 本が両 input を terminal
definition RHS を canonical source とする builtin-set identity 1 個へ再帰的に
normalize してから、1 `Inferred` term と 1 fact-free `Checked` type assertion
を記録する。この exact slice は `BindingId(0)` と source-order use ordinal 1
を要求し、non-exact definition、reserve、formula、各 expansion payload は
fail closed しなければならない。分類は `test_gap`、`source_drift`、
`design_drift` であり、`spec_gap` ではない。mode declaration acceptance/
inhabitation、formula-side local-mode asserted-head extraction、general
reachability/widening/`qua`、truth/fact、implicit closure/order、theorem
acceptance、proof、CoreIr、ControlFlowIr、VC は open のままである。
production route と real frontend/resolver sidecar が exact active slice を
guard する。

Task 151 MC-G020 current-state override: exact three-edge object-terminal local-mode-
chain reserved-variable type assertion は task 73 の real AST-derived
`OuterThreeEdgeObjectModeTypeAssertion -> MiddleThreeEdgeObjectModeTypeAssertion -> InnerThreeEdgeObjectModeTypeAssertion -> BaseThreeEdgeObjectModeTypeAssertion -> object`
expansion 4 本と、tasks 122 / 149 / 150 の identifier-result / 独立した formula-
anchored asserted-type consumer を compose する。raw subject result は written
outer-mode provenance、asserted builtin `object` は独立した formula source node
を保持しなければならない。real expansion 4 本が両 input を terminal
definition RHS を canonical source とする builtin-object identity 1 個へ再帰的に
normalize してから、1 `Inferred` term と 1 fact-free `Checked` type assertion
を記録する。この exact slice は `BindingId(0)` と source-order use ordinal 1
を要求し、non-exact definition、reserve、formula、各 expansion payload は fail
closed しなければならない。分類は `test_gap`、`source_drift`、
`design_drift` であり、`spec_gap` ではない。mode declaration acceptance/
inhabitation、formula-side local-mode asserted-head extraction、general
reachability/widening/`qua`、object/set coercion、truth/fact、implicit closure/
order、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は open のままである。
production route と real frontend/resolver sidecar が exact active slice を
guard する。

Task 152 MC-G020 current-state override: exact four-edge set-terminal local-mode-chain
reserved-variable type assertion は task 74 の real AST-derived
`TooDeepFourEdgeModeTypeAssertion -> OuterFourEdgeModeTypeAssertion -> MiddleFourEdgeModeTypeAssertion -> InnerFourEdgeModeTypeAssertion -> BaseFourEdgeModeTypeAssertion -> set`
expansion 5 本と、tasks 122 / 150 / 151 の identifier-result および独立した
formula-anchored asserted-type consumer を composition する。raw subject result
は written outermost-mode provenance、asserted builtin `set` は独立した formula
source node を保持する。expansion 5 本が両 input を terminal definition RHS を
canonical source とする builtin-set identity 1 個へ再帰的に normalize してから
1 `Inferred` term と 1 fact-free `Checked` type assertion を記録する。この exact
slice は `BindingId(0)` と source-order use ordinal 1 を要求し、non-exact
definition、reserve、formula、各 expansion payload は fail closed する。分類は
`test_gap`、`source_drift`、`design_drift` であり、`spec_gap` ではない。mode
declaration acceptance/inhabitation、formula-side local-mode asserted-head
extraction、general reachability/widening/`qua`、truth/fact、implicit closure/
order、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は open のままで
ある。production route と real frontend/resolver sidecar が exact active slice
を guard する。

Task 153 MC-G020 current-state override: exact four-edge object-terminal local-mode-
chain reserved-variable type assertion は task 74 の real AST-derived
`TooDeepFourEdgeObjectModeTypeAssertion -> OuterFourEdgeObjectModeTypeAssertion -> MiddleFourEdgeObjectModeTypeAssertion -> InnerFourEdgeObjectModeTypeAssertion -> BaseFourEdgeObjectModeTypeAssertion -> object`
expansion 5 本と、tasks 122/151/152 の identifier-result および独立した formula-
side asserted-type consumer を合成する。raw subject result は written
outermost-mode provenance を保持し、asserted builtin `object` は独立した formula
source node を保持する。全 expansion は両 input を terminal definition RHS に
canonical anchor された builtin-object identity 1 個へ再帰的に normalize し、1
`Inferred` term と 1 fact-free `Checked` type assertion を記録する。この exact
slice は `BindingId(0)` と source-order use ordinal 1 を要求し、non-exact
definition/reserve/formula または expansion payload 5 本のいずれかで fail closed
とする。分類は `test_gap`、`source_drift`、`design_drift` で `spec_gap` ではない。
declaration acceptance/inhabitation、formula-side local asserted-head、general
reachability/widening/`qua`、object/set coercion、truth/fact、closure/order、
theorem acceptance、proof/Core/ControlFlow/VC は open のままである。production
route と real sidecar が exact active slice を guard し、active count は 104 件で
ある。

Task 154 MC-G020 implementation specification: exact three-edge set-terminal
local-mode-chain reserved-variable equality は task 73 の real AST-derived
`OuterThreeEdgeModeEquality -> MiddleThreeEdgeModeEquality -> InnerThreeEdgeModeEquality -> BaseThreeEdgeModeEquality -> set`
expansion 4 本と task 134 の equality consumer を合成する。raw result/expected
input 4 個は written outer-mode provenance を保持し、両 operand は ordinal 1、2
で `BindingId(0)` へ解決し、全 role は terminal-RHS builtin-set identity 1 個へ
normalize されてから 2 `Inferred` term と 1 fact/deferred-free `Checked`
equality を記録する。exact definition/radix/expansion corruption、withheld-
family near miss、real frontend/resolver sidecar を必須とする。分類は
`test_gap`、`source_drift`、`design_drift` であり `spec_gap` ではない。mode
declaration acceptance/inhabitation、equality truth/fact、closure/order、theorem
acceptance、proof/Core/ControlFlow/VC は open のままである。fixture、expectation、
trace row、production route、full near-miss/corruption matrix、real frontend/
resolver sidecar が exact slice を guard し、active count は 105 件である。

Task 155 MC-G020 implementation specification: exact three-edge object-terminal
local-mode-chain reserved-variable equality は task 73 の real AST-derived
`OuterThreeEdgeObjectModeEquality -> MiddleThreeEdgeObjectModeEquality -> InnerThreeEdgeObjectModeEquality -> BaseThreeEdgeObjectModeEquality -> object`
expansion 4 本と task 135 の equality consumer を合成する。raw result/expected
input 4 個は written outer-mode provenance を保持し、両 operand は ordinal 1、2
で `BindingId(0)` へ解決し、全 role は terminal-RHS builtin-object identity 1 個
へ normalize されてから 2 `Inferred` term と 1 fact/deferred-free `Checked`
equality を記録する。exact definition/radix/expansion corruption、withheld-
family near miss、real frontend/resolver sidecar を必須とする。分類は
`test_gap`、`source_drift`、`design_drift` であり `spec_gap` ではない。mode
declaration acceptance/inhabitation、object/set coercion、equality truth/fact、
closure/order、theorem acceptance、proof/Core/ControlFlow/VC は open のままで
ある。fixture、expectation、trace row、production route、full near-miss/
corruption matrix、real frontend/resolver sidecar が exact slice を guard し、
active count は 106 件である。

Task 156 MC-G020 implementation specification: exact three-edge set-terminal
local-mode-chain reserved-variable inequality は task 73 の real AST-derived
`OuterThreeEdgeModeInequality -> MiddleThreeEdgeModeInequality -> InnerThreeEdgeModeInequality -> BaseThreeEdgeModeInequality -> set`
expansion 4 本と task 136 の pre-desugaring inequality consumer を合成する。
raw result/expected input 4 個は written outer-mode provenance を保持し、両
operand は ordinal 1、2 で `BindingId(0)` へ解決し、全 role は terminal-RHS
builtin-set identity 1 個へ normalize されてから 2 `Inferred` term と 1 fact/
deferred-free pre-desugaring `Checked` inequality を記録する。exact definition/
radix/expansion corruption、withheld-family near miss、real frontend/resolver
sidecar を必須とする。分類は `test_gap`、`source_drift`、`design_drift` であり
`spec_gap` ではない。mode declaration acceptance/inhabitation、inequality
desugaring、truth/fact、closure/order、theorem acceptance、proof/Core/
ControlFlow/VC は open のままである。fixture、expectation、trace row、
production route、full near-miss/corruption matrix、real frontend/resolver
sidecar が exact slice を guard し、active count は 107 件である。

Task 157 MC-G020 implementation specification: exact three-edge object-terminal
local-mode-chain reserved-variable inequality は task 73 の real AST-derived
`OuterThreeEdgeObjectModeInequality -> MiddleThreeEdgeObjectModeInequality -> InnerThreeEdgeObjectModeInequality -> BaseThreeEdgeObjectModeInequality -> object`
expansion 4 本と task 137 の builtin-object pre-desugaring inequality consumer
を合成する。raw result/expected input 4 個は written outer-mode provenance を
保持し、両 operand は ordinal 1、2 で `BindingId(0)` へ解決し、全 role は
terminal-RHS builtin-object identity 1 個へ normalize されてから 2 `Inferred`
term と 1 fact/deferred-free pre-desugaring `Checked` inequality を記録する。
exact definition/radix/expansion corruption、withheld-family near miss、real
frontend/resolver sidecar を必須とする。分類は `test_gap`、`source_drift`、
`design_drift` であり `spec_gap` ではない。mode declaration acceptance/
inhabitation、object/set coercion、inequality desugaring、truth/fact、closure/
order、theorem acceptance、proof/Core/ControlFlow/VC は open のままである。
fixture、expectation、trace row、production route、full near-miss/corruption
matrix、real frontend/resolver sidecar が exact slice を guard し、active count
は 108 件である。

Task 158 MC-G020 implementation specification: exact three-edge set-terminal
local-mode-chain left reserved-variable membership は task 73 の real AST-
derived
`OuterThreeEdgeModeMembership -> MiddleThreeEdgeModeMembership -> InnerThreeEdgeModeMembership -> BaseThreeEdgeModeMembership -> set`
expansion 4 本と task 143 の two-binding right-only expected-set membership
consumer を compose する。raw left result は outer-mode provenance、独立した
right result と sole expected input は explicit-set reserve provenance を保持し、
left expected input は持たない。operand は source-order ordinal 2/3 で
`BindingId(0/1)` へ解決し、全 4 expansion は 3 role を terminal-RHS builtin-
set identity 1 個へ normalize してから 2 `Inferred` term と exactly one right-
owned constraint を持つ 1 fact/deferred-free `Checked` membership を記録する。
independent definition/radix/expansion corruption、withheld-family near miss、
real frontend/resolver sidecar を必須とする。分類は `test_gap`、
`source_drift`、`design_drift` であり `spec_gap` ではない。mode declaration
acceptance/inhabitation、membership truth/fact、implicit closure/order、theorem
acceptance、proof/Core/ControlFlow/VC、object-terminal behavior、broader depth
は open のままである。fixture、expectation、trace row、production route、
full near-miss/corruption matrix、real frontend/resolver sidecar が exact slice
を guard し、active count は 109 件である。

Task 159 MC-G020 current-state override: exact distinct-binding shared-
reserve membership は task 123 の one-item/two-binding/shared-range producer と
tasks 120/125 の right-only expected-set membership consumer を compose する。
`x/y` は ordinal 2/3 で `BindingId(0/1)` へ解決され、left result、right result、
sole right expected input は written set range 1 個を保持し、left expected input
はなく、3 role は builtin-set identity 1 個へ normalize してから 2 `Inferred`
term と right-owned constraint 1 個を持つ 1 fact/deferred-free `Checked`
membership を記録する。分類は `test_gap`、`source_drift`、`design_drift` で
`spec_gap` ではない。truth/fact、closure/order、theorem acceptance、proof/Core/
ControlFlow/VC、separate declaration、broader shape は open のままである。
fixture/expectation/trace row、production route、full near-miss/corruption
matrix、real sidecar が exact slice を guard するため active count は 110 件である。

Task 160 MC-G020 current-state override: exact distinct-binding shared-reserve
inequality は task 123 の one-item/two-binding/shared-range producer と task 121 の
pre-desugaring inequality consumer を compose する。`x/y` は ordinal 2/3 で
`BindingId(0/1)` へ解決され、両 binding と left/right result/expected role 4 個は
written set range 1 個を保持して shared-source builtin-set identity 1 個へ
normalize してから、2 `Inferred` term と 2 ordered operand-owned constraint を
持つ 1 fact/deferred-free `Checked` inequality を記録しなければならない。分類は
`test_gap`、`source_drift`、`design_drift` であり `spec_gap` ではない。
desugaring/truth/fact、closure/order、theorem acceptance、proof/Core/ControlFlow/VC、
separate declaration、broader shape は open のままである。test-first fixture/
expectation/trace contract、production route、near-miss/corruption guard、real
sidecar が exact slice を保護するため active count は 111 件である。

Task 161 MC-G020 current-state override: exact multiple-reserve-declaration
inequality は task 124 の two-item/two-binding/distinct-written-range producer と
task 160 の pre-desugaring inequality consumer を compose する。`x/y` は ordinal
2/3 で `BindingId(0/1)` へ解決され、各 operand の result/expected role は固有の
written set range を保持しながら全 4 role を earlier `x` range に canonical anchor
された builtin-set identity 1 個へ normalize し、2 `Inferred` term と 2 ordered
constraint を持つ 1 fact/deferred-free `Checked` inequality を記録しなければならない。
分類は `test_gap`、`source_drift`、`design_drift` であり `spec_gap` ではない。
desugaring/truth/fact、closure/order、theorem acceptance、proof/Core/ControlFlow/VC、
shared range、broader shape は open のままである。source/trace contract、production
route、near-miss/corruption guard、real sidecar が exact slice を保護するため active
count は 112 件である。

Task 162 MC-G020 current-state override: exact multiple-reserve-declaration
membership は task 124 の two-item/two-binding/distinct-written-range producer と
task 120/159 の right-only expected-set membership consumer を compose する。
`x/y` は ordinal 2/3 で `BindingId(0/1)` へ解決され、left result は first written
set range、right result と sole right expected input は second range を保持し、left
expected input は持たない。3 role は earlier `x` range に canonical anchor された
builtin-set identity 1 個へ normalize してから、2 `Inferred` term と exactly one
right-owned constraint を持つ 1 fact/deferred-free `Checked` membership を記録する。
分類は `test_gap`、`source_drift`、`design_drift` であり `spec_gap` ではない。
membership truth/fact、closure/order、theorem acceptance、proof/Core/ControlFlow/
VC、shared range、broader shape は open のままである。fixture/expectation/trace
contract、production routing、near-miss/corruption guard、real sidecar が exact
slice を保護するため active count は 113 件である。

Task 163 MC-G020 current-state override: exact three-edge local-object-mode-chain
left membership を `spec_gap` ではなく `test_gap`、`source_drift`、
`design_drift` と分類する。Chapter 3、4、7、13、14.5.3、16 は test-first
source を直接支える。task 73/151/155/157 は real four-expansion object-terminal
producer、task 144 は real object-left/set-right membership consumer を提供する。
実装は ordinal 2/3 の `BindingId(0/1)`、raw outer-mode left
result、独立した explicit-set right result / sole right expected input、no left
expected input、distinct terminal-object-RHS / explicit-set identity、2
`Inferred` term、exactly one right-owned constraint を持つ 1 fact/deferred-free
`Checked` membership を保持しなければならない。truth/fact、object/set
coercion、closure/order、theorem acceptance、proof/Core/ControlFlow/VC、他の
depth、broader shape は open のままである。production routing、near-miss/
corruption guard、real frontend/resolver sidecar が active runner 114 を保護する。

Task 164 MC-G020 current-state override: exact four-edge set-terminal
local-mode-chain left membership を `spec_gap` ではなく `test_gap`、
`source_drift`、`design_drift` と分類する。Chapter 4、7、13、14.5.3、16
が test-first source を直接支え、task 74/152 が real five-expansion producer、
task 158 が real set-left/set-right membership consumer を提供する。contract
は ordinal 2/3 の `BindingId(0/1)`、raw outermost-mode left provenance、独立
した explicit-set right result/sole expected provenance、no left expected
input、source-derived expansion 5 本、terminal-set-RHS identity 1 個、2
`Inferred` term、exactly one right-owned constraint を持つ 1 fact/deferred-free
`Checked` membership を要求する。truth/fact、closure/order、theorem
acceptance、proof/Core/ControlFlow/VC、object-terminal behavior、他 depth、
broader shape は open のままである。fixture/expectation、6 trace backlink、
exact route、corruption/near-miss coverage、real sidecar が active runner 115
を保護する。

Task 165 MC-G020 current-state override: exact four-edge object-terminal local-
mode-chain left membership を `spec_gap` ではなく `test_gap`、`source_drift`、
`design_drift` と分類する。Chapter 3、4、7、13、14.5.3、16 が test-first
source を直接支える。task 74/153 が real five-expansion object-terminal
producer、task 163 が real object-left/set-right membership consumer を提供
する。exact contract は ordinal 2/3 の `BindingId(0/1)`、raw outermost-mode
left provenance、独立した explicit-set right result / sole expected provenance、
no left expected input、source-derived expansion 5 本、distinct terminal-
object-RHS / explicit-set identity、2 `Inferred` term、exactly one right-owned
constraint を持つ 1 fact/deferred-free `Checked` membership を保持する。
truth/fact、object/set coercion、closure/order、theorem acceptance、proof/Core/
ControlFlow/VC、他 depth、broader shape は open のままとする。fixture、
expectation、trace backlink 6 件、exact production route、corruption/near-miss
coverage、real frontend/resolver sidecar が active runner 116 を保護する。

Task 166 MC-G020 current-state override: exact four-edge set-terminal
local-mode-chain equality を `spec_gap` ではなく `test_gap`、`source_drift`、
`design_drift` と分類する。Chapter 4、7、13、14.5.2、16 が test-first source
を支え、task 74/152 が real five-expansion producer、task 154 が equality
consumer を提供する。exact contract は ordinal 1/2 の `BindingId(0)`、raw
outermost-mode result/expected input 4 個、source-derived expansion 5 本、
terminal-set-RHS identity 1 個、2 `Inferred` term、1 fact/deferred-free
`Checked` equality、ordered operand-owned expected constraint 2 個を保持する。
fixture、trace backlink 6 件、exact production route、corruption matrix、real
frontend/resolver sidecar が active runner 117 を保護する。declaration acceptance/inhabitation、
truth/fact、closure/order、theorem acceptance、proof/Core/ControlFlow/VC、
object-terminal behavior、他 depth、broader shape は open のままである。

Task 167 MC-G020 current-state override: exact four-edge object-terminal
local-mode-chain equality を `spec_gap` ではなく `test_gap`、`source_drift`、
`design_drift` と分類する。Chapter 3、4、7、13、14.5.2、16 が test-first
source を支え、tasks 74/153 が real five-expansion producer、task 155 が
equality consumer を提供する。exact contract は ordinal 1/2 の
`BindingId(0)`、raw outermost-mode result/expected input 4 個、source-derived
expansion 5 本、terminal-object-RHS identity 1 個、2 `Inferred` term、1 fact/
deferred-free `Checked` equality、ordered operand-owned expected constraint 2
個を object/set coercion なしで保持する。fixture、trace backlink 6 件、
production route、full corruption matrix、real frontend/resolver sidecar が
active runner 118 を保護する。declaration
acceptance/inhabitation、truth/fact、closure/order、theorem acceptance、proof/
Core/ControlFlow/VC、set-terminal behavior、他 depth、broader shape は open の
ままである。

Task 168 MC-G020 current-state override: exact four-edge set-terminal local-mode-chain
inequality を `spec_gap` ではなく `test_gap`、`source_drift`、`design_drift`
と分類する。Chapter 4、7、13、14.5.2、16 が test-first source を支え、
tasks 74/152 が real five-expansion producer、task 156 が pre-desugaring
inequality consumer を提供する。exact contract は ordinal 1/2 の
`BindingId(0)`、raw outermost-mode result/expected input 4 個、source-derived
expansion 5 本、terminal-set-RHS identity 1 個、2 `Inferred` term、1 fact/
deferred-free pre-desugaring `Checked` inequality、ordered operand-owned
expected constraint 2 個を保持する。fixture、trace backlink 6 件、production
routing、full corruption coverage、real sidecar が active runner 119 を保護する。
declaration acceptance/inhabitation、desugaring/truth/
fact、closure/order、theorem acceptance、proof/Core/ControlFlow/VC、object-
terminal behavior、他 depth、broader shape は open のままである。

Task 169 MC-G020 current-state override: exact four-edge object-terminal local-mode-chain
inequality を `spec_gap` ではなく `test_gap`、`source_drift`、`design_drift`
と分類する。Chapter 3、4、7、13、14.5.2、16 が test-first source を支え、
tasks 74/153 が real five-expansion producer、task 157 が pre-desugaring
inequality consumer を提供する。exact contract は ordinal 1/2 の
`BindingId(0)`、raw outermost-mode result/expected input 4 個、source-derived
expansion 5 本、terminal-object-RHS identity 1 個、2 `Inferred` term、1 fact/
deferred-free pre-desugaring `Checked` inequality、ordered operand-owned
expected constraint 2 個を object/set coercion なしで保持する。fixture、trace
backlink 6 件、production routing、full corruption coverage、real sidecar が
active runner 120 を保護する。declaration acceptance/
inhabitation、desugaring/truth/fact、closure/order、theorem acceptance、proof/
Core/ControlFlow/VC、set-terminal behavior、他 depth、broader shape は open の
ままである。

Task 172 MC-G020 current-state override: exact set-terminal local-mode long-chain
reserved-variable equality を `spec_gap` ではなく `test_gap`、`source_drift`、
`design_drift` と分類する。Chapter 4、7、13、14.5.2、16 が test-first source
を支え、task 74 が active long-chain fixture で既に実行する real seven-
expansion producer、task 166 が equality consumer を提供する。contract は
ordinal 1/2 の `BindingId(0)`、raw `ChainMode6` result/expected input 4 個、
source-derived expansion 7 本、terminal `BaseMode` RHS の builtin-set identity
1 個、2 `Inferred` term、1 fact/deferred-free `Checked` equality、ordered
operand-owned expected constraint 2 個を保持する。exact source routing、full
near-miss / corruption coverage、real frontend/resolver sidecar が active
runner 121 を保護する。
declaration acceptance/inhabitation、truth/fact、closure/order、theorem
acceptance、proof/Core/ControlFlow/VC payload、imported/attributed/argument-
bearing または別 chain shape、general unbounded semantics は open のままで
ある。

Task 173 MC-G020 current-state override: exact set-terminal local-mode long-chain
inequality を `spec_gap` ではなく `test_gap`、`source_drift`、`design_drift`
と分類する。Chapter 4、7、13、14.5.2、16 が source を支え、task 74 が real
expansion 7 本、task 168 が real pre-desugaring inequality consumer を提供する。
raw `ChainMode6` role 4 個、ordinal 1/2 の `BindingId(0)`、terminal `BaseMode`
RHS identity 1 個、2 `Inferred` term、1 fact/deferred-free checked inequality、
ordered constraint 2 個を保持する。exact routing、full guard、real sidecar が
active runner 122 を保護する。desugaring/truth/fact、acceptance、closure/order、theorem/proof/
Core/ControlFlow/VC、別 chain、general semantics は open のままである。

Task 174 MC-G020 current-state override: exact set-terminal local-mode long-chain left
reserved-variable membership を `spec_gap` ではなく `test_gap`、
`source_drift`、`design_drift` と分類する。Chapter 4、7、13、14.5.3、16 が
test-first source を支え、task 74 が real seven-expansion producer、task 164 が
right-only expected-set membership consumer を提供する。raw `ChainMode6` left
result、独立した explicit-set right result と sole right expected input、ordinal
2/3 の `BindingId(0/1)`、terminal `BaseMode` RHS identity 1 個、left expected
input なし、2 `Inferred` term、1 fact/deferred-free checked membership、right-
owned constraint 1 個を保持する。exact routing、membership-specific corruption
test、shared full structural guard、real sidecar が active runner 123 を保護する。
membership truth/fact、acceptance、closure/order、theorem/proof/Core/
ControlFlow/VC、別 chain、general semantics は open のままである。

Task 175 MC-G020 current-state override: exact set-terminal local-mode long-chain
reserved-variable type assertion を `spec_gap` ではなく `test_gap`、
`source_drift`、`design_drift` と分類する。Chapter 3、4、7、13、14.2.3、16 が
test-first source を支え、task 74 が real seven-expansion producer、task 152 が
normalized-reflexive type-assertion consumer を提供する。raw `ChainMode6`
subject result、独立した formula-side builtin-set asserted input、ordinal 1 の
`BindingId(0)`、terminal `BaseMode` RHS identity 1 個、1 `Inferred` term、general
reachability を用いない 1 fact/deferred-free checked type assertion を保持する。
exact production route、task 172 の shared structural guard、type-assertion-
specific corruption test、real frontend/resolver sidecar が active runner 124 を
保護する。widening/`qua`、assertion truth/fact、acceptance、closure/order、theorem/proof/
Core/ControlFlow/VC、別 chain、general semantics は open のままである。

Task 176 MC-G020 current-state override: exact builtin-object-terminal local-mode
long-chain reserved-variable equality を `spec_gap` ではなく `test_gap`、
`source_drift`、`design_drift` と分類する。Chapter 3、4、7、13、14.5.2、16 が
test-first source を支え、task 74 が real AST-bounded object-terminal chain
producer、task 167 が object-normalizing equality consumer を提供する。raw
`ChainObjectMode6` result/expected input 4 個、ordinal 1/2 の `BindingId(0)`、
real expansion 7 本、terminal `BaseObjectMode` RHS identity 1 個、2 `Inferred`
term、1 fact/deferred-free checked equality、ordered constraint 2 個を object/set
coercion なしで保持する。exact production route、task 172 の shared structural
guard、object-specific
corruption test、real frontend/resolver sidecar が active runner 125 を保護する。
truth/fact、acceptance、closure/order、theorem/proof/
Core/ControlFlow/VC、別 chain、general semantics は open のままである。

Task 177 MC-G020 current-state override: exact builtin-object-terminal local-mode
long-chain reserved-variable inequality を `spec_gap` ではなく `test_gap`、
`source_drift`、`design_drift` と分類する。Chapter 3、4、7、13、14.5.2、16 が
test-first source を支え、task 74 が real AST-bounded object-terminal chain
producer、task 169 が object-normalizing pre-desugaring inequality consumer を
提供する。raw `ChainObjectMode6` result/expected input 4 個、ordinal 1/2 の
`BindingId(0)`、real expansion 7 本、terminal `BaseObjectMode` RHS identity 1 個、
2 `Inferred` term、1 fact/deferred-free checked inequality、ordered constraint 2
個を object/set coercion なしで保持する。exact production route、task 172 の
shared structural guard、object-specific corruption test、real frontend/resolver
sidecar が active runner 126 を保護する。desugaring、truth/fact、
acceptance、closure/order、theorem/proof/Core/ControlFlow/VC、別 chain、general
semantics は open のままである。

Task 178 MC-G020 current-state override: exact builtin-object-terminal local-mode
long-chain left reserved-variable membership を `spec_gap` ではなく `test_gap`、
`source_drift`、`design_drift` と分類する。Chapter 3、4、7、13、14.5.3、16 が
test-first source を支え、task 74 が real AST-bounded object-terminal chain
producer、task 165 が object-left/set-right membership consumer を提供する。raw
`ChainObjectMode6` left result、独立した explicit-set right result/sole expected
input、ordinal 2/3 の `BindingId(0/1)`、real expansion 7 本、distinct terminal-
object-RHS と explicit-set identity、left expected input なし、2 `Inferred`
term、1 fact/deferred-free checked membership、right-owned constraint 1 個を
object/set coercion なしで保持する。exact route、task 172 shared guard、
membership/object-specific corruption test、real sidecar が active runner 127 を
保護する。truth/fact、acceptance、closure/order、theorem/proof/
Core/ControlFlow/VC、別 chain、general semantics は open のままである。

Task 179 MC-G020 current-state override: exact builtin-object-terminal local-mode
long-chain reserved-variable normalized-reflexive type assertion を `spec_gap`
ではなく `test_gap`、`source_drift`、`design_drift` と分類する。Chapter 3、4、
7、13、14.2.3、16 が test-first source を支え、task 74 が real AST-bounded
object-terminal chain producer、task 153 が object-normalizing type-assertion
consumer、task 175 が seven-expansion sibling guard pattern を提供する。raw
`ChainObjectMode6` subject result、独立した formula-side builtin-object asserted
input、ordinal 1 の `BindingId(0)`、real expansion 7 本、terminal-object-RHS
identity 1 個、1 `Inferred` term、general reachability と object/set coercion を
用いない 1 fact/deferred-free normalized-reflexive checked type assertion を
保持する。exact route、task 172 shared guard、task 153 の real object consumer/
source near miss、builtin-set asserted head または raw subject provenance
corruption を reject するよう task 175 から適応した matched-output guard、real
sidecar が active runner 128 を保護する。widening/`qua`、truth/fact、acceptance、closure/order、theorem/proof/
Core/ControlFlow/VC、別 chain、general semantics は open のままである。

Task 180 MC-G020 current-state override: exact standalone
`SourceDerivedContradictionConstantBoundary: contradiction` leaf を `spec_gap`
ではなく `test_gap`、`source_drift`、`design_drift` と分類する。Chapter 14、16
は leaf と theorem slot を直接支える。新規 exact standalone extractor は既存
contradiction-kind mapping と theorem-shape validation pattern を再利用し、real
leaf site/range を module-root context で deferred reason なしに既存 checker
consumer へ渡す。term/type/constraint/candidate/fact/deferred/diagnostic payload
が空の 1 checked formula に type/well-formedness credit を与える。exact/near-
miss/corruption と real-sidecar guard が active runner 129 を保護する。
falsehood/fact publication、theorem acceptance、proof-goal closure、implicit
closure/child graph、`formula_statement`、proof、CoreIr、ControlFlowIr、VC は
open のままである。

Task 182 MC-G020 current-state override: exact direct formula-side local-mode
asserted head は `spec_gap` ではなく `test_gap`、narrow `source_drift`、
`design_drift` を閉じる。Chapter 3、4、7、13、14.2.3、16 は `mode
LocalModeAssertedHeadDef: LocalModeAssertedHead is set;` を含む exact definition
block と test-first source を直接支える。task 55 は
real direct bare-mode expansion producer、tasks 122/138 は normalized-reflexive
type-assertion consumer と direct local-mode subject route を提供する。exact source
は同じ resolved mode 向けの独立した reserve-subject と formula-side asserted raw
input、real expansion 1 個、ordinal 1 の `BindingId(0)`、terminal-RHS builtin-set
identity 1 個へ intern する known type entry 3 個、1 inferred term、general
reachability を用いない 1 fact/deferred-free checked formula を保持する。
exact/near-miss/corruption、production-route、real sidecar guard が active runner
130 を保護する。mode declaration acceptance/
inhabitation、widening/`qua`、truth/fact、theorem/proof/Core/ControlFlow/VC、
他 asserted head/chain、general semantics は open のままである。

Task 183 MC-G020 current-state override: exact direct object-terminal formula-
side local-mode asserted head は `spec_gap` ではなく `test_gap`、narrow
`source_drift`、`design_drift` を閉じる。Chapter 3、4、7、13、14.2.3、16 は `mode
LocalObjectModeAssertedHeadDef: LocalObjectModeAssertedHead is object;` を持つ
exact source を直接支える。task 55 は real bare-object mode expansion、tasks
145/182 は normalized object consumer と same-symbol formula-side asserted-head
producer を提供する。exact route は独立した raw reserve-subject/formula-side
asserted input、real expansion 1 個、ordinal 1 の `BindingId(0)`、terminal-RHS
builtin-object identity 1 個へ intern する known type entry 3 個、1 inferred term、
general reachability と object/set coercion を用いない 1 fact/deferred-free checked
formula を保持する。exact/near-miss/corruption、production-route、real sidecar
guard が active runner 131 を保護する。
declaration acceptance/inhabitation、truth/fact、theorem/proof/Core/ControlFlow/
VC、他 asserted head/chain、general semantics は open のままである。

Task 184 MC-G020 current-state override: exact one-edge set-terminal same-outer-mode
asserted head は `spec_gap` ではなく `test_gap`、narrow `source_drift`、
`design_drift` である。Chapter 3、4、7、13、14.2.3、16 は
`mode BaseModeAssertedHeadDef: BaseModeAssertedHead is set;` と `mode
ChainModeAssertedHeadDef: ChainModeAssertedHead is BaseModeAssertedHead;` を
含む ordered definition block 2 個、outer-mode reserve 1 個、
`ChainedLocalModeAssertedHeadPayloadBoundary: x is ChainModeAssertedHead;`
から成る test-first source を直接支える。task 56 は real one-edge expansion
producer、tasks 146/182 は normalized set consumer と same-symbol formula-side
asserted-head producer を提供する。exact route は同じ outer symbol 向けの独立した
raw reserve-subject/formula-side asserted input、real expansion 2 個、ordinal 1 の
`BindingId(0)`、terminal base-definition-RHS builtin-set identity 1 個へ intern
する known type entry 3 個、1 inferred term、general reachability を用いない
1 fact/deferred-free checked formula を保持する。exact/near-miss/corruption、
production-route、real sidecar guard が active runner 132 を保護する。
declaration acceptance/inhabitation、widening/`qua`、truth/fact、closure/order、
theorem/proof/Core/ControlFlow/VC、object/deeper/他 asserted head、general chain
semantics は open のままである。

Task 185 MC-G020 current-state override: exact one-edge object-terminal same-outer-mode
asserted head を `spec_gap` ではなく `test_gap`、narrow `source_drift`、
`design_drift` と分類する。Chapter 3、4、7、13、14.2.3、16 は `mode
BaseObjectModeAssertedHeadDef: BaseObjectModeAssertedHead is object;` と `mode
ChainObjectModeAssertedHeadDef: ChainObjectModeAssertedHead is
BaseObjectModeAssertedHead;` を含む ordered definition block 2 個、outer-mode
reserve 1 個、`ChainedLocalObjectModeAssertedHeadPayloadBoundary: x is
ChainObjectModeAssertedHead;` から成る test-first source を直接支える。task 56
は real one-edge expansion producer、tasks 147/183/184 は normalized object
consumer と same-symbol recursive formula-side asserted-head seam を提供する。
exact route は同じ outer symbol 向けの独立した raw reserve-subject/formula-side
asserted input、real expansion 2 個、ordinal 1 の `BindingId(0)`、terminal base-
definition-RHS builtin-object identity 1 個へ intern する known type entry 3 個、
1 inferred term、general reachability、widening、`qua`、object/set coercion を
用いない 1 fact/deferred-free checked formula を保持する。exact/near-miss/
corruption、production-route、real sidecar guard が active runner 133 を保護する。
shared trace backlink 5 個と dedicated row 1 個が exact credit を担う。imported
provenance、declaration/attribute acceptance、
broader term/formula、child graph、theorem/proof/Core/ControlFlow/VC、deeper/他
asserted head、general chain semantics は open のままである。module layout 更新は
不要である。

Task 186 MC-G020 current-state override: exact two-edge set-terminal same-outer-
mode asserted head を `spec_gap` ではなく `test_gap`、narrow `source_drift`、
`design_drift` と分類する。Chapter 3、4、7、13、14.2.3、16 は `mode
BaseTwoEdgeModeAssertedHeadDef: BaseTwoEdgeModeAssertedHead is set;`、`mode
MiddleTwoEdgeModeAssertedHeadDef: MiddleTwoEdgeModeAssertedHead is
BaseTwoEdgeModeAssertedHead;`、`mode OuterTwoEdgeModeAssertedHeadDef:
OuterTwoEdgeModeAssertedHead is MiddleTwoEdgeModeAssertedHead;` を含む ordered
definition block 3 個、`OuterTwoEdgeModeAssertedHead` の reserve 1 個、
`TwoEdgeLocalModeAssertedHeadPayloadBoundary: x is
OuterTwoEdgeModeAssertedHead;` を直接支える。task 72 は real two-edge expansion
producer、tasks 148/184 は normalized set consumer と same-symbol formula-side
asserted-head seam、task 185 は object-terminal sibling guard を提供する。exact
route は同じ resolved outer symbol 向けの distinct raw reserve-subject/formula-
side asserted site/range、real expansion 3 個、ordinal 1 の `BindingId(0)`、terminal
`BaseTwoEdgeModeAssertedHead` definition RHS builtin-set identity 1 個へ intern
する known type entry 3 個、reachability、widening、`qua` を用いない 1
`Inferred` term と 1 fact/deferred-free normalized-reflexive `Checked` formula を
保持しなければならない。exact/near-miss/corruption、production-route、real
frontend/resolver-sidecar guard は各 missing expansion、wrong link/terminal/order/
depth、duplicate、forward/recovered/contextual/parameterized/argument-bearing/
attributed definition、direct/one-edge/deeper/object-terminal shape、non-exact
reserve/formula、wrong subject、builtin/base/middle/other/attributed/argument-
bearing asserted head、imported/ambiguous provenance、recovery、extra item、
collapsed provenance、builtin-object output corruption を reject する。shared
trace backlink 5 個と dedicated row 1 個が existing expectation を rebaseline
せず active runner 134 を保護する。object-terminal/deeper/
imported asserted head、declaration/attribute acceptance、broader term/formula/
child graph、truth/fact、theorem/proof/Core/ControlFlow/VC、general chain
semantics は open のままである。module layout 更新は不要である。

Task 187 MC-G020 current-state override: exact two-edge object-terminal same-outer-
mode asserted head を `spec_gap` ではなく `test_gap`、narrow `source_drift`、
`design_drift` と分類する。Chapter 3、4、7、13、14.2.3、16 は ordered
definition `mode BaseTwoEdgeObjectModeAssertedHeadDef:
BaseTwoEdgeObjectModeAssertedHead is object;`、`mode
MiddleTwoEdgeObjectModeAssertedHeadDef: MiddleTwoEdgeObjectModeAssertedHead is
BaseTwoEdgeObjectModeAssertedHead;`、`mode OuterTwoEdgeObjectModeAssertedHeadDef:
OuterTwoEdgeObjectModeAssertedHead is MiddleTwoEdgeObjectModeAssertedHead;`、
outer-mode reserve 1 個、`TwoEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
OuterTwoEdgeObjectModeAssertedHead;` を支える。tasks 72/149 は real object-
terminal producer/consumer、tasks 185/186 は same-symbol asserted-head route
pattern を提供する。exact route は同じ local outer symbol 向けの distinct raw
subject/asserted site/range、real expansion 3 個、ordinal 1 の `BindingId(0)`、
terminal base-definition-RHS builtin-object identity 1 個へ intern する known
entry 3 個、expected constraint、reachability、widening、`qua`、object/set
coercion を持たない 1 `Inferred` term と 1 fact/deferred-free normalized-
reflexive `Checked` assertion を保持する。exact/near-miss/
corruption、production-route、real frontend/resolver-sidecar guard は missing/
wrong/duplicate/reordered/forward/recovered/contextual/parameterized/argument-
bearing/attributed link または wrong label、set-terminal/direct/one-edge/deeper
shape、builtin/base/middle/other asserted head および attributed/argument-bearing
formula-side asserted head、imported Base/Middle/Outer と imported/ambiguous
asserted provenance、extra/collapsed provenance、`BuiltinSet` output corruption
を reject する。shared trace backlink 5 個と dedicated row 1 個により active
runner 135 を保護する。positive imported semantics、declaration/attribute
acceptance、broader term/formula/child graph、truth/fact、implicit closure/order、
theorem acceptance、proof/Core/ControlFlow/VC、general chain semantics は open
のままである。Step 5 は active、
Steps 6/7 は deferred のまま。module layout 更新は不要である。

Task 188 MC-G020 current-state override: exact builtin-object same-binding
equality を `spec_gap` ではなく `test_gap`、narrow `source_drift`、
`design_drift` と分類する。Chapter 3、4、13、14.5.2、16 は `reserve x for
object; theorem ReservedObjectVariableEqualityPayloadBoundary: x = x;` を直接
支える。tasks 48/125 は real builtin-object reserve handoff、task 119 は exact
same-binding equality route/builder、task 128 は real builtin-object normalization
consumer を提供する。route は ordinal 1/2 を `BindingId(0)` に解決し、written
`object` range 1 個上の distinct result/expected role site 4 個、canonical
builtin-object identity 1 個、`Inferred` variable 2 個、ordered expected constraint
2 個、fact/deferred-free `Checked` equality 1 個を保持しなければならない。
exact/near-miss、matched-output、canonical-source、`BuiltinSet` corruption、
route-order、real frontend/resolver-sidecar guard を必須とする。shared trace
backlink 5 個 + dedicated row 1 個により既存 expectation を変更せず active runner
136 を保護する。object/set coercion、general/non-reflexive object
equality、truth/fact、implicit closure/order、theorem acceptance、proof/Core/
ControlFlow/VC、broader source shape は open のままである。Step 5 は active、
Steps 6/7 は deferred のまま。module layout 更新は不要である。

Task 189 MC-G020 current-state override: exact builtin-object same-binding
normalized-reflexive type assertion を `spec_gap` ではなく `test_gap`、narrow
`source_drift`、`design_drift` と分類する。Chapter 3、4、13、14.2.3、16 は
`reserve x for object; theorem
ReservedObjectVariableTypeAssertionPayloadBoundary: x is object;` を直接支える。
tasks 48/125/188 は real builtin-object reserve handoff、task 122 は exact one-
subject assertion route/builder、task 145 は real builtin-object normalization
consumer を提供する。route は ordinal 1 を `BindingId(0)` に解決し、distinct
reserve-subject result/formula-side asserted site/range、written reserve type を
anchor とする canonical builtin-object identity 1 個、`Inferred` variable 1 個、
known type entry 3 個、expected constraint 0 個、fact/deferred-free `Checked`
assertion 1 個を保持しなければならない。exact/near-miss、matched-output、
canonical-source、`BuiltinSet` corruption、route-order、real frontend/resolver-
sidecar guard を必須とする。shared trace backlink 5 個 + dedicated row 1 個により
既存 expectation を変更せず active runner 137 を保護する。
reachability/widening/`qua`、object/set coercion、truth/fact、implicit closure/
order、theorem acceptance、proof/Core/ControlFlow/VC、broader source shape は open
のままである。Step 5 は active、Steps 6/7 は deferred のまま。module layout
更新は不要であった。

Task 190 MC-G020 current-state override: exact builtin-object same-binding inequality
は `test_gap`、narrow `source_drift`、`design_drift` であり、`spec_gap` ではない。
Chapters 3、4、13、14.5.2、16 は `reserve x for object; theorem
ReservedObjectVariableInequalityPayloadBoundary: x <> x;` を直接 support する。
Tasks 48/125/188 は real builtin-object reserve handoff と canonical
normalization producer を提供し、Task 121 は real exact same-binding pre-
desugaring inequality consumer を提供する。Task 128 も builtin-object
normalization consumer を独立に実証する。route は ordinal 1/2 を
`BindingId(0)` へ解決し、written `object` range 1 個上の distinct result/
expected role site 4 個を保持し、canonical builtin-object identity 1 個へ
intern し、`Inferred` variable 2 個、known type entry 6 個、ordered expected
constraint 2 個、fact/candidate/diagnostic/deferred-free `Checked` inequality
1 個を記録しなければならない。exact/near-miss、matched-output、canonical-
source、`BuiltinSet` corruption、route-order、real frontend/resolver-sidecar
guard を備える。shared trace backlink 5 個 + dedicated row 1 個により、
既存 expectation を変更せず active runner 138 を保護する。inequality
desugaring/equality truth、object/set coercion、fact、implicit closure/order、
theorem acceptance、proof/Core/ControlFlow/VC、broader source shape は open の
ままである。Step 5 は active、Steps 6/7 は deferred のまま。checker source
または module layout 更新は不要であった。

Task 191 MC-G020 current-state override: exact distinct-binding shared-
builtin-object equality は `test_gap`、narrow `source_drift`、
`design_drift` であり、`spec_gap` ではない。Chapters 3、4、13、14.5.2、
16 は `reserve x, y for object; theorem
DistinctReservedObjectVariableEqualityPayloadBoundary: x = y;` を直接
support する。task 123 は real one-item/two-binding shared-written-range
producer を提供し、tasks 48/125/188 は real builtin-object reserve、
normalization、equality consumer を提供する。route は ordinal 2/3 を
`BindingId(0/1)` に解決し、両 binding と distinct result/expected role site
4 個に shared written `object` range 1 個を保持し、その reserve range を
anchor とする canonical builtin-object identity 1 個、`Inferred` variable
2 個、known type entry 6 個、operand-owned ordered expected constraint 2 個、
fact/candidate/diagnostic/deferred-free `Checked` equality 1 個を記録しなければ
ならない。exact/near-miss、matched-output、canonical-source、`BuiltinSet`
corruption、route-order、real frontend/resolver sidecar を備える。shared
trace backlink 5 個 + dedicated row 1 個により既存 expectation を変更せず
active runner 139 を保護する。equality truth、object/set coercion、
fact、implicit closure/order、theorem acceptance、proof/Core/ControlFlow/VC、
broader distinct-object source shape は open のままである。Step 5 は active、
Steps 6/7 は deferred のまま。checker source または module-layout change は
不要であった。

Task 192 MC-G020 current-state override: exact distinct-binding shared-
builtin-object inequality は `test_gap`、narrow `source_drift`、
`design_drift` であり、`spec_gap` ではない。Chapters 3、4、13、14.5.2、
16 は `reserve x, y for object; theorem
DistinctReservedObjectVariableInequalityPayloadBoundary: x <> y;` を直接
support する。tasks 123/191 は real one-item/two-binding shared-written-range
builtin-object producer を提供し、tasks 121/160/190 は real pre-desugaring
inequality consumer を提供する。route は ordinal 2/3 を `BindingId(0/1)` に
解決し、両 binding と distinct result/expected role site 4 個に shared written
`object` range 1 個を保持し、その reserve range を anchor とする canonical
builtin-object identity 1 個、`Inferred` variable 2 個、known type entry 6 個、
operand-owned ordered expected constraint 2 個、fact/candidate/diagnostic/
deferred-free `Checked` inequality 1 個を記録する。shared trace backlink 5 個 +
dedicated row 1 個により既存 expectation を変更せず active runner 140 を
保護する。exact/near-miss、matched-output、canonical-source、`BuiltinSet`
corruption、route-order、real frontend/resolver sidecar が contract を guard する。
inequality desugaring/equality truth、object/set coercion、fact、implicit closure/order、
theorem acceptance、proof/Core/ControlFlow/VC、broader distinct-object source
shape は open のままである。Step 5 は active、Steps 6/7 は deferred のまま。
checker source または module-layout change は不要であった。

Task 193 MC-G020 current-state override: exact multiple-reserve-declaration
builtin-object equality は `test_gap`、narrow `source_drift`、
`design_drift` であり、`spec_gap` ではない。Chapters 3、4、13、14.5.2、
16 は `reserve x for object; reserve y for object; theorem
MultipleObjectReserveDeclarationEqualityPayloadBoundary: x = y;` を直接
support する。Task 124 は real two-item/two-binding/distinct-written-range
producer を提供し、tasks 188/191 は real builtin-object equality consumer を
提供する。route は ordinal 2/3 を `BindingId(0/1)` に解決し、distinct written
`object` range 2 個を distinct result/expected role site 4 個に保持し、先行する
`x` reserve range を anchor とする canonical builtin-object identity 1 個、
`Inferred` variable 2 個、known type entry 6 個、operand-owned ordered expected
constraint 2 個、fact/candidate/diagnostic/deferred-free `Checked` equality 1 個を
記録する。shared trace backlink 5 個 + dedicated row 1 個により既存 expectation
を変更せず active runner 141 を保護する。exact structural/provenance near
miss、matched-output、canonical-source、`BuiltinSet` corruption、route
isolation、real frontend/resolver sidecar が contract を guard する。equality
truth、object/set coercion、fact、implicit closure/order、theorem acceptance、
proof/Core/ControlFlow/VC、shared-range shape、broader multiple-reserve object
shape は open のままである。Step 5 は active、Steps 6/7 は deferred のまま。
checker source または module-layout change は不要であった。

Task 194 MC-G020 current-state override: exact multiple-reserve-declaration
builtin-object inequality は `test_gap`、narrow `source_drift`、
`design_drift` であり、`spec_gap` ではない。Chapters 3、4、13、14.5.2、
16 は `reserve x for object; reserve y for object; theorem
MultipleObjectReserveDeclarationInequalityPayloadBoundary: x <> y;` を直接
support する。Task 193 は real ordered two-item/two-binding/distinct-written-
object-range producer を提供し、tasks 190/192 は real pre-desugaring builtin-
object inequality consumer を提供する。route は ordinal 2/3 を
`BindingId(0/1)` に解決し、binding ごとの written `object` range 2 個を
distinct raw result/expected role 4 個に保持し、先行する `x` range を anchor
とする canonical builtin-object identity 1 個、`Inferred` variable 2 個、known
type entry 6 個、operand-owned ordered expected constraint 2 個、fact/candidate/
diagnostic/deferred-free `Checked` inequality 1 個を記録する。shared backlink 5
個 + dedicated row 1 個により既存 expectation を変更せず active runner 142 を
保護する。exact structural/provenance near miss、raw/canonical-source その他の
corruption probe、route isolation、immutable-output check、real frontend/
resolver sidecar が contract を guard する。inequality desugaring/equality
truth、object/set coercion、fact、implicit closure/order、theorem acceptance、
proof/Core/ControlFlow/VC、shared-range shape、broader multiple-reserve object
shape は open のままである。Step 5 は active、Steps 6/7 は deferred のまま。
checker source または module-layout change は不要であった。

Task 195 MC-G020 current-state override: exact three-edge set-terminal same-
outer-mode asserted head は `test_gap`、narrow `source_drift`、`design_drift`
であり、`spec_gap` ではない。Chapters 3、4、7、13、14.2.3、16 は ordered
mode definition 4 個 `Outer -> Middle -> Inner -> Base -> set`、`reserve x for
OuterThreeEdgeModeAssertedHead`、`ThreeEdgeLocalModeAssertedHeadPayloadBoundary:
x is OuterThreeEdgeModeAssertedHead;` を直接 support する。Task 73 は real
four-expansion producer、Task 150 は同じ深さの subject-side normalization、Task
186 は same-symbol formula-side asserted-head consumer を提供する。exact route
は outer symbol の distinct raw subject/asserted site/range を保持し、ordinal 1
を `BindingId(0)` に解決し、AST-derived expansion 4 個を消費し、known type
entry 3 個を base-definition-RHS anchor の `BuiltinSet` identity 1 個へ
normalize し、expected constraint 0 個、`Inferred` variable 1 個、fact/
candidate/diagnostic/deferred-free normalized-reflexive `Checked` type assertion
1 個を記録する。shared backlink 5 個 + dedicated row 1 個により既存
expectation を変更せず active runner 143 を保護する。structural/provenance
near miss は unrelated local、imported、ambiguous asserted head を含み、
corruption、immutable-output、route-isolation、real frontend/resolver sidecar
guard が contract を完成させる。object-terminal/deeper/imported/attributed/
argument-bearing/other asserted head、reachability/widening/`qua`、declaration/
theorem acceptance、truth/fact、closure/order、broader term/formula/child-graph
semantics、proof/Core/ControlFlow/VC、general chain semantics は open のままで
ある。Step 5 は active、Steps 6/7 は deferred のまま。checker source または
module-layout change は不要であった。

Task 196 MC-G020 current-state override: exact three-edge object-terminal same-
outer-mode asserted head は `test_gap`、narrow `source_drift`、`design_drift`
であり、`spec_gap` ではない。Chapters 3、4、7、13、14.2.3、16 は ordered
mode definition 4 個 `Outer -> Middle -> Inner -> Base -> object`、`reserve x
for OuterThreeEdgeObjectModeAssertedHead`、
`ThreeEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
OuterThreeEdgeObjectModeAssertedHead;` を直接 support する。Tasks 73/151 は real
four-expansion object-terminal producer、Task 187 は same-symbol formula-side
asserted-head consumer、Task 195 は depth-matched set sibling を提供する。exact
route は outer symbol の distinct raw subject/asserted site/range を保持し、
ordinal 1 を `BindingId(0)` に解決し、AST-derived expansion 4 個を消費し、
known type entry 3 個を base-definition-RHS anchor の `BuiltinObject` identity 1
個へ normalize し、expected constraint 0 個、`Inferred` variable 1 個、fact/
candidate/diagnostic/deferred-free normalized-reflexive `Checked` type assertion
1 個を object/set coercion なしで記録する。shared backlink 5 個 + dedicated
row 1 個により既存 expectation を変更せず active runner 144 を保護する。
structural/provenance near miss は unrelated local、imported、ambiguous asserted
head を含み、`BuiltinSet`/canonical-source corruption、immutable-output、route-
isolation、real frontend/resolver sidecar guard が contract を完成させる。
deeper/imported/attributed/argument-bearing/other asserted head、reachability/
widening/`qua`、declaration/theorem acceptance、truth/fact、closure/order、
broader term/formula/child-graph semantics、proof/Core/ControlFlow/VC、general
chain semantics は open のままである。Step 5 は active、Steps 6/7 は deferred
のまま。checker source または module-layout change は不要であった。

Task 197 MC-G020 current-state override: exact four-edge set-terminal same-
outermost-mode asserted head は `test_gap`、narrow `source_drift`、
`design_drift` であり、`spec_gap` ではない。Chapters 3、4、7、13、14.2.3、
16 は ordered mode definition 5 個 `TooDeep -> Outer -> Middle -> Inner ->
Base -> set`、`reserve x for TooDeepFourEdgeModeAssertedHead`、
`FourEdgeLocalModeAssertedHeadPayloadBoundary: x is
TooDeepFourEdgeModeAssertedHead;` を直接 support する。Tasks 74/152 は real
five-expansion set-terminal producer、Tasks 186/195 は same-symbol formula-
side asserted-head consumer を提供する。exact route は outermost symbol の
distinct raw subject/asserted site/range を保持し、ordinal 1 を `BindingId(0)`
に解決し、AST-derived expansion 5 個を消費し、known type entry 3 個を base-
definition-RHS anchor の `BuiltinSet` identity 1 個へ normalize し、expected
constraint 0 個、`Inferred` variable 1 個、fact/candidate/diagnostic/deferred-
free normalized-reflexive `Checked` type assertion 1 個を記録する。shared
backlink 5 個 + dedicated row 1 個により既存 expectation を変更せず 360
cases / 324 requirements 内の active runner 145 を保護する。full reorder、
connected deeper-chain、structural/provenance、unrelated local/imported/
ambiguous asserted-head、mutable corruption、immutable-output、route-
isolation、real frontend/resolver sidecar guard が contract を完成させる。
object-terminal/other-depth/imported/attributed/argument-bearing/other asserted
head、reachability/widening/`qua`、declaration/theorem acceptance、truth/fact、
closure/order、broader term/formula/child-graph semantics、proof/Core/
ControlFlow/VC、general chain semantics は open のままである。Step 5 は
active、Steps 6/7 は deferred のまま。checker source または module-layout
change は不要であった。

Task 198 MC-G020 current-state override: exact four-edge object-terminal same-
outermost-mode asserted head は `test_gap`、narrow `source_drift`、
`design_drift` であり、`spec_gap` ではない。Chapters 3、4、7、13、14.2.3、
16 は ordered mode definition 5 個 `TooDeep -> Outer -> Middle -> Inner ->
Base -> object`、`reserve x for TooDeepFourEdgeObjectModeAssertedHead`、
`FourEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
TooDeepFourEdgeObjectModeAssertedHead;` を直接 support する。Tasks 74/153 は
real five-expansion object-terminal producer、Tasks 187/196 は same-symbol
formula-side asserted-head consumer を提供する。exact route は outermost
symbol の distinct raw subject/asserted site/range を保持し、ordinal 1 を
`BindingId(0)` に解決し、AST-derived expansion 5 個を消費し、known type
entry 3 個を base-definition-RHS anchor の `BuiltinObject` identity 1 個へ
normalize し、expected constraint 0 個、`Inferred` variable 1 個、fact/
candidate/diagnostic/deferred-free normalized-reflexive `Checked` type assertion
1 個を object/set coercion なしで記録する。shared backlink 5 個 + dedicated
row 1 個により既存 expectation を変更せず 361 cases / 325 requirements 内の
active runner 146 を保護する。full reorder、connected deeper-chain、
structural/provenance、unrelated local/imported/ambiguous asserted-head、mutable
corruption、immutable-output、route-isolation、real frontend/resolver sidecar
guard が contract を完成させる。set-terminal/other-depth/imported/attributed/
argument-bearing/other asserted head、reachability/widening/`qua`、declaration/
theorem acceptance、truth/fact、closure/order、broader term/formula/child-graph
semantics、proof/Core/ControlFlow/VC、general chain semantics は open のまま
である。Step 5 は active、Steps 6/7 は deferred のまま。checker source
または module-layout change は不要であった。

Task 199 MC-G020 current-state override: exact seven-expansion set-terminal
same-`ChainMode6` asserted head は `test_gap`、narrow `source_drift`、
`design_drift` であり、`spec_gap` ではない。Chapters 3、4、7、13、14.2.3、16
は `BaseMode -> set`、`ChainMode1 -> BaseMode` から `ChainMode6 -> ChainMode5`
までの ordered local link 6 個、`reserve x for ChainMode6`、
`LongLocalModeAssertedHeadPayloadBoundary: x is ChainMode6;` を直接 support
する。Tasks 74/175 は real seven-expansion set-terminal producer、Tasks 186/
195/197 は same-symbol formula-side asserted-head consumer を提供する。exact
route は `ChainMode6` の distinct raw subject/asserted site/range を保持し、
ordinal 1 を `BindingId(0)` に解決し、AST-derived expansion 7 個を消費し、
known type entry 3 個を `BaseModeDef` RHS anchor の `BuiltinSet` identity 1 個
へ normalize し、expected constraint 0 個、`Inferred` variable 1 個、fact/
candidate/diagnostic/deferred-free normalized-reflexive `Checked` type assertion
1 個を記録する。shared backlink 5 個 + dedicated row 1 個により既存
expectation を変更せず 362 cases / 326 requirements 内の active runner 147 を
保護する。per-link removal/reorder、complete reverse order、connected eighth-
link、structural/provenance、unrelated local/imported/ambiguous asserted-head、
mutable corruption、immutable-output、route-isolation、real frontend/resolver
sidecar guard が contract を完成させる。object-terminal/other-depth/imported/
attributed/argument-bearing/other asserted head、reachability/widening/`qua`、
declaration/theorem acceptance、truth/fact、closure/order、broader term/formula/
child-graph semantics、proof/Core/ControlFlow/VC、general unbounded chain
semantics は open のままである。Step 5 は active、Steps 6/7 は deferred の
まま。checker source または module-layout change は不要であった。

Task 200 MC-G020 current-state override: exact seven-expansion object-terminal
same-`ChainObjectMode6` asserted head は `test_gap`、narrow `source_drift`、
`design_drift` であり、`spec_gap` ではない。Chapters 3、4、7、13、14.2.3、
16 は `BaseObjectMode -> object`、`ChainObjectMode6 -> ChainObjectMode5` まで
の ordered local link 6 個、`reserve x for ChainObjectMode6`、
`LongLocalObjectModeAssertedHeadPayloadBoundary: x is ChainObjectMode6;` を
直接 support する。Tasks 74/179 は real seven-expansion object-terminal
producer、Tasks 187/196/198 は same-symbol formula-side asserted-head consumer
を提供する。exact route は `ChainObjectMode6` の distinct raw subject/
asserted site/range を保持し、ordinal 1 を `BindingId(0)` に解決し、AST-
derived expansion 7 個を消費し、known type entry 3 個を
`BaseObjectModeDef` RHS anchor の `BuiltinObject` identity 1 個へ normalize
し、expected constraint 0 個、`Inferred` variable 1 個、fact/candidate/
diagnostic/deferred-free normalized-reflexive `Checked` type assertion 1 個を
object/set coercion なしで記録する。shared backlink 5 個 + dedicated row 1
個により既存 expectation を変更せず 363 cases / 327 requirements 内の
active runner 148 を保護する。per-link removal/reorder、complete reverse
order、connected eighth-link、structural/provenance、unrelated local/imported/
ambiguous asserted-head、mutable corruption、immutable-output、route-isolation、
real frontend/resolver sidecar guard が contract を完成させる。set-terminal/
other-depth/imported/attributed/argument-bearing/other asserted head、
reachability/widening/`qua`、declaration/theorem acceptance、truth/fact、
closure/order、broader term/formula/child-graph semantics、proof/Core/
ControlFlow/VC、general unbounded chain semantics は open のままである。
Step 5 は active、Steps 6/7 は deferred のまま。checker source または
module-layout change は不要であった。

Task 106 addendum: active `mizar-test` type-elaboration runner は task-87 の generic
boundary のうち `theorem TermFormulaPayloadBoundary: 1 = 1;` を supersede する。
unrecovered builtin equality theorem shape かつ structural Chapter 13 numeral
operand が 2 つだけの場合に限り、runner は real module-shell checker binding
context を作り、source-derived checker `TermInput` と equality `FormulaInput`
payload を `TermFormulaChecker` に渡す。その後 missing numeric type payload と
partial formula checking で fail closed する。numeric type payload extraction、
equality semantic checking、recorded fact、theorem acceptance、dedicated
`formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload coverage は
主張しない。

Task 110 addendum: active `mizar-test` type-elaboration runner は
`theorem ImportedPredicateFunctorPayloadBoundary: 1 divides (1 ++ 2);` という
imported predicate/functor theorem formula source について task 98 を supersede
する。parser / resolver 実行後に `parser.type_fixtures` の `divides` / `++`
imported provenance を検証し、source-derived checker term/formula payload を
渡してから、missing numeric type payload、missing functor signature payload、
missing predicate signature payload、partial formula checking を報告する。これは
Chapter 11、12、13、14、16 の exact imported predicate/functor checker bridge
だけを credit する。imported module AST extraction、semantic predicate/functor
signature、term inference、formula checking、recorded fact、theorem acceptance、
dedicated `formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload
coverage は主張しない。

Task 100 addendum: active `mizar-test` type-elaboration runner は当初
`theorem BuiltinMembershipPayloadBoundary: 1 in 1;` という builtin membership
theorem formula source を parser / resolver 実行後に
`type_elaboration.external_dependency.ast_payload_extraction` として観測した。
task 108 はこの exact source を supersede し、real source-derived checker
`TermInput` / `FormulaInput` payload を渡して
`type_elaboration.checker.checker.term.external.numeric_type_payload` と
`type_elaboration.checker.checker.formula.term.partial` を報告する。これは
Chapter 13、14、16 の narrow source-derived builtin membership term/formula
checker bridge だけを credit する。numeric type payload extraction、membership
operand expected-type construction/checking、recorded fact、theorem acceptance、
dedicated `formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload
coverage は主張しない。

Task 107 addendum: active `mizar-test` type-elaboration runner は task-101
generic boundary の exact builtin inequality theorem formula source
`theorem BuiltinInequalityPayloadBoundary: 1 <> 2;` を supersede する。parser /
resolver 実行後、runner は real source-derived checker `TermInput` /
`FormulaInput` payload を渡し、
`type_elaboration.checker.checker.term.external.numeric_type_payload` と
`type_elaboration.checker.checker.formula.term.partial` を報告する。これは Chapter
13、14、16 の narrow source-derived builtin inequality term/formula checker
bridge だけを credit する。numeric type payload extraction、inequality
desugaring または equality semantic checking、recorded fact、theorem acceptance、
dedicated `formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload
coverage は主張しない。

Task 109 addendum: active `mizar-test` type-elaboration runner は
`theorem BuiltinTypeAssertionPayloadBoundary: 1 is set;` という exact builtin
type-assertion theorem formula source を parser / resolver 実行後に観測し、
source-derived checker `TermInput`、`FormulaInput`、asserted builtin `set`
`TypeExpressionInput` payload を渡してから missing numeric type payload と
partial formula checking で fail closed する。これは Chapter 3、13、14、16 の
exact source-derived builtin type-assertion bridge だけを credit する。より広い
asserted type payload extraction、type-assertion semantic checking、recorded
fact、theorem acceptance、dedicated `formula_statement` runner、CoreIr、
ControlFlowIr、VC、proof payload coverage は主張しない。

Task 113 addendum: active `mizar-test` type-elaboration runner は
`import parser.type_fixtures; theorem ImportedAttributeAssertionPayloadBoundary: 1 is empty;`
という exact imported attribute assertion theorem formula source について task 103 を
supersede する。parser / resolver 実行後に imported `empty` provenance を検証し、
source-derived numeral と attribute-assertion checker payload を渡し、missing
numeric type payload、missing formula / attribute semantic payload、partial formula
checking で fail closed する。これは Chapter 6、11、12、13、14、16 の exact
source-derived imported attribute assertion checker handoff だけを credit する。
imported module AST extraction、theorem formula 向け checker `AttributeInput`
payload extraction、attribute-chain semantic payload extraction、term inference、
attribute admissibility/semantic checking、formula checking、recorded fact、
theorem acceptance、dedicated `formula_statement` runner、CoreIr、ControlFlowIr、
VC、proof payload coverage は主張しない。

Task 114 addendum: active `mizar-test` type-elaboration runner は
`import parser.type_fixtures; theorem ImportedNonEmptyAttributeAssertionPayloadBoundary: 1 is non empty;`
という exact attribute-level `non empty` imported attribute assertion theorem
formula source について task 104 を supersede する。parser / resolver 実行後に
direct `non` surface と imported `empty` provenance を検証し、source-derived
numeral と attribute-assertion checker payload を渡してから missing numeric type
payload、missing formula / attribute semantic payload、partial formula checking
で fail closed する。これは Chapter 6、11、12、13、14、16 の exact
source-derived attribute-level `non empty` imported attribute assertion checker
handoff だけを credit する。imported module AST extraction、theorem formula 向け
checker `AttributeInput` payload extraction、negated attribute-chain semantic
payload extraction、term inference、negated attribute admissibility/semantic
checking、formula checking、recorded fact、theorem acceptance、dedicated
`formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload coverage は
主張しない。

Task 111 addendum: active `mizar-test` type-elaboration runner は task 105 のうち
exact set-enumeration theorem formula source
`theorem SetEnumerationPayloadBoundary: {1, 2} = {1, 2};` だけを supersede する。
parser / resolver 実行後に、4 つの numeral item term、2 つの
set-enumeration term、builtin equality formula の real source-derived checker
payload を渡し、missing numeric type payload、missing set-enumeration
result-type payload、partial formula checking を報告する。これは Chapter
13、14、16 の exact checker handoff だけを credit する。broader
set-enumeration term extraction、term inference、equality/formula checking、
recorded fact、theorem acceptance、dedicated `formula_statement` runner、CoreIr、
ControlFlowIr、VC、proof payload coverage は主張しない。

Task 112 addendum: active `mizar-test` type-elaboration runner は task 99 のうち
exact formula connective / quantifier theorem source
`theorem FormulaConnectiveQuantifierPayloadBoundary: contradiction implies for x being set holds not contradiction;`
だけを supersede する。parser / resolver 実行後に implication、quantified
formula、negation の real source-derived checker `FormulaInput` shell を渡し、
missing formula payload と missing quantifier payload を報告する。これは Chapter
14 と 16 の exact checker shell handoff だけを credit する。task 117 はその
source 内の 2 つの exact source-derived `contradiction` constant だけを
`FormulaKind::Contradiction` payload に進め、同じ missing formula payload
diagnostic に留める。formula constant semantic truth value、child-formula graph
payload、quantifier binder/context payload、formula checking、recorded fact、
theorem acceptance、dedicated `formula_statement` runner、CoreIr、ControlFlowIr、
VC、proof payload coverage は主張しない。

Task 88 addendum: active `mizar-test` type-elaboration runner は
`theorem ProofSkeletonPayloadBoundary: thesis proof thus thesis; end;` という
proof-block theorem source を parser / resolver 実行後に観測し、
`type_elaboration.external_dependency.ast_payload_extraction` を報告する。これは
Chapter 15 conclusion statement と Chapter 16 proof block の source-derived
proof-block / proof-skeleton extraction-gap boundary だけを credit する。checker
proof skeleton payload extraction、local proof context、formula payload extraction、
recorded fact、theorem acceptance、dedicated `formula_statement` runner、CoreIr、
ControlFlowIr、VC、proof payload coverage は主張しない。

Task 90 addendum: active `mizar-test` type-elaboration runner は predicate
definition と functor definition を含む definition block を parser / resolver
実行後に観測し、`type_elaboration.external_dependency.ast_payload_extraction`
を報告する。これは Chapter 9 predicate definitions と Chapter 10 functor
definitions の source-derived predicate/functor definition extraction-gap
boundary だけを credit する。checker definition declaration payload extraction、
definition-local context、definiens formula/term payload extraction、overload
payload、recorded fact、dedicated `formula_statement` runner、CoreIr、
ControlFlowIr、VC、proof payload coverage は主張しない。

Task 91 addendum: active `mizar-test` type-elaboration runner は attribute
definition を parser / resolver 実行後に観測し、
`type_elaboration.external_dependency.ast_payload_extraction` を報告する。
これは Chapter 6 attribute definitions の source-derived attribute
definition extraction-gap boundary だけを credit する。checker attribute
definition declaration payload extraction、definition-local context、
formula-definiens payload extraction、attributed-type evidence、recorded fact、
dedicated `formula_statement` runner、CoreIr、ControlFlowIr、VC、proof
payload coverage は主張しない。

Task 92 addendum: active `mizar-test` type-elaboration runner は mode definition
と structure definition を parser / resolver 実行後に観測し、
`type_elaboration.external_dependency.ast_payload_extraction` を報告する。
これは Chapter 5 と Chapter 7 の source-derived mode/structure definition
extraction-gap boundary だけを credit する。checker mode/structure definition
declaration payload extraction、mode expansion、structure base-shape /
constructor / selector evidence、definition-local context、recorded fact、
dedicated `formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload
coverage は主張しない。

Task 93 addendum: active `mizar-test` type-elaboration runner は theorem proof
内の proof-local `let`、`given`、`consider`、`set`、`reconsider` statement を
parser / resolver 実行後に観測し、
`type_elaboration.external_dependency.ast_payload_extraction` を報告する。
これは Chapter 15 と Chapter 16 の source-derived proof-local declaration
extraction-gap boundary だけを credit する。checker proof-local declaration
payload extraction、local proof context、formula / term payload extraction、
RHS term inference、reconsider coercion / obligation evidence、recorded fact、
theorem acceptance、dedicated `formula_statement` runner、CoreIr、
ControlFlowIr、VC、proof payload coverage は主張しない。

Task 94 addendum: active `mizar-test` type-elaboration runner は theorem proof
内の proof-local `deffunc` と `defpred` inline definition を parser / resolver
実行後に観測し、`type_elaboration.external_dependency.ast_payload_extraction`
を報告する。これは Chapter 15 の source-derived proof-local inline definition
extraction-gap boundary だけを credit する。checker inline definition
formal/body payload extraction、local abbreviation expansion、term / formula
body payload extraction、guard evidence、recorded fact、theorem acceptance、dedicated
`formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload coverage は
主張しない。

Task 95 addendum: active `mizar-test` type-elaboration runner は existential
cluster と conditional cluster を含む top-level registration block を parser /
resolver 実行後に観測し、`type_elaboration.external_dependency.ast_payload_extraction`
を報告する。これは Chapter 17 の source-derived registration-block
extraction-gap boundary だけを credit する。checker registration-item payload
extraction、correctness-condition / proof-obligation payload、accepted
activation / evidence status、cluster / reduction semantics、recorded fact、
Chapter 17 semantic cluster / reduction row、dedicated `formula_statement` または
`advanced_semantics` runner、CoreIr、ControlFlowIr、VC、proof payload coverage は
主張しない。

Task 96 addendum: active `mizar-test` type-elaboration runner は top-level と
definition-local の synonym / antonym alias、および attribute、predicate、
functor redefinition declaration を parser / resolver 実行後に観測し、
`type_elaboration.external_dependency.ast_payload_extraction` を報告する。これは
Chapter 11 / 19 の source-derived redefinition / notation extraction-gap
boundary だけを credit する。checker redefinition payload extraction、notation
alias relation payload、redefinition target inference、coherence
proof-obligation payload、overload candidate payload、recorded fact、Chapter 11
alias semantic resolution、Chapter 19 overload / redefinition semantics、
dedicated `formula_statement` または `advanced_semantics` runner、CoreIr、
ControlFlowIr、VC、proof payload coverage は主張しない。

### `registration_resolution`

生成 public newtype:

- `CheckerRegistrationId`, `RejectedRegistrationId`,
  `RegistrationDiagnosticId`, `ExistentialGateId`
- `RegistrationTriggerKey`, `RegistrationLabelKey`,
  `RegistrationPatternKey`, `RegistrationParameterKey`,
  `AcceptedCorrectnessKey`, `ActivationEvidenceKey`,
  `RegistrationFingerprint`, `RegistrationTypeKey`,
  `RegistrationAttributeKey`, `RegistrationFunctorKey`,
  `RegistrationTermKey`, `RegistrationVariableKey`,
  `ExistentialGateGuardKey`

literal top-level public item:

- `RegistrationDatabase`, `PendingRegistration`, `PendingRegistrationTable`,
  `RegistrationPatternStatus`, `PendingRegistrationStatus`,
  `ActivatedRegistration`, `ActivatedRegistrationIndex`,
  `RejectedRegistration`, `RejectedRegistrationTable`,
  `RejectedRegistrationReason`, `RegistrationSource`, `ResolverTargetShell`,
  `RegistrationValidationKind`, `RegistrationValidationInput`,
  `RegistrationValidationPattern`, `RegistrationTermPattern`,
  `RegistrationVariableOccurrence`, `RegistrationValidationParameter`,
  `RegistrationReferencedSymbolRole`, `RegistrationReferencedSymbol`,
  `ActivationInput`, `ActivationVerifierStatus`, `ExistentialGateInput`,
  `ExistentialGateCandidate`, `ExistentialGateBaseEvidence`,
  `ExistentialGateBaseEvidenceKind`, `ExistentialGateBaseEvidenceCoverage`,
  `ExistentialGateGuardEvidence`,
  `ExistentialGateRecovery`, `ExistentialGateOutput`,
  `ExistentialGateResult`, `ExistentialGateStatus`,
  `RegistrationDiagnostic`, `RegistrationDiagnosticDraft`,
  `RegistrationDiagnosticTable`, `RegistrationDiagnosticClass`,
  `RegistrationDiagnosticSeverity`, `RegistrationDiagnosticRecovery`

対応:

| 仕様上の約束 | source 根拠 | test 根拠 | 状態 |
|---|---|---|---|
| Pending/activated registration database は resolver origin を保持し、不完全 payload を activate しない。 | `RegistrationDatabase`, pending/activated/rejected tables and source records. | pending/activation/source/invalid activation tests. | 実装済み。MC-G021 は残る。 |
| Validation は obligation を emit し、kind-specific payload を検証し、accepted verifier/artifact status で activation を gate する。 | `RegistrationValidationInput`, validation pattern/parameter/reference types, `ActivationInput`. | validation/invalid/routing/reduction-size/accepted-unaccepted activation tests. | explicit payload について実装済み。MC-G025 は残る。 |
| Existential gate は accepted activation、visible guard、exact pattern/base-evidence match、base-shape coverage、deterministic recovery を要求する。 | `ExistentialGateInput`, candidates, base evidence, guard evidence, output/result/status types. | missing/inactive/pending/unaccepted/accepted/rejected/degraded existential tests; task 47 base-object/set、accepted-mode、structure-field、schema-parameter evidence tests。 | explicit payload について実装済み。MC-G026 は残る。 |
| Diagnostic と deterministic rendering は stable。 | `RegistrationDiagnosticTable` and diagnostic classes/recovery. | debug rendering and validation diagnostic tests. | 実装済み。public diagnostic code は MC-G005。 |
| public enum は forward-compatible。 | public enum の `#[non_exhaustive]`。 | `checker_public_enums_are_forward_compatible_and_documented`. | task 31 で guard 済み。 |

### `cluster_trace`

生成 public newtype:

- `ClusterFactId`, `ClusterStepId`, `ReductionStepId`,
  `ClusterDiagnosticId`
- `ClusterFactFingerprint`, `ClusterTypeFingerprint`,
  `ClusterAttributeFingerprint`, `ClusterRuleFingerprint`,
  `ClusterAuditKey`, `ClusterOrderingVersion`, `ClusterTraversalCacheKey`,
  `ReductionFingerprint`, `ReductionRuleFqn`, `ReductionTermFingerprint`,
  `ReductionRedexPath`, `ReductionBindingKey`, `ReductionGuardKey`,
  `ReductionGuardEvidenceKey`, `ReductionRuleViewFingerprint`,
  `ReductionSelectionKey`, `ReductionStrategyAuditKey`

literal top-level public item:

- `ClusterClosureOutput`, `ClusterClosureStatus`, `ReductionTraceOutput`,
  `ClusterTraceBuilder`, `ReductionTraceBuilder`, `ClusterTraversalConfig`,
  `ResolutionTrace`, `ResolutionTraceStep`, `ClusterStep`,
  `ReductionStep`, `ClusterAntecedentRef`, `ClusterTraversalProfile`,
  `ClusterReplayReport`, `ClusterReplayStatus`, `ClusterFactInput`,
  `ClusterRuleInput`, `ClusterRuleDraft`, `ReductionInput`,
  `ReductionDraft`, `ReductionBinding`, `ReductionGuardKind`,
  `ReductionGuardRequirement`, `ReductionGuardEvidenceRef`,
  `ClusterRuleKind`, `ClusterFact`, `ClusterFactDraft`,
  `ClusterFactTable`, `ClusterFactProvenance`, `ClusterDiagnostic`,
  `ClusterDiagnosticDraft`, `ClusterDiagnosticTable`,
  `ClusterDiagnosticClass`, `ClusterDiagnosticSeverity`,
  `ClusterDiagnosticRecovery`

対応:

| 仕様上の約束 | source 根拠 | test 根拠 | 状態 |
|---|---|---|---|
| Cluster closure は replayable deterministic cluster step と derived fact を記録する。 | `ClusterTraceBuilder`, `ClusterClosureOutput`, `ResolutionTrace`, `ClusterStep`, `ClusterFactTable`. | closure/inactive/order/conditional/subtype/transitive/mismatch/rejected/duplicate tests. | explicit payload について実装済み。MC-G023 は残る。 |
| Saturation bound、loop、explicit contradiction は silent truncation でなく visible failure。 | `ClusterTraversalConfig`, `ClusterTraversalProfile`, `ClusterClosureStatus`, diagnostics. | loop/bound/zero-antecedent/contradiction tests; task 46 class/severity/recovery assertion。 | explicit payload について実装済み。 |
| Replay は active registration fingerprint を再検証する。 | `ResolutionTrace::replay`, `ClusterReplayReport`, `ClusterReplayStatus`. | `replay_revalidates_active_registration_fingerprint`, `active_pattern_fallback_must_match_rule_fingerprint`. | 実装済み。 |
| Reduction step は architecture 17 provenance、guard evidence、strategy audit key を保持する。 | `ReductionTraceBuilder`, `ReductionTraceOutput`, `ReductionStep`, reduction input/guard types. | reduction provenance/inactive/rejected/invalid/`such` guard tests; task 46 discharged-side-condition trace identity determinism coverage。 | explicit payload について実装済み。source-derived rewrite extraction と normalization-result dependence は MC-G023。 |
| public enum は forward-compatible。 | public enum の `#[non_exhaustive]`。 | `checker_public_enums_are_forward_compatible_and_documented`. | task 31 で guard 済み。 |

### `overload_resolution`

生成 public newtype:

- `OverloadSiteId`, `OverloadCandidateId`, `OverloadDiagnosticId`,
  `TemplateExpansionId`, `CandidateViabilityId`, `SpecificityGraphId`,
  `SpecificityComparisonId`, `SpecificityEdgeId`, `OverloadResultId`,
  `InsertedViewId`
- `OverloadSiteKey`, `OverloadNameKey`,
  `OverloadDiagnosticMessageKey`, `CandidateProvenanceKey`,
  `TemplateInstantiationKey`, `TemplateParameterKey`, `QuaPathKey`,
  `ViabilityEvidenceKey`, `SpecificityReasonKey`, `SelectionReasonKey`,
  `InsertedViewReasonKey`

literal top-level public item:

- `OverloadCollectionOutput`, `TemplateExpansionOutput`,
  `CandidateViabilityOutput`, `SpecificityGraphOutput`,
  `OverloadSelectionOutput`, `OverloadSiteInput`, `SourceQuaView`,
  `OverloadSiteKind`, `UnsupportedOverloadRole`, `OverloadSiteRecovery`,
  `OverloadCandidateInput`, `CandidateDeclarationKind`, `CandidateOrigin`,
  `CoherenceStatus`, `TemplateCandidatePayload`, `TemplateArgument`,
  `TemplateQuaStatus`, `TemplateArgumentInference`,
  `TemplateConstraintEvidence`, `TemplateConstraintEvidenceStatus`,
  `CandidateProvenance`, `CandidateScope`, `OverloadSite`,
  `OverloadSiteStatus`, `OverloadSiteTable`, `OverloadCandidate`,
  `OverloadCandidateStatus`, `OverloadCandidateTable`,
  `TemplateExpansion`, `TemplateExpansionTable`, `TemplateSubstitution`,
  `TemplateSubstitutionSource`, `TemplateExpansionStatus`,
  `TemplateExpansionFailure`, `CandidateViabilityInput`,
  `ArgumentViabilityEvidence`, `ViabilityFactStatus`,
  `ViabilityCoercionKind`, `ViabilityCoercionStatus`,
  `CandidateViability`, `CandidateViabilityTable`,
  `CandidateViabilityStatus`, `ArgumentViewPlan`, `ArgumentViewKind`,
  `CandidateRejection`, `CandidateRejectionReason`,
  `CandidateBlockedReason`, `CandidateBlockedReasonKind`,
  `SpecificityComparisonInput`, `SpecificityComparisonStatus`,
  `SpecificityBlockedReasonKind`, `SpecificityGraph`,
  `SpecificityGraphTable`, `SpecificityNode`, `SpecificityComparison`,
  `SpecificityComparisonOutcome`, `SpecificityEdge`,
  `SpecificityFailureReason`, `OverloadSiteResolutionInput`,
  `RefinementJoinPayload`, `RefinementJoinStatus`,
  `RefinementJoinFailure`, `ExposedResultPayload`,
  `ExposedResultSource`, `InsertedViewInput`, `InsertedViewKind`,
  `InsertedViewStatus`, `OverloadResult`, `OverloadResultTable`,
  `OverloadResultStatus`, `OverloadBlockedReason`, `InsertedView`,
  `InsertedViewTable`, `OverloadDiagnostic`, `OverloadDiagnosticDraft`,
  `OverloadDiagnosticProvenance`, `OverloadDiagnosticTable`,
  `OverloadDiagnosticClass`, `OverloadDiagnosticSeverity`,
  `OverloadDiagnosticRecovery`

対応:

| 仕様上の約束 | source 根拠 | test 根拠 | 状態 |
|---|---|---|---|
| Site/candidate collection は resolver-filtered input、source `qua`、provenance、supported role、stable ordering を保持する。 | `OverloadCollectionOutput::collect`, site/candidate input and table types. | collection/filter/order/provenance/template/source-`qua`/unsupported/duplicate tests. | explicit payload について実装済み。MC-G027 は残る。 |
| Template expansion は explicit template/inference/constraint payload を使い、rejected/deferred case を保持する。 | `TemplateExpansionOutput::expand`, template payload/substitution/constraint types. | expansion/omitted/constraint/source-`qua`/diagnostic/deferred tests. | explicit payload について実装済み。MC-G006/MC-G027 は残る。 |
| Viability は recorded evidence を消費し、新規 fact derivation、registration firing、root selection をしない。 | `CandidateViabilityOutput::filter`, viability evidence/status/rejection/block/view-plan types. | exact/source-`qua`/non-consumable/narrowing/missing/deferred/remap tests. | 実装済み。 |
| Specificity は explicit comparison から per-site graph を作り、ordinary root ordering に return type を使わない。 | `SpecificityGraphOutput::build`, graph/node/comparison/edge types. | graph/equivalence/return-type/empty/blocked/missing tests; task 45 encoded non-template/template priority comparison。 | 実装済み。 |
| Selection は unique maximal ordinary root を選択し、accepted refinement を join し、widening/source-`qua` view を記録し、failure を保持する。 | `OverloadSelectionOutput::resolve`, result/view/refinement/exposed-result types. | selection/no-match/ambiguity/missing/redefinition/refinement/invalid/deterministic tests; task 45 equivalent-template ambiguity、encoded priority、unencoded tie、redefinition-metadata tie tests。 | explicit payload について実装済み。`coherence with` 省略 target inference は MC-G027/MC-G030 の下で upstream producer obligation のままであり、rejected omitted-target declaration は active candidate としてこの API に到達してはならない。 |
| public enum は forward-compatible。 | public enum の `#[non_exhaustive]`。 | `checker_public_enums_are_forward_compatible_and_documented`. | task 31 で guard 済み。 |

### `resolved_typed_ast`

生成 public newtype:

- `ResolvedTypedNodeId`, `ExpressionMetadataId`, `OverloadResolutionId`,
  `CoercionInsertionId`, `ResolvedTypedDiagnosticId`, `StatementSemanticId`,
  `StatementProofIntentId`, `CheckedProofId`, `CheckedProofNodeId`,
  `CheckedTerminalGoalId`
- `ExprId`, `SourceNodeRole`

literal top-level public item:

- `ResolvedTypedAst`, `ResolvedTypedAstInputs`, `ExpressionMetadataInput`,
  `ResolvedNodeKindHint`, `ResolvedNodeKindHintKind`, `ResolvedTypedArena`,
  `ResolvedTypedNode`, `ResolvedTypedNodeKind`, `ResolvedNodeRecovery`,
  `ResolvedNodeRecoveryReason`, `ExpressionMetadata`,
  `ExpressionMetadataTable`, `OverloadCandidateSummary`,
  `OverloadCandidateSummaryTable`, `TemplateExpansionSummary`,
  `TemplateExpansionSummaryTable`, `CandidateViabilitySummary`,
  `CandidateViabilitySummaryTable`, `ResolvedSpecificityComparison`,
  `ResolvedSpecificityGraph`, `ResolvedSpecificityGraphTable`,
  `OverloadResolutionRecord`, `OverloadResolutionStatus`,
  `OverloadResolutionTable`, `CoercionInsertion`,
  `CoercionInsertionSource`, `CoercionInsertionTable`,
  `ResolvedTypedDiagnostic`, `ResolvedTypedDiagnosticSource`,
  `ResolvedTypedDiagnosticSeverity`, `ResolvedTypedDiagnosticTable`,
  `CandidateSummaryNamespace`, `StatementSemanticInputs`,
  `StatementSemanticInput`, `StatementSemantic`, `StatementSemanticTable`,
  `StatementProofInputs`, `StatementProofIntentInput`,
  `TheoremPolicyIntent`, `TheoremJustificationIntent`, `CheckedProofStatus`,
  `CheckedProofNodeKind`, `CheckedCitation`, `CheckedProofLabel`,
  `CheckedProof`, `CheckedProofTable`, `CheckedProofNode`,
  `CheckedProofNodeTable`, `CheckedTerminalGoal`, `CheckedTerminalGoalTable`,
  `ResolvedTypedAstError`

対応:

| 仕様上の約束 | source 根拠 | test 根拠 | 状態 |
|---|---|---|---|
| Final source-shaped projection は typed AST node、expression metadata、overload summary、cluster fact、diagnostic を保持する。 | `ResolvedTypedAst::assemble`, `ResolvedTypedAstInputs`, arena/metadata/summary/table types. | assembly/template/candidate/diagnostic remap tests. | explicit predecessor output について実装済み。source extraction/artifact は MC-G027。 |
| Failed overload site と failed node は success に書き換えられず可視のまま残る。 | `OverloadResolutionStatus`, recovery/reason enum, result and diagnostic tables. | failed-site/failed-selection/validation rejection tests. | 実装済み。 |
| exact Task-180 omitted-justification intentはauthenticated pending proof/direct terminal goalをatomicに生成する。 | `StatementProofInputs`、`CheckedProofTable`、`CheckedProofNodeTable`、`CheckedTerminalGoalTable`、private postvalidation。 | exact statement/proof projection、owner visibility、corruption、deterministic nonempty、captured empty-rendering test。 | Task-180 singletonだけ実装。broader proofはTask 247、Core loweringはCore Task 31。 |
| Inserted coercion は source と widening/source-`qua` evidence だけを記録する。 | `CoercionInsertion`, `CoercionInsertionSource`, `CoercionInsertionTable`. | assembly and validation tests, upstream invalid-view tests. | explicit input について実装済み。 |
| deterministic debug projection は equivalent input order を canonicalize する。 | deterministic table iteration and `debug_text`. | `deterministic_debug_text_canonicalizes_equivalent_input_orderings`, task-30 determinism suite. | 実装済み。 |
| public enum は forward-compatible。 | public enum の `#[non_exhaustive]`。 | `checker_public_enums_are_forward_compatible_and_documented`. | task 31 で guard 済み。 |

## Cross-Cutting Test And Policy Evidence

| 根拠 | coverage |
|---|---|
| `crates/mizar-checker/tests/lint_policy.rs` | workspace lint opt-in、dependency boundary、no direct syntax import、documented public module、explicit overload/resolved-AST boundary guard、public enum forward-compatibility policy、source/spec audit の public-surface と MC-G reconciliation guard、bilingual documentation sync の pair-inventory / companion-link guard、module-boundary source-layout guard、documented `allow` exception。 |
| `crates/mizar-checker/src/determinism_suite.rs` | type normalization、fact query、cluster closure、reduction trace discharged-side-condition identity、overload pipeline、final `ResolvedTypedAst` projection の cross-module deterministic rerun と equivalent-order permutation。 |
| 各 source module の unit tests | 実装済み checker seam に対する task-local behavior tests。source-to-checker extraction と後続 semantic corpus runner が存在するまで、これが active executable coverage。 |
| `tests/coverage/spec_trace.toml` deferred rows | formula/statement、cluster/reduction、overload/refinement、review-audit semantic corpus obligation を active fixture と偽らずに記録する。 |

## Gap Reconciliation

task 32 は implementation と test evidence なしに gap を閉じない。crate-plan の
MC-G row はすべて次のように照合する。

| ID | Classification | task 32 audit disposition |
|---|---|---|
| MC-G002 | `test_gap` | real semantic `.miz` coverage について active。source-to-checker extraction が入るまで Rust tests と deferred traceability で mitigated。 |
| MC-G003 | `design_drift` plus deferred external gate | 歴史的 wording drift は修復済み。accepted proof/artifact status は MC-G025 で表現する。 |
| MC-G004 | `source_drift` / `external_dependency_gap` planning gate | 現在 code における checker-source drift ではない。artifact producer/reuse integration は task-scoped cross-crate dependency のまま。checker は schema を invent しない。 |
| MC-G005 | `spec_gap` / `external_dependency_gap` | active。public checker diagnostic code-space は未割当。module は crate-local diagnostic class と stable message/detail key を使う。 |
| MC-G006 | `source_drift` / `external_dependency_gap` | parser/syntax template/scheme role について active。overload code は unsupported role を fabricate せず defer する。 |
| MC-G007 | `design_drift` | checker crate plan と module spec で解決済み。task 32 で architecture-file rename は不要。 |
| MC-G009 | `repo_metadata_conflict` | report-only sentinel。task 32 で metadata conflict は観測していない。 |
| MC-G011 | `external_dependency_gap` | AST-wide local binding extraction、use-site scope/ordinal payload、reserve payload、closure payload、syntax-free `ResolvedAst` fixture について active。 |
| MC-G014 | `external_dependency_gap` | AST-wide type-expression payload、mode/radix/attribute expansion payload、arity payload について active。 |
| MC-G016 | `external_dependency_gap` | declaration/type-site table、reserve default、RHS/body payload、evidence query について active。 |
| MC-G017 | `external_dependency_gap` | term/formula payload table、built-in numeric payload、candidate signature、structure/selector payload、source `qua` evidence、sethood/non-emptiness evidence について active。 |
| MC-G018 | `external_dependency_gap` | coercion request table、dependency-summary fact、inheritance graph、cluster evidence、sethood/non-emptiness evidence、proof-query result について active。 |
| MC-G019 | `external_dependency_gap` | statement/proof assumption、theorem acceptance payload、phase-7 trace fact payload について active。 |
| MC-G020 | `external_dependency_gap` / `deferred` | task 7-11 と後続 consumer の semantic pass fixture を妨げる source-to-checker extraction blocker として active。task 16 から task 81 までは reserve-only source-derived checker bridge を段階的に構築し、builtin reserve / type-expression payload は `TypedAst` と `ResolvedTypedAst` まで到達し、builtin `set` / `object` に終端する supported same-module bare local-mode expansion family は pass できる一方、local structure / attribute / argument / bracket / forward-reference / historical task-80 imported-attribute boundary は fabricated evidence なしの active diagnostic に留める。task 82 は task-79 imported mode source を一段進め、documented `parser.type_fixtures` 由来の imported `SymbolKind::Mode` を checker type-head payload として渡し、checker が `ImportedSource` provenance を検証してから missing imported `ModeExpansion` payload で fail closed する。task 83 は documented `parser.type_fixtures` imported structure `R` source を一段進め、checker が `ImportedSource` provenance を検証してから missing base-shape / constructor-witness evidence で fail closed する。task 97 は documented `TypeCaseStruct` source を同じ real checker type-head boundary と missing evidence query に進める。task 84 は documented `parser.type_fixtures` imported attribute `TypeCaseAttr` source を一段進め、checker が `ImportedSource` provenance を検証してから missing attributed-type existential/evidence payload で fail closed する。task 85 は既存 `non empty set` source を一段進め、builtin `set` 上の real imported negative `empty` checker `AttributeInput` payload として渡し、missing attributed-type existential/evidence payload で fail closed する。task 116 は既存 `empty set` source を一段進め、builtin `set` 上の real imported positive `empty` checker `AttributeInput` payload として渡し、同じ missing attributed-type existential/evidence payload で fail closed する一方、`non empty object` runner sidecar は extraction-gap boundary に残す。task 86 は parser / resolver 実行後の formula-only theorem source を active boundary として記録し、task 117 は task 115 を exact `FormulaPayloadBoundary: thesis` source について supersede し、source-derived `thesis` formula constant を real `FormulaKind::Thesis` checker payload として渡して missing formula payload で fail closed するが、formula constant checking、theorem acceptance、recorded fact、proof context、`formula_statement` runner は主張しない。task 106 は exact builtin equality theorem source について real source-derived checker `TermInput` と equality `FormulaInput` payload を `TermFormulaChecker` に渡し、missing numeric type payload と partial formula checking で fail closed するが、equality semantic checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 110 は task 98 の exact imported predicate/functor theorem source を supersede し、imported `divides`/`++` provenance を検証して real source-derived numeral、imported functor-application、predicate-application checker payload を `TermFormulaChecker` に渡し、missing numeric/signature payload と partial formula checking で fail closed するが、semantic predicate/functor signature、term inference、formula checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 108 は exact builtin membership theorem source について real source-derived checker `TermInput` と membership `FormulaInput` payload を `TermFormulaChecker` に渡し、missing numeric type payload と partial formula checking で fail closed するが、membership operand expected-type construction/checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 107 は exact builtin inequality theorem source について real source-derived checker `TermInput` と inequality `FormulaInput` payload を `TermFormulaChecker` に渡し、missing numeric type payload と partial formula checking で fail closed するが、inequality desugaring または equality semantic checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 109 は exact builtin type-assertion theorem source について real source-derived checker `TermInput`、type-assertion `FormulaInput`、asserted builtin `set` `TypeExpressionInput` を `TermFormulaChecker` に渡し、missing numeric type payload と partial formula checking で fail closed するが、broader asserted type payload、type-assertion semantic checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 113 は exact imported attribute assertion theorem formula について task 103 を supersede し、imported `empty` provenance を検証して source-derived checker term/formula payload を渡し、missing numeric type payload、missing formula / attribute semantic payload、partial formula checking で fail closed するが、imported module AST extraction、theorem formula 向け checker `AttributeInput` payload extraction、attribute-chain semantic payload extraction、term inference、attribute admissibility/semantic checking、formula checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 114 は exact attribute-level non-empty imported attribute assertion theorem formula について task 104 を supersede し、direct `non` surface と imported `empty` provenance を検証して source-derived checker term/formula payload を渡し、missing numeric type payload、missing formula / attribute semantic payload、partial formula checking で fail closed するが、imported module AST extraction、theorem formula 向け checker `AttributeInput` payload extraction、negated attribute-chain semantic payload、term inference、negated attribute admissibility/semantic checking、formula checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 111 は task 105 の exact set-enumeration theorem formula source だけを supersede し、source-derived numeral item term、set-enumeration term、builtin equality formula checker payload を `TermFormulaChecker` に渡して missing numeric/result-type payload と partial formula checking で fail closed するが、broader set-enumeration result-type payload extraction、term inference、equality/formula checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 112 は task 99 の exact formula connective/quantifier theorem formula surface だけを supersede し、parser / resolver 実行後に real checker formula shell payload を渡して missing formula/quantifier payload で fail closed し、task 117 は同じ exact source の 2 つの `contradiction` constants を real `FormulaKind::Contradiction` payload に進めるが、formula constant semantic truth value、child-formula graph payload、quantifier binder/context payload、formula checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 180 は別に exact standalone `SourceDerivedContradictionConstantBoundary: contradiction` leaf を type/well-formedness だけの 1 `Checked` `FormulaKind::Contradiction` として check し、truth/fact publication または theorem/proof/downstream credit を主張しない。task 88 は parser / resolver 実行後の proof-block theorem source を同じ active extraction-gap boundary として記録し、proof skeleton payload、local proof context、formula payload、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 89 は parser / resolver 実行後の statement-level proof-justification theorem source を同じ active extraction-gap boundary として記録し、statement proof payload、nested proof skeleton payload、local proof context、formula payload、label-reference semantic checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 90 は parser / resolver 実行後の predicate/functor definition source を active extraction-gap boundary として記録し、definition declaration payload、definition-local context、definiens formula/term payload、overload payload、recorded fact、`formula_statement` runner は主張しない。task 91 は parser / resolver 実行後の attribute definition source を active extraction-gap boundary として記録し、attribute definition declaration payload、definition-local context、formula-definiens payload、attributed-type evidence、recorded fact、`formula_statement` runner は主張しない。task 92 は parser / resolver 実行後の mode/structure definition source を同じ active extraction-gap boundary として記録し、mode/structure definition declaration payload、mode expansion、structure base-shape / constructor / selector evidence、definition-local context、recorded fact、`formula_statement` runner は主張しない。task 93 は parser / resolver 実行後の proof-local declaration statement source を同じ active extraction-gap boundary として記録し、proof-local declaration payload、local proof context、formula / term payload、RHS term inference、reconsider coercion / obligation evidence、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 94 は parser / resolver 実行後の proof-local inline definition source を同じ active extraction-gap boundary として記録し、inline definition formal/body payload、local abbreviation expansion、term / formula body payload、guard evidence、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 95 は parser / resolver 実行後の registration block source を同じ active extraction-gap boundary として記録し、registration-item payload、correctness-condition / proof-obligation payload、accepted activation / evidence status、cluster / reduction semantics、recorded fact、Chapter 17 semantic row、`formula_statement` / `advanced_semantics` runner は主張しない。task 96 は parser / resolver 実行後の redefinition/notation source を同じ active extraction-gap boundary として記録し、redefinition payload、notation alias relation payload、target inference、coherence proof-obligation payload、overload candidate payload、Chapter 11 alias semantic resolution、Chapter 19 overload/redefinition semantics、`formula_statement` / `advanced_semantics` runner は主張しない。より広い non-builtin declaration（task 96 の redefinition/notation extraction-gap boundary を超えるもの）、task 84 `TypeCaseAttr` provenance / `AttributeInput` bridge、task 85 negative `empty`/builtin-`set` bridge、task 116 positive `empty`/builtin-`set` bridge を超える imported attribute、task 83 `R` と task 97 `TypeCaseStruct` provenance/type-head bridge を超える imported structure、task 82 provenance/type-head bridge を超える imported mode expansion、attribute argument、qualified attribute qualifier / owner provenance、mode / structure argument、bracket `type_arg_list` と `qua`-argument provenance、term-argument provenance、structure base-shape / full attributed-type existential evidence、broader / attributed / argument-bearing / parameterized / contextual / ambiguous / cyclic mode expansion、task-106/task-107/task-108/task-109/task-110/task-111/task-112/task-113/task-114/task-117/task-180 exact leaf を超える numeric/signature/result-type payload と equality/inequality/membership/type-assertion/imported predicate-functor/set-enumeration semantic checking および task-112/task-117 を超える formula child/binder semantics、task-110/task-111/task-112/task-113/task-114/task-117 checker bridge、task-180 exact leaf、task-105/task-88/task-89 extraction-gap boundary を超える term / formula / proof skeleton、task-93 extraction-gap boundary を超える proof-local declaration payload、task-94 extraction-gap boundary を超える inline definition payload、task-95 extraction-gap boundary を超える registration payload / correctness-condition / activation payload、task-96 extraction-gap boundary を超える redefinition/notation payload、coercion、overload、recorded fact、CoreIr、ControlFlowIr、VC、proof payload extraction は未解決のまま。 |
| MC-G021 | `external_dependency_gap` / `deferred` | registration payload、accepted-status、source extraction blocker として active。registration code は explicit payload seam のみ消費する。 |
| MC-G023 | `test_gap` / `external_dependency_gap` / `deferred` | source-derived cluster/reduction fixture、artifact/cache integration、source-derived normalization-result dependence、real trace extraction について active。task 46 は explicit-payload fatal contradiction と reduction trace-identity seam だけを cover する。 |
| MC-G025 | `external_dependency_gap` / `deferred` | accepted registration status の proof/artifact production または import について active。 |
| MC-G026 | `test_gap` / `external_dependency_gap` / `deferred` | source-derived existential gate case、artifact reuse、accepted-status integration について active。 |
| MC-G027 | `test_gap` / `external_dependency_gap` / `deferred` | source-derived overload payload、`coherence with` 省略 target diagnostic production、diagnostic code allocation、artifact emission/reuse、semantic fixture について active。task 45 は explicit-payload Rust regression だけを追加し、source-derived seed は inactive のままにする。 |
| MC-G030 | `test_gap` / `external_dependency_gap` / `deferred` | `formula_statement` と `advanced_semantics` runner/tag support、および source payload extraction について active。 |

Resolved setup-history row は closed のまま: MC-G001、MC-G010、MC-G012、
MC-G013、MC-G015、MC-G022、MC-G024、MC-G028、MC-G029 は task commit を持ち、
この audit で再オープンされた source/spec mismatch はない。

Task 171 current-state override: gap row の、`non empty object` sidecar が
extraction gap に残るという旧記述は supersede される。exact negative imported
`empty`/builtin-`object` source は checker evidence-query diagnostic に到達する。
positive `empty object`、symbol head 上の imported attribute、imported module AST
extraction、attribute evidence/acceptance、downstream payload は deferred のまま。

Task 181 current-state override: imported attributed-reserve routing は credit
済み task-84/85/116/171 source shape 5 件だけを許す exact guard を持つ。
corpus、expectation、traceability、runner count、semantic-coverage row は変更せず、
broader imported attributed-reserve shape は記録済み extraction gap のままである。

## Task 32 Classification

| Class | Evidence | Action |
|---|---|---|
| `spec_gap` | この audit は新しい language behavior や checker diagnostic code allocation を導入しない。MC-G005 が public diagnostic-code gap のまま。 | task 32 では public code を追加しない。 |
| `test_gap` | active `.miz` semantic fixture coverage は MC-G002/MC-G023/MC-G026/MC-G027/MC-G030 により deferred のまま。 | Rust task-local coverage と deferred traceability を維持し、pass fixture を fabricate しない。 |
| `design_drift` | blocking drift は観測していない。歴史的 MC-G003/MC-G007 drift は crate plan/module specs で解決済み。 | この audit record 以外の design repair はない。 |
| `source_drift` | 実装済み explicit-payload API は owning module spec と矛盾しない。 | source 変更なし。 |
| `source_undocumented_behavior` | top-level public item は上記 module inventory 外に存在しない。 | future public item は module spec と本 audit を更新するか、該当箇所では lint policy により捕捉される必要がある。 |
| `external_dependency_gap` | source extraction、accepted proof/artifact status、public diagnostic code、artifact emission/reuse、後続 semantic runner は現在の checker input の外。 | deferred row と follow-up task を維持し、stub は作らない。 |
| `deferred` | formula/statement、cluster/reduction、overload/refinement、audit-negative corpus obligation は記録済みだが inactive。 | owning extraction/runner/artifact task が入った後にだけ再訪する。 |

## Completion Decision

task 32 は、この English audit と Japanese companion、crate plan / todo update、
lint-policy audit guard が同じ commit に含まれた時点で完了する。この audit は
単体では crate completion を主張しない。task 33、task 34、closeout task は
bilingual synchronization audit、module-boundary refactor gate、crate exit report を
すでに記録している。


## Task 201 MC-G020 current-state override

Task 201 は exact one-edge set-terminal immediate-radix asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。Chapters 3、4、7、13、14.2.3、16 は local definition 2 個、outer reserve、identifier subject、Base asserted type、theorem を直接 support する。Tasks 56/146 は real expansion 2 個、Task 184 は formula consumer を提供する。closed asserted-head relation は builtin/same-mode route を不変に保ち、resolved outer-to-base immediate edge だけを受理する。

active route は distinct Outer/Base symbol/site/range を保持し、ordinal 1 を `BindingId(0)` に解決し、known entry 3 個を Base-definition-RHS `BuiltinSet` 1 個へ normalize し、expected constraint 0 個、`Inferred` variable 1 個、fact/candidate/diagnostic/deferred-free `Checked` assertion 1 個を記録する。shared backlink 5 個 + dedicated row 1 個が既存 expectation を変更せず、364 cases / 328 requirements 内の active runner 149 を保護する。exact structural/provenance、corruption、immutable-output、Task 146/184 isolation、real sidecar guard は executable である。broader asserted head/semantics、proof/CoreIr/ControlFlowIr/VC、general chain は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままである。checker source または module-layout change は不要であった。


## Task 202 MC-G020 current-state override

Task 202 は exact object-terminal immediate-radix asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。Tasks 56/147 は real object expansion 2 個、Task 185 は object formula consumer、Task 201 は変更しない immediate-radix relation を提供する。active route は distinct Outer/Base provenance を保持し、ordinal 1 を `BindingId(0)` に解決し、known entry 3 個を Base-definition-RHS `BuiltinObject` 1 個へ normalize し、`Inferred` variable 1 個と constraint/fact/candidate/diagnostic/deferred 0 個の `Checked` assertion 1 個を object/set coercion なしで記録する。shared backlink 5 個 + dedicated row 1 個が 365 cases / 329 requirements 内の active runner 150 を保護する。exact/corruption、immutable-output、real Tasks 147/185/201 isolation、sidecar guard は executable である。broader semantics と downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままである。checker source または module-layout change は不要であった。


## Task 203 MC-G020 current-state override

Task 203 は exact two-edge set-terminal immediate-radix asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。Task 72 は real source-derived expansion 3 個、Task 186 は formula consumer、Tasks 201/202 は変更しない immediate-radix relation を提供する。active route は distinct Outer/Middle provenance を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 3 個を消費し、known entry 3 個を Base-definition-RHS `BuiltinSet` 1 個へ normalize し、`Inferred` variable 1 個と constraint/fact/candidate/diagnostic/deferred 0 個の `Checked` assertion 1 個を記録する。shared backlink 5 個 + dedicated row 1 個が既存 expectation を変更せず、366 cases / 330 requirements 内の active runner 151 を保護する。exact/corruption/immutable-output coverage、全 definition-order/duplicate/spelling/imported/ambiguous/deeper near miss、real Tasks 122/148/149/186/187/201/202 isolation、real sidecar は executable である。two-hop Base assertion、object sibling、broader semantics、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままである。checker source または module-layout change は不要であった。


## Task 204 MC-G020 current-state override

Task 204 は exact two-edge object-terminal immediate-radix asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。Task 72 は real source-derived object expansion 3 個、Task 187 は formula consumer、Tasks 202/203 は変更しない immediate-radix relation を提供する。active route は distinct Outer/Middle provenance を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 3 個を消費し、known entry 3 個を Base-definition-RHS `BuiltinObject` 1 個へ normalize し、`Inferred` variable 1 個と constraint/fact/candidate/diagnostic/deferred 0 個の `Checked` assertion 1 個を object/set coercion なしで記録する。shared backlink 5 個 + dedicated row 1 個が既存 expectation を変更せず、367 cases / 331 requirements 内の active runner 152 を保護する。exact/corruption/immutable-output coverage、全 definition order と duplicate/spelling/imported/ambiguous/deeper near miss、real Tasks 189/145/147/149/187/202 および set Tasks 148/186/203 isolation、real sidecar は executable である。two-hop Base assertion、broader semantics、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままである。checker source または module-layout change は不要であった。

## Task 205 MC-G020 current-state override

Task 205 は exact three-edge set-terminal immediate-radix asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。Task 73 は real source-derived set-terminal expansion 4 個、Task 195 は formula consumer、Tasks 201/203/204 は変更しない immediate-radix relation を提供する。active route は distinct Outer/Middle provenance を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 4 個を消費し、known entry 3 個を Base-definition-RHS `BuiltinSet` 1 個へ normalize し、`Inferred` variable 1 個と constraint/fact/candidate/diagnostic/deferred 0 個の `Checked` assertion 1 個を記録する。shared backlink 5 個 + dedicated row 1 個が既存 expectation を変更せず、368 cases / 332 requirements 内の active runner 153 を保護する。exact/corruption/immutable-output coverage、全 23 nonidentity definition order、missing/duplicate/label/spelling/radix と imported/ambiguous/deeper/multi-hop near miss、set Tasks 122/138/146/148/150/195/201/203 および object Tasks 189/145/147/149/151/196/202/204 との bidirectional isolation、real sidecar は executable である。multi-hop Inner/Base assertion、matching object sibling、broader semantics、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままである。checker source または module-layout change は不要であった。

## Task 206 MC-G020 current-state override

Task 206 は exact three-edge object-terminal immediate-radix asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。Task 73 は real source-derived object-terminal expansion 4 個、Task 196 は formula consumer、Tasks 201/204/205 は変更しない immediate-radix relation を提供する。active route は distinct Outer/Middle provenance を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 4 個を消費し、known entry 3 個を Base-definition-RHS `BuiltinObject` 1 個へ normalize し、`Inferred` variable 1 個と constraint/fact/candidate/diagnostic/deferred 0 個の `Checked` assertion 1 個を object/set coercion なしで記録する。shared backlink 5 個 + dedicated row 1 個が既存 expectation を変更せず、369 cases / 333 requirements 内の active runner 154 を保護する。exact/corruption/immutable-output coverage、全 23 nonidentity definition order、各 definition の missing/duplicate/label/spelling/radix と imported/ambiguous/deeper/multi-hop/local-other near miss、set Tasks 122/138/146/148/150/195/201/203/205 および object Tasks 189/145/147/149/151/196/202/204 との bidirectional isolation、real sidecar は executable である。multi-hop Inner/Base assertion、broader semantics、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままである。checker source または module-layout change は不要であった。

## Task 207 MC-G020 current-state override

Task 207 は exact four-edge set-terminal immediate-radix asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。Task 74 は real source-derived set-terminal expansion 5 個、Task 197 は formula consumer、Tasks 201/203/205/206 は変更しない immediate-radix relation を提供する。active route は distinct TooDeep/Outer provenance を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 5 個を消費し、known entry 3 個を Base-definition-RHS `BuiltinSet` 1 個へ normalize し、`Inferred` variable 1 個と constraint/fact/candidate/diagnostic/deferred 0 個の `Checked` assertion 1 個を記録する。shared backlink 5 個 + dedicated row 1 個が既存 expectation を変更せず、370 cases / 334 requirements 内の active runner 155 を保護する。exact/corruption/immutable-output coverage、全 119 nonidentity definition order、全 per-definition/asserted-head near miss、全 symbol の imported/ambiguous check、declared owner route 20 件との bidirectional isolation、real sidecar は executable である。multi-hop Middle/Inner/Base assertion、matching object sibling、broader semantics、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままである。checker source または module-layout change は不要であった。

## Task 208 MC-G020 current-state override

Task 208 は exact four-edge object-terminal immediate-radix asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。Tasks 74/153 は real source-derived object-terminal expansion 5 個、Task 198 は formula consumer、Tasks 202/204/206/207 は変更しない immediate-radix relation を提供する。active route は distinct TooDeep/Outer provenance を保持し、ordinal 1 を `BindingId(0)` に解決し、expansion 5 個を消費し、known entry 3 個を Base-definition-RHS `BuiltinObject` 1 個へ normalize し、`Inferred` variable 1 個と constraint/fact/candidate/diagnostic/deferred 0 個の `Checked` assertion 1 個を object/set coercion なしで記録する。shared backlink 5 個 + dedicated row 1 個が既存 expectation を変更せず、371 cases / 335 requirements 内の active runner 156 を保護する。exact/corruption/immutable-output coverage、全 119 order、全 source/provenance near miss、owner route 21 件との bidirectional isolation、real sidecar は executable である。multi-hop Middle/Inner/Base assertion、broader semantics、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままである。checker source または module-layout change は不要であった。

## task 209 MC-G020 current-state override

Task 209 は exact seven-expansion set-terminal immediate-radix asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 が直接支え、Task 74 は real source-derived expansion 7 個、Task 199 は formula consumer、Task 175 は builtin sibling/guard、closed relation は exact immediate edge を提供する。active route は distinct ChainMode6/ChainMode5 provenance を保持し、ordinal 1 を `BindingId(0)` に解決し、known entry 3 個を BaseModeDef-RHS `BuiltinSet` 1 個へ normalize し、`Inferred` variable 1 個と constraint/fact/candidate/diagnostic/deferred 0 個の `Checked` assertion 1 個を記録する。shared 5 + dedicated 1 backlink が既存 expectation を変更せず、372 cases / 336 requirements 内の active runner 157 を保護する。全 5,039 nonidentity order、finite source/provenance/corruption matrix、Task 209 実装前の owner route 34 件、immutable output、real sidecar は executable である。multi-hop、broader semantics、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred である。checker source/module-layout change は不要であった。

## task 210 MC-G020 current-state override

Task 210 は exact seven-expansion object-terminal immediate-radix asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 が直接支え、Task 74 は real source-derived object-terminal expansion 7 個、Task 200 は formula consumer、Task 179 は builtin-object sibling/guard、Task 209 は set-terminal sibling、closed relation は exact immediate edge を提供する。active route は distinct ChainObjectMode6/ChainObjectMode5 provenance を保持し、ordinal 1 を `BindingId(0)` に解決し、known entry 3 個を BaseObjectModeDef-RHS `BuiltinObject` 1 個へ normalize し、object/set coercion なしで `Inferred` variable 1 個と constraint/fact/candidate/diagnostic/deferred 0 個の `Checked` assertion 1 個を記録する。shared 5 + dedicated 1 backlink が既存 expectation を変更せず、373 cases / 337 requirements 内の active runner 158 を保護する。全 5,039 nonidentity order、finite source/provenance/corruption matrix、Task 210 実装前の owner route 35 件、immutable output、real sidecar は executable である。multi-hop、broader semantics、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred である。checker source/module-layout change は不要であった。

## task 211 MC-G020 current-state override

Task 211 は exact two-edge set-terminal two-hop asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 が直接支え、Task 72 は real source-derived expansion 3 個、Tasks 148/186/203 は既存 formula/checker seam と sibling guard を提供する。独立 closed relation は pairwise-distinct symbol を持つ actual bare Outer-to-Middle/Middle-to-Base link 2 本と exact Base-to-set terminal を検証し、generic terminal traversal だけを relation evidence にしない。active route は distinct Outer/Base provenance を保持し、ordinal 1 を `BindingId(0)` に解決し、known entry 3 個を Base-definition-RHS `BuiltinSet` 1 個へ normalize し、`Inferred` variable 1 個と constraint/fact/candidate/diagnostic/deferred 0 個の `Checked` assertion 1 個を記録する。shared 5 + dedicated 1 backlink が既存 expectation を変更せず、374 cases / 338 requirements 内の active runner 159 を保護する。全5 nonidentity order、finite structural/provenance/corruption matrix、既存 owner route 36 件、immutable output、real sidecar は executable である。object sibling、他 distance、broader semantics、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred である。checker source/module-layout change は不要であった。

## task 212 MC-G020 current-state override

Task 212 は exact two-edge object-terminal two-hop asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 が直接支え、Task 72 は real source-derived object expansion 3 個、Tasks 149/187/204/211 は既存 formula/checker seam、object sibling、closed two-link relation を提供する。relation は pairwise-distinct symbol を持つ actual bare Outer-to-Middle/Middle-to-Base link 2 本と exact Base-to-object terminal を検証し、generic terminal traversal だけを relation evidence にしない。active route は distinct Outer/Base provenance を保持し、ordinal 1 を `BindingId(0)` に解決し、known entry 3 個を Base-definition-RHS `BuiltinObject` 1 個へ normalize し、object/set coercion なしで `Inferred` variable 1 個と constraint/fact/candidate/diagnostic/deferred 0 個の `Checked` assertion 1 個を記録する。shared 5 + dedicated 1 backlink が既存 expectation を変更せず、375 cases / 339 requirements 内の active runner 160 を保護する。全5 nonidentity order、finite structural/provenance/corruption matrix、既存 owner route 37 件、immutable output、real sidecar は executable である。他 distance、broader semantics、object/set coercion、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred である。checker source/module-layout change は不要であった。

## task 213 MC-G020 current-state override

Task 213 は exact three-edge set-terminal two-hop asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 が直接支え、Task 73 は real source-derived expansion 4 個、Tasks 195/205/211/212 は formula/checker seam、immediate-edge sibling、closed two-link relation、object-terminal guard を提供する。refine した relation は pairwise-distinct な Outer-to-Middle/Middle-to-Inner bare link を直接検証し、Inner-to-Base-to-set tail は cycle-safe terminal normalization だけで検証して relation reachability を確立しない。active route は distinct Outer/Inner provenance を保持し、ordinal 1 を `BindingId(0)` に解決し、known entry 3 個を Base-definition-RHS `BuiltinSet` 1 個へ normalize し、`Inferred` variable 1 個と constraint/fact/candidate/diagnostic/deferred 0 個の `Checked` assertion 1 個を記録する。shared 5 + dedicated 1 backlink が既存 expectation を変更せず、376 cases / 340 requirements 内の active runner 161 と type-elaboration coverage 208/196 を保護する。全23 nonidentity order、finite structural/provenance/corruption matrix、Task 211/212 focused regression、既存 owner route 38 件、immutable output、real sidecar は executable である。object sibling、full-distance と broader semantics、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred である。checker source/module-layout change は不要であった。

## task 214 MC-G020 current-state override

Task 214 は exact three-edge object-terminal two-hop asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 が直接支え、Task 73 は real source-derived object expansion 4 個、Tasks 196/206/211/212/213 は formula/checker seam、immediate-edge sibling、変更しない closed two-link relation、set-terminal guard を提供する。relation は pairwise-distinct な Outer-to-Middle/Middle-to-Inner bare link を直接検証し、Inner-to-Base-to-object tail は terminal normalization だけで検証して relation reachability を確立しない。active route は distinct Outer/Inner provenance を保持し、ordinal 1 を `BindingId(0)` に解決し、known entry 3 個を Base-definition-RHS `BuiltinObject` 1 個へ normalize し、object/set coercion なしで `Inferred` variable 1 個と constraint/fact/candidate/diagnostic/deferred 0 個の `Checked` assertion 1 個を記録する。shared 5 + dedicated 1 backlink が既存 expectation を変更せず、377 cases / 341 requirements 内の active runner 162、type-elaboration coverage 209/197、pass/fail 193/184 を保護する。全23 nonidentity order、finite structural/provenance/corruption matrix、Task 211/212/213 focused regression、既存 owner route 39 件、immutable output、real sidecar は executable である。full-distance と broader semantics、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred である。checker source/module-layout change は不要であった。

## task 215 MC-G020 current-state override

Task 215 は exact four-edge set-terminal two-hop asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 が直接支え、Task 74 は real source-derived set expansion 5 個、Task 197 は formula/checker consumer、Task 207 は immediate-edge sibling、Tasks 211-214 は変更しない closed two-link relation を提供する。relation は pairwise-distinct な TooDeep-to-Outer/Outer-to-Middle bare link を直接検証し、Middle-to-Inner-to-Base-to-set tail は terminal normalization だけで検証して relation reachability を確立しない。active route は distinct TooDeep/Middle provenance を保持し、ordinal 1 を `BindingId(0)` に解決し、known entry 3 個を Base-definition-RHS `BuiltinSet` 1 個へ normalize し、object/set coercion なしで `Inferred` variable 1 個と constraint/fact/candidate/diagnostic/deferred 0 個の `Checked` assertion 1 個を記録する。shared 5 + dedicated 1 backlink が既存 expectation を変更せず、378 cases / 342 requirements 内の active runner 163、type-elaboration coverage 210/198、pass/fail 194/184 を保護する。全119 nonidentity order、finite structural/provenance/corruption matrix、Tasks 211-214 focused regression、既存 owner route 40 件、immutable output、real sidecar は executable である。object sibling、three-hop/full-distance と broader semantics、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred である。checker source/module-layout change は不要であった。

## task 216 MC-G020 current-state override

Task 216 は exact four-edge object-terminal two-hop asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 が直接支え、Task 74 は real source-derived object expansion 5 個、Task 198 は formula/checker consumer、Task 208 は immediate-edge sibling、Tasks 211-215 は byte-for-byte 変更しない closed two-link relation を提供する。relation は pairwise-distinct な TooDeep-to-Outer/Outer-to-Middle bare link を直接検証し、Middle-to-Inner-to-Base-to-object tail は terminal normalization だけで検証して relation reachability を確立しない。active route は distinct TooDeep/Middle provenance を保持し、ordinal 1 を `BindingId(0)` に解決し、known entry 3 個を Base-definition-RHS `BuiltinObject` 1 個へ normalize し、object/set coercion なしで `Inferred` variable 1 個と constraint/fact/candidate/diagnostic/deferred 0 個の `Checked` assertion 1 個を記録する。shared 5 + dedicated 1 backlink が既存 expectation を変更せず、379 cases / 343 requirements 内の active runner 164、type-elaboration coverage 211/199、pass/fail 195/184 を保護する。全119 nonidentity order、finite structural/provenance/corruption matrix、Tasks 211-215 focused regression、既存 owner route 41 件、immutable output、real sidecar は executable である。three-hop/full-distance と broader semantics、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred である。checker source/module-layout change は不要であった。

## task 217 MC-G020 current-state override

Task 217 は exact three-edge set-terminal full-distance three-hop asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 が直接支え、Task 73 は real source-derived expansion 4 個、Task 195 は formula/checker consumer、Tasks 211-216 は shorter-distance/terminal-sibling guard を提供する。新しい closed relation は pairwise-distinct な Outer-to-Middle/Middle-to-Inner/Inner-to-Base bare link を直接検証し、Base-to-set は terminal normalization のみで relation reachability を確立しない。active route は distinct Outer/Base provenance を保持し、ordinal 1 を `BindingId(0)` に解決し、known entry 3 個を Base-definition-RHS `BuiltinSet` 1 個へ normalize し、`Inferred` variable 1 個と constraint/fact/candidate/diagnostic/deferred 0 個の `Checked` assertion 1 個を記録する。shared 5 + dedicated 1 backlink が既存 expectation を変更せず、380 cases / 344 requirements 内の active runner 165、type-elaboration coverage 212/200、pass/fail 196/184 を保護する。全23 nonidentity order、finite structural/provenance/corruption matrix、Tasks 211-216 focused regression、既存 owner route 42 件、immutable output、real sidecar は executable である。object sibling、他 depth、broader semantics、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred である。checker source/module-layout change は不要であった。

## task 218 MC-G020 current-state override

Task 218 は exact three-edge object-terminal full-distance three-hop asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 が直接支え、Task 73 は real source-derived object expansion 4 個、Task 196 は formula/checker consumer、Tasks 211-217 は shorter-distance/terminal-sibling guard と byte-for-byte 変更しない `BindingThreeHopRadix` を提供する。active route は pairwise-distinct な Outer-to-Middle/Middle-to-Inner/Inner-to-Base bare link を直接検証し、Base-to-object は terminal normalization のみに使い relation reachability を確立しない。distinct Outer/Base provenance を保持し、ordinal 1 を `BindingId(0)` に解決し、known entry 3 個を Base-definition-RHS `BuiltinObject` 1 個へ normalize し、object/set coercion なしで `Inferred` variable 1 個と constraint/fact/candidate/diagnostic/deferred 0 個の `Checked` assertion 1 個を記録する。shared 5 + dedicated 1 backlink は既存 expectation を変更せず、381 cases / 345 requirements 内の active runner 166、type-elaboration coverage 213/201、pass/fail 197/184 を計上する。全23 nonidentity order、finite structural/provenance/corruption matrix、Tasks 211-217 focused regression、既存 owner route 43 件、immutable output、real sidecar は executable である。他 depth、broader semantics、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred である。checker source/module-layout change は不要であった。

## task 219 MC-G020 current-state override

Task 219 は exact four-edge set-terminal three-hop asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 が直接支え、Task 74 は real source-derived set expansion 5 個、Task 197 は formula/checker consumer、Task 207 は four-edge immediate-edge sibling guard、Tasks 211-218 は shorter-distance/terminal-sibling guard と byte-for-byte 変更しない `BindingThreeHopRadix` を提供する。active route は pairwise-distinct な TooDeep-to-Outer/Outer-to-Middle/Middle-to-Inner bare link を直接検証し、Inner-to-Base-to-set tail は terminal normalization のみに使って relation reachability を確立しない。distinct TooDeep/Inner provenance、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinSet` 1 個へ normalize する known entry 3 個、`Inferred` variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の `Checked` assertion 1 個を保持する。shared 5 + dedicated 1 backlink は既存 expectation を変更せず、382 cases / 346 requirements 内の active runner 167、type-elaboration coverage 214/202、pass/fail 198/184 を計上する。全119 nonidentity order、unconnected unsupported deeper asserted head と actual connected sixth-definition/sixth-edge asserted head の独立 guard を含む finite structural/provenance/corruption matrix、Tasks 207 と 211-218 focused regression、既存 owner route 44 件、immutable output、real sidecar は executable である。object sibling、Base full-distance assertion、broader semantics、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred である。checker source/module-layout change は不要であった。

## task 220 MC-G020 current-state override

Task 220 は exact four-edge object-terminal three-hop asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 が直接支え、Task 74 は real source-derived object expansion 5 個、Task 198 は formula/checker consumer、Task 208 は four-edge immediate-edge sibling guard、Tasks 211-219 は shorter-distance/terminal-sibling guard と byte-for-byte 変更しない `BindingThreeHopRadix` を提供する。active route は pairwise-distinct な TooDeep-to-Outer/Outer-to-Middle/Middle-to-Inner bare link を直接検証し、Inner-to-Base-to-object tail は terminal normalization のみに使って relation reachability を確立しない。distinct TooDeep/Inner provenance、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinObject` 1 個へ normalize する known entry 3 個、`Inferred` variable 1 個、object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の `Checked` assertion 1 個を保持する。shared 5 + dedicated 1 backlink は既存 expectation を変更せず、383 cases / 347 requirements 内の active runner 168、type-elaboration coverage 215/203、pass/fail 199/184 を計上する。全119 nonidentity order、unconnected unsupported deeper asserted head と actual connected sixth-definition/sixth-edge asserted head の独立 guard を含む finite structural/provenance/corruption matrix、Tasks 208 と 211-219 focused regression、既存 owner route 45 件、immutable output、real sidecar は executable である。Base full-distance assertion、broader semantics、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred である。checker source/module-layout change は不要であった。

## task 221 MC-G020 active override

Task 221 は exact four-edge set-terminal full-distance four-hop asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 が直接支え、Task 74 は real source-derived set expansion 5 個、Task 197 は formula/checker consumer を供給する。active closed `BindingFourHopRadix` は pairwise-distinct な TooDeep-to-Outer/Outer-to-Middle/Middle-to-Inner/Inner-to-Base bare link を直接検証し、Base-to-set は terminal normalization のみに保つ。route は distinct TooDeep/Base provenance、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinSet` 1 個、`Inferred` variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の `Checked` assertion 1 個を保持する。backlink 6 件、全119 nonidentity order、unconnected-deeper と actual connected fifth-link の独立 guard を含む exhaustive finite structural/provenance/corruption coverage、Task 207 と Tasks 211-220 focused regression、先行 owner route 46 件、immutable output、real sidecar が active runner 169 を保護する。plan は 384 cases、348 requirements、type-elaboration coverage 216/204、pass/fail 200/184 を持つ。object sibling、longer chain、imported-positive definition、attributed/argument-bearing behavior、general reachability、broader semantics、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままであり、checker source/module-layout change は不要であった。

## task 222 MC-G020 active override

Task 222 は exact four-edge object-terminal full-distance four-hop asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 が直接支え、Task 74 は real source-derived object expansion 5 個、Task 198 は formula/checker consumer を供給する。変更しない closed `BindingFourHopRadix` は pairwise-distinct な TooDeep-to-Outer/Outer-to-Middle/Middle-to-Inner/Inner-to-Base bare link を直接検証し、Base-to-object は terminal normalization のみに保つ。active route は distinct TooDeep/Base provenance、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinObject` 1 個、`Inferred` variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の `Checked` assertion 1 個を object/set coercion なしで保持する。backlink 6 件、全119 nonidentity order、unconnected-deeper と actual connected fifth-link の独立 guard を含む exhaustive finite structural/provenance/corruption coverage、Task 208 と Tasks 211-221 focused regression、先行 owner route 47 件、immutable output、real sidecar が active runner 170 を保護する。active corpus は 385 cases、349 requirements、type-elaboration coverage 217/205、pass/fail 201/184 を持つ。relevant-crate と workspace verification は成功した。longer chain、imported-positive definition、attributed/argument-bearing behavior、general reachability、broader semantics、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままであり、checker source/module-layout change は不要であった。

## task 223 MC-G020 active override

Task 223 は exact single-left-parenthesized reserved-variable equality seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 04/13/14/16 は、free reserved theorem identifier が real reserve type を得ること、parenthesization が inner expression の型を保持して FOL encoding を変えないこと、equality が term 2 個を消費することを直接規定する。real parser は `ParenthesizedTerm` を生成し、Task 119 は reserve extraction、`BindingEnv` lookup、equality consumer を供給する。active source route は exact `TermExpression -> ParenthesizedTerm -> TermExpression -> TermReference` structure を検証し、独立 wrapper/inner/right provenance を保持し、inner/right `x` を ordinal 1/2 で `BindingId(0)` へ解決して、parenthesis 独自 type/child graph/axiom/fact/FOL node を捏造せず inner reference の reserve-derived builtin-set type/value を透明に再利用する。finite matrix は direct/right/both/nested/non-identifier/recovered variant を reject し、matched output/source corruption、immutable output、先行 reserved-variable binary-formula owner 52 件との双方向 isolation、real sidecar を保護する。backlink 5 件は 386 cases / 350 requirements、type-elaboration coverage 218/206、pass/fail 202/184 内の active runner 171 を計上する。focused、relevant-crate、workspace verification は成功した。arbitrary nesting/operand/precedence、formula grouping、closure materialization、truth/fact、acceptance、proof/IR/VC、broader child semantics は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままであり、checker source/module-layout change は不要であった。

## task 224 MC-G020 active override

Task 224 は exact seven-expansion set-terminal two-hop asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 は mode unfolding、reserved theorem-variable typing、assertion reachability を直接支える。Task 74 は real expansion 7 個、Task 199 は real formula/checker consumer、Task 211 は変更しない closed `BindingTwoHopRadix` を供給し、Task 209 は immediate-edge sibling のみである。active route は pairwise-distinct `ChainMode6 -> ChainMode5 -> ChainMode4` を直接検証し、残る tail は terminal normalization のみに使い、distinct source provenance、ordinal 1 / `BindingId(0)`、BaseModeDef-RHS `BuiltinSet` 1 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。backlink 6 件、全5,039 order finite matrix、先行 owner 48 件、immutable output、real sidecar は 387 cases / 351 requirements、type-elaboration 219/207 内の active runner 172 を保護する。focused、relevant-crate、workspace verification は成功した。broader semantics/downstream payload は deferred、Step 5 は active、Steps 6/7 は deferred のままとする。checker source/module-layout change は不要であった。

## Task 225 MC-G020 active override

Task 225 は exact seven-expansion object-terminal two-hop asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 は mode unfolding、reserved theorem-variable typing、assertion reachability を直接支える。Task 74 は real object expansion 7 個、Task 200 は real formula/checker consumer、Task 211 は変更しない closed `BindingTwoHopRadix` を供給し、Task 210 は immediate-edge sibling、Task 224 は set-terminal two-hop sibling である。active route は pairwise-distinct `ChainObjectMode6 -> ChainObjectMode5 -> ChainObjectMode4` を直接検証し、残る tail は object-terminal normalization のみに使い、distinct source provenance、ordinal 1 / `BindingId(0)`、BaseObjectModeDef-RHS `BuiltinObject` 1 個、inferred variable 1 個、object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。backlink 6 件、全5,039 order finite matrix、先行 owner 49 件、immutable output、real sidecar は 388 cases / 352 requirements、type-elaboration 220/208 内の active runner 173 を保護する。focused、relevant-crate、workspace verification は成功した。broader semantics/downstream payload は deferred、Step 5 は active、Steps 6/7 は deferred のままとし、checker source/module-layout change は不要であった。

## Task 226 MC-G020 active override

Task 226 は exact seven-expansion set-terminal three-hop asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 は mode unfolding、reserved theorem-variable typing、assertion reachability を直接支える。Task 74 は real set expansion 7 個、Task 199 は real formula/checker consumer、Task 217 は変更しない closed `BindingThreeHopRadix` を供給する。Task 219 は set-terminal three-hop longer-tail sibling、Tasks 209/224 は immediate/two-hop long-chain sibling である。active route は pairwise-distinct `ChainMode6 -> ChainMode5 -> ChainMode4 -> ChainMode3` を直接検証し、残る tail は set-terminal normalization のみに使い、distinct source provenance、ordinal 1 / `BindingId(0)`、BaseModeDef-RHS `BuiltinSet` 1 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。backlink 6 件、全5,039 order finite matrix、先行 owner 50 件、immutable output、real sidecar は 389 cases / 353 requirements、type-elaboration 221/209 内の active runner 174 を保護する。focused、relevant-crate、workspace verification は成功した。object sibling、broader semantics、downstream payload は deferred、Step 5 は active、Steps 6/7 は deferred のままとし、checker source/module-layout change は不要であった。

## Task 227 MC-G020 active override

Task 227 は exact seven-expansion object-terminal three-hop asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 は mode unfolding、reserved theorem-variable typing、assertion reachability を直接支える。Task 74 は real object expansion 7 個、Task 200 は real formula/checker consumer、Task 217 は変更しない closed `BindingThreeHopRadix` を供給する。Task 220 は object-terminal three-hop longer-tail sibling、Task 226 は depth-matched set sibling、Tasks 210/225 は immediate/two-hop object long-chain sibling である。active route は pairwise-distinct `ChainObjectMode6 -> ChainObjectMode5 -> ChainObjectMode4 -> ChainObjectMode3` を直接検証し、残る tail は object-terminal normalization のみに使い、distinct source provenance、ordinal 1 / `BindingId(0)`、BaseObjectModeDef-RHS `BuiltinObject` 1 個、inferred variable 1 個、object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。backlink 6 件、全5,039 order finite matrix、先行 owner 51 件、immutable output、focused sibling、real sidecar は 390 cases / 354 requirements、type-elaboration 222/210 内の active runner 175 を保護する。focused、relevant-crate、workspace verification は成功した。broader semantics と downstream payload は deferred、Step 5 は active、Steps 6/7 は deferred のままとし、checker source/module-layout change は不要であった。

## Task 228 MC-G020 active override

Task 228 は exact seven-expansion set-terminal four-hop asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 は mode unfolding、reserved theorem-variable typing、assertion reachability を直接支える。Task 74 は real set expansion 7 個、Task 199 は real formula/checker consumer、Task 221 は変更しない closed `BindingFourHopRadix` を供給する。Tasks 224/226 は two/three-hop long-chain sibling、Task 222 は object-terminal relation sibling、Task 227 は latest terminal sibling である。active route は pairwise-distinct `ChainMode6 -> ChainMode5 -> ChainMode4 -> ChainMode3 -> ChainMode2` を直接検証し、残る tail は set-terminal normalization のみに使い、distinct source provenance、ordinal 1 / `BindingId(0)`、BaseModeDef-RHS `BuiltinSet` 1 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。backlink 6 件、全5,039 order finite matrix、先行 owner 52 件、immutable output、focused sibling、real sidecar は 391 cases / 355 requirements、type-elaboration 223/211 内の active runner 176 を保護する。focused、relevant-crate、workspace verification は成功した。broader semantics と downstream payload は deferred、Step 5 は active、Steps 6/7 は deferred のままとし、checker source/module-layout change は不要であった。

## Task 229 MC-G020 active override

Task 229 は exact seven-expansion object-terminal four-hop asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 は object-rooted mode unfolding、reserved theorem-variable typing、assertion reachability を直接支える。Task 74 は real object expansion 7 個、Task 200 は real formula/checker consumer、Task 221 は変更しない closed `BindingFourHopRadix` を供給する。Tasks 225/227 は two/three-hop object long-chain sibling、Task 222 は object-terminal relation sibling、Task 228 は depth-matched set sibling である。active route は pairwise-distinct `ChainObjectMode6 -> ChainObjectMode5 -> ChainObjectMode4 -> ChainObjectMode3 -> ChainObjectMode2` を直接検証し、残る tail は object-terminal normalization のみに使い、distinct source provenance、ordinal 1 / `BindingId(0)`、BaseObjectModeDef-RHS `BuiltinObject` 1 個、inferred variable 1 個、object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。backlink 6 件、全5,039 order finite matrix、先行 owner 53 件、immutable output、focused sibling、real sidecar は 392 cases / 356 requirements、type-elaboration 224/212 内の active runner 177 を保護する。focused、relevant-crate、workspace verification は成功した。broader semantics と downstream payload は deferred、Step 5 は active、Steps 6/7 は deferred のままとし、checker source/module-layout change は不要であった。

## Task 230 MC-G020 active override

Task 230 は exact seven-expansion set-terminal five-hop asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 は mode unfolding、reserved theorem-variable typing、assertion reachability を直接支える。Task 74 は real set expansion 7 個、Task 199 は real formula/checker consumer を供給する。新規 closed `BindingFiveHopRadix` は pairwise-distinct `ChainMode6 -> ChainMode5 -> ChainMode4 -> ChainMode3 -> ChainMode2 -> ChainMode1` を直接検証し、`ChainMode1 -> BaseMode -> set` は terminal-normalization evidence のみに使う。active route は distinct source provenance、ordinal 1 / `BindingId(0)`、BaseModeDef-RHS `BuiltinSet` 1 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。backlink 6 件、全5,039 order finite matrix、先行 owner 54 件、immutable output、focused sibling、real sidecar は 393 cases / 357 requirements、type-elaboration 225/213、pass/fail 209/184 内の active runner 178 を既存 expectation の変更なしで保護する。focused、relevant-crate、workspace verification は成功した。object-terminal five-hop、broader semantics、downstream payload は deferred、Step 5 は active、Steps 6/7 は deferred のままとし、checker source/module-layout change は不要であった。

## Task 231 MC-G020 active override

Task 231 は exact seven-expansion object-terminal five-hop asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 は object-rooted mode unfolding、reserved theorem-variable typing、assertion reachability を直接支える。Task 74 は real object expansion 7 個、Task 200 は real formula/checker consumer、Task 230 は byte-for-byte unchanged closed `BindingFiveHopRadix` を供給する。active route は pairwise-distinct `ChainObjectMode6 -> ChainObjectMode5 -> ChainObjectMode4 -> ChainObjectMode3 -> ChainObjectMode2 -> ChainObjectMode1` を直接検証し、`ChainObjectMode1 -> BaseObjectMode -> object` は terminal-normalization evidence のみに使う。distinct source provenance、ordinal 1 / `BindingId(0)`、BaseObjectModeDef-RHS `BuiltinObject` 1 個、inferred variable 1 個、object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。backlink 6 件、全5,039 order finite matrix、先行 owner 55 件、immutable output、focused sibling、real sidecar は 394 cases / 358 requirements、type-elaboration 226/214、pass/fail 210/184 内の active runner 179 を既存 expectation の変更なしで保護する。focused、relevant-crate、workspace verification は成功した。imported-positive definition、broader semantics、downstream payload は deferred、Step 5 は active、Steps 6/7 は deferred のままとし、checker source/module-layout change は不要であった。

## Task 233 MC-G020 active override

Task 233 は exact single-left-parenthesized builtin-object reserved-variable equality seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/13/14/16 は builtin `object`、reserved theorem-variable typing、追加 FOL meaning のない parenthesized type preservation、equality operand、theorem syntax を直接支える。Task 223 は real unrecovered single-left `ParenthesizedTerm` producer、Task 188 は real object reserve/BindingEnv/equality consumer を供給する。credit は exact `reserve x for object; theorem ParenthesizedReservedObjectVariableEqualityPayloadBoundary: (x) = x;` source、独立 wrapper/inner/right provenance、ordinal 1/2 の `BindingId(0)` lookup、canonical `BuiltinObject` 1 個、inferred term 2 個、ordered expected constraint 2 個、object/set coercion と独立 wrapper payload のない checked equality 1 個に限定する。backlink 6 件、finite exact/near-miss/provenance/corruption matrix、先行 binary-formula owner 53 件との bidirectional isolation、immutable output、real sidecar は 395 cases / 359 requirements、type-elaboration 227/215、pass/fail 211/184 内の active runner 180 を保護する。arbitrary parenthesization/operand、formula grouping、closure/order、equality truth/fact、acceptance、child graph、proof/CoreIr/ControlFlowIr/VC、downstream payload は deferred、Step 5 は active、Steps 6/7 は deferred のままとする。checker source/module-layout change は不要であった。

## Task 234 MC-G020 active override

Task 234 は exact seven-expansion set-terminal full-distance six-hop asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 は mode unfolding、reserved theorem-variable typing、assertion reachability を直接支える。Task 74 は real set expansion 7 個、Task 199 は real formula/checker consumer を供給する。新規 closed `BindingSixHopRadix` は pairwise-distinct `ChainMode6 -> ChainMode5 -> ChainMode4 -> ChainMode3 -> ChainMode2 -> ChainMode1 -> BaseMode` を直接検証し、`BaseMode -> set` は terminal-normalization evidence のみに使う。credit は distinct source provenance、ordinal 1 / `BindingId(0)`、BaseModeDef-RHS `BuiltinSet` 1 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個に限定する。backlink 6 件、全5,039 order finite matrix、先行 owner 56 件、immutable output、focused sibling、real sidecar は既存 expectation を変更せず 396 cases / 360 requirements、type-elaboration 228/216、pass/fail 212/184 内の active runner 181 を保護する。object-terminal six-hop、imported-positive definition、broader semantics、downstream payload は deferred、Step 5 は active、Steps 6/7 は deferred。checker source/module-layout change は不要であった。

## Task 236 MC-G020 active override

Task 236 は exact seven-expansion object-terminal full-distance six-hop asserted-head seam を `test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とはしない。specs 03/04/07/13/14/16 は mode unfolding、reserved theorem-variable typing、assertion reachability を直接支える。Task 74 は real object expansion 7 個、Task 200 は real formula/checker consumer を供給する。unchanged closed `BindingSixHopRadix` は pairwise-distinct `ChainObjectMode6 -> ChainObjectMode5 -> ChainObjectMode4 -> ChainObjectMode3 -> ChainObjectMode2 -> ChainObjectMode1 -> BaseObjectMode` を直接検証し、`BaseObjectMode -> object` は terminal-normalization evidence のみに使う。credit は distinct source provenance、ordinal 1 / `BindingId(0)`、BaseObjectModeDef-RHS `BuiltinObject` 1 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個で object/set coercion のない checked assertion 1 個に限定する。backlink 6 件、全5,039 order finite matrix、先行 owner 57 件、immutable output、focused sibling、real sidecar は既存 expectation を変更せず 397 cases / 361 requirements、type-elaboration 229/217、pass/fail 213/184 内の active runner 182 を保護する。imported-positive definition、broader semantics、downstream payload は deferred、Step 5 は active、Steps 6/7 は deferred。checker source/module-layout change は不要であった。

## Task 241 MC-G020 Active Override

Task 241 は exact single-left-parenthesized builtin-set inequality seam を
`test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とは
しない。Chapter 04/13/14/16 は reserved theorem-variable typing、
type-preserving/FOL-transparent parenthesis、builtin `<>`、theorem formula を
直接支える。Task 223 は real wrapper producer、Task 121 は real inequality
consumer を供給する。credit は exact source、独立した wrapper/inner/right
provenance、ordinal 1/2 の `BindingId(0)` lookup、canonical `BuiltinSet` 1 個、
inferred term 2 個、ordered expected constraint 2 個、独立 wrapper payload の
ない checked inequality 1 個に限定する。shared 4 + dedicated 1 backlink、
parenthesized membership と builtin-object near miss を含む finite matrix、
先行 owner 54 件との bidirectional isolation、immutable output、real sidecar
は active runner 183、398 cases / 362 requirements、type-elaboration 230/218、
pass/fail 214/184 を保護する。既存 fixture/expectation は rebaseline しない。
parenthesized membership、imported/other parenthesized variant、inequality
desugaring/truth、acceptance、proof/CoreIr/ControlFlowIr/VC、downstream payload
は Task 241 の credit 外。Step 5 は active、Steps 6/7 は deferred。checker
source/API/module-layout change は不要であった。

## Task 242 MC-G020 Active Override

Task 242 は exact single-left-parenthesized builtin-object inequality seam を
`test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とは
しない。Chapter 03/04/13/14/16 は builtin `object`、reserved theorem-variable
typing、type/FOL-transparent parenthesis、atomic `<>`、theorem formula を直接
支える。Task 233 は real wrapper producer、Task 190 は real object inequality
consumer を供給する。credit は exact source、独立した wrapper/inner/right
provenance、ordinal 1/2 の `BindingId(0)` lookup、canonical `BuiltinObject` 1
個、inferred term 2 個、type entry 6 個、ordered expected constraint 2 個、
checked inequality 1 個、独立 wrapper payload/object-set coercion なしに限定
する。shared 5 + dedicated 1 backlink、finite matrix、先行 owner 55 件との
bidirectional isolation、immutable output、focused regression、real sidecar は
active runner 184、399 cases / 363 requirements、type-elaboration 231/219、
pass/fail 215/184 を既存 expectation の rebaseline なしで保護する。
parenthesized membership と active imported provenance は globally deferred
ではなく Task 242 credit 外。未成立 imported expansion/evidence/signature
payload、proof/CoreIr/ControlFlowIr/VC、broader dependency のみ deferred。
Step 5 は active、Steps 6/7 は deferred。checker source/API/module-layout
change は不要であった。

## Task 243 MC-G020 Active Override

Task 243 は exact single-left-parenthesized builtin-set membership seam を
`test_gap`、narrow `source_drift`、`design_drift` と分類し、`spec_gap` とは
しない。Chapter 04/13/14/16 は reserved theorem-variable typing、type/FOL-
transparent parenthesis、atomic membership、theorem formula を直接支える。
Task 223 は real wrapper producer、Task 120 は変更しない direct-right
expected-set producer を含む real membership consumer を供給する。credit は
exact source、独立 wrapper/inner/right provenance、ordinal 1/2 の
`BindingId(0)` lookup、written-set-anchored canonical `BuiltinSet` 1 個、inferred
term 2 個、type entry 5 個、left expected input 0 個、right-owned expected-set
constraint 1 個、独立 wrapper payload のない checked membership 1 個に限定
する。shared 4 + dedicated 1 backlink、finite matrix、先行 owner 56 件との
bidirectional isolation、immutable output、focused regression、real sidecar は
active runner 185、400 cases / 364 requirements、type-elaboration 232/220、
pass/fail 216/184 を既存 expectation の rebaseline なしで保護する。旧
parenthesized-membership extraction gap の解除はこの exact source だけ。
object-left/set-right parenthesized membership と active imported provenance は
Task 243 credit 外。未成立 imported expansion/evidence/signature payload、
proof/CoreIr/ControlFlowIr/VC、broader dependency は deferred。Step 5 は
active、Steps 6/7 は deferred。checker source/API/module-layout change は不要
であった。

## Task 244 MC-G020 Active Override

Task 244 は exact two-reserve single-left-parenthesized heterogeneous membership
seam を `test_gap` + narrow `source_drift` + `design_drift` と分類する。
`spec_gap` ではない。authority source は `reserve x for object; reserve y for
set; theorem ParenthesizedHeterogeneousReserveMembershipPayloadBoundary: (x) in
y;` であり、Chapters 03/04/13/14/16 と既存 Task 125 intent から導く。

real Task 233 wrapper producer を Task 125 の real two-binding consumer へ接続
する。finite config-driven private helper が exact ordered reserve/binding
matrix、operand index、shared/distinct written type-range policy を検査し、
従来5 config を維持する。payload は ordinal 2/3、distinct `BindingId(0/1)`、
written range に anchor された object/set identity 2件、inferred term 2件、
type entry 5件、right-only expected-set constraint、coercion/wrapper semantics
なしの checked membership を保持する。57-owner isolation、real imported-gap
diagnostic guard、real frontend/resolver sidecar が override を限定する。

trace は shared backlink 5件と dedicated requirement 1件を追加する。active
runner は186、metadata は401/365、type は233/221、pass/fail は217/184。
Task 244 credit は exact source だけ。その他 parenthesized shape と imported-
positive provenance は credit 外。未成立 imported expansion/evidence/signature
payload、proof/CoreIr/ControlFlowIr/VC、broader dependency は deferred。Step 5
は active、Steps 6/7 は deferred。checker source/API/module-layout change は
不要。

## Task 245 MC-G020 Active Override

Task 245 は exact `x in (x)` builtin-set membership seam を `test_gap`、narrow
`source_drift`、`design_drift` と分類する。`spec_gap` ではない。Chapters
04/13/14/16、real right-side wrapper producer、Task 120 real consumer が根拠。
credit は explicit `Right` side/config、distinct wrapper/left/right-inner/
formula provenance、ordinal 1/2 の双方 `BindingId(0)`、written-set
`BuiltinSet` 1件、inferred term 2件、type entry 5件、right-inner-owned sole
expected constraint、wrapper semantics なしの checked membership に限定する。

side/config/range/expected corruption、Task-243 cross-route rejection、既存
owner 58件の双方向、Left route 6件、real sidecar が override を限定する。
runner 187、plan 402/366、type 234/222、pass/fail 218/184、shared 4 + dedicated
1 backlink を expectation rebaseline なしで同期。その他 shape/imported-
positive provenance は credit 外、未成立 imported/proof/downstream payload は
deferred。Step 5 は active、Steps 6/7 は deferred。checker source/API/module-
layout change は不要。

## Task 246 MC-G020 Active Override

Task 246 は exact 3-definition set-terminal chain と single-left-parenthesized
equality の交差だけを `test_gap`、narrow `source_drift`、`design_drift` とし、
`spec_gap` とはしない。Chapters 04/07/13/14/16 と primary Tasks 134/223 が
intent を直接支える。route は pairwise-distinct Base/Middle/Outer provenance、
wrapper/inner-left/direct-right/formula site、real expansion 3件、raw Outer
input 4件、ordinal 1/2 の `BindingId(0)`、terminal-RHS `BuiltinSet` 1件、
inferred term 2件、entry 6件、ordered constraint 2件、clean checked equality
1件を保持し、wrapper semantics を生成しない。conditional node admission と
finite matrix により旧 empty-mode config は closed のまま。runner 188、
metadata 403/367、type 235/223、pass/fail 219/184、shared 5 + dedicated 1。
imported/attributed/argument-bearing、他 side/nesting/depth、acceptance、truth/
facts、child graph、proof/IR/VC は credit 外または deferred。Step 5 active、
Steps 6/7 deferred。

Task 265 follow-up ownership override: joint Task 266はexact Task-180 final
`ResolvedTypedAst` theorem-owner-to-checked-contradiction projectionだけを所有する。
Task 267はomitted-justification proof/terminal-goal representationを別に決定し、
Task 268がaccepted checker producerを実装する。Checker Task 247はその他全
AST-wide/Task-49 payload family decompositionを所有し、core Task 32のexhaustive
remaining-family decompositionへ渡す。Core/VC、truth/fact、
theorem acceptance、broader semantic creditはdeferredで、current coverage/status/
testは不変。

Task 266 implementation addendum: `type_checker` は documented
`TermFormulaInferenceError`、`CheckedStatementOwner`、`StatementOwnerError`
surface を公開し、formula range/recovery と source/module identity を保持して
exact real local theorem owner を validate する。`resolved_typed_ast` は
`StatementSemanticId`、`StatementSemanticInputs`、`StatementSemanticInput`、
`StatementSemantic`、`StatementSemanticTable` を公開し、exact singleton
three-node owner/formula projection を validate する。owner/formula mismatch、
recovery、duplicate、omission、order、range、provenance、module/source corruption
は fail closed。unit/active-runner/corruption/determinism coverage を持つ。
raw-syntax dependency、truth/fact、acceptance、proof、Core、CFG、VC、fixture、
expectation、trace-status の変更はない。

Task 267 target-contract addendum: current sourceにはproof tableがまだないため、
これはimplementation creditではなくaccepted `design_drift` repairである。
Task 268は`resolved_typed_ast.md`記載のexact syntax-free
`Unmodified`/`Omitted` inputとsingleton `PendingAutomaticProof`/direct-terminal
outputを所有する。Task-266 real formula-site identityとcompact formula nodeは
distinctのまま、`proof/0`、empty context/citations、atomic failureを固定した。
Core Task 31がfuture exact non-accepting projectionを所有する。Task 267はcurrent
public API/test/fixture/expectation/trace status/semantic coverageを変更しない。
Task 268 targetはexisting validated `CheckedStatementOwner`もauthenticated
resolver Public/Exported fact保持へ拡張し、proof assemblyがduplicated row constantを
trustしないようにする。これはtarget stateでありcurrent API/implementation
creditではないというTask 267時点の記録であり、次のTask-268 addendumがその
historical stateを更新する。

Task 268 implementation addendum: target stateはexact Task-180 sourceに対する
current checker APIとunit/runner coverageになった。checkerはauthenticated
Public/Exported owner factを保持し、explicit `Unmodified`/`Omitted` singleton
intentだけを受理して、`PendingAutomaticProof`付きproof/direct-terminal-node/
terminal-goal tableをall-or-noneでpublishする。private postvalidationとcorruption
testはcardinality、dense identity/order、provenance/range/recovery、status/root、
cross-reference、empty citation/context、absent label、`proof/0`をcoverする。
broader proof familyはTask 247 ownershipのまま。acceptance、fact、Core/CFG/VC
payload、existing fixture/expectation、trace statusは不変で、Core Task 31が次の
exact consumerである。

## Task 247 remaining-family ownership reconciliation

Task 247は広いfuture-owner wordingを
[payload_family_decomposition.md](./payload_family_decomposition.md)のaccepted
graphへ置換する。current public-surface correspondence/implementation creditは
変更しない。

| Gap | accepted owner/gate |
|---|---|
| MC-G002 | `MT10-FS`、`MT10-AS`、既存Task 49。 |
| MC-G004 | 未命名external artifact/schema owner gate。schema捏造なし。 |
| MC-G005 | 既存nonblocking `spec_gap`: 未命名public diagnostic registry/consumer-adoption gate。stable internal detail keyのみ。 |
| MC-G006 | Task 277 direct-role sliceとblocked未命名parser/syntax/resolver scheme-role gate。 |
| MC-G011 | Tasks 248/257-258/269-270/272。 |
| MC-G014 | Tasks 249-251/262-264。 |
| MC-G016 | Tasks 248-251/258-264/269-273。 |
| MC-G017 | Tasks 252-264。 |
| MC-G018 | Tasks 251/254-255/263/271/278。evidence resultはexternal input。 |
| MC-G019 | Tasks 258/272と`MT10-FS`。accepted theorem factなし。 |
| MC-G020 | 全Tasks 248-264/269-279 producerとpairedなTask-10 consumerのsource extraction。 |
| MC-G021 | Task 273とblocked-reserved accepted-status Task 274。 |
| MC-G023 | Tasks 275-276と`MT10-AS`。 |
| MC-G025 | blocked-reserved Task 274。canonical accepted verifier/artifact producer/schema未命名。 |
| MC-G026 | Task 251 request/referenceとTask 274 accepted-status gate。 |
| MC-G027 | Tasks 277-279と`MT10-AS`。 |
| MC-G030 | `MT10-FS`、`MT10-AS`、Task 49。 |

各producerは適用可能な`TypedAst`/`ResolvedTypedAst` tableへのtransactional
projectionと実Task-10 assertionを所有し、未消費DTOではcloseしない。correctness
producerはcorrectness-condition identity、`InitialObligationId`、source anchor
inputを保持するが`VcId`割当/discharge claimをしない。Task 49はexact 24-fixture
setの23件をactivateして24件全体をreconcile/deduplicateし、same-return memberは
resolver Task 31が`declaration_symbol`でsole activationする。active different-return conflict、
capture-avoidance row、escape/guard row、無関係template seedをbundleへ黙って
二重計上しない。

disagreement classは`design_drift`、`source_drift`、`test_gap`、parser-Task-47
`test_expectation_drift`。MC-G005の既存nonblocking `spec_gap`は残るが、新しい
payload-family `spec_gap`、`source_undocumented_behavior`、現在の
`boundary_violation`、`repo_metadata_conflict`はない。existing trace status/tests/
source/fixture/expectation/coverage creditは不変。

## Task 248 current-state addendum

最初のbounded MC-G011/MC-G016 producer/consumer sliceは実行可能になった。
`SourceBindingContextProducer`はsyntax-free resolver-shell/binding projectionだけを
consumeし、source-item、declaration、`BindingEnv`、local-context linkを
transactionalに`TypedAst`/`ResolvedTypedAst`へpublishする。exact active
reserve-plus-definition-parameter fixtureはsource order、same-spelling distinct
identity、structural shadowingを証明する。Task-248 `test_gap`と2つの
`source_drift`はこのsliceだけrepair済みである。term-use selection、composite binder、
statement/proof context、closure capture、proof-local declarationはTasks
252/257/258/269/270/272に残るため、MC-G011/MC-G016全体はpartialのままである。

## Task 249 frozen-contract audit addendum

fresh inventoryはnext executable source-type sliceをChapters 03/05/07/08/12/18、
Appendix A、MC-G014/MC-G016/MC-G020配下のTask 249へassignした。paired crate
planはexact ten-reserve-root consumer、Task-248 two-row dependency regression、
syntax-free table/validation contract、future single trace row、
post-implementation count oracleを固定した。

## Task 249 implementation audit addendum

frozen producerはpublic syntax-free `source_type` moduleとprivate runner leaf
1件として実装済みである。broad real consumerはexact 10/13/6 tableをpublishして
runner-owned dependency detailで停止する。unchanged Task-248 fixtureはactual
checker-owned binding environmentからexact 2/2/0 tableをco-installする。
`TypedAst`が両handoffを所有し、`ResolvedTypedAst`はcloneだけする。

selected `test_gap`、`source_drift`、`design_drift`はこのbounded input-handoff
sliceについてのみcloseした。existing Tasks 68-71とTask-248 source/sidecar/trace
artifactはbyte-identicalである。new bounded diagnostic trace rowによりplan
411/372、type 238/226、pass/fail 224/187、active type 190、warnings unchangedを
計上する。MC-G014/MC-G016/MC-G020全体はpartialのままで、normalization、
evidence、term/`qua` selection、definition semantics、accepted fact/proof、
downstream IRにcreditしない。implementation reviewでimport-closureとgenerated
declarationの`source_drift`、recursive public-input graphの
`boundary_violation`を検出しrepairした。未解消の`spec_gap`、
`source_undocumented_behavior`、`test_expectation_drift`、
`boundary_violation`、`repo_metadata_conflict`はない。

## Task 250 frozen-contract audit addendum

fresh inventoryはnext executable raw source-attribute sliceをChapters
03/06/11/12、Chapter-17 restricted-adjective boundary、MC-G014/MC-G020配下の
Task 250へassignした。paired crate planはflat chain/polarity/qualifier/group/
actual handoff、exact Task-67/81/84/85 consumer/cardinality、immutable final
ownership、legacy `AttributeInput` coexistence、runner outcome、trace
progression、synthetic prefix/order extractor coverage、corruption/determinism
testを固定した。

このprerequisiteがrepairするのは`design_drift`だけである。implementationまで、
executable handoff不在は`test_gap`、incomplete chain/qualifier/argument/
provenance/final-handoff seamは`source_drift`のまま。Task 250はwritten prefix/
argument-list formを保持し、そのcanonical semantic equivalence、arity/type
checking、admissibility、evidence、truthをlater ownerへpositiveにdeferする。
current source、fixture、expectation、trace row/status、plan 411/372、type
238/226、pass/fail 224/187、active type 190、warnings/errors 23/0、全hashは不変。
blocking `spec_gap`、`source_undocumented_behavior`、current
`test_expectation_drift`、`boundary_violation`、`repo_metadata_conflict`はない。

## Task 250 implementation audit addendum

public `source_attribute` moduleはfrozen five-table syntax-free handoffを実装し、
`TypedAst`/`ResolvedTypedAst`はsyntax dependencyを追加せず保持する。exact real
route 4件、synthetic prefix extractor、producer corruption matrix、
deterministic rendering、immutable final-handoff assertionはbounded
`test_gap`とraw chain/qualification/argument/provenance `source_drift`をcloseする。
new exact covered trace rowが変えるのはplan/type coverageだけで411/373・
239/227となる。semantic attribute instance、evidence、accepted
fact/declaration/proof、downstream IRへのcreditは与えず、これらのgapはTasks
251+とexisting ownerに残る。

## Task 251 frozen-contract audit addendum

fresh inventoryはnext executable evidence-request transport sliceをChapters
03/05-08/13/17/19とMC-G016/MC-G018/MC-G026配下のTask 251へassignした。paired
crate planはdense syntax-free request/response-reference table 2件、transport-only state
4件、later `ExistentialGateInput` association用opaque attributed request
identity、immutable `TypedAst`/`ResolvedTypedAst` ownership、exact corruption
boundary、representative Task-249-broad + Task-84/85 selectorをfreezeする。

exact current oracleはmode-expansion 5、structure-inhabitation 3、attributed
2のrequest 10件。real
responseは全件missingでdependency referenceはpublishしない。requested/rejected/
supplied stateはreal `.miz` extraction後に同じproduction Task-10 pathへだけinject
し、final handoffまでassertする。suppliedはreference arrivalだけを意味し、later
semantic ownerがreferent/statusを独立にvalidateする。imported symbol shellを
evidenceとして扱わない。

このdocs prerequisiteがcloseするのは`design_drift`だけである。implementation
までexecutable absenceは`test_gap`、request/reference/final-handoff seamは
`source_drift`のまま。current source/fixture/expectation/trace row/status/coverage
credit、plan 411/373、type 239/227、pass/fail 224/187、active
parse/declaration/type/proof 101/5/190/1、warnings/errors 23/0、全hashは不変。
blocking `spec_gap`、`source_undocumented_behavior`、current
`test_expectation_drift`、`boundary_violation`、`repo_metadata_conflict`はない。

## Task 252 frozen-contract audit addendum

fresh inventoryはmissing exact primary-term contractを`design_drift`、
syntax-free producer/final handoff欠落を`source_drift`、producer/corruption
coverageとreal constant/`it` composition欠落を`test_gap`と分類する。Chapter 04/
13はbinding category、`it` role、numeral、transparent parenthesisを既に定義し、
blocking `spec_gap`はない。current `source_undocumented_behavior`、
`test_expectation_drift`、`boundary_violation`、`repo_metadata_conflict`もない。

paired crate planはpublic 3-table source-only modelとexisting numeral equality、
reserved-variable equality、parenthesized reserved-variable equality consumerを
freezeする。future aggregate oracleはterm 7/reference 4/numeric request 2である。
parenthesisはsource child edgeだけを保持し、semantic term/type entry/fact/axiom/
FOL nodeを追加しない。numeric requestはresult/factをpublishしない。

constant/`it` schema validationはTask 252で意図的にsyntheticとする。real
`LocalAbbreviation` productionはTask 269、real `func ... means`/`property ...
means` current-result owner/typeはTasks 260/264がretainし、Task 252はdependencyを
guessしない。このprerequisiteはsource/fixture/expectation/trace row/status/
count/hash/executable coverageを変更しない。current baselineはplan 411/374、
type 240/228、pass/fail 224/187、active parse/declaration/type/proof
101/5/190/1、warnings/errors 23/0。implementationだけがbounded covered
requirementを追加し、no-new-case oracle 411/375、241/229へ進める。

MC-G017/MC-G020はpartialのままである。Tasks 253-260/264/269がapplication、
other term、formula、definition、real local binding、semantic resultをretainする。
numeric type selection、theorem fact、accepted fact/declaration/proof、downstream
IR、Steps 6/7はTask-252 prerequisite creditを得ない。

## Task 252 contract-correction audit addendum

implementation inventoryでpost-freeze `design_drift` 1件が見つかった。reference
useをbinding declarationと同じordinal streamへ数えるruleは、useがlater
declarationより前にあるときTask 248のbinding-table
`visible_after_ordinal` semanticsと両立しない。同ruleはsame-priority duplicate
binding groupも不可能にし、要求済みのreachable `Ambiguous` rejectionをtest
できなかった。

corrected contractはdeclaration range endがreference start以前であるbinding
rowだけを数える。previous referenceはordinalを進めない。normal declaration
groupはsource orderを保ち、dense binding id/index位置のrow 1件を持つ。exact
consecutive duplicate-priority groupはspelling/kind/owner context/
`BinderIdentity`/declaration rangeを共有し、group final dense row indexを
visibility ordinalとして使う。`BindingEnv::lookup`が`Ambiguous`としてreject
するまでこのgroupを保持する。
`BindingLookupSite::new`はresolver payloadを持たないため、このproducer pathで
`Resolver`はstructurally unreachableであり、reachableな全non-local resultは
引き続きrejectする。

correction commit時、このchangeはspecification、`.miz`、expectation、trace
row/status、owner、deferred boundary、source、test、count、hash、coverage creditを
変更しなかった。remaining executable absenceは`source_drift`/`test_gap`の
ままだったが、下記implementation addendumがそのstatusをsupersedeする。
`spec_gap`、
`source_undocumented_behavior`、`test_expectation_drift`、
`boundary_violation`、`repo_metadata_conflict`は発生していない。Task-251
baselineとTask-252 implementation oracleは不変である。

## Task 252 implementation audit addendum

public syntax-free producer、exact private three-route consumer、immutable
`TypedAst` ownership、clone-only `ResolvedTypedAst` preservationがcorrected
Task-252 contractを実装した。real aggregateはterm 7 row、authenticated binding
reference 4 row、unresolved numeric-type request 2 rowである。synthetic
constant、`it`、nested-parenthesis、mixed-family probeはcorpus creditやsemantic
acceptanceを追加せずfrozen dependency boundaryをexerciseする。

implementationはbounded primary-term producer/final-handoff `source_drift`と
producer/corruption `test_gap`をcloseし、earlier `design_drift`はresolvedのまま
である。MC-G017/MC-G020はpartialのままであり、application/other term family、
numeric result、formula/definition semantics、real local binding/current-result
owner、accepted fact/declaration/proof、downstream IR、Steps 6/7はexplicit owner
に残る。blocking `spec_gap`、`source_undocumented_behavior`、
`test_expectation_drift`、`boundary_violation`、`repo_metadata_conflict`は
foundされなかった。
