# Task DOC-COMPACT-MANIFEST: Data-Driven Legacy-Compaction Ledger

> Canonical language: English. Japanese companion:
> [../ja/DOC-COMPACT-MANIFEST.md](../ja/DOC-COMPACT-MANIFEST.md).

This is a derived documentation/test-policy prerequisite. It cannot introduce
or override language behavior, test intent, diagnostics, public API, or
coverage credit. It replaces a Rust-coded historical ledger with one bounded,
reviewable data source before further legacy-evidence batches are migrated.

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-COMPACT-MANIFEST` |
| Status | Implementation, independent reviews, and required verification complete; exact task-only staging and commit remain. |
| Purpose | Move exact legacy-compaction inventory out of lint implementation code, preserve the existing Task-269SDP/269SDC guarantees byte-for-byte, and let later bounded batches extend data without adding a Rust test or ledger branch per task. |
| Primary owners | Repository documentation policy and `mizar-test` lint policy |
| Consumers | `task_contracts_are_recursively_paired_and_supported_links_resolve`, paired task contracts, owning crate-plan Task Index tables, and every later exact-section compaction batch |
| Dependencies | `DOC-269SD-COMPACT` commit `5080d3fddaad6e9683e5eecc5e497b4b16908e8a` and its exact 82 redirects/12 index rows |
| Readiness | The documentation prerequisite is commit `a16fee8fa7d059dd2d1930cca1ce067434b3ebe2`; its fresh inventory was clean, `origin/main...HEAD` was `0/1`, and protected `stash@{0}` remained `f65cf4a13752ec380710814a9ac6392ccb9d75d4`. |

The [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index) and
[runner plan](../../mizar-test/en/00.crate_plan.md#task-index) index this
contract.

## Authority And Classification

Authority is the user's explicit decision to prioritize repository-wide
documentation consolidation, [`AGENTS.md`](../../../../AGENTS.md), and the
[legacy migration policy](../../autonomous_crate_development.md#migration-policy).
No language-specification authority is consumed or changed.

| Class | Decision |
|---|---|
| `design_drift` | The existing lint embeds Task-269SDP/269SDC IDs, file groups, forbidden headings, redirects, and Task Index rows in Rust, making policy code a second historical ledger. |
| `test_gap` | The current exact checks pass, but no versioned data boundary lets a later batch extend the ledger while keeping the test name/count and policy implementation stable. |
| `spec_gap` | None for this derived policy task. Historical semantic gaps remain out of scope and unchanged. |
| `source_drift` | None in production source. The only later Rust delta is a test-policy refactor with identical accepted repository state. |
| `source_undocumented_behavior` | None introduced or inferred. |
| `test_expectation_drift` | None; semantic expectations are protected. |
| `boundary_violation` | Avoided by keeping the manifest under the task-contract owner and consuming it only from `mizar-test` lint policy. |
| `repo_metadata_conflict` | The prior post-commit remote-tracking ref moved through an external `update by push`; the agent issued no push. Current HEAD and `origin/main` agree, so this remains report-only and does not obscure the safe commit target. |

## Documentation-Prerequisite Scope

This prerequisite changes exactly six Markdown files: this EN/JA pair and one
compact Task Index row in each checker/test EN/JA crate plan. It changes no
policy text, Rust, manifest data, production source, specification, test,
fixture, sidecar, expectation, trace, Cargo file, count, status, or hash. After
its dedicated commit, fresh inventory returns to the implementation frozen
below.

## Frozen Manifest Contract

Implementation adds exactly one language-neutral, versioned UTF-8 TSV file:
`doc/design/task_contracts/legacy_compactions.tsv`. TSV is selected instead of
TOML so the lint needs no Cargo dependency or hand-written partial TOML parser.
Blank lines and lines beginning with `#` are ignored. Exactly one
`schema<TAB>1` row is required as the first non-comment, nonblank row. Every
later data row uses one literal tab between fields; fields may not contain a
tab, carriage return, or line feed. After comments and blank lines are removed,
all data rows following `schema` must be in ascending unsigned UTF-8 byte order
over the complete physical row, including its record kind and every separating
tab. Unknown record kinds, versions, columns, duplicate identities, unsorted
data rows, absolute paths, `..` traversal, and paths outside the workspace fail
closed.

The schema contains these exact records:

| Kind | Fields after kind |
|---|---|
| `schema` | version (`1`); exactly one record in the first data position |
| `batch` | batch id; EN batch-contract path; JA batch-contract path; canonical expanded-inventory SHA-256; task count; redirect count; distinct source-file count; index-row count |
| `task` | batch id; task id; EN historical-contract path; JA historical-contract path |
| `redirect` | batch id; task id; language (`en`/`ja`); source path; legacy heading level (`2` through `6`); exact forbidden legacy heading; exact replacement line; exact preceding same-or-higher-level heading or `BOF`; exact following same-or-higher-level heading or `EOF` |
| `index` | batch id; indexed task/batch id; language (`en`/`ja`); owning `00.crate_plan.md` path; exact Task Index row |

All paths are slash-separated workspace-relative paths. Task and batch IDs use
`[A-Za-z0-9][A-Za-z0-9._-]*`. A `redirect` represents replacement of one exact
whole ATX heading section through the next heading of equal or higher level;
mixed owner-local sections and paragraph-only migrations are intentionally not
representable in schema version 1. They require a separately reviewed schema
extension rather than an inferred deletion.

The legacy and neighboring heading fields are exact raw ATX heading lines,
including their `#` level and text. A heading is recognized only outside fenced
code. Relative to the declared legacy level, the anchor check walks outward
from the unique redirect and selects the nearest preceding and following raw
ATX headings of equal or higher level; intervening lower-level headings do not
terminate the legacy section. Repeated identical anchor text is allowed and is
resolved only by this nearest-heading rule. `BOF` or `EOF` is valid only when no
qualifying heading exists on that side.

Batch IDs are unique. Task IDs are globally unique across manifest batches,
and each `task` belongs to exactly one declared batch. Declared EN/JA batch and
task contracts must exist as paired files, have the declared ID as their file
stem/title ID, and occupy their corresponding `task_contracts/en` or `ja`
tree. Every `redirect` task must belong to its batch. Its replacement must be
the standard central completion-evidence sentence and resolve exactly to that
task record's declared language-local contract and `#completion-evidence`, not
merely to any existing document. Every `index` ID must be either a task in its
batch or that batch ID; its exact EN/JA row must resolve to the corresponding
declared language-local task or batch contract. Plan-path language and the
record's language must agree, and the plan filename must be exactly
`00.crate_plan.md`.

For each batch, canonical expanded inventory excludes `schema`, `batch`,
comments, and blank lines. It consists of the complete physical UTF-8 bytes,
including record kind and literal separating tabs, of every `task`, `redirect`,
and `index` row whose batch-id field equals that batch. Those rows are sorted
in ascending unsigned byte order over the whole row and concatenated with one
LF after every row, including the last. The declared SHA-256 and all declared
counts must match that exact byte string. This authenticates the review surface
without making Rust source the ledger or creating a self-referential hash.
SHA-256 is implemented inside lint-policy test support with no Cargo dependency
and no external executable. It must pass the standard empty-input known-answer
vector
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
before validating manifest inventory hashes.

## Frozen Lint Consumer

The existing single test
`task_contracts_are_recursively_paired_and_supported_links_resolve` remains
the sole consumer and retains its name. Its implementation must:

1. parse the strict schema, validate physical ordering, identities, relational
   membership/linkage, batch totals, and exact inventory hashes;
2. cache each referenced Markdown document and heading-slug set;
3. reject duplicate expanded keys, every exact forbidden heading anywhere in
   `doc/design`, missing/multiple expected redirects, and unexpected central
   historical-contract redirects;
4. validate each redirect's exact source path, nearest preceding/following
   same-or-higher-level heading anchors, language-local file target, and
   `#completion-evidence` fragment;
5. validate every exact `index` row once inside the specified plan's
   `## Task Index` section; and
6. retain recursive EN/JA contract pairing, title/marker, reciprocal contract
   link, owning-plan backlink, supported inline-link, fence, and ATX-fragment
   checks already enforced.

The manifest owns only exact full lines in this reserved grammar:
`Completion evidence: [central Task-<id> historical contract](<relative-md-path>#completion-evidence).`
or the same line ending in Japanese `。`. `<id>` uses the task-ID grammar and
the path must be a supported relative `.md` target. “Unexpected” means a line
matching this complete grammar that has no manifest `redirect` record; ordinary
dependency/reference links and partial prose are never rejected by this rule.

Implementation first encodes the completed `DOC-269SD-COMPACT` batch exactly:
two historical tasks, 82 redirects across 42 distinct source files, and 12
Task Index rows. The current accepted repository must remain accepted, and
mutations of schema/count/hash/path/heading/redirect/anchor/index/link/fragment
must fail in focused parser/checker vectors inside the same test. The test
target remains 15 tests with raw list hash
`b044e771a655e72131d0371636bbac5684ef93a3ea503984537a4bb9dd13a7cf`.

The same-test section-boundary vectors cover every H2 through H6 legacy level,
`BOF`/`EOF`, a lower-level heading that does not terminate a section, equal-
and higher-level terminators, repeated identical raw anchor text, and heading-
shaped lines inside both backtick and tilde fences. Digest vectors include the
empty-input known answer before any manifest mutation case.

The implementation scope is exactly the TSV manifest, the existing lint-policy
test file, `AGENTS.md`, `doc/design/README.md`, the autonomous protocol, and
this EN/JA pair for completion status/evidence. It adds no new Rust test and
changes no Cargo file.

## Protected Boundaries And Deferrals

Implementation must not change `doc/spec/**`, `.miz`, fixture, sidecar,
expectation, `tests/coverage/spec_trace.toml`, trace row/status/backlink,
production Rust, public API, diagnostic, parser/resolver output, active route,
CLI result, or executable coverage credit. It must not compact another legacy
section, add a historical task contract, or select future language semantics.
The Chapter-4/15 `set` disagreement and all proof/goal/guard/fact/obligation/
capture questions remain outside this task.

`doc/design/spec_coverage_audit.md` has no coverage-status or ownership impact
and remains unchanged. The next exact-section batch is selected only after the
manifest implementation commit from fresh inventory; current review evidence
favors the exact shared Task-269GUP/269GCT/269GCU family, but this task does not
pre-authorize that migration. The consolidation program prioritizes
`mizar-checker`-owned task families until their structure is stable;
`mizar-test` documents participate only where they own a runner, harness,
traceability, audit, plan-index, or redirect consumer for the selected checker
family.

## Documentation-Prerequisite Evidence

Independent specification/policy/EN-JA and test-contract reviews ended **NO
FINDINGS** after the schema's relational linkage, canonical bytes, H2–H6
boundaries, digest implementation, and reserved grammar were made exact. The
diff contains only the frozen six Markdown paths. The full 15-test
`mizar-test` lint-policy target, `cargo fmt --all --check`, Cargo metadata,
local link/fragment checks, protected-scope inspection, and `git diff --check`
pass. No specification, semantic test, production source, coverage audit,
trace, Cargo file, or executable status changed.

## Implementation Evidence

- The seven-path implementation surface is exactly this EN/JA pair, the TSV
  ledger, the existing `mizar-test` lint-policy file, `AGENTS.md`, the design
  index, and the autonomous protocol. No production, specification, fixture,
  sidecar, expectation, trace, Cargo, public-API, or coverage-audit path
  changed.
- The 99-line TSV has physical SHA-256
  `c537eda8401c1cdc0a3386ca648d112075b0728b702b56d03f89e353d4a4347f`.
  It declares one batch, two tasks, 82 redirects over 42 distinct source
  files, and 12 index rows. Independent replay reproduces expanded-inventory
  SHA-256
  `66087afe7a11a73aafeda4853dba2b684ef9edccae5fc014cc8fa01bb8265f8b`.
- The Rust consumer contains no Task-269SDP/269SDC or batch-specific ledger.
  Strict mutation vectors, document-evidence boundary vectors, fence-aware
  headings, canonical containment, non-symlink traversal, and cached document/
  fragment validation all use the existing single test. The lint target stays
  at 15 tests with raw list hash
  `b044e771a655e72131d0371636bbac5684ef93a3ea503984537a4bb9dd13a7cf`.
- Independent test-sufficiency, implementation/security, and source/document/
  EN-JA reviews ended **NO FINDINGS** after all findings were corrected.
- Focused and full lint policy, checker 530-test and runner 600-test libraries,
  137 metadata tests, checker lint, formatting, warnings-denied workspace
  Clippy, and full `cargo test` pass. The five CLI hashes remain respectively
  `700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
  `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
  `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
  `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
  and `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.
  The protected trace hash remains
  `55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.

## Reviews, Verification, And Exit

The documentation prerequisite requires independent specification/policy and
EN/JA review to **NO FINDINGS**, all nine documentation gates PASS without a
score cap at `>=90/100`, exact six-file staging, one docs-only commit, and clean
post-commit HEAD/origin/stash inventory.

Implementation then requires independent test-sufficiency,
implementation/security, and source/document/EN-JA consistency reviews to
**NO FINDINGS**. Verification includes focused malformed-manifest vectors, the
full lint-policy target, checker/runner libraries and metadata tests,
`cargo fmt --all --check`, workspace Clippy with warnings denied, full
`cargo test`, all five CLIs, protected count/hash replay, exact current
82/42/12 acceptance, `git diff --check`, and cached/unstaged audits. Final
read-only quality requires all nine gates PASS, no score cap, and at least
`90/100`; implementation exits with one task-only commit, clean inventory,
unchanged stash, and no agent-issued push.
