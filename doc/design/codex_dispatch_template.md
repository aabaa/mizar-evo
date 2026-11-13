# Codex Dispatch Template

Owner: the outer orchestrator (Muse, or the user acting in that role). This
file is the fill-in form and pre-dispatch checklist only. The rules live in
the protocol's [Dispatch Boundary](./autonomous_crate_development.md#dispatch-boundary)
and [Gate Tiering](./autonomous_crate_development.md#gate-tiering); the dispatch
fields and the final-response form live in
[`AGENTS.md`](../../AGENTS.md#dispatch-boundary).

## Before Dispatching

1. Tier. A further slice of a family with a full-tier predecessor is pattern
   tier by default. Declare full tier only by naming the pattern criterion
   the slice fails (new owner module, consumer, refusal class, public seam,
   semantic credit, or protected surface).
2. Bundle. Compare the next open ledger candidates and bundle three to five
   identical-shape slices into one dispatch.
3. Freeze. A slice whose contract is not yet frozen is dispatched once for
   freeze and implementation. Dispatch a freeze alone only when it needs an
   orchestrator or user decision, and name that decision.
4. Test matrix. The contract fixes each case's input shape, expected outcome,
   and a case cap. For an unfrozen slice, the unit text states the intended
   matrix so the freeze can record it.
5. Verification state. A freeze and implementation pair counts as one changed
   task. Name the full `cargo test` trigger when one applies and set `push`.
6. Concurrency. Run one dispatch per worktree at a time and do not edit that
   worktree while a run is active; land process changes between runs or
   through a separate worktree.

## Form

```text
# Dispatch from <orchestrator> (outer orchestrator)

This is a dispatch under the AGENTS.md Dispatch Boundary. Execute exactly this
one unit, then write the fixed-form final response and end the run.

- Unit: <freeze and implement | implement> <contract id> (<contract path>;
  JA pair alongside): <one-sentence scope>. <Bundled slices, if any.>
- Authority paths: <contract>; <family ledger row>; <checkpoint commits>.
- Gate tier: <pattern | full: failed criterion | light>.
- Test matrix: <"fixed in the contract" | intended cases and cap>.
- Verification state: last full success <rev>; outstanding <n> -> <n+1>;
  full trigger <none | name>.
- push: <yes | no>
- Model and reasoning setting: <model>/<effort> (set by the launcher).

Codex agent を使い、AGENTS.md のワークフローに従って、完了まで進めてください。
```

Launch with `codex exec -m <model> -c model_reasoning_effort="<effort>"
-s workspace-write -o <run>/last_message.txt "$(cat <run>/dispatch.md)"` from
the workspace root. The Rust toolchain is already on the sandbox `PATH`; do
not add toolchain variables to the dispatch.
