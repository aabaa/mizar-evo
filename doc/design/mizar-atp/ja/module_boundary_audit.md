# Module-Boundary Audit: mizar-atp

> 正本言語: 英語。英語正本:
> [../en/module_boundary_audit.md](../en/module_boundary_audit.md)。

## Task 27 Module-Boundary Refactor Gate

Task 27 は source/spec audit と bilingual documentation audit の後に
`mizar-atp` の source layout を監査する。この refactor は private test module
split であり、大きな inline unit-test suite を production module file から移す。
public API、diagnostic、deterministic rendering、artifact-facing schema、
candidate-evidence shape、trust-boundary rule はすべて保持される。

public API change はない。production behavior は変更しない。この crate は untrusted
formula/substitution evidence candidate producer のままであり、kernel checking、
proof policy、witness publication、proof-cache promotion、backend proof material、
resolution-trace acceptance、SAT problem payload trust を追加しない。

## Method

監査では次を確認した:

- [todo.md](./todo.md) の module table。
- [internal 07](../../internal/ja/07.crate_module_layout.md) の ownership rule。
- export されているすべての module の paired module spec。
- production source file と inline test size。
- 既存の lint-policy public API allowlist と crate-file guard。

Oversized production module file は inline unit test に由来する review-bottleneck
layout issue として分類し、source/spec behavior drift とは分類しない。新しい public
module を要求する mixed production responsibility は見つからなかった。Production
helper extraction は、paired module spec が具体的な helper boundary を定義するまで
deferred のままである。

## Layout Result

次の private `cfg(test)` child module が unit-test suite を所有する:

| Public module | Production source | Private test module | Test gate | Result |
|---|---|---|---|---|
| `backend` | `src/backend.rs` | `src/backend/tests.rs` | `cfg(all(test, unix))` | private test module split |
| `portfolio` | `src/portfolio.rs` | `src/portfolio/tests.rs` | `cfg(test)` | private test module split |
| `problem` | `src/problem.rs` | `src/problem/tests.rs` | `cfg(test)` | private test module split |
| `property_encoding` | `src/property_encoding.rs` | `src/property_encoding/tests.rs` | `cfg(test)` | private test module split |
| `smtlib_encoder` | `src/smtlib_encoder.rs` | `src/smtlib_encoder/tests.rs` | `cfg(test)` | private test module split |
| `tptp_encoder` | `src/tptp_encoder.rs` | `src/tptp_encoder/tests.rs` | `cfg(test)` | private test module split |
| `translator` | `src/translator.rs` | `src/translator/tests.rs` | `cfg(test)` | private test module split |

Production module は既存の module spec と一対一のままである: `backend`、
`portfolio`、`problem`、`property_encoding`、`smtlib_encoder`、`tptp_encoder`、
`translator`。Private test module は `src/lib.rs` から export されず、public API を
定義せず、所有 module の implementation-local test fixture に留まる。

## Classification

新しい `spec_gap`、`test_gap`、`design_drift`、`source_drift`、
`source_undocumented_behavior`、`test_expectation_drift`、`boundary_violation`、
`repo_metadata_conflict`、bilingual drift は見つからなかった。No new ATP-AUDIT gap
is required.

残る external/deferred follow-up は [source_spec_audit.md](./source_spec_audit.md)
から不変である: real backend output extraction、active source-derived corpus
execution、downstream proof/cache/artifact integration、typed/native encoder
extension、proof-policy finality はこの behavior-preserving layout refactor の外に残る。

## Verification Expectations

Task 27 は split 後も既存の behavior test を通す必要がある:

- `cargo test -p mizar-atp`
- `cargo clippy -p mizar-atp --all-targets --all-features -- -D warnings`
- `cargo fmt --check`

lint-policy guard はさらに、source tree が文書化済み production module と private test
module split だけを含むこと、および paired source/spec audit と bilingual audit がこの
task を記録することを確認する。
