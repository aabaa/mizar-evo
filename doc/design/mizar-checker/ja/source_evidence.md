# Source-evidence handoff

> Canonical language: English. English canonical:
> [../en/source_evidence.md](../en/source_evidence.md).

## Purpose と authority

public `source_evidence` module は Checker Task 251 を実装する。syntax owner
である runner から checker へ、raw syntax を import せず source-derived
evidence request と authenticated dependency reference を transport する。
canonical authority slice は Chapter 03 §§3.2-3.6、05 §§5.3-5.5、
06 §§6.2-6.5、07 §§7.2-7.3/7.8-7.8.1、08 §§8.1-8.3、
13 §§13.4.2/13.5/13.6、17 §§17.1/17.3.4/17.7-17.8、19
§§19.2.1-19.2.2/19.4.2/19.6.2 である。source/spec audit は bounded
implementation を MC-G016、MC-G018、MC-G026 配下で追跡する。

この module は transport-only である。`Requested`、`Missing`、`Rejected`、
`Supplied` は dependency-input state であり、truth、acceptance、
consumability、satisfaction、proof ではない。Task 251 は existential gate を
evaluate せず、fact を create せず、inheritance path や coercion を select
せず、type normalization や accepted semantic result publication を行わない。

## Public model

`SourceEvidenceHandoffInput` は一つの `SourceId`/`ModuleId`、ordered
`SourceEvidenceRequestInput` row、ordered `SourceEvidenceResponseInput`
reference を持つ。`SourceEvidenceProducer::build` は transaction を以下へ
authenticate する。

- Task-249 `SourceTypeApplicationHandoff`
- optional Task-250 `SourceAttributeHandoff`
- resolver-authenticated `SymbolEnv`
- existing `TypeFactTable`
- caller-supplied `SourceEvidenceDependencyCatalog`

成功時は dense `SourceEvidenceRequestTable` /
`SourceEvidenceResponseTable` を持つ immutable `SourceEvidenceHandoff` を
publish する。`SourceEvidenceRequestId` / `SourceEvidenceResponseId` は
table-local dense identity である。`SourceEvidenceRequest` /
`SourceEvidenceResponse` は read-only accessor を expose し、`debug_text()`
は transaction を deterministic に render する。

各 request は `owner: TypedSiteRef`、`site: TypedSiteRef`、
`source_range: SourceRange`、application `source_ordinal`、
`SourceEvidenceRecovery`、`SourceEvidenceRequestKind`、
`SourceEvidenceInputState`、`SourceEvidenceRequestOrigin` を記録する。
public request kind は `ModeExpansion`、`StructureInhabitation`、
`AttributedTypeInhabitation`、`Sethood`、`NonEmptiness`、
`InheritancePath`、`CoercionViability`。Task 251 が emit するのは最初の
3 kind だけである。
`SourceEvidenceRequestOrigin::SourceTypeApplication` は Task-249
application/expression pair と optional Task-250 attribute chain を
authenticate する。

`SourceEvidenceResponseKey` は opaque nonempty dependency identity。
`SourceEvidenceDependencyRecord` は key を authenticated parent request、
`SourceEvidenceResponseDisposition`、`SourceEvidenceResponseProvenance`、
optional `SourceEvidenceResponsePayload` に associate する。
`SourceEvidenceDependencyCatalog` は construction、lookup、iteration、
length、emptiness check を expose するが、published handoff へ whole-copy
されない。

response payload vocabulary は `ModeExpansion(ModeExpansion)`、
`StructureBaseEvidence(ExistentialGateBaseEvidence)`、
`ExistentialGate(ExistentialGateInput)`、`TypeFact(TypeFactId)`。
`SourceEvidenceResponseDisposition` は `Rejected` / `Supplied`、
`SourceEvidenceResponseProvenance` は `ExplicitInput` /
`ExternalDependency`、`SourceEvidenceRecovery` は `Normal` / `Degraded`。
これら public enum と `SourceEvidenceError` は `#[non_exhaustive]` である。

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceEvidenceRequestKind` | `#[non_exhaustive]`。callerはlater evidence-request familyを許容する。 |
| `SourceEvidenceInputState` | `#[non_exhaustive]`。callerはlater transport input stateを許容する。 |
| `SourceEvidenceRequestOrigin` | `#[non_exhaustive]`。callerはlater authenticated source ownerを許容する。 |
| `SourceEvidenceResponseDisposition` | `#[non_exhaustive]`。callerはlater dependency dispositionを許容する。 |
| `SourceEvidenceResponsePayload` | `#[non_exhaustive]`。callerはlater syntax-free dependency payloadを許容する。 |
| `SourceEvidenceResponseProvenance` | `#[non_exhaustive]`。callerはlater authenticated provenance classを許容する。 |
| `SourceEvidenceRecovery` | `#[non_exhaustive]`。callerはlater recovery classを許容する。 |
| `SourceEvidenceError` | `#[non_exhaustive]`。callerはvalidation failureをexhaustive matchしない。 |

この module が所有する exhaustive public enum exception はない。

## Validation と atomicity

request は dense かつ Task-249 application order を保つ。unattributed
application は root expression を origin、expression site を owner、head
site を request site、expression range、application ordinal、mapped
expression recovery を使う。builtin set/object head は request を emit
しない。authenticated resolver `Mode` / `Structure` head は exactly one
`ModeExpansion` / `StructureInhabitation` request を emit する。

attributed application は application root expression、chain site/range、
application ordinal、mapped chain recovery を使う。chain はその expression
を name し、独自の dense Task-250 ordinal で独立に authenticate される。
request kind は underlying head にかかわらず常に
`AttributedTypeInhabitation`。`NodeRecoveryState::Normal` は `Normal`、
`Recovered` / `Degraded` は `Degraded` へ map する。

`Requested` / `Missing` は response 0、`Rejected` は no-payload
authenticated response exactly 1、`Supplied` は request kind に適切な
payload を持つ authenticated response 1 件以上である。catalog key は全て
nonempty/unique、各 response は matching key を一度だけ consume し、全
catalog record が consume される。missing key、duplicate reuse、
cross-request/stale record、wrong disposition/payload combination、dangling
fact、gate owner/range/recovery/guard-fact mismatch は
`SourceEvidenceError` となる。

construction は atomic である。table identity/parent/order、source/module、
owner/site/range/ordinal/recovery、origin、symbol kind、response cardinality、
catalog association、payload kind、fact reference、gate association を
publication 前に validate する。invalid input は partial handoff や
transport-level `Rejected` へ変換されない。

## Ownership と consumer

`TypedAst` は `Option<SourceEvidenceHandoff>` を own し、
`TypedAst::with_source_evidence` が typed source/module と existing fact に
対して installation を validate する。`ResolvedTypedAst` は handoff を
clone-preserve するだけで、`source_evidence()` getter を expose する。

raw `SurfaceAst`、`SurfaceNodeId`、`SyntaxKind` は private `mizar-test`
concern のままである。private
`runner::type_elaboration::source_evidence` leaf は Task-249/250 extractor を
reuse し、exact request association/dispatch だけを own する。syntax
traversal は duplicate しない。

real Task-251 selector は以下の 3 件だけである。

1. `fail_type_elaboration_source_type_application_payload_001`
2. `fail_type_elaboration_imported_attribute_gap_001`
3. `fail_type_elaboration_attributed_reserve_gap_001`

合計で response 0 の `Missing` request 10 件、すなわち mode-expansion 5、
structure-inhabitation 3、attributed-type-inhabitation 2 を publish する。
broad Task-249 case だけが
`type_elaboration.checker.source_evidence.dependency_input_missing` /
`source_evidence_dependency_input_missing` へ advance し、attributed 2 case
は existing evidence-query outcome を preserve する。

## Verification と deferral

checker unit test は four states、dense order、exact Task-249/250
association、distinct application/chain ordinal、state/response cardinality、
catalog/payload corruption、symbol authentication、gate/fact corruption、
deterministic rendering、transactional typed-AST installation を cover する。
runner test は real `.miz` extraction を使い exact selector/cardinality、
final `TypedAst`/`ResolvedTypedAst` ownership、production consumer を通る
four-state injection、corruption、determinism を cover する。

`.miz` source は追加しない。covered trace requirement
`spec.en.checker.type_elaboration.source_evidence_request_payload` 一件が
existing expectation sidecar 3 件を own する。broader source site、semantic
evidence interpretation、fact creation、gate evaluation、accepted
registration/artifact status、Core/CFG/VC output、Tasks 252+、Steps 6/7 は
explicit owner へ deferred のままである。
