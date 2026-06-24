# Module: dependency_slice

> 正本は英語です。英語版:
> [../en/dependency_slice.md](../en/dependency_slice.md)。

## 目的

この module は generation、normalization、status policy、discharge 後の canonical
`VcSet` data に対する conservative dependency slice を仕様化する。dependency
slice は、deterministic diagnostics、cache decision、artifact metadata、後続の
prover/proof consumer のために、VC が依存する local fact、generated formula、
imported fact、definition、registration、cluster/reduction trace、policy input、
evidence input を記録する。

Task 13 は仕様のみである。
[architecture 18](../../architecture/ja/18.dependency_fingerprint.md) と phase-12 boundary
を精緻化するが、language semantics、`.miz` fixture、expectation、`doc/spec`、
traceability metadata、Rust source は変更しない。

## 責務

この module が所有するもの:

- stable per-VC dependency-slice data shape;
- context entry、premise ref、proof hint、discharge evidence、anchor、generated
  formula の dependency classification;
- dependency が存在しないと偽らず、cache miss または downstream recomputation を
  強制する conservative unknown-coverage marker;
- artifact と reuse key に使える stable dependency-slice fingerprint。

範囲外:

- task 14 より前の Rust による slice computation;
- ATP translation、proof search、kernel proof acceptance、certificate validation;
- proof/cache artifact persistence または corpus-runner integration;
- 新しい source extraction、新しい VC generation、新しい semantic payload family;
- unknown dependency coverage を empty dependency set として扱うこと。

## この仕様の gap 分類

| ID | 分類 | 証拠 | 扱い |
|---|---|---|---|
| DEP-G001 | `spec_gap` | task 13 より前に `dependency_slice.md` は存在せず、tasks 14 と 20 には dependency/fingerprint contract が必要である。 | Task 13 は英語/日本語 spec だけを追加する。 |
| DEP-G002 | `source_drift` / `test_gap` | `src/dependency_slice.rs`、slice data shape、fingerprint helper、focused slice tests はまだ存在しない。 | Task 14 がこの仕様に従って source/tests を実装する。 |
| DEP-G003 | `external_dependency_gap` | complete registration、cluster、reduction、import、proof/cache、corpus、artifact consumer はすべてが `mizar-vc` に接続済みではない。 | この仕様は unavailable coverage に conservative marker を要求し、consumer integration を deferred にする。 |
| DEP-G004 | `deferred` | cross-edit reuse identity、canonical VC/context fingerprint、[architecture 22](../../architecture/ja/22.incremental_verification_contract.md) proof-reuse gate は task 20 以降の作業である。 | Task 13 は dependency slice が後続 fingerprint にどう供給されるべきかを定義するが、実装しない。 |

## 入力と出力

必須入力:

- validated `VcSet`;
- generated formula、local context、premise、proof hint、anchor、status、seed
  accounting、利用可能な場合の discharge output evidence/explanation;
- `VcIr` に既に存在する explicit upstream identifier、たとえば `CoreFormulaId`、
  `ContextEntryId`、`CoreDefinitionId`、premise ref、trace ref、policy key、evidence hash。

各 concrete VC の必須出力:

- snapshot-local VC id、VC kind、slice 計算時に観測した status;
- stable class と source reference を持つ sorted dependency entries;
- coverage が不完全または利用不能な dependency family に対する conservative unknown marker;
- normalized entries、schema version、policy input、status/evidence boundary、unknown marker に基づく stable cross-edit dependency-slice fingerprint。

slice は batch output 内の VC order を保持しなければならない。VC を削除したり、goal を書き換えたり、
status を変更したり、hidden fact を推論したり、missing upstream payload を dependency がない証拠として扱ったりしてはならない。

`VcId` は 1 つの `BuildSnapshot` 内の ownership/order metadata であり、reuse 可能な
cross-edit dependency-slice fingerprint の入力ではない。artifact record は diagnostics と result
collation のために現在の `VcId` を fingerprint の横に保存してよいが、proof/cache reuse は
[architecture 22](../../architecture/ja/22.incremental_verification_contract.md) が述べる
`ObligationAnchor` と canonical VC/context key を通して fingerprint content を検証しなければならない。

## Dependency Entry Classes

Task 14 は structured Rust enum を導入してよいが、semantic class は次である:

- `local_context`: `ContextEntryId` とその formula、kind、provenance、policy-input relationship;
- `generated_formula`: `VcGeneratedFormulaId` とそれが transitively に参照する generated formula;
- `core_formula`: goal、context formula、checker fact、type predicate、generated formula leaf、premise target として使われる `CoreFormulaId`;
- `definition`: definition boundary、permitted unfolding、unfold request、definitional discharge evidence が参照する `CoreDefinitionId`;
- `imported_fact`: `PremiseRef` として既に存在する imported symbol と cited premise;
- `trace`: explicit registration、cluster、reduction、conservative-unknown trace ref;
- `policy`: status、discharge、unfolding、computation limit、ATP dispatch に影響した policy key/value;
- `anchor`: cache/reuse eligibility に影響する complete/incomplete anchor ingredient;
- `discharge_evidence`: `DischargeOutput` 由来の rule name、evidence hash、evidence input、preserved-evidence marker;
- `seed`: concrete-VC cardinality を stable に保つための seed handoff id と seed mapping row。

entry は VC id、class rank、stable local key、debug-stable payload の順で並ぶ。hash-map
iteration order、absolute path、wall-clock time、backend availability、worker scheduling は順序に影響してはならない。

## Conservative Unknown Coverage

unknown coverage は first-class dependency result であり、それ自体は error ではない。次の場合は
conservative marker を使う:

- premise が `ConservativeUnknown` である;
- anchor が incomplete、または必要な anchor ingredient が unavailable である;
- registration、cluster、reduction、import、definition、computation trace が opaque textual marker でしか分からない;
- discharge が replay data なしの pre-existing status evidence を preserved した;
- upstream crate が dependency family を完全に列挙する payload をまだ公開していない。

unknown coverage を含む slice は、complete dependency precision を必要とする consumer に対して cache miss
または downstream revalidation を強制しなければならない。それでも deterministic であり、どの family が不完全かを説明しなければならない。
unknown coverage を fingerprint から黙って落としてはならない。

## Fingerprint Contract

dependency-slice fingerprint は proof certificate ではない。untrusted で deterministic な reuse input である。
reuse 可能な cross-edit fingerprint は snapshot-local `VcId` を除外し、次を含まなければならない:

- dependency-slice schema version;
- `VcKind`、status boundary、evidence boundary;
- ordered dependency entries と conservative unknown marker;
- 関連する policy key/value;
- generated formula reference と discharge evidence hash;
- stable anchor/context hash marker が利用可能な場合はそれ、利用不能な場合は conservative unknown marker。

consumer は matching `VcId`、source range、anchor だけで proof/cache reuse を許可してはならない。後続の reuse
task は confident な `ObligationAnchor` match、canonical VC fingerprint、context fingerprint、
dependency-slice fingerprint、policy/evidence hash、consumer-specific validation policy を組み合わせなければならない。

## Planned Tests

Task 14 は Rust coverage として次を追加しなければならない:

- local context、generated formula、core goal formula、premise、proof hint、
  policy、anchor、seed、discharge-evidence dependency;
- definition と permitted-unfolding dependency;
- registration、cluster、reduction、conservative unknown marker の trace ref;
- stable ordering と deterministic debug/fingerprint rendering;
- reuse 可能な fingerprint boundary: `VcId` だけが違う otherwise identical slice は
  distinct owner/order metadata を保持しながら同じ reusable dependency-slice fingerprint を持つこと;
- goal、premise、proof hint、discharge evidence、policy boundary から参照されない unused local fact の除外;
- missing または incomplete dependency coverage が conservative unknown marker、cache-miss intent、
  fingerprint participation を生成すること;
- `NeedsAtp`、policy、skipped、deferred、error、discharged status boundary の保持。

canonical VC/context fingerprint と artifact consumer が存在する後続 task では、cross-edit reuse identity と
architecture-22 gate の coverage を追加しなければならない。
