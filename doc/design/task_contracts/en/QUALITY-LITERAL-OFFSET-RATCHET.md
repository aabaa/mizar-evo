# Task QUALITY-LITERAL-OFFSET-RATCHET: no new literal source offsets in production
Canonical language: English; [Japanese pointer](../ja/QUALITY-LITERAL-OFFSET-RATCHET.md).
Status: complete. Tier: full. Owner: [test plan](../../mizar-test/en/00.crate_plan.md) (repository lint); consumers: every crate with production `src`.
Dependencies: none.
Readiness: Ready as repository-lint work; this task adds no producer generality or semantic credit.
Authority: [autonomous_crate_development.md](../../autonomous_crate_development.md#authority-order); the existing documentation-volume ratchet is the model for baseline handling.
Gap: `design_drift`. Production code in `mizar-checker` and `mizar-core` compares against source ranges and node indices written as integer literals, which only matches specific corpus files.

## Scope

Add a repository lint that counts, per production source file outside test
modules, source-range constructions with integer-literal offsets and
`expected_ranges` / `expected_sites` arrays. Record the current counts in a
baseline whose rows may only be lowered or removed, like
`tests/coverage/doc_volume_baseline.tsv`. A new file must have a count of zero.

## Counting boundary

Use test-only `syn` full/visit in the existing lint target; the necessary dev-dependency and unchanged-version lock edge are in scope. No new parser module or public seam.
Include all Rust files under crate `src` by default and external production-reachable modules. Exclude only proven exclusively test-reachable files/descendants; any possible production route wins and orphan files remain included. Deduplicate physical source occurrences.
Follow unqualified literal item-position `include!` dependencies in test ancestry recursively, relative to each including file, with mod.rs-style child ownership; this discovers source ownership without macro expansion/generated-path evaluation.
Before any include exclusion, conservatively certify builtin ownership across module/ancestor and complete same-scope literal-include fragments: include macro definitions/imports, namespace globs/macro_use, or unresolved item/procedural/generated binding sources invalidate proof. Unrelated function-local imports/functions/variables are not module macro bindings.
Shadowed/ambiguous, nonliteral, qualified or aliased includes supply no exclusion proof: retain possible production or fail visibly. Production includes, including standard qualified spellings, remain unsupported; any genuine production route wins.
Discover lib/bin manifest roots and Rust module/path edges. Source-read/parse failures, unsupported roots/structural attributes or production `include!` fail visibly. File, item and expression cfg evaluates all/any/not in three-valued logic with `test = false` and other predicates unknown; only definitely false code is excluded. Unknown `cfg_attr` affecting cfg/path fails visibly.
One range count means one exact-final-name `SourceRange` struct construction or `SourceRange::new(source_id, start, end)` with either whole-literal endpoint; field order/shorthand and nested real expressions are respected. Source-id literals alone do not count.
Whole literals allow radix/separators/integer suffixes, transparent groups/parentheses and unary signs; arithmetic, casts, variable values and constant evaluation are outside this syntax counter.
Discover module-local free helpers returning exact or qualified `SourceRange` whose complete body is a sole tail exact/qualified `SourceRange` struct or sole explicit return of it, allowing transparent parentheses/groups. Map endpoint fields forwarded directly from identifier parameters (allowing transparent parentheses/groups) to argument positions; one parameter may feed both fields.
Count bare or `self::` helper calls at those positions. Respect parameter/local/closure/loop/match-arm and block-local function/import bindings; ambiguous target-call shadowing fails visibly. Unrelated same-name bindings are not errors. Imported/aliased/cross-module helpers, arbitrary methods and type aliases are excluded.
Exact `let [mut] expected_ranges` / `expected_sites` bindings, optionally typed, count one per array element that is a whole integer literal or a tuple with at least one immediate whole integer-literal member. Follow only value-producing references/groups/parentheses, block tails, match arms and if branches; scratch/closure/nested-function arrays do not contribute to this binding.
Ordinary non-array call/path/index initializers contribute zero; types, patterns and unrelated array names do not count. Opaque macros in value-producing named initializer positions and named repeat arrays fail visibly. Empty or single-string-literal `unreachable!` is a known diverging value and contributes no named-array entries. Normal traversal still counts independent genuine range constructions and nested named bindings.
Accept `vec!` expression lists/repeats as array/repeat expressions; visit their real payload expressions. Accept `matches!(expression, pattern [if guard])`, visiting scrutinee/guard, never its pattern.
Accept only `assert!`, `assert_eq!`, `assert_ne!`, `debug_assert!`, `debug_assert_eq!`, `debug_assert_ne!`, `format!`, `format_args!`, `print!`, `println!`, `eprint!`, `eprintln!`, `write!` and `writeln!` as comma-separated expressions, including named format assignments. Malformed target-bearing forms fail visibly.
For unknown invocations and macro definitions, recursively reject literal target shapes: `SourceRange {`, `SourceRange::new(`, discovered helper calls, or exact named-array bindings followed by `=`/type colon. A type-name token alone or identifier text inside a literal is not a target. Macro expansion and metavariable-generated/aliased/imported targets are excluded.
Baseline rows are sorted unique positive counts for existing production paths; malformed, duplicate, unsorted, zero and stale-path rows fail. Missing rows allow zero; current counts may be below their stored ceilings. Historical raises remain review/commit policy, not a claimed Git-history check.
The guard detects per-file count increases above allowance; same-count replacements, compensating removals or spending retained allowance are not location-based detection.

## Forbidden

Rewriting production code in this task; counting test modules or fixtures;
raising a baseline row without explicit user approval recorded in the commit
body.

## Tests and exit

Unit tests fix independent counts for real range/helper/macro/typed-match-array shapes, literal/nonliteral and lexical decoys, cfg/module/path exclusion and shadowing, plus exact/lowered/new-file/over-ceiling/malformed/stale baseline controls. Require independent
specification, test-sufficiency, implementation, volume/scope, and consistency
reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: the lint passes with the reviewed initial debt; a new file has zero allowance and any per-file count above its ceiling fails. Only `lint_policy.rs`, the required TSV, test dev-dependency/lock edge and necessary existing contract/plan/todo updates may change.
