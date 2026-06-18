# mizar-doc TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

Module specs do not exist yet; each is written by its own spec task (English
and Japanese in the same change) before the implementation tasks that cite it.
Per [internal 07](../../internal/en/07.crate_module_layout.md) this crate owns
both documentation rendering **and** extraction (the architecture-13 module
list predates that consolidation and named a separate `mizar-extract`); the
crate refines architecture 13 and internal 05.

| Module | Spec | Source | Status |
|---|---|---|---|
| artifact_reader | `artifact_reader.md` (task 2) | `src/artifact_reader.rs` | [ ] |
| doc_build | `doc_build.md` (task 4) | `src/doc_build.rs` | [ ] |
| comments | `comments.md` (task 6) | `src/comments.rs` | [ ] |
| links | `links.md` (task 9) | `src/links.rs` | [ ] |
| math | `math.md` (task 11) | `src/math.rs` | [ ] |
| render | `render.md` (task 13) | `src/render.rs` | [ ] |
| extract_select | `extract_select.md` (task 16) | `src/extract_select.rs` | [ ] |
| runtime_ir | `runtime_ir.md` (task 18) | `src/runtime_ir.rs` | [ ] |
| extract_backend | `extract_backend.md` (task 21) | `src/extract_backend.rs` | [ ] |
| publisher | `publisher.md` (task 23) | `src/publisher.rs` | [ ] |

`mizar-doc` implements pipeline phase 16: verified artifacts and doc comments
in, rendered documentation, search indexes, and extracted runtime code out.
Phase 16 is a consumer phase: it reads published artifacts and never re-runs
semantic analysis, never influences proof validity, and its outputs can be
deleted and regenerated freely. Documentation may degrade gracefully on
presentation errors; extraction must instead reject unsupported runtime
constructs.

Dependency order: `artifact_reader` → `doc_build` → `comments` → `links` /
`math` → `render` (documentation strand), then `extract_select` →
`runtime_ir` → `extract_backend` (extraction strand) → `publisher`.

Each task below is deliberately small — one module spec, or one behavior slice
of one module — so that a single task can be implemented, tested, and
committed autonomously without holding the rest of the crate in flight.

## Crate Prerequisites

The crate depends on `mizar-session` and `mizar-artifact` (schemas and
readers). Real end-to-end inputs need phase-15 emission
(`mizar-artifact` task 17); until then, fixture artifacts drive development.
Architecture: [13.documentation_and_extraction.md](../../architecture/en/13.documentation_and_extraction.md);
internal: [05](../../internal/en/05.documentation_extraction.md).

## Resolved And Open Decisions

- **Extraction lives in this crate: resolved by internal 07.** The
  architecture-13 module list named `mizar-extract`; internal 07
  consolidates rendering and extraction here. If extraction grows large, a
  split decision will be raised and registered at the top level then.
- **Doc-comment source: open, resolved by task 6.** Decide how doc comments
  reach phase 16: projected into artifacts at emission (default candidate,
  honoring the consumer-phase rule) or re-read from `PreprocessedSource`
  via `mizar-frontend`. The decision sets whether this crate gains a
  frontend dependency.
- **First extraction target: open, resolved by task 21.** One target
  backend first, then generalize (architecture 13 adopted approach); choose
  the first target language and record the decision in
  `extract_backend.md`.

## Ordered Task List

Keep `cargo test -p mizar-doc` green after each task (see
[Recommended Verification](#recommended-verification)).

### Inputs

1. **Crate scaffold and lint-policy guard.** [ ]
   - Add the `mizar-doc` workspace member depending on `mizar-session` and
     `mizar-artifact`; add `tests/lint_policy.rs` mirroring the
     `mizar-frontend` guard.
   - Tests: lint-policy guard passes; workspace builds.
   - Deps: `mizar-artifact` task 11. Spec: architecture 13.

2. **Spec: `artifact_reader.md`.** [ ]
   - Write the artifact-reader spec (English and Japanese, no code):
     validating reads of `VerifiedArtifact`s and manifests, schema
     compatibility handling, and the documentation-input projection
     (internal 05 "Artifact Documentation Reader").
   - Deps: 1. Spec: [internal 05](../../internal/en/05.documentation_extraction.md).

3. **Artifact reader.** [ ]
   - Implement validating artifact/manifest reads with schema-version
     checks; fixture artifacts for tests.
   - Tests: valid/invalid artifact fixtures; incompatible versions fail
     cleanly with diagnostics.
   - Deps: 2. Spec: `artifact_reader.md`.

4. **Spec: `doc_build.md`.** [ ]
   - Write the documentation build-plan spec (English and Japanese, no
     code): phase-16 requests, build planning over packages, the
     documentation index, determinism rules, and the consumer-phase rule.
   - Deps: 2. Spec: architecture 13 "Documentation Build Plan",
     [internal 05](../../internal/en/05.documentation_extraction.md).

5. **Documentation build planning and index.** [ ]
   - Implement build planning and the documentation index over reader
     output.
   - Tests: plan fixtures over multi-package inputs; deterministic index
     ordering.
   - Deps: 3, 4. Spec: `doc_build.md`.

### Documentation strand

6. **Spec: `comments.md`.** [ ]
   - Write the doc-comment spec (English and Japanese, no code):
     attachment targets, Markdown subset, diagnostics policy (degrade
     gracefully), and the doc-comment-source decision.
   - Deps: 4. Spec: architecture 13 "Step 2",
     [24.documentation_generation.md](../../../spec/en/24.documentation_generation.md).

7. **Doc-comment attachment.** [ ]
   - Attach doc comments to documented items per the decided source.
   - Tests: attachment fixtures per item kind; unattached comments
     diagnosed, not dropped silently.
   - Deps: 5, 6. Spec: `comments.md`.

8. **Markdown parsing.** [ ]
   - Parse the documented Markdown subset with graceful degradation and
     presentation diagnostics.
   - Tests: subset fixtures; malformed Markdown degrades without aborting
     the build.
   - Deps: 7. Spec: `comments.md`.

9. **Spec: `links.md`.** [ ]
   - Write the cross-reference spec (English and Japanese, no code): link
     syntax, resolution against exported symbol identity (shared with
     extraction), and unresolved-link policy.
   - Deps: 4. Spec: architecture 13 "Step 3"/"Documentation and Extraction
     Share Symbol Identity".

10. **Cross-reference resolution.** [ ]
    - Resolve doc links against artifact symbol identity with deterministic
      unresolved-link diagnostics.
    - Tests: intra- and inter-package link fixtures; unresolved links
      diagnosed with positions.
    - Deps: 8, 9. Spec: `links.md`.

11. **Spec: `math.md`.** [ ]
    - Write the math-rendering spec (English and Japanese, no code): LaTeX
      subset, formula rendering strategy, and determinism rules.
    - Deps: 4. Spec: architecture 13 "Step 3".

12. **Math rendering.** [ ]
    - Implement deterministic LaTeX/formula rendering with graceful
      degradation.
    - Tests: formula fixtures; identical output across runs; malformed
      math degrades with diagnostics.
    - Deps: 8, 11. Spec: `math.md`.

13. **Spec: `render.md`.** [ ]
    - Write the rendering spec (English and Japanese, no code): static
      HTML site layout, search index format, and deterministic output
      rules.
    - Deps: 4. Spec: architecture 13 "Step 4",
      [24.documentation_generation.md](../../../spec/en/24.documentation_generation.md).

14. **HTML rendering.** [ ]
    - Emit the static site from the documentation index deterministically.
    - Tests: golden-file fixtures; byte-identical output across runs.
    - Deps: 10, 12, 13. Spec: `render.md`.

15. **Search index emission.** [ ]
    - Emit the search index alongside the site with canonical ordering.
    - Tests: index fixtures; deterministic ordering; entries trace to
      symbol identity.
    - Deps: 14. Spec: `render.md`.

### Extraction strand

16. **Spec: `extract_select.md`.** [ ]
    - Write the selection spec (English and Japanese, no code): the
      verified executable subset, selection rules, and rejection of
      unsupported runtime constructs (no graceful degradation here).
    - Deps: 2. Spec: architecture 13 "Step 1 (extraction)"/"Extraction
      Uses Verified Executable Subset".

17. **Extractable item selection.** [ ]
    - Implement selection over artifact metadata; unsupported constructs
      rejected with stable diagnostics.
    - Tests: selection fixtures; rejection cases per unsupported construct.
    - Deps: 3, 16. Spec: `extract_select.md`.

18. **Spec: `runtime_ir.md`.** [ ]
    - Write the `RuntimeIr` spec (English and Japanese, no code):
      target-neutral runtime representation, ghost/proof-only erasure
      rules, and traceability to source items and artifact hashes.
    - Deps: 16. Spec: architecture 13 "Step 2 (extraction)",
      [internal 05](../../internal/en/05.documentation_extraction.md).

19. **`RuntimeIr` construction.** [ ]
    - Build `RuntimeIr` from selected items.
    - Tests: construction fixtures; every node traceable to artifact
      hashes.
    - Deps: 17, 18. Spec: `runtime_ir.md`.

20. **Ghost and proof-only erasure.** [ ]
    - Erase ghost state and proof-only annotations; extracted code must
      contain neither.
    - Tests: erasure fixtures; ghost leakage fails the build.
    - Deps: 19. Spec: `runtime_ir.md` (erasure section).

21. **Spec: `extract_backend.md`.** [ ]
    - Write the target-backend spec (English and Japanese, no code):
      backend interface, determinism per `RuntimeIr` and target options,
      and the first-target decision.
    - Deps: 18. Spec: architecture 13 "Step 3 (extraction)".

22. **First target backend.** [ ]
    - Implement code emission for the chosen first target language.
    - Tests: golden-file fixtures; deterministic output; emitted code
      traces to source items.
    - Deps: 20, 21. Spec: `extract_backend.md`.

### Publication and follow-ups

23. **Spec: `publisher.md`, output publishing, and the phase-16 manifest.** [ ]
    - Write the publisher spec and implement atomic publication of docs,
      search indexes, extracted code, and the extraction manifest; outputs
      are deletable and regenerable without affecting proof validity.
    - Tests: interrupted publication leaves no mixed state; regeneration is
      byte-identical.
    - Deps: 15, 22. Spec: [internal 05](../../internal/en/05.documentation_extraction.md)
      "Output Publisher", architecture 13 "Step 4 (extraction)".

24. **Determinism suite.** [ ]
    - Property coverage that identical artifacts produce byte-identical
      sites, indexes, and extracted code.
    - Deps: 23. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md).

25. **Public-enum forward-compatibility policy.** [ ]
    - Apply the `mizar-frontend` task-25 procedure to each public enum.
    - Deps: 23. Spec: all module specs.

26. **Source/spec correspondence audit.** [ ]
    - Trace every public API and promised behavior in the module specs to
      implementation and tests; record gaps as follow-up tasks.
    - Deps: 25. Spec: all module specs and this TODO.

27. **Bilingual documentation sync audit.** [ ]
    - Compare each English canonical document under
      `doc/design/mizar-doc/en/` with its Japanese companion and
      synchronize content.
    - Deps: 26. Spec: repository documentation policy.

28. **Module-boundary refactor gate.** [ ]
    - Before treating the crate as ready for downstream consumers, audit the
      source layout for oversized files, mixed responsibilities, and private
      helpers that should be split along the module table and spec boundaries.
      Split any review-bottleneck implementation files into private modules
      without changing public APIs, diagnostics, deterministic renderings,
      artifact-facing schemas, or consumer-visible behavior.
    - After any split, update this module table/source paths as needed and
      re-run the source/spec and bilingual documentation audit scopes for the
      moved APIs. Do not mix behavior cleanup or API exposure into the move;
      those require their own spec tasks.
    - Deps: 27. Spec: this TODO,
      [internal 07](../../internal/en/07.crate_module_layout.md), all module
      specs.

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-doc
cargo clippy -p mizar-doc --all-targets -- -D warnings
```

For tasks that touch artifact schemas, also run:

```text
cargo test -p mizar-artifact
```

Check the task off here once tests pass.

## Notes

- Phase 16 is a consumer phase: it never re-runs semantic analysis and is
  never a publication gate for proof validity.
- Documentation degrades gracefully on presentation errors; extraction
  rejects unsupported constructs instead.
- Ghost state and proof-only annotations must never appear in extracted
  runtime code; extracted code must be traceable to source items and
  artifact hashes.
- Outputs may be deleted and regenerated at any time; nothing downstream
  may treat them as authoritative.
