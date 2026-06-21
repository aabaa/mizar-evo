# mizar-resolve: Recovered Syntax Policy

> Canonical language: English. Japanese companion: [../ja/recovery.md](../ja/recovery.md).

Status: implemented for resolver tasks R-022 and earlier. This document is
derived from the `mizar-syntax` recovery contract and does not introduce new
parser recovery behavior.

## Purpose

`mizar-syntax` owns recovery production, recovery kinds, syntax diagnostics,
and the `recovered` flag on `SurfaceAst` nodes and tokens. `mizar-resolve`
consumes those markers so semantic phases can continue deterministically
without treating recovered syntax as semantically valid source.

The resolver policy is:

- preserve represented source shape when it is useful for later navigation,
  invalidation, or downstream degraded facts;
- avoid fabricating identities, signatures, or relation edges when recovery
  hides required source information;
- keep parser diagnostics as syntax diagnostics instead of converting them
  into public resolver diagnostic codes;
- suppress dependent semantic diagnostics that would only repeat a recovered
  syntax root.

## Stage Disposition

| Stage | Recovered input disposition | Diagnostic rule |
|---|---|---|
| import path candidates | Retain the candidate as an unresolved import with `RecoveredSyntax` when the parser represented the directive. Do not add graph edges from recovered candidates. | No public resolver code is assigned. Later name diagnostics may depend on unresolved imports, but recovered import candidates are not promoted to independent user-facing resolver diagnostics. |
| declaration shells and export shells | Retain represented declaration/export shells and mark them recovered when the shell, a transparent wrapper, or a descendant contains recovery. Shells without deterministic identity stay shell-only. | Declaration collection itself emits no diagnostics. |
| namespace paths | Retain unresolved namespace records with `RecoveredSyntax`; do not resolve through recovered namespace candidates. | Recovered namespace records are not diagnostic roots in the internal name report. |
| ordinary names and dot chains | Keep `NameRefTable` entries unresolved or checker-deferred as appropriate. A recovered dot-chain marks the reference origin recovered even if the original candidate only carried a local recovered flag. | Recovered reference origins are omitted from internal name diagnostic roots and cascades. |
| label references | Keep unresolved label-reference table entries when the reference origin is recovered or the spelling is empty. | Label-reference failures have no separate diagnostic report in R-022. |
| label declarations | Keep label projections and label index facts when a deterministic origin path remains available, but do not use recovered projections as candidates for clean label-reference lookup. | Recovered label projections are excluded from duplicate/conflicting-label diagnostics. |
| symbol declarations | Keep symbols and definitions when identity data remains deterministic, but use recovered origins, local-only export status, and malformed `recovered-shell` signatures. Context-only recovered shells remain shell-only. | Recovered symbol projections are excluded from duplicate and illegal-overload diagnostics. Their definition conflict is `RecoveredShell` only. |
| lexical summaries | Seed only projections marked as lexer-visible notation by the parser-backed extractor when the collected shell is not recovered and is export-visible. | No resolver diagnostic is emitted for skipped lexical summary seeds. |

## Boundary Rules

- Resolver code must not inspect skipped source text to infer missing syntax.
- Resolver code must not create parser recovery nodes or change
  `SyntaxRecoveryKind`.
- Resolver diagnostics remain crate-local/internal while R-G001 is open; this
  policy does not allocate public resolver diagnostic codes.
- Downstream checker/type/proof phases may reject or skip recovered facts, but
  the resolver must preserve enough provenance for them to recognize degraded
  input.

## Tests

R-022 unit coverage verifies that:

- recovered declaration and symbol inputs do not panic collection;
- recovered namespace/name/dot-chain inputs are retained in resolver tables but
  do not become internal name diagnostic roots;
- recovered label declarations do not produce duplicate/conflict diagnostics;
- recovered symbol declarations do not produce duplicate or illegal-overload
  diagnostics, while `RecoveredShell` conflict metadata remains available.
