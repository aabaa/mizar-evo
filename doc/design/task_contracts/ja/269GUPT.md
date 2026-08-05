# Task 269GUPT: historical given-use source-type profile

> canonical English: [../en/269GUPT.md](../en/269GUPT.md)。

本historical recordはreview済みcompletion evidenceをcentralizeする。language
behavior、test intent、API、diagnostic、trace、coverage changeをauthorizeしない。

## Identity And Owners

| Field | Historical value |
|---|---|
| Task | `269GUPT` |
| Plans | [checker](../../mizar-checker/ja/00.crate_plan.md#task-index)、[runner](../../mizar-test/ja/00.crate_plan.md#task-index) |
| Documentation prerequisite | `66c75afa2d94108136b73a42ebd70ac355e970b3` |
| Implementation | `c529245138b6d40be65c590ba701fef4f4ea0881` |
| Scope | existing Rust 7 files、checker/private runner各4 tests |
| Durable checker owner | [source type](../../mizar-checker/ja/source_type.md#task-269gupt-frozen-proof-given-use-profile-source-type) |
| Other durable owners | [binding overlay](../../mizar-checker/ja/binding_env.md#task-269gupt-frozen-binding-overlay-boundary)、[lower dependency](../../mizar-checker/ja/source_proof_local_declaration.md#task-269gupt-frozen-dependency-consumer)、[Typed](../../mizar-checker/ja/typed_ast.md#task-269gupt-frozen-typed-ownership)、[Resolved](../../mizar-checker/ja/resolved_typed_ast.md#task-269gupt-frozen-final-ownership)、[private harness](../../mizar-test/ja/harness.md#task-269gupt-frozen-private-harness-route) |
| Historical successor | Task 269GU。capture/exportとTask 270はdeferred |

## Completion Evidence

public checker familyは`SourceProofLocalGivenUseType{Handoff,Producer,Error}`、
errorは`InvalidDependency`、`InvalidBindingEnvironment`、`InvalidSourceType`、
`InvalidInstallation`だった。GUP bindingをby-valueでconsumeし、dependency
fingerprintと`2/2/0`環境を保持、binding 1だけを`Missing`から`Source(84..87)`へ
変更し、binding 0は`Source(14..17)`を保持した。source-type profileは
`2/2/0/0/0/0`、bare builtin `set` rowsは`(0,0,0)`/`(1,1,1)`。3-node arenaは
reserved type、given-use type、root `[0,1]`でroleは
`source.proof-local.given-use.type`だった。

final-LF 128-byte sourceとnormal 54-node/root-53 Surface hashは
`ec15ded78ae96022840a8419a85d74643de3b3737e9a202cbda77ee97aa7c01` /
`c64297ce72e380a2e4146276966e085d780f8b38f2528d5abaa440a50c67db6d`。
Typed/Resolvedは同じoptional boxed compositeをone-shot、same-identity、both-order、
replay、atomic failureで所有した。

private dormant runnerは`SourceProofLocalGivenUseTypeRouteOutput`、mutation seam
（`None`、`WrongDependencyModule`、`WrongTypeRange`、`WrongArenaRoot`、
`WrongArenaKind`）、local error `Task269GUPT reserve type range is missing`を所有し、
public dispatchへ入らなかった。testsはexact payload/
fingerprints、validation precedence、corruption、ownership、replay、isolation、empty
semantic publicationを凍結した。

librariesはchecker/runner `506/568`、productionは`30/174332` / `37/75074`。
path hashesは
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5` /
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`、
content hashesは
`fc85ad8c271614a4474cab3ef6a6d212b168546d1f76d1bc3edb9fa4354378b0` /
`afef82f149a350314a9160685e094e4a1b580d772790cf1c9e2a7efd89d0c870`。
raw/normalized test-list hashesはchecker
`d9c3c7e10b836f1e5ab987bfc54b1c06eaf8af15e2d6f3532fad51a756fca140` /
`9342b51b7e26745f5e04770fe254b8954524dccd45a01ced475b5f097d941cb1`、runner
`30fce970d193edf3a0a84607b6015e017e91f8e6c8f35fc9b10be88e16fdff93` /
`48261f74e202e4496db6e231c335f842942ab3049b61196884984b16cc997c99`。

source-statement route、fixture、sidecar、expectation、trace metadata、Cargo、metadata、
diagnostic、CLI、dispatch、active result、coverage creditは変更なし。occurrence、
condition/fact、guard、capture、goal、proof、acceptance、initial obligation、Core/CFG/VC、
downstream IRを実装しなかった。reviewsは **NO FINDINGS**、全9 gatesはscore capなし
`100/100`だった。

## Migration Boundary

`DOC-269G-INTERMEDIATE-COMPACT`はfrozen source inventory記載のcompletion H3
34節だけを置換できる。全frozen H2 ownerとdurable API/validation/boundary/trace/audit/
deferral textは保持する。
