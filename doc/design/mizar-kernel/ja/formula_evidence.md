# Module: formula_evidence

> 正本は英語です。英語版:
> [../en/formula_evidence.md](../en/formula_evidence.md)。

## 目的

`formula_evidence` module は、通常 proof acceptance で legacy resolution-trace
certificate を supersede する corrected kernel evidence schema を所有する。この module は
architecture 08、architecture 15、architecture 16、architecture 19、internal 04 を
精緻化する。

Schema は、kernel が check を求められる material であるという狭い意味でのみ trusted
input である。Provenance、substitution、target binding、formula identity、
deterministic SAT encoding、SAT refutation がすべて pass するまでは accepted ではない。

## Trust Statement

この module は trusted kernel code である。Formula/substitution evidence を parse し
structural validation できるが、parse は proof acceptance を与えない。

この module は proof search、premise selection、ATP search、backend invocation、
overload resolution、cluster search、implicit coercion insertion、fallback inference、
source loading、cache lookup、artifact lookup、wall-clock / random-state read、
unordered iteration dependence、mutable compiler-global state の hidden read を行っては
ならない。Instantiated formula、SAT clause、backend proof method、resolution trace、
SMT proof object、backend log を trusted payload として受理してはならない。

## Evidence shape

Corrected evidence object:

```text
KernelEvidence
  version
  target_vc
  kernel_profile
  formula_evidence
  substitutions
  final_goal
  provenance
```

`formula_evidence` entry は target VC で利用可能な formulas を記録する:

```text
FormulaEvidenceEntry
  formula_id
  source_class
  formula
  formula_fingerprint
  required_proof_status?
  imported_fact_ref?
  local_context_ref?
  vc_fact_ref?
  provenance_ref
```

`source_class` は local hypothesis、cited premise、generated VC fact、accepted imported
axiom、accepted imported theorem、または policy-bounded built-in fact のいずれかである。
最初の実装は normalized `clause::Atom` values 上の kernel-owned propositional formula
tree として formula を model してよい。より豊かな Mizar/core formula は paired spec update
と tests なしに追加してはならない。

各 entry は、stored formula、source class、provenance binding から導出される canonical
formula identity と hash input を持つ。Hash は source path、backend log、timestamp、
worker order、binding 後の display name、SAT clause を含んではならない。

## Substitutions

Substitution record は explicit evidence である。それは適用先 formula、normalized binder
context、side condition、`substitution_checker` が必要とする substitution payload を識別する。

Instantiated formula は trusted field ではない。Task 26 は checked substitutions を source
formulas に適用して instantiated formulas を導出する。Missing、stale、duplicate、
inconsistent な substitution provenance は kernel rejection であり、repair を推論してはならない。

## Provenance and target binding

すべての formula は一つの利用可能な proof source に bind しなければならない。Imported fact は
stable package/module/item identity、statement fingerprint、required proof status に bind する。
Local context と generated VC fact は caller-supplied target と VC provenance に bind する。
Kernel は accepted imported axiom/theorem source class を持つ accepted formula evidence から
だけ trusted `used_axioms` を導出しなければならない。

`final_goal` は target formula と refutation polarity を記録する。Kernel は supplied formulas
と negated goal を profile に従って check する。Target VC、goal fingerprint、kernel profile、
context identity が caller の immutable context と一致しない evidence は reject する。

## Legacy evidence

Legacy `Certificate`、`generated_clauses`、`resolution_trace`、backend proof method fields、
backend logs は compatibility または migration-audit material だけである。通常 proof policy では
unsupported evidence に map し、accepted `KernelCheckResult`、`used_axioms`、proof witness、
cache promotion、artifact `kernel_verified` status を生成できない。

## Gap classification

- `design_drift` / `source_drift`: task-22 source はまだ `checker` で legacy
  resolution-trace certificate を受理する。Tasks 25-29 がその path を置換または gate する。
- `test_gap`: task 25 は round-trip、malformed evidence、provenance-gap、
  deterministic rendering、hash-stability tests を追加する必要がある。
- `external_dependency_gap`: VC/ATP producers からの full source-derived formula payload は
  まだ complete ではない。Kernel schema は missing producer payload を fabricate せず reject する。

## Planned tests

Task 25 は structural round-trip、stable hash/rendering、unknown schema version、
duplicate id、malformed formula、missing provenance、imported fact identity/fingerprint
mismatch、missing target/goal binding、migration/audit mode 外の legacy evidence rejection を
test しなければならない。
