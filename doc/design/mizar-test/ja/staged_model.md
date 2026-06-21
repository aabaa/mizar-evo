# Module: staged_model

> Canonical language: English. English canonical version: [../en/staged_model.md](../en/staged_model.md).

## Purpose

この module は Mizar Evo tests を追加するための staged model を定義する。

この model は、lower-level tests でまだ固定されていない language features に test cases が依存することを防ぐ。Primary ordering axis は pipeline maturity、coverage axis は `doc/spec/` chapters とする。

## Design Decision

Test addition order は specification chapter order だけではなく、compiler pipeline dependencies によって決める。

`doc/spec/` は最終的に何を cover すべきかを所有する。Pipeline は、earlier unresolved feature を誤って test することなく、ある test をいつ追加できるかを所有する。

```text
Primary ordering:  pipeline maturity / dependency layer
Coverage mapping:  doc/spec chapter and section
```

## Stages

| Stage | Stage Id | Fixture Style | Primary Pipeline Boundary | Spec Coverage |
|---|---|---|---|---|
| 1. Lexical | `lexical` | token fixtures, minimal source snippets | lexer | `02.lexical_structure` |
| 2. Parse-only | `parse_only` | parsing までで check する `.miz` snippets | parser | each chapter の syntax portions |
| 3. Declaration / symbol | `declaration_symbol` | declarations plus resolution expectations | symbol collection and name resolution | structs, attributes, modes, predicates, functors, modules |
| 4. Type / elaboration | `type_elaboration` | typed declarations and expressions | type checking and elaboration | type system, attributes, modes, terms, formulas |
| 5. Formula / statement | `formula_statement` | resolved symbols を持つ formulas and statements | typed AST and statement checking | terms, formulas, statements |
| 6. Proof / verification | `proof_verification` | theorem/proof fixtures | VC generation and verification | theorems, proofs, algorithms |
| 7. Advanced semantics | `advanced_semantics` | focused integration and negative tests | clusters, overload, templates, substitution, ATP, certificates, kernel | advanced semantic chapters and guardrails |

`Stage Id` は `.expect.toml`、`tests/coverage/spec_trace.toml`、reports、Rust enums で使う canonical value である。Display names は localize してよいが、stage ids は localize してはならない。

## Stage Rules

### 1. Lexical

Lexical tests は parsing、name resolution、type checking、library symbols を必要としてはならない。

対象:

- reserved words and identifiers
- comments and annotations as tokens
- symbolic characters and punctuation
- malformed lexical input

### 2. Parse-only

Parse-only tests は source shape が parser に受理または拒否されることだけを assert する。Semantic validity を主張してはならない。

対象:

- block structure
- declaration forms
- theorem, proof, and statement syntax
- malformed syntax からの recovery

### 3. Declaration / Symbol

Declaration and symbol tests は later stages に必要な最小の valid declarations を導入する。Undefined symbol failures は、later-stage test が resolver behavior を明示的に target する場合を除き、この stage に属する。

この stage の active runner は `mizar-test declaration-symbol` subcommand である。
これは `active_declaration_symbol` tag を持つ `.miz` pass/fail expectation だけを
実行する。tag のない declaration-symbol sidecar は、所有する resolver behavior が
実行可能になるまで traceability metadata のままにする。public resolver diagnostic
code が specification gap の間、fail case は user-facing code ではなく
crate-local internal detail key を assert し、non-empty `diagnostic_codes` は active
gate で拒否される。

対象:

- symbol registration
- duplicate or conflicting declarations
- visibility and qualification
- undefined name diagnostics

### 4. Type / Elaboration

Type and elaboration tests は built-ins と lower stages で既に cover された symbols だけを使う。Proof search、overload ambiguity、cluster saturation、kernel evidence に依存してはならない。ただしそれ自体が explicit subject の場合を除く。

対象:

- built-in radix types
- mode and attribute use
- type argument checking
- term and formula elaboration

### 5. Formula / Statement

Formula and statement tests は token、parse、name、type prerequisites が確立された後の source forms を check する。

対象:

- equality and predicate application
- quantifiers and binders
- assumptions, labels, and local statements
- statement-level failure classification

### 6. Proof / Verification

Proof and verification tests は earlier syntactic and semantic layers が stable であることに依存してよい。Proof boundaries、VC generation、verifier outcomes を check する。

対象:

- valid trivial proofs
- proof reference resolution
- failed proof obligations
- deterministic verification diagnostics

### 7. Advanced Semantics

Advanced tests は lower prerequisites に dedicated coverage が揃った後にだけ追加する。Fail-heavy であり、expected failure boundary を precise に記録しなければならない。

対象:

- cluster expansion and cycle detection
- overload resolution and template inference
- substitution and binder normalization
- ATP interface behavior
- certificate and kernel rejection
- soundness regressions

## Spec Mapping

Committed corpus test は、sidecar metadata または corpus manifest に、cover する specification section を記録する。

Mapping は many-to-many である。

- one spec section can require tests at multiple stages
- one integration test can cover several spec sections
- coverage credit is assigned only to stages whose prerequisites are already satisfied

例えば cluster-cycle test は `17.clusters_and_registrations` に map されるが、その chapter に syntax が現れるというだけで stage 2 に追加しない。Explicitly parse-only fixture でない限り、stage 7 に属する。

## Admission Checklist

`.miz` test が committed corpus に入る条件:

- intended stage が宣言されている
- referenced `doc/spec/` section が明確である
- all lower-stage prerequisites が既に cover されているか built-ins として列挙されている
- name resolution が target でない限り undefined library symbols を使わない
- expected phase が earliest sound rejection point である
- fail expectations は sidecars と stable failure identities を使う
- test が claim する behavior に対して minimal である

## Growth Policy

Corpus は staged model に沿って前方へ成長する。Lower prerequisites がない段階で higher-stage test が必要な場合、その test は committed corpus には入れないか、default discovery 外の draft material として扱う。

Default fast corpus は、曖昧な broad examples よりも少数の trustworthy tests を優先する。
