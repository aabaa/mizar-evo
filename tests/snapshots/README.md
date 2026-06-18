# Snapshots

Reserved for deterministic snapshot baselines. Snapshot updates must be
explicit and must not be regenerated silently from current compiler behavior.

`parser/*.surface_ast.snap` files are parse-only
`SurfaceAst::snapshot_text()` baselines referenced from `.expect.toml` sidecars
by tests-root-relative `snapshots` paths.
