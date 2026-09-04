# Source Structure Semantics

> Canonical language: English. Japanese companion:
> [../en/source_structure_semantics.md](../en/source_structure_semantics.md).

この module は bounded Step 5C.2 semantic checker を所有する。syntax-free record、
resolver-backed structure/member identity、および構造的に認証した source-local variable
identity を受け取り、structure definition、inheritance、constructor、selector、field
update、variable、equality claim を検査し、immutable output を公開する。
source-order の最初の semantic diagnostic で停止し、malformed または未認証 payload は
publication 前に拒否する。

入力 record の field は非公開で、constructor と getter のみを提供する。source に書かれた
member spelling と resolver の canonical spelling は分離して保持するため、structure-field
projection の primary spelling が結果型 token 由来でも、正確な resolver identity と origin を認証できる。

この exact slice の variable は `SymbolEnv` declaration ではない。producer は private な
`step5c2/variable` local/FQN namespace を使用し、checker は variable type を使用する前に
canonical module/spelling encoding、identity と spelling の一意性、正しい source
range/order、declared type、および declaration/reference の完全一致を要求する。

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceStructureType` | `#[non_exhaustive]`; later type forms を許容する。 |
| `SourceStructureMemberKind` | `#[non_exhaustive]`; later member roles を許容する。 |
| `SourceStructureInheritanceParent` | `#[non_exhaustive]`; later parent roots を許容する。 |
| `SourceStructureTerm` | `#[non_exhaustive]`; later term forms を許容する。 |
| `SourceStructureDiagnosticPhase` | `#[non_exhaustive]`; later phases を許容する。 |
| `SourceStructurePayloadError` | `#[non_exhaustive]`; later payload failures を許容する。 |

この module が所有する exhaustive public enum exception はない。
