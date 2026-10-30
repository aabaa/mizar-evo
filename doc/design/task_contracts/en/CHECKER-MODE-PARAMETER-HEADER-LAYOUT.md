# Task CHECKER-MODE-PARAMETER-HEADER-LAYOUT: qualified parameter headers
Canonical language: English; [Japanese pointer](../ja/CHECKER-MODE-PARAMETER-HEADER-LAYOUT.md).
Status: complete. Tier: full. Owner: [checker](../../mizar-checker/en/00.crate_plan.md).
Parent: TEST-METAMORPHIC-BRIDGE-GENERALIZATION; dependency: TEST-MODE-JOINT-LOWER-LAYOUT.
Authority: lexical §2.2, types §3.3, scope §4.6, Mode §7.2 and unchanged
Task262 payload MIZ. Gap: upper source_drift after genuine lower header relocation.
Use the [Mode owner](../../mizar-checker/en/source_mode_definition.md#frozen-lower-bundle-and-ownership).

Relax only parameter end/declaration/written-Type ranges and physical spelling
when upper input, Context declaration, actual BindingEnv and Type application/root
agree on the header tuple and owned header Arena nodes are exact and normal.
Parameter starts13/29, names x/y, sites37/41, Type sites35/34/39/38, order and
all non-header definition/resolver/block/pattern/RHS/property geometry stay fixed.
Full parameter envelopes remain ordered/nonoverlapping before definition45;
source IDs match, declaration/type widths stay1/3, and physical spelling length
matches its envelope. Require exact let/name/be/set/semicolon fragments at their
claimed relative offsets, separated only by space/tab/LF; word gaps are nonempty.
No comment, CR, new lexical family or independent loaded-source seal is granted.

Eligibility is non-error-producing. Unequal/unavailable header claims and inexact
or recovered owned header Arena make it false; retain ALL canonical checks and
original phases in that case. In particular moved Context/canonical Type retains
InvalidParameter; the inverse retains DependencyMismatch. Moved agreed headers
with containing Type anchors retain canonical fallback InvalidParameter; canonical
late Arena faults retain InvalidArenaOwnership. Eligibility never replaces later
validation, normal predicates, fingerprints or installation/replay checks.
Keep dependency/shape/baseline/resolver/owner-before-header order. A newly qualified
header may expose existing later non-header errors; no new blame classifier.

Test first: original success and actual public Context→its BindingEnv→generic
Type→Mode-RHS plus joint Typed/final before upper accumulated RED for x-only,
both-parameter moves and TAB/LF spelling. Use real compensated source fragments: extra space before
x or y, removed following indentation, preserving suffix29 or45 and length141.
GREEN observes physical headers, fingerprints, unchanged sibling/body identity,
nonempty unrelated baseline plus one Pending Sethood, upper Typed/final/replay,
input immutability and semantic isolation. Cover both unequal Type/Context and
input/Context declaration directions, unavailable tuple, fragment/offset/gap/width
refusals, exact/containing Arena fallback and earlier-phase controls. Preserve old helpers/assertions/oracles;
reuse prior lower/corruption proofs, not duplicate matrices or mutate sealed positives.

No public seam, new payload representation/provider/standalone one-use helper,
production lower/runner/Core change, MIZ/expectation/hash/debug/trace/exception or
semantic-credit change. Only upper Mode, its support test, paired live owner/
plan/TODO/contracts and measured inventory/ratchet decreases may change.
Require SPEC/intent/trust, TEST, CODE, VOLUME/repeat, consistency and scoped gates;
focused Mode/249M/checker/Core/Task262/metamorphic/type corpus/docs/link checks,
fmt/Clippy and inherited full cadence. No raw runner, parent or crate exit.
