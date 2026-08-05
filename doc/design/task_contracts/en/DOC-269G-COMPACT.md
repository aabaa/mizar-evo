# Task DOC-269G-COMPACT: Given-Family Completion-Evidence Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-269G-COMPACT.md](../ja/DOC-269G-COMPACT.md).

This is a derived documentation-maintenance contract. It preserves completed
historical evidence and cannot introduce or reinterpret language behavior,
test intent, diagnostics, public API, or coverage credit.

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-269G-COMPACT` |
| Status | Implementation, independent reviews, and final read-only quality review complete at `100/100` with all nine hard gates PASS and no score cap; ready for exact task-only staging and commit. |
| Purpose | Centralize the exact shared Task-269GUP/GCT/GCU completion sections while preserving every nonidentical plan, audit, trace-status, verification, boundary, and sequencing owner. |
| Owners | Repository documentation policy and the data-driven `mizar-test` legacy-compaction lint |
| Consumers | [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index), [runner plan](../../mizar-test/en/00.crate_plan.md#task-index), 40 declared source documents, and the versioned compaction manifest |
| Dependencies | Task 269GUP `076c1425`; Task 269GCT `d6fb0ed2`; Task 269GCU `f984ae68`; `DOC-COMPACT-MANIFEST` `0ec5fce293a6105e04761c5298b605d3f4ff60ca`; generic multi-batch mutation prerequisite `deb2e823ef6bc5d68a53aa871a4a9dd7ed333253` |
| Readiness | Implementation began from clean HEAD `deb2e823`, `origin/main...HEAD=0/4`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`; no blocking authority gap. |

## Authority And Classification

Authority is the user's checker-first documentation-consolidation decision,
[`AGENTS.md`](../../../../AGENTS.md), the
[autonomous protocol](../../autonomous_crate_development.md#migration-policy),
the three historical task records, and the exact current shared sections.
Language specification and semantic test authority are unchanged.

| Class | Decision |
|---|---|
| `design_drift` | 116 exact shared completion sections repeat 2,567 lines across 40 files without central historical owners. |
| `test_gap` | None. The generic schema-v1 manifest consumer can encode this batch without a Rust-code or test-count change. |
| `spec_gap` | None for the structural migration; no semantic issue is selected. |
| `source_drift` | None; production source is protected. |
| `source_undocumented_behavior` | None introduced or inferred. |
| `test_expectation_drift` | None; `.miz`, sidecars, expectations, and trace data are protected. |
| `boundary_violation` | Avoided by excluding all nonidentical and owner-local sections and by disambiguating, not deleting, the two mixed checker-plan GCT headings. |
| `repo_metadata_conflict` | None obscures the target. At documentation-prerequisite inventory `origin/main` was two commits behind; during implementation the external ref advanced to `deb2e823` (`0/0`) without an agent push. This is report-only metadata movement; no repair or push is authorized. |

## Documentation-Prerequisite Scope

The prerequisite changes exactly 12 Markdown paths: the new EN/JA historical
contracts for 269GUP, 269GCT, and 269GCU; this EN/JA batch pair; and the four
checker/test EN/JA crate plans. Each plan receives exactly four Task Index
rows, one per new contract, for 16 rows total. In addition, only the two
nonidentical checker-plan GCT headings are renamed:

- EN: `### Task 269GCT implementation status` becomes
  `### Task 269GCT plan-local implementation and GCU sequencing status`;
- JA: `### Task 269GCT implementation status` becomes
  `### Task 269GCT plan-local implementation/GCU sequencing status`.

The bodies below those headings remain byte-identical. The prerequisite does
not edit the manifest, lint Rust, any other design file, specification, test,
fixture, sidecar, expectation, trace, Cargo file, production source, protected
artifact count/hash/status, or executable behavior.

## Exact Shared Inventory

Each section begins at the exact listed H3 heading and ends immediately before
the next visible ATX heading of H3 or higher. Hashes cover the physical UTF-8
section bytes including the heading, internal/final LF bytes, and blank lines,
but exclude the following heading. Within each row every section is byte
identical. These are the only sections authorized for replacement:

| Task | Owner/language | Exact heading | SHA-256 | Sections | Lines |
|---|---|---|---|---:|---:|
| 269GUP | checker/en | `### Task 269GUP implemented binding profile` | `a2253a41346c83b0e4ea477d8ab864ca9171b015e2e0aba15e91af98edbd4af3` | 13 | 78 |
| 269GUP | checker/ja | `### Task 269GUP binding profile 実装状況` | `21f913d74088901df228d34b8e97d626f35e32522757a2a860bb8c1b40ee9ca9` | 13 | 78 |
| 269GUP | test/en | `### Task 269GUP implemented dormant runner` | `8e84a6249a185787602017e7645fb9ac7f62144827c35cdb3aebac428ece6222` | 6 | 36 |
| 269GUP | test/ja | `### Task 269GUP dormant runner 実装状況` | `6701ebec916449691c5acdbd09953c49323f1f508a281c1dfc8bdee95bff3c0e` | 6 | 36 |
| 269GCT | checker/en | `### Task 269GCT implementation status` | `b21d19691d7ee99d1bb27425fc1fdecd9986dcaa0090a984bf4ee218cc84b65f` | 13 | 416 |
| 269GCT | checker/ja | `### Task 269GCT implementation status` | `da82c262c070b9ac85f6e94d5d56df78d6eb02ce7154c0222515cd390ef293ed` | 13 | 403 |
| 269GCT | test/en | `### Task 269GCT implemented private runner status` | `521d1fd500d969bc8a7c7728372072a522bca17c7a60d15ac5a4d65e2c75443e` | 6 | 108 |
| 269GCT | test/ja | `### Task 269GCT implemented private runner status` | `b01173bc886ea3fd6476e3f7130486e4064aa9fac00e010482d70b6891dcf947` | 6 | 108 |
| 269GCU | checker/en | `### Task 269GCU implementation status` | `3569b601b33c119f5147b6953c4dc14bcd56cae084ffc08c003fe36faa91827a` | 14 | 532 |
| 269GCU | checker/ja | `### Task 269GCU implementation status` | `1b0492c909233d924eca8f725bccefa838256ecd3eca63677b190d0a7c9990f1` | 14 | 532 |
| 269GCU | test/en | `### Task 269GCU implemented private runner status` | `3319584f02a98e2dee03f48640fa32a3dd2edf7237bffc57531f27be9eb0ada5` | 6 | 120 |
| 269GCU | test/ja | `### Task 269GCU implemented private runner status` | `a2ed60dfa2cb8c4df00523be089dc9cc142c7186371c00ee9c97a4cbb26baeb9` | 6 | 120 |
| **Total** |  |  |  | **116** | **2,567** |

The exact path/task matrix is symmetric across EN and JA:

| Component | Relative file | Authorized tasks per language |
|---|---|---|
| mizar-checker | [`00.crate_plan.md`](../../mizar-checker/en/00.crate_plan.md#checker-task-269gcu-frozen-given-condition-termreference-plan) | 269GCU only |
| mizar-checker | [`binding_env.md`](../../mizar-checker/en/binding_env.md#task-269gup-new-source-binding-profile) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`bilingual_sync_audit.md`](../../mizar-checker/en/bilingual_sync_audit.md#task-269gup-documentation-synchronization) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`module_boundary_audit.md`](../../mizar-checker/en/module_boundary_audit.md#task-269gup-frozen-boundary) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`payload_family_decomposition.md`](../../mizar-checker/en/payload_family_decomposition.md#task-269gup-payload-delta) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`resolved_typed_ast.md`](../../mizar-checker/en/resolved_typed_ast.md#task-269gup-final-owner-exclusion) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`semantic_spec_audit.md`](../../mizar-checker/en/semantic_spec_audit.md#task-269gup-zero-semantic-boundary) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`source_proof_local_declaration.md`](../../mizar-checker/en/source_proof_local_declaration.md#checker-task-269gup-frozen-use-profile-binding-prerequisite) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`source_spec_audit.md`](../../mizar-checker/en/source_spec_audit.md#task-269gup-frozen-sourceapi-delta) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`source_statement.md`](../../mizar-checker/en/source_statement.md#task-269gup-statement-boundary) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`source_term.md`](../../mizar-checker/en/source_term.md#task-269gup-source-term-exclusion) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`source_type.md`](../../mizar-checker/en/source_type.md#task-269gup-source-type-exclusion) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`todo.md`](../../mizar-checker/en/todo.md#checker-task-269gup-proof-given-use-profile-binding-prerequisite) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`typed_ast.md`](../../mizar-checker/en/typed_ast.md#task-269gup-typed-owner-exclusion) | 269GUP, 269GCT, 269GCU |
| mizar-test | [`00.crate_plan.md`](../../mizar-test/en/00.crate_plan.md#task-269gup-frozen-dormant-binding-profile-prerequisite) | 269GUP, 269GCT, 269GCU |
| mizar-test | [`bilingual_sync_audit.md`](../../mizar-test/en/bilingual_sync_audit.md#checker-task-269gup-documentation-synchronization) | 269GUP, 269GCT, 269GCU |
| mizar-test | [`harness.md`](../../mizar-test/en/harness.md#checker-task-269gup-frozen-dormant-harness) | 269GUP, 269GCT, 269GCU |
| mizar-test | [`module_boundary_audit.md`](../../mizar-test/en/module_boundary_audit.md#checker-task-269gup-frozen-runner-boundary) | 269GUP, 269GCT, 269GCU |
| mizar-test | [`todo.md`](../../mizar-test/en/todo.md#checker-task-269gup-dormant-use-profile-binding-prerequisite) | 269GUP, 269GCT, 269GCU |
| mizar-test | [`traceability.md`](../../mizar-test/en/traceability.md#checker-task-269gup-zero-credit-trace-boundary) | 269GUP, 269GCT, 269GCU |

This expands to 14 checker plus six test files in each language, exactly 40
distinct paths. The manifest must enumerate all 116 path/task rows and their
actual nearest same-or-higher-level anchors; no wildcard or inferred source is
allowed.

## Redirect And Manifest Contract

Each authorized section is replaced by exactly one language-local reserved
line targeting the matching historical contract's `#completion-evidence`.
English uses a final period and Japanese uses `。`. The implementation adds one
`batch` record (`DOC-269G-COMPACT`), three `task` records, 116 `redirect`
records, and all 16 exact Task Index `index` records to
`legacy_compactions.tsv`. Counts and the expanded-inventory SHA-256 are
recomputed from complete sorted rows. No task-specific Rust branch, test, or
test-name/count change is allowed.

## Explicit Exclusions And Deferrals

All nonidentical Task-269GUP checker-plan completion sections, the two retained
GCT plan-local bodies, GCT/GCU documentation-prerequisite verification
sections, frozen zero-credit trace-status H2 sections, and every H2 owner
section surrounding the shared H3 evidence are excluded. The root
`spec_coverage_audit.md` Task-269GUP implementation audit remains untouched.

The migration does not decide or change given-scope semantics. In particular,
it preserves the existing historical statement that a `given` binding is
valid through the remainder of its innermost block and descendant blocks with
inner shadowing, while descendant use/capture remains separate. It invents no
goal, guard, fact, equality, proof, discharge, acceptance, obligation, export,
capture, IR, VC, or Task-270 behavior.

`doc/design/spec_coverage_audit.md` has no changed coverage or ownership status
and remains unchanged. Production, public API, tests, trace, corpus, and all
five CLI outputs/hashes remain protected.

## Documentation-Prerequisite Evidence

- Independent specification/policy, exact-inventory/boundary, and EN/JA/
  owner-link reviews ended **NO FINDINGS** after all stable owner links were
  added. Recursive local link/fragment validation and `git diff --check` pass.
- Fresh replay reproduces all 12 frozen group hashes and exact totals of 116
  sections, 2,567 physical lines, and 40 paths. The four plans contain exactly
  16 selected index rows. Only the two declared checker-plan headings change;
  their retained bodies are byte-identical.
- The existing manifest remains physically
  `c537eda8401c1cdc0a3386ca648d112075b0728b702b56d03f89e353d4a4347f`
  with one batch, two tasks, 82 redirects, 42 source paths, and 12 index rows.
  No specification, test, trace, source, Cargo, or coverage-audit path changes.
- Full lint policy (15 tests), checker library (530), runner library (600),
  metadata (137), checker lint (15), `cargo fmt --all --check`, Cargo metadata,
  warnings-denied workspace Clippy, and full `cargo test` pass. Plan, parse,
  declaration, type, and proof CLI hashes remain respectively
  `700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
  `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
  `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
  `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
  and `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.
  The protected trace hash remains
  `55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.

## Implementation Evidence

- Fresh implementation preflight exposed a generic mutation-oracle
  `test_gap`: the first byte-sorted H3 redirect made a hard-coded H3 malformed
  heading level-consistent. The isolated one-line prerequisite changed that
  mutation to an always-invalid H1; independent reviews ended **NO FINDINGS**,
  all gates passed at `100/100`, and commit
  `deb2e823ef6bc5d68a53aa871a4a9dd7ed333253` completed it before this batch
  was reapplied from clean inventory.
- Exactly the 116 frozen section byte ranges across the declared 40 paths are
  replaced by language-local redirects. All eight forbidden legacy headings
  are absent from checker/test documents; exactly 116 matching redirects are
  present. Nonidentical plan, prerequisite-verification, zero-credit trace,
  owner-local, sequencing, and root coverage-audit exclusions remain intact.
- The 235-line manifest has physical SHA-256
  `d794d78662b570260f777e1b074ff20d7f5fa3ed911bb3c3e8730471ff96a46a`.
  Globally it declares two batches, five tasks, 198 redirects, and 28 index
  rows. This batch independently replays inventory SHA-256
  `deba263f24954ac6f7e081a3919933277fbb7152e5f256c38b9b992231716b53`
  with three tasks, 116 redirects, 40 source paths, and 16 index rows.
- Independent equivalence, test-sufficiency, and source/document/EN-JA reviews
  ended **NO FINDINGS** after the report-only external-origin wording was made
  exact. The focused and full lint policies pass with 15 tests each; checker
  and runner libraries pass with 530 and 600 tests, and metadata passes with
  137 tests. Cargo formatting, metadata, warnings-denied workspace Clippy, and
  full `cargo test` pass. All five CLI hashes and the protected trace hash are
  unchanged from the frozen prerequisite values above. The manifest counts,
  physical and inventory hashes, exact 43-path scope, forbidden-heading zero,
  116 redirects, protected-path exclusion, and `git diff --check` all pass.
  Only the cached/unstaged staging audit remains before commit.

## Reviews, Verification, And Exit

The documentation prerequisite requires independent specification/policy,
exact-inventory/boundary, and EN/JA review to **NO FINDINGS**, all nine hard
gates PASS without a score cap at `>=90/100`, exact 12-path staging, one
docs-only commit, and clean post-commit inventory.

After fresh inventory, implementation changes only the declared 40 source
paths, this EN/JA status/evidence pair, and the TSV manifest. Independent test
sufficiency, equivalence, and source/document/EN-JA reviews must end **NO
FINDINGS**. Verification includes focused/full lint policy, checker/runner
libraries, metadata and checker lint, formatting, warnings-denied workspace
Clippy, full `cargo test`, all five CLIs, exact 116/2,567/path/hash replay,
manifest counts/hashes, protected trace/corpus/source hashes, local links,
`git diff --check`, cached/unstaged audit, and final 9/9 hard gates at
`>=90/100`. One task-only commit completes the batch; the agent does not push.

## Final Quality And Handoff

The final read-only quality review found only the missing terminal handoff. All
nine hard gates passed and no score cap applied. After paired EN/JA correction
and finding-specific re-review, the final score is `100/100`. Focused lint and
diff checks must pass again before exactly the same 43 paths are staged.

After commit, begin with clean read-only repository and canonical-authority
inventory. Inventory `mizar-checker` first and treat `mizar-test` only as its
consumer where applicable. Select and freeze exactly one dependency-ready
duplication family under the migration policy; this contract does not
preauthorize any particular next batch or semantic task. Keep the parent at
`xhigh` because ownership and byte-preservation boundaries span many documents;
a bounded, mechanically frozen review packet may use a `high` review agent.
