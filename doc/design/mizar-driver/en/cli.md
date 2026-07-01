# mizar-driver CLI Surface

> Canonical language: English. Japanese companion: [../ja/cli.md](../ja/cli.md).

Status: specified by task D-012. Source implementation completed by task D-013.

## Purpose

The `cli` module is the batch command-line entry point over `mizar-driver`.
It turns command-line options and package/workspace files into protocol-agnostic
driver requests, subscribes to the driver event stream for progress, renders
diagnostics through `mizar-diagnostics`, and maps terminal driver outcomes to
stable process exit codes.

The CLI is a user interface. It does not own manifest syntax, dependency
resolution, phase semantics, type checking, proof acceptance, cache
compatibility, artifact serialization, artifact publication tokens, or LSP
protocol conversion.

## Public Enum Compatibility

All public enums in this module are downstream-facing CLI boundary types and are
marked `#[non_exhaustive]`. D-017 records no exhaustive exceptions for:

- `CliCommand`;
- `CliBuildProfile`;
- `CliMessageFormat`;
- `CliExitCode`.

Downstream crates must use wildcard arms when matching these enums. Future
commands, profiles, message formats, or exit codes may be added without moving
manifest semantics, diagnostics identity, artifact output authority, or LSP
protocol conversion into the CLI.

## Command Surface

The binary name is `mizar`. Task D-013 implements only the batch build command:

```text
mizar build [OPTIONS]
```

Future commands such as `mizar explain`, `mizar refine`, `mizar minimize`, and
`mizar semver-check` are specified by spec 23, but they are not owned by D-013
unless a later task explicitly extends the CLI module.

`mizar build` accepts:

| Option | Meaning |
|---|---|
| `--workspace <path>` | Workspace root. Defaults to the current working directory. |
| `--manifest-path <path>` | Package manifest path for a single-package build. Mutually exclusive with `--workspace` when it would select a different root. |
| `--package <name>` | Limit the build to one package. May be repeated for workspace builds. |
| `--profile <check|release>` | Driver build profile. `release` requires owner-provided verifier policy and must not weaken trusted/kernel requirements locally. |
| `--target <package-or-module>` | Limit build targets. The CLI records the target identity and lets the driver/build owner resolve it. |
| `--jobs <n>` | Worker count hint passed to scheduler input. Values below 1 are usage errors. |
| `--locked` | Require the existing lock file; do not update dependency resolution outputs. |
| `--no-incremental` | Disable incremental/cache reuse for this invocation without changing semantic behavior. |
| `--message-format <human|json>` | Human text or JSON Lines command output. Defaults to `human`. |
| `--quiet` | Suppress progress events, but not diagnostics or the final outcome. |

The CLI may add narrow aliases only when they are documented here before source
implementation. Undocumented flags are usage errors.

## Request Mapping

`mizar build` constructs:

- a `BuildRequestDraft` with `BuildRequestOrigin::Batch`, lane `0` unless a
  later multi-invocation batch mode specifies otherwise, and generation `0` for
  one-shot builds;
- `DriverSubmitInput` values using owner APIs from `mizar-build` for plan
  requests, package manifests, lock files, source layouts, dependency artifact
  indexes, dependency overlays, VC descriptors, scheduler controls, resource
  budgets, and cancellation policy;
- source/dependency/verifier snapshot inputs through the request/session layer,
  never through ad hoc hashing in the CLI.

The CLI must call `CompilerDriver::submit` to create the session. It must not
construct `BuildSession` manually, bypass the phase registry, pre-run phase
services to make scheduler submission succeed, or convert scheduler synthetic
outputs into phase artifacts.

Manifest parsing and lockfile parsing are delegated to `mizar-build`. Diagnostic
record creation, ordering, rendering, and explanation are delegated to
`mizar-diagnostics`. Artifact writing is delegated to the artifact owner seam
when it exists.

## Progress And Diagnostics

Progress is rendered from `BuildEventStream`:

- `SessionAccepted`, `SnapshotCaptured`, `PlanningReady`, `TaskProgress`,
  `PhaseServiceGap`, `DispatchGap`, `OwnerReadinessGap`, `PublicationSuppressed`,
  and `SessionFinished` may be shown as progress/status lines;
- event order must follow the replayed event stream and must not be recomputed
  from worker completion order;
- event text is presentation only and must never become diagnostic identity.

Diagnostics are rendered only from `mizar-diagnostics` records, indexes, or
owner-provided batches. Until the bridge from driver/planning/scheduler records
to diagnostics owner records exists, the CLI must report a classified
`external_dependency_gap` or structured owner-readiness gap rather than inventing
diagnostic ids, codes, severities, or message identities.

Human output writes progress and diagnostics to stderr. Machine output uses JSON
Lines on stdout with a `schema_version`, a stable `kind`, and owner-provided
identities. JSON output must still keep CLI records distinct from LSP JSON-RPC
payloads.

## Exit Codes

The CLI maps terminal outcomes deterministically:

| Code | Name | Meaning |
|---|---|---|
| `0` | `Success` | Driver session finished `Succeeded` and no error-severity diagnostics remain. |
| `1` | `BuildFailed` | Language/build diagnostics reject the package, or the session finished `Failed`. |
| `2` | `Usage` | Invalid flags, conflicting workspace/manifest selection, invalid numeric values, or unreadable command inputs before a driver request is accepted. |
| `3` | `UnavailableOwner` | The request is blocked by an `external_dependency_gap`, `deferred`, or unavailable owner seam, including missing phase services or the scheduler-to-registry dispatch gap. |
| `4` | `Cancelled` | The session finished `Cancelled` or was superseded before publication. |
| `101` | `InternalError` | A driver invariant failed or an unexpected internal error escaped structured diagnostics. |

Exit-code mapping must inspect structured driver/session state and owner
diagnostic severity, not rendered message text.

## Gap Classification

| Gap | Classification | CLI disposition |
|---|---|---|
| Driver-to-diagnostics owner record bridge for planning/scheduler errors is not complete. | `external_dependency_gap` | Report an owner-readiness/gap status; do not allocate diagnostic ids or render fake diagnostics. |
| Filesystem-backed `--manifest-path` selection and workspace/member discovery are not complete in the D-013 library entry point. | `external_dependency_gap` | Exit `UnavailableOwner`; do not pretend a manifest path was selected without an owner-provided batch input. |
| Package/module target filtering beyond the current single-package input is not complete. | `external_dependency_gap` | Accept only package selections that exactly match the supplied package; otherwise exit `UnavailableOwner` until a real target resolver exists. |
| Source layout and request snapshot inputs disagree. | `external_dependency_gap` | Reject before driver submission so a session cannot publish current events for work outside the captured snapshot. |
| Real semantic/proof/artifact phase adapters are unavailable. | `external_dependency_gap` / `deferred` | Exit `UnavailableOwner`; do not report build success or fabricate artifacts. |
| Real artifact publication token/manifest commit seam is unavailable. | `external_dependency_gap` | Do not claim artifact output paths were committed. |
| LSP protocol conversion is outside the CLI. | out of scope | Do not emit JSON-RPC, document URIs, code actions, progress tokens, or LSP severities. |

## Testing Requirements

Task D-013 tests must cover:

- argument parsing into batch request/profile/target/scheduler controls;
- stable exit-code mapping for success, failed diagnostics, unavailable owner
  gaps, cancellation, usage errors, and internal errors;
- human and JSON progress rendering from replayed event order;
- diagnostics rendering only through `mizar-diagnostics` owner records or an
  explicit gap when that bridge is missing;
- no LSP protocol terms, artifact publication tokens, artifact serialization,
  manifest commit calls, committed output-path claims, artifact owner APIs,
  phase semantics, proof acceptance, cache compatibility decisions, or fake
  output refs in `src/cli.rs`.
