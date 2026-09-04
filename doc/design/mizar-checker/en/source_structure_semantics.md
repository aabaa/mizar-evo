# Source Structure Semantics

> Canonical language: English. Japanese companion:
> [../ja/source_structure_semantics.md](../ja/source_structure_semantics.md).

This module owns the bounded Step 5C.2 semantic checker described by the
[central task contract](../../task_contracts/en/STEP5C2-STRUCTURE-SEMANTICS.md).
It consumes syntax-free records with resolver-backed structure/member identities
and structurally authenticated source-local variable identities, checks
structure definitions, inheritance, constructors, selectors, field updates,
variables, and equality claims, and publishes immutable output. It stops at the
first source-order semantic diagnostic and rejects malformed or unauthenticated
payloads without publication.

Input records use private fields with constructor/getter accessors.  Source
member spellings are retained separately from resolver canonical spellings so
the checker can authenticate an exact resolver identity and origin even when a
structure-field projection's primary spelling comes from its result-type
token.

Variables in this exact slice are not `SymbolEnv` declarations. The producer
uses the private `step5c2/variable` local/FQN namespace, while the checker
requires the canonical module/spelling encoding, unique identity and spelling,
valid source range/order, declared type, and an exact declaration/reference
pair before it uses a variable type.

The only semantic diagnostic keys in this slice are
`structures.definition.duplicate_member` (resolve),
`structures.constructor.missing_field_argument`,
`structures.inherit.diamond_inconsistency`,
`structures.inherit.uncovered_base_member`,
`structures.inherit.unknown_source_member`, and
`structures.selector.unknown_field` (type check).  Exact identity is used for
all types; coercion, defaults, proof search, property implementations, and
overload resolution remain outside this module.

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceStructureType` | `#[non_exhaustive]`; callers tolerate later type forms. |
| `SourceStructureMemberKind` | `#[non_exhaustive]`; callers tolerate later member roles. |
| `SourceStructureInheritanceParent` | `#[non_exhaustive]`; callers tolerate later parent roots. |
| `SourceStructureTerm` | `#[non_exhaustive]`; callers tolerate later term forms. |
| `SourceStructureDiagnosticPhase` | `#[non_exhaustive]`; callers tolerate later phases. |
| `SourceStructurePayloadError` | `#[non_exhaustive]`; callers tolerate later payload failures. |

No exhaustive public enum exceptions are owned by this module.
