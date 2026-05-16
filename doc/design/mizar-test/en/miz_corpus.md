# Module: miz_corpus

> Canonical language: English. Japanese companion: [../ja/miz_corpus.md](../ja/miz_corpus.md).

## Purpose

This module defines the strategy for building and maintaining a large `.miz` corpus.

The corpus is an implementation asset. It is used to pin syntax, type behavior, cluster behavior, overload behavior, proof rejection, dependency fingerprints, and deterministic artifact output.

## Corpus Classes

| Class | Purpose | Ownership |
|---|---|---|
| handwritten minimal cases | focused parser/type/cluster/proof behavior | reviewed by developers |
| migrated examples | realistic source patterns from existing Mizar-like material | reviewed before acceptance |
| generated `.miz` | broad coverage of grammar and semantic combinations | generated with stored seeds |
| fuzz-minimized reproducers | permanent regression cases from fuzz failures | committed after minimization |
| bug regressions | protect fixed bugs and soundness failures | linked to issue/PR when possible |
| stress/integration articles | large-file and incremental rebuild behavior | reviewed for stability |

## Growth Targets

| Stage | Target |
|---|---|
| early module development | 100-300 `.miz` files |
| evo2 alpha | 300-1,000 `.miz` files |
| evo2 beta / release candidate | 1,000-5,000 `.miz` files |
| mature ecosystem | 500,000-1,000,000 LOC equivalent including generated/fuzz/property corpora |

Growth should favor high-signal fail/soundness tests near the kernel over large numbers of shallow pass tests.

## File Size Guidelines

| Purpose | Lines |
|---|---:|
| parser test | 5-30 |
| type test | 10-50 |
| cluster test | 20-80 |
| theorem test | 30-150 |
| integration test | 100-300 |
| stress test | 500-1,000 |

Generated corpora may exceed these ranges only in stress directories.

## Generation Policy

Generated `.miz` files must record:

- generator name and version;
- seed;
- generation profile;
- expected outcome;
- minimization status;
- schema version for metadata.

Generated tests are committed only when they add coverage, reproduce a bug, or serve as stable stress cases. Bulk generated corpora may live outside the default fast test set until they are minimized or promoted.

## Review Rules

Corpus additions are reviewed for:

- stable expected outcome;
- deterministic diagnostics and snapshots;
- absence of hidden reliance on test execution order;
- minimality for fail/soundness regressions;
- clear domain placement and naming.

## Tests

Key scenarios:

- generated tests can be reproduced from stored seed metadata;
- minimized fuzz reproducers preserve the original failure category;
- corpus manifest counts pass/fail ratios by domain;
- stress tests are excluded from default fast runs unless requested.

## Constraints and Assumptions

- `.miz` corpus files are long-lived compatibility inputs.
- Fail tests are not loosened to match current compiler behavior.
- A soundness regression case is never deleted without architecture-level review.
