# Module-Boundary Audit: mizar-test Runner

> Canonical language: English. Japanese companion:
> [../ja/module_boundary_audit.md](../ja/module_boundary_audit.md).

## Task 248 Gate

Task 248 audits the active-runner implementation before any source move. The
maintenance series repairs a `design_drift` in source layout and reviewability;
it does not change Mizar language behavior, runner admission, public APIs,
diagnostics, detail keys, payloads, ordering, expectation meaning, or
traceability credit.

The authority order remains `doc/spec/en` > `.miz` tests > `spec_trace.toml` >
expectations > design > source. Chapters 03, 04, 07, 13, 14, and 16 and their
existing executable intent remain inputs to the runner, not targets of this
refactor. [harness.md](./harness.md), [minimal_crate.md](./minimal_crate.md),
[expectation_schema.md](./expectation_schema.md), and
[internal 07](../../internal/en/07.crate_module_layout.md) define the derived
harness and ownership boundaries.

## Baseline

At Task 248 inventory:

- `src/runner.rs` has 111,262 lines;
- the pre-test prefix ends at line 17,142 and contains the public runner
  facade, private phase helpers, and 137 `#[cfg(test)]` helper attributes;
- one private `mod tests` begins at line 17,143 and occupies about 94,120
  lines;
- the private module contains 272 `#[test]` attributes: 244 at its direct
  scope and 28 in existing nested task modules;
- its direct tests comprise one parse-only import-provider test and
  type-elaboration source-extraction, payload, fixture, corruption, and
  cross-owner isolation families;
- declaration-symbol runner tests are integration-owned by
  `tests/metadata.rs`; no private declaration-symbol test exists to move;
- the active type-elaboration runner has 188 cases, the metadata plan has
  403 cases / 367 requirements, type-elaboration coverage is 235 / 223,
  pass/fail is 219 / 184, and the unit-test count is 272.

## Task 249 Move Result

Task 249 replaced the inline module with private `#[cfg(test)]`,
`#[rustfmt::skip] mod tests;` and moved its body byte-for-byte to
`src/runner/tests.rs`. The formatter guard prevents the newly top-level test
imports and body from being reordered or reflowed during this move-only task.
The runner file is now 17,144 lines and the test module is 94,118 lines. The
exact extracted body hash is
`ab658ad10bcbb2d415778f6289cbb9ae2bed48e21c19b5496fa8f676309d3b69`;
the sorted 272-test list remained
`5e41e4dbfcc303322c246a612de61926a628957a168589b45864d0a5070bb07e`.
Module privacy, qualified test names, public API, active-runner counts,
diagnostics, payloads, ordering, and fail-closed behavior are unchanged.

## Task 250 Move Result

Task 250 root-included `src/runner/tests/support.rs` without a wrapper module.
The 6,546-line fragment contains the 17 import groups and the contiguous shared
environment, fixture-specification, AST-builder, corruption, range, and id
support: 201 non-test functions and 24 type/constant items. Its exact moved
hash is
`b880b4605345b1156f125292134d62aff91a32799b5f5834fe7d2a1e5de068a8`.
The retained 87,572 lines remained byte-identical with hash
`197f2d6dc31da2130674954667383bb9aec502a613f3e5b1c33bf0299ea2959b`;
the resulting 87,574-line `tests.rs` hash is
`7d85a8ecd4dffcb0475afc53693e581af661ccbb01b44eab974e030abb046a66`.
All 272 tests remain in `runner::tests` with the same sorted-name hash.

## Task 251 Move Result

Task 251 root-included `src/runner/tests/parse_only.rs` without a wrapper
module. The byte-identical 111-line fragment contains only
`parse_only_provider_resolves_every_stub_and_deduplicates_fixture_summaries`
and has hash
`3cddce85155b72597cfc4c2ea5841dbf3fe5f88d0c8123d98ba9cb958f90a3a8`.
The retained 87,463 lines, including the separator blank, remained
byte-identical with hash
`010f86378bca27c0620998c0de0242d6376fb8b3c37c002d0ca430fb01f7e35c`;
the resulting 87,464-line `tests.rs` hash is
`16480c65416a611c732153360775f10180f609b012027b0a0a970cff1f5a3d84`.
The fully qualified test name and sorted 272-test list are unchanged.

## Task 252 Move Result

Task 252 root-included
`src/runner/tests/type_elaboration/source_extraction.rs` without a wrapper
module. The byte-identical 3,680-line fragment contains the three baseline
reserve extraction, local-mode expansion-chain extraction, and real
declaration-checked `ResolvedTypedAst` handoff tests. Its hash is
`aa9a16c3ed36439ac8c5a4756e3818d6e5f0abd7e076e2e2df8b46487e88c358`.
The retained 83,784 lines, including the separator before Task 253, remained
byte-identical with hash
`2d9ef7d8369c4d654af3bd91598d306c8a9777c9d0981454ce9396095c8a6d79`;
the resulting 83,785-line `tests.rs` hash is
`16f3d6ceb1e75655ea39825f0294896393e676d0a7391bb2a409e14b3b904d22`.
All three fully qualified names and the sorted 272-test list are unchanged.

## Task 253A Move Result

Task 253A root-included
`src/runner/tests/type_elaboration/reserved_binary.rs` without a wrapper
module. The byte-identical 9,982-line fragment contains the leading 23
baseline reserved-variable and binary-formula bridge tests, including the
audited Task 189 and Task 246 ownership exceptions. Its hash is
`88f1a793e139ea808c823fd68956d0dc9863735905ae3fb34e214efa86a11d8e`.
The retained 73,803 lines, including the separator before Task 254, remained
byte-identical with hash
`faf592952a4c871b840b6a1cbbb977ca3f1bbddc98def4f99d54c1a900fdcb06`;
the resulting 73,804-line `tests.rs` hash is
`97d05a3dc35774246af301ad7b4dc6601d2ab85ca669bebfdbcfa140767d150f`.
All 23 fully qualified names, their original order position, and both the
canonical raw and secondary normalized 272-test list hashes are unchanged.
Task 253 remains pending until Task 253B.

## Task 254 Move Result

Task 254 root-included
`src/runner/tests/type_elaboration/mode_chain.rs` without a wrapper module.
The byte-identical 10,232-line fragment contains the 26 audited non-long-chain
local-mode/object-mode tests. Its hash is
`2989031d64871c726f325a5d5bd2ebb4ed4b9a078b83bab3c4f04f456cf3225f`.
The retained 63,572 lines, including the separator before Task 253B, remained
byte-identical with hash
`6725980d7842af5c398f58139ce371ac64d8912ba744f4417ac20c88165d5d81`;
the resulting 63,573-line `tests.rs` hash is
`7e5d0f5735c551be19ac13b2dc96732bf4a9f3cd7088317beb22c760e0d03b68`.
All 26 fully qualified names, their original order position, the Task 253B
boundary, and both 272-test list hashes are unchanged.

## Task 253B Move Result

Task 253B root-included
`src/runner/tests/type_elaboration/reserved_direct.rs` without a wrapper
module. The byte-identical 284-line fragment contains the two audited direct
reserved-variable membership and inequality tests. Its hash is
`c65a5f27463950979368bc702e36f42fa0398884029cff450b54b31095f30e4e`.
The retained 63,289 lines, including the separator before Task 255, remained
byte-identical with hash
`fffe06106cca615e370bb4c2da222da5a4bc21a264cadb5ae8c2d79ed7fdbcce`;
the resulting 63,290-line `tests.rs` hash is
`c90905d94abd1a43c0d65d4abffe8bc970262eee2d64e22da1db4024d614bbf4`.
Both fully qualified names, their original order position, the Task 255
boundary, and both 272-test list hashes are unchanged. Parent Task 253 is
complete.

## Task 255A Move Result

Task 255A root-included
`src/runner/tests/type_elaboration/asserted_head_base.rs` without a wrapper
module. The byte-identical 6,653-line fragment contains the 12 audited leading
source tests and both dedicated Task 205 isolation helpers. Its hash is
`9ecea3c52ae64b83d6d5de9b825307f31c7d331e3ba29d78bb69cd931709d020`.
The retained 56,637 lines, including the separator before Task 255B, remained
byte-identical with hash
`d9f772962e590f49d188ca1d0cbe8cf5863b7dd84bb9e73606f878f33036007a`;
the resulting 56,638-line `tests.rs` hash is
`535968a53524b3741d9adeed0ee6e42f2e4c45184af285a2bae077810b3bd682`.
All 12 fully qualified names, helper ownership, their original order position,
the Task 255B boundary, and both 272-test list hashes are unchanged. Parent
Task 255 remains pending.

## Task 255B Move Result

Task 255B root-included
`src/runner/tests/type_elaboration/asserted_head_four_edge_radix.rs` without a
wrapper module. The byte-identical 3,303-line fragment contains the two audited
four-edge radix source tests and four dedicated Task 208/207 helper functions.
Its hash is
`5fcc4240fff400bda08e3d6678a61f3db444f0a8c6c055802d7ba7bea961092e`.
The retained 53,335 lines, including the separator before Task 255C, remained
byte-identical with hash
`16d36bc1978973931a673a7620c569c70b021fe4ed210e19540a0ee8fa7c7d9d`;
the resulting 53,336-line `tests.rs` hash is
`78594f98a92a30445d251cf0fb394e5537ecab73cf9b8e9c67c357e4a0135389`.
Both fully qualified names, all four helper owners, their original order
position, the Task 255C boundary, and both 272-test list hashes are unchanged.
Parent Task 255 remains pending.

## Task 255C Move Result

Task 255C root-included
`src/runner/tests/type_elaboration/asserted_head_three_edge_object_radix.rs`
without a wrapper module. The byte-identical 1,278-line fragment contains the
one audited three-edge object-radix source test and its two dedicated Task 206
helper functions. Its hash is
`c5c1b04ab663fe3557e24c86b551352d6d1c54c5511870ba224edb7538f95442`.
The retained 52,058 lines, including the separator before Task 255D, remained
byte-identical with hash
`e841b80390d879d910bfc50a34547ef56b8b2ab40c6c4b9681e8b07f707dc12b`;
the resulting 52,059-line `tests.rs` hash is
`23caa0585a96be2db997295fccad436de5bfefdbe033fdd4516ca8e30dacea9f`.
The fully qualified name, both helper owners, original order position, Task
255D boundary, and both 272-test list hashes are unchanged. Parent Task 255
remains pending.

## Task 255D Move Result

Task 255D root-included
`src/runner/tests/type_elaboration/asserted_head_two_edge_object_radix.rs`
without a wrapper module. The byte-identical 1,046-line fragment contains the
one audited two-edge object-radix source test and its dedicated Task 204 helper
function. Its hash is
`e20a04ba33ffc1f344da0aa990795576b7096eb6a016a69d730d0d29377349f4`.
The retained 51,013 lines, including the separator before Task 255E, remained
byte-identical with hash
`16d6ec2333861ac9d78d3694efe76a71bb9a9830f16def60c4a425fb7da63dc7`;
the resulting 51,014-line `tests.rs` hash is
`68bf3cf08b26a449f46aee00d7fe8f716d1663ac9aeb7005b311f4f7c6c15906`.
The fully qualified name, helper owner, original order position, Task 255E
boundary, and both 272-test list hashes are unchanged. Parent Task 255 remains
pending.

## Task 255E Move Result

Task 255E root-included
`src/runner/tests/type_elaboration/asserted_head_type_assertion.rs` without a
wrapper module. The byte-identical 7,649-line fragment contains the final 16
audited non-long-chain type-assertion/asserted-head source tests and no helper
functions. Its hash is
`27bb8b3f17cabfce79ec9e32e390fbad3c9356c845dab4c7fb53dfd9f3b5160a`.
The retained 43,365 lines, including the separator before the first active
fixture, remained byte-identical with hash
`b0465c9378a8f0151e0c58ba4986876f3de163ceb5918b7ceb49db4462b6d1c3`;
the resulting 43,366-line `tests.rs` hash is
`75fc0ff2b4a48362a1184185ea1315c0d8dab90b9b5a9b45a3fafe13b14d7278`.
All 16 fully qualified names, their original order positions, assertions, the
following active-fixture boundary, and both 272-test list hashes are unchanged.
Parent Task 255 is complete.

## Task 256 Move Result

Task 256 root-included `src/runner/tests/type_elaboration/long_chain.rs`
without a wrapper module. The byte-identical 20,977-line fragment contains all
44 audited long-chain source/active seven-expansion tests. Its 12
`next_permutation` functions remain nested test-local finite guards; no
module-level helper or unrelated item moved. The fragment hash is
`c4bcb161ac7bbb03593beff0fd55c6fbf8bc1960618a92263d127856e709d8b0`.
The retained 22,389 lines remained byte-identical with hash
`d737b5160458533039c7535423cffa03265deacb719d167e486897a612d7afbf`;
the resulting 22,390-line `tests.rs` hash is
`603263b325a00d45a41ec3087dafab05ab4ebe448fe3be70a7c0d107f907df8d`.
All 44 fully qualified names, original order positions, finite guards,
assertions, and both 272-test list hashes are unchanged. The preceding
four-edge equality test and following four-edge inequality test remain in
`tests.rs`, and Task 257's nested Task 216-222 modules remain outside the
fragment. Task 256 is complete; Task 257 is next.

## Task 257A Move Result

Fresh Task 257 inventory divides the remaining 141 tests into eight contiguous,
order-preserving families. Task 257A has 18 binary-route tests; Task 257B has
three builtin-object reserve fixtures; Task 257C isolates the one standalone
Task 180 contradiction formula-constant fixture under its dedicated trace
intent; Task 257D has 11 distinct/multiple/heterogeneous reserve fixtures;
Task 257E has 26 mode-chain fixtures; Task 257F has 35 active
reserve/asserted-head/type-assertion fixtures plus four interleaved owner-route
isolation guards; Task 257G has three source-gap/equality tests; and Task 257H
has nine root source/active bridge fixtures, three root synthetic/route-
isolation tests, plus 28 tests nested in the existing Task 216-222 modules.
The eight counts total the remaining 113 root and 28 nested tests. Parent Task
257 remains pending through 257H.

Task 257A root-included
`src/runner/tests/type_elaboration/binary_route_fixtures.rs` without a wrapper
module. The byte-identical 2,960-line fragment contains the 18 audited
binary/parenthesized active-fixture and route-isolation tests and no
module-level helper or unrelated item. Its hash is
`b00af949465486166f8a5d012dce6b02345aad29b2e576c4b574cf1c6ea23eee`.
The retained 19,430 lines, including the separator before Task 257B, remained
byte-identical with hash
`d07c5006c01b8975342d95a5fff8c447106c38e8754ddaac2f87be442c7d07a5`;
the resulting 19,431-line `tests.rs` hash is
`e2f877ddf29c6f9e2e22225e97ff4294d7e27affda04145f78a950e567022e5e`.
All 18 fully qualified names, original order positions, assertions, the Task
257B boundary, and both 272-test list hashes are unchanged. Task 257A is
complete; parent Task 257 remains pending.

## Task 257B Move Result

Task 257B root-included
`src/runner/tests/type_elaboration/reserve_object_fixtures.rs` without a
wrapper module. The byte-identical 156-line fragment contains only the three
audited Task 188/190/189 builtin-object reserve equality, inequality, and
type-assertion active fixtures. Its hash is
`9cfb91fad7f537fbe790ac8e8206e383b0068a8bdcb14158c554219702d9446f`.
The retained 19,275 lines, including the separator before Task 257C, remained
byte-identical with hash
`c4459d3170895c98e4d6018ae491adce8889f12351a9a4b834c8669e84eb285d`;
the resulting 19,276-line `tests.rs` hash is
`509d784ce5f2b23c98675fdfcb74324dfede166204067c8c3bdd0a1339ba6d18`.
All three fully qualified names, original order positions, assertions, and
both 272-test list hashes are unchanged. The Task 180 contradiction fixture
remains in `tests.rs` as the first Task 257C item. Task 257B is complete;
parent Task 257 remains pending.

## Task 257C Move Result

Task 257C root-included
`src/runner/tests/type_elaboration/formula_constant_fixture.rs` without a
wrapper module. The byte-identical 53-line fragment contains only the audited
Task 180 standalone contradiction active fixture and its exact checked
`FormulaKind::Contradiction` payload assertions. Its hash is
`986b9120d84a487093c4ce3392a11eba03d65441cfb66d09ec9c34bc72dc03c5`.
The retained 19,223 lines, including the separator before Task 257D, remained
byte-identical with hash
`e271687874a614c317a3d0a6a7ff3da5b1081235c9ec18233ddefc91167122a0`;
the resulting 19,224-line `tests.rs` hash is
`a8140de0a533cb4e2f3d4093155d14f188abcef707094a2b10fe5dda469958ad`.
The fully qualified name, original order position, assertions, both
reserve-family boundaries, and both 272-test list hashes are unchanged. Task
257C is complete; Task 257D is next and parent Task 257 remains pending.

## Task 257D Move Result

Task 257D root-included
`src/runner/tests/type_elaboration/reserve_fixtures.rs` without a wrapper
module. The byte-identical 739-line fragment contains the 11 audited distinct,
multiple-declaration, and heterogeneous reserve active fixtures and no
module-level helper or unrelated item. Its hash is
`24b4811f26418afe9de5efbf0cf3d7ea54be329ddf1255f89bafc38546301b40`.
The retained 18,485 lines, including the separator before Task 257E, remained
byte-identical with hash
`5dfbf14737caf47e36f7a0c6bb6a1cab58bea8d608da41c0d74cf1fd58eeda4f`;
the resulting 18,486-line `tests.rs` hash is
`4e40491533df5102655f803e899c032d20adbcaf4c68c6e4980867da87849cf0`.
All 11 fully qualified names, original order positions, assertions, the Task
257E mode-chain boundary, and both 272-test list hashes are unchanged. Task
257D is complete; parent Task 257 remains pending.

## Task 257E Move Result

Task 257E root-included
`src/runner/tests/type_elaboration/mode_chain_fixtures.rs` without a wrapper
module. The byte-identical 1,578-line fragment contains the 26 audited
non-long-chain local-mode/object-mode active membership, equality, and
inequality fixtures and no helper or unrelated item. Its hash is
`9e3c1a6e11b01dc257982002379d884f9de24ec5093982d7604e9a988dc2e593`.
The retained 16,908 lines, including the separator before Task 257F, remained
byte-identical with hash
`dd144c50d0b24adfc690e99f160e5ab73362f6b972107ac71ff6bed0513a3774`;
the resulting 16,909-line `tests.rs` hash is
`cacc1dd5a5fcd2e14526bac47e277d900b0c0f9b56c6cc1bee2b7ea2e7229c3f`.
All 26 fully qualified names, original order positions, assertions, the Task
257F boundary, and both 272-test list hashes are unchanged. Task 257E is
complete; parent Task 257 remains pending.

## Task 257F Move Result

Task 257F root-included
`src/runner/tests/type_elaboration/asserted_head_fixtures.rs` without a wrapper
module. The byte-identical 3,374-line fragment contains 35 audited active
reserve/asserted-head/type-assertion fixtures plus the four interleaved
two-hop owner-route isolation guards. It contains no helper, unrelated item,
or long-chain test. Its hash is
`19623c52e34c57fc664f01370139ce253a834513c47fe0f6b7b2563f7684bf26`.
The retained 13,535 lines, including the separator before Task 257G, remained
byte-identical with hash
`4c19658998190c21cbd8a72efa112e29659664d55a7c5b3040ef54ec7cbbb3e8`;
the resulting 13,536-line `tests.rs` hash is
`9e3bb0de8742d0371e4e686815ba70337b8c278a1e26799069baef8758e093ec`.
All 39 fully qualified names, original order positions, expansion/payload and
prior-owner rejection assertions, the Task 257G boundary, and both 272-test
list hashes are unchanged. Task 257F is complete; parent Task 257 remains
pending.

## Task 257G Move Result

Task 257G root-included
`src/runner/tests/type_elaboration/source_gap_and_equality.rs` without a
wrapper module. The byte-identical 2,923-line fragment contains only the
source-reserve gap/evidence fail-closed test and the four-edge local-mode
equality source/active pair. Its hash is
`7726ee451322c547406da5c5b3800be2527685df41ca2de4dc60d47644164487`.
The retained 10,613 lines remained byte-identical with hash
`2ebb32f99fa9001d0a5d303deb5f477a369074b54b3b71ba2ea690aa3f38e49c`;
the resulting 10,614-line `tests.rs` hash is
`b1a22962fefb7a2cc54aa37ff5f601c9995bce432d78fa499cfca8e6c35423bf`.
All three fully qualified names, original order positions, detail-key and
fail-closed assertions, the immediately following `long_chain.rs` include,
Task 257H start, and both 272-test list hashes are unchanged. Task 257G is
complete; parent Task 257 remains pending.

## Task 257H Move Result

Task 257H root-included
`src/runner/tests/type_elaboration/remaining_bridges_and_nested_isolation.rs`
without a wrapper module. The byte-identical 10,578-line fragment contains
the final nine root source/active bridge fixtures, three root synthetic or
route-isolation tests, and all 28 tests inside the existing seven Task
216-222 modules. Its hash is
`96a64963bc06ec3f6f076d00296ebb48450611fb6a512d5f16283c2999e43d50`.
The retained 36 lines remained byte-identical with hash
`a3cba5854fc315b6c9c3dd20be2fdeaf7a5e972cb7a626299d2dcb2bb6c56f06`;
the resulting 37-line `tests.rs` hash is
`0e9b7e861a13fe593435ee8169c28658b5290f054789a3e2f73b896fa2b39061`.
All 40 fully qualified names, original order positions, seven nested module
names, bridge and isolation assertions, and both canonical 272-test list
hashes are unchanged. Task 257H and parent Task 257 are complete; the private
test layout is stable, and Task 258 is next.

## Task 258 Move Result

Task 258 created the private `src/runner/shared.rs` owner and moved the two
cohesive source/frontend staging fragments out of `runner.rs`. The original
30-line frontend-execution/result fragment had hash
`7d03c8561f87b95d5b777beba830998f44c0cd1cbe72a245c29573a64fa1b0f6`;
the original 89-line package/root/path/snapshot fragment had hash
`34fd4b86829394b95f5ae3125c5bf2f010b0ca0357254ea93446e50e7f384672`.
The resulting 138-line `shared.rs`, including its direct dependency imports,
has hash
`11a52bf7fb0e729ac680df33dfa4b7fd65b9fdd922ee9aca6e9ba4a96d7f8f56`,
and `runner.rs` has 17,022 lines with hash
`dde9e23dfb8092be02f3b1139b59dbfddcbb8e55c0c21eac7ad70e1f1fcbda04`.
Only `run_frontend`, `FrontendRun` and its fields, root normalization, and the
shared module-path projection use parent-only `pub(super)` visibility;
package preparation, cleanup, temp naming, and snapshot identity remain
private to `shared.rs`. Direct imports keep the owner independent of the
facade namespace; the explicitly imported parent-owned
`ParseOnlyImportProvider` is the sole temporary parent dependency until Task
261 rather than being generalized or moved early. Public surface hash
`0cb48ae8ac2ccdf14595112df24b8a4c083a989a631580e9044707aa514a267e`,
all four CLI byte hashes, both 272-test list hashes, counts, payloads,
diagnostics, ordering, and fail-closed behavior are unchanged. Task 258 is
complete; Task 259 is next.

## Task 259 Move Result

Task 259 created the private `src/runner/parse_only.rs` owner and moved the
three cohesive parse-only fragments out of `runner.rs`. The original 51-line
case-execution fragment had hash
`6ff68ec8610c9e5ded44f69369850e11d7adfbaf1685f540398fd465d58f4361`;
the original 24-line failure-projection fragment had hash
`2504fbeae49d240c8897f50f00303124ab7c0c3d4bde56393a316dc2419d4275`;
the original 32-line Surface-AST snapshot comparison fragment had hash
`e8e1698aa3af9e86e80baf03f799af89490782e3202c20ab22a58011f6d65176`.
The resulting 121-line `parse_only.rs`, including direct dependency imports,
has hash
`d1c1dd0f0c322f3bd4a6e829e66bf6aeaf0dc01b46d60dd177a7fe8e4619ae5a`,
and `runner.rs` has 16,913 lines with hash
`5579a126eccfbbb937e36149d74a940e146619254c1bb8301dca57d191cdfec9`.
Only the case runner and failure projection use parent-only `pub(super)`
visibility; snapshot comparison remains private. The owner calls sibling
`shared::run_frontend` directly and keeps only
`assertion_diagnostic_codes` and `frontend_error_code` as explicit temporary
parent diagnostic dependencies until Task 263. The fixture import provider
and its adapters remain in `runner.rs` for Task 261; no fixture ownership moved
early. Public surface hash
`0cb48ae8ac2ccdf14595112df24b8a4c083a989a631580e9044707aa514a267e`,
all four CLI byte hashes, both 272-test list hashes, counts, payloads,
diagnostics, ordering, and fail-closed behavior are unchanged. Task 259 is
complete; Task 260 is next.

## Task 260A Move Result

Fresh Task 260 review split the original production-helper task into two
independent ownership moves: Task 260A for the cross-phase resolver collection
leaf and Task 260B for its declaration-symbol caller. Task 260A moved the
shared leaf first. The original 29-line `ResolverSymbolCollection` and
shell/projection/collection fragment had hash
`b7f13156c77bfc75d5f6a4f1682fe752b4fe9dfd12b3c7c0cd3913cef44458e0`;
the original 18-line resolver module-id and diagnostic projection fragment had
hash
`d1bed7b1c59ab13e997a72ed492fdfdabf38466a9921c0254be64934846e1c61`;
the original 9-line diagnostic-class key fragment had hash
`363ae1321d663c1d597cdf033c449fe0226c87672e2eefd3bf92b819458cb0e0`.
The resulting 203-line `shared.rs` has hash
`0cd2eb09c043e564470b4003a34dfc4f9e89cb695b1d2df1404b76dd7e8bc299`,
and `runner.rs` has 16,851 lines with hash
`72340a9aeca93ec338375b8bfc51beeb47a2499325faf452733c3e1dec48bbab`.
Only `resolver_symbol_collection`, its result type, and the three result fields
used by the existing declaration and type consumers have parent-only
`pub(super)` visibility; module identity and diagnostic projection remain
private to `shared.rs`. Neither caller moved or changed. Public surface hash
`0cb48ae8ac2ccdf14595112df24b8a4c083a989a631580e9044707aa514a267e`,
all four CLI byte hashes, both 272-test list hashes, counts, payloads,
diagnostics, ordering, and fail-closed behavior are unchanged. Task 260A is
complete; Task 260B is next.

## Task 260B Move Result

Task 260B created the private `src/runner/declaration_symbol.rs` owner after
Task 260A had moved its shared resolver prerequisite. The original 37-line
case-execution fragment had hash
`b58aebc17cd350c5107775b9027d78037b32e0bb1d72782e101746dd6c2d318f`;
the original 36-line observation fragment had hash
`8e9bb3e70c1368aa1882bf623b13664ea12129ffc9f6f44a079148f5eee29631`;
the original 125-line payload encoding, classification, and expected-value
projection fragment had hash
`02df2d29157e2469ca8139178dec9cabd199d25fdfa554749d999556b2b05376`;
the original 19-line failure-diagnostic fragment had hash
`3b366648f438663e7412c2e567bb307ff7245b92739f9bbed38a16fd8862573e`.
The resulting 231-line `declaration_symbol.rs`, including direct dependency
imports, has hash
`cf29e362d3109fc8a45e366c8abaa9f98baae7329f83c3556fe8452ec3347232`,
and `runner.rs` has 16,632 lines with hash
`a6e9d547d68e18e1de2d22ce4393552cf760e8f6b8081fe608f8ffdcab67005d`.
Only the case runner and failure projection use parent-only `pub(super)`
visibility; observation, payload encoding, classification, and expected-value
projection remain private. The owner consumes `shared::run_frontend` and
`shared::resolver_symbol_collection` directly. Its sole temporary parent
dependency, `frontend_detail_keys`, retains plain private visibility because a
child module can access its parent's private item; Task 263 will move the
common diagnostic family. Existing `tests/metadata.rs` integration ownership
is unchanged. Public surface hash
`0cb48ae8ac2ccdf14595112df24b8a4c083a989a631580e9044707aa514a267e`,
all four CLI byte hashes, both 272-test list hashes, counts, payloads,
diagnostics, ordering, and fail-closed behavior are unchanged. Tasks 260A-260B
are complete; Task 261 is next.

## Task 261 Move Result

Task 261 created the private `src/runner/import_fixtures.rs` owner and moved
the two families that share the single `parser.type_fixtures` vocabulary. The
original 161-line type-elaboration import-summary adapter fragment had hash
`98d9ebc8ff104583bca469f66a89c5f90dfd91085f811012fc06d173b6224d8b`;
the original 167-line lexical-summary provider and 15-symbol vocabulary
fragment had hash
`3097dc061f34ef0d08482aa785f7827b38b17a8b15dbc8f9fc0e7ca876a49c34`.
The resulting 349-line `import_fixtures.rs`, including direct dependency
imports, has hash
`bb2d10572184600c2121ae680ff936586a8b525eaea7e2a358f1d3b4305bc04d`,
and `runner.rs` has 16,293 lines with hash
`5e878da91e11b7d69709e94dfc9ad851e298fe7b46ed111c174696c2e2b12363`.
Only `ParseOnlyImportProvider`, the type import-summary adapter entry, and the
module-path projection still consumed by a Task 262 parent caller use
parent-only `pub(super)` visibility; vocabulary, environment cloning, imported
module discovery, and symbol-kind mapping remain private. The parent keeps
private aliases so `shared.rs` and existing test support retain their imports
without test edits. Stub order/span identity, per-module deduplication,
fingerprints, the exact 15-symbol kind/arity/operator/rank order, resolver
symbol/provenance order, and diagnostics are unchanged. Public surface hash
`0cb48ae8ac2ccdf14595112df24b8a4c083a989a631580e9044707aa514a267e`,
all four CLI byte hashes, both 272-test list hashes, counts, payloads,
diagnostics, ordering, and fail-closed behavior are unchanged. Task 261 is
complete; Task 262 is next.

## Task 262A Move Result

Fresh Task 262 inventory found a shared source-AST leaf that must precede both
the reserve and formula extractor owners. Task 262A created the private
`src/runner/type_elaboration.rs` phase facade and private
`src/runner/type_elaboration/source_ast.rs` leaf, then moved the exact
compilation-item-list recognizer, structural-child projection, direct-token
projection, checker-site projection, and recursive recovery predicate. Their
original fragment hashes were, in that order,
`84bf7a706ff2295e0087484fda11f210a6f363f4bfa386567004b1b291abcb1b`,
`5684a8ad7fa11893580921465265d7343a6cd1d9824ad5a9b9b6443153380981`,
`3c7621566d18a891f2390be433cf292ee67affcebbc2ea591ee09ffddb1bc5d3`,
`a12398685131398da0a9a3a0200d1b7e988be6d1e12ea7dc6a9fe9019eab7bb4`,
and
`9aa975ae84b5bed868095e19969b5db18f59a113d96963f21656f2358fb87326`.
The resulting 63-line `source_ast.rs` has hash
`e785097028171a78e3f8764618ac4bced422756b4c1a985e72de3138ae46a1ed`;
the 6-line facade has hash
`a5d786f3fce6b7d6b5661918e4fb46a3116b41f33fe307adebed4ddefe2e3efa`;
and the 16,240-line `runner.rs` has hash
`01990093ec8ac5b2360bf174e8b1d13b21550f599c3b51ab3fd0e02725762bd9`.
Rust does not permit a child `pub(super)` item to be re-exported to its
grandparent, so the leaf functions use the explicit runner-subtree-only
`pub(in crate::runner)` scope while the private phase facade re-exports them
with `pub(super)`. Private `runner.rs` aliases preserve every production and
test caller. Function bodies, traversal and filtering order, recovery
recursion, typed-site identity, exact-shape rejection, and fail-closed behavior
are unchanged. Task 262A is complete; parent Task 262 remains open for the
remaining bounded source-extraction families.

## Current Ownership

| Current area | Responsibility | Dependency direction | Audit decision |
|---|---|---|---|
| public report/result/status types and `run_*_corpus` functions | Stable public runner facade and corpus-level orchestration | plan/discovery to phase execution | Keep in `runner.rs`. |
| source/frontend and resolver staging | Source package preparation and cleanup, root/path/snapshot identity, frontend execution/result transport, and resolver shell/projection/symbol collection | shared by parse, declaration-symbol, and type-elaboration as applicable | Frontend staging moved in Task 258 and the declaration/type resolver leaf in Task 260A to private `shared.rs` with minimal parent-only visibility. |
| active-case admission and stable failure assembly | Tag/phase gates, expected-output matching, and deterministic failure diagnostics | phase-specific facade-to-owner transition | Tasks 259 and 260B moved parse-only and declaration case/failure boundaries; keep the remaining type boundary in `runner.rs` until Tasks 262-263 move it. |
| parse-only execution | Surface-AST snapshots and parse-only failure projection | shared frontend to parse-only result | Moved in Task 259 to private `parse_only.rs` with minimal parent-only visibility. |
| fixture import provider | Parser fixture lexical summaries and type import-summary adapters | parser/frontend seams shared by active phases | Moved in Task 261 to private `import_fixtures.rs`; later phases retain the same provider and adapter paths. |
| declaration-symbol observation | Consume the shared resolver result and assemble deterministic payload, expected-value, and failure projections | shared resolver output to declaration-symbol result | Moved in Task 260B to private `declaration_symbol.rs`; existing integration tests remain in `tests/metadata.rs`. |
| type-elaboration admission/execution | Lower-stage fail-closed gates and checker/core handoff dispatch | resolver output to source bridge | Remains in `runner.rs` through Tasks 262-263. The current `type_elaboration.rs` is only the initial private phase facade for `source_ast`; later moves make it the orchestration owner. |
| source extraction | Exact source-shape recognition and real AST/resolver payload construction | syntax/resolver inputs to checker inputs | Task 262A moved the common source-AST primitives to private `type_elaboration/source_ast.rs`; remaining extractor families stay in `runner.rs` until later Task 262 subtasks. |
| payload validation and detail-key rendering | Exact checker/core output validation, expected/actual matching, deterministic keys, diagnostics | source bridge output to runner result | Private type-elaboration leaf owner; no key or ordering edits. |
| fixture builders and corruption probes | AST/env/sidecar builders and finite negative matrices | test support to private production seams | Private test support/fragments only. |
| cross-owner isolation tests | Bidirectional route rejection and immutable/module guards | all supported source-bridge owners | Keep intact and move as a cohesive fragment. |

## Dependency Map

The permitted dependency direction is:

```text
public runner facade
  -> parse-only owner
     -> shared plan/admission/source/frontend staging
        -> fixture/import-summary owner (lexical provider)
  -> declaration-symbol owner
     -> shared plan/admission/source/frontend/resolver staging
        -> fixture/import-summary owner (lexical provider)
  -> type-elaboration owner
     -> shared plan/admission/source/frontend/resolver staging
        -> fixture/import-summary owner (lexical provider)
     -> fixture/import-summary owner (resolver adapter)
     -> source extraction
        -> common source-AST primitives
     -> checker/core payload validation
     -> deterministic detail keys and failure diagnostics

private runner::tests
  -> shared test support and fixture builders
  -> the same private phase seams
```

Leaf helpers move before their callers. Phase modules may depend on shared
staging, but parse-only and declaration-symbol must not depend on checker/core
payload validation. Metadata `plan` remains payload-free.

## Target Source Layout

The exact leaf split may be made smaller when fresh inventory proves a family
is still too large, but no empty or synthetic owner module is permitted.

| Target path | Ownership |
|---|---|
| `src/runner.rs` | Public facade, public report/result/status types, public active-case iterators, and top-level corpus orchestration only. |
| `src/runner/shared.rs` | Private source package preparation, frontend execution, admission support, and genuinely cross-phase helpers. |
| `src/runner/parse_only.rs` | Parse-only case execution, snapshots, and parse-only failure projection. |
| `src/runner/declaration_symbol.rs` | Declaration-symbol case execution, resolver observation, payload keys, and failure projection. |
| `src/runner/import_fixtures.rs` | Existing parser fixture summaries/adapters used by active phases. |
| `src/runner/type_elaboration.rs` and `src/runner/type_elaboration/` | Type-elaboration orchestration plus private source-extraction and payload-validation/detail/diagnostic leaves. |
| `src/runner/tests.rs` | The single private `runner::tests` module and root-level `include!` declarations. |
| `src/runner/tests/support.rs` | Shared test imports, builders, environments, ids, and corruption helpers. |
| `src/runner/tests/parse_only.rs` | The nonempty parse-only private test family. |
| `src/runner/tests/type_elaboration/*.rs` | Nonempty cohesive source-extraction, reserved/binary, mode-chain, asserted-head, long-chain, and isolation families. |
| `tests/metadata.rs` | Existing declaration-symbol integration-test owner; unchanged unless a later independent nonempty move is justified. |

Test fragments are included directly at the root of `runner::tests`, without a
new wrapper module. This preserves existing qualified test names, including the
already nested Task 216-222 module names. A child-module split is forbidden
when it would change the discovered test list.

Fresh Task 253 inventory splits the reserved/binary owner into two physical,
order-preserving fragments around the Task 254 mode-chain block. Task 253A is
the leading 23-test baseline reserve/binary block. It retains Task 189's
reserved-object type-assertion test because that test is embedded in and owns
the baseline reserved-object bridge boundary; it also retains Task 246's
parenthesized two-edge local-mode equality because that test belongs to the
parenthesized binary route. These classifications do not transfer either test
to the broader asserted-head or mode-chain families. Task 253 remains pending
after 253A: Task 254 moves the intervening local-mode/object-mode chain block,
then Task 253B moves the following direct reserved-variable membership and
inequality tests. Task 255 begins with the direct reserved-variable
type-assertion test. This sequence preserves source and discovery order.

Fresh Task 254 inventory fixes that intervening block at 26 complete tests:
the non-long-chain direct-through-four-edge set/object membership, equality,
and inequality families. It contains no long-chain test or non-test helper/item.
The separator after the block is retained, and Task 253B begins immediately
after it. Task 254 moves only this contiguous block to
`src/runner/tests/type_elaboration/mode_chain.rs`.

Fresh Task 253B inventory fixes the next block at two complete direct
reserved-variable tests: membership and inequality. It contains no non-test
helper/item. The following separator is retained, and Task 255 begins with the
direct reserved-variable type-assertion test immediately after it. Task 253B
moves only this 284-line block to
`src/runner/tests/type_elaboration/reserved_direct.rs`; completing that move
completes parent Task 253.

Fresh Task 255 inventory splits the non-long-chain type-assertion/asserted-head
source family into five physical, order-preserving blocks. Task 255A contains
6,653 lines: 12 leading source tests through the three-edge set-side radix
owner plus the two dedicated Task 205 isolation helpers that only those tests
consume. Task 255B contains 3,303 lines: the two four-edge set/object radix
tests plus their dedicated Task 208/207 helpers. Task 255C and Task 255D keep
the single three-edge and two-edge object-radix source tests with their
dedicated Task 206 and Task 204 helpers, respectively. Task 255E contains the
final 16 contiguous source tests and stops before the active-fixture block.
Each separator between these blocks remains in `tests.rs`. These dedicated
helper items move with their owner tests; standalone active-fixture and
cross-owner-isolation tests remain for later fresh inventory. No Task 255
subtask includes a long-chain test. Parent Task 255 remains pending through
Task 255E.

## Ordered Move Tasks

| Task | Bounded action |
|---|---|
| 248 | Add this paired audit, update the paired crate plan, and establish the preservation matrix. No source move. |
| 249 | Complete: mechanically moved the complete inline private `mod tests` body to `src/runner/tests.rs`. |
| 250 | Complete: moved nonempty shared test support into a root-included support fragment. |
| 251 | Complete: moved the nonempty parse-only private test family into a root-included fragment. |
| 252 | Complete: moved the baseline type-elaboration source-extraction and real handoff tests. |
| 253A | Complete: moved the leading 23-test baseline reserved-variable/binary-formula block; Task 253 remains pending. |
| 254 | Complete: moved the 26-test non-long-chain local-mode/object-mode chain bridge block, retaining the following Task 253B boundary. |
| 253B | Complete: moved the two direct reserved-variable membership and inequality tests to `reserved_direct.rs`, retained the following Task 255 boundary, and completed Task 253. |
| 255A | Complete: moved the leading 12 type-assertion/asserted-head source tests and their two dedicated Task 205 helpers to `asserted_head_base.rs`. |
| 255B | Complete: moved the two four-edge radix source tests and dedicated Task 208/207 helpers to `asserted_head_four_edge_radix.rs`. |
| 255C | Complete: moved the three-edge object-radix source test and dedicated Task 206 helpers. |
| 255D | Complete: moved the two-edge object-radix source test and dedicated Task 204 helper. |
| 255E | Complete: moved the final 16 non-long-chain source tests, retained the active-fixture boundary, and completed Task 255. |
| 256 | Complete: moved all 44 long-chain source/active bridge tests and their 12 test-local finite guards to `long_chain.rs`, retaining both adjacent four-edge boundaries. |
| 257 | Complete: moved all eight inventoried remaining fixture, bridge-gap, corruption, and isolation families through Task 257H. |
| 257A | Complete: moved the leading 18 binary/parenthesized fixture and route-isolation tests to `binary_route_fixtures.rs`, retaining the Task 257B separator. |
| 257B | Complete: moved the three builtin-object reserve active fixtures to `reserve_object_fixtures.rs`, retaining the Task 257C separator. |
| 257C | Complete: moved only the Task 180 standalone contradiction fixture to `formula_constant_fixture.rs`, retaining both reserve-family boundaries. |
| 257D | Complete: moved the 11 distinct/multiple/heterogeneous reserve fixtures to `reserve_fixtures.rs`, retaining the Task 257E separator. |
| 257E | Complete: moved the 26 non-long-chain active mode-chain fixture tests to `mode_chain_fixtures.rs`, retaining the Task 257F separator. |
| 257F | Complete: moved the 35 active reserve/asserted-head/type-assertion fixtures plus four interleaved owner-route isolation guards to `asserted_head_fixtures.rs`, retaining the Task 257G separator. |
| 257G | Complete: moved the three source-gap/four-edge-equality tests to `source_gap_and_equality.rs`, retaining the immediate long-chain include and Task 257H boundary. |
| 257H | Complete: moved the final nine root bridge fixtures, three root isolation tests, and 28 nested tests to `remaining_bridges_and_nested_isolation.rs` while retaining Task 216-222 modules; completed Task 257. |
| 258 | Complete: moved shared source/frontend staging helpers to private `shared.rs` after the test layout stabilized. |
| 259 | Complete: moved parse-only case execution, Surface-AST snapshot comparison, and failure projection to private `parse_only.rs`. |
| 260A | Complete: moved the cross-phase resolver shell/projection/symbol collection leaf to private `shared.rs` before its declaration and type callers. |
| 260B | Complete: moved existing declaration-symbol case/observation/payload/expectation/failure helpers to private `declaration_symbol.rs`; integration tests stayed in place. |
| 261 | Complete: moved the lexical provider, exact fixture vocabulary, and type import-summary adapter to private `import_fixtures.rs`. |
| 262 | Parent: move type-elaboration source-extraction leaves; remains open after Task 262A. |
| 262A | Complete: moved the five common exact source-AST primitives behind the private type-elaboration phase facade. |
| 263 | Move payload validation, detail-key, expected-output, and failure-diagnostic leaves. |
| 264 | Close out paired source-layout inventories, path tables, todo/plan state, and ownership guards. |

Every listed source-moving task must be nonempty. If fresh inventory requires a
smaller family, add a bounded subtask before editing; never create a no-op
commit.

## Preservation Matrix

| Surface | Required invariant |
|---|---|
| public API | `mizar_test::runner` re-exports, signatures, enum attributes, and CLI behavior are unchanged. |
| tests | Function names, fully qualified discovered names, nested module names, discovery order/set, and all 272 tests are unchanged. |
| corpus/trace | Active runner 188, plan 403/367, type 235/223, pass/fail 219/184, backlinks, requirements, and expectation meaning are unchanged. |
| diagnostics | Codes, stable detail keys, fallback keys, text, source identity, and ordering are byte-for-byte unchanged. |
| payloads | Keys, values, shapes, provenance, source ranges, binding identities, deterministic ordering, and immutable outputs are unchanged. |
| fail-closed behavior | Unsupported, malformed, ambiguous, imported-gap, evidence-gap, and lower-stage cases continue to reject at the same boundary. |
| authority | No `doc/spec`, `.miz`, expectation, or traceability edit is allowed merely to accommodate a move. |

Before and after each move, capture and compare the exact sorted test lines
from `cargo test -p mizar-test --lib -- --list` in addition to running the
tests. The canonical raw-list oracle, including the `: test` suffix, has 272
lines and hash
`5e41e4dbfcc303322c246a612de61926a628957a168589b45864d0a5070bb07e`.
The suffix-stripped normalized-name list is only a secondary oracle; its hash
before Task 253A is
`c0c2b80f8b4e6c84cd25d77573fda722c4d1846fed168cd4a478781cdb42775e`.

## Classification And Coverage-Audit Impact

| Class | Result |
|---|---|
| `design_drift` | Active: source layout obscures phase and ownership review boundaries. Tasks 249-264 repair it without changing behavior. |
| `spec_gap`, `test_gap`, `source_drift`, `test_expectation_drift` | None introduced or repaired by this series. |
| `source_undocumented_behavior`, `boundary_violation` | No new finding; existing runner behavior remains governed by the paired harness plan and higher authorities. |
| `repo_metadata_conflict` | None found. |

`doc/design/spec_coverage_audit.md` remains unchanged. The series changes no
specification chapter coverage, design mapping, traceability status, owner
crate, follow-up ownership, or deferred rationale.

## Per-Task Review And Verification

Each source move requires review-only checks for visibility drift,
test-discovery drift, owner-boundary drift, source/documentation inconsistency,
and accidental behavior change. Required commands are:

```text
cargo test -p mizar-test
cargo run -q -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml
cargo run -q -p mizar-test -- parse-only --tests-root tests --manifest tests/coverage/spec_trace.toml
cargo run -q -p mizar-test -- declaration-symbol --tests-root tests --manifest tests/coverage/spec_trace.toml
cargo run -q -p mizar-test -- type-elaboration --tests-root tests --manifest tests/coverage/spec_trace.toml
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
git diff --check
```

The active CLI preservation counts are parse-only 96, declaration-symbol 4,
and type-elaboration 188.

## Exit Criteria

The series is complete only when `runner.rs` is limited to the public facade
and top-level orchestration; every private owner has minimal visibility; the
preservation matrix passes; paired source-layout, crate-plan, todo, harness
path-table, bilingual, and ownership-guard documentation is synchronized; and
all verification is green. Fresh Step 5 inventory resumes only after Task 264.
Steps 6 and 7 remain deferred until their existing dependency gates are met.
