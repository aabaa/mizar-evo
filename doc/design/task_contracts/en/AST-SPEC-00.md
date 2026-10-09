# Task AST-SPEC-00: Reflection direction correction

Canonical language: English; [Japanese pointer](../ja/AST-SPEC-00.md).
Status: complete. Tier: full gates. Primary owner: [kernel plan](../../mizar-kernel/en/00.crate_plan.md).
Consumers are architecture 15 and later AST specification tasks. Dependency: user-authorized 2026-10-09 decisions.
Authority: [decision record](../../../spec/temp/algorithm_on_AST.md), reflection
plan unit 6 prerequisite, decisions 13, 18, 23; AGENTS.md and autonomous protocol.
Gap: design_drift: architecture 15 and the roadmap still adopt tree/`Eval`.
Scope: replace only that direction in architecture EN/JA and the roadmap;
record deferred implementation ownership in the coverage audit.
Forbidden: Rust, temp authority, existing .miz/expectations, trace status and
coverage achievement changes; other historical sections remain frozen.
No executable tests change: this prerequisite introduces no runnable behavior.
Acceptance: withdrawn `Eval` direction absent from these live owner paragraphs;
EN/JA agree and later specification transfer remains a deferred task.
Reviews: specification/documentation, test sufficiency, implementation,
volume/scope, source/document consistency, and full-gate quality assessment.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; changed local links and scope checks.
Exit: acceptance satisfied, no unresolved blocking review findings; commit
only task files and preserve the pre-existing .gitignore change.
