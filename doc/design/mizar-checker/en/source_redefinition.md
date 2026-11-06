# Source redefinition targets

Canonical language: English; [Japanese companion](../ja/source_redefinition.md).

## API and ownership

`source_redefinition` owns authenticated declaration-time targets for intrinsic
attribute, binary predicate and unary equals-functor redefinitions.
`SourceOrdinaryRootHandoff` is the Task278 declaration-root prerequisite;
`SourceRedefinitionHandoff` consumes it for the three Task279 source-kind rules.
Both are immutable sealed values using existing original source graphs, resolver
identities, binding contexts and normalization; they add no parallel syntax tree.

## Root identity and pending requests

Eligible roots are all visible earlier ordinary declarations of matching source
kind, notation and arity. Their ordered intrinsic guards must be strictly
sharpened by the Set parameters; an attribute subject is a separate semantic
guard from its explicit resolver pattern arity. Functor return compatibility is
checked separately. Use-site selection, result types and declaration order do
not choose a target. Actual symbols and source sites retain the root relation.
Coherence requests retain their actual owners and remain Pending, with no active
refinement, usable fact, checked proof, equation or accepted equivalence.

## Transaction and boundaries

One common handoff is installed atomically in TypedAst and replayed by
ResolvedTypedAst; all type IDs retain their owning normalization universe.
Declaration-only sources introduce no overload application or selected root.
Imported, template, nominal/attributed, explicit-target and refinement-chain
profiles remain outside this initial boundary. Existing marked-Set/registration
consumers retain their independent prerequisites. Real type-elaboration Cases
inspect the installed root, target, body/type and Pending request associations.
Each admitted definition block contains its parameter declarations followed by
exactly one ordinary declaration or redefinition; declaration labels are distinct.
