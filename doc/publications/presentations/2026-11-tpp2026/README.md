# TPP 2026 Presentation Draft

Working materials for the TPP 2026 presentation on Mizar Evolution.

## Candidate title

**AI時代の定理証明支援系を再考する  
— FOL-native ATP と Mizar Evolution —**

Alternative:

**LLM Thinks, ATP Proves  
— Mizar Evolution における FOL-native Hammer の設計思想 —**

## Purpose

This directory is intentionally a working area rather than part of the language
specification.  The presentation should explain the architectural motivation of
Mizar Evolution, especially the role of first-order automated theorem provers
(ATPs) in an AI-assisted mathematics workflow.

The central question is:

> What becomes possible if a large-scale interactive theorem prover is designed
> from the outset around a first-order logical substrate and a native hammer,
> rather than connecting first-order ATPs through a higher-order translation
> layer?

The talk should connect four threads:

1. the success and architectural cost of higher-order hammer systems;
2. the empirical evidence from MPTP/MizAR on top-level Mizar lemmas;
3. the division of labor between expensive semantic reasoning by LLMs and cheap,
   high-throughput proof search by ATPs;
4. the current Mizar Evolution architecture, where proof search remains outside
   the trusted acceptance boundary.

## Files

- `draft.ja.md` — Japanese narrative draft and proposed slide structure.
- `references.bib` — seed bibliography. Verify metadata before the final talk.

## Instructions for Codex

When turning this draft into slides:

- read `draft.ja.md` first;
- also inspect:
  - `doc/design/architecture/en/00.pipeline_overview.md`
  - `doc/design/architecture/en/08.reasoning_boundary.md`
  - `doc/design/architecture/en/09.atp_interface_protocol.md`
  - `doc/design/architecture/en/10.atp_backend_integration.md`;
- do **not** modify the language specification merely to fit the presentation;
- distinguish established facts, interpretation, and research hypotheses;
- do not compare success percentages across MizAR and Sledgehammer as if the
  benchmarks were identical;
- preserve the distinction between proof search and trusted verification;
- keep the presentation focused on architecture and research questions, not on
  attacking other theorem provers.
