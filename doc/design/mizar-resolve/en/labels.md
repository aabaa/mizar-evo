# Module: labels

> Canonical language: English. Japanese companion: [../ja/labels.md](../ja/labels.md).

Status: task R-017 specifies the resolver-owned label-resolution contract for
task R-018. It records the dedicated label scope families, proof-block nesting
rules, forward-reference policy, citation lookup behavior, and normalized
label-origin paths used later by proof, checker, VC, and artifact phases. This
is a design-only task: executable label collection and `LabelRefTable`
population land in R-018.

## References

This design derives the resolver-owned label contract from:

- architecture 03 "Label Resolution Is Scoped Separately from Item Resolution";
- spec chapter 15 statement labels, proof organization, justification forms,
  and scoping rules;
- spec chapter 16 theorem labels, proof-block visibility, and citation forms;
- spec chapter 22 diagnostic payload requirements and the current resolver-code
  `spec_gap`;
- architecture 22 `ObligationAnchor` provenance requirements;
- resolver-local `resolved_ast.md`, `env.md`, `imports.md`, `names.md`, and
  `declarations.md`.

## Purpose

The labels phase resolves label declarations and citation use sites after
imports, declaration shells, and namespace lookup are available, but before
proof checking, type checking, ATP dispatch, template instantiation, or
obligation generation. It consumes source-shaped syntax and resolver-owned
indexes, then records explicit label outcomes in `ResolvedAst` and visible
label projections in `SymbolEnv`.

Inputs:

- `SurfaceAst` for the current module;
- resolved imports and namespace lookup behavior from `imports.md` and
  `names.md`;
- declaration shells from `declarations.md`;
- module and dependency label projections from source-backed fixtures or
  summaries when available;
- syntax recovery markers and source ranges owned by `mizar-syntax`.

Outputs:

- label declaration records for represented theorem, definition, proof-step,
  and registration labels;
- `LabelIndex` entries and visible label projections;
- `LabelRefTable` entries for resolver-attempted citation use sites;
- explicit unresolved and ambiguous label records;
- crate-local/internal label diagnostic records with deterministic ordering.

## Boundary

The labels phase may:

- classify label declarations by label scope family and source role;
- resolve simple, qualified, and grouped citation labels;
- decide label visibility, duplicate-label conflicts, and forward-reference
  failures;
- preserve normalized provenance for downstream `ObligationAnchor` label
  hints and dependency slices.

It must not:

- prove a theorem, proof step, definition correctness condition, or
  registration condition;
- generate `ObligationAnchor` values or verification conditions;
- run ATP, select premises, or expand template arguments semantically;
- type-check definition bodies, registrations, or proof statements;
- choose an overload winner for ordinary names;
- invent public user-facing resolver diagnostic codes.

## Label Scope Families

Labels are not ordinary symbols. A label declaration belongs to one
resolver-owned family:

| Family | Sources | Visibility surface | Downstream consumers |
|---|---|---|---|
| theorem / lemma result | `theorem` and `lemma` items | current module after declaration; exported table when public | citations, artifacts, ATP premise selection |
| definition | definition and redefinition labels | defining item and source correctness-role provenance | checker, VC generation, diagnostics |
| proof step | labeled propositions, assumptions, conclusions, cases, `now` blocks, iterative equality chains | enclosing reasoning block and nested child blocks after declaration | proof justification and local context |
| registration | registration and reduction labels | registration item and registration trace | checker, kernel replay, diagnostics |

The expected label family comes from the use-site syntax. A `by` citation may
refer to a local proof-step label or a module theorem/lemma label. Definition
and registration label references are resolved only in syntax positions that
expect those families, such as correctness or registration trace sites. If a
use site can legally accept multiple families and more than one visible
candidate remains, the resolver records deterministic ambiguity instead of
choosing one by source order.

## Proof-Block Scope

Proof labels are scoped to reasoning blocks, not to the ordinary symbol
namespace.

- A label attached to a statement becomes visible only after the statement is
  complete.
- A label attached to `now ... end` belongs to the enclosing block and becomes
  visible only after that block closes.
- Labels declared inside a nested proof, case, suppose, or diffuse reasoning
  block are visible to that block and nested child blocks, but not to the
  enclosing block after the child block closes.
- Enclosing proof labels are visible inside nested child blocks unless the
  nested construct starts a separate module-level item.
- Inner-scope label shadowing is forbidden by spec chapter 15. A new label that
  repeats any label visible from the current label scope is a duplicate or
  conflict, not a shadowing declaration.
- Same-scope duplicate labels are duplicate-label conflicts.

The resolver must keep resolving the rest of the module after a duplicate or
conflicting label. It records the conflict as crate-local/internal diagnostic
data and keeps enough candidate provenance for later diagnostics and editor
navigation.

## Declaration Point And Forward References

Label lookup is declaration-point sensitive.

- A label is visible only after its declaring statement, item, or block is
  complete.
- A citation to a later label in the same proof block is unresolved.
- A theorem or lemma label is visible to later module items only after the
  theorem or lemma item is complete. A citation to a later theorem or lemma in
  the same module is unresolved.
- Definition and registration labels are visible at resolver-visible
  correctness-role and trace-provenance positions according to the enclosing
  item structure, but not before their declaring syntax has been collected.
- A self-reference from a label's own declaration body is unresolved unless a
  later proof/checker phase defines a separate recursive rule. R-017 defines no
  such rule.

Forward-reference failures are represented as explicit
`UnresolvedLabelRef`-style outcomes with the attempted spelling, use-site
range, and expected label family. They do not fabricate a label origin path.

## Citation Lookup

Simple unqualified citation lookup is label-family specific:

1. visible proof-step labels in the current proof block chain;
2. current-module theorem/lemma labels visible at the use site;
3. imported public theorem/lemma labels made visible through resolved imports
   and exports.

Because inner proof-label shadowing is forbidden, more than one proof-step
candidate for the same spelling is a conflict record. If an unqualified
citation still has multiple legal candidates after family and visibility
filtering, the resolver records `AmbiguousLabelRef` with candidates sorted by
normalized origin path, kind, and source range.

Qualified citations split namespace and label lookup:

1. Resolve the module prefix through the namespace rules in `names.md`.
2. Resolve the final label spelling in the target module's exported label
   table.

Citation prefixes are namespace paths only. The R-016 dot-chain finalization
rules for local-term shadowing, selectors, and `DeferredSelector` records do
not apply to simple, qualified, grouped, or bulk citation prefixes.

Grouped citations use the same resolved module prefix for each grouped label
and produce one label-resolution outcome per concrete grouped item. A failure
in the shared module prefix is recorded once as an unresolved namespace/module
dependency and each grouped item records a dependent unresolved label outcome.

Bulk citations (`module_path.*`) are not permission to fabricate individual
label entries. If the target module's exported theorem/lemma label table is
available, the resolver may expand the bulk citation into the deterministic
public theorem/lemma label set required by spec chapter 16. If that table is
not available, the resolver records an unresolved module-label-set dependency
for the citation container; it does not invent synthetic `LabelRef` entries.

Template arguments attached to citations are carried as use-site provenance for
later template/proof phases. R-017 and R-018 do not validate, instantiate, or
type-check those arguments.

## Label Origin Paths

`LabelOriginPath` is the resolver-owned stable identity used in `LabelRef`,
`LabelIndex`, dependency edges, and later `ObligationAnchor` label hints. It is
not proof evidence and must not replace proof/checker-owned identities.

A normalized label-origin path contains enough structure to be stable under
formatting and unrelated local edits:

- canonical `ModuleId` or module path;
- label family and primary spelling;
- defining item kind and source contribution;
- source-shaped structural path to the declaring statement, proof block,
  definition clause, or registration clause;
- for proof labels, the enclosing theorem or proof owner plus proof-block and
  local statement path;
- for definition and registration labels, the source correctness-role or trace
  provenance when available without checker-owned semantics.

Source ranges and `SurfaceNodeId`s remain provenance for diagnostics and editor
navigation. They are not canonical label identity by themselves.

## Recovery And Diagnostics

Recovered or malformed label syntax is retained as unresolved or recovered
label records when the surrounding source shape is still represented. The
resolver must not panic on recovered proof or declaration subtrees.

Diagnostic records remain crate-local/internal while R-G001 is open. Label
diagnostics must preserve:

- primary use-site or declaration range;
- duplicate/conflicting declaration ranges;
- expected label family;
- failed namespace or unresolved import dependency for qualified citations;
- deterministic candidate lists for ambiguity.

No public numeric resolver diagnostic code is assigned by this module spec.

## Determinism

Label collection and resolution are deterministic:

- declaration traversal follows stable source order;
- table ids are insertion-order ids from deterministic traversal;
- candidate lists are sorted by `LabelOriginPath`, label kind, and source
  range;
- diagnostics are sorted by primary source range, diagnostic class, and stable
  origin path;
- debug rendering uses normalized origin paths and never raw hash-map order.

## Test Obligations

R-017 adds no executable tests because it is documentation-only. R-018 must add
unit tests for:

- proof-block visibility and nested-block confinement;
- duplicate/conflicting labels across visible scopes, including the
  spec-forbidden inner-scope shadowing case;
- rejection of forward references to later labels;
- simple, qualified, and grouped citation lookup where the parser already
  produces the relevant syntax;
- deterministic `LabelRefTable`, `LabelIndex`, and diagnostic ordering.

Semantic `.miz` corpus coverage and traceability metadata are introduced by
task R-023 under the existing R-G002 `test_gap`.
