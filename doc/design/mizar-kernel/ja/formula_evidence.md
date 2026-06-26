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

この module には no proof search、no SAT solving、no ATP search or backend
invocation、no premise selection、no overload resolution、no cluster search、no implicit
coercion insertion、no fallback inference、no acceptance from backend-reported success
alone、no source loading、no cache lookup、no artifact lookup、no wall-clock or
random-state reads、no unordered iteration dependence、no hidden reads of mutable
compiler-global state が適用される。Instantiated formula、SAT clause、backend proof
method、resolution trace、SMT proof object、backend log を trusted payload として受理しては
ならない。

## Evidence shape

Corrected evidence object:

```text
KernelEvidence
  schema_version
  encoding_version
  kernel_profile
  target_vc
  symbol_manifest
  variable_manifest
  formula_evidence
  substitutions
  provenance
  final_goal
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

各 entry は tree-only formula fingerprint と、source class / provenance binding に対する
別個の kernel-derived entry hash input を持つ。どちらの hash input も source path、backend log、
timestamp、worker order、binding 後の display name、SAT clause を含んではならない。

`ParsedKernelEvidence` は private field と read-only accessor によって parser-validated
binding を保持する。Caller は target、profile、manifest、formula、substitution、
provenance、final-goal、canonical hash input record を inspect してよいが、validation 後かつ
kernel checking 前に parsed object を mutate または reconstruct できてはならない。

## Canonical V1 envelope

Task 25 は `src/formula_evidence.rs` が所有する deterministic binary envelope を実装する。
v1 envelope は domain separator `MIZAR_KERNEL_EVIDENCE\0`、schema version `1`、
encoding version `1`、既存の `certificate_parser::KernelProfileRecord`、expected target
VC fingerprint、固定順序の sections を使う:

1. symbol manifest;
2. variable manifest;
3. formula evidence entries;
4. substitution evidence records;
5. provenance entries;
6. final goal.

各 section は length-framed item の列である。Parser は unknown schema / encoding version、
noncanonical section order、trailing bytes、duplicate id、unsorted id list、section count
mismatch、resource-limit violation を reject する。Parsed evidence object の canonical hash
input は検証済み envelope bytes そのものであり、producer が別の trusted hash payload を供給しては
ならない。

Symbol / variable manifest は formula validation のための structural input にすぎない。
これらは formula と substitution payload 内の `clause::Atom` / `clause::Term` を検査する
`clause::ClauseValidationContext` を定義する。Hidden symbol lookup、overload resolution、
source loading を許可しない。

## Formula grammar

最初の実装は次の propositional formula grammar を support する:

```text
Formula =
  Atom(clause::Atom)
  Not(Formula)
  And(nonempty Formula list)
  Or(nonempty Formula list)
```

すべての child list は length-framed で、parser limit により bounded であり、caller order を
保持する。Parser は各 atom を manifest-derived clause context に対して検証し、malformed term、
missing symbol、unsupported symbol kind、noncanonical variable、oversized term、empty
conjunction/disjunction、過大な formula depth / node count を reject する。Formula rendering は
deterministic だが diagnostics/tests 用であり、trusted input ではない。

Task 25 の normalized formula fingerprint algorithm は
`SUPPORTED_FORMULA_FINGERPRINT_ALGORITHM_ID = 2` である。その digest は parsed formula tree
から kernel が導出する canonical formula hash input bytes である。Parser はこの fingerprint を
再計算し、mismatch を reject する。これは stable identity binding であり、cryptographic
acceptance claim ではない。

Formula fingerprint は tree-only identity である。Formula entry は formula id、source class、
source binding、tree fingerprint、provenance reference に対する kernel-derived entry hash input も
持つが、その entry hash は caller-supplied trusted field ではない。Provenance check は explicit な
target と formula-tree fingerprint を比較し、entry hash を proof acceptance material として扱わない。

## Source and provenance binding

各 formula entry は `source_class` と形状が一致するちょうど一つの source binding に bind する:

- local hypothesis と cited premise entry は nonzero local-context id を使う;
- generated VC fact は nonzero VC-fact id を使う;
- accepted imported axiom / accepted imported theorem は package id、module path、
  exported item id、statement fingerprint、required proof status を使う;
- policy-bounded built-in は nonempty built-in id を使う。

すべての formula entry は一つの provenance entry を参照する。Provenance entry は provenance
id、target VC fingerprint、formula-tree fingerprint、opaque nonempty producer-owned payload を
bind する。Parser は missing provenance、empty provenance payload、provenance target-binding
mismatch、formula fingerprint mismatch を reject する。Richer source formula projection が仕様化されるまでは、
imported source binding の imported statement fingerprint は formula-tree fingerprint と一致しなければ
ならない。

## Substitutions

Substitution record は explicit evidence である。それは適用先 formula、normalized binder
context、side condition、`substitution_checker` が必要とする substitution payload を識別する。

Instantiated formula は trusted field ではない。Task 26 は checked substitutions を source
formulas に適用して instantiated formulas を導出する。Missing、stale、duplicate、
inconsistent な substitution provenance は kernel rejection であり、repair を推論してはならない。

Task 25 は substitution record を explicit payload evidence としてだけ保存する:

```text
FormulaSubstitutionEvidence
  substitution_id
  source_formula_id
  binder_context_encoding
  payload: substitution_checker::SubstitutionPayload
  freshness_witnesses
  free_variable_constraints
  provenance_ref
```

Target formula / instantiated formula field はない。Parser は id、参照 source formula、provenance
binding、term path、payload owner id、replacement role、witness owner、
free-variable constraint owner、deterministic ordering、resource limit を structural に検証する。
Semantic replay と formula instantiation は task 26 の作業である。

## Provenance and target binding

すべての formula は一つの利用可能な proof source に bind しなければならない。Imported fact は
stable package/module/item identity、statement fingerprint、required proof status に bind する。
Local context と generated VC fact は caller-supplied target と VC provenance に bind する。
Kernel は accepted imported axiom/theorem source class を持つ accepted formula evidence から
だけ trusted `used_axioms` を導出しなければならない。

`final_goal` は target formula と refutation polarity を記録する。Kernel は supplied formulas
と negated goal を profile に従って check する。Target VC、goal fingerprint、kernel profile、
context identity が caller の immutable context と一致しない evidence は reject する。

Task 25 の final goal record は standalone goal formula、goal polarity、formula fingerprint、
provenance reference を含む。これは asserted premise formula set の一部ではなく、`used_axioms` の
source でもない。Parser は同じ manifest-derived context で goal formula を structural に検証し、
fingerprint が goal formula tree と一致すること、provenance が target VC と goal fingerprint に
bind することを要求する。Target binding は記録するが、SAT encoding や acceptance は行わない。

## Legacy evidence

Legacy `Certificate`、`generated_clauses`、`resolution_trace`、backend proof method fields、
backend logs は compatibility または migration-audit material だけである。通常 proof policy では
unsupported evidence に map し、accepted `KernelCheckResult`、`used_axioms`、proof witness、
cache promotion、artifact `kernel_verified` status を生成できない。

## Gap classification

- `design_drift` / `source_drift`: task-22 source は legacy resolution-trace certificate を
  `checker` に保持していた。Task 29 はその path を explicit migration/audit policy の
  背後に gate し、normal proof policy では replay 前に拒否する。
- `test_gap`: task 25 は round-trip、malformed evidence、provenance-gap、
  deterministic rendering、hash-stability tests を追加する必要がある。
- `external_dependency_gap`: VC/ATP producers からの full source-derived formula payload は
  まだ complete ではない。Kernel schema は missing producer payload を fabricate せず reject する。
- `deferred`: semantic formula instantiation、SAT encoding、SAT checking、service
  acceptance、artifact witness projection、ATP candidate evidence production は後続 task であり、
  ここで stub してはならない。

## Rejection mapping

Task 25 は envelope parsing と evidence-binding validation を分離する。Envelope / byte-shape error
は `certificate_rejection` に map し、evidence-binding error は `kernel_rejection` に map する。
Stable detail は次のとおり:

- domain、schema、encoding、section order、profile support failure は
  `unsupported_certificate_format`;
- envelope expected-target mismatch は certificate-level `context_mismatch`;
- provenance / final-goal target-binding mismatch は kernel `missing_provenance`;
- malformed formula/source/substitution/final-goal bytes は `malformed_witness_data`;
- missing / inconsistent source/provenance/goal binding は kernel `missing_provenance`;
- parser / canonical-byte limit は `resource_exhaustion`.

Legacy `Certificate` / `resolution_trace` bytes は task-25 domain separator を共有しないため、
この parser では unsupported として reject される。

## Planned tests

Task 25 は structural round-trip、stable hash/rendering、unknown schema version、
duplicate id、malformed formula、missing provenance、imported fact identity/fingerprint
mismatch、missing target/goal binding、migration/audit mode 外の legacy evidence rejection を
test しなければならない。
