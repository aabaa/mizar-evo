# Diagnostic Registry

> 正本は英語です。英語版:
> [../en/registry.md](../en/registry.md)。

## 目的

`mizar-diagnostics` は `DiagnosticCode` registry を所有する。registry は、
共有 diagnostics model を通じて公開される各 diagnostic に安定した機械可読
identity を与え、コードが別の意味に再利用されていないことを検証し、
rendering、aggregation、fix、explanation、外部 consumer のための小さな
metadata を公開する。

この task は
`doc/spec/en/22.error_handling_and_diagnostics.md` に列挙された diagnostic の
ための初期 stable code space を定義する。build failure、cache/artifact
compatibility、documentation extraction、editor-only overlay など、diagnostic に
言及する他の architecture surface は、normative range と descriptor がここに追加
されるまで code-space gap として残る。`mizar-diagnostics` はそれらの将来の割り
当てについても owner であり続けるが、task 3 は placeholder range や descriptor
を発明してはならない。

registry は phase semantics、proof acceptance、trusted status、kernel
acceptance、artifact mutation、LSP protocol conversion、driver session
orchestration を所有しない。これらの service は registry metadata を消費して
よいが、message text、localized text、ordering、semantic name ではなく
`DiagnosticCode` を key にしなければならない。

## コードの形

`DiagnosticCode` は `Xnnnn` の形を持つ。

- `X` は severity prefix であり、`E`、`W`、`I` のいずれかである。
- `nnnn` は 4 桁の 10 進数である。
- prefix と数値は stable identity の一部である。
- 番号は恒久的に割り当てる。retired になった番号を再発行してはならない。

コード範囲は言語仕様に従う。この表の `PhaseFamily` label が descriptor と
test で用いる canonical registry vocabulary である。

| 範囲 | `PhaseFamily` | Scope | 既定 severity |
|---|---|---|---|
| `E0001`-`E0099` | `Syntax` | Lexical and syntax diagnostics | Error |
| `E0100`-`E0199` | `Type` | Type diagnostics | Error |
| `E0200`-`E0299` | `Resolution` | Resolution, overload, and template diagnostics | Error |
| `E0300`-`E0399` | `Proof` | Proof and ATP diagnostics | Error |
| `E0400`-`E0499` | `Logic` | Logical consistency and verification-condition diagnostics | Error |
| `E0500`-`E0599` | `Algorithm` | Algorithm verification diagnostics | Error |
| `W0001`-`W0099` | `StructuralWarning` | Structural warnings | Warning |
| `W0100`-`W0199` | `ProofWarning` | Proof and ATP warnings | Warning |
| `W0200`-`W0299` | `AlgorithmWarning` | Algorithm and contract warnings | Warning |
| `W0300`-`W0399` | `CompatibilityWarning` | Compatibility and packaging warnings | Warning |
| `I0001`-`I0099` | `Info` | Informational display diagnostics | Info |

これらの範囲外の実装 diagnostic は registry error であり、通常の user
diagnostic ではない。

## Descriptor Metadata

active または retired の各 code は descriptor を 1 つ持つ。

| Field | 必須 | 互換性上の役割 |
|---|---:|---|
| `code` | yes | 公開 stable identity。割り当て後は immutable。 |
| `meaning_key` | yes | 互換性検証用の安定した非 localized registry key。user-facing identity として表示しない。 |
| `semantic_name` | yes | dot-separated の人間向け名。`meaning_key` が同じで、古い名前を alias として記録するなら rename できる。 |
| `default_severity` | yes | code prefix と一致しなければならない。descriptor の default severity を変更するには、言語仕様が code range を変えない限り新しい code が必要である。policy layer は registry identity の外で warning を build failure に昇格したり consumer-specific presentation を選んだりしてよい。 |
| `phase_family` | yes | 割り当て範囲と一致しなければならない。別 family へ移すには新しい code が必要である。 |
| `summary` | yes | registry audit 用の短い英語 summary。identity を変えずに明確化できる。 |
| `doc_url` | yes | canonical documentation URL または repository-relative documentation target。stable redirect または replacement link を保つなら移動できる。 |
| `status` | yes | `active` または `retired`。 |
| `since` | yes | code を初めて割り当てた version または design task。 |
| `retired_since` | retired の場合 | code を retired にした version または design task。 |
| `replacement_codes` | optional | retired diagnostic を置き換える code。これは提案であり identity は移転しない。 |
| `aliases` | optional | 同じ `meaning_key` の過去の semantic name。 |

message text、localized text、renderer layout、CLI ordering、LSP severity
mapping、fix suggestion wording、explanation text は diagnostic record からの
projection である。これらは registry identity ではない。

初期 registry では、row が明示的に override しない限り次を適用する。

- `meaning_key` は `semantic_name` と同じである。
- `status` は `active` である。
- `since` は `spec-22.7-v1` である。
- `doc_url` は
  `doc/spec/en/22.error_handling_and_diagnostics.md#227-error-code-reference`
  である。
- `aliases`、`retired_since`、`replacement_codes` は空である。
- `summary` は initial-allocation table の `Summary` 値である。

## Deferred Code-Space Gaps

次の surface は task-2 initial allocation の外に明示的に置く。

| Surface | Classification | task-2 disposition |
|---|---|---|
| architecture/internal document に書かれた build、cache、artifact、documentation-extraction diagnostics | `external_dependency_gap` / `spec_gap` | 後続の仕様更新が canonical range と descriptor を追加するまで、code を割り当てず、current record として公開しない。 |
| この crate より前からある lexer、frontend、parser、resolver diagnostics | `external_dependency_gap` / `deferred` | task 16 が実 adoption trigger を記録するまで local diagnostics を触らない。message text から code を推測しない。 |
| `info.thesis`、`info.resolution`、`info.type` などの spec-21 display annotation name | `spec_gap` | `I0001`-`I0099` を reserve する。言語仕様がそれらの semantic name を数値 `DiagnosticCode` に map するまで concrete info code を割り当てない。 |

定義済み initial range の外にある unknown code や descriptor は、現在の registry
では validation failure である。この failure は code space がまだ仕様化されて
いないことを意味し、ad hoc な local range を認めるものではない。

## 割り当て規則

1. diagnostic を生む normative な言語仕様または architecture requirement から、
   かつこの document で既に定義済みの range から phase family を選ぶ。range が
   定義されていない場合は、placeholder を割り当てず code-space gap を記録する。
2. その範囲内で未使用の番号を割り当てる。gap は意図的であり、関連する将来の
   diagnostic のために残してよい。
3. 新しい `meaning_key`、semantic name、default severity、summary、
   documentation target を持つ descriptor を追加する。
4. 割り当て済み descriptor set を固定する registry test を追加または更新する。
5. 外部 adoption placeholder、将来の LSP conversion shape、実 producer/consumer
   seam を持たない driver event のために code を割り当ててはならない。

既存の lexer、frontend、parser、resolver diagnostic を後で移行する場合、その
移行は実 producer output を割り当て済み code に map しなければならない。古い
message string から identity を推測してはならない。

## Retirement Rules

verifier がその diagnostic meaning を emit しなくなったとき、code は retired に
なる。retirement は `status = retired`、`retired_since`、任意の
`replacement_codes` とともに descriptor を保持する。

retired code は historical artifact、cache、log、explanation handle のために
valid なまま残る。新しい current diagnostic は、compatibility mode が historical
data を明示的に読む場合を除き retired code を emit してはならない。その場合でも
record は consuming layer によって historical または stale と mark されなければ
ならない。

retirement は番号を allocator に戻さない。

## Compatibility Validation

registry implementation は locked built-in registry に対して descriptor
compatibility を検証しなければならない。

- code は消えてはならない。削除ではなく `retired` にする。
- retired code を active に戻してはならない。同じ diagnostic meaning が
  retirement 後に復活する場合は、新しい code を割り当て、それを
  `replacement_codes` に列挙する。
- code は prefix、numeric value、phase family、default severity、
  `meaning_key` を変えてはならない。
- semantic-name rename は、古い名前を `aliases` に保持する場合にだけ許される。
- documentation target の移動は、descriptor が同じ diagnostic meaning を指し
  続ける場合にだけ許される。
- `summary` text は identity を変えずに明確化できる。
- 2 つの active descriptor が同じ current `(phase_family, semantic_name)` を共有
  してはならない。`aliases` は同じ code の過去の名前を保存するだけであり、重複
  active semantic name を許可せず、consumer identity path でもない。
- 1 つの `phase_family` の中では、すべての active `semantic_name` とすべての
  active alias 文字列は、combined lookup domain 全体で一意でなければならない。
  current `semantic_name` は別 descriptor の alias を再利用してはならず、alias は
  別 descriptor の current name や alias を再利用してはならない。

invalid または reused diagnostic code は compiler implementation error である。
registry validation test または developer-mode internal error として報告され、
通常の user diagnostic にはならない。

## Initial Allocations

初期 active registry は
`doc/spec/en/22.error_handling_and_diagnostics.md#227-error-code-reference`
の canonical code reference を反映する。Info code は display diagnostic が数値
`DiagnosticCode` とともに normative に列挙されるまで reserved のままとする。

| Code | Semantic name | `PhaseFamily` | 既定 severity | Summary |
|---|---|---|---|---|
| `E0001` | `syntax.unexpected_token` | `Syntax` | Error | Unexpected token in current syntactic context |
| `E0002` | `syntax.malformed_literal` | `Syntax` | Error | Numeric or string literal does not conform to lexical rules |
| `E0003` | `syntax.unexpected_end_of_file` | `Syntax` | Error | File ended with open construct pending |
| `E0010` | `syntax.missing_end` | `Syntax` | Error | Block opened without matching `end` |
| `E0011` | `syntax.unmatched_delimiter` | `Syntax` | Error | Parenthesis, bracket, or `do` without matching close |
| `E0012` | `syntax.reserved_keyword_as_identifier` | `Syntax` | Error | Reserved keyword used as identifier |
| `E0101` | `type.mismatch` | `Type` | Error | Expression type incompatible with required type |
| `E0102` | `type.narrowing_requires_proof` | `Type` | Error | Narrowing coercion without justification |
| `E0103` | `type.sethood.missing` | `Type` | Error | Fraenkel comprehension for type without `sethood` |
| `E0110` | `type.inference_conflict` | `Type` | Error | Conflicting type constraints within an expression |
| `E0120` | `type.mode_mismatch` | `Type` | Error | Mode incompatibility without registered widening |
| `E0121` | `type.attribute_required` | `Type` | Error | Required attribute not registered for the type |
| `E0122` | `type.attribute_contradiction` | `Type` | Error | Attribute combination rejected: mutually exclusive attributes or missing existential cluster |
| `E0201` | `resolve.ambiguous_symbol` | `Resolution` | Error | Two or more equally-ranked overload candidates |
| `E0202` | `resolve.no_viable_overload` | `Resolution` | Error | No candidate survives type-checking |
| `E0203` | `template.argument_omitted_not_inferable` | `Resolution` | Error | Template schema parameter cannot be inferred |
| `E0204` | `resolve.incompatible_refinement_join` | `Resolution` | Error | Same-root redefinitions expose incompatible joined facts |
| `E0301` | `proof.by.search_exhausted` | `Proof` | Error | `by` step: ATP exhausted resource budget |
| `E0302` | `proof.by.missing_fact` | `Proof` | Error | Goal likely provable but required lemma not in scope |
| `E0303` | `proof.obligation.open` | `Proof` | Error | Proof block closed with goals remaining |
| `E0310` | `proof.counterexample.found` | `Proof` | Error | Counterexample model found for the goal |
| `E0320` | `proof.atp.timeout` | `Proof` | Error | All ATP backends timed out |
| `E0321` | `proof.atp.axiom_budget_exceeded` | `Proof` | Error | Axiom set exceeds `max_axioms` limit for the obligation |
| `E0350` | `proof.kernel.unsupported_evidence` | `Proof` | Error | Legacy or backend proof material is unsupported under normal proof policy |
| `E0351` | `proof.kernel.missing_provenance` | `Proof` | Error | Kernel evidence is missing required provenance or context binding |
| `E0352` | `proof.kernel.invalid_substitution` | `Proof` | Error | Explicit substitution evidence failed kernel side conditions |
| `E0353` | `proof.kernel.invalid_sat_refutation` | `Proof` | Error | Kernel-derived SAT refutation check failed |
| `E0401` | `logic.contradictory_axioms` | `Logic` | Error | ATP derived `False` from declared axioms |
| `E0410` | `logic.circular_definition` | `Logic` | Error | Non-recursive definition refers to itself outside an algorithm block |
| `E0411` | `logic.circular_cluster` | `Logic` | Error | Cluster registration creates attribute inheritance cycle |
| `E0420` | `vc.postcondition.return` | `Logic` | Error | `ensures` not provable at `return` site |
| `E0421` | `vc.assert.failed` | `Logic` | Error | `assert` in algorithm body not provable |
| `E0422` | `vc.precondition.call_site` | `Logic` | Error | Callee `requires` clause not provable at call site |
| `E0423` | `vc.loop.establish` | `Logic` | Error | Loop invariant not provable before first iteration |
| `E0424` | `vc.loop.maintain` | `Logic` | Error | Loop invariant not provable to be preserved |
| `E0425` | `vc.loop.decrease` | `Logic` | Error | Termination measure not provably decreasing |
| `E0426` | `vc.recursion.decrease` | `Logic` | Error | Termination measure not provably decreasing at recursive call |
| `E0430` | `logic.cluster.inconsistency` | `Logic` | Error | Cluster registration creates contradiction |
| `W0001` | `warn.unused_variable` | `StructuralWarning` | Warning | Variable declared but never read |
| `W0002` | `warn.unused_definition` | `StructuralWarning` | Warning | Definition never referenced in package |
| `W0003` | `warn.unused_hypothesis` | `StructuralWarning` | Warning | Proof hypothesis never referenced in subsequent steps |
| `W0010` | `warn.deprecated_syntax` | `StructuralWarning` | Warning | Deprecated construct; replacement provided |
| `W0101` | `warn.redundant_hypothesis` | `ProofWarning` | Warning | `by` clause contains fact not used by accepted evidence |
| `W0102` | `warn.externally_attested_proof` | `ProofWarning` | Warning | External backend success without kernel-accepted evidence |
| `W0103` | `proof.citation.unused` | `ProofWarning` | Warning | Explicit citation absent from kernel-accepted `used_axioms` |
| `W0201` | `warn.unreachable_code` | `AlgorithmWarning` | Warning | Statement unreachable under static control-flow analysis |
| `W0202` | `warn.loop_may_not_terminate` | `AlgorithmWarning` | Warning | `terminating` algorithm with unverified loop measure |
| `W0210` | `warn.weakened_postcondition` | `AlgorithmWarning` | Warning | `ensures` weaker than what the verifier can prove |
| `W0301` | `compat.breaking_change` | `CompatibilityWarning` | Warning | Public API change requires a MAJOR bump |
| `W0302` | `compat.feature_addition` | `CompatibilityWarning` | Warning | Backward-compatible API addition requires a MINOR bump |
| `W0303` | `compat.overload_resolution_shift` | `CompatibilityWarning` | Warning | Registration, redefinition, or conditional-cluster change may shift overload/refinement resolution (heuristic MAJOR) |
| `W0304` | `compat.version_bump_insufficient` | `CompatibilityWarning` | Warning | Declared version bump smaller than required |
| `W0305` | `compat.edition_increase` | `CompatibilityWarning` | Warning | Package edition raised; MAJOR by default, review recommended |

## Lookup Contract

`DiagnosticCode` による registry lookup は active code と retired code の
descriptor metadata を返す。current semantic name による lookup は人間向け
tooling と test のためにだけ許され、build tool や consumer の identity path に
してはならない。alias lookup は audit aid だけであり、二つ目の descriptor では
なく、一意な owning `DiagnosticCode` を返す。compatibility validation は
alias/current-name collision を拒否するため、alias lookup は deterministic でなければ
ならない。validation を通っていない registry を caller が渡した場合、alias lookup
は code を選ぶのではなく失敗しなければならない。

unknown code、malformed code string、compatibility validation に失敗した
descriptor は registry failure である。aggregation と rendering は unknown code
の descriptor を捏造してはならない。
