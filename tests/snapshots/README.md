# Snapshots

Reserved for deterministic snapshot baselines. Snapshot updates must be
explicit and must not be regenerated silently from current compiler behavior.

`parser/*.surface_ast.snap` files are parse-only
`SurfaceAst::snapshot_text()` baselines referenced from `.expect.toml` sidecars
by tests-root-relative `snapshots` paths.

`core/*.core_ir.snap` currently contains exactly the Core Task-31 Task-180
`CoreIr::debug_text()` baseline. Its existing type-elaboration sidecar is the
sole allowlisted consumer. Normal runs are verify-only; they never create or
rewrite this file. Broader CoreIr/ControlFlowIr baselines remain deferred.
