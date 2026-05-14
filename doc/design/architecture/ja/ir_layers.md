# Architecture: IR Layers

> Canonical language: English. English canonical version: [../en/ir_layers.md](../en/ir_layers.md).

## 目的

Mizar Evo の処理パイプラインで受け渡す IR の層構造を定義する。
この日本語版では、各 IR が何を所有し、何を所有しないかを中心に整理する。

## 中心方針

- 各 IR は、その生成フェーズ完了時点の immutable snapshot として扱う。
- 後続フェーズは前段 IR を直接書き換えず、新しい IR または side table を作る。
- `ResolvedTypedAst` までは source-shaped な構造を保つ。
- `CoreIr` 以降は proof / verification / kernel check に向いた正規化表現にする。
- `VerifiedArtifact` は raw IR dump ではなく、外部 tool 向けの安定 projection とする。

## IR 一覧

| IR | Produced By | Role |
|---|---|---|
| `SourceUnit` | Source Loading | file path, package, source text, line map を保持する |
| `PreprocessedSource` | Preprocessing | comment-stripped lexical text, doc comments, import stubs を保持する |
| `TokenStream` | Lexing | source span 付き token sequence |
| `SurfaceAst` | Parsing | source syntax に近い AST |
| `ResolvedAst` | Name Resolution | names, labels, imports, namespace references が解決された AST |
| `SymbolEnv` | Signature Collection | visible symbols, definitions, overload sets, registration declarations |
| `TypedAst` | Type Checking | inferred types, type facts, coercion candidates |
| `ResolutionTrace` | Registration Resolution | cluster / registration の derivation trace |
| `ResolvedTypedAst` | Cluster / Overload Resolution | final types, overload winners, inserted `qua`, cluster facts |
| `CoreIr` | Elaboration | surface sugar を除去した logical representation |
| `ControlFlowIr` | Algorithm Preparation | algorithm body の CFG, locals, contracts, ghost effects |
| `VcIr` | VC Generation | prover-independent verification condition |
| `AtpProblem` | ATP Translation | backend-neutral prover problem |
| `ProofCertificate` | ATP Backend | backend が返す proof evidence |
| `VerifiedArtifact` | Artifact Emit | LSP, docs, downstream packages 向けの安定 metadata |

## 重要な境界

### `PreprocessedSource`

Preprocessing は comments, doc comments, import pre-scan を扱う。
annotation syntax は parser が所有するため、preprocessing では別 channel に切り出さない。

### `ResolvedTypedAst`

source-shaped な semantic AST の最終形。
LSP hover、`@show_resolution`、documentation metadata は主にここから作る。

### `CoreIr`

kernel と VC generation が扱いやすいように、surface syntax を落とした表現。
soft type annotation の erasure は暗黙にせず、明示的に扱う。

### `VcIr`

Mizar 側の obligation generation と ATP translation の境界。
具体的な TPTP / SMT-LIB text はまだ持たない。

### `VerifiedArtifact`

外部公開される安定 schema。
raw AST arena や kernel-internal proof state は含めない。

## Cross-Layer Identity

`SourceId`, `ModuleId`, `ItemId`, `NodeId`, `ExprId`, `FactId`, `VcId` を使い分ける。
同一 source / lockfile / toolchain / verifier settings では deterministic になるようにする。

## Incremental Build

IR 層は cache と invalidation の単位にもなる。
例えば import edit は active lexicon 以降を無効化し、registration edit は `RegistrationIndex`, `ResolutionTrace`, `ResolvedTypedAst`, 関連 VC を無効化する。
