# Task 269GUPT: Historical Given-Use Source-Type Profile

> Canonical language: English. Japanese companion: [../ja/269GUPT.md](../ja/269GUPT.md).

This historical record centralizes already-reviewed completion evidence. It
does not authorize language behavior, test intent, API, diagnostic, trace, or
coverage changes.

## Identity And Owners

| Field | Historical value |
|---|---|
| Task | `269GUPT` |
| Plans | [checker](../../mizar-checker/en/00.crate_plan.md#task-index), [runner](../../mizar-test/en/00.crate_plan.md#task-index) |
| Documentation prerequisite | `66c75afa2d94108136b73a42ebd70ac355e970b3` |
| Implementation | `c529245138b6d40be65c590ba701fef4f4ea0881` |
| Scope | Seven existing Rust files; four checker and four private runner tests |
| Durable checker owner | [source type](../../mizar-checker/en/source_type.md#task-269gupt-frozen-proof-given-use-profile-source-type) |
| Other durable owners | [binding overlay](../../mizar-checker/en/binding_env.md#task-269gupt-frozen-binding-overlay-boundary), [lower dependency](../../mizar-checker/en/source_proof_local_declaration.md#task-269gupt-frozen-dependency-consumer), [Typed](../../mizar-checker/en/typed_ast.md#task-269gupt-frozen-typed-ownership), [Resolved](../../mizar-checker/en/resolved_typed_ast.md#task-269gupt-frozen-final-ownership), and [private harness](../../mizar-test/en/harness.md#task-269gupt-frozen-private-harness-route) |
| Historical successor | Task 269GU; capture/export and Task 270 remained deferred |

## Completion Evidence

The public checker family was
`SourceProofLocalGivenUseType{Handoff,Producer,Error}` with error variants
`InvalidDependency`, `InvalidBindingEnvironment`, `InvalidSourceType`, and
`InvalidInstallation`. It consumed the GUP binding handoff by value, preserved
its dependency fingerprint and `2/2/0` environment, changed only binding 1
from `Missing` to `Source(84..87)`, and retained binding 0 as
`Source(14..17)`. The source-type profile was `2/2/0/0/0/0`, with two bare
builtin-`set` rows `(0,0,0)` and `(1,1,1)`. The three-node arena contained the
reserved type, given-use type, and root `[0,1]`, with role
`source.proof-local.given-use.type`.

The final-LF 128-byte source and normal 54-node/root-53 Surface hashes were
`ec15ded78ae96022840a8419a85d74643de3b3737e9a202cbda77ee97aa7c01` and
`c64297ce72e380a2e4146276966e085d780f8b38f2528d5abaa440a50c67db6d`.
Typed and Resolved each owned the same optional boxed composite with one-shot,
same-identity, both-order, replay, and atomic-failure enforcement.

The private dormant runner owned
`SourceProofLocalGivenUseTypeRouteOutput`, its mutation seam (`None`,
`WrongDependencyModule`, `WrongTypeRange`, `WrongArenaRoot`,
`WrongArenaKind`), and local error
`Task269GUPT reserve type range is missing`. It never entered public dispatch. Tests froze exact
payload and fingerprints, validation precedence, corruption, ownership,
replay, isolation, and empty semantic publication.

Libraries were checker/runner `506/568`; production was `30/174332` and
`37/75074`. Path hashes were
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5` /
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`,
and content hashes were
`fc85ad8c271614a4474cab3ef6a6d212b168546d1f76d1bc3edb9fa4354378b0` /
`afef82f149a350314a9160685e094e4a1b580d772790cf1c9e2a7efd89d0c870`.
Raw/normalized test-list hashes were checker
`d9c3c7e10b836f1e5ab987bfc54b1c06eaf8af15e2d6f3532fad51a756fca140` /
`9342b51b7e26745f5e04770fe254b8954524dccd45a01ced475b5f097d941cb1`
and runner
`30fce970d193edf3a0a84607b6015e017e91f8e6c8f35fc9b10be88e16fdff93` /
`48261f74e202e4496db6e231c335f842942ab3049b61196884984b16cc997c99`.

No source-statement route, fixture, sidecar, expectation, trace metadata,
Cargo, metadata, diagnostic, CLI, dispatch, active result, or coverage credit
changed. No occurrence, condition/fact, guard, capture, goal, proof,
acceptance, initial obligation, Core/CFG/VC, or downstream IR behavior was
implemented. Reviews ended **NO FINDINGS**; all nine gates passed without a
score cap at `100/100`.

## Migration Boundary

`DOC-269G-INTERMEDIATE-COMPACT` may replace only the 34 completion H3 sections
listed in its frozen source inventory. Every frozen H2 owner and all durable
API, validation, boundary, trace, audit, and deferral text remain in place.
