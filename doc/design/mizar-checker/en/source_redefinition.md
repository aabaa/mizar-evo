# Source redefinition targets

Canonical language: English; [Japanese companion](../ja/source_redefinition.md).

## API and ownership

`source_redefinition` owns authenticated declaration-time targets for intrinsic
attribute, binary predicate and unary equals-functor redefinitions.
`SourceOrdinaryRootHandoff` is the Task278 declaration-root prerequisite;
`SourceRedefinitionHandoff` consumes it for the three Task279 source-kind rules.
Both are immutable sealed values using existing original source graphs, resolver
identities, binding contexts and normalization; they add no parallel syntax tree.
`SourceRedefinitionInput::from_source` retains ordered targets, explicit-label
tokens or omission, bound-target diagnostic anchors, notation relations and
Pending coherence inputs. `build_with_input` authenticates the complete input
against fresh source derivation before sealing it in the existing handoff.

## Root identity and pending requests

Eligible roots are all visible earlier ordinary declarations of matching source
kind, notation and arity. Their ordered intrinsic guards must be strictly
sharpened by the Set parameters; an attribute subject is a separate semantic
guard from its explicit resolver pattern arity. Functor return compatibility is
checked separately. Use-site selection, result types and declaration order do
not choose a target. Actual symbols and source sites retain the root relation.
An explicit unqualified local declaration label selects its eligible ordinary
root; omission requires a unique eligible root and an invalid label never falls back.
Coherence requests retain their actual owners and remain Pending, with no active
refinement, usable fact, checked proof, equation or accepted equivalence.

## Relation and selection transport

An ordinary definition block may end with one bare attribute synonym or binary
predicate antonym using the same ordered formal loci. Both notation patterns
and the earlier ordinary root are authenticated; alias use remains deferred.
A final Object reserve and reflexive equality theorem may contain one unary
ordinary functor application. The ordinary-root handoff seals its collection,
expansion, viability and specificity outputs before redefinition binding.
The redefinition intake authenticates the bound root, source signature and
Pending refinement candidate in the root normalization domain. Pending inputs
yield an empty active-refinement set for the existing selector; the exposed
result retains the ordinary root type with no evidence or inserted views.
No comparison or priority edge is created. The theorem proof remains unchecked.
The sealed input and nested selection output travel through the same AST slots.
Final assembly rejects exposed results on redefinition source without that seal.

## Transaction and boundaries

One common handoff is installed atomically in TypedAst and replayed by
ResolvedTypedAst; all type IDs retain their owning normalization universe.
Declaration-only sources introduce no overload application or selected root.
Imported, template, nominal/attributed, qualified-target and refinement-chain
profiles remain outside this initial boundary. Existing marked-Set/registration
consumers retain their independent prerequisites. Real type-elaboration Cases
inspect the installed root, target, body/type and Pending request associations.
Each admitted definition block contains its parameters followed by exactly one
ordinary declaration or redefinition, optionally the ordinary declaration's alias;
declaration labels are distinct.
