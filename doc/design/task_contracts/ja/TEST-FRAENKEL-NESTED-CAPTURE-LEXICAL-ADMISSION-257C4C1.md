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

**Status:** dependency-ready documentation prerequisite。Userはdedicated
explicit-import test-intent repairだけを明示承認した。Required spelling 2件をone
separate fixture moduleでdefineし、existing inactive C4C0 oracleからimportし、
frontend lexical/parser admissionとpreprocessed import provenanceだけをproveしてstopする。
Capture/semantic workはauthorizeしない。

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
2件だけを供給する。Current parser diagnostic 6件は`source_drift`、dedicated support
module/test欠如は`test_gap`、provider/owner record欠如は`design_drift`。Blocking
`spec_gap`、expectation rebaseline、public-API `boundary_violation`は残らない。

C4C0、canonical import/public-definition rule、current one-stub/one-summary
frontend/provider seamが揃っているため、exact synchronized docs prerequisiteのreview/
separate commit後にC4C1はreadyである。

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

Preprocess outputはexact one `ImportStub`を持つ：declaration span `0..38`、path
spelling `parser.nested_capture_fixtures`、path span `7..37`、components
`parser@7..13`/`nested_capture_fixtures@14..37`、relative prefix/aliasなし。
Providerはordinal `0` stubを上記exact one-summary/two-shape lexical environmentへ
resolveし、real frontend ASTはdiagnostic/recovery 0。本taskはresolver import
augmentationをrun/extendしない。Resolver imported identity、use-site name reference、
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
C4C1はcrate-private `ParseOnlyImportProvider` summary branchだけをextendする。
Existing `augment_type_elaboration_import_summaries` allowlistと全resolver
augmentation behaviorはbyte-identical。

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
