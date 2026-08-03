# 二言語ドキュメント同期監査

> 正本は英語です。英語版:
> [../en/bilingual_documentation_synchronization.md](../en/bilingual_documentation_synchronization.md)。

状態: task R-028 audit complete; task R-029 and close-out scopes re-run complete;
2026-07-02 roadmap synchronization overlay complete; task R-024 implementation
overlay complete; R-032A implementation synchronization complete。R-032B
implementationは`b3a7e79a6b60db2974e911c69bb56ff5f4609064`でcommit済みで、
Checker Task 258B5Cも`33ac57e96f048dc40559565f54369cac854409a7`としてcommit済みの
historical completed task。

## 範囲

この監査は、`doc/design/mizar-resolve/en/` の各英語正本 design document と
`doc/design/mizar-resolve/ja/` の日本語 companion を比較する。確認対象は paired
filename、API list、task status、gap / deferred classification、behavior promise、
boundary statement、terminology、`mizar-resolve` task stream に関係する link である。

監査範囲は close-out までの完了済み non-deferred resolver work、当初の R-024
`external_dependency_gap` deferral record、artifact 側 blocker を解消済みとして記録した
2026-07-02 roadmap synchronization update、そして R-024 resolver-side implementation
overlay である。この監査は
[source_spec_correspondence.md](./source_spec_correspondence.md) の source/spec
correspondence audit を置き換えない。また、`doc/spec`、`.miz` source、expectation
sidecar は変更しない。

## 結果

- 現在の英語 design file はすべて同名の日本語 companion を持ち、この監査も両言語
  directory に同じ paired file として追加した。
- public resolver API family、public enum forward-compatibility decision、task
  completion state、deferred / external dependency record、milestone handoff wording に
  残る英日不一致は見つからなかった。
- task status は、artifact 側 `external_dependency_gap` の解消後に実装された R-024 を含め、
  R-001〜R-029 完了として同期している。
- 既存 follow-up classification は同期している: R-G001 `spec_gap`、R-G002
  `test_gap`、R-G003 R-024 で解消済み、R-G004 `boundary_violation` risk、
  R-G005 resolved `design_drift`、R-G006 `external_dependency_gap`、そして R-G002 の
  現在の具体的な精緻化である R-G007 `test_gap`。
- implemented R-032A / R-032B pair は Medium normal-source proof-label `source_drift`、stale R-023
  attribution `design_drift`、R-G007 B5C `test_gap`、Low deferred R-G001
  public-code `spec_gap` を同期する。
- この監査により新しい `spec_gap`、`test_gap`、`design_drift`、`source_drift`、
  `source_undocumented_behavior`、`test_expectation_drift`、`boundary_violation`、
  `repo_metadata_conflict` は導入されていない。

## pair checklist

| 英語正本 document | 日本語 companion | 同期結果 |
|---|---|---|
| [00.crate_plan.md](../en/00.crate_plan.md) | [./00.crate_plan.md](./00.crate_plan.md) | responsibility、inventory、gap table、completed extension、historical pre-S-026 four-task record、effective S-026-docs/S-026-implementation/R-032A-lint-docs/R-032A-implementation/R-032B-lint-docs/R-032B-implementation/B5C seven-task order が同期。 |
| [declarations.md](../en/declarations.md) | [./declarations.md](./declarations.md) | declaration shell kind、excluded / transparent node、visibility、recovery、identity / provenance、public enum policy が同期している。 |
| [env.md](../en/env.md) | [./env.md](./env.md) | `SymbolEnv` index family、contribution tracking、invalidation note、determinism、public enum policy が同期している。 |
| [imports.md](../en/imports.md) | [./imports.md](./imports.md) | import input/output、two-pass contract、path resolution、alias / export / cycle / unresolved policy、determinism、boundary note、public enum policy が同期している。 |
| [labels.md](../en/labels.md) | [./labels.md](./labels.md) | existing label policy、committed R-032B API/subtree/origin/error contract、active private B5C consumer statusが同期。 |
| [module_summary_reuse.md](../en/module_summary_reuse.md) | [./module_summary_reuse.md](./module_summary_reuse.md) | R-024 summary reuse scope、known-field identity validation、fallback policy、source-backed agreement、determinism、public enum policy が同期している。 |
| [names.md](../en/names.md) | [./names.md](./names.md) | name-use site、scope model、namespace-before-symbol lookup、visibility / shadowing、unresolved / ambiguous record、dot-chain finalization、diagnostics、public enum policy が同期している。 |
| [recovery.md](../en/recovery.md) | [./recovery.md](./recovery.md) | recovered syntax stage disposition、boundary rule、test intent が同期している。 |
| [resolved_ast.md](../en/resolved_ast.md) | [./resolved_ast.md](./resolved_ast.md) | top-level `ResolvedAst` shape、stable identity、node / name / label / import table、recovered shell、provenance、determinism、public enum policy が同期している。 |
| [source_spec_correspondence.md](../en/source_spec_correspondence.md) | [./source_spec_correspondence.md](./source_spec_correspondence.md) | existing audit、implemented R-032A/R-032B correspondence、active B5C consumer statusが同期。 |
| [symbols.md](../en/symbols.md) | [./symbols.md](./symbols.md) | symbol-bearing shell、collection order、identity / origin、signature、duplicate / overload、visibility / export / summary policy、dependency relation、recovery / diagnostics、determinism、public enum policy が同期している。 |
| [todo.md](../en/todo.md) | [./todo.md](./todo.md) | ordered task state と split R-032A/R-032B ownership/dependency が同期。 |
| [bilingual_documentation_synchronization.md](../en/bilingual_documentation_synchronization.md) | [./bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md) | この R-028 audit、R-029 scope re-run、close-out re-run、roadmap synchronization overlay は、同じ scope、result、pair checklist、handoff note を両言語で記録している。 |
| [module_boundary_refactor.md](../en/module_boundary_refactor.md) | [./module_boundary_refactor.md](./module_boundary_refactor.md) | R-029 source-layout audit、private helper / test split list、re-run audit note、verification requirement、bounded R-032 ownership recheck が同期している。 |
| [crate_exit_report.md](../en/crate_exit_report.md) | [./crate_exit_report.md](./crate_exit_report.md) | close-out status、quality score、hard gate、deferred item、human-review surface、verification、task commit、next-task handoff、planned R-032 extension が同期している。 |

## R-031 pair recheck

R-031はpaired plan、TODO、symbols design、source correspondence、close-out extensionを
再確認する。両言語は同じordinary-functor-only syntactic key、appendした
`SameSignatureDefinitionConflict` diagnostic / definition variant、exact
`same_signature_definition_conflict` SymbolEnv snapshot spelling、exact declaration-symbol
detail key、mixed-group priority、candidate/range/order behavior、sidecar/trace transition、
coverage impact、禁止するsemantic/public-code/phase boundaryを記録する。R-031 extensionに
bilingual driftは残らない。

## R-032A / R-032B pair recheck

paired docs は同じ historical pre-S-026 four-task record、effective
S-026 docs -> S-026 implementation -> R-032A lint-policy docs correction ->
R-032A implementation -> R-032B lint-policy docs correction ->
R-032B implementation -> active B5C
seven-task order/classification、R-032A arena API/error variant/derive、
R-032B collector API/error variant/derive、`u32` overflow、
file ownership、collector lifetime/storage/module rule、theorem-root、
module-global ordinal/completion、exact length-framed `proof-step-v1` grammar、
B5C origin path、subtree/exclusion、cross-theorem direction、own-proof boundary、
mutation matrix、private key、forbidden change を freeze する。R-032A
resolution-state/reference-key mismatch variant も同期する。R-032A arena
origin `[surface_id]` と R-032B richer table origin の意図的差も同期する。
exhaustive default-deny direct Surface edge table semantics と
positive-per-edge/negative mutation/mixed-list/representative all-other test
obligation も同期する。upper hierarchy は同じ `Root` -> `CompilationUnit` ->
`ItemList` -> direct theorem で、missing/additional/wrong/relocated/wrapped test
も同期する。
rejected callback/unmapped contract は両言語に残さない。

## handoff

post-close-out の resolver update は、この監査を二言語同期状態の baseline として扱う。
S-026 documentation/implementation、R-032A lint-policy docs correction、
R-032A implementationとR-032B dedicated commitは完了済み。historical B5Cも
`33ac57e96f048dc40559565f54369cac854409a7`で完了し、current dependency stepは
fresh Task-263 preflight後のTask 263R。将来 design fileを
追加する場合は両言語 directory に同時に追加する。挙動 cleanup、public API
change、新しいdiagnosticsは完了済み resolver milestone の範囲外であり、
独立した spec/test authority を要求する。

S-026/R-032A dependency overlay は EN/JA 同期済み。両言語はhistorically
同じboundary defectを分類し、separate syntax commitまでR-032A sourceを
deferした。それらのcommit、lint-policy correction、R-032A implementationは
完了済み。resolver ownership、validation precedence、exclusionは同期したままで、
R-032B sourceはcommitted。historical B5C consumerのtest、implementation、
source/documentation reviewと全verification gateは完了。independent final
qualityは**NO FINDINGS**、全9 hard gates PASS、score capなし、valid
`100/100`。task-only cached-diff review、dedicated commit
`33ac57e96f048dc40559565f54369cac854409a7`、post-commit fresh inventoryも完了。

## R-032A lint-policy scope correction

EN/JAはomitted mandatory R-026 enum-decision ownerを同じHigh
`design_drift`として分類し、semantic `spec_gap`なしとする。later
implementationはexact `src/resolved_ast.rs`、
`src/resolved_ast/tests.rs`、`tests/lint_policy.rs`の3 filesで、last fileは
`SurfaceResolvedArenaError` owning-spec decision entryだけを受けられる。
runtime/API/test contractと全forbidden boundaryは維持する。このpaired
correctionはdocs-only separate prerequisite commitで、coverage stateを変更せず、
implementation前のfresh inventoryを要求する。

## R-032A implementation synchronization

EN/JAはimplemented `SurfaceResolvedArena` API、exact three-field ownership、
complete dense same-index lowering、fail-closed validation precedence、public
non-exhaustive error surface、helper payload、equivalent-input determinism、
sole R-026 decisionを同一に記録する。両言語は同じRust 3 ownerと
label/runner/artifact/trace/semantic prohibited scopeを記録する。その
prerequisite record時点ではR-032Bはpending。active mapping、trace
status/count、owner、deferral、coverage creditを
変更しないため`spec_coverage_audit.md`はdeliberate no-op。

## R-032B lint-policy scope correction（completed prerequisite record）

EN/JAはomitted mandatory R-026 enum-decision ownerを同じHigh
`design_drift`として分類し、semantic `spec_gap`、`test_gap`、test-intent
changeはない。later R-032B implementationはexact
`crates/mizar-resolve/src/labels.rs`、
`crates/mizar-resolve/src/labels/tests.rs`、
`crates/mizar-resolve/tests/lint_policy.rs`の3 filesで、last fileはsole
`ProofLabelSourceCollectionError` owning-spec decision
`spec_name: "labels.md"`だけを受けられる。

completed docs-only correctionはexact 31 design files、resolver 16、checker 8、
`mizar-test` 6、global ledger 1。semantic/API/test contractを保存し、source、
fixture、sidecar、expectation、trace status/count、Cargo metadataを変更しない。
coverage ownership/statusを変更しないため`spec_coverage_audit.md`はdeliberate
no-op。independent specification、test/scope、source/documentation
consistency reviewはすべて**NO FINDINGS**で、docs-only verification/count/hash
gateはPASS。independent final read-only qualityも**NO FINDINGS**で、全9
hard gates PASS、capなし、valid `100/100`
（`20/20/15/15/10/10/5/5`）。そのpre-commit record時点ではtask-only
staging/cached-diff review、commit、post-commit invariant/fresh-inventory
gateだけがpendingだった。これらはcorrection commit
`f1cf0a5d15f2db51176e9e91a4f5a6447a88ad7a`とそのfresh inventoryで後に完了。

## R-032B implementation synchronization

EN/JAは同じcommitted implementationを記録する。すなわちexact
three-Rust-file ownership、public collector、collection accessor、
non-exhaustive error table、R-032A validation、AST/arena-only borrow、
default-deny direct traversal、module-global ordinal、proof scope、completion
boundary、simple citation、structural origin、`proof-step-v1` identityである。
両言語ともprivate B5C consumerを維持し、checker handoff、semantic
phase、fixture、expectation、sidecar、trace state、public diagnostic、Cargo
metadataを除外する。

両言語はinitial High/Mediumとfresh 2件のMedium test gap、Medium third-child
implementation defect、Mediumのunauthorized `Default` / `From` findingsを
fixedとして記録する。preimplementation specification reviewとfinal fresh
test-sufficiency、implementation、source/documentation rereviewはすべて
**NO FINDINGS**。focused、crate、formatting、workspace Clippy/test、diff、
CLI、test-list、production、exact 20-file scope gateはPASS。independent final
qualityは**NO FINDINGS**、全9 hard gates PASS、score capなし、valid
`100/100`（`20/20/15/15/10/10/5/5`）。task-only
restaging/cached-diff review、commit
`b3a7e79a6b60db2974e911c69bb56ff5f4609064`、post-commit
invariant/fresh inventoryはcomplete。
`spec_coverage_audit.md`はactive mapping、trace status/count、owner、
deferral、coverage creditが変わらないためdeliberate no-opのまま。

## Checker Task 258B5C implementation synchronization

EN/JAは`33ac57e96f048dc40559565f54369cac854409a7`としてcommit済みの同じ
historical B5C implementationを記録する。unchanged R-032A
`SurfaceResolvedArena`とR-032B `ProofLabelSourceCollector` /
`LabelResolver` APIを`mizar-test`でprivateにconsumeし、exact fail fixture
2件、sidecar 2件、covered trace row 2件を追加する。
`crates/mizar-test/tests/metadata.rs`のfrozen active-count/CLI assertion
4件はdeclaration stage `5`から`7`へ更新する。
resolver production sourceとpublic APIは変更しない。

両言語ともplan `421/389`、pass/fail `228/193`、active
parse/declaration/type/proof `101/7/198/1`、warnings/errors `23/0`を記録する。
public diagnostic codeは空のままで、private route keyは
`declaration_symbol.label.proof_scope_confinement`。B5Cが閉じるのは
inner-to-outer/sibling confinement negativeだけで、R-G007は
import/name/dot-chain/other label-reference coverageについてopenのまま。
test、implementation、source/documentation reviewと全verification gateは
完了。independent final qualityは**NO FINDINGS**、全9 hard gates PASS、
score capなし、valid `100/100`。task-only cached-diff review、dedicated B5C
commit `33ac57e96f048dc40559565f54369cac854409a7`、post-commit fresh inventoryは
このhistorical checkpointで完了済み。

## Checker Task 263R frozen synchronization

EN/JAは同じChapter-5 authority、320-byte probe/hash、実測`75/10/8/8/2` profile、
selector-only nearest-structure conflict key、missing-owner fallback、same-owner
collision、two-file implementation scope、two-test intent、semantic exclusion、
unchanged executable count、docs/implementation two-commit sequenceをfreezeする。
両言語はlower defectを`source_drift` + `design_drift` + `test_gap`と分類し、origin
divergenceをreport-only `repo_metadata_conflict`として扱う。

## Checker Task 263R implementation synchronization

EN/JAは同じimplemented private nearest-structure selector owner、selector-only
conflict-key partition、conservative `None` fallback、unchanged non-selector/public
behavior、exact two-file scope、exact extractor-backed test 2件を記録する。両言語は
cross-owner `75/10/8/8/0` result、same-owner `30/4/3/3/1` control、resolver test
`146`、production `15/18896`、同じhash、corpus/trace/runner/checker/metadata
coverage deltaなしを記録する。両言語はfindings-free consistency/full verificationと
independent final quality全9 hard gate PASS、capなし`100/100`を記録する。Task 263へ
戻る前にdedicated implementation commit、fresh inventoryだけを残す。

## Checker Task 264R frozen synchronization

EN/JAのresolver plan、TODO、declarations、symbols、source/spec correspondence、
module-boundary recordは同じcontext-only `PropertyImplementation` shellをfreezeする。
両languageはChapter 7 placement、Chapter 13 means-only `it`、ad-hoc `assume`不在、
referenced-property return-type lookupのlater ownership、append-only enum/code/key、semantic-
sibling/`LocalSource` anchor stability、exact fixture hash/profile、implementation file 4件、
future test 2件、`146 -> 148`、corpus/checker/runner/trace/Cargo creditゼロを同期する。
一方の変更はreview PASS前に同じlogical updateをもう一方へ必要とする。
