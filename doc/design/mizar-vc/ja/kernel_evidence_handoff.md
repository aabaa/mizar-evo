# Module: kernel_evidence_handoff

> 正本言語: 英語。英語正本:
> [../en/kernel_evidence_handoff.md](../en/kernel_evidence_handoff.md)。

## 目的

Task 24 は、`mizar-vc` data から修正後の `mizar-kernel`
formula/substitution evidence format への producer-side handoff を定義する。
handoff は untrusted かつ prover-independent な package shape である。これは
proof acceptance ではなく、`mizar-vc` を SAT checker、ATP backend、proof-policy
owner、kernel caller にしない。

handoff は、validated immutable `VcSet` と選択された `VcIr` を、kernel checker が
parse / check できる material に写像する:

- target VC binding;
- kernel profile;
- kernel formula validation に必要な symbol / variable manifest;
- local hypothesis、cited premise、generated VC fact、accepted imported fact、
  policy-bounded built-in の formula evidence entry;
- upstream payload が既に存在する場合の explicit substitution evidence;
- formula、substitution、final goal ごとの provenance binding;
- standalone final goal record。

trusted acceptance boundary は `mizar-kernel` 内に残る。kernel は formula と
substitution から instantiated formula を再導出し、deterministic SAT problem を自分で
構築し、trusted in-process Rust SAT checking が必要な UNSAT result を返した場合だけ
受理する。

## Boundary Rules

`mizar-vc` は prover-independent のままでなければならない。task 25 が予定する
handoff builder は、既存の VC data、canonical formula payload、context entry、
premise reference、discharge record、dependency slice、provenance を調べてよいが、
次を行ってはならない:

- SAT solving を実行する、または `mizar-kernel` を呼び出す;
- ATP backend を呼び出す、または backend log を parse する;
- premise selection、substitution invention、binder repair、overload resolution、
  cluster search、implicit coercion insertion、fallback inference を行う;
- TPTP、SMT-LIB、DIMACS、SAT clause、resolution trace、
  MiniSAT-compatible certificate、solver proof method、instance/inverse method、
  SMT proof object、backend stdout/stderr、backend success flag、backend
  `used_axioms` を `VcIr` または canonical kernel evidence input に追加する;
- legacy `Certificate`、`LegacyCertificate`、`LegacyResolutionTrace` object を
  trusted handoff material として構築する。

Instantiated formula と SAT clause は handoff field ではない。これらは kernel-derived
acceptance material のみである。

## Conceptual Handoff Shape

Task 25 は、既存 `VcIr` と kernel parser API に合う具体的な Rust type を選んで、
次の conceptual shape に相当する immutable builder を実装する。canonical evidence
section は kernel v1 envelope の field と section name に一致しなければならない:

```text
VcKernelEvidenceHandoff
  canonical_evidence
    schema_version
    encoding_version
    target_vc
    kernel_profile
    symbol_manifest
    variable_manifest
    formula_evidence
    substitutions
    provenance
    final_goal
  formula_context_requirements?
  diagnostic_inputs?
```

`formula_context_requirements` は canonical evidence-envelope section ではない。
imported axiom / theorem を accepted と扱う前に `mizar-kernel` へ
`FormulaEvidenceContext` として渡す必要がある immutable imported-fact context を記録する。
`mizar-vc` は candidate source binding と required proof-status requirement を運んでよいが、
imported fact が accepted であるとは認定しない。imported-fact context の不足または不一致は
fail-closed builder error、kernel rejection、または `external_dependency_gap` である。

`diagnostic_inputs` は explainability 用の任意 producer-side detail である。後続 spec が
stable field として明示的に昇格しない限り、canonical kernel evidence bytes、hash
input、proof reuse identity から除外する。snapshot-local な `VcId`、generated formula
id、context-entry id、source range、handoff row id は diagnostics に現れてよいが、
canonical evidence は stable formula fingerprint、target identifier、source binding、
provenance record で binding しなければならない。

## Input Mapping

builder input は validated `VcSet`、選択された `VcIr`、および prior VC phase が既に
計算した任意の producer-owned record である:

| VC input | Kernel evidence mapping |
|---|---|
| `VcSet` schema、module、source、canonical VC fingerprint、選択された `VcIr` | `target_vc`、target provenance binding、deterministic package identity。stable target binding を計算できない場合、builder は fail closed する。 |
| formula ref を持つ `LocalContext` entries | local-hypothesis source binding を持つ formula evidence entry。stable formula payload または provenance を欠く entry は捏造せず missing payload と記録する。 |
| `PremiseRef::LocalContext` と `PremiseRef::GeneratedFact` | 対応する local-hypothesis または generated-VC-fact formula evidence entry への reference。 |
| `PremiseRef::ImportedFact` | package/module/exported item identity、statement fingerprint、required proof-status requirement、matching `FormulaEvidenceContext` input が利用可能な場合だけ candidate imported axiom/theorem formula entry にする。`mizar-vc` は imported fact が accepted であるとは認定せず、proof/kernel-owned context がそれを行わなければならない。それ以外は `external_dependency_gap` または fail-closed builder error。 |
| `PremiseRef::CheckerFact`、`TypePredicate`、trace、registration、cluster、reduction、definition、policy、conservative-unknown variants | explicit formula payload、許可された source class、target binding、provenance が既に利用可能な場合だけ写像する。marker-only または trace-only record は trusted evidence にならない。 |
| `VcGeneratedFormula` table | formula tree を kernel-supported formula grammar に projection でき、provenance が選択 target に bind する場合、generated VC fact entry にする。 |
| `VcIr.goal` | standalone `final_goal` record。premise ではなく、`used_axioms` の source にもならない。 |
| `ProofHint` と premise restriction | diagnostic または candidate-production metadata のみ。premise を選択、追加、削除せず、acceptance を認可しない。builder は immutable `VcIr` input に既に materialized された exact premise ref だけを参照してよい。それらの input に既に反映されていない restriction は diagnostic のままにする。 |
| `DischargeEvidenceRecord` | record が handoff 内に既に存在する replayable formula reference を含む場合だけ、candidate formula/substitution/provenance input になれる。discharge rule name や evidence hash は trusted acceptance material ではない。 |
| `DependencySlice` と proof-reuse candidate data | task 26 の identity / invalidation input。VC を証明せず、kernel checking を置き換えない。 |

builder は deterministic ordering を保持する。formula payload 不足、imported-fact
identity 不足、provenance 不足、projection 不能な formula、substitution payload 不足は
fail-closed builder error または classified deferred row であり、complete evidence package
を主張しながら黙って落としてはならない。

## Formula Projection

Kernel task 25 は、現在 normalized kernel atom 上の propositional formula tree を
support する。`mizar-vc` が VC formula をその grammar へ projection できるのは、source
formula payload が必要な normalized atom、symbol、variable、binder、provenance data を
すべて既に提供している場合だけである。

`mizar-vc` は display text、source range、debug rendering、backend encoding、trace name、
local id、proof-method metadata から formula を再構築してはならない。`CoreFormulaId`、
`VcFormulaRef`、generated formula shape を stable kernel formula tree へ解決できない場合、
builder は `external_dependency_gap` を記録し、その VC の trusted handoff package を返さない。

## Substitutions

Substitution evidence は explicit である。substitution record は、upstream または
producer-owned payload が次を既に提供している場合だけ含めてよい:

- source formula id;
- binder-context encoding;
- `substitution_checker` payload;
- freshness witness と free-variable constraint;
- target VC と source formula fingerprint への provenance binding。

handoff は substitution record 内に instantiated formula や target formula field を含めては
ならない。kernel が checking 中に checked substitution を適用し、instantiated formula を
導出する。missing、stale、duplicate、inconsistent な substitution record は builder failure
または kernel rejection であり、repair の機会ではない。

## Legacy And Prohibited Material

legacy resolution-trace certificate は、修正後 evidence pipeline では migration/audit-only
material である。normal proof policy では unsupported と扱われ、kernel-accepted status、
proof witness、artifact `kernel_verified` status、cache promotion、trusted `used_axioms` を
生じさせられない。

したがって VC handoff は次を除外しなければならない:

- TPTP または SMT-LIB problem;
- DIMACS または SAT clause;
- caller-supplied instantiated formula;
- resolution trace と MiniSAT-compatible certificate;
- backend proof method、instance method、inverse method、SMT proof object、
  backend log;
- backend `used_axioms`、success flag、timing、stdout/stderr;
- accepted evidence としての legacy certificate parser output。

## Gap Classification

Task 24 は修正後 handoff contract を記録し、`mizar-kernel` が存在しないとしていた
closeout 時点の分類を更新する。Kernel task 23-29 は formula/substitution evidence
schema、deterministic instantiation / SAT encoding、trusted SAT checker wrapper、
SAT-backed check service、legacy-audit gating を提供済みである。VC 側は引き続き
producer-side のみである。

残る gap:

- `external_dependency_gap` `VC-HANDOFF-G001`: source-derived stable full core
  formula payload、definition payload、quantified binder payload、一部 generated
  obligation payload family はまだ upstream で incomplete。
- `external_dependency_gap` `VC-HANDOFF-G002`: imported fact package/module/item
  identity、required proof-status payload、immutable `FormulaEvidenceContext` input は、
  すべての `PremiseRef::ImportedFact` でまだ一様には利用できない。
- `external_dependency_gap` `VC-HANDOFF-G003`: ATP candidate evidence production、
  proof witness policy、cache consumer、artifact witness consumer は downstream work。
- `deferred` `VC-HANDOFF-G004`: immutable Rust handoff builder、canonical rendering、
  builder error、focused tests は task 25 が所有する。
- `deferred` `VC-HANDOFF-G005`: kernel evidence hash を含む dependency-slice /
  proof-reuse identity update は task 26 が所有する。

## Planned Tests

Task 25 は Rust coverage を追加し、次を確認しなければならない:

- deterministic handoff rendering と canonical byte/hash input stability;
- local context、premise、generated formula、final goal、provenance mapping;
- imported fact payload completeness と missing identity の fail-closed;
- instantiated-formula field を含まない substitution payload inclusion;
- discharge record が replayable formula/substitution evidence のみを供給し、
  trusted rule name や evidence hash にならないこと;
- public API に backend text、SAT clause、resolution trace、backend proof method、
  legacy certificate acceptance field がないこと;
- formula/provenance/substitution payload 不足が builder error または classified
  deferred record になること。

Task 26 は、canonical kernel evidence hash が変わると proof-reuse identity が変わり、
downstream proof/cache/artifact schema がない場合には reuse が利用不能なままであることを
示す invalidation test を追加しなければならない。

## Task 25 Handoff

推奨 reasoning: `xhigh`。

Prompt:

```text
Continue mizar-vc autonomous correction from completed task 24. Before editing,
verify a clean worktree, confirm the task 24 commit in git log, and re-read
doc/design/mizar-vc/en/kernel_evidence_handoff.md,
doc/design/mizar-kernel/en/formula_evidence.md,
doc/design/mizar-kernel/en/checker.md,
doc/design/architecture/en/15.kernel_certificate_format.md,
doc/design/architecture/en/08.reasoning_boundary.md,
crates/mizar-vc/src/vc_ir.rs, crates/mizar-vc/src/discharge.rs, and
crates/mizar-vc/src/dependency_slice.rs. Implement task 25 only: add an
immutable kernel evidence handoff builder over existing VcSet/VcIr data. Keep
the builder prover-independent; do not run SAT solving, call mizar-kernel, call
ATP backends, include backend proof methods, include resolution traces, or
fabricate missing formula/substitution/provenance payloads. Add focused Rust
tests for deterministic rendering, local context/premise/generated formula/goal
mapping, missing payload fail-closed behavior, and absence of prohibited
backend/legacy fields. Run cargo fmt --check, cargo test -p mizar-vc,
cargo clippy -p mizar-vc --all-targets --all-features -- -D warnings, git diff
--check, and git diff --cached --check after explicit path staging. Use
review-only agents for the required AGENTS.md review phases.
```

根拠: task 25 は VC/kernel evidence boundary での最初の Rust 実装である。小さな
API mistake が producer-owned candidate material を accidental trusted acceptance
material に変え得るため、`xhigh` を維持する。typo-only documentation synchronization の
場合だけ、より低い reasoning が適切である。
