# Module: miz_corpus

> Canonical language: English. English canonical version: [../en/miz_corpus.md](../en/miz_corpus.md).

## 目的

この module は large `.miz` corpus を構築・維持する strategy を定義する。

corpus は implementation asset である。syntax、type behavior、cluster behavior、overload behavior、proof rejection、dependency fingerprints、deterministic artifact output を固定するために使う。

## Corpus Classes

| Class | Purpose | Ownership |
|---|---|---|
| handwritten minimal cases | focused parser/type/cluster/proof behavior | developers が review |
| migrated examples | existing Mizar-like material 由来の realistic source patterns | acceptance 前に review |
| generated `.miz` | grammar and semantic combinations の broad coverage | stored seeds 付きで generate |
| fuzz-minimized reproducers | fuzz failures 由来の permanent regression cases | minimization 後に commit |
| bug regressions | fixed bugs and soundness failures を保護 | 可能なら issue/PR に link |
| stress/integration articles | large-file and incremental rebuild behavior | stability を review |

## Growth Targets

| Stage | Target |
|---|---|
| early module development | 100-300 `.miz` files |
| evo2 alpha | 300-1,000 `.miz` files |
| evo2 beta / release candidate | 1,000-5,000 `.miz` files |
| mature ecosystem | generated/fuzz/property corpora を含む 500,000-1,000,000 LOC equivalent |

growth は shallow pass tests の大量追加より、kernel 近傍の high-signal fail/soundness tests を優先する。

## File Size Guidelines

| Purpose | Lines |
|---|---:|
| parser test | 5-30 |
| type test | 10-50 |
| cluster test | 20-80 |
| theorem test | 30-150 |
| integration test | 100-300 |
| stress test | 500-1,000 |

validation はこれらの範囲を upper-bound review gate として扱う。多くの
fail/soundness cases は意図的に小さいため、短い minimal regression は diagnostic
なしで許可する。Oversized generated `.miz` files は `tests/stress/` 配下でない限り
error とし、oversized handwritten `.miz` files は warning とする。

## Generation Policy

generated `.miz` files は次を記録する。

- generator name and version
- seed
- generation profile
- expected outcome
- minimization status
- metadata schema version

generated、fuzz、property sidecars はこの provenance を `[origin]` に記録する。
metadata-only handoff anchors は crate-local test family または harness handoff を
generator とし、`generator_version = "handoff"`、fixture の stable phase/name を seed
としてよい。`origin.expected_outcome` は harness sidecar outcome を mirror する。
すべての fuzz seeds は `origin.original_failure_category` を記録する。promoted fuzz
failures では、その original fuzz failure class が executable `failure_category` と
一致しなければならない。

generated tests は coverage を増やす、bug を再現する、または stable stress case として機能する場合にのみ commit する。bulk generated corpora は minimize または promote されるまで default fast test set の外に置いてよい。

## Review Rules

corpus additions は次を review する。

- stable expected outcome
- deterministic diagnostics and snapshots
- test execution order への hidden reliance がないこと
- fail/soundness regressions としての minimality
- clear domain placement and naming

generated sidecars は stress cases として `tests/stress/` 配下に置く場合を除き
`tests/generated/` 配下に置く。fuzz seeds は `tests/fuzz/`、property seeds は
`tests/property/` 配下に置く。Unminimized generated/fuzz/property seeds は default
`fast` profile の外に置く。metadata-only fuzz handoff seeds は `fuzz_regression`
profile を使う。Stress cases は `profiles = ["stress"]` を使い、同時に `fast` に
opt in しない。

## Tests

key scenarios:

- generated tests は stored seed metadata から reproduce できる
- minimized fuzz reproducers は original failure category を保持する
- corpus manifest は domain ごとの pass/fail ratios を数える
- stress tests は要求されない限り default fast runs から除外される

## Constraints and Assumptions

- `.miz` corpus files は long-lived compatibility inputs である。
- Fail tests は current compiler behavior に合わせて loosen してはならない。
- soundness regression case は architecture-level review なしに削除しない。

## Checker Task 263 frozen corpus increment

separate docs prerequisite後、Task 263はcanonical-derived pass source/sidecar pair
`pass_type_elaboration_structure_definition_payload_001`をexactly 1件追加できる。
sourceはfrozen 320-byte/final-LF text、SHA-256
`078eaee4b17341c9d8ebeb8a1f631ca984873bd07eb4e5d9c1a9486b39ac6671`である。
sidecarはpass/type_elaboration/type_check、public diagnostics/payloads empty、new
Task-263 trace ref 1件である。

existing mixed mode/structure-definition failure pairと全parser/resolver structure
fixtureはbyte-identicalに保つ。docs prerequisiteはcorpus fileを追加せず、fresh
inventory後のimplementationだけがcases `425 -> 426`、pass `232 -> 233`をprojectする。

## Checker Task 263 active corpus increment

sole new pairは`pass_type_elaboration_structure_definition_payload_001.miz`とmatching
sidecarである。sourceはexact 320 bytes/16 lines、SHA-256は
`078eaee4b17341c9d8ebeb8a1f631ca984873bd07eb4e5d9c1a9486b39ac6671`、sidecar hashは
`d82c8d3102ea34fdb4a32792167c4b109b96b9c05265d3f04e6310278178e8ac`。
existing `.miz`/expectationはbyte-identicalで、cases `426`、pass/fail `233/193`、
active type `203`である。

## Task 257C4C0 frozen corpus increment

[C4C0 contract](../../task_contracts/ja/TEST-FRAENKEL-NESTED-CAPTURE-257C4C0.md)は
Chapter 13 nested-comprehension outer-generator capture oracle用implemented
handwritten pass pair 1件を`tests/miz/pass/types/`でownする。Pair inventoryは
`344/344`であるがinactiveであり、current parser/semantic/route/warning-error/Task277B creditは
0。Exact source bytes/hash/sidecarはcontract ownerであり、local-lookalike/builtin-set
variantは禁止。

## Task 257C4C1 explicit-import corpus repair

[C4C1 contract](../../task_contracts/ja/TEST-FRAENKEL-NESTED-CAPTURE-LEXICAL-ADMISSION-257C4C1.md)は
existing oracleをexact 164-byte explicit-import sourceへin-place update済みで、inactive
sidecar noteだけを更新する。Separate 140-byte module
`crates/mizar-test/tests/testdata/parser/nested_capture_fixtures.miz`はseparate
definition blocksでordered `Element`/`NAT` lexical shapesをdefineする。Crate-local
testdataでordinary corpus payloadではなくsidecarなし。Corpus pairsは`344/344`のまま、
active cases/outcomes/routes/warnings-errors/capture/Task277B creditは不変。
Measured file hashesとunchanged inventoriesはcontractの[precommit completion checkpoint](../../task_contracts/ja/TEST-FRAENKEL-NESTED-CAPTURE-LEXICAL-ADMISSION-257C4C1.md#precommit-implementation-completion-checkpoint)に一度だけrecordする。

Independent source-doc consistency/bilingual/boundary reviewは**NO FINDINGS**。
Independent final-quality reviewも**NO FINDINGS**で、canonical [precommit completion
checkpoint](../../task_contracts/ja/TEST-FRAENKEL-NESTED-CAPTURE-LEXICAL-ADMISSION-257C4C1.md#precommit-implementation-completion-checkpoint)からlinkする。
同historical review-time checkpointにrecordしたexact staging/cached reviewも
**NO FINDINGS**。
Task-only commit、post-commit proof、accepted fresh-inventory STOPはlanguage-local
[historical postimplementation checkpoint](../../task_contracts/ja/TEST-FRAENKEL-NESTED-CAPTURE-LEXICAL-ADMISSION-257C4C1.md#historical-immediate-postimplementation-pre-closure-checkpoint)でclosed。
Inactive zero-credit corpus dispositionは不変。
