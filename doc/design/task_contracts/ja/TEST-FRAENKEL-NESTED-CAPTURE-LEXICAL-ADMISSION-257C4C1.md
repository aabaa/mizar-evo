# Task TEST-FRAENKEL-NESTED-CAPTURE-LEXICAL-ADMISSION-257C4C1: explicit-import lexical admission

> canonical English: [English contract](../en/TEST-FRAENKEL-NESTED-CAPTURE-LEXICAL-ADMISSION-257C4C1.md)。本書はlogical synchronized Japanese companionである。

Owning plans: [mizar-checker](../../mizar-checker/ja/00.crate_plan.md#task-index)と
[mizar-test](../../mizar-test/ja/00.crate_plan.md#task-index)。

Stable owner sections: checker [source/spec classification](../../mizar-checker/ja/source_spec_audit.md#task-257c4c1-explicit-import-lexical-admission)、
[TODO](../../mizar-checker/ja/todo.md#task-257c4c1-explicit-import-lexical-admission)、
[bilingual record](../../mizar-checker/ja/bilingual_sync_audit.md#task-257c4c1-frozen-contract-parity)。
mizar-test [harness](../../mizar-test/ja/harness.md#task-257c4c1-private-import-provider)、
[boundary](../../mizar-test/ja/module_boundary_audit.md#task-257c4c1-crate-local-testdata-boundary)、
[corpus](../../mizar-test/ja/miz_corpus.md#task-257c4c1-explicit-import-corpus-repair)、
[traceability](../../mizar-test/ja/traceability.md#task-257c4c1-inactive-trace-repair)、
[TODO](../../mizar-test/ja/todo.md#task-257c4c1-lexical-admission-prerequisite)、
[bilingual record](../../mizar-test/ja/bilingual_sync_audit.md#task-257c4c1-frozen-contract-parity)。

## Status, authority, readiness

**Status:** implementation lifecycleはaccepted explicit-import lexical-repair STOPで
closed、semantic successorはselectしない。Userはdedicated explicit-import test-intent
repairだけを明示承認した。Required spelling 2件をone separate fixture moduleで
defineし、existing inactive C4C0 oracleからimportし、frontend lexical/parser
admissionとpreprocessed import provenanceだけをproveしてstopする。Capture/semantic
workはauthorizeしない。

Authority orderは次の通り。

1. canonical [Chapter 2 §2.10](../../../spec/en/02.lexical_structure.md)、
   [Chapter 3 §3.3](../../../spec/en/03.type_system.md)、
   [Chapter 7 §7.2](../../../spec/en/07.modes.md)、
   [Chapter 10 §10.1](../../../spec/en/10.functors.md)、
   [Chapter 11 §§11.2.1/11.2.4](../../../spec/en/11.symbol_management.md)、
   [Chapter 12 §§12.3/12.5](../../../spec/en/12.modules_and_namespaces.md)、
   [Chapter 13 §§13.4.2/13.4.4/13.8.6](../../../spec/en/13.term_expression.md)。
2. completed C4C0 oracleからderivedした、下記approved exact imported `.miz` test intent。
3. existing C4C0 trace row。
4. existing inactive C4C0 expectation sidecar。
5. completed R2/C4A/C4B/C4C0 contract、design records、current private
   provider/source observation。

Chapter 12はimport preludeをfile-scopedとしpublic definitionをmodule signatureに
含める。Chapter 13はnested occurrenceがouter generatorをresolved identityで参照する
ことを要求する。本repairは既存frozen sourceをadmitするimported lexical spelling
2件だけを供給する。Task freeze時点では、then-current parser diagnostics 6件を
`source_drift`、then-absent dedicated support module/testを`test_gap`、then-absent
provider/owner recordを`design_drift`とclassifyした。このsliceにblocking `spec_gap`、
expectation rebaseline、public-API `boundary_violation`はなかった。下記exact C4C1
implementation/post-commit proofがclassified gaps 3件をすべてcloseした。

Task freeze時点では、C4C0 inactive 124-byte historical oracle、explicit canonical
import/public-definition rules、existing frontend/provider one-stub/one-summary seamsにより、
exact synchronized docs prerequisiteのreview/separate commit後だけC4C1がreadyだった。
このpreimplementation readiness statementはhistoricalであり、prerequisite/
implementation/post-commit proofはcomplete、lifecycleは下記accepted STOPでclosed。

## Exact support module and exported summary

Implementationはordinary corpus discovery外のcrate-local testdata file 1件だけを追加する。

```text
crates/mizar-test/tests/testdata/parser/nested_capture_fixtures.miz
```

Logical module identityはexact `parser.nested_capture_fixtures`。Final-LF sourceはexact：

```mizar
definition
  let S be set;
  public mode ElementDef: Element of S is set;
end;

definition
  public func NatDef: NAT -> set equals {};
end;
```

`140` bytes、SHA-256
`dd721a48620f985d5612cc718a94aef576e87d616c239712b8deb2d65c84a11c`。
Exact anchorsは`S@17..18`、definition label
`ElementDef@41..51`、exported mode `Element@53..60`、locus `S@64..65`、
definition label `NatDef@105..111`、exported functor `NAT@113..116`、body
`{}@131..133`。Separate definition blocksによりmode locus `S`がzero-arity
`NAT` functorをcontaminateしない。これはtest-owned support moduleであり、
ordinary corpus case/sidecar/trace row/active route/independent creditは持たない。

C4C1 harnessがこのexact crate-local testdata sourceをsynthetic
`ModuleId("parser.nested_capture_fixtures")`へassociateし、crate-private providerは
そのexact module idだけをrecognizeする。Frozen testsがphysical sourceとsummaryを
cross-validateする。これはpackage/MML/implicit-prelude/production source-manifest/
general file-resolution behaviorではない。

Sole import stub ordinal `0`はfingerprint `LexicalSummaryFingerprint(1)`のone
summaryとexact ordered full-field shapes 2件をresolveする。

| rank | spelling | symbol id | source module | kind | arity | operator |
|---:|---|---|---|---|---|---|
| `0` | `Element` | `parser.nested_capture_fixtures#parse-only#Element` | `parser.nested_capture_fixtures` | `Mode` | exact `1` | `None` |
| `1` | `NAT` | `parser.nested_capture_fixtures#parse-only#NAT` | `parser.nested_capture_fixtures` | `Functor` | exact `0` | `None` |

`parser.type_fixtures`と全unrelated moduleのexisting summary bytes/rank/identity/
fingerprintは不変。Shapesをold moduleへappend、multiple module synthesize、approximate
spellingへのprovider broadeningは禁止。

## Exact repaired C4C0 source and frontend provenance

Existing corpus sourceはexact次へ変わる。

```mizar
import parser.nested_capture_fixtures;

definition
  func NestedCapture -> set equals
    { { x where y is Element of NAT }
      where x is Element of NAT };
end;
```

Final-LF `164` bytes、SHA-256
`c3b8bd62c16406ccedee2e64a71ef62a5c4b329d2319be33ad3834a9541af431`。
One import stub/one summary。Significant rangesはmodule path `7..37`、
`NestedCapture@58..71`、inner mapper `x@94..95`、inner generator `y@102..103`、
first `Element@107..114`/`NAT@118..121`、outer generator `x@136..137`、
second `Element@141..148`/`NAT@152..155`。

Preprocess outputはexact one `ImportStub`を持つ：stub span `7..37`、path
spelling `parser.nested_capture_fixtures`、path span `7..37`、components
`parser`/`nested_capture_fixtures`、raw source segment一つ`7..37`、relative
prefix/aliasなし。parsed ASTは`ImportAliasDecl@7..37`、component ranges
`parser@7..13`/`nested_capture_fixtures@14..37`、および周囲の
`ImportItem@0..38`を保持する。Providerはordinal `0` stubを上記exact
one-summary/two-shape lexical environmentへ
resolveし、real frontend ASTはdiagnostic/recovery 0。本taskはresolver import
augmentationをproduction routeでrun/extendせず、下記validation-only no-op testだけが
direct callする。Resolver imported identity、use-site name reference、
generator identity、capture readiness、type application、sethood、formula semanticsは
separately deferred。

Historical no-import sourceはfourth regression内でexact final-LF `124` bytes、
SHA-256 `f1a35d2d7f6cb4a57ece3b1143a68c1a01ab9ac478960862057025cc9838cea7`
として保持する。Exactly parser diagnostics 6件、first `Element@67..74`を維持し、
imported-summary leakageは0。Repaired corpus fileがhistorical bytesをsupersedeするのは
上記explicit test-intent authorityだけによる。Implicit prelude/builtin substituteを
推論しない。

## Existing API ownership and exact implementation scope

Public API deltaなし。`mizar-frontend::lexical_env`が
`LexicalEnvironmentRequest`、`LexicalSummaryProvider::resolve_imports`、
`ResolvedImports`、provenance validation、active-environment assemblyをownする。
`mizar-lexer`が`ModuleLexicalSummary`、`ExportedSymbolShape`、`ExportRank`、
`UserSymbolKind`、`UserSymbolArity`、`LexicalSummaryFingerprint`をownする。
C4C1はcrate-private `ParseOnlyImportProvider` summary branchをextendし、resolver
augmentationを`parser.type_fixtures`だけに保つexact private discovery guardを追加する。
C4C1 testはcanonical C4C1 ASTが`SymbolEnv`をbyte-for-byte unchangedにすることだけを
証明するためにそのseamを呼ぶ。このvalidation-only exclusionはresolver creditを
与えず、existing `augment_type_elaboration_import_summaries` allowlistと全resolver
augmentation outputはbyte-identical。

Docs prerequisite commit後のimplementationはexact 7 paths：

```text
crates/mizar-test/tests/testdata/parser/nested_capture_fixtures.miz
tests/miz/pass/types/pass_types_nested_comprehension_outer_generator_capture_001.miz
tests/miz/pass/types/pass_types_nested_comprehension_outer_generator_capture_001.expect.toml
tests/coverage/spec_trace.toml
doc/design/spec_coverage_audit.md
crates/mizar-test/src/runner/import_fixtures.rs
crates/mizar-test/src/runner/tests/parse_only.rs
```

New test leaf/`tests.rs` includeはない。Existing `parse_only.rs`へexact tests 4件：

1. `task257c4c1_physical_fixture_declarations_are_exact`;
2. `task257c4c1_provider_summary_is_exact_and_unrelated_modules_are_isolated`;
3. `task257c4c1_canonical_imported_source_is_zero_diagnostic_with_exact_frontend_provenance`;
4. `task257c4c1_historical_no_import_source_retains_six_diagnostics_without_leakage`。

Physical bytes/hash/declaration ranges/zero-diagnostic declaration surface、全summary
field/one-stub-one-summary fingerprint/unrelated isolation、repaired source bytes/hash/
zero frontend-parser diagnostics/exact preprocessed stub/provider summary/AST/no recovery、historical source/hash/6 diagnostics/
no leakageを順にpinする。Raw mizar-test library testは`614 -> 618`。Public runner
command/case dispatchは追加しない。

## Sidecar, trace, audit, and count impact

Existing sidecarはschema `1`、kind `pass`、stage `advanced_semantics`、domain
`set_expressions.nested_capture`、outcome `pass/type_check`、empty diagnostics、sole
spec ref、no active tags/failure fieldsを保持する。Noteだけがexplicit imported
lexical/parser admission complete、capture transport/execution/Task277B deferredを記録する。

Existing sole requirement `spec.en.13.set_expressions.nested_capture.semantic`は
`advanced_semantics/covered/required/pass`とsole sidecarを保持する。Dependencyはexact
`spec.en.13.set_expressions.parser`のまま。New requirement/backlinkなし。Row noteは
Chapter-12 import authorityとlexical-admission boundaryをrecordする。Coverage auditは
Chapter-13 rowだけでexact lexical/import blocker closureを記録し、resolver/checker
capture/advanced runnerはfollow-upのまま。
Status/semantic creditは変えない。

Clean baseline HEAD `d93fa7133e77e070d4ba0d016a1c3519ae80dd4b`、
`origin/main...HEAD=0/27`、clean worktree、protected stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4` unchanged。Baselines：

- corpus pairs `344/344` unchanged。Support fileは`tests/miz`外、C4C0 pairはin-place update。
- contract trees `90/90 -> 91/91` after prerequisite。
- trace `5924` lines / SHA-256 `d1df314665998fe5271a73d7102b6e6d6098fd6636d78e2a6ded779d5f44cbae`。
- audit `7005` lines / SHA-256 `99720173f84f1713ed2bf63e9806566b2aa6a904d18d6855b20544bab96928a5`。
- sidecar SHA-256 `2c7d987baa988b9ea1ae179d6ed1a3b9c8df334694cdd9d43626342647d59701`。
- `import_fixtures.rs` `469` lines / SHA-256 `991c400cedb084fe9cd4b59a17be068d843ce806f1fae74c49c22208970e400f`。
- `parse_only.rs` `111` lines / SHA-256 `3cddce85155b72597cfc4c2ea5841dbf3fe5f88d0c8123d98ba9cb958f90a3a8`。
- library `614`、metadata `137/137`、cases/requirements `429/396`、pass/fail
  `236/193`、active routes `101/7/205/1`、aggregate warnings/errors `23/0`。

Implementation時にRust 2 files、support/source/sidecar/trace/audit hashes、raw test list、
metadata、CLI stdout 5本をremeasureする。Expected deltaはtest countとinactive artifact/doc
bytesだけ。Corpus pair/metadata/active route/warning-error/executable-stage countsは不変で、
replayなしのunchanged claimは禁止。

## Precommit implementation completion checkpoint

Precommit completion checkpointではexact seven-path implementationがthen-current
worktreeでcompleteだった。Private support module/provider profileを追加し、explicit importでinactive oracleをrepairし、許可された
sidecar/trace/Chapter-13 audit deltaだけをrecordし、frozen tests 4件を追加した。Measured
artifact/sourceは次の通り。

- support fixture：`8` lines / `140` bytes / SHA-256
  `dd721a48620f985d5612cc718a94aef576e87d616c239712b8deb2d65c84a11c`。
- repaired source：`7` lines / `164` bytes / SHA-256
  `c3b8bd62c16406ccedee2e64a71ef62a5c4b329d2319be33ad3834a9541af431`。
- inactive sidecar：`13` lines / SHA-256
  `9ed000a30c1d519bd665f338c636fb9e529e9848a285209bebe6728f19961b92`。
- trace：`5924` lines / SHA-256
  `d4d817e83aac78d19e729702b26c62604fc57581eec18672a5c26ec44efe7a81`。
- coverage audit：`7005` lines / SHA-256
  `931f0e4eb8d95998838dc115b8f3ad2a4f5c452e0e5eb0b384cc8d8d5351ac39`。
- `import_fixtures.rs`：`490` lines / SHA-256
  `b6e4112848710c75be254d9eff7fbcf5e8d5029d3e0544b7c790549dc9ef880b`。
- `parse_only.rs`：`454` lines / SHA-256
  `c73ede41962ecf135b18ceb6b020e35a6eb741d4795578bbf42b882e61079c59`。
- raw mizar-test library list：`618` tests / SHA-256
  `d145e5bf5c8ae3f8231ffe73ee034b639001d349c99dd4f00f3c60b6382db4c1`。

Focused tests 4件とmizar-test library `618/618`はPASS。Metadata `137/137`、checker/
mizar-test lint-policyは各`15/15`、`cargo fmt --all -- --check`、full-workspace
`cargo clippy --all-targets --all-features -- -D warnings`、full `cargo test`はPASS。
Replayed plan/parse/declaration/type/proof CLI stdout SHAは順に
`2d2accef2c6fc32c1b3372530f6136af1299ac6ae7db6a0158798336b779c7e7`、
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`、
`71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`、
`4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`、
`ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`、
warnings/errorsは`23/0`。Corpus/metadata/executable-stage/active-route inventoryは不変。

Independent test-sufficiency/implementation reviewは**NO FINDINGS**。本exact `20`-path
Markdown completion-documentation deltaはpaired contractとexisting checker/
mizar-test EN/JA owner docs 18件だけで、crate plans 4件はunchanged。Source-doc
consistencyとbilingual/boundaryのindependent reviewも**NO FINDINGS**。Independent
final-quality reviewも**NO FINDINGS**、hard gates `9/9` PASS、valid
uncapped score `100/100`、split `20/20/15/15/10/10/5/5`。Task-only commit、
post-commit proof、fresh inventoryは下記historical checkpointでclosed。本taskは
lexical-admission repairでstopし、resolver/capture/semantic/Task277B creditを与えず、
successorをselectしない。

Final precommit lifecycle sync前のhistorical staging-review checkpointでは、cached scopeは
exact `27` paths（seven implementation paths + twenty completion-doc paths）、new
support `.miz`はexact one、review-time unstaged pathsはzero、
`git diff --cached --check` PASS、cached statは`664` insertions / `41` deletionsだった。
Exact staging/cached reviewは**NO FINDINGS**。これはreview-time historical factであり、
current `HEAD`/worktreeがunstaged zeroというclaimではない。

## Historical immediate-postimplementation pre-closure checkpoint

Task-only implementation commit直後かつ本docs-only closure前に、commit
`6371b9983efb1899cb4d4de28fe0bebfcaf47381`を`HEAD`としてobserveし、parentは
`f0845558c9b35af315462af0e1e60faf3770f62b`だった。Worktree clean、
`origin/main...HEAD=0/4`、protected `stash@{0}`は
`f65cf4a13752ec380710814a9ac6392ccb9d75d4` unchanged。Commit pathsはexact `27`、
sorted path list SHA-256は
`26aa22a7ae0fbcd98692bd01252922e7eb56fb4ef5a31c51160889383ebfc3dc`。
Final statは`716` insertions / `41` deletions、
`git show --check 6371b9983efb1899cb4d4de28fe0bebfcaf47381`はPASS。

このfinal commit evidenceは上記historical review-time cached `27` paths / `664`
insertions / `41` deletionsとdistinct。Immediate-postimplementation/pre-closureの
observationであり、本docs update後のcurrent `HEAD`/worktree claimではない。
Task-only implementation commitとpost-commit proofはclosed。

Fresh inventoryはuser-approved protocol disposition **STOP**でcloseする。Explicit-
import lexical repairはcompleteだが、本taskはcapture/resolver-semantic successor、
Task252 work、Task277B workをselectしない。Capture transport、resolver semantics、
Task252、Task277Bはtask外で、Task277Bはnot-ready/zero creditのまま。

## Scope, prohibitions, reviews, and exit

Prerequisiteはexact 24 Markdown pathsだけを変更する：本contract pair、checker/test
EN/JA crate plans 4件、checker EN/JA `source_spec_audit.md`/`todo.md`/
`bilingual_sync_audit.md` 6件、mizar-test EN/JA `harness.md`/
`module_boundary_audit.md`/`miz_corpus.md`/`traceability.md`/`todo.md`/
`bilingual_sync_audit.md` 12件。Rust/artifact/canonical spec/legacy manifest/protected
audit-ledger/staging/commitはdocs prerequisite外。

Forbidden/deferred：

- `doc/spec`変更とsourceからのlanguage semantics推論なし。
- frontend/parser/resolver/checker public API/production semantic routeなし。
  Chapter-2/3/7/10/11/12 coverage creditなし。
- new corpus case/sidecar/trace requirement/active tag/failure field/diagnostic
  expectation/warning-error/route creditなし。
- `SourcePrimaryTerm`、term/use row、Task252、`CapturedFreeVariables`、role enum
  duplication、binding/capture table、formula/type/sethood request/verdict/diagnostic、
  Typed/Resolved install、dispatch/executionなし。
- F5/R2/C4A/C4B/protected legacy/schema-v2 compaction dataはedit/reinterpretしない。
- Task277Bはnot-ready/semantic credit zero。

Docs exitはexact24 scope、EN/JA parity/recursive link、checker/test両lint-policy、
`git diff --check`、protected anchor/schema-v2 stability、independent final-quality 9 gates
PASS/valid `>=90/100`、task-only docs commit、clean fresh implementation preflightを要求する。

Implementation exitはexact tests 4件、mizar-test library/full workspace tests、format、
package/full-workspace Clippy、metadata、CLI 5本、両lint、diff/scope、independent test/
implementation/source-doc/bilingual-boundary/final-quality NO FINDINGS、inactive zero-credit
reproofを要求する。Immediate successorはfresh inventoryだけで、capture semanticsは
separately select/freezeする。

Recommended routingはauthority/contract/scope expansion/final scoringをGPT-5.6 Sol
`xhigh`。Frozen後のbounded crate-private provider/test/artifact implementationとindependent
reviewはGPT-5.6 Terra `xhigh`。Module identity/symbol shape/arity/rank/frontend provenance/
diagnostic profile差異またはsemantic expansion要求はedit前にSolへ戻す。
