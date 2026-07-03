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
