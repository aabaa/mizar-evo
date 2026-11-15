# Task QUALITY-LITERAL-OFFSET-RATCHET: no new literal source offsets
Canonical language: English; [Japanese pointer](../ja/QUALITY-LITERAL-OFFSET-RATCHET.md).
Status: complete. Tier: full. Owner: [test plan](../../mizar-test/en/00.crate_plan.md) (repository lint); consumers: every crate with production `src`.
Dependencies: none. Readiness: ready. Gap: `design_drift` (fixture-specific offsets).
Authority: [autonomous_crate_development.md](../../autonomous_crate_development.md#authority-order).

## Scope

Add a repository lint that counts, per source file under crate `src`,
source-range constructions with integer-literal offsets and
`expected_ranges` / `expected_sites` arrays. Independent production/test ceilings may only be lowered or removed; new files have zero allowance.

## Counting boundary

Use test-only `syn` full/visit in the existing lint target; the necessary dev-dependency and unchanged-version lock edge are in scope. No new parser module or public seam.
Include all Rust files under crate `src` by default and external reachable modules. [R5](./REFACTOR-R5-CHECKER-RETIRED-PATH.md) extends accounting to both production and test configurations independently, retaining production ceilings and initializing newly measured test debt. Deduplicate physical source occurrences per configuration; orphan files remain included.
Follow unqualified literal item-position `include!` dependencies in test ancestry recursively, relative to each including file, with mod.rs-style child ownership; this discovers source ownership without macro expansion/generated-path evaluation.
Before any include exclusion, conservatively certify builtin ownership across module/ancestor and complete same-scope literal-include fragments: include macro definitions/imports, namespace globs/macro_use, or unresolved item/procedural/generated binding sources invalidate proof. Unrelated function-local imports/functions/variables are not module macro bindings.
Shadowed/ambiguous, nonliteral, qualified or aliased includes supply no exclusion proof: retain possible production or fail visibly. Production includes, including standard qualified spellings, remain unsupported; any genuine production route wins.
Discover lib/bin manifest roots and Rust module/path edges. Source-read/parse failures, unsupported roots/structural attributes or production `include!` fail visibly. File, item and expression cfg evaluates all/any/not in three-valued logic separately with `test = false` and `test = true`, other predicates unknown; only definitely false code is excluded. Unknown `cfg_attr` affecting cfg/path fails visibly.
One range count means one exact-final-name `SourceRange` struct construction or `SourceRange::new(source_id, start, end)` with either whole-literal endpoint; field order/shorthand and nested real expressions are respected. Source-id literals alone do not count.
Whole literals allow radix/separators/integer suffixes, transparent groups/parentheses and unary signs; arithmetic, casts, variable values and constant evaluation are outside this syntax counter.
Discover module-local free helpers returning exact or qualified `SourceRange` whose complete body is a sole tail exact/qualified `SourceRange` struct or sole explicit return of it, allowing transparent parentheses/groups. Map endpoint fields forwarded directly from identifier parameters (allowing transparent parentheses/groups) to argument positions; one parameter may feed both fields.
Count bare or `self::` helper calls at those positions. Respect parameter/local/closure/loop/match-arm and block-local function/import bindings; ambiguous target-call shadowing fails visibly. Unrelated same-name bindings are not errors. Imported/aliased/cross-module helpers, arbitrary methods and type aliases are excluded.
Test configuration recognizes local, lexically scoped, cfg-aware one-arm expr-fragment macros whose single expression contains one discovered helper call with directly forwarded endpoint metavariables. Count literal endpoints and visit unqualified invocation expressions. Ambiguous shadowing/mapping, malformed calls, multiple arms/helpers or opaque target-bearing bodies fail visibly; production macro policy and non-expansion exclusions remain.
Exact `let [mut] expected_ranges` / `expected_sites` bindings, optionally typed, count one per array element that is a whole integer literal or a tuple with at least one immediate whole integer-literal member. Follow only value-producing references/groups/parentheses, block tails, match arms and if branches; scratch/closure/nested-function arrays do not contribute to this binding.
Ordinary non-array call/path/index initializers contribute zero; types, patterns and unrelated array names do not count. Opaque macros in value-producing named initializer positions and named repeat arrays fail visibly. Empty or single-string-literal `unreachable!` is a known diverging value and contributes no named-array entries. Normal traversal still counts independent genuine range constructions and nested named bindings.
Accept `vec!` expression lists/repeats as array/repeat expressions; visit their real payload expressions. Accept `matches!(expression, pattern [if guard])`, visiting scrutinee/guard, never its pattern.
Accept only `assert!`, `assert_eq!`, `assert_ne!`, `debug_assert!`, `debug_assert_eq!`, `debug_assert_ne!`, `format!`, `format_args!`, `print!`, `println!`, `eprint!`, `eprintln!`, `write!` and `writeln!` as comma-separated expressions, including named format assignments. Malformed target-bearing forms fail visibly.
For unknown invocations and macro definitions, recursively reject literal target shapes: `SourceRange {`, `SourceRange::new(`, discovered helper calls, or exact named-array bindings followed by `=`/type colon. A type-name token alone or identifier text inside a literal is not a target. Macro expansion and metavariable-generated/aliased/imported targets are excluded.
Baseline rows are sorted unique existing scanned source paths with independent production/test ceilings; at least one ceiling must be positive and the other may be zero. Malformed, duplicate, unsorted, all-zero and stale-path rows fail. Missing rows allow zero; current counts may be below their stored ceilings. Historical raises remain review/commit policy, not a claimed Git-history check.
The guard detects per-file count increases above allowance; same-count replacements, compensating removals or spending retained allowance are not location-based detection.

## Forbidden

Rewriting production code in this task; adding fixture or integration-test scan roots;
raising a baseline row without explicit user approval recorded in the commit
body.

## Tests and exit

Regressions cover independent range/helper/macro/array counts, lexical decoys, cfg/module/path/shadowing and baseline exact/lowered/new-zero/increased/malformed/stale cases. Require independent specification, sufficiency, implementation, volume/scope and consistency reviews; `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`. Reject counts above either ceiling; new files have zero allowance. Only `lint_policy.rs`, the required TSV, test dev-dependency/lock edge and necessary existing contract/plan/todo updates may change.
