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

## Task 262B Move Result

Fresh inventory after Task 262A found two additional AST-only projections
shared by the formula and reserve extractors. Task 262B moved the preorder
node-kind collector, including its private recursive helper, and the exact
qualified-symbol spelling projection into `source_ast.rs`. Their original
fragment hashes were
`e06bf8e9c5252a3bfefea3ff16804414fe813cbc903cd0afcdfd0d237a1185c5`
and
`8ab94eafe97e9b28c7a236efd3071834b36ba02a2fce1988d721123f19272f7e`.
The resulting 113-line `source_ast.rs` has hash
`d9bff4c1c4bbeb2bd988502db2bff2a7370dbf9b61d7e817e6e82077878da78a`;
the 6-line facade has hash
`f89edc2b2dcd2065c9445aae9a7b05084750d7689f8a2029c4e78050a7c797c9`;
and the 16,193-line `runner.rs` has hash
`fea5c857a73a6f7429c2517b187b354fa321c0e6be14ffd64922eb10d57c42d0`.
The two caller-facing functions retain runner-subtree-only
`pub(in crate::runner)` visibility through parent-only facade re-exports; the
recursive collector remains private to `source_ast.rs`. Traversal order,
token/path validation, spelling assembly, every caller, and every test import
are unchanged. Visible-symbol resolution, source-text assembly, range merging,
and reserve/mode extraction remain in `runner.rs` for Task 262C. Task 262B is
complete; parent Task 262 remains open.

## Task 262C Move Result

Fresh inventory after Task 262B isolated the reserve type-expression and
symbol-projection family from declaration segmentation and mode expansion.
Task 262C created the private
`src/runner/type_elaboration/source_reserve.rs` leaf and moved the
`SourceTypeExpression` transport, exact builtin/symbol type-expression
projection, visible attribute/type-head resolution, local/imported-fixture
admission, and recursive source-text assembly. The original 8-line transport
fragment had hash
`6b95aec82269efe807537832e551e0bac37480cb653ad02cd3492e7ccd304afe`;
the original 266-line type-expression/resolution fragment had hash
`918d2e22b0c18555cc0bffe1c2721f1563bc22427e7902959e7b7dcb56328f0e`;
and the original 89-line provenance/source-text fragment had hash
`8b06c6b116d5f420a40a645a274516051052a56908a2974c3e25efa43af80e2a`.
The resulting 370-line `source_reserve.rs`, including direct dependency
imports, has hash
`16b9a05842b3db5c22468d9674526bd7efc6739572d933ebf57e6ba0b69e34fb`;
the 14-line facade has hash
`e768f927bbf7263a7930f2ae73dcc8787b4df29f019f9f81ed50ec799f5d1f9d`;
and the 15,834-line `runner.rs` has hash
`0574cd3bbdbf4df09c02a2a9be07af07b9732c5dc1d5036feb9919641c3a6007`.
The transport, its four fields, and the extraction entry retain
runner-subtree-only visibility through parent-only facade re-exports. The two
visible resolver entries use test-only facade and runner aliases because only
the existing private preservation tests call them from outside the leaf.
Three helpers still consumed by the retained Task 262E
declaration/mode callers—`source_reserve_symbol_head_kind`,
`is_imported_fixture_reserve_attribute`, and
`imported_fixture_reserve_attribute_spelling`—use the same temporary scope;
Task 262E must narrow them when those callers move. All other type-head,
attribute, admission, and source-text helpers remain leaf-private. Exact AST
shape and recovery rejection, local-before-imported ambiguity handling,
symbol kind/provenance admission, attribute polarity/order, spelling/range,
and fail-closed behavior are unchanged. Formula-only resolution, range merge,
reserve declaration segmentation, and local-mode traversal/expansion remain
in `runner.rs` for Task 262E and later bounded inventory. Task 262C is complete;
parent Task 262 remains open.

## Task 262D Move Result

Fresh dependency inventory found that the exact `parser.type_fixtures`
import-item recognizer was shared by two formula callers and the retained
reserve caller. Task 262D moved this common source-AST prerequisite to
`src/runner/type_elaboration/source_ast.rs` before moving the reserve family.
The original 28-line fragment had hash
`d137915a4bac8d6922ea86d34975b07004b4cef389a5ea9d008fb955f3f83bdc`.
The resulting 147-line `source_ast.rs` has hash
`baf131e5f82846df2286ad68c6e8bad9d2642af2ce530f7b8c7362900ef2aa9e`;
the 15-line facade has hash
`10db1015db9d0a653f511ffaa5a48a2a708b9c1b3d254a194894f44430ff384f`;
and the 15,803-line `runner.rs` has hash
`4dfc36f6f8f204b705688c5762d42281be949ce7c7eae2751e12d1aeb84c13d6`.
The unchanged 370-line `source_reserve.rs` retains hash
`16b9a05842b3db5c22468d9674526bd7efc6739572d933ebf57e6ba0b69e34fb`.
`source_ast` directly consumes `import_fixtures::module_path_spelling`, which
stays in the fixture owner with runner-subtree visibility; no child-to-parent
runner helper dependency remains. The recognizer uses runner-subtree-only
`pub(in crate::runner)` visibility, a parent-only facade re-export, and a
private `runner.rs` alias for all three callers. Apart from that visibility and
rustfmt signature wrapping, its body is unchanged. Import-item/alias/path
cardinality, direct-token filtering, recovery rejection, exact module spelling,
caller order, and fail-closed behavior are unchanged. Task 262D is complete;
parent Task 262 remains open and Task 262E is next.

## Task 262E Pre-Move Inventory and Specification

Fresh inventory classifies the remaining reserve-family placement in
`runner.rs` as `design_drift`: the exact source-derived declaration and
local-mode payload producer is cohesive with the existing private
`type_elaboration/source_reserve.rs` owner, while its current parent placement
does not express that ownership. No language, test, expectation, trace, or
metadata disagreement was found. The move consists of the contiguous 1,074-line
fragment from `SourceReserveExtraction` through
`extract_builtin_reserve_segment`, whose hash is
`31f8e27a1835ea31e6d65ff67acbfa8fcc040fc588df7f24453ff848e0bd690b`,
plus the separate 10-line `merge_optional_range` helper, whose hash is
`aa186a9105816e62352473111ffe3b9958a332086e9d1fc459c024fbc2cfac5c`.

The move preserves one runner-subtree transport boundary:
`SourceReserveExtraction` and its `bridge`/`mode_expansions` fields, including
the existing test-only accessors. It also preserves four runner-subtree helper
boundaries: `extract_builtin_source_reserve_declarations`,
`extract_builtin_source_reserve_declarations_after_node_guard`,
`source_mode_symbol_spelling`, and `mode_definition_pattern_spelling`. The
private phase facade re-exports only those five boundary items to its parent,
and `runner.rs` keeps private aliases for unchanged callers. All other moved
items become or remain leaf-private. The three Task 262C temporary helpers
`source_reserve_symbol_head_kind`,
`is_imported_fixture_reserve_attribute`, and
`imported_fixture_reserve_attribute_spelling` become leaf-private once their
remaining callers move.

`source_reserve` will consume the common source-AST projections directly,
including the Task 262D fixture-import recognizer, and will continue to consume
its own Task 262C type-expression/symbol projections without a child-to-parent
dependency. `SourceReserveHandoff`, `source_module_binding_env`, formula-only
imported term/formula resolution, checker handoff/validation, and later
orchestration stay in `runner.rs`. The preservation matrix is exact AST/import
shape, node allowlist, recovery rejection, traversal budget/order, dependency
and provenance admission, attribute polarity/order, spelling/range, payload
contents, diagnostics/detail keys, and fail-closed behavior. This task changes
no test body or name, public API, spec/trace/expectation artifact, harness count,
or specification-coverage credit.

## Task 262E Move Result

Task 262E moved both inventoried fragments into the existing private
`src/runner/type_elaboration/source_reserve.rs` owner. After rustfmt and the
minimal import/visibility adjustments, `runner.rs` is 14,718 lines with hash
`f38352151d71474b676fb3c2a50e313c33f6de6dad5a09097c28aa9de729ce62`;
the 16-line phase facade has hash
`07c19a11381d002cd3a6503470df6e1e63d09a2b435350608b1cc8fe1724a50a`;
and the 1,474-line `source_reserve.rs` has hash
`88cf0cf08de2e61b2e6342aacc36ee01e20e00606d0c51f4bf7b5c64495253db`.
The unchanged 147-line `source_ast.rs` retains hash
`baf131e5f82846df2286ad68c6e8bad9d2642af2ce530f7b8c7362900ef2aa9e`.

The extraction transport, its two fields/test-only accessors, the two
extraction entries, and the two spelling helpers use runner-subtree visibility,
parent-only facade re-exports, and private `runner.rs` aliases. The three
temporary Task 262C helpers are now leaf-private. `source_reserve` consumes the
common source-AST primitives and fixture-import recognizer directly; no
child-to-parent source dependency was introduced. `SourceReserveHandoff`, the
module binding environment, formula-only imported symbol resolver, checker
handoff/validation, and orchestration remain in `runner.rs`.

The moved declaration/import gates, node allowlist, recovery checks, traversal
budget, dependency ordering, expansion provenance, segment/range assembly, and
fail-closed branches are unchanged apart from required visibility and rustfmt
wrapping. All 272 unit tests pass, and the sorted raw and normalized test-list
hashes remain `5e41e4dbfcc303322c246a612de61926a628957a168589b45864d0a5070bb07e`
and `c0c2b80f8b4e6c84cd25d77573fda722c4d1846fed168cd4a478781cdb42775e`.
Task 262E is complete. Parent Task 262 remains open for fresh inventory of the
remaining formula-extraction families; no `spec_coverage_audit.md` update is
required because behavior, tests, trace credit, and owner crate are unchanged.

## Task 262F Pre-Move Inventory and Specification

Fresh inventory classifies the first remaining formula-family placement as the
same `design_drift`: the exact standalone formula-constant AST projection is a
cohesive private source-extraction leaf, while its transport, extraction, and
node policy are split across `runner.rs`. Task 262F creates private
`src/runner/type_elaboration/source_formula.rs` and moves only three fragments:
the 6-line `SourceFormulaStatement` transport with hash
`8ab3f277e5a8e0dabe1caacf76e5f54d81804c3619209bf94ac88ed01ebbc5e7`,
the 84-line `thesis`/`contradiction` entry and common exact extractor family
with hash
`eb1927127ca995ad3e9f090cb04aaf2b0326aac240b58dcbc14cfb731666061c`,
and the 12-line dedicated theorem-node allowlist with hash
`acc01a4adb0ee02529a2fce8d8f0772c944f1b606f108bbde4e4096cc143c840`.

The transport and its two fields use runner-subtree-only visibility so callers
can consume the inferred return payload without a type alias. The two
extraction entries use runner-subtree-only visibility, parent-only facade
re-exports, and private `runner.rs` aliases. The common exact extractor and node
allowlist remain leaf-private. The new leaf directly consumes the existing common source-AST
token, structural-child, recovery, node-kind, and site projections. The real
consumers—`source_formula_statement_output`,
`source_contradiction_formula_output`, their detail-key paths, the module
binding environment, and checker inference—remain in `runner.rs` with unchanged
caller bodies.

This is a move-only task. It preserves exact theorem label/token/cardinality,
formula-expression/constant kind and spelling, recovery and node rejection,
real AST site/range, `FormulaKind`, deferred reason, checker output, diagnostic
and detail-key ordering, and fail-closed behavior. Reserved-variable/binary,
builtin/imported formula, set-enumeration, connective/quantifier extraction,
formula semantics, child graphs, theorem acceptance, facts, proof, CoreIr,
ControlFlowIr, and VC are excluded. Existing exact, near-miss, corruption,
active-fixture, detail-key, and route-isolation tests are the preservation
matrix; no spec/test/trace/expectation or specification-coverage credit changes.

## Task 262F Move Result

Task 262F created the private
`src/runner/type_elaboration/source_formula.rs` leaf and moved all three
inventoried fragments. After rustfmt and minimal visibility/import adjustment,
`runner.rs` is 14,615 lines with hash
`b0d19f08a642b8b29e0f6c74e063b35909c3a9fbac30f9c1ee713de9fefa57f2`;
the 20-line phase facade has hash
`59f458f5336f60be419c9d8e86b4a2dbed8f01dcc7ddc087cc437a25e72f3e7a`;
and the new 116-line leaf has hash
`d13b2ca47ad8c1580f38f363fac79881b304bcc5425e557ec7bdc6bd7a8264c2`.
The unchanged 147-line `source_ast.rs` and 1,474-line `source_reserve.rs`
retain hashes
`baf131e5f82846df2286ad68c6e8bad9d2642af2ce530f7b8c7362900ef2aa9e`
and `88cf0cf08de2e61b2e6342aacc36ee01e20e00606d0c51f4bf7b5c64495253db`.

The transport and fields remain runner-subtree-visible only through the two
entry return types; the compiler-confirmed minimal facade/private alias surface
contains the two entries only. The common exact extractor and node allowlist
are leaf-private, and the leaf depends directly on `source_ast`. Both checker
consumer bodies, their detail-key paths, and the module binding environment
remain in `runner.rs`. Apart from required visibility, imports, and rustfmt
wrapping, the three moved bodies are unchanged.

All 272 unit tests pass. The sorted raw and normalized test-list hashes remain
`5e41e4dbfcc303322c246a612de61926a628957a168589b45864d0a5070bb07e`
and `c0c2b80f8b4e6c84cd25d77573fda722c4d1846fed168cd4a478781cdb42775e`.
Task 262F is complete; parent Task 262 remains open for fresh inventory of the
remaining reserved/binary, builtin/imported, enumeration, and
connective/quantifier formula families. No `spec_coverage_audit.md` update is
required because behavior, tests, trace credit, and owner crate are unchanged.

## Task 262G Pre-Move Inventory and Specification

Fresh inventory classifies the shared exact numeral projection as the same
`design_drift`. Its three AST-only helpers are formula-source policy but remain
in `runner.rs`. The 47-line contiguous fragment has hash
`b415692ed2ee250be1bd4b66bfe90d21cc5cb444124eb249cca8890d1d488631`.
`exact_numeral_term_operand` has seven retained call sites across the builtin
binary, builtin type-assertion, imported predicate/functor, imported attribute,
and set-enumeration extractors. `exact_numeral_term_node_or_expression` has two
retained call sites in the imported infix-functor projection, and the common
`exact_numeral_term_node` recognizer is called only by the other two helpers.
The connective/quantifier and standalone constant families do not consume this
prerequisite.

Task 262G moves only that fragment into the existing private
`src/runner/type_elaboration/source_formula.rs` leaf. The operand and
node-or-expression entries receive runner-subtree-only visibility, parent-only
facade re-exports, and private `runner.rs` aliases; the numeral-node recognizer
remains leaf-private. The leaf continues to consume `SurfaceAst`,
`SurfaceNodeId`, `SurfaceNodeKind`, `SourceRange`, and the existing common
source-AST token, structural-child, and recovery projections. All five caller
families, their transports/configuration, resolver use, node allowlists,
checker consumers, detail keys, diagnostics, and failure assembly remain in
`runner.rs` with unchanged bodies.

This is a move-only prerequisite. It preserves exact `TermExpression` wrapper
and single-child cardinality, recovery rejection, `NumeralTerm` kind, direct
token spelling (`1`/`2` as requested by each caller), empty structural-child
requirement, returned real node identity/range, caller order, and fail-closed
behavior. The existing spec-derived `.miz` sources, trace/expectation rows,
exact/near-miss/corruption matrices, active fixtures, and bidirectional route
isolation form the preservation matrix. No test-first addition is appropriate,
and no spec, test, trace, expectation, API, public surface, or
`spec_coverage_audit.md` change is permitted. Formula-family moves, helper
renaming/deduplication/generalization, semantic payload changes, theorem
acceptance, facts, proof, CoreIr, ControlFlowIr, and VC are excluded.

## Task 262G Move Result

Task 262G moved the inventoried three-helper fragment into the existing private
`source_formula.rs` leaf. After stripping the required visibility qualifiers,
the moved 47 lines retain the original hash
`b415692ed2ee250be1bd4b66bfe90d21cc5cb444124eb249cca8890d1d488631`.
`runner.rs` is now 14,569 lines with hash
`f3858539557d392e1d85fcf98bbfac615ef2564c1b3b9475c522994e7a6d94d4`;
the 21-line phase facade has hash
`702a81c671cc435d8dd1c1c4e1444070823372340308e319eeaf8790a0fcb8db`;
and the 165-line source-formula leaf has hash
`ffbb81c4b76339f26c23423785e1139260d92426b6b56fc9295c0065635ab3f6`.
The unchanged 147-line source-AST and 1,474-line source-reserve leaves retain
hashes `baf131e5f82846df2286ad68c6e8bad9d2642af2ce530f7b8c7362900ef2aa9e`
and `88cf0cf08de2e61b2e6342aacc36ee01e20e00606d0c51f4bf7b5c64495253db`.

The exact numeral-node recognizer is leaf-private. Only the operand and
node-or-expression entries cross the parent-only facade, and all seven plus
two retained caller sites remain in `runner.rs` in their original order. The
caller bodies, transports/configuration, resolver dependencies, node
allowlists, checker consumers, detail keys, diagnostics, and failure assembly
are unchanged. Focused preservation and all 188 active type cases pass. All
272 unit tests and the relevant-crate suite pass; the sorted raw and normalized
test-list hashes remain
`5e41e4dbfcc303322c246a612de61926a628957a168589b45864d0a5070bb07e`
and `c0c2b80f8b4e6c84cd25d77573fda722c4d1846fed168cd4a478781cdb42775e`.

Task 262G is complete. Parent Task 262 remains open for fresh bounded inventory
of the remaining reserved/binary, builtin/imported, enumeration, and
connective/quantifier formula families. No `spec_coverage_audit.md` update is
required because behavior, tests, trace credit, and owner crate are unchanged.

## Task 262H Pre-Move Inventory and Specification

Fresh inventory classifies the builtin equality/inequality/membership formula
family as the same `design_drift`. Three cohesive fragments remain separated in
`runner.rs`: the 43-line config and source transport with hash
`cd7bf9a595ba8d6b73c1cafa567da306092c1953e92e9695c3bf67c5c653336d`,
the 84-line exact extractor with hash
`ce691c4917fc00c8b4fe0799f02f8e252e4cf005d3a3a1082ae01c8c0e35bc3c`,
and the 17-line dedicated node allowlist with hash
`979560644d3d5827e2abbb016d2b5ea5da22a21cf71f3c35feca89404f3b29d8`.
The three configs map only `TermFormulaPayloadBoundary`,
`BuiltinInequalityPayloadBoundary`, and `BuiltinMembershipPayloadBoundary` to
their exact operator, numeral spellings, and `FormulaKind`. The production
extractor has one caller in `source_builtin_binary_term_formula_detail_keys`.
The private preservation matrix also reads the config constant and its label,
left, operator, and right fields to reject status-prefixed near misses.

Task 262H moves only those three fragments into the existing private
`source_formula.rs` leaf. The source transport and its fields, extraction entry,
config type, four test-consumed config fields, and config constant receive
runner-subtree-only visibility. The config's `formula_kind` field, extractor
implementation, and node allowlist remain leaf-private where possible. The
facade exposes the extraction entry unconditionally, while the config constant
uses `#[cfg(test)]` on both its facade re-export and `runner.rs` import. The
inferred transport/config types require no parent aliases. The leaf imports
`FormulaKind` directly and consumes its existing local exact-numeral helper plus
the common source-AST token, structural-child, recovery, node-kind, and site
projections. The production checker/detail consumer and private test code stay
byte-for-byte in `runner.rs` and its test subtree.

Task 262H remains a move-only task. It preserves the three-entry config order and values,
exact theorem labels/tokens/cardinality, single formula-expression and two-term
shape, operator and numeral spelling, recovery/node rejection, real AST
site/range, formula kind, checker payload/detail-key ordering, and fail-closed
behavior. The canonical `.miz` sources and their spec/trace/expectation intent,
the Task 262H0-strengthened exact/near-miss/corruption matrix, all three active
cases, and route isolation are the preservation matrix. Task 262H itself adds
no tests and makes no spec, trace, expectation, public API, or
`spec_coverage_audit.md` change. Config edits, renaming/deduplication/generalization,
reserved-variable formulas, builtin type assertion,
imported/enumeration/connective formulas, checker/detail movement, semantic
checking, facts, theorem acceptance, proof, CoreIr, ControlFlowIr, and VC are
excluded.

## Task 262H0 Test-Gap Inventory and Specification

The Task 262H test-sufficiency review classified one independent `test_gap`
before the production move. Existing tests prove the three active sources reach
the expected fail-closed detail keys and reject status, wrong-label, wrong
operator, wrong-right-numeral, and extra-root cases. They do not directly assert
the extracted formula/left/right sites and ranges, each config's resulting
`FormulaKind`, or three-entry config order. They also omit wrong-left-numeral,
recovered theorem/formula, duplicate theorem, duplicate formula-expression,
and extra operand cardinality rejection. A move could therefore corrupt source
provenance or config mapping while retaining the same final detail keys.

Task 262H0 is a test-only repair before Task 262H. Extend the existing
`source_reserve_bridge_reports_gap_or_evidence_detail_for_unsupported_shapes`
test rather than adding a new test: enumerate the three configs in canonical
order, assert their exact label/left/operator/right values, extract each exact
payload, and compare `FormulaKind`, formula/operand `TypedSiteRef`, and hard
source-derived ranges. Add bounded synthetic builder states for recovered label
and operator tokens, duplicate theorem, duplicate formula expression, and extra
term expression, plus a wrong-left case for every config; every case must fail
at the existing payload-extraction boundary. Test support may add only the
minimal corruption flags/builders needed by this matrix.

No production source, `.miz`, expectation, trace, spec, public API, diagnostic,
payload behavior, test name, or test count changes in Task 262H0. Existing
behavior is the test subject, not new intent; assertions may only strengthen.
The focused test, all 272 unit tests and unchanged list hashes, all 188 active
type cases, relevant-crate tests, workspace fmt/Clippy/tests, and diff check are
required. This test-only task commits separately before the move-only Task
262H. No `spec_coverage_audit.md` update is required because coverage credit,
owner crate, and authority artifacts remain unchanged.

## Task 262H0 Test Repair Result

Task 262H0 strengthened the existing test without adding or renaming any test.
The canonical config loop now fixes all three entries in order, their exact
label/left/operator/right values, resulting `FormulaKind`, and independently
computed formula/operand ranges and matching real AST sites. Every config also
has a wrong-left rejection. Five bounded corruptions cover recovered theorem
label, recovered formula operator, duplicate theorem, duplicate formula
expression, and extra operand cardinality. The synthetic duplicate/extra nodes
use independently owned allowlisted node kinds, so each case reaches the
intended extractor cardinality branch rather than an earlier node-policy guard.

The default and status-bearing builders preserve their prior exact token kinds,
text, child order, ranges, offsets, and root construction. Only the private test
subtree imports the production extractor. `support.rs` is now 6,655 lines with
hash `5db1b0dc66f8149050d04f3f487c7e9efb201b990e871e8766cafbfca77b7d97`;
`source_gap_and_equality.rs` is 3,067 lines with hash
`0178a217c935d42d4f229a30e3875989ac1aa9ae6bcd56057e931b7b05d7660a`.
Production source, `.miz`, spec, trace, expectation, public API, diagnostics,
payload behavior, test names, and test count are unchanged.

The focused test and all 272 unit tests pass, all 188 active type cases pass,
and the raw/normalized test-list hashes remain
`5e41e4dbfcc303322c246a612de61926a628957a168589b45864d0a5070bb07e`
and `c0c2b80f8b4e6c84cd25d77573fda722c4d1846fed168cd4a478781cdb42775e`.
Task 262H0 is complete; Task 262H subsequently completed the production move.
No `spec_coverage_audit.md` update is required because test intent, coverage
credit, and owner crate are unchanged.

## Task 262H Move Result

Task 262H moved the three inventoried builtin-binary fragments into the existing
private `source_formula.rs` leaf. Review normalization confirms that the
config/transport, extractor, and allowlist bodies are identical to HEAD after
removing only required runner-subtree visibility and rustfmt whitespace. The
three config entries retain their exact order, labels, operators, numeral
spellings, and `FormulaKind` values.

`runner.rs` is now 14,430 lines with hash
`c0f358ac368f31c560f204df8e89e8885144366c9871f288a0306fa84e2ae981`;
the 24-line phase facade has hash
`d3b9de31b1bf6c2b68d4bafd088c7b88addab6db083a5b5adff93e581f1981d4`;
and the 310-line source-formula leaf has hash
`32978c9783b913439e9f8e94d326789c13aefff4d5e8326c669cb1a7d9745d6c`.
The unchanged 147-line source-AST and 1,474-line source-reserve leaves retain
hashes `baf131e5f82846df2286ad68c6e8bad9d2642af2ce530f7b8c7362900ef2aa9e`
and `88cf0cf08de2e61b2e6342aacc36ee01e20e00606d0c51f4bf7b5c64495253db`.

The extraction entry crosses the private facade unconditionally. The config
constant crosses only under `#[cfg(test)]`; the config type and its four
test-consumed fields have runner-subtree visibility, while its `formula_kind`
and the node allowlist remain leaf-private. The inferred transport and fields
are runner-subtree-visible without a facade type alias. The production
detail/checker caller and all H0 test/support code remain byte-for-byte
unchanged. All 272 unit tests and relevant-crate tests pass, all 188 active type
cases pass, and the raw/normalized test-list hashes remain
`5e41e4dbfcc303322c246a612de61926a628957a168589b45864d0a5070bb07e`
and `c0c2b80f8b4e6c84cd25d77573fda722c4d1846fed168cd4a478781cdb42775e`.

Task 262H is complete. Parent Task 262 remains open for fresh bounded inventory
of the reserved-variable/binary, builtin type-assertion, imported,
set-enumeration, and connective/quantifier formula families. No
`spec_coverage_audit.md` update is required because behavior, authority,
coverage credit, and owner crate are unchanged.

## Task 262I Pre-Move Inventory and Specification

Fresh inventory at clean HEAD `628b3272` isolates the builtin type-assertion
formula family as three contiguous `runner.rs` fragments:

- the 8-line `SourceBuiltinTypeAssertionFormula` transport at lines
  1,649-1,656, hash
  `88bc334c400dd92327d5fdc25e90efef1560cc097f5f2ecd6a5a822883082da4`;
- the 82-line `extract_source_builtin_type_assertion_formula` entry at lines
  12,988-13,069, hash
  `c4d2a3911147e0ceefdb6d4f0b767e19ea829cc66e8f52d67fb5c146e2b3540d`;
- the 16-line dedicated node allowlist at lines 13,686-13,701, hash
  `1e7c125594df441e775eac25259e0dd5c3a1081896ac461a5c441fb53748a844`.

The family implements the exact source-derived slice for Chapter 14.2.3's
type-assertion shape and the active
`fail_type_elaboration_builtin_type_assertion_formula_gap_001.miz` sidecar. Its
trace and expectation require the real numeral term, type-assertion formula,
and builtin-set type payload before fail-closed numeric-type and partial-formula
diagnostics. The production checker/detail caller remains in `runner.rs`.

Task 262I moves only these three fragments into the existing private
`source_formula.rs` leaf. The extractor directly consumes
`SourceTypeExpression` and `extract_builtin_source_type_expression` from the
existing lower-level `source_reserve.rs` leaf. This establishes the acyclic
`source_formula -> source_reserve` projection dependency; `source_reserve` has
no reverse dependency. The extraction entry crosses the phase facade
unconditionally. The inferred transport and all six consumed fields need only
runner-subtree visibility, the allowlist remains leaf-private, and no config,
test-only re-export, or facade type alias is needed.

Task 262I is move-only: theorem label/tokens, recovery policy, structural
cardinality, numeral spelling, asserted builtin-set constraints, payload sites
and ranges, checker outputs, detail keys, ordering, and fail-closed behavior
must not change. There is no helper rename, deduplication, generalization,
semantic edit, or test rewrite. The exact active source, trace, expectation,
Task 262I0 matrix, route-isolation cases, 188 active type cases, and 272-test
list are the preservation oracle. No `spec_coverage_audit.md` update is required
because coverage credit, owner crate, and deferred status remain unchanged.

## Task 262I0 Test-Gap Inventory and Specification

The Task 262I test-sufficiency review found an independent `test_gap` that must
be repaired before the production move. The existing positive unit test checks
checker kind/status and compares formula/subject sites and asserted range back
to values from the same extraction payload. It does not independently fix the
formula, subject, and asserted-type `TypedSiteRef`s or their source ranges, and
does not directly assert `payload.asserted_type_site` or the payload-level
builtin-set spelling, head, and empty attributes.

The negative matrix already rejects a wrong label, status prefix, wrong numeral,
builtin `object`, attributed `set`, and extra reserve/root content. It does not
exercise recovery on the exact theorem label or `is` token, duplicate theorem
or formula-expression nodes, an extra formula child, negation/wrong direct
formula tokens, or an extra assertion operand. These branches guard the exact
recovery, singleton, token, and two-operand contract that Task 262I must
preserve.

Task 262I0 is a test-only repair. Extend the existing shared synthetic builder
with a bounded builtin type-assertion corruption shape, and strengthen the
existing `source_reserve_bridge_reports_gap_or_evidence_detail_for_unsupported_shapes`
test. Independently derive the expected source offsets from the fixed theorem
label, numeral, `is`, and `set` spellings; assert source/node kind plus exact
site/range for formula, subject, and asserted type; assert payload spelling,
head, attributes, and checker handoff; and require every recovery/duplicate/
token/cardinality corruption to return no extraction and the unchanged payload
extraction-gap detail key.

Task 262I0 adds no test and changes no production source, `.miz`, expectation,
trace, specification, public API, diagnostic, payload behavior, test name, or
test count. It must be a separate commit before move-only Task 262I. This
repairs preservation coverage only, so `spec_coverage_audit.md` remains
unchanged.

## Task 262I0 Test Repair Result

Task 262I0 strengthened the existing test without adding or renaming any test.
The positive matrix now derives formula, numeral-subject, and asserted-type
ranges from the exact label, subject, `is`, and `set` spellings; selects each
expected site independently by node kind and range; and fixes every extraction
payload field. It also fixes the checker type-entry cardinality at two, requires
exactly one asserted-type entry owned by the asserted-type site, requires the
subject term entry to retain the subject site, and anchors the checked formula
and normalized asserted type to the independent source expectations.

The bounded corruption builder adds only default-off recovery, duplicate,
token-shape, and structural-cardinality controls. The existing negative matrix
now verifies recovered exact label, recovered `is`, duplicate theorem,
duplicate formula expression, extra formula child, negation, and extra
assertion operand cases. Every case returns no extraction and the unchanged
`type_elaboration.external_dependency.ast_payload_extraction` detail key.

`support.rs` is 6,765 lines with hash
`757e507c998c0a0acdc6334b3d9ea1c68a0dbe9b87bb0eb623fca93e49942b4b`;
`source_gap_and_equality.rs` is 3,250 lines with hash
`ed70cdc2536d6f44362c56b303cedee4ac0c666809abc4c1189b283963ce4b90`.
Production source, `.miz`, specification, trace, expectation, public API,
diagnostics, payload behavior, test names, and test count are unchanged.

The focused test, all relevant-crate tests, all 272 unit tests, all 188 active
type cases, formatting, all-target/all-feature Clippy, and workspace tests pass.
Plan/count remains 403/367, type coverage 235/223, pass/fail 219/184, and the
raw/normalized test-list and four CLI hashes remain unchanged. Task 262I0 is
complete, and move-only Task 262I subsequently completed the production move.
No `spec_coverage_audit.md` update is required because behavior, test intent,
coverage credit, and owner crate are unchanged.

## Task 262I Move Result

Task 262I moved the three inventoried builtin type-assertion fragments into the
existing private `source_formula.rs` leaf. Review normalization confirms that
the transport, extractor, and allowlist bodies are byte-equivalent to HEAD
`1b113e8b` after removing only the required runner-subtree visibility. The
strengthened Task 262I0 test/support files and every production checker/detail
consumer remain byte-for-byte unchanged.

`runner.rs` is now 14,320 lines with hash
`7d347e8a932ec5a4115540a6e6822b0ee23a6e41e919300ec56c04e5511303e4`;
the 24-line phase facade has hash
`61b5b82055f4f726d3b5209e2e6b57a176d0acaac5fbef9e1614780460306270`;
and the 423-line source-formula leaf has hash
`a055d6e2220961f5445bbf4b5394b2ffc72738160dbd228af399e267241ec43d`.
The unchanged 147-line source-AST and 1,474-line source-reserve leaves retain
hashes `baf131e5f82846df2286ad68c6e8bad9d2642af2ce530f7b8c7362900ef2aa9e`
and `88cf0cf08de2e61b2e6342aacc36ee01e20e00606d0c51f4bf7b5c64495253db`.

The extraction entry crosses the private phase facade unconditionally. The
inferred transport and all six runner-consumed fields use runner-subtree
visibility without a facade type alias, while the dedicated allowlist remains
leaf-private. `source_formula` now directly consumes the existing
`SourceTypeExpression` and builtin type-expression projection from
`source_reserve`; there is no reverse import, so the dependency remains
acyclic. Checker payload construction, detail rendering, route ordering, and
top-level orchestration stay in `runner.rs`.

The focused preservation test, all relevant-crate tests, all 272 unit tests,
and all 188 active type cases pass. Plan/count remains 403/367, type coverage
235/223, pass/fail 219/184, and the raw/normalized test-list plus four CLI
hashes are unchanged. Formatting, all-target/all-feature Clippy, workspace
tests, and diff cleanliness also pass. Task 262I is complete. It changes no
behavior, authority, coverage credit, owner crate, or deferred status, so
`spec_coverage_audit.md` remains unchanged. Parent Task 262 remains open for a
fresh bounded inventory of the remaining formula families.

## Tasks 262J1-J2 Pre-Move Inventory and Specification

Fresh inventory at clean HEAD `fdce5d8a` isolates the imported
predicate/functor formula work into five `runner.rs` fragments:

- the 15-line transport at lines 1,648-1,662, hash
  `474b345cfa983e95fcce895a08a56c89a51bd1d3b8cf542b0fbacb16c42fe76e`;
- the 127-line family extractor at lines 12,978-13,104, hash
  `6b967aff4d407f448cd8fd72aac205e88824c327f0048bb325786ef9a73e8bd4`;
- the 61-line exact infix transport/helper at lines 13,486-13,546, hash
  `9b6b8d4f5fd417f6654f4232448514a279f006309c1308219514024bee4421b2`;
- the 23-line dedicated allowlist at lines 13,593-13,615, hash
  `2daf39d17bde7186fe4a7fff4ad7fe6270ffc7a71e6ec1bdb44dbc2ba03fdafa`;
- the 43-line shared imported symbol resolver/provenance pair at lines
  13,706-13,748, hash
  `fc4914d1c4a557f1401db035032c22e84430faf0ac9355b8d3a1cf3716761955`.

The exact active sidecar is governed by imported visibility/conflict semantics,
import-prelude semantics, the parenthesized infix term surface, and predicate
application syntax. It supplies real imported `divides` predicate and `++`
functor symbols, three numeral sites, one infix functor site, and one predicate
formula before failing closed on missing numeric/signature/predicate-signature
payloads and partial formula checking.

The shared resolver has three callers: predicate and functor resolution in this
family plus the retained imported-attribute extractor. Task 262J1 therefore
moves only that resolver/provenance pair first. The resolver entry receives
runner-subtree visibility and an unconditional parent-facade alias; the
provenance predicate remains leaf-private, and all callers remain unchanged.

After J1, Task 262J2 moves only the transport, exact family extractor, exact
infix projection, and dedicated allowlist. The extractor crosses the facade
unconditionally. The transport and all 12 fields use runner-subtree visibility
without a facade type alias; the infix transport/helper and allowlist remain
leaf-private. Checker/detail/orchestration callers and the imported-attribute
family remain in `runner.rs`. Both tasks are move-only and forbid renaming,
deduplication, generalization, accepted-shape or symbol-admission changes,
operator metadata changes, diagnostics/detail/order changes, tests, and
authority edits. The dependency direction remains acyclic through
`source_formula -> source_ast` plus the existing `source_formula ->
source_reserve` edge.

## Task 262J0 Test-Gap Inventory and Specification

The J1/J2 test-sufficiency review found an independent `test_gap`. The current
positive matrix finds checker terms and formulas through extractor-returned
sites and checks imported symbols only by module path. It does not independently
fix the formula, outer numeral, infix term, or both infix operand sites/ranges,
all 12 transport fields, exact symbol kind/spelling/module/contribution
provenance, or checker ordering.

The existing negative matrix covers 12 source near-misses and six symbol-env
cases but lacks direct extractor assertions plus bounded recovery, duplicate,
predicate segment/head cardinality, parenthesized/infix cardinality, and
imported-contribution provenance corruption. Task 262J0 is a test-only repair:
extend the existing builder/environment support with default-off bounded
corruption controls, strengthen the existing test with independently derived
source expectations and exact checker handoff/order, and require every negative
case to return no extraction plus the unchanged extraction-gap detail key.

Task 262J0 adds no test and changes no production source, `.miz`, expectation,
trace, specification, public API, diagnostics, payload behavior, test name, or
test count. It is a separate commit before move-only J1/J2. Coverage credit,
owner crate, follow-up ownership, and deferred rationale remain unchanged, so
`spec_coverage_audit.md` remains unchanged.

## Task 262J0 Test Repair Result

Task 262J0 strengthened the existing imported predicate/functor test without
adding or renaming a test. The positive matrix now derives the formula, outer
numeral, infix term, both infix operands, and predicate formula sites and
ranges independently from the source spellings. It fixes all 12 extraction
transport fields, exact imported predicate/functor kind, spelling, module, and
contribution provenance, the checker term order, and the checked formula/term
site handoff.

The default-off bounded corruption builder covers recovered label/functor,
duplicate theorem/formula, formula/segment/head cardinality, parenthesized and
infix cardinality, and imported-contribution provenance. The 12 existing source
near misses, 11 structural corruptions, six existing symbol-environment cases,
and the isolated local-contribution case all return no direct extraction and
the unchanged `type_elaboration.external_dependency.ast_payload_extraction`
detail key.

`support.rs` is 6,943 lines with hash
`68e90fa32900462fbeac2065209d183762d85e4e32ddbe16d261680d564eed98`;
`source_gap_and_equality.rs` is 3,525 lines with hash
`69e2a9f82e83d95247f5ec1d88244b38a071db1a09bcae34ed4772401b35924d`.
Production source, `.miz`, specification, trace, expectation, public API,
diagnostics, payload behavior, test names, and test count are unchanged.

The focused test, relevant-crate tests, all 272 unit tests, and all 188 active
type cases pass. Plan/count remains 403/367, type coverage 235/223, pass/fail
219/184, and the raw/normalized test-list plus four CLI hashes are unchanged.
Formatting, all-target/all-feature Clippy, workspace tests, and diff cleanliness
also pass. Task 262J0 is complete and move-only Task 262J1 is next. No
`spec_coverage_audit.md` update is required because behavior, test intent,
coverage credit, owner crate, and deferred status are unchanged.

## Task 262J1 Move Result

Task 262J1 moved only the inventoried shared imported formula symbol
resolver/provenance pair into the existing private `source_formula.rs` leaf.
After removing the required `pub(in crate::runner)` visibility from the resolver
entry, the 43-line moved body retains baseline hash
`fc4914d1c4a557f1401db035032c22e84430faf0ac9355b8d3a1cf3716761955`.
The predicate, functor, and imported-attribute caller bodies and their order are
unchanged.

Only the resolver entry crosses the private phase facade through an
unconditional parent-only alias; its provenance predicate remains leaf-private.
`ContributionKind` and `NamespacePath` moved with the implementation while
runner-owned `SymbolEnv`, `SymbolKind`, and `ResolverSymbolId` uses remain in
`runner.rs`. The existing `source_formula -> source_ast` and `source_formula ->
source_reserve` dependencies stay acyclic, with no reverse edge.

`runner.rs` is now 14,277 lines with hash
`8d4e3ec02e275e3a5e69f3599285270cc176496b52321af72e29e063ca10fade`;
the 25-line phase facade has hash
`a969e263beb6eee47cbd111ff3efc25ef71122af1e7c7a8ae32a63c5c75dbd25`;
and the 467-line source-formula leaf has hash
`eb6ef963457cf16625e00b03fc81795ff89772e253f5c0b3a45a7c592e324bcf`.
Tests, authority artifacts, checker/detail consumers, public API, diagnostics,
payloads, ordering, and fail-closed behavior are unchanged.

The focused preservation test, relevant-crate tests, all 272 unit tests, and all
188 active type cases pass. Plan/count remains 403/367, type coverage 235/223,
pass/fail 219/184, and the raw/normalized test-list plus four CLI hashes are
unchanged. Formatting, all-target/all-feature Clippy, workspace tests, and diff
cleanliness also pass. Task 262J1 is complete and move-only Task 262J2 is next.
No `spec_coverage_audit.md` update is required because behavior, authority,
coverage credit, owner crate, and deferred status are unchanged.

## Task 262J2 Move Result

Task 262J2 moved only the four inventoried imported predicate/functor fragments
into the existing private `source_formula.rs` leaf. After removing only the
required runner-subtree visibility, the 15-line transport, 127-line extractor,
61-line exact infix projection, and 23-line allowlist retain baseline hashes
`474b345cfa983e95fcce895a08a56c89a51bd1d3b8cf542b0fbacb16c42fe76e`,
`6b967aff4d407f448cd8fd72aac205e88824c327f0048bb325786ef9a73e8bd4`,
`9b6b8d4f5fd417f6654f4232448514a279f006309c1308219514024bee4421b2`,
and `2daf39d17bde7186fe4a7fff4ad7fe6270ffc7a71e6ec1bdb44dbc2ba03fdafa`.

Only the extractor crosses the private phase facade unconditionally. The
transport and all 12 fields use runner-subtree visibility without a facade type
alias, while the exact infix transport/helper and dedicated allowlist remain
leaf-private. The checker/detail/orchestration caller and the imported-attribute
family remain unchanged in `runner.rs`; the moved extractor reuses Task 262J1's
leaf-owned resolver and direct source-AST projections. The now-unused
`exact_numeral_term_node_or_expression` facade and runner aliases were removed
after its sole external caller moved into the leaf; its implementation and
visibility are unchanged.

`runner.rs` is now 14,047 lines with hash
`9e47a64eedd35ae7e66629bdfefdaa39a86389d5002925af3887a2b7282222d0`;
the 25-line phase facade has hash
`2fad12f17b75a9ec51e97132846fbe926abeeeffb9f8c32eb78df93d0eab1330`;
and the 698-line source-formula leaf has hash
`a4d3fbe9708eade5d3b6ca3db965f3fd119aff8723c30d6ed6fbf9ccd982f049`.
Tests, authority artifacts, public API, diagnostics, payloads, ordering,
accepted shapes, and fail-closed behavior are unchanged.

The focused preservation test, relevant-crate tests, all 272 unit tests, and all
188 active type cases pass. Plan/count remains 403/367, type coverage 235/223,
pass/fail 219/184, and the raw/normalized test-list plus four CLI hashes are
unchanged. Formatting, all-target/all-feature Clippy, workspace tests, and diff
cleanliness also pass. Task 262J2 is complete; parent Task 262 remains open for
fresh bounded inventory of the remaining formula families. No
`spec_coverage_audit.md` update is required because behavior, authority,
coverage credit, owner crate, and deferred status are unchanged.

## Tasks 262K0-K Pre-Move Inventory and Specification

Fresh inventory at clean HEAD `9625d0a1` isolates the exact imported attribute
assertion family in three `runner.rs` fragments:

- the 8-line five-field transport at lines 1,649-1,656, hash
  `f6b78fea06f451c61eac5286ea41b8f85e33bfa80d4b392cfd68d65e9117f5ca`;
- the 141-line exact `empty`/`non empty` two-entry and shared-shape extractor at
  lines 12,963-13,103, hash
  `a7aa82e3b3a97cbdcf2b7506920bda40cf7d4ddeef2feb5a1124c5d7e3b93c05`;
- the 21-line dedicated node allowlist at lines 13,388-13,408, hash
  `3f13f99cd6fe64cd8baddceefdeed904e4b118d2132c6cecd06a2fe7187f0e76`.

The exact active bridges distinguish positive `empty` from attribute-level
`non empty`; the latter is not formula-level negation. Both import
`parser.type_fixtures`, require the imported `empty` attribute, pass one real
source-derived numeral and one attribute-assertion formula to the checker, and
then fail closed on missing numeric and formula/attribute semantic payloads.
Chapter 14 and the canonical harness preserve that polarity boundary and keep
broader attribute semantics deferred.

Task 262K moves only these three fragments after K0. Both exact extractor
entries use runner-subtree visibility and unconditional parent-facade aliases;
the shared-shape extractor and allowlist remain leaf-private. The transport and
all five fields use runner-subtree visibility. The transport also crosses the
facade through a parent-only type re-export because the retained checker helper
names it as an argument; this avoids changing or duplicating that consumer.
Checker/detail/orchestration consumers stay in `runner.rs`. The moved family
reuses the leaf-owned imported-symbol resolver, exact numeral projection, and
source-AST projections. After their last external caller moves, K also removes
only the obsolete runner/facade aliases for `exact_compilation_item_list`,
`is_exact_parser_type_fixtures_import`, `qualified_symbol_spelling`, and
`resolve_imported_fixture_term_formula_symbol`, plus the runner-only
`SymbolKind` import. The leaf implementations and visibility remain unchanged.
No new reverse dependency is introduced.

## Task 262K0 Test-Gap Inventory and Specification

The K test-sufficiency review found an independent `test_gap`. Both current
positive matrices locate checker terms/formulas through extractor-returned
sites and check imported provenance only by module path. They do not
independently fix formula/subject sites and ranges, all five transport fields,
exact `AttributeRef` polarity, symbol kind/spelling/module/contribution
provenance, or singleton checker ordering and formula-to-subject handoff.

Existing source and environment near misses assert only the rendered gap
detail. They lack direct extractor rejection, recovery, duplicate theorem or
formula expression, formula/assertion/attribute-chain/attribute-ref/qualified-
symbol/numeral cardinality corruption, duplicate or mismatched `non`, and an
isolated imported-contribution-kind corruption. Task 262K0 is a test-only
repair: add default-off family-specific corruption controls to existing support
and strengthen the existing test for both variants with independently derived
source expectations, exact provenance/order, direct `None`, and the unchanged
`type_elaboration.external_dependency.ast_payload_extraction` detail key.

Task 262K0 adds no test and changes no production source, `.miz`, expectation,
trace, specification, public API, diagnostic, payload behavior, test name, or
test count. It is a separate commit before move-only K. Coverage credit, owner
crate, follow-up ownership, and deferred rationale remain unchanged, so
`spec_coverage_audit.md` remains unchanged.

## Task 262K0 Test Repair Result

Task 262K0 strengthened both existing imported attribute assertion variants
without adding or renaming a test. Each positive matrix now derives formula and
subject ranges from the exact source spellings, selects the `IsAssertion` and
`NumeralTerm` sites independently, fixes all five transport fields, checks
direct `AttributeRef` polarity (`[]` versus `["non"]`), and fixes exact imported
attribute kind, spelling, module, and contribution provenance. Singleton checker
term/formula order and formula-to-subject handoff are anchored to those
independent sites.

The default-off bounded builder covers recovered label and attribute symbol,
duplicate theorem and formula expression, formula/assertion/attribute-chain/
attribute-ref/qualified-symbol/numeral cardinality, and unexpected or duplicate
`non` for both variants. Every existing source/environment near miss and all 22
corruption cases return no direct extraction plus the unchanged
`type_elaboration.external_dependency.ast_payload_extraction` detail key. A
shared boolean environment builder proves an otherwise-identical ImportedSource
control extracts before its LocalSource contribution twin rejects.

`support.rs` is 7,146 lines with hash
`46340ae9aa4ac03b7e5e458a515814bea4db86de177625c97b57762d894a8025`;
`source_gap_and_equality.rs` is 3,974 lines with hash
`101fb755532276a12ce2202f297c318ad77249eab9aa27ce2670fe59e08ab47c`.
Production source, `.miz`, specification, trace, expectation, public API,
diagnostics, payload behavior, test names, and test count are unchanged.

The focused test, relevant-crate tests, all 272 unit tests, and all 188 active
type cases pass. Plan/count remains 403/367, type coverage 235/223, pass/fail
219/184, and the raw/normalized test-list plus four CLI hashes are unchanged.
Formatting, all-target/all-feature Clippy, workspace tests, and diff cleanliness
also pass. Task 262K0 is complete and move-only Task 262K is next. No
`spec_coverage_audit.md` update is required because behavior, test intent,
coverage credit, owner crate, and deferred status are unchanged.

## Task 262K Move Result

Task 262K moved only the three inventoried imported attribute assertion
fragments into the existing private `source_formula.rs` leaf. After removing
only the required runner-subtree visibility, the 8-line transport, 141-line
two-entry/shared extractor, and 21-line allowlist retain baseline hashes
`f6b78fea06f451c61eac5286ea41b8f85e33bfa80d4b392cfd68d65e9117f5ca`,
`a7aa82e3b3a97cbdcf2b7506920bda40cf7d4ddeef2feb5a1124c5d7e3b93c05`,
and `3f13f99cd6fe64cd8baddceefdeed904e4b118d2132c6cecd06a2fe7187f0e76`.

Both exact extractor entries cross the private phase facade unconditionally;
the shared shape extractor and allowlist remain leaf-private. The transport and
all five fields use runner-subtree visibility, and the transport has one
parent-only type re-export because the unchanged checker helper names it as an
argument. Checker/detail/orchestration consumers and their order remain
byte-identical in `runner.rs`.

The moved family directly reuses the leaf-owned imported-symbol resolver, exact
numeral projection, and source-AST projections. After their final external
caller moved, only the obsolete facade/runner aliases for
`exact_compilation_item_list`, `is_exact_parser_type_fixtures_import`,
`qualified_symbol_spelling`, and `resolve_imported_fixture_term_formula_symbol`,
plus runner's unused `SymbolKind` import, were removed. Leaf implementations
and visibility remain unchanged; dependencies stay acyclic.

`runner.rs` is now 13,874 lines with hash
`d03812923d461dc718cb4236ee5568dfa03ac07e3bfb0f5995627d46f345b2c6`;
the 26-line phase facade has hash
`8e5b39254a2ca468d62db55d3ba7a69bdfaea5248881d5a5c62ca8d3eed526dd`;
and the 871-line source-formula leaf has hash
`f1a6888ca7c10bfbf1a8a868261e34d31fa74003512250cdbe5b117e018f19de`.
Tests, authority artifacts, public API, diagnostics, payloads, polarity,
ordering, accepted shapes, and fail-closed behavior are unchanged.

The focused preservation test, relevant-crate tests, all 272 unit tests, and all
188 active type cases pass. Plan/count remains 403/367, type coverage 235/223,
pass/fail 219/184, and the raw/normalized test-list plus four CLI hashes are
unchanged. Formatting, all-target/all-feature Clippy, workspace tests, and diff
cleanliness also pass. Task 262K is complete; parent Task 262 remains open for
fresh bounded inventory of remaining formula families. No
`spec_coverage_audit.md` update is required because behavior, authority,
coverage credit, owner crate, and deferred status are unchanged.

## Tasks 262L0-L Pre-Move Inventory and Specification

Fresh inventory at clean HEAD `be7a2c6e` isolates the exact set-enumeration
formula family in four `runner.rs` fragments:

- the 11-line eight-field transport at lines 1,649-1,659, hash
  `5aa3f3e859cc0313f935e80011ef7be4e05299a0763f97de572eccc500fd71c8`;
- the 57-line exact extractor at lines 12,954-13,010, hash
  `f05ab26f14f3d28e2f721575ca7a53c74fae9dfeebb0779906fd0a6d45b7fc99`;
- the 43-line private exact-set transport and projection at lines
  13,148-13,190, hash
  `45c155d6556740807b395b0e1a8114094db074ac6768ee7d892b7e0eb2d26036`;
- the 15-line dedicated node allowlist at lines 13,237-13,251, hash
  `461650cdedc2f56cdf072e95e1ef0243bc7be1a3c7323e0628c652ad562b6dd1`.

The exact active bridge recognizes only
`SetEnumerationPayloadBoundary: {1, 2} = {1, 2}`, projects four real numeral
items, two set-enumeration terms, and one equality formula from the source AST,
and then fails closed on missing numeric type payloads, missing set result-type
payloads, and partial formula checking. Canonical Chapter 13, the exact `.miz`,
trace row, expectation, and harness agree; broader set-enumeration extraction
and semantics remain deferred.

Task 262L moves only these four fragments after L0. The eight-field transport
and its fields use runner-subtree visibility, and only the exact extraction
entry receives an unconditional parent-facade alias. The exact-set transport,
projection helper, and allowlist remain leaf-private. The checker/detail
consumer `source_set_enumeration_formula_output` stays byte-identical in
`runner.rs`; because that consumer does not name the transport type, no facade
type re-export is required. The moved family directly reuses the leaf-owned
exact numeral and source-AST projections and introduces no reverse dependency.
After the last external exact-numeral caller moves, L removes only the obsolete
`exact_numeral_term_operand` facade alias and runner import while leaving its
leaf implementation and visibility unchanged.

## Task 262L0 Test-Gap Inventory and Specification

The L test-sufficiency review found an independent `test_gap`. The positive
matrix already independently fixes the eight transport fields, but merges the
left/right item vectors, locates all six checker terms with unordered search,
and compares formula handoff through extractor-returned sites. It therefore
does not independently fix 2+2 item grouping, both exact punctuation triples,
the deterministic six-term checker-output order, exact corresponding term-kind
order, or the formula's independently derived site and `[left_set, right_set]`
handoff.

The existing near-miss matrix checks only the rendered extraction-gap detail,
does not call the extractor directly, and couples two left item mismatches.
It lacks isolated four-position numeral near misses and allowlisted corruption
for formula-expression/formula/operand, term-wrapper/set/item, punctuation,
and numeral-child kind or cardinality guards. Task 262L0 is a test-only repair:
add default-off family-specific corruption controls to existing support and
strengthen the existing test with independent grouping/punctuation/order
expectations, direct `None`, and the unchanged
`type_elaboration.external_dependency.ast_payload_extraction` detail key.

Task 262L0 adds no test and changes no production source, `.miz`, expectation,
trace, specification, public API, diagnostic, payload behavior, test name, or
test count. It is a separate commit before move-only L. Coverage credit, owner
crate, follow-up ownership, and deferred rationale remain unchanged, so
`spec_coverage_audit.md` remains unchanged.

## Task 262L0 Test Repair Result

Task 262L0 strengthened the existing exact set-enumeration matrix without
adding or renaming a test. The positive path now fixes the separate 2+2 item
groups and both punctuation triples, and anchors the deterministic six-site
and six-kind checker output plus the equality formula's site and ordered set
terms to independently derived source sites. All eight transport fields remain
independently fixed.

The four item-spelling near misses now isolate left-first, left-second,
right-first, and right-second positions. Every existing source near miss calls
the extractor directly before checking the unchanged gap detail. Eleven
default-off, allowlisted corruption variants independently cover formula-
expression cardinality/kind, formula child/kind/operand cardinality, term-
wrapper kind/cardinality, set kind/punctuation/item cardinality, and numeral-
child cardinality; each returns no direct extraction plus the unchanged
`type_elaboration.external_dependency.ast_payload_extraction` detail key.

`support.rs` is 7,330 lines with hash
`451611d56191b98685fc27fd9a87eec36090f7b1dba11aa3a7a7f8e8d9e801e6`;
`source_gap_and_equality.rs` is 4,079 lines with hash
`e1836ed29e9b6593970047b5e68f746def70cbd86f9fd98b11aad7841459afb7`.
Production source, `.miz`, specification, trace, expectation, public API,
diagnostics, payload behavior, test names, and test count are unchanged.

The focused test, relevant-crate tests, all 272 unit tests, and all 188 active
type cases pass. Plan/count remains 403/367, type coverage 235/223, pass/fail
219/184, and the raw/normalized test-list plus four CLI hashes are unchanged.
Formatting, all-target/all-feature Clippy, workspace tests, and diff cleanliness
also pass. Task 262L0 is complete and move-only Task 262L is next. No
`spec_coverage_audit.md` update is required because behavior, test intent,
coverage credit, owner crate, and deferred status are unchanged.

## Task 262L Move Result

Task 262L moved only the four inventoried set-enumeration fragments into the
existing private `source_formula.rs` leaf. After normalizing only required
runner-subtree visibility and the wrapped public extraction signature, the
11-line transport, 57-line extractor, 43-line exact-set projection, and 15-line
allowlist retain baseline hashes
`5aa3f3e859cc0313f935e80011ef7be4e05299a0763f97de572eccc500fd71c8`,
`f05ab26f14f3d28e2f721575ca7a53c74fae9dfeebb0779906fd0a6d45b7fc99`,
`45c155d6556740807b395b0e1a8114094db074ac6768ee7d892b7e0eb2d26036`,
and `461650cdedc2f56cdf072e95e1ef0243bc7be1a3c7323e0628c652ad562b6dd1`
respectively.

The transport and all eight fields use runner-subtree visibility; only the
exact extractor crosses the phase facade. The exact-set transport, helper, and
allowlist remain leaf-private, and no transport type alias was added. The
retained `source_set_enumeration_formula_output` checker/detail consumer remains
byte-identical to HEAD with hash `710f25b9f406aad51eeb99c105abd79f9477e0c18b60ea3f27124a1b81330355`.
After its final external caller moved, only the obsolete
`exact_numeral_term_operand` facade alias and runner import were removed; the
leaf implementation, visibility, and body are unchanged.

`runner.rs` is now 13,744 lines with hash
`2fa77cd1126d591f37c13e2e7c0fb2522a3e9a269ecb81dbb26f86ffcd93f234`;
the 25-line phase facade has hash
`8aca34293b02fad31567ec4b3d2865e8c8fac95c333d060718885d462c19b8af`;
and the 1,003-line source-formula leaf has hash
`4bbe60d38ca7af3a320ab97c8b4f6e2aa61abd50dc41c68c6431e0fb7684af01`.
Tests, authority artifacts, public API, diagnostics, payloads, ordering,
accepted shapes, and fail-closed behavior are unchanged.

The focused preservation test, relevant-crate tests, all 272 unit tests, and
all 188 active type cases pass. Plan/count remains 403/367, type coverage
235/223, pass/fail 219/184, and the raw/normalized test-list plus four CLI
hashes are unchanged. Formatting, all-target/all-feature Clippy, workspace
tests, and diff cleanliness also pass. Task 262L is complete; parent Task 262
remains open for fresh bounded inventory of the remaining formula families. No
`spec_coverage_audit.md` update is required because behavior, authority,
coverage credit, owner crate, and deferred status are unchanged.

## Tasks 262M0-M Pre-Move Inventory and Specification

Fresh inventory at clean HEAD `334b83e2` isolates the exact formula
connective/quantifier family in three `runner.rs` fragments:

- the 13-line ten-field transport at lines 1,649-1,661, hash
  `98f4a9a771cebc18faa43d1b266dd78f931f00a7d9435c7f6606cfd807a6e424`;
- the 135-line exact extractor at lines 12,942-13,076, hash
  `a64080512c757a0a8f85357ec5a086285d7139257bd816bdbb85a5ae19bcd56f`;
- the 18-line dedicated node allowlist at lines 13,123-13,140, hash
  `80c39e182da04f34e2598f0670fcba4c17785dbea34373ef7d7847e3488cce1f`.

The exact active bridge recognizes only a contradiction premise implying a
universal `set`-typed binder whose body is a negated contradiction. It projects
two contradiction constants plus implication, quantified, and negation shells
from the real source AST, then fails closed on missing formula and quantifier
payloads. Canonical Chapter 14, the exact `.miz`, trace row, expectation, and
harness agree; broader connective, binder, child-formula, and theorem semantics
remain deferred.

Task 262M moves only these three fragments after M0. The ten-field transport
and its fields use runner-subtree visibility, and only the exact extraction
entry receives an unconditional parent-facade alias. The allowlist remains
leaf-private. The checker/detail consumer
`source_formula_connective_quantifier_output` stays byte-identical in
`runner.rs`; because it does not name the transport type, no facade type
re-export is required. The moved extractor directly reuses the leaf-owned
source-AST projections and `source_reserve` builtin type-expression projection,
so dependency direction remains acyclic. After the move, remove only the now-
unused production runner imports `SurfaceFormulaConnective`,
`SurfaceFormulaConstant`, `SurfaceFormulaPrefixOperator`, and
`SurfaceQuantifierKind`; test-support imports are independent. Retain
`extract_builtin_source_type_expression`, `TypeHeadInput`, and all source-AST
facade aliases because other runner callers still require them.

## Task 262M0 Test-Gap Inventory and Specification

The M test-sufficiency review found an independent `test_gap`. The positive
matrix independently fixes all ten transport fields and the five shell states,
but locates checker formulas with unordered searches through extractor-returned
sites. It does not fix deterministic five-site/five-kind output order or the
complete diagnostic provenance: only the two contradiction-constant formula
diagnostics are source-anchored, while implication, quantified, and negation
diagnostic key/range pairs are not checked. Binder segment/type-expression/head
shape and direct `x being` / `set` tokens also lack independent assertions.

The existing near-miss matrix checks only the rendered extraction-gap detail
and does not call the extractor directly. It lacks allowlisted corruption for
formula-expression, implication/repetition/token/operands, premise constant,
universal token/children, binder segment/token/children, negation token/children,
body constant, and recovered inner nodes. An attributed-set binder is a separate
non-allowlisted near miss because its `AttributeChain`, `QualifiedSymbol`, and
`PathSegment` nodes necessarily fail the current family allowlist; M0 must not
widen that production boundary or claim to isolate the later attributes-empty
guard. Task 262M0 is a test-only repair: add default-off family-specific
allowlisted corruption controls to existing support, add that separate
attributed-set near miss, and strengthen the existing test with independent
binder, output-order/state, diagnostic key/range, direct `None`, and unchanged
`type_elaboration.external_dependency.ast_payload_extraction` detail-key
assertions. The default-off corruptions must preserve the family allowlist and
preceding guards so each rejection remains isolated.

Task 262M0 adds no test and changes no production source, `.miz`, expectation,
trace, specification, public API, diagnostic, payload behavior, test name, or
test count. It is a separate commit before move-only M. Coverage credit, owner
crate, follow-up ownership, and deferred rationale remain unchanged, so
`spec_coverage_audit.md` remains unchanged.

## Task 262M0 Test Repair Result

Task 262M0 strengthened the existing exact connective/quantifier matrix without
adding or renaming a test. The positive path now fixes the binder segment,
type-expression, and type-head ranges plus direct `x`, `being`, and `set`
tokens. All ten transport fields remain independently anchored. The observed
deterministic checker order is fixed as body contradiction, negation,
quantified shell, implication, then premise contradiction, with exact site,
kind, context, partial status, and deferred reason for all five entries. The
four formula-payload and one quantifier-payload diagnostic key/range pairs are
fixed as a complete multiset.

Every existing connective/quantifier near miss now asserts direct extractor
rejection before the unchanged detail key. The attributed-set binder is kept as
an explicit non-allowlisted near miss without widening production or claiming
the later attributes-empty guard. Eighteen default-off, allowlisted corruptions
independently cover theorem/formula-expression shape, implication repetition/
token/operands, premise kind/token, universal token/children, binder segment
kind/token/children, negation token/children, body kind/token, and descendant
recovery. Each returns no direct extraction plus the unchanged
`type_elaboration.external_dependency.ast_payload_extraction` detail key.

`support.rs` is 7,551 lines with hash
`7315c2d22d5d0e7dbf27c2086e34f3177e6b1fba6c57f3e9db0cd51660081af0`;
`source_gap_and_equality.rs` is 4,260 lines with hash
`dd39dcbaf71644d6e6a9d0035fb9d838925e6d2db0892b58009c53e495fe6369`.
Production source, `.miz`, specification, trace, expectation, public API,
diagnostics, payload behavior, test names, and test count are unchanged.

The focused test, relevant-crate tests, all 272 unit tests, and all 188 active
type cases pass. Plan/count remains 403/367, type coverage 235/223, pass/fail
219/184, and the raw/normalized test-list plus four CLI hashes are unchanged.
Formatting, all-target/all-feature Clippy, workspace tests, and diff cleanliness
also pass. Task 262M0 is complete and move-only Task 262M is next. No
`spec_coverage_audit.md` update is required because behavior, test intent,
coverage credit, owner crate, and deferred status are unchanged.

## Task 262M Move Result

Task 262M moved only the three inventoried formula connective/quantifier
fragments into the existing private `source_formula.rs` leaf. After removing
only required runner-subtree visibility, the 13-line transport, 135-line
extractor, and 18-line allowlist retain baseline hashes
`98f4a9a771cebc18faa43d1b266dd78f931f00a7d9435c7f6606cfd807a6e424`,
`a64080512c757a0a8f85357ec5a086285d7139257bd816bdbb85a5ae19bcd56f`,
and `80c39e182da04f34e2598f0670fcba4c17785dbea34373ef7d7847e3488cce1f`.

The transport and all ten fields use runner-subtree visibility; only the exact
extractor crosses the phase facade, no transport type alias was added, and the
allowlist remains leaf-private. The retained
`source_formula_connective_quantifier_output` checker/detail consumer remains
byte-identical to HEAD with hash
`7bc5d0899674fda17899b4c78463ac1d83e9ed8ad99196a4b0bb2eaf11f844f0`.
Only the four now-unused production runner syntax-enum imports were removed;
test support, `TypeHeadInput`, the builtin type-expression extractor, and all
still-used source-AST facade aliases remain unchanged. Dependency direction is
acyclic.

`runner.rs` is now 13,573 lines with hash
`1ea8e97e9f87e92bbcdd5b9e17e8a1d829b46f34f14c1a53d983529ece9ce58f`;
the 26-line phase facade has hash
`1eb16a6815df883433ef6de6e7814cba7102e5962c8b5425ac875caba0c5fb69`;
and the 1,173-line source-formula leaf has hash
`d418905106d5b6313fe62644c4145c83428c056880f2f9b2d74cc2eb2d00760d`.
Tests, authority artifacts, public API, diagnostics, payloads, ordering,
accepted shapes, and fail-closed behavior are unchanged.

The focused preservation test, relevant-crate tests, all 272 unit tests, and
all 188 active type cases pass. Plan/count remains 403/367, type coverage
235/223, pass/fail 219/184, and the raw/normalized test-list plus four CLI
hashes are unchanged. Formatting, all-target/all-feature Clippy, workspace
tests, and diff cleanliness also pass. Task 262M is complete; parent Task 262
remains open for fresh bounded inventory of the remaining formula families. No
`spec_coverage_audit.md` update is required because behavior, authority,
coverage credit, owner crate, and deferred status are unchanged.

## Tasks 262N0-262Q Fresh Reserved-Variable Formula Inventory

Fresh inventory after Task 262M classifies the remaining source-formula code
as one shared reserved-variable source model/substrate followed by three
bounded extraction owners: direct binary, parenthesized binary, and type
assertion. Checker-output transports, builders, validators, detail keys, and
diagnostics remain Task 263 work. Concrete configs and their thin named route
wrappers stay paired in `runner.rs` until a later inventory can move them
without coupling source extraction to the result-role and invalid-detail-key
contracts consumed by Task 263.

The shared source substrate comprises the reserved-variable config/model type
definitions and schemas (not the concrete config values/statics), builtin/mode
projection predicates, asserted-head relation check, exact
mode-definition/expansion checks, the shared identifier operand projection,
and source-use ordinal calculation. The single-parenthesized operand projection
and the three family-specific node allowlists stay with their generic cores in
Tasks 262P and 262O-262Q respectively. The first candidate family range ending
at `runner.rs:12974` was rejected by review because it included the opening of
`source_module_binding_env`; the clean overall reserved-variable helper and
allowlist region ends at line 12969, with each task selecting only its assigned
subfragments. The projection predicates cannot move before their model type
definitions because that would invert ownership back toward runner-owned
source configuration. Task 262N therefore moves those definitions and the
shared substrate together, with only the aliases still required by retained
validators exposed parent-only. Concrete config values/statics and their thin
named wrappers remain paired in `runner.rs` pending later inventory at the Task
263 contract boundary.

Independent test review found a bounded `test_gap` before that move. The
existing exact equality bridge proves real checker handoff and many detail-key
failures, but it does not independently lock every config field, the formula
and two operand sites/ranges, direct extractor rejection for its existing
near-miss matrix, or allowlisted expression/predicate/term kind and cardinality
corruptions. Task 262N0 is test-only: strengthen the existing equality test and
its default-off private AST builder with those preservation assertions. It
adds no test and changes no production source, `.miz`, expectation, trace,
specification, public API, behavior, or coverage credit. Task 262N follows only
after N0 review and verification. Tasks 262O, 262P, and 262Q then move the
direct-binary, parenthesized-binary, and type-assertion source cores
respectively, with a fresh test-sufficiency review before 262Q.

## Task 262N0 Test Repair Result

Task 262N0 strengthened the existing exact reserved-variable equality test
without adding or renaming a test. It now independently derives and fixes the
formula plus both operand sites/ranges from the AST, fixes every binary config
field, directly rejects all 13 retained near misses, and rejects 16 default-off
corruptions. The corruption matrix independently covers formula-expression and
predicate kind/cardinality, left/right term-expression and reference
kind/cardinality, and left/right/operator recovery. The default path retains
the prior node/token sequence, kinds, ranges, order, and IDs.

No production source, `.miz`, expectation, trace, specification, API,
diagnostic, payload, ordering, or coverage artifact changed. The focused exact
test, all 272 unit tests, all 188 active type cases, relevant-crate tests, and
workspace tests pass. Plan/count remains 403/367, type coverage 235/223, and
pass/fail 219/184. The raw/normalized test-list and four CLI hashes remain
unchanged. Formatting, all-target/all-feature Clippy, and diff cleanliness also
pass. Task 262N0 is complete and move-only Task 262N is next. No
`spec_coverage_audit.md` update is required because authority, behavior,
coverage credit, owner crate, and deferred status are unchanged.

## Task 262N Pre-Move Inventory and Specification

Task 262N moves exactly four source-substrate fragments from `runner.rs` into
the existing private `type_elaboration/source_formula.rs` leaf:

- 136 lines of reserved-variable config/model type definitions and methods,
  hash `2c3ebcfe343f60ddae3bb2124f4f15f942c0f8236f54b42954ed4453766e2ac1`;
- 562 lines of builtin/mode projection predicates, asserted-head relation
  checking, and terminal-mode traversal, hash
  `ffe1ae491ff3b7548171410a840e4ea6ea5edbdf69cee595b6c18b74e4612da6`;
- 115 lines of exact mode-definition/expansion checks plus the shared direct
  identifier operand projection, hash
  `eb5d150c267e2f7e3a1259ecb75b7e985caab81aba973be0b71ed15522d8cfcb`;
- 50 lines of source binding/use ordinal validation, hash
  `4bdef09433003048b6b439f7dd2ee3bc154fa8c3cc63082aedae7a8bcb44b3a0`.

The 863 moved lines are byte-preserved except for the minimum
`pub(in crate::runner)` visibility needed by retained concrete configs,
validators, and later generic cores. `source_mode_expansion_for_spelling`
stays leaf-private. The phase facade re-exports only the moved model schemas
and helpers that `runner.rs` still consumes. Task 262N does not move or edit
concrete config values/statics, thin named wrappers, source transports,
generic extractors, the single-parenthesized operand projection, family
allowlists, checker/output transports or bodies, detail keys, or diagnostics.
No rename, deduplication, generalization, or semantic cleanup is permitted.

Completion requires normalized moved-fragment equivalence after stripping the
visibility-only prefixes; the strengthened N0 test; all direct,
parenthesized, asserted-head, type-assertion, long-chain, and cross-owner
isolation tests; unchanged 272-name raw/normalized test lists; all 188 active
type cases; unchanged plan/count, coverage, pass/fail, and four CLI hashes;
formatting, Clippy, relevant-crate/workspace tests, diff cleanliness; and a
no-findings review for visibility, dependency direction, behavior, and paired
source/docs consistency.

## Task 262N Move Result

Task 262N moved only the four inventoried reserved-variable source-substrate
fragments into the existing private `source_formula.rs` leaf. After stripping
the runner-scoped visibility qualifiers, the model, predicate, mode/identifier,
and ordinal fragments retain hashes `2c3ebcfe343f60dd...`,
`ffe1ae491ff3b754...`, `eb5d150c267e2f7...`, and
`4bdef09433003048...` respectively. Review narrowed `spelling` and
`input_head` back to leaf-private; `source_mode_expansion_for_spelling` also
remains leaf-private. All other runner-scoped types, fields, methods, helpers,
and facade aliases have retained consumers. The production facade no longer
aliases the two reserve spelling projections that only the formula leaf uses;
the one spelling helper still consumed by private tests is exposed only under
`#[cfg(test)]`.

Concrete config values/statics and thin named wrappers, source transports,
generic direct/parenthesized/type-assertion cores, the single-parenthesized
operand projection, all three family allowlists, checker/output transports and
bodies, detail keys, and diagnostics remain in `runner.rs`. The dependency
direction is acyclic: `source_formula` depends on the sibling source-AST and
source-reserve leaves, and retained runner code consumes it only through the
35-line phase facade.

The resulting `runner.rs` has 12,717 lines and hash
`2a20df9e786bac81e30a60fdd1824b44fc87dbd38eeb20ba97bdeb3862a0a33a`;
the 35-line facade has hash
`65d8c6a8bbd1421f827888d9444502c41ae7f2e7e69c1eb15928ea34f347b2e2`;
and the 2,044-line source-formula leaf has hash
`8fabf38e9dea88b7fc1387508ce21a6d29080659af1148fb694c2da74c8aae49`.
The focused N0 test, all 272 unit tests, relevant-crate tests, workspace tests,
and all 188 active type cases pass. Plan/count remains 403/367, type coverage
235/223, pass/fail 219/184, and the raw/normalized test-list plus four CLI
hashes are unchanged. Formatting, all-target/all-feature Clippy, and diff
cleanliness also pass. Task 262N is complete and Task 262O is next. No
`spec_coverage_audit.md` update is required because authority, behavior,
coverage credit, owner crate, and deferred status are unchanged.

## Task 262O Pre-Move Inventory and Specification

Task 262O moves exactly three direct reserved-variable binary source-core
fragments from `runner.rs` into the existing private `source_formula.rs` leaf:

- the 16-line source transport, hash
  `d7c400d8c6c6d101c40159a3f76b910a27113a6f9092c4b6672ef4bd1e41a303`;
- the 126-line generic direct-binary extractor, hash
  `9f7e12badc208e4a7686bcabffb0da648748b9be7b672a2706f846690c42f4c3`;
- the 19-line direct-binary node allowlist, hash
  `8b6d0b2e43a4346121e3b571246210b16d487a635a618c5ff66eeefe05fb1a77`.

The 161 lines move with normalized equivalence after adding only the minimum
runner-scoped visibility. The source transport and extractor remain consumed
by retained named route wrappers and checker/output validators. The binary
allowlist temporarily receives a parent-only alias because the retained
parenthesized allowlist delegates to it; Task 262P must remove that alias when
the parenthesized core moves. Task 262O adds the direct reserve-extraction
dependency to the formula leaf but does not move concrete configs/wrappers,
the parenthesized or type-assertion transports/cores/allowlists, the
single-parenthesized operand projection, checker/output transports or bodies,
detail keys, or diagnostics.

Task 262N0 plus the existing direct binary route/source/corruption/isolation
matrix is sufficient; no new test is required. Completion requires all three
normalized hashes, minimal visibility, preserved fail-closed shape/order and
payload provenance, unchanged 272-name lists and 188 active cases, unchanged
plan/count/coverage/pass-fail/CLI hashes, full Rust verification, diff
cleanliness, and no-findings implementation/source-doc review. No rename,
deduplication, generalization, or semantic cleanup is permitted.

## Task 262O Move Result

Task 262O moved only the three inventoried direct reserved-variable binary
source-core fragments into the existing private `source_formula.rs` leaf. The
transport and extractor retain their original hashes after stripping the added
runner-scoped visibility. The allowlist retains its original hash after the
same visibility normalization and undoing only rustfmt's signature wrapping.
The formula leaf now directly consumes the sibling reserve-extraction entry;
the phase facade exposes the transport, extractor, and temporarily the binary
allowlist to their retained runner consumers. The latter alias is used only by
the retained parenthesized allowlist and must disappear with Task 262P.

The resulting `runner.rs` has 12,558 lines and hash
`25eff814585b074fc137f87f8da8172dadef3aa02b703bab1b35b5287156c920`;
the 38-line facade has hash
`5083cf8a6bcc49144c0f8f594b1a1a4d30007a1d4c2da840b8bda136c0d2dce4`;
and the 2,209-line source-formula leaf has hash
`88132f00f4f925c9293142310660b495e688f6a1d65659e88ec1dcc51ea83c14`.
Concrete configs/wrappers, the parenthesized and type-assertion
transports/cores/allowlists, the single-parenthesized operand projection, and
checker/output/detail/diagnostic code remain in `runner.rs` unchanged.

All 272 unit tests and 188 active type cases pass. Plan/count remains 403/367,
type coverage 235/223, and pass/fail 219/184. The raw/normalized test-list and
four CLI hashes remain unchanged. Formatting, all-target/all-feature Clippy,
workspace tests, and diff cleanliness pass. Task 262O is complete and Task
262P is next. No `spec_coverage_audit.md` update is required because authority,
behavior, coverage credit, owner crate, and deferred status are unchanged.

## Task 262P Pre-Move Inventory and Specification

Task 262P moves exactly four parenthesized reserved-variable binary source-core
fragments from `runner.rs` into the existing private `source_formula.rs` leaf:

- the 13-line source-side wrapper enum and transport, hash
  `9574d330441d576284cfabaadcb9963efb1bf4ff441a1c88bff631a68706ab00`;
- the 184-line generic parenthesized extractor, hash
  `a252111f84228774ed187e4bfc22ddaa4f297171f7f23dad4e16e0971745f080`;
- the 52-line exact single-parenthesized identifier projection, hash
  `b776ca58fb0873f1bb050d15b9ab90a878b02809f980a00c6f05433ceb57cab2`;
- the 6-line parenthesized node allowlist, hash
  `c45e2f1d15cd1dfc503df711dd58615291f909faff37433d49bca8a741f71157`.

The 255 lines move with normalized equivalence after adding only the minimum
runner-scoped visibility to the enum, source transport and its four retained
consumer fields, and generic extractor. The single-parenthesized projection
and parenthesized allowlist become leaf-private. Moving the latter eliminates
Task 262O's temporary facade alias and runner import for the direct-binary
allowlist and narrows that direct allowlist itself back to leaf-private
visibility. The formula leaf retains its acyclic dependencies on the common
source-AST and sibling reserve-extraction leaves.

All eight named route wrappers, concrete configs, source-output transport,
checker/output conversion and validation, detail keys, diagnostics, and tests
remain in `runner.rs` unchanged. The existing exact left/right-parenthesized
active slices plus their direct source, wrapper/range/provenance corruption,
near-miss, cross-route isolation, immutable-output, and real frontend/resolver
sidecar coverage are sufficient; no test-only prerequisite or new test is
required. Completion requires all four normalized hashes, removal of the
temporary allowlist alias and visibility, preserved wrapper side/site/range and
inner operand ordering, unchanged fail-closed behavior and 272-name/188-active
inventories, unchanged plan/count/coverage/pass-fail/CLI hashes, full Rust
verification, diff cleanliness, and no-findings implementation/source-doc
review. No rename, deduplication, generalization, semantic cleanup, or
checker/output movement is permitted.

## Task 262P Move Result

Task 262P moved only the four inventoried parenthesized reserved-variable
binary source-core fragments into the existing private `source_formula.rs`
leaf. After stripping the added runner-scoped visibility, the enum/transport
and generic extractor retain hashes `9574d330...` and `a252111f...`; the
leaf-private single-parenthesized projection and allowlist retain hashes
`b776ca58...` and `c45e2f1d...` exactly. The direct-binary and parenthesized
allowlists are both leaf-private. Task 262O's temporary facade alias, runner
import, and direct-allowlist visibility are gone.

The resulting `runner.rs` has 12,300 lines and hash
`563bb974845d95da52e723f1c3e853b79beb55c02e283e1cd10707589d1e5b70`;
the 39-line facade has hash
`5082a9a6a52c72ed8c95482b425823161bad64b5d75cfb8f14b4143110745c6f`;
and the 2,466-line source-formula leaf has hash
`a09c2c1d757f00c3e27ddb993d78f5aeed06dd08ef0f20aa27c7b080334c9c28`.
All eight named wrappers/configs, output transport, checker conversion and
validation, detail keys, diagnostics, and tests remain in `runner.rs`; only
the now-unused `SurfaceNodeId` import was removed with the moved helpers.

All 272 unit tests and 188 active type cases pass. Plan/count remains 403/367,
type coverage 235/223, and pass/fail 219/184. The raw/normalized test-list and
four CLI hashes remain unchanged. Formatting, all-target/all-feature Clippy,
workspace tests, and diff cleanliness pass. Task 262P is complete and Task
262Q is next. No `spec_coverage_audit.md` update is required because authority,
behavior, coverage credit, owner crate, and deferred status are unchanged.

## Task 262Q Pre-Move Inventory and Specification

Fresh test-sufficiency and source inventory identifies exactly three remaining
reserved-variable type-assertion source-core fragments for Task 262Q:

- the 13-line source transport, hash
  `1a8d06350de32059528b6af1240457874a323a24cb17cbedce128f560c50b00e`;
- the 121-line generic type-assertion extractor, hash
  `9334dbda0d88f8efbd75a7597471f08777df7f651761c132af4672034bcdf89e`;
- the 18-line type-assertion node allowlist, hash
  `2fd9587c78d740ffa0893baac5dfc18031ff43296e76bfa69819c2e2ba6b41d1`.

The 152 lines move from `runner.rs` into the existing private
`source_formula.rs` leaf with normalized equivalence. Only the source transport
and its ten fields plus the generic extractor receive runner-scoped visibility;
the allowlist remains leaf-private. The move uses only the formula leaf's
existing common source-AST, source-reserve, mode-expansion, and exact identifier
dependencies, so dependency direction remains acyclic.

All 58 concrete configs and named route wrappers, source-output transport,
checker/output conversion and validation, detail keys, diagnostics, and tests
remain in `runner.rs` unchanged. Existing base/object, local-mode, asserted-head,
two-through-six-hop, long-chain, exact/near-miss, source/range/ordinal/head/
provenance corruption, immutable-output, cross-route isolation, and real
frontend/resolver coverage includes 58 paired active `.miz` slices and 137
matching unit-test names. Fresh review must confirm that this matrix is
sufficient before implementation; if it finds a bounded preservation gap, a
separate test-only prerequisite task and commit must precede 262Q.

Completion requires all three normalized hashes, minimum visibility, preserved
accepted shape and exact asserted-head relation, unchanged fail-closed behavior
and 272-name/188-active inventories, unchanged plan/count/coverage/pass-fail/
CLI hashes, full Rust verification, diff cleanliness, and no-findings
implementation/source-doc review. No chain generalization, config/wrapper
split, rename, deduplication, semantic cleanup, or checker/output movement is
permitted.

## Task 262Q0 Test Repair Specification

The required fresh review classified a bounded `test_gap` before Task 262Q.
The existing base reserved-variable type-assertion test reaches the real source
extractor and checker handoff from a synthetic AST, but does not independently
fix all ten source transport fields and config identity, observes its 15 near
misses only through aggregate detail keys, and does not feed four already
modeled structural corruptions through the identifier-subject generic
extractor. The broader active family supplies the real frontend/resolver
sidecars, but another extraction route could still mask a generic-core
regression after the move.

Task 262Q0 is test-only. It makes the existing private identifier type-assertion
AST builder's corruption argument available through a default-off wrapper and
strengthens only
`source_reserved_variable_type_assertion_bridge_checks_reflexive_admissibility`.
The positive assertions must independently derive the formula, subject, and
asserted-type sites/ranges from the AST; fix the exact config identity and every
config field; and fix the reserve, spelling, ordinal, asserted type, and
distinct-range payload. Every existing near miss must directly return `None`
from the named extractor before also producing the aggregate extraction-gap
key. Four bounded identifier-route corruptions—recovered `is`, duplicate
formula expression, extra formula child, and extra assertion operand—must have
the same direct and aggregate rejection assertions.

Q0 adds no test and changes no production source, `.miz`, expectation, trace,
specification, public API, behavior, diagnostic, or coverage credit. Test name
and count hashes, active cases, repository counts, and CLI output must remain
unchanged. Task 262Q may proceed only after Q0 review and full verification.

## Task 262Q0 Test Repair Result

Task 262Q0 strengthened only the existing base reflexive-admissibility test and
its private default-off AST builder. The test now derives and fixes all ten
source transport fields from the AST, fixes all nine config values through the
named extractor route, and fixes the reserve payload, spelling, ordinal,
asserted type, and distinct source ranges. All 15 existing near misses now
directly reject through the named extractor before the aggregate gap check.
Recovered `is`, duplicate formula expression, extra formula child, and extra
assertion operand corruptions receive the same direct and aggregate rejection.

No production source, `.miz`, expectation, trace, specification, public API,
behavior, diagnostic, coverage credit, test name, or test count changed. All
272 unit tests and 188 active type cases pass. Plan/count remains 403/367, type
coverage 235/223, and pass/fail 219/184. The raw/normalized test-list and four
CLI hashes remain unchanged. Formatting, all-target/all-feature Clippy,
workspace tests, and diff cleanliness pass. Task 262Q0 is complete and
move-only Task 262Q is next. No `spec_coverage_audit.md` update is required
because authority, behavior, coverage credit, owner crate, and deferred status
are unchanged.

## Task 262Q Move Result

Task 262Q moved only the three inventoried reserved-variable type-assertion
source-core fragments into the existing private `source_formula.rs` leaf.
After stripping the added runner-scoped visibility, the transport and generic
extractor retain hashes `1a8d0635...` and `9334dbda...`; the leaf-private
allowlist retains hash `2fd9587c...` exactly. The transport, its ten fields, and
the generic extractor alone are runner-scoped. All 58 concrete configs and
named wrappers plus output/checker/validation/detail/diagnostic code remain in
`runner.rs` unchanged.

The resulting `runner.rs` has 12,144 lines and hash
`0454931d868a11b6cdfd90b845b8b091f2cd636add4fc8fb6c7aaf43a64cd6e4`;
the 40-line facade has hash
`a9f7b768ad32e6c51337f3b764db5243a80fc6cf2c16a7d97e57d1e99ef3a770`;
and the 2,621-line source-formula leaf has hash
`a7ffd9dad1e60a7e7890e494e9abc5bafb38e2f9cb11f62d14a03f617fe32b21`.
Imports and facade aliases made obsolete by the moved core were removed. Six
aliases retained solely for private tests are now explicitly `#[cfg(test)]`,
including the reserve-extraction guard entry used by its direct unit test.

All 272 unit tests and 188 active type cases pass. Plan/count remains 403/367,
type coverage 235/223, and pass/fail 219/184. The raw/normalized test-list and
four CLI hashes remain unchanged. Formatting, all-target/all-feature Clippy,
workspace tests, and diff cleanliness pass. Task 262Q is complete and Task 263
is next. No `spec_coverage_audit.md` update is required because authority,
behavior, coverage credit, owner crate, and deferred status are unchanged.

## Task 263A Pre-Move Inventory and Specification

Fresh Task 263 inventory classifies the retained checker-handoff substrate as
the first acyclic bounded family. The exact `runner.rs:11542-12047` fragment is
506 lines with hash
`95532967e13e1ab39b4ebc23c3403ffe15e57b5a73bda2810d915ccf170175f0`.
It starts at `source_module_binding_env` and ends at
`typing_for_type_entry`. It owns the empty module binding environment, the
`SourceReserveHandoff` transport, reserve declaration-to-`TypedAst` and
`ResolvedTypedAst` assembly, handoff validation, bounded Core context
readiness checks, and the test-only complete handoff entry.

Task 263A mechanically moves this fragment to new private
`type_elaboration/checker_handoff.rs`. The leaf depends only on checker, Core,
resolver, session, and syntax inputs, plus the sibling `SourceReserveExtraction`
transport for its test-only entry. It does not reference a concrete route
config, named source extractor, detail key, expected-output projection,
failure diagnostic, or top-level orchestration. This establishes the acyclic
direction `source_reserve -> checker_handoff -> retained checker/output and
orchestration consumers` before later Task 263 families move.

While those consumers remain in `runner.rs`, runner-scoped visibility is
limited to `source_module_binding_env`, `SourceReserveHandoff` and its four
fields, `assemble_source_reserve_checker_handoff`,
`assert_source_reserve_handoff`,
`assert_source_reserve_core_summary_readiness`, and
`assert_source_reserve_core_context_readiness`. The test-only
`assemble_source_checker_handoff` is exposed only under `#[cfg(test)]`.
Resolved/typed assembly and type-entry projection helpers remain leaf-private.

This is move-only `design_drift`; there is no test prerequisite. The existing
direct handoff test in `source_extraction.rs`, generic output validators and
corruption matrices, all 272 unit tests, and all 188 active type cases form the
preservation matrix. Configs, named wrappers, source extraction, payloads,
detail keys, diagnostics, ordering, fail-closed behavior, public API, tests,
and authority artifacts must not change. `spec_coverage_audit.md` remains
unchanged because behavior, coverage credit, owner crate, and deferred status
do not change.

## Task 263A Move Result

Task 263A moved the inventoried checker-handoff substrate to the new private
`type_elaboration/checker_handoff.rs` leaf. After stripping only the reviewed
runner-scoped visibility and restoring the former separator newline, the moved
fragment retains exact hash `95532967e13e1ab39b4ebc23c3403ffe15e57b5a73bda2810d915ccf170175f0`.
No body, control-flow branch, validation string, payload, or ordering changed.
The resolved/typed assembly and type-entry helpers remain leaf-private; the
complete handoff entry and its sibling source-reserve dependency remain
`#[cfg(test)]` only.

The resulting `runner.rs` has 11,617 lines and hash
`4c0aa87165f31fe66816666f1fc33f47b64643e7d644d30db21e8e8f4eb4ed8b`;
the 46-line phase facade has hash
`daf8415255a5af402436c792414c5fd635b32c5cf397deaff051efbfb16d7ece`;
and the 550-line checker-handoff leaf has hash
`a7cf9bcd076dbc68098ddecbab6c58eca988ecdd8ea378324bad44a32cf5288b`.
Only moved-only imports were removed from `runner.rs`. The existing
`SourceReserveDeclarationBridge` test namespace alias remains explicitly
`#[cfg(test)]` for the unchanged corruption tests.

The direct handoff test, all 272 unit tests, and all 188 active type cases pass.
Plan/count remains 403/367, type coverage 235/223, and pass/fail 219/184. The
raw/normalized test-list hashes and all four CLI byte hashes remain unchanged.
Formatting, all-target/all-feature Clippy, workspace tests, and diff cleanliness
pass. Task 263A is complete; fresh Task 263 inventory selects the next bounded
family. `spec_coverage_audit.md` remains unchanged because authority, behavior,
coverage credit, owner crate, and deferred status are unchanged.

## Task 263B Pre-Move Inventory and Specification

Fresh inventory selects the common frontend diagnostic projection as the next
acyclic bounded family. It consists of three exact `runner.rs` fragments: the
one-line recovery-tag constant at line 78, the seven-line
`frontend_detail_keys` fragment at lines 794-800 with hash
`394797911f19bd3904b4f66d8beed648d418bec9c6f172218f7e8912d21d2038`, and
the 41-line diagnostic-code/assertion/error fragment at lines 11528-11568 with
hash `ea3f9ffb0862e0a37575de150b82a3d654000778e87fa5abd0d9d41a40ff50a3`.
The recovery-tag hash in full is
`381e1d7f0e9ab985a0ce5436a8b6e19f63ca169da43f54c35fcfb42d68972b04`.
Their source-order
concatenation is 49 lines with hash
`0a4d80ff40dbf1d936ea0f5a965047e1a5f3a961812ede65deca56a8866a4ba5`.

Task 263B mechanically moves those fragments into existing private
`runner/shared.rs`, which already owns `FrontendRun` and directly imports
`FrontendDiagnostic` and `TestCase`; its frontend import adds `DiagnosticCode`.
The recovery tag and
`frontend_diagnostic_code` remain leaf-private. Only `frontend_detail_keys`,
`assertion_diagnostic_codes`, and `frontend_error_code` become parent-only
entries. Parse-only and declaration-symbol import their shared sibling
entries directly; `runner.rs` imports the shared detail-key entry for the
retained type consumer and removes its now-unused `DiagnosticCode` and
`FrontendDiagnostic` imports. This establishes `shared frontend/diagnostic projection -> phase
consumers` without a child-to-parent or checker dependency.

This is move-only `design_drift`; there is no Task 263B0 test prerequisite.
The active parse matrix including recovery-tag cases, declaration/type
lower-stage cases, active-runner byte-stability and repository execution tests,
and all four CLI projections already preserve code mapping, key prefixes,
ordering, and fallback behavior. No test, expectation, public API, diagnostic,
payload, source behavior, or authority artifact may change. No source file is
added, so the paired Source Inventory file list is unchanged.
`spec_coverage_audit.md` remains unchanged because coverage credit, owner crate,
and deferred status do not change.

## Task 263B Move Result

Task 263B moved the exact common frontend diagnostic family into existing
private `shared.rs`. After removing the three reviewed `pub(super)` modifiers,
folding whitespace, and normalizing rustfmt's optional trailing signature
commas, the old and moved families both hash to
`f7b793a4a93ec14cb24869c5de1e8b87ad35c79012185308c7ebaaf06d2f994b`.
The recovery tag and fallback mapper remain leaf-private. Parse-only and
declaration-symbol now import their shared entries directly; the retained type
consumer imports only `frontend_detail_keys` through the runner owner.

The resulting `runner.rs` has 11,566 lines and hash
`6cc0b8a7a70f4298761df02f1d8be755ba22416625cffd8e8fcf6d8660dc5f59`;
the 260-line `shared.rs` has hash
`1c5f780fbb0df10faf8f363594e5b19fbd7eb19abc852ece67308559141689b8`.
No diagnostic string, match arm, syntax/non-syntax branch, iteration order,
prefix, wildcard fallback, or frontend-error formatting changed.

All 272 unit tests and all 96 parse, four declaration-symbol, and 188
type-elaboration active cases pass. Plan/count remains 403/367, type coverage
235/223, and pass/fail 219/184. The raw/normalized test-list hashes and four
CLI byte hashes remain unchanged. Formatting, all-target/all-feature Clippy,
workspace tests, and diff cleanliness pass. Task 263B is complete; fresh Task
263 inventory selects the next bounded family. No Source Inventory or
`spec_coverage_audit.md` change is required because files, authority, behavior,
coverage credit, owner crate, and deferred status are unchanged.

## Task 263C Pre-Move Inventory and Specification

Fresh inventory selects the exact 24-line expected-result/failure-projection
family at `runner.rs:11512-11535`. It contains
`expected_type_elaboration_detail_keys` and
`type_elaboration_failure_diagnostic` and has raw hash
`b9efaec531ff58c52d028b413f8ea644640a5f0aeccaf57da3682cd7c5d1317c`.
Its only direct dependencies are `TestCase`, `ValidationDiagnostic`, and the
stable public `TypeElaborationCaseResult` DTO.

Task 263C mechanically moves the family into new private
`type_elaboration/result.rs`. Both functions become parent-only entries through
the type-elaboration facade; there are no other exports. The leaf's dependency
on the stable runner result DTO is an explicit facade-contract edge. The
payload-list-over-stable-key precedence, failure code/key/text, expected/actual
formatting, and vector order remain exact.

`run_type_elaboration_case` stays in `runner.rs` for this task because it still
depends on the large retained actual-detail dispatcher. Moving it now would
create a result-leaf-to-parent-private reverse edge or mix the later detail and
output families. Generic output validators also remain pending because their
current output/config/source-helper dependency graph requires a separate
bounded inventory.

This is move-only `design_drift`; there is no Task 263C0 prerequisite. Stable-
detail fallback tests, all 188 active type cases, repository/CLI byte-stability,
and normalized exact-body equivalence preserve result matching and failure
assembly. No test, expectation, diagnostic, API, payload, behavior, or authority
artifact may change. The new source path must be added to the paired Source
Inventory with the move. `spec_coverage_audit.md` remains unchanged because
coverage credit, owner crate, and deferred status do not change.

## Task 263C Move Result

Task 263C moved the exact 24-line expected-result/failure-projection family
into new private `type_elaboration/result.rs`. After removing only the two
required `pub(in crate::runner)` visibility qualifiers, the old and moved
bodies both hash to
`b9efaec531ff58c52d028b413f8ea644640a5f0aeccaf57da3682cd7c5d1317c`.
The facade exposes exactly those two entries parent-only. Case execution and
the actual-detail dispatcher remain in `runner.rs`, so the move introduces no
reverse dependency and changes no public API.

The resulting `runner.rs` has 11,541 lines and hash
`2e6bc713114f726af47de08d7ceb622f9d0f79282d00994be458f7f35e0c435e`;
the 50-line `type_elaboration.rs` facade has hash
`44634b3b24f645bbb49ea66c1569cf251c8f11db505c94de252877e9112c02cc`;
the new 29-line `result.rs` has hash
`608b458dd0d7491d7af1d6ef9261e468ec548b39966ecfa8acbc81bd8b7bd4c2`.
Payload-list precedence, stable-key fallback, failure code/key/text,
expected/actual formatting, vector order, and fail-closed caller flow are
unchanged.

All 272 unit tests and all 96 parse, four declaration-symbol, and 188
type-elaboration active cases pass. Plan/count remains 403/367, type coverage
235/223, and pass/fail 219/184. The raw/normalized test-list hashes and four
CLI byte hashes remain unchanged. The paired Source Inventory now includes the
new leaf. Task 263C is complete; fresh Task 263 inventory selects the next
bounded validation/detail family. No `spec_coverage_audit.md` change is
required because authority, behavior, coverage credit, owner crate, and
deferred status are unchanged.

## Task 263D Pre-Move Inventory and Specification

Fresh inventory selects the type-elaboration active-admission family as four
exact fragments: the one-line `ACTIVE_TYPE_ELABORATION_TAG` constant
(`4629969fa68b61384e96b345b2a646d786b6f843ca5ad128fa17723d473d68ac`),
the 13-line runnable predicate
(`5303e0c27405121d8aeefb7c6e2163dfcd288419c44b6e44779b1df4e0a41c9d`),
the six-line tag predicate
(`c91740986c91df19297de24f8c6f7441fed4886de246e18c65b5361e4a1fdd5b`),
and the 30-line gate validator
(`b0cb1652b4046473ce2bc12285ac09a69411c08d32b6a7144a501a9f27818945`).
Concatenated in source order, the exact 50 lines hash to
`937c032b2504225dbe5e79f5526545d969929afbd8dbfc9c09faf4cc5ad7a429`.

Task 263D mechanically moves this family into new private
`type_elaboration/admission.rs`. The tag constant and tag predicate remain
leaf-private. Only `is_active_type_elaboration` and
`validate_active_type_elaboration_tags` become parent-only entries through the
type-elaboration facade. The public `active_type_elaboration_cases` iterator
and corpus-level orchestration remain in `runner.rs`. Direct dependencies are
only `ValidationDiagnostic`, `ExpectedOutcome`, `PipelinePhase`, `TestCase`,
`TestPlan`, and `Stage`; the leaf has no source, checker, output, or parent DTO
dependency.

This is move-only `design_drift`; there is no Task 263D0 prerequisite. Existing
non-type, wrong-phase, and public-diagnostic-code gate tests directly preserve
the gate branches, diagnostic codes, and silent-skip rejection. Normalized
exact-body equivalence plus repository/report/CLI byte-stability preserve
detail keys, text, ordering, and iteration behavior across all 188 active
cases. No test, expectation,
diagnostic, API, payload, behavior, or authority artifact may change. The new
source path must be added to the paired Source Inventory with the move.
`spec_coverage_audit.md` remains unchanged because coverage credit, owner
crate, and deferred status do not change.

## Task 263D Move Result

Task 263D moved the exact four-fragment 50-line type active-admission family
into new private `type_elaboration/admission.rs`. After removing only the two
required `pub(in crate::runner)` visibility qualifiers, folding ASCII
whitespace, and normalizing only rustfmt's trailing `TestPlan` signature comma,
the old and moved families both hash to
`ea1a50947f895bcbc5bcca417432b3860369174677ea9b8b4b7626ca651157c4`.
The tag constant and tag predicate remain leaf-private; the facade exposes
exactly the runnable predicate and gate validator parent-only. The public
iterator and corpus orchestration remain in `runner.rs`.

The resulting `runner.rs` has 11,490 lines and hash
`5d58dcfe62d1d724a731f5421ad6547d7e8e7757581297efe7b6a000adec2230`;
the 52-line `type_elaboration.rs` facade has hash
`b06293cc471453df1bb373a53b51cbba2d8b3991ec5206c5b0ecd719047839e7`;
the new 60-line `admission.rs` has hash
`b5261a23dae29eb656ba6f414a622a4cc40501dabd0fcf457fedf53b23aba150`.
Admission branches, diagnostic codes/keys/text, per-case diagnostic ordering,
and silent-skip rejection are unchanged.

All three focused gate tests, all 272 unit tests, and all 96 parse, four
declaration-symbol, and 188 type-elaboration active cases pass. Plan/count
remains 403/367, type coverage 235/223, and pass/fail 219/184. The
raw/normalized test-list hashes and four CLI byte hashes remain unchanged. The
paired Source Inventory includes the new leaf. Task 263D is complete; fresh
Task 263 inventory selects the next bounded validation/detail family. No
`spec_coverage_audit.md` change is required because authority, behavior,
coverage credit, owner crate, and deferred status are unchanged.

## Task 263E Pre-Move Inventory and Specification

Fresh dependency inventory selects the exact contiguous 33-line checker-output
transport substrate at `runner.rs:5361-5393`: the binary-formula,
parenthesized-binary-formula, and type-assertion output structs with 22 fields.
The raw family hash is
`e5da36674f0779384d90fa35a7f42ee209dfbca2049efe76b2893c0b36705ce0`.
These are real transports with 153 production/test type references and 346
named output/validator test references, not an empty or synthetic owner.

Task 263E mechanically moves only these three transports into new private
`type_elaboration/output.rs`. The three types and their 22 fields receive the
minimum `pub(in crate::runner)` visibility required by retained builders,
validators, named output helpers, and the existing corruption tests; the
type-elaboration facade re-exports only the three types parent-only. Direct
dependencies are the sibling source payload transports and checker handoff,
plus checker/session typed inputs and inference output. Builders, validators,
detail projections, named wrappers, configs, and all call sites remain in
`runner.rs` for separate bounded tasks.

This is a move-only `design_drift` prerequisite for later output-owner tasks;
there is no Task 263E0 prerequisite. Existing field-level payload and
corruption matrices plus normalized exact-body equivalence preserve every
field, type, order, and debug shape. The 272-test list, all 188 active cases,
repository/report/CLI byte-stability, and full gates preserve API, payload,
diagnostic, ordering, and fail-closed behavior. No test, expectation, semantic
helper, source behavior, or authority artifact may change. The new source path
must be added to the paired Source Inventory with the move.
`spec_coverage_audit.md` remains unchanged because coverage credit, owner
crate, and deferred status do not change.

## Task 263E Move Result

Task 263E moved the exact 33-line three-transport checker-output substrate
into new private `type_elaboration/output.rs`. After removing only the 25
required `pub(in crate::runner)` qualifiers from the three types and 22 fields,
the moved lines 11-43 reproduce the original raw hash byte-for-byte:
`e5da36674f0779384d90fa35a7f42ee209dfbca2049efe76b2893c0b36705ce0`.
The facade re-exports exactly the three transports parent-only. Builders,
validators, detail projections, named wrappers, configs, and call sites remain
in `runner.rs`; only the now-unused parent `SourceRange` and
`SourceReserveHandoff` imports were removed.

The resulting `runner.rs` has 11,457 lines and hash
`d43d0f6a62cff726fffc88ebe2452932371626a71a9e13aa9bae09eb8168708e`;
the 57-line `type_elaboration.rs` facade has hash
`0c068fd8a7bca6f7d0194e06cda9723eb0bfe8d39b1bc3d3c6553c5a6cb61c86`;
the new 43-line `output.rs` has hash
`bb056c40bdafeb2d3f60821da8cf4fa908045b16dc0230defbff85bc27bdb350`.
Derives, field types and order, debug shape, payloads, and fail-closed behavior
are unchanged.

The four focused output tests, all 272 unit tests, and all 96 parse, four
declaration-symbol, and 188 type-elaboration active cases pass. Plan/count
remains 403/367, type coverage 235/223, and pass/fail 219/184. The
raw/normalized test-list hashes and four CLI byte hashes remain unchanged. The
paired Source Inventory includes the new leaf. Task 263E is complete; fresh
Task 263 inventory selects the next bounded builder/validator/detail family.
No `spec_coverage_audit.md` change is required because authority, behavior,
coverage credit, owner crate, and deferred status are unchanged.

## Task 263F Pre-Move Inventory and Specification

Fresh dependency inventory selects the three contiguous checker-output
builders at `runner.rs:8441-8701` (261 lines, hash
`cb4396e080d9f31f79e57feebfd5de5badad92f3aedfdf358b0eb277eb416b25`)
and their sole 16-line `source_reserved_type_projection` helper at
`runner.rs:9473-9488` (hash
`c450e8588af637f3f3a8dc04f522ef988dc470a54b4d005001c4ba5f102f33b0`).
Concatenated in source order, the exact 277-line producer family hashes to
`b4939bbe52118a6b6e1d268bff26c6fa11e2994e14e0bb0b4e7215e94a41efaa`.

Task 263F mechanically moves this family into the existing private
`type_elaboration/output.rs` transport owner. The type-assertion, binary-
formula, and parenthesized-binary builder entries become parent-only through
the type-elaboration facade; the projection helper remains leaf-private. Their
direct dependencies are the Task 263E output transports, sibling source
payload/config transports, sibling checker-handoff assembly, resolver symbols,
and checker binding/type/formula input APIs. The family has no dependency on
retained validators, detail projections, named wrappers, active orchestration,
or public result DTOs.

This is move-only `design_drift`; there is no Task 263F0 prerequisite. Existing
source-output, field-provenance, lookup-ordinal, checker-payload, corruption,
and active fixture matrices already execute all three builders and their
fail-closed branches. Exact-body equivalence plus 272-test, 188-active-case,
repository/report/CLI byte-stability, and full gates preserve construction
order, error strings, source ranges, binding identities, inputs, payloads, and
failure boundaries. No test, expectation, validator, detail key, config,
semantic behavior, or authority artifact may change. The Source Inventory is
unchanged because the existing `output.rs` owner is extended.
`spec_coverage_audit.md` remains unchanged because coverage credit, owner
crate, and deferred status do not change.

## Task 263F Move Result

Task 263F moved the exact three-builder and sole projection-helper producer
family into existing private `type_elaboration/output.rs`. After removing only
the three required `pub(in crate::runner)` builder qualifiers, moved lines
51-311 retain hash
`cb4396e080d9f31f79e57feebfd5de5badad92f3aedfdf358b0eb277eb416b25`;
the private helper at lines 313-328 retains hash
`c450e8588af637f3f3a8dc04f522ef988dc470a54b4d005001c4ba5f102f33b0`;
their source-order combination retains
`b4939bbe52118a6b6e1d268bff26c6fa11e2994e14e0bb0b4e7215e94a41efaa`.
The facade exposes exactly the three builders parent-only. Validators, detail
projections, named wrappers, configs, and call sites remain in `runner.rs`.

The resulting `runner.rs` has 11,180 lines and hash
`cfefc3b316fe7d9ff33153475ed42540fcf8605a16ad11132f4380c7ca0350a7`;
the 60-line `type_elaboration.rs` facade has hash
`c673946fddb223a2ae566073205bffaac56ce34ccbb393ae0e755ad6d5c15658`;
the 328-line `output.rs` has hash
`41a151db0d3e6fc4ba45c04989e1bbf577cfc4a8ae55ba9d570998794c90bbcd`.
Construction order, error text, source ranges, binding identities, checker
inputs, payloads, and fail-closed flow are unchanged.

The four focused builder tests, all 272 unit tests, and all 96 parse, four
declaration-symbol, and 188 type-elaboration active cases pass. One initial
full-crate run encountered a transient unrelated missing `/tmp` fixture path;
the exact failing route-isolation test and the full crate rerun both pass.
Plan/count remains 403/367, type coverage 235/223, and pass/fail 219/184. The
raw/normalized test-list hashes and four CLI byte hashes remain unchanged.
Task 263F is complete; fresh Task 263 inventory selects the next bounded
validator/detail family. Source Inventory and `spec_coverage_audit.md` remain
unchanged because no path, authority, behavior, coverage credit, owner crate,
or deferred status changed.

## Task 263G Pre-Move Inventory and Specification

Fresh dependency inventory selects the exact type-assertion output validator
and its private role-entry helper at `runner.rs:8443-8656` (214 lines, hash
`17ad7203816094ef55580f9356388510e6164cdc2f4a38412639d496db1b623c`),
plus the shared normalized-builtin-type predicate at `runner.rs:9197-9211`
(15 lines, hash
`c1e417207bcc04654fdeb3fee13a00985a5aff63181298d1b65d149d3d6f15aa`).
Concatenated in source order, the exact 229-line family hashes to
`b6557af65c99430f112772b665c36a3545bdb39f48541e1c817f06eadfc0b10f`.

Task 263G mechanically moves this family into existing private
`type_elaboration/output.rs`. The type-assertion validator becomes
parent-only, its role-entry helper remains leaf-private, and the normalized-
type predicate is temporarily parent-only because the retained binary-formula
validator has two existing call sites. The family depends only on the Task
263E/F output and checker-handoff owners, exact source-formula predicates, and
checker typed-output APIs. Binary and parenthesized validators, detail-key
projection, named wrappers, configs, call sites, and orchestration remain in
`runner.rs`.

This is move-only `design_drift`; there is no Task 263G0 prerequisite. The
production detail-result path and 212 direct validator assertions across ten
existing test modules cover exact success, provenance, lookup ordinal,
checker counts and identities, canonical source, corruption rejection,
route isolation, and fail-closed behavior. Exact-body equivalence plus the
272-test, 188-active-case, repository/report/CLI byte-stability, and full gates
must preserve every error string, comparison, ordering decision, detail key,
payload, and failure boundary. No test, expectation, config, wrapper,
validator logic, semantic behavior, or authority artifact may change. Source
Inventory and `spec_coverage_audit.md` remain unchanged because the existing
`output.rs` path is extended without changing coverage credit, owner crate, or
deferred status.

## Task 263G Move Result

Task 263G moved the exact type-assertion validator, private role-entry helper,
and shared normalized-type predicate into existing private
`type_elaboration/output.rs`. After removing only the two required
`pub(in crate::runner)` qualifiers, moved lines 337-550 retain hash
`17ad7203816094ef55580f9356388510e6164cdc2f4a38412639d496db1b623c`,
lines 552-566 retain hash
`c1e417207bcc04654fdeb3fee13a00985a5aff63181298d1b65d149d3d6f15aa`,
and their source-order combination retains
`b6557af65c99430f112772b665c36a3545bdb39f48541e1c817f06eadfc0b10f`.
The facade exposes only the validator and temporarily shared predicate
parent-only; the role helper remains leaf-private. The retained binary
validator still has exactly its two original predicate calls.

The resulting `runner.rs` has 10,948 lines and hash
`97247c5bedcee1baebaec2f5caae2d332dea5de246e18671992db4ddbc64e2aa`;
the 61-line `type_elaboration.rs` facade has hash
`c36560ef2972e383d2a0d59aa1021fb8341d0bfbf8c79ebded0e1dbc16d2df0c`;
the 566-line `output.rs` has hash
`01c75f7906b759308c9c52f36768dbd46b1d3f8fd462507bc448f538601224d5`.
Every validation branch, error string, comparison, checker lookup, normalized-
type identity check, canonical-source check, and fail-closed return is
unchanged.

All 47 focused type-assertion tests, all 272 unit tests, and all 96 parse, four
declaration-symbol, and 188 type-elaboration active cases pass. Plan/count
remains 403/367, type coverage 235/223, and pass/fail 219/184. The 272-line
raw/normalized test-list hashes and four CLI byte hashes remain unchanged.
Task 263G is complete; fresh Task 263 inventory selects the next bounded
binary/parenthesized-validator or detail family. Source Inventory and
`spec_coverage_audit.md` remain unchanged because no path, authority,
behavior, coverage credit, owner crate, or deferred status changed.

## Task 263H Pre-Move Inventory and Specification

Fresh dependency inventory selects the exact binary-formula output validator
at `runner.rs:8442-8779` (338 lines, hash
`528876adb6cda98d2030df317d2589733799917682f9bdcf0d04f1333ff46ddf`),
its source-type projection predicate at `runner.rs:8937-8953` (17 lines, hash
`4317df8d93687b6357cc2f5943cd4c1b248fc69e2092c02586588c00bfa40170`),
and its type-entry validator at `runner.rs:8955-8979` (25 lines, hash
`135354e0b3aa68dbd5435a869134722b2617b0e65faea16810ff9a3ad657f43e`).
Concatenated in source order, the exact 380-line family hashes to
`76fcab1f8c068b9b0ee0bd552b106e9a23cce794e7ff0f9134120e2285de7836`.

Task 263H mechanically moves this family into existing private
`type_elaboration/output.rs`. The binary validator becomes parent-only for the
retained production detail and parenthesized-validator consumers; both helper
predicates remain leaf-private. Its two calls to the Task 263G normalized-type
predicate move with it, so that predicate narrows from temporary parent-only
to leaf-private and disappears from the facade. Parenthesized validators,
detail projection, named wrappers, configs, call sites, and orchestration
remain in `runner.rs`.

This is move-only `design_drift`; there is no Task 263H0 prerequisite. The
production detail path, retained parenthesized consumer, and 104 direct
validator assertions across eleven existing test modules cover exact success,
binding/provenance/ordinal identity, expected/result constraints, checker
counts and order, semantic type sharing, canonical sources, corruption
rejection, route isolation, and fail-closed behavior. Exact-body equivalence
plus the 272-test, 188-active-case, repository/report/CLI byte-stability, and
full gates must preserve every error string, comparison, ordering decision,
detail key, payload, and failure boundary. No test, expectation, config,
wrapper, validator logic, semantic behavior, or authority artifact may
change. Source Inventory and `spec_coverage_audit.md` remain unchanged because
the existing `output.rs` path is extended without changing coverage credit,
owner crate, or deferred status.

## Task 263H Move Result

Task 263H moved the exact binary-formula validator and its two private helpers
into existing private `type_elaboration/output.rs`. After removing only the
required validator `pub(in crate::runner)` qualifier, moved lines 570-907
retain hash
`528876adb6cda98d2030df317d2589733799917682f9bdcf0d04f1333ff46ddf`,
lines 909-925 retain hash
`4317df8d93687b6357cc2f5943cd4c1b248fc69e2092c02586588c00bfa40170`,
lines 927-951 retain hash
`135354e0b3aa68dbd5435a869134722b2617b0e65faea16810ff9a3ad657f43e`,
and their source-order combination retains
`76fcab1f8c068b9b0ee0bd552b106e9a23cce794e7ff0f9134120e2285de7836`.
Only the validator is parent-only. Both helpers and the normalized-type
predicate are leaf-private; the temporary normalized predicate facade alias
is gone. Parenthesized validators, detail projections, configs, wrappers, and
call sites remain in `runner.rs`.

The resulting `runner.rs` has 10,558 lines and hash
`2440c1f2cce788ed4f58437338124639f36327b88572105b4b3a80c4e4679446`;
the 62-line `type_elaboration.rs` facade has hash
`1ff372989d8ccce83ce68645ac054e245ec4c85f90cf1c2919fb56fac3c8216f`;
the 951-line `output.rs` has hash
`2fe4650c4be3c5560ab991278dcc701e32581c75b8ab7429c90d95ccc86a9689`.
Every validation branch, error string, collection order, lookup, expected and
result constraint, semantic identity check, canonical-source choice, and
fail-closed return is unchanged.

All 123 focused reserved-variable tests, all 272 unit tests, and all 96 parse,
four declaration-symbol, and 188 type-elaboration active cases pass.
Plan/count remains 403/367, type coverage 235/223, and pass/fail 219/184. The
272-line raw/normalized test-list hashes and four CLI byte hashes remain
unchanged. Task 263H is complete; fresh Task 263 inventory selects the next
bounded parenthesized-validator or detail family. Source Inventory and
`spec_coverage_audit.md` remain unchanged because no path, authority,
behavior, coverage credit, owner crate, or deferred status changed.

## Task 263I Pre-Move Inventory and Specification

Fresh dependency inventory selects only the config-independent shared
parenthesized-binary validator core at `runner.rs:8523-8589` (67 lines, hash
`2de635a7524ac3734edb15c9d953dab6fc73b4800f5d3804866c0cffba7e5d88`).
It is the sole parenthesized wrapper/range/typed-output isolation predicate
and calls the Task 263H binary validator directly.

Task 263I mechanically moves this exact core into existing private
`type_elaboration/output.rs` with one parent-only entry for the retained eight
named test wrappers and production detail consumer. Concrete configs, named
validators, detail projection, output wrappers, call sites, and orchestration
remain in `runner.rs`; moving them together would cross the config-independent
core boundary.

This is move-only `design_drift`; there is no Task 263I0 prerequisite. The
eight named wrappers have 16 direct assertions across two existing test
modules, while the active/report detail path exercises the production
consumer. Those tests cover left/right wrapper sides, config identity, source
and copied wrapper site/range equality, distinct typed sites, source-id/range
containment, exclusion from terms/type entries/formulas, corruption rejection,
route isolation, and fail-closed behavior. Exact-body equivalence plus the
272-test, 188-active-case, repository/report/CLI byte-stability, and full gates
must preserve every error string, comparison, ordering decision, detail key,
payload, and failure boundary. No test, expectation, config, wrapper logic,
semantic behavior, or authority artifact may change. Source Inventory and
`spec_coverage_audit.md` remain unchanged because the existing `output.rs`
path is extended without changing coverage credit, owner crate, or deferred
status.

## Task 263I Move Result

Task 263I moved only the exact config-independent parenthesized-binary
validator core into existing private `type_elaboration/output.rs`. After
removing the required `pub(in crate::runner)` qualifier, moved lines 954-1020
retain hash
`2de635a7524ac3734edb15c9d953dab6fc73b4800f5d3804866c0cffba7e5d88`.
The facade exposes this one validator parent-only. All eight named validators,
concrete configs, detail projections, output wrappers, and call sites remain
in `runner.rs`; the leaf imports only the generic config type.

The resulting `runner.rs` has 10,491 lines and hash
`3d75554d7cc1c45b5cdbab06ce27a30bd660cb01a4cd5e9311157442c5a43205`;
the 63-line `type_elaboration.rs` facade has hash
`dfd15b3390d53dd6c84decf0babb117077e53ce400b4325126757faff3061453`;
the 1,020-line `output.rs` has hash
`0c18a5d1244da77a85d73368d622dda2699b95463b015e980cd98604b79a6a16`.
Wrapper-side selection, pointer identity, source/copy site and range checks,
containment, typed-output exclusion, error text, and fail-closed return are
unchanged.

All 25 focused parenthesized tests, all 272 unit tests, and all 96 parse, four
declaration-symbol, and 188 type-elaboration active cases pass. Plan/count
remains 403/367, type coverage 235/223, and pass/fail 219/184. The 272-line
raw/normalized test-list hashes and four CLI byte hashes remain unchanged.
Task 263I is complete; fresh Task 263 inventory selects the next bounded
detail or config/named-wrapper family. Source Inventory and
`spec_coverage_audit.md` remain unchanged because no path, authority,
behavior, coverage credit, owner crate, or deferred status changed.

## Task 263J Pre-Move Inventory and Specification

Fresh dependency inventory selects the exact type-assertion result/detail core
at `runner.rs:6973-7018` (46 lines, hash
`3d4f7e8ce9ff1b60d0960e33fb8e1689fb4862a1730cf3144137e720db053fb8`).
The public-to-parent result projection has 125 existing consumers; its output
diagnostic collector is used only inside the selected core.

Task 263J mechanically moves this family into existing private
`type_elaboration/output.rs`. The result projection becomes parent-only for
retained named detail wrappers; the collector remains leaf-private beside the
Task 263G validator and output transport. Binary and parenthesized detail
cores, configs, named wrappers, output wrappers, call sites, and orchestration
remain in `runner.rs`.

This is move-only `design_drift`; there is no Task 263J0 prerequisite. Existing
direct matrices across eight test modules plus active/report consumers cover
validator-first rejection, invalid-key fallback, binding/declaration/formula
diagnostic collection, checker prefixing, canonical iteration, sort/dedup,
empty success, corruption rejection, and fail-closed behavior. Exact-body
equivalence plus the 272-test, 188-active-case, repository/report/CLI byte-
stability, and full gates must preserve every key, fallback, ordering decision,
payload, and failure boundary. No test, expectation, config, wrapper logic,
semantic behavior, or authority artifact may change. Source Inventory and
`spec_coverage_audit.md` remain unchanged because the existing `output.rs`
path is extended without changing coverage credit, owner crate, or deferred
status.

## Task 263J Move Result

Task 263J moved only the exact type-assertion result/detail core into existing
private `type_elaboration/output.rs`. After removing the required
`pub(in crate::runner)` qualifier, moved lines 536-581 retain hash
`3d4f7e8ce9ff1b60d0960e33fb8e1689fb4862a1730cf3144137e720db053fb8`.
The result projection is parent-only and the diagnostic collector is
leaf-private. The facade and runner now gate the direct validator/output type
aliases test-only because production detail consumers use only the result
projection. Binary/parenthesized detail, configs, named/output wrappers, and
call sites remain in `runner.rs`.

The resulting `runner.rs` has 10,444 lines and hash
`66bda6fe475617e30298b8dfb9384b92d55a033a23ee11726ada2e8ba9e6a8c2`;
the 68-line `type_elaboration.rs` facade has hash
`5a2412bfbf81a7505ccc03d68a12266a9ce5ec238247ed2c583c5cf08666ec4a`;
the 1,067-line `output.rs` has hash
`0afb49bbd16b8eb320e70d6997818302290cf1352fefe0b2c7ad3a3a2e9be1df`.
Validator-first rejection, fallback selection, diagnostic sources and prefix,
canonical iteration, sort/dedup, empty success, and fail-closed behavior are
unchanged.

All 47 focused type-assertion tests, all 272 unit tests, and all 96 parse, four
declaration-symbol, and 188 type-elaboration active cases pass. Plan/count
remains 403/367, type coverage 235/223, and pass/fail 219/184. The 272-line
raw/normalized test-list hashes and four CLI byte hashes remain unchanged.
Task 263J is complete; fresh Task 263 inventory selects the next bounded
binary/parenthesized detail or config/named-wrapper family. Source Inventory
and `spec_coverage_audit.md` remain unchanged because no path, authority,
behavior, coverage credit, owner crate, or deferred status changed.

## Task 263K Pre-Move Inventory and Specification

Fresh dependency inventory selects the exact binary-formula result/detail core
at `runner.rs:6973-7008` (36 lines, hash
`be8659f6d1bd22caba5270f0ea180521a90375e8b37c8f1a7b9e8f0cb4068b37`).
The result projection has 52 retained production consumers. Its diagnostic
collector is also consumed by the retained shared parenthesized-detail core.
Six test modules contain 145 direct references to the two entries, excluding
the two shared-support imports.

Task 263K mechanically moves only these two functions into the existing
private `type_elaboration/output.rs`. Both entries become parent-only because
retained runner consumers still call them. Once the production collector moves
beside its binary validator, the facade and runner gate the direct validator
and output-type aliases test-only. Parenthesized detail, every config and
named/output wrapper, and all call sites remain in `runner.rs`.

This is move-only `design_drift`; there is no Task 263K0 prerequisite. Existing
direct result/output matrices cover validator-first rejection, configured
invalid-key fallback, declaration and formula diagnostic sources, checker-key
prefixing, canonical iteration, sort/dedup, and empty success. The preservation
matrix is exact function-body equivalence after removing required visibility,
unchanged stable keys and diagnostic payload order, unchanged 272-test raw and
normalized lists, unchanged 188 active type cases and plan/count bytes, and no
`.miz`, expectation, trace, spec, API, config, wrapper, or call-site edits.

The authority inventory finds no canonical contradiction: `doc/spec/en`, the
existing `.miz` corpus, `spec_trace.toml`, and expectations retain their prior
intent; `harness.md` and `expectation_schema.md` continue to define the active
runner and deterministic detail contract; source is only the derived layout
being repaired. Source Inventory and `spec_coverage_audit.md` remain unchanged
because the existing `output.rs` path, authority, behavior, credit, owner crate,
and deferred status do not change.

## Task 263K Move Result

Task 263K moved only the exact binary-formula result/detail core into existing
private `type_elaboration/output.rs`. After removing the two required
`pub(in crate::runner)` qualifiers, moved lines 957-992 retain hash
`be8659f6d1bd22caba5270f0ea180521a90375e8b37c8f1a7b9e8f0cb4068b37`.
Both projections are parent-only. The first non-test crate build identified
that the direct binary output-type alias, as well as the validator alias, had
become test-only; the paired inventory was refined and both facade/runner
aliases are now `#[cfg(test)]`. Parenthesized detail, every config and
named/output wrapper, and all call sites remain in `runner.rs`.

The resulting `runner.rs` has 10,411 lines and hash
`bc7d9d3dc7536c8311eb9b7c5c6131657114ad1b3bdc2f5a3b13149642ccc1b3`;
the 69-line `type_elaboration.rs` facade has hash
`3411dfac21ea4872bdbea24466a64c7cdaafc27c54828b397913f483ed00e2e7`;
the 1,104-line `output.rs` has hash
`94a0aa92cacfacf2ef32bc0b5b8e336f7340c19a3bcc4ef505052e568b3b69e9`.
Builder-error fallback, validator-first rejection, configured invalid-key
fallback, declaration/formula diagnostic sources, checker-key prefix,
canonical iteration, sort/dedup, empty success, and fail-closed behavior are
unchanged.

All four focused source-reserved-variable tests, all 272 unit tests, relevant
crate integration tests, and all 96 parse, four declaration-symbol, and 188
type-elaboration active cases pass. Plan/count remains 403/367, type coverage
235/223, pass/fail 219/184, and warnings/errors 23/0. The raw/normalized test-
list hashes and four CLI byte hashes remain unchanged. Formatting,
all-target/all-feature Clippy with warnings denied, workspace tests, and diff
cleanliness pass. Task 263K is complete; fresh Task 263 inventory selects the
next bounded parenthesized-detail or config/named-wrapper family. Source
Inventory and `spec_coverage_audit.md` remain unchanged because no path,
authority, behavior, coverage credit, owner crate, or deferred status changed.

## Task 263L Pre-Move Inventory and Specification

Fresh dependency inventory selects the exact shared parenthesized-binary
output-detail core at `runner.rs:7065-7080` (16 lines, hash
`700b2283f7a6ea7b61c97ec59a27166404a72eccdce8f8e7aa7c681dd9003e47`).
Its sole production caller is the retained payload-detail wrapper serving all
eight parenthesized active routes. Eight retained test-only named wrappers call
the same core, with 26 direct assertions in `reserved_binary.rs`.

Task 263L mechanically moves only this shared core into the existing private
`type_elaboration/output.rs`. It becomes parent-only for the retained payload
and named-wrapper callers. Once it is beside the parenthesized validator, the
facade and runner gate the direct parenthesized validator/output-type aliases
and direct binary detail-collector alias test-only. The payload-detail wrapper,
all eight configs, all named detail/validator/output wrappers, and every call
site remain in `runner.rs`.

This is move-only `design_drift`; there is no Task 263L0 prerequisite. Existing
active and direct matrices cover builder fallback at the retained caller,
validator-first wrapper rejection, configured invalid-key fallback, nested
binary diagnostic projection, left/right wrapper identity, and fail-closed
behavior. The preservation matrix is exact function-body equivalence after
removing required visibility, unchanged 272-test raw and normalized lists,
unchanged active and CLI bytes, and no `.miz`, expectation, trace, spec, API,
config, wrapper, payload-detail, or call-site edits.

The authority inventory finds no canonical contradiction. The existing
`output.rs` path and owner are unchanged, and authority, behavior, coverage
credit, and deferred status do not change, so Source Inventory and
`spec_coverage_audit.md` remain unchanged.

## Task 263L Move Result

Task 263L moved only the exact shared parenthesized-binary output-detail core
into existing private `type_elaboration/output.rs`. After removing the required
`pub(in crate::runner)` qualifier, moved lines 1106-1121 retain hash
`700b2283f7a6ea7b61c97ec59a27166404a72eccdce8f8e7aa7c681dd9003e47`.
The shared core is parent-only. The first non-test build identified that the
direct binary detail-collector alias also became test-only once the
parenthesized core moved; the paired inventory was refined, and that alias plus
the direct parenthesized validator/output-type aliases are now `#[cfg(test)]`.
The payload-detail wrapper, eight configs, named detail/validator/output
wrappers, and all call sites remain in `runner.rs`.

The resulting `runner.rs` has 10,395 lines and hash
`46338bc436d6fac02ed5ecd33ef454bed44e4ea8ed55427723e0781be0fadd44`;
the 70-line `type_elaboration.rs` facade has hash
`720cecb3656838d7b2362db0c8c37a5fbc836d9e5b40e7713aa418ebe42b2576`;
the 1,121-line `output.rs` has hash
`c07eec9a8e118462998ac9d99e0c983ed140bf1197c3bfd3125a0ed2a34c70c3`.
Builder fallback remains in the retained caller, while validator-first
rejection, configured fallback, nested binary detail projection, left/right
wrapper identity, and fail-closed behavior are unchanged in the moved core.

All 25 focused parenthesized tests, all 272 unit tests, relevant crate
integration tests, and all 96 parse, four declaration-symbol, and 188 type-
elaboration active cases pass. Plan/count remains 403/367, type coverage
235/223, pass/fail 219/184, and warnings/errors 23/0. The raw/normalized test-
list hashes and four CLI byte hashes remain unchanged. Formatting,
all-target/all-feature Clippy with warnings denied, workspace tests, and diff
cleanliness pass. Task 263L is complete; fresh Task 263 inventory selects the
remaining payload-detail or config/named-wrapper family. Source Inventory and
`spec_coverage_audit.md` remain unchanged because no path, authority, behavior,
coverage credit, owner crate, or deferred status changed.

## Task 263M Pre-Move Inventory and Specification

Fresh dependency inventory selects the exact parenthesized-binary payload-
detail wrapper at `runner.rs:5508-5524` (17 lines, hash
`5807184d2ce9cfa8f7fb5a9be4d8401b8a538a335f28d07768a251840a169605`).
Eight production route wrappers and eight direct assertions in
`reserved_binary.rs` call this entry.

Task 263M mechanically moves only this wrapper into the existing private
`type_elaboration/output.rs`. It becomes parent-only for the retained route and
test callers. The builder and shared output-detail core become leaf-internal for
production, so their facade and runner aliases become test-only. All eight
configs, named route/detail/validator/output wrappers, named extractors, and
every call site remain in `runner.rs`.

This is move-only `design_drift`; there is no Task 263M0 prerequisite. Existing
active and direct matrices cover builder-error fallback, configured invalid-key
selection, validator-first rejection, nested binary diagnostics, side identity,
and fail-closed behavior. The preservation matrix is exact function-body
equivalence after removing required visibility, unchanged 272-test lists and
active/CLI bytes, and no `.miz`, expectation, trace, spec, API, config, wrapper,
extractor, or call-site edits.

The authority inventory finds no canonical contradiction. The existing
`output.rs` path and owner are unchanged, and authority, behavior, coverage
credit, and deferred status do not change, so Source Inventory and
`spec_coverage_audit.md` remain unchanged.

## Task 263M Move Result

Task 263M moved only the exact parenthesized-binary payload-detail wrapper into
existing private `type_elaboration/output.rs`. After removing the required
`pub(in crate::runner)` qualifier, moved lines 1123-1139 retain hash
`5807184d2ce9cfa8f7fb5a9be4d8401b8a538a335f28d07768a251840a169605`.
The wrapper is parent-only. The direct parenthesized builder and shared-detail
aliases are now test-only, while the configs, named route/detail/validator/
output wrappers, named extractors, and all call sites remain in `runner.rs`.

The resulting `runner.rs` has 10,377 lines and hash
`ee5ce9753442a91cea9642c32941f0bda71f05c956ad13b49d36d90d17639e35`;
the 71-line `type_elaboration.rs` facade has hash
`62bd63a6aaaac7fbf83f8783b90bfa4546dfab99308b4ff420fd66803ebc9678`;
the 1,139-line `output.rs` has hash
`55fd0eae01f417d011a3800d532f65eed1a2fd76d60d7387e9630fe3d9c92e57`.
Builder-error fallback, configured invalid-key selection, success delegation,
nested binary diagnostics, side identity, and fail-closed behavior are
unchanged.

All 25 focused parenthesized tests, all 272 unit tests, relevant crate
integration tests, and all 96 parse, four declaration-symbol, and 188 type-
elaboration active cases pass. Plan/count remains 403/367, type coverage
235/223, pass/fail 219/184, and warnings/errors 23/0. The raw/normalized test-
list hashes and four CLI byte hashes remain unchanged. Formatting,
all-target/all-feature Clippy with warnings denied, workspace tests, and diff
cleanliness pass. Task 263M is complete; fresh Task 263 inventory selects the
remaining config/named-wrapper family. Source Inventory and
`spec_coverage_audit.md` remain unchanged because no path, authority, behavior,
coverage credit, owner crate, or deferred status changed.

## Task 263N Pre-Move Inventory and Specification

Fresh dependency inventory selects a cohesive private parenthesized route
owner made of seven exact `runner.rs` fragments: invalid keys at 220-235 (16
lines, `f0a67ec1...`), eight configs at 3099-3298 (200, `d374247d...`), eight
production detail routes at 5374-5506 (133, `683e4c79...`), test-only named
detail wrappers at 6960-7046 (87, `08f628be...`), output wrappers at 7058-7142
(85, `9139389e...`), validator wrappers at 8322-8408 (87, `87d26ecb...`), and
eight source extractors at 8819-8930 (112, `95dce665...`). Total: 720 lines.

Task 263N mechanically moves only those fragments into new private
`type_elaboration/parenthesized_routes.rs`. Keeping configs with thin source,
detail, and test wrappers avoids reverse source/output ownership. Only the
eight production detail routes cross the normal phase facade; configs and
test-consumed wrappers/extractors cross under `#[cfg(test)]`. No call site,
name, config value, key, payload, ordering, fallback, or fail-closed behavior
changes. Existing active routes and parenthesized matrices in
`reserved_binary.rs` and `binary_route_fixtures.rs` are the preservation oracle.

This is move-only `design_drift`; no N0 is needed. The new real source path
must be added to the paired Source Inventory with the move and is already
listed in the paired target layout, while
`spec_coverage_audit.md` remains unchanged because authority, behavior,
coverage credit, owner crate, and deferred status do not change.

## Task 263N Move Result

The seven moved fragments retain their exact normalized hashes
`f0a67ec1...`, `d374247d...`, `683e4c79...`, `08f628be...`, `9139389e...`,
`87d26ecb...`, and `95dce665...`; their combined normalized hash is
`93a45180...`. The new 745-line private owner has raw hash `490cc42b...`,
while `runner.rs` is 9,721 lines with raw hash `9cb5f972...`. Invalid-key
constants remain leaf-private. Existing runner test names resolve through
config-derived test-only aliases, and configs, 24 named test wrappers, and
eight extractors cross only the test facade. The normal facade exposes only
the eight production detail routes.

All 25 focused parenthesized tests and all 272 crate unit tests pass. The raw
and normalized 272-name list hashes, four CLI byte hashes, active counts
96/4/188, plan 403/367, type coverage 235/223, pass/fail 219/184, and 23
warnings/zero errors remain unchanged. Formatting, all-target/all-feature
Clippy with warnings denied, workspace tests, and diff cleanliness pass. Task
263N is complete; fresh Task 263 inventory returns to the remaining
non-parenthesized route-owner families. The paired Source Inventory now lists
the new real path, while `spec_coverage_audit.md` remains unchanged because no
authority, behavior, coverage credit, owner crate, or deferred status changed.

## Task 263O Pre-Move Inventory and Specification

Fresh dependency inventory selects the leading direct-binary owner: reserved-
variable equality; reserved-object-variable equality and inequality; distinct
reserved-object-variable equality and inequality; distinct reserved-variable
equality, membership, and inequality; and heterogeneous reserve membership.
The owner is eight exact `runner.rs` fragments: six invalid keys at 150-161
(12 lines, `d3c61a92...`), three invalid keys at 287-292 (6,
`6c3ab931...`), the first five configs at 3131-3244 (114, `aca11227...`),
three distinct configs at 3287-3359 (73, `7febfe4a...`), the heterogeneous
config at 3484-3507 (24, `abe7d7f1...`), nine production detail routes at
5214-5322 (109, `3d564030...`), nine test-only output wrappers at 6678-6768
(91, `475ab5d7...`), and nine source extractors at 8262-8378 (117,
`5499a8cb...`). Total: 546 lines; combined hash `f2271cc0...`.

Task 263O mechanically moves only those fragments into new private
`type_elaboration/binary_routes.rs`. The leaf directly consumes the existing
`source_formula` config/extractor substrate and `output` builder/detail core;
neither sibling depends on the new leaf, so the dependency remains acyclic.
Only the nine production detail routes cross the normal phase facade. Configs,
test-consumed outputs, and extractors cross under `#[cfg(test)]`; invalid-key
constants remain leaf-private while config-derived runner test aliases retain
their existing names and values. No call site, name, config value, key,
payload, ordering, fallback, or fail-closed behavior changes.

The existing 187 direct occurrences—162 output/extractor references plus 25
invalid-key references—across `reserved_binary.rs`,
`binary_route_fixtures.rs`, `reserve_fixtures.rs`,
`reserve_object_fixtures.rs`, and shared test support cover source exactness,
checker payloads, invalid-key fallback, active real fixtures, and route
isolation. Therefore this is move-only `design_drift` and no O0 test task is
needed. The new real path must be added to the paired Source Inventory and
target layout with the move. `spec_coverage_audit.md` remains unchanged because
authority, behavior, coverage credit, owner crate, and deferred status do not
change. Forbidden changes are route-set expansion, direct-family regrouping,
config/key/role edits, wrapper generalization, assertion weakening, test or
expectation edits, and movement of later binary/type-assertion/formula routes.

## Task 263O Move Result

The eight moved fragments retain the original raw hashes `d3c61a92...`,
`6c3ab931...`, `aca11227...`, `7febfe4a...`, `abe7d7f1...`, `3d564030...`,
`475ab5d7...`, and `5499a8cb...` as their pre-move oracle. After removing only
required runner visibility and formatting whitespace, every old/new fragment
pair is token-identical and the combined normalized hash is `86bf7cad...`.
The new 559-line private owner has raw hash `c4546956...`, while `runner.rs` is
9,234 lines with raw hash `8a55c57d...`. Invalid-key constants remain leaf-
private. Existing runner test names resolve through config-derived test-only
aliases, and configs, nine test output wrappers, and nine extractors cross only
the test facade. The normal facade exposes only the nine production detail
routes.

All selected-family focused filters and all 272 crate unit tests pass. The raw
and normalized 272-name list hashes, four CLI byte hashes, active counts
96/4/188, plan 403/367, type coverage 235/223, pass/fail 219/184, and 23
warnings/zero errors remain unchanged. Formatting, all-target/all-feature
Clippy with warnings denied, workspace tests, and diff cleanliness pass. Task
263O is complete; fresh Task 263 inventory returns to the later direct-binary
route-owner families. The paired Source Inventory and target layout list the
new real path, while `spec_coverage_audit.md` remains unchanged because no
authority, behavior, coverage credit, owner crate, or deferred status changed.

## Task 263P Pre-Move Inventory and Specification

Fresh dependency inventory selects the five multiple-reserve declaration
binary routes: set equality, set inequality, set membership, object equality,
and object inequality. They form five exact `runner.rs` fragments: invalid
keys at 324-333 (10 lines, `c1091c1b...`), five configs at 3214-3337 (124,
`85224887...`), five production detail routes at 5522-5583 (62,
`518d4e55...`), five test-only output wrappers at 6805-6856 (52,
`1af7a5ab...`), and five source extractors at 8360-8424 (65,
`55bb8ec4...`). Total: 313 lines; combined hash `790eba84...`.

Task 263P mechanically appends only those fragments to existing private
`type_elaboration/binary_routes.rs`. The dependency remains the Task 263O
acyclic graph to `source_formula` and `output`. Only the five production detail
routes cross the normal phase facade. Configs, test-consumed outputs, and
extractors cross under `#[cfg(test)]`; invalid-key constants remain leaf-private
while config-derived runner test aliases retain their existing names and
values. No call site, name, config value, key, payload, ordering, fallback, or
fail-closed behavior changes.

The existing 104 direct occurrences—96 output/extractor references plus eight
invalid-key references—across `reserved_binary.rs`,
`binary_route_fixtures.rs`, `reserve_fixtures.rs`, and shared test support
cover source exactness, checker payloads, invalid-key fallback, active real
fixtures, and route isolation. Therefore this is move-only `design_drift` and
no P0 test task is needed. No new source path is introduced; the paired target
layout records the expanded owner. `spec_coverage_audit.md` remains unchanged
because authority, behavior, coverage credit, owner crate, and deferred status
do not change. Forbidden changes are route-set expansion, config/key/role
edits, wrapper generalization, assertion weakening, test or expectation edits,
and movement of base, mode-chain, type-assertion, or formula routes.

## Task 263P Move Result

The five moved fragments retain the corrected original raw hashes
`c1091c1b...`, `85224887...`, `518d4e55...`, `1af7a5ab...`, and
`55bb8ec4...` as their pre-move oracle. Initial compile-mode verification found
that the draft 6805-6857 output range incorrectly included the next unmoved
route's `#[cfg(test)]`; the range was corrected to 6805-6856 before completion,
the attribute was restored to that route, and the first moved extractor was
restored to normal leaf visibility. After removing only required runner
visibility and formatting whitespace, every corrected old/new fragment pair is
token-identical and the combined normalized hash is `340d2658...`.

The expanded 872-line private owner has raw hash `883042d7...`, while
`runner.rs` is 8,956 lines with raw hash `48ba9d05...`. Invalid-key constants
remain leaf-private. Existing runner test names resolve through config-derived
test-only aliases, and configs, five test output wrappers, and five extractors
cross only the test facade. The normal facade adds only the five production
detail routes; the phase still owns nine private leaves.

All ten focused multiple-reserve tests and all 272 crate unit tests pass. The
raw and normalized 272-name list hashes, four CLI byte hashes, active counts
96/4/188, plan 403/367, type coverage 235/223, pass/fail 219/184, and 23
warnings/zero errors remain unchanged. Formatting, all-target/all-feature
Clippy with warnings denied, workspace tests, and diff cleanliness pass. Task
263P is complete; fresh Task 263 inventory returns to the remaining base and
mode-chain binary route-owner families. No new source path was introduced and
`spec_coverage_audit.md` remains unchanged because no authority, behavior,
coverage credit, owner crate, or deferred status changed.

## Task 263Q Pre-Move Inventory and Specification

Fresh dependency inventory selects the base reserved-variable membership and
inequality binary routes. They form five exact `runner.rs` fragments: invalid
keys at 361-364 (4 lines, `5d41a022...`), two configs at 3197-3238 (42,
`aa8213c1...`), two production detail routes at 5423-5446 (24,
`81da3344...`), two test-only output wrappers at 6644-6663 (20,
`ae5f0131...`), and two source extractors at 8147-8172 (26,
`1b44be5a...`). Total: 116 lines; combined hash `ec7a766a...`.

Task 263Q mechanically appends only those fragments to existing private
`type_elaboration/binary_routes.rs`. The dependency remains the Tasks
263O-263P acyclic graph to `source_formula` and `output`. Only the two
production detail routes cross the normal phase facade. Configs, test-consumed
outputs, and extractors cross under `#[cfg(test)]`; invalid-key constants remain
leaf-private while config-derived runner test aliases retain their existing
names and values. No call site, name, config value, key, payload, ordering,
fallback, or fail-closed behavior changes.

The existing 38 direct occurrences—34 output/extractor references plus four
invalid-key references—across `reserved_direct.rs`, `reserved_binary.rs`,
`binary_route_fixtures.rs`, `asserted_head_fixtures.rs`, and shared test
support cover source exactness, checker payloads, invalid-key fallback, active
real fixtures, and route isolation. Therefore this is move-only `design_drift`
and no Q0 test task is needed. No new source path is introduced; the paired
target layout records the expanded owner. `spec_coverage_audit.md` remains
unchanged because authority, behavior, coverage credit, owner crate, and
deferred status do not change. Forbidden changes are route-set expansion,
config/key/role edits, wrapper generalization, assertion weakening, test or
expectation edits, and movement of mode-chain, type-assertion, or formula
routes.

## Task 263Q Move Result

Task 263Q moved only the five approved fragments totaling 116 lines into the
existing private `type_elaboration/binary_routes.rs`, preserving original raw
hashes `5d41a022...`, `aa8213c1...`, `81da3344...`, `ae5f0131...`, and
`1b44be5a...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is
token-identical and the combined normalized hash is `e8c45cf1...`. The next
type-assertion output attribute/body and the next extractor remain in
`runner.rs`; no later route crossed the boundary.

The expanded 988-line private owner has raw hash `087967cc...`, while
`runner.rs` is 8,851 lines with raw hash `a039be76...`. Invalid-key constants
remain leaf-private. Existing runner test names resolve through config-derived
test-only aliases, and configs, two test output wrappers, and two extractors
cross only the test facade. The normal facade adds only the two production
detail routes; the phase still owns nine private leaves and its dependency
graph remains acyclic.

The two focused membership/inequality filters pass with 33 and 31 tests,
respectively, and all 272 crate unit tests pass. The raw and normalized
272-name list hashes, four CLI byte hashes, active counts 96/4/188, plan
403/367, type coverage 235/223, pass/fail 219/184, and 23 warnings/zero errors
remain unchanged. Formatting, all-target/all-feature Clippy with warnings
denied, workspace tests, and diff cleanliness pass. Task 263Q is complete;
fresh Task 263 inventory returns to the remaining mode-chain binary route-owner
families. No new source path was introduced and `spec_coverage_audit.md`
remains unchanged because no authority, behavior, coverage credit, owner
crate, or deferred status changed.

## Task 263R Pre-Move Inventory and Specification

Fresh dependency inventory selects the direct local-mode membership, equality,
and inequality binary routes as the next bounded family. They form ten exact
`runner.rs` fragments: the membership invalid key at 255-256 (2 lines,
`c4db5ce6...`), equality/inequality invalid keys at 276-279 (4,
`70a954f2...`), membership config at 3204-3231 (28, `77ebd7a7...`),
equality/inequality configs at 3626-3675 (50, `81a2369d...`), membership
production detail route at 4910-4920 (11, `6545f96f...`), equality/inequality
detail routes at 5043-5065 (23, `74305b0b...`), membership test-only output at
6179-6187 (9, `a0c62cc0...`), equality/inequality test-only outputs at
6292-6310 (19, `0367ba53...`), membership extractor at 7600-7611 (12,
`508569dd...`), and equality/inequality extractors at 7730-7754 (25,
`c1e52d0c...`). Total: 183 lines; combined raw hash `16bcea2e...`.

Task 263R mechanically appends only those fragments to existing private
`type_elaboration/binary_routes.rs`. The leaf continues to depend only on the
existing `source_formula` and `output` owners. Only the three production detail
routes cross the normal phase facade. Configs, test-consumed outputs, and
extractors cross under `#[cfg(test)]`; invalid-key constants remain
leaf-private while config-derived runner test aliases retain their existing
names and values. No call site, name, config value, mode-definition chain, key,
payload, ordering, fallback, or fail-closed behavior changes.

The existing 52 direct test occurrences across `support.rs`,
`binary_route_fixtures.rs`, `mode_chain.rs`, and `mode_chain_fixtures.rs` cover
exact real source expansion, checker payloads, invalid-key fallback, active
fixtures, and route isolation. The three active `.miz`/expectation pairs and
their covered trace requirements directly preserve the canonical reserved
variable plus atomic equality/inequality/membership intent and the exact
source-derived checker seam. Therefore this is move-only `design_drift` and no
R0 test task is needed. No new source path is introduced; the paired target
layout records the expanded owner. `spec_coverage_audit.md` remains unchanged
because authority, behavior, coverage credit, owner crate, and deferred status
do not change. Forbidden changes are route-set expansion, config/key/role/mode
edits, wrapper generalization, assertion weakening, test or expectation edits,
and movement of object-mode, deeper-chain, type-assertion, or formula routes.

## Task 263R Move Result

Task 263R moved only the ten approved fragments totaling 183 lines into the
existing private `type_elaboration/binary_routes.rs`, preserving original raw
hashes `c4db5ce6...`, `70a954f2...`, `77ebd7a7...`, `81a2369d...`,
`6545f96f...`, `74305b0b...`, `a0c62cc0...`, `0367ba53...`,
`508569dd...`, and `c1e52d0c...` as the pre-move oracle. After removing only
required runner visibility and formatting whitespace, every old/new fragment
pair is token-identical and the combined normalized hash is `be8e0e9b...`.
Adjacent chained membership, local-object inequality, and every deeper-chain
config, route, output attribute/body, and extractor remain in `runner.rs`.

The expanded 1,181-line private owner has raw hash `70feaa70...`, while
`runner.rs` is 8,681 lines with raw hash `7131c8b7...`. Invalid-key constants
remain leaf-private. Existing runner test names resolve through config-derived
test-only aliases, and configs, three test output wrappers, and three
extractors cross only the test facade. The normal facade adds only the three
production detail routes; the phase still owns nine private leaves and its
dependency graph remains acyclic.

The focused membership/equality/inequality filters pass with 10, 12, and 10
tests, respectively, and all 272 crate unit tests pass. The raw and normalized
272-name list hashes, four CLI byte hashes, active counts 96/4/188, plan
403/367, type coverage 235/223, pass/fail 219/184, and 23 warnings/zero errors
remain unchanged. Formatting, all-target/all-feature Clippy with warnings
denied, workspace tests, and diff cleanliness pass. Task 263R is complete;
fresh Task 263 inventory returns to the remaining object-mode and deeper-chain
binary route-owner families. No new source path was introduced and
`spec_coverage_audit.md` remains unchanged because no authority, behavior,
coverage credit, owner crate, or deferred status changed.

## Task 263S Pre-Move Inventory and Specification

Fresh dependency inventory selects the direct local-object-mode membership,
equality, and inequality binary routes as the next bounded family. They form
ten exact `runner.rs` fragments: membership/inequality invalid keys at 293-296
(4 lines, `0c2d5a85...`), the equality invalid key at 383-384 (2,
`759fc61a...`), membership/inequality configs at 3583-3636 (54,
`bf587e0d...`), the equality config at 4819-4843 (25, `ff4ef313...`),
membership/inequality production detail routes at 4953-4977 (25,
`08141211...`), the equality detail route at 5274-5285 (12, `7c4207cd...`),
membership/inequality test-only outputs at 6170-6190 (21, `d67627c1...`), the
equality test-only output at 6443-6452 (10, `1b1d490e...`),
membership/inequality extractors at 7573-7597 (25, `889aa420...`), and the
equality extractor at 7885-7896 (12, `3cfd12b2...`). Total: 190 lines;
combined raw hash `3e39b474...`.

Task 263S mechanically appends only those fragments to existing private
`type_elaboration/binary_routes.rs`. The leaf continues to depend only on the
existing `source_formula` and `output` owners. Only the three production detail
routes cross the normal phase facade. Configs, test-consumed outputs, and
extractors cross under `#[cfg(test)]`; invalid-key constants remain
leaf-private while config-derived runner test aliases retain their existing
names and values. No call site, name, config value, mode-definition chain, key,
payload, ordering, fallback, or fail-closed behavior changes.

The existing 52 direct test occurrences—44 output/extractor references plus
eight invalid-key references—across `support.rs`, `binary_route_fixtures.rs`,
`mode_chain.rs`, and `mode_chain_fixtures.rs` cover exact real source
expansion, checker payloads, invalid-key fallback, active fixtures, and route
isolation. The three active `.miz`/expectation pairs and their covered trace
requirements directly preserve the canonical reserved variable plus atomic
equality/inequality/membership intent and the exact source-derived checker
seam. Therefore this is move-only `design_drift` and no S0 test task is needed.
No new source path is introduced; the paired target layout records the
expanded owner. `spec_coverage_audit.md` remains unchanged because authority,
behavior, coverage credit, owner crate, and deferred status do not change.
Forbidden changes are route-set expansion, config/key/role/mode edits, wrapper
generalization, assertion weakening, test or expectation edits, and movement
of chained/deeper-chain, type-assertion, or formula routes.

## Task 263S Move Result

Task 263S moved only the ten approved fragments totaling 190 lines into the
existing private `type_elaboration/binary_routes.rs`, preserving original raw
hashes `0c2d5a85...`, `759fc61a...`, `bf587e0d...`, `ff4ef313...`,
`08141211...`, `7c4207cd...`, `d67627c1...`, `1b1d490e...`,
`889aa420...`, and `3cfd12b2...` as the pre-move oracle. After removing only
required runner visibility and formatting whitespace, every old/new fragment
pair is token-identical and the combined normalized hash is `e0590337...`.
Adjacent chained families and the next type-assertion detail, output
attribute/body, and extractor remain in `runner.rs`.

The expanded 1,380-line private owner has raw hash `2b7e1aef...`, while
`runner.rs` is 8,504 lines with raw hash `f5080dee...`. Invalid-key constants
remain leaf-private. Existing runner test names resolve through config-derived
test-only aliases, and configs, three test output wrappers, and three
extractors cross only the test facade. The normal facade adds only the three
production detail routes; the phase still owns nine private leaves and its
dependency graph remains acyclic.

The focused membership/equality/inequality filters each pass 10 tests, and all
272 crate unit tests pass. The raw and normalized 272-name list hashes, four
CLI byte hashes, active counts 96/4/188, plan 403/367, type coverage 235/223,
pass/fail 219/184, and 23 warnings/zero errors remain unchanged. Formatting,
all-target/all-feature Clippy with warnings denied, workspace tests, and diff
cleanliness pass. Task 263S is complete; fresh Task 263 inventory returns to
the remaining chained and deeper-chain binary route-owner families. No new
source path was introduced and `spec_coverage_audit.md` remains unchanged
because no authority, behavior, coverage credit, owner crate, or deferred
status changed.

## Task 263T Pre-Move Inventory and Specification

Fresh dependency inventory selects the chained local-mode membership,
equality, and inequality binary routes as the next bounded family. They form
fourteen exact `runner.rs` fragments: the membership invalid key at 297-298 (2
lines, `3547f56d...`), equality/inequality invalid keys at 314-317 (4,
`a33a4243...`), membership/equality/inequality configs at 3234-3268 (35,
`9266cead...`), 3598-3629 (32, `ff54a0ed...`), and 4672-4703 (32,
`b624f397...`), the three production detail routes at 4779-4790 (12,
`77d10775...`), 4887-4898 (12, `fd4ddd74...`), and 5142-5153 (12,
`603f4e69...`), the three test-only outputs at 5973-5982 (10,
`5214fdac...`), 6065-6074 (10, `e26f53b0...`), and 6282-6291 (10,
`dd43dd7a...`), and the three source extractors at 7331-7342 (12,
`77c8abb7...`), 7435-7446 (12, `54f042db...`), and 7682-7693 (12,
`92c2a218...`). Total: 207 lines; combined raw hash `dd7a8b0c...`.

Task 263T mechanically appends only those fragments to existing private
`type_elaboration/binary_routes.rs`. The leaf continues to depend only on the
existing `source_formula` and `output` owners. Only the three production detail
routes cross the normal phase facade. Configs, test-consumed outputs, and
extractors cross under `#[cfg(test)]`; invalid-key constants remain
leaf-private while config-derived runner test aliases retain their existing
names and values. No call site, name, config value, two-definition chain, key,
payload, ordering, fallback, or fail-closed behavior changes.

The existing 50 direct test occurrences—43 output/extractor references plus
seven invalid-key references—across `support.rs`, `binary_route_fixtures.rs`,
`mode_chain.rs`, and `mode_chain_fixtures.rs` cover exact real two-expansion
source chains, checker payloads, invalid-key fallback, active fixtures, and
route isolation. The three active `.miz`/expectation pairs and their covered
trace requirements preserve the canonical reserved-variable atomic-formula
intent and exact source-derived checker seam. Therefore this is move-only
`design_drift` and no T0 test task is needed. No new source path is introduced;
the paired target layout records the expanded owner. `spec_coverage_audit.md`
remains unchanged because authority, behavior, coverage credit, owner crate,
and deferred status do not change. Forbidden changes are route-set expansion,
config/key/role/mode edits, chain generalization, assertion weakening, test or
expectation edits, and movement of chained object-mode, deeper-chain,
type-assertion, or formula routes.

## Task 263T Move Result

Task 263T moved only the fourteen approved fragments totaling 207 lines into
the existing private `type_elaboration/binary_routes.rs`, preserving original
raw hashes `3547f56d...`, `a33a4243...`, `9266cead...`, `ff54a0ed...`,
`b624f397...`, `77d10775...`, `fd4ddd74...`, `603f4e69...`,
`5214fdac...`, `e26f53b0...`, `dd43dd7a...`, `77c8abb7...`,
`54f042db...`, and `92c2a218...` as the pre-move oracle. After removing only
required runner visibility and formatting whitespace, every old/new fragment
pair is token-identical and the combined normalized hash is `aa98a27d...`.
Adjacent chained-object, two-/three-/deeper-edge, type-assertion, and other
route families remain in `runner.rs`.

The expanded 1,600-line private owner has raw hash `03d9236d...`, while
`runner.rs` is 8,306 lines with raw hash `3f73039e...`. Invalid-key constants
remain leaf-private. Existing runner test names resolve through config-derived
test-only aliases, and configs, three test output wrappers, and three
extractors cross only the test facade. The normal facade adds only the three
production detail routes; the phase still owns nine private leaves and its
dependency graph remains acyclic.

The focused membership/equality/inequality filters each pass two tests, and all
272 crate unit tests pass. The raw and normalized 272-name list hashes, four
CLI byte hashes, active counts 96/4/188, plan 403/367, type coverage 235/223,
pass/fail 219/184, and 23 warnings/zero errors remain unchanged. Formatting,
all-target/all-feature Clippy with warnings denied, workspace tests, and diff
cleanliness pass. Task 263T is complete; fresh Task 263 inventory returns to
the remaining chained object-mode and deeper-chain binary route-owner
families. No new source path was introduced and `spec_coverage_audit.md`
remains unchanged because no authority, behavior, coverage credit, owner
crate, or deferred status changed.

## Task 263U Pre-Move Inventory and Specification

Fresh dependency inventory selects the chained local-object-mode membership,
equality, and inequality binary routes as the next bounded family. They form
nine exact `runner.rs` fragments: all three invalid keys at 331-338 (8 lines,
`972beff3...`), the membership config at 3537-3575 (39, `71bb150f...`), the
equality/inequality configs at 4618-4690 (73, `32f853aa...`), the membership
production detail route at 4773-4785 (13, `84c8bd3d...`), equality/inequality
detail routes at 5029-5054 (26, `4fc8b564...`), the membership test-only
output at 5916-5926 (11, `5b884de2...`), equality/inequality test-only outputs
at 6134-6155 (22, `7c165117...`), the membership extractor at 7250-7261 (12,
`c84f51e1...`), and equality/inequality extractors at 7497-7521 (25,
`2240a58d...`). Total: 229 lines; combined raw hash `ae0066dd...`.

Task 263U mechanically appends only those fragments to existing private
`type_elaboration/binary_routes.rs`. The leaf continues to depend only on the
existing `source_formula` and `output` owners. Only the three production detail
routes cross the normal phase facade. Configs, test-consumed outputs, and
extractors cross under `#[cfg(test)]`; invalid-key constants remain
leaf-private while config-derived runner test aliases retain their existing
names and values. No call site, name, config value, two-definition object
chain, key, payload, ordering, fallback, or fail-closed behavior changes.

The existing 48 direct test occurrences—41 output/extractor references plus
seven invalid-key references—across `support.rs`, `binary_route_fixtures.rs`,
`mode_chain.rs`, and `mode_chain_fixtures.rs` cover exact real two-expansion
object-mode source chains, checker payloads, invalid-key fallback, active
fixtures, and route isolation. The three active `.miz`/expectation pairs and
their covered trace requirements preserve the canonical reserved-variable
atomic-formula intent and exact source-derived checker seam. Therefore this is
move-only `design_drift` and no U0 test task is needed. No new source path is
introduced; the paired target layout records the expanded owner.
`spec_coverage_audit.md` remains unchanged because authority, behavior,
coverage credit, owner crate, and deferred status do not change. Forbidden
changes are route-set expansion, config/key/role/mode edits, chain
generalization, assertion weakening, test or expectation edits, and movement
of deeper-chain, type-assertion, or formula routes.

## Task 263U Move Result

Task 263U moved only the nine approved fragments totaling 229 lines into the
existing private `type_elaboration/binary_routes.rs`, preserving original raw
hashes `972beff3...`, `71bb150f...`, `32f853aa...`, `84c8bd3d...`,
`4fc8b564...`, `5b884de2...`, `7c165117...`, `c84f51e1...`, and
`2240a58d...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is token-
identical and the combined normalized hash is `a6b1bb6b...`. Adjacent two-/
three-/four-edge, long/deeper-chain, type-assertion, formula, and other route
families remain in `runner.rs`.

The expanded 1,838-line private owner has raw hash `4e4c0125...`, while
`runner.rs` is 8,090 lines with raw hash `687c85be...`; the 235-line phase
facade has raw hash `8980cdd9...`. Invalid-key constants remain leaf-private.
Existing runner test names resolve through config-derived test-only aliases,
and configs, three test output wrappers, and three extractors cross only the
test facade. The normal facade adds only the three production detail routes;
the phase still owns nine private leaves and its dependency graph remains
acyclic.

The focused membership/equality/inequality filters each pass two tests, and all
272 crate unit tests pass. The raw and normalized 272-name list hashes, four
CLI byte hashes, active counts 96/4/188, plan 403/367, type coverage 235/223,
pass/fail 219/184, and 23 warnings/zero errors remain unchanged. Formatting,
all-target/all-feature Clippy with warnings denied, workspace tests, and diff
cleanliness pass. Task 263U is complete; fresh Task 263 inventory returns to
the deeper-chain binary route-owner families. No new source path was
introduced and `spec_coverage_audit.md` remains unchanged because no authority,
behavior, coverage credit, owner crate, or deferred status changed.

## Task 263V Pre-Move Inventory and Specification

Fresh dependency inventory selects the two-edge local-mode membership,
equality, and inequality binary routes as the next bounded family. They form
fifteen exact `runner.rs` fragments: membership/equality/inequality invalid
keys at 339-340 (2 lines, `f02fb8e4...`), 352-353 (2, `ac20181b...`), and
422-423 (2, `a40e0c6f...`); the three configs at 3262-3301 (40,
`54b49166...`), 3550-3586 (37, `0694dde7...`), and 4469-4505 (37,
`30030132...`); the three production detail routes at 4591-4602 (12,
`bc4a798e...`), 4672-4683 (12, `b6bb868b...`), and 4874-4885 (12,
`815c915b...`); the three test-only outputs at 5705-5714 (10,
`d4bb53d3...`), 5774-5783 (10, `65190120...`), and 5946-5955 (10,
`99a8c9c1...`); and the three source extractors at 6995-7006 (12,
`a17900f5...`), 7073-7084 (12, `f77cfcd9...`), and 7268-7279 (12,
`fbe87d76...`). Total: 222 lines; combined raw hash `f680fb91...`.

Task 263V mechanically appends only those fragments to existing private
`type_elaboration/binary_routes.rs`. The leaf continues to depend only on the
existing `source_formula` and `output` owners. Only the three production detail
routes cross the normal phase facade. Configs, test-consumed outputs, and
extractors cross under `#[cfg(test)]`; invalid-key constants remain leaf-
private while config-derived runner test aliases retain their existing names
and values. No call site, name, config value, exact three-definition chain,
key, payload, ordering, fallback, or fail-closed behavior changes.

The existing 54 direct test occurrences—46 output/extractor references, seven
invalid-key references, and one config reference—across `support.rs`,
`binary_route_fixtures.rs`, `mode_chain.rs`, `mode_chain_fixtures.rs`, and
`reserved_binary.rs` cover exact real three-expansion source chains, checker
payloads, invalid-key fallback, active fixtures, cross-route isolation, and the
direct/parenthesized owner boundary. The three active `.miz`/expectation pairs
and their covered trace requirements preserve the canonical reserved-variable
atomic-formula intent and exact source-derived checker seam. Therefore this is
move-only `design_drift` and no V0 test task is needed. No new source path is
introduced; the paired target layout records the expanded owner.
`spec_coverage_audit.md` remains unchanged because authority, behavior,
coverage credit, owner crate, and deferred status do not change. Forbidden
changes are route-set expansion, config/key/role/mode edits, chain
generalization, assertion weakening, test or expectation edits, and movement
of the object-mode, three-/four-edge, long-chain, type-assertion, or formula
routes.

## Task 263V Move Result

Task 263V moved only the fifteen approved fragments totaling 222 lines into
the existing private `type_elaboration/binary_routes.rs`, preserving original
raw hashes `f02fb8e4...`, `ac20181b...`, `a40e0c6f...`, `54b49166...`,
`0694dde7...`, `30030132...`, `bc4a798e...`, `b6bb868b...`,
`815c915b...`, `d4bb53d3...`, `65190120...`, `99a8c9c1...`,
`a17900f5...`, `f77cfcd9...`, and `fbe87d76...` as the pre-move oracle.
After removing only required runner visibility and formatting whitespace,
every old/new fragment pair is token-identical and the combined normalized
hash is `53865bd6...`. Adjacent object-mode, three-/four-edge, long-chain,
type-assertion, formula, and other route families remain in `runner.rs`.

The expanded 2,073-line private owner has raw hash `024f1b74...`, while
`runner.rs` is 7,877 lines with raw hash `e609ff69...`; the 247-line phase
facade has raw hash `8d12176a...`. Invalid-key constants remain leaf-private.
Existing runner test names and the retained direct parenthesized corruption
consumer resolve through config-derived test-only aliases. Configs, three test
output wrappers, and three extractors cross only the test facade. The normal
facade adds only the three production detail routes; the phase still owns nine
private leaves and its dependency graph remains acyclic.

The focused membership/equality/inequality filters pass two, four, and two
tests respectively, including both parenthesized boundary tests, and all 272
crate unit tests pass. The raw and normalized 272-name list hashes, four CLI
byte hashes, active counts 96/4/188, plan 403/367, type coverage 235/223,
pass/fail 219/184, and 23 warnings/zero errors remain unchanged. Formatting,
all-target/all-feature Clippy with warnings denied, workspace tests, and diff
cleanliness pass. Task 263V is complete; fresh Task 263 inventory returns to
the two-edge object-mode and remaining three-/four-edge and long-chain binary
route-owner families. No new source path was introduced and
`spec_coverage_audit.md` remains unchanged because no authority, behavior,
coverage credit, owner crate, or deferred status changed.

## Task 263W Pre-Move Inventory and Specification

Fresh dependency inventory selects the two-edge local-object-mode membership,
equality, and inequality binary routes as the next bounded family. They form
eleven exact `runner.rs` fragments: the membership invalid key at 369-370 (2
lines, `d572e286...`) and inequality/equality invalid keys at 439-442 (4,
`d571dc2e...`); the three configs at 3479-3522 (44, `2f964b21...`),
4405-4447 (43, `9438d880...`), and 4449-4487 (39, `246a2852...`);
the membership production detail route at 4543-4555 (13, `7277fccf...`) and
inequality/equality detail routes at 4746-4771 (26, `041e760f...`); the
membership test-only output at 5610-5620 (11, `1231694c...`) and inequality/
equality test-only outputs at 5783-5804 (22, `2d5ae89e...`); and the membership
extractor at 6873-6884 (12, `82ab31ea...`) and inequality/equality extractors at
7068-7092 (25, `63fa9c8a...`). Total: 241 lines; combined raw hash
`a57c6acd...`.

Task 263W mechanically appends only those fragments to existing private
`type_elaboration/binary_routes.rs`. The leaf continues to depend only on the
existing `source_formula` and `output` owners. Only the three production detail
routes cross the normal phase facade. Configs, test-consumed outputs, and
extractors cross under `#[cfg(test)]`; invalid-key constants remain leaf-
private while config-derived runner test aliases retain their existing names
and values. No call site, name, config value, exact three-definition object
chain, key, payload, ordering, fallback, or fail-closed behavior changes.

The existing 50 direct test occurrences—43 output/extractor references plus
seven invalid-key references—across `support.rs`,
`binary_route_fixtures.rs`, `mode_chain.rs`, and `mode_chain_fixtures.rs`
cover exact real three-expansion object-mode source chains, checker payloads,
invalid-key fallback, active fixtures, cross-route isolation, and the object/
set identity boundary. The three active `.miz`/expectation pairs and their
covered trace requirements preserve the canonical reserved-variable atomic-
formula intent and exact source-derived checker seam. Therefore this is move-
only `design_drift` and no W0 test task is needed. No new source path is
introduced; the paired target layout records the expanded owner.
`spec_coverage_audit.md` remains unchanged because authority, behavior,
coverage credit, owner crate, and deferred status do not change. Forbidden
changes are route-set expansion, config/key/role/mode edits, chain
generalization, object/set coercion, assertion weakening, test or expectation
edits, and movement of the three-/four-edge, long-chain, type-assertion, or
formula routes.

## Task 263W Move Result

Task 263W moved only the eleven approved fragments totaling 241 lines into the
existing private `type_elaboration/binary_routes.rs`, preserving original raw
hashes `d572e286...`, `d571dc2e...`, `2f964b21...`, `9438d880...`,
`246a2852...`, `7277fccf...`, `041e760f...`, `1231694c...`,
`2d5ae89e...`, `82ab31ea...`, and `63fa9c8a...` as the pre-move oracle.
After removing only required runner visibility and formatting whitespace,
every old/new fragment pair is token-identical and the combined normalized
hash is `0e58ae98...`. Adjacent three-/four-edge, long-chain, type-assertion,
formula, and other route families remain in `runner.rs`.

The expanded 2,324-line private owner has raw hash `9ef34cf7...`, while
`runner.rs` is 7,649 lines with raw hash `394ebbe8...`; the 259-line phase
facade has raw hash `361f6e9c...`. Invalid-key constants remain leaf-private.
Existing runner test names resolve through config-derived test-only aliases,
and configs, three test output wrappers, and three extractors cross only the
test facade. The normal facade adds only the three production detail routes;
the phase still owns nine private leaves and its dependency graph remains
acyclic.

The focused membership/equality/inequality filters each pass two tests, and all
272 crate unit tests pass. The raw and normalized 272-name list hashes, four
CLI byte hashes, active counts 96/4/188, plan 403/367, type coverage 235/223,
pass/fail 219/184, and 23 warnings/zero errors remain unchanged. Formatting,
all-target/all-feature Clippy with warnings denied, workspace tests, and diff
cleanliness pass. Task 263W is complete; fresh Task 263 inventory returns to
the remaining three-/four-edge and long-chain binary route-owner families. No
new source path was introduced and `spec_coverage_audit.md` remains unchanged
because no authority, behavior, coverage credit, owner crate, or deferred
status changed.

## Task 263X Pre-Move Inventory and Specification

Fresh dependency inventory selects the three-edge local-mode membership,
equality, and inequality binary routes as the next bounded family. They form
fifteen exact `runner.rs` fragments: invalid keys at 382-384 (3 lines,
`3d3783b9...`), 391-392 (2, `24d4d6cc...`), and 454-456 (3,
`57a14811...`); configs at 3293-3337 (45, `9d05006c...`), 3495-3536
(42, `86200198...`), and 4284-4325 (42, `8163a029...`); production detail
routes at 4376-4387 (12, `11980a6b...`), 4430-4441 (12, `09665060...`),
and 4592-4603 (12, `7f640564...`); test-only outputs at 5410-5420 (11,
`0973c2cd...`), 5456-5466 (11, `3b6b99b8...`), and 5594-5604 (11,
`063e707f...`); and extractors at 6632-6643 (12, `a9540df8...`),
6684-6695 (12, `127e3811...`), and 6840-6851 (12, `00752953...`). Total:
242 lines; combined raw hash `4af1d41e...` and whitespace-normalized pre-move
hash `1cb58abe...`.

Task 263X mechanically appends only those fragments to existing private
`type_elaboration/binary_routes.rs`. The leaf continues to depend only on the
existing `source_formula` and `output` owners. Only the three production detail
routes cross the normal phase facade. Configs, test-consumed outputs, and
extractors cross under `#[cfg(test)]`; invalid-key constants remain leaf-
private while config-derived runner test aliases retain their existing names
and values. No call site, name, config value, exact four-expansion set-terminal
chain, key, payload, ordering, fallback, or fail-closed behavior changes.

Canonical `doc/spec/en` requires top-level reserves to supply the default type
and implicit theorem closure, modes to expose their radix type during type
inference, and `=`, `<>`, and `in` to remain single built-in atomic formulas.
The three active `.miz`/expectation pairs and their covered trace requirements
instantiate that intent through the real source AST, resolver environment,
four mode expansions, and checker output. The existing 50 direct test symbol
references—43 output/extractor references and seven invalid-key references—
across `support.rs`, `binary_route_fixtures.rs`, `mode_chain.rs`, and
`mode_chain_fixtures.rs` protect the exact payload, source provenance,
invalid-key fallback, active fixtures, and cross-route isolation. Therefore
this is move-only `design_drift` and no X0 test task is needed. No new source
path is introduced; the paired target layout records the expanded owner.
`spec_coverage_audit.md` remains unchanged because authority, behavior,
coverage credit, owner crate, and deferred status do not change. Forbidden
changes are route-set expansion, config/key/role/mode edits, chain
generalization, object/set coercion, assertion weakening, test or expectation
edits, and movement of the adjacent object-mode, four-edge, long-chain,
type-assertion, or formula routes.

## Task 263X Move Result

Task 263X moved only the fifteen approved fragments totaling 242 lines into
the existing private `type_elaboration/binary_routes.rs`, preserving original
raw hashes `3d3783b9...`, `24d4d6cc...`, `57a14811...`, `9d05006c...`,
`86200198...`, `8163a029...`, `11980a6b...`, `09665060...`,
`7f640564...`, `0973c2cd...`, `3b6b99b8...`, `063e707f...`,
`a9540df8...`, `127e3811...`, and `00752953...` as the pre-move oracle.
After removing only required runner visibility and formatting whitespace,
every old/new fragment pair is token-identical and the combined normalized
hash is `1cb58abe...`. Adjacent three-edge object-mode, four-edge, long-chain,
type-assertion, formula, and other route families remain in `runner.rs`.

The expanded 2,578-line private owner has raw hash `75d75cb7...`, while
`runner.rs` is 7,419 lines with raw hash `68c7c44d...`; the 271-line phase
facade has raw hash `7934071f...`. Invalid-key constants remain leaf-private.
Existing runner test names resolve through config-derived test-only aliases,
and configs, three test output wrappers, and three extractors cross only the
test facade. The normal facade adds only the three production detail routes;
the phase still owns nine private leaves and its dependency graph remains
acyclic.

The focused membership/equality/inequality filters each pass two tests, and all
272 crate unit tests pass. The raw and normalized 272-name list hashes, four
CLI byte hashes, active counts 96/4/188, plan 403/367, type coverage 235/223,
pass/fail 219/184, and 23 warnings/zero errors remain unchanged. Formatting,
all-target/all-feature Clippy with warnings denied, workspace tests, and diff
cleanliness pass. Task 263X is complete; fresh Task 263 inventory returns to
the three-edge object-mode, four-edge, and long-chain binary route-owner
families. No new source path was introduced and `spec_coverage_audit.md`
remains unchanged because no authority, behavior, coverage credit, owner
crate, or deferred status changed.

## Task 263Y Pre-Move Inventory and Specification

Fresh dependency inventory selects the three-edge local-object-mode
membership, equality, and inequality binary routes as the next bounded family.
They form eleven exact `runner.rs` fragments: the membership invalid key at
407-408 (2 lines, `280af2bf...`) and equality/inequality invalid keys at
468-471 (4, `9c823dee...`); the three configs at 3412-3460 (49,
`dfaab518...`), 4163-4206 (44, `e9d7705a...`), and 4208-4255 (48,
`a9e040ec...`); the membership production detail route at 4284-4296 (13,
`78cef703...`) and equality/inequality detail routes at 4433-4459 (27,
`014d3897...`); the membership test-only output at 5275-5285 (11,
`dd7e079e...`) and equality/inequality test-only outputs at 5402-5424 (23,
`60f64f3d...`); and the membership extractor at 6467-6478 (12,
`44e666e7...`) and equality/inequality extractors at 6610-6634 (25,
`d868202c...`). Total: 258 lines; combined raw hash `21918677...` and
whitespace-normalized pre-move hash `ad754ac3...`.

Task 263Y mechanically appends only those fragments to existing private
`type_elaboration/binary_routes.rs`. The leaf continues to depend only on the
existing `source_formula` and `output` owners. Only the three production detail
routes cross the normal phase facade. Configs, test-consumed outputs, and
extractors cross under `#[cfg(test)]`; invalid-key constants remain leaf-
private while config-derived runner test aliases retain their existing names
and values. No call site, name, config value, exact four-expansion object-
terminal chain, key, payload, ordering, fallback, or fail-closed behavior
changes.

The canonical reserve/mode/built-in atomic-formula requirements inventoried
for Task 263X apply unchanged. The three active `.miz`/expectation pairs and
their covered trace requirements instantiate them through the real source AST,
resolver environment, four object-mode expansions, and checker output. The
existing 50 direct test symbol references—43 output/extractor references and
seven invalid-key references—across `support.rs`,
`binary_route_fixtures.rs`, `mode_chain.rs`, and `mode_chain_fixtures.rs`
protect the exact payload, source provenance, invalid-key fallback, active
fixtures, cross-route isolation, and object/set identity boundary. Therefore
this is move-only `design_drift` and no Y0 test task is needed. No new source
path is introduced; the paired target layout records the expanded owner.
`spec_coverage_audit.md` remains unchanged because authority, behavior,
coverage credit, owner crate, and deferred status do not change. Forbidden
changes are route-set expansion, config/key/role/mode edits, chain
generalization, object/set coercion, assertion weakening, test or expectation
edits, and movement of the adjacent four-edge, long-chain, type-assertion, or
formula routes.

## Task 263Y Move Result

Task 263Y moved only the eleven approved fragments totaling 258 lines into the
existing private `type_elaboration/binary_routes.rs`, preserving original raw
hashes `280af2bf...`, `9c823dee...`, `dfaab518...`, `e9d7705a...`,
`a9e040ec...`, `78cef703...`, `014d3897...`, `dd7e079e...`,
`60f64f3d...`, `44e666e7...`, and `d868202c...` as the pre-move oracle.
After removing only required runner visibility and formatting whitespace,
every old/new fragment pair is token-identical and the combined normalized
hash is `ad754ac3...`. Adjacent four-edge, long-chain, type-assertion, formula,
and other route families remain in `runner.rs`.

The expanded 2,847-line private owner has raw hash `1e4fc792...`, while
`runner.rs` is 7,173 lines with raw hash `51cb7b50...`; the 283-line phase
facade has raw hash `a2b84b11...`. Invalid-key constants remain leaf-private.
Existing runner test names resolve through config-derived test-only aliases,
and configs, three test output wrappers, and three extractors cross only the
test facade. The normal facade adds only the three production detail routes;
the phase still owns nine private leaves and its dependency graph remains
acyclic.

The focused membership/equality/inequality filters each pass two tests, and all
272 crate unit tests pass. The raw and normalized 272-name list hashes, four
CLI byte hashes, active counts 96/4/188, plan 403/367, type coverage 235/223,
pass/fail 219/184, and 23 warnings/zero errors remain unchanged. Formatting,
all-target/all-feature Clippy with warnings denied, workspace tests, and diff
cleanliness pass. Task 263Y is complete; fresh Task 263 inventory returns to
the four-edge and long-chain binary route-owner families. No new source path
was introduced and `spec_coverage_audit.md` remains unchanged because no
authority, behavior, coverage credit, owner crate, or deferred status changed.

## Task 263Z Pre-Move Inventory and Specification

Fresh dependency inventory selects the four-edge local-mode membership,
equality, and inequality binary routes as the next bounded family. They form
fifteen exact `runner.rs` fragments: invalid keys at 424-425 (2 lines,
`8f324bb2...`), 428-429 (2, `29c5996b...`), and 481-482 (2,
`25ab8aa5...`); configs at 3321-3370 (50, `bbe09f99...`), 3427-3473
(47, `7688c6b6...`), and 3976-4022 (47, `2b8d6ce0...`); production detail
routes at 4128-4139 (12, `5ecba726...`), 4155-4166 (12,
`1153ec2e...`), and 4249-4260 (12, `d8f7be05...`); test-only outputs at
5081-5090 (10, `9b36914d...`), 5104-5113 (10, `b06499a8...`), and
5184-5193 (10, `fd3deb01...`); and extractors at 6234-6245 (12,
`032d0570...`), 6260-6271 (12, `31eae655...`), and 6351-6362 (12,
`8ae80c4f...`). Total: 252 lines; combined raw hash `139c5d9b...` and
whitespace-normalized pre-move hash `e1865620...`.

Task 263Z mechanically appends only those fragments to existing private
`type_elaboration/binary_routes.rs`. The leaf continues to depend only on the
existing `source_formula` and `output` owners. Only the three production detail
routes cross the normal phase facade. Configs, test-consumed outputs, and
extractors cross under `#[cfg(test)]`; invalid-key constants remain leaf-
private while config-derived runner test aliases retain their existing names
and values. No call site, name, config value, exact five-expansion set-terminal
chain, key, payload, ordering, fallback, or fail-closed behavior changes.

The canonical reserve/mode/built-in atomic-formula requirements apply
unchanged. The three active `.miz`/expectation pairs and their covered trace
requirements instantiate them through the real source AST, resolver
environment, five set-terminal mode expansions, and checker output. The
existing 56 direct test symbol references—47 output/extractor references and
nine invalid-key references—across `support.rs`,
`binary_route_fixtures.rs`, `mode_chain.rs`, `mode_chain_fixtures.rs`,
`remaining_bridges_and_nested_isolation.rs`, and
`source_gap_and_equality.rs` protect the exact payload, source provenance,
invalid-key fallback, active fixtures, and cross-route isolation. Therefore
this is move-only `design_drift` and no Z0 test task is needed. No new source
path is introduced; the paired target layout records the expanded owner.
`spec_coverage_audit.md` remains unchanged because authority, behavior,
coverage credit, owner crate, and deferred status do not change. Forbidden
changes are route-set expansion, config/key/role/mode edits, chain
generalization, object/set coercion, assertion weakening, test or expectation
edits, and movement of the adjacent four-edge object-mode, long-chain,
type-assertion, or formula routes.

## Task 263Z Move Result

Task 263Z moved only the fifteen approved fragments totaling 252 lines into
the existing private `type_elaboration/binary_routes.rs`, preserving original
raw hashes `8f324bb2...`, `29c5996b...`, `25ab8aa5...`, `bbe09f99...`,
`7688c6b6...`, `2b8d6ce0...`, `5ecba726...`, `1153ec2e...`,
`d8f7be05...`, `9b36914d...`, `b06499a8...`, `fd3deb01...`,
`032d0570...`, `31eae655...`, and `8ae80c4f...` as the pre-move oracle.
After removing only required runner visibility and formatting whitespace,
every old/new fragment pair is token-identical and the combined normalized
hash is `e1865620...`. Adjacent four-edge object-mode, long-chain,
type-assertion, formula, and other route families remain in `runner.rs`.

The expanded 3,114-line private owner has raw hash `73de594a...`, while
`runner.rs` is 6,930 lines with raw hash `fb4a4a2b...`; the 295-line phase
facade has raw hash `f0ed4b4e...`. Invalid-key constants remain leaf-private.
Existing runner test names resolve through config-derived test-only aliases,
and configs, three test output wrappers, and three extractors cross only the
test facade. The normal facade adds only the three production detail routes;
the phase still owns nine private leaves and its dependency graph remains
acyclic.

The focused membership/equality/inequality filters each pass two tests, and all
272 crate unit tests pass. The raw and normalized 272-name list hashes, four
CLI byte hashes, active counts 96/4/188, plan 403/367, type coverage 235/223,
pass/fail 219/184, and 23 warnings/zero errors remain unchanged. Formatting,
all-target/all-feature Clippy with warnings denied, workspace tests, and diff
cleanliness pass. Task 263Z is complete; fresh Task 263 inventory returns to
the four-edge object-mode and long-chain binary route-owner families. No new
source path was introduced and `spec_coverage_audit.md` remains unchanged
because no authority, behavior, coverage credit, owner crate, or deferred
status changed.

## Task 263ZA Pre-Move Inventory and Specification

Fresh dependency inventory selects the four-edge local-object-mode membership,
equality, and inequality binary routes as the next bounded family. They form
eleven exact `runner.rs` fragments: the membership invalid key at 445-446
(2 lines, `7ff1e465...`) and paired equality/inequality keys at 498-501 (4,
`e096a1f5...`); configs at 3336-3389 (54, `d2cd8eea...`), 3892-3940 (49,
`51430aa3...`), and 3942-3994 (53, `1b676067...`); production detail routes at
3996-4008 (13, `11986cb6...`) and 4091-4117 (27, `538b9ee7...`); test-only
outputs at 4910-4920 (11, `3f508c4d...`) and 4991-5013 (23,
`3523e34b...`); and extractors at 6030-6041 (12, `d3f59d9a...`) and 6121-6145
(25, `2f6a0d86...`). Total: 273 lines; combined raw hash `39ad5285...` and
whitespace-normalized pre-move hash `594c1e49...`.

Task 263ZA mechanically appends only those fragments to existing private
`type_elaboration/binary_routes.rs`. The leaf continues to depend only on the
existing `source_formula` and `output` owners. Only the three production detail
routes cross the normal phase facade. Configs, test-consumed outputs, and
extractors cross under `#[cfg(test)]`; invalid-key constants remain leaf-private
while config-derived runner test aliases retain their existing names and
values. No call site, name, config value, exact five-expansion object-terminal
chain, key, payload, ordering, fallback, or fail-closed behavior changes.

The canonical reserve/mode/built-in atomic-formula requirements apply
unchanged. The three active `.miz`/expectation pairs and their covered trace
requirements instantiate them through the real source AST, resolver
environment, five object-terminal mode expansions, and checker output. The
existing 56 direct test symbol references—47 output/extractor references and
nine invalid-key references—across `support.rs`, `binary_route_fixtures.rs`,
`mode_chain.rs`, `mode_chain_fixtures.rs`, and
`remaining_bridges_and_nested_isolation.rs` protect the exact payload, source
provenance, invalid-key fallback, active fixtures, and cross-route isolation.
Therefore this is move-only `design_drift` and no ZA0 test task is needed. No
new source path is introduced; the paired target layout records the expanded
owner. `spec_coverage_audit.md` remains unchanged because authority, behavior,
coverage credit, owner crate, and deferred status do not change. Forbidden
changes are route-set expansion, config/key/role/mode edits, chain
generalization, object/set coercion, assertion weakening, test or expectation
edits, and movement of the adjacent long-chain, type-assertion, or formula
routes.

## Task 263ZA Move Result

Task 263ZA moved only the eleven approved fragments totaling 273 lines into
the existing private `type_elaboration/binary_routes.rs`, preserving original
raw hashes `7ff1e465...`, `e096a1f5...`, `d2cd8eea...`, `51430aa3...`,
`1b676067...`, `11986cb6...`, `538b9ee7...`, `3f508c4d...`, `3523e34b...`,
`d3f59d9a...`, and `2f6a0d86...` as the pre-move oracle. After removing only
required runner visibility and formatting whitespace, every old/new fragment
pair is token-identical and the combined normalized hash is `594c1e49...`.
Adjacent long-chain, type-assertion, formula, and other route families remain
in `runner.rs`.

The expanded 3,398-line private owner has raw hash `8fd56903...`, while
`runner.rs` is 6,669 lines with raw hash `6f8b9737...`; the 307-line phase
facade has raw hash `59ae62b4...`. Invalid-key constants remain leaf-private.
Existing runner test names resolve through config-derived test-only aliases,
and configs, three test output wrappers, and three extractors cross only the
test facade. The normal facade adds only the three production detail routes;
the phase still owns nine private leaves and its dependency graph remains
acyclic.

The focused membership/equality/inequality filters each pass two tests, and all
272 crate unit tests pass. The raw and normalized 272-name list hashes, four
CLI byte hashes, active counts 96/4/188, plan 403/367, type coverage 235/223,
pass/fail 219/184, and 23 warnings/zero errors remain unchanged. Formatting,
all-target/all-feature Clippy with warnings denied, workspace tests, and diff
cleanliness pass. Task 263ZA is complete; fresh Task 263 inventory returns to
the long-chain binary route-owner families. No new source path was introduced
and `spec_coverage_audit.md` remains unchanged because no authority, behavior,
coverage credit, owner crate, or deferred status changed.

## Task 263ZB Pre-Move Inventory and Specification

Fresh dependency inventory finds that the long-chain binary routes cannot move
independently yet: the set-terminal and object-terminal seven-expansion tables
each feed eleven retained production configs—three binary, one reserved-variable
type-assertion, and seven asserted-head routes. Moving either table with only a
binary family would introduce a child-to-parent dependency or mix route
families. Task 263ZB therefore selects the two shared definition tables as a
bounded prerequisite owner: `runner.rs` 3351-3387 (37 lines, `b9ef5e33...`) and
3411-3447 (37, `23d65f84...`). Total: 74 lines; combined raw hash
`3941ea98...` and whitespace-normalized pre-move hash `ab85b7ea...`.

Task 263ZB mechanically moves only those tables to a new nonempty private
`type_elaboration/long_chain_config.rs` leaf. The leaf depends only on the
existing `source_formula` config types. The phase facade temporarily exposes
exactly the two tables to the 22 retained `runner.rs` production consumers;
later long-chain route-owner tasks import the sibling directly and shrink that
facade surface. Names, table order, labels, spellings, builtin terminals,
radices, cardinality, and every consumer remain unchanged. No binary,
type-assertion, asserted-head, detail, output, extractor, dispatch, or test item
moves in this task.

Canonical mode/radix and reserve/formula requirements apply unchanged. The 44
unit tests in the cohesive long-chain owner, 23 active `.miz`/expectation pairs,
and the active metadata integration test exercise both exact seven-expansion
tables through every binary, type-assertion, and asserted-head consumer. This is
move-only `design_drift`; no ZB0 test task is needed. The paired source layout
adds the real shared owner. `spec_coverage_audit.md` remains unchanged because
authority, behavior, coverage credit, owner crate, and deferred status do not
change. Forbidden changes are table deduplication/generalization, spelling or
radix edits, accepted-shape expansion, assertion weakening, test/expectation
edits, and movement of any consumer route.

## Task 263ZB Move Result

Task 263ZB moved only the two approved definition-table fragments totaling 74
lines into new private `type_elaboration/long_chain_config.rs`, preserving raw
hashes `b9ef5e33...` and `23d65f84...` as the pre-move oracle. After removing
only required runner visibility and formatting whitespace, both old/new table
pairs are token-identical and the combined normalized hash is `ab85b7ea...`.
All 22 binary, type-assertion, and asserted-head config consumers remain in
`runner.rs` with unchanged values and order.

The new real owner is 82 lines with raw hash `3b0e2638...`, while `runner.rs` is
6,594 lines with raw hash `5f8c17de...`; the 311-line phase facade has raw hash
`453068d3...`. The leaf imports only the three existing `source_formula` config
types, exports exactly the two tables with runner-only visibility, and adds no
public API. The phase facade temporarily re-exports those tables to retained
production consumers; dependencies remain acyclic.

All 44 focused long-chain unit tests and the focused metadata integration test
pass, as do all 272 crate unit tests. The raw and normalized 272-name list
hashes, four CLI byte hashes, active counts 96/4/188, plan 403/367, type
coverage 235/223, pass/fail 219/184, and 23 warnings/zero errors remain
unchanged. Formatting, all-target/all-feature Clippy with warnings denied,
workspace tests, and diff cleanliness pass. Task 263ZB is complete; fresh Task
263 inventory can now split the local-mode and local-object-mode long-chain
binary route families through sibling dependencies. `spec_coverage_audit.md`
remains unchanged because no authority, behavior, coverage credit, owner crate,
or deferred status changed.

## Task 263ZC Pre-Move Inventory and Specification

Fresh dependency inventory selects the local-mode long-chain membership,
equality, and inequality binary routes as the next bounded family now that Task
263ZB owns their shared table. They form fifteen exact `runner.rs` fragments:
invalid keys at 467-468 (2 lines, `18b60fd0...`), 475-477 (3,
`1a844717...`), and 478-480 (3, `ed156484...`); configs at 3352-3372 (21,
`387e7f5e...`), 3449-3469 (21, `684fa9a6...`), and 3471-3494 (24,
`745185c5...`); production detail routes at 3777-3788 (12, `8a480a24...`),
3832-3843 (12, `1567378c...`), and 3845-3856 (12, `d8c1184a...`); test-only
outputs at 4649-4658 (10, `6ed554fb...`), 4696-4705 (10, `2b4ffa33...`), and
4707-4716 (10, `58399ffd...`); and extractors at 5733-5744 (12,
`543bd8cd...`), 5785-5796 (12, `d1365809...`), and 5798-5809 (12,
`5baa8351...`). Total: 176 lines; combined raw hash `076d8561...` and
whitespace-normalized pre-move hash `8859b993...`.

Task 263ZC mechanically appends only those fragments to existing private
`type_elaboration/binary_routes.rs`, importing the shared set-terminal table
directly from sibling `long_chain_config`. Only the three production detail
routes cross the normal phase facade. Configs, test-consumed outputs, and
extractors cross under `#[cfg(test)]`; invalid-key constants remain leaf-private
while config-derived runner test aliases retain their existing names and
values. No call site, name, config value, exact seven-expansion set-terminal
chain, key, payload, ordering, fallback, or fail-closed behavior changes.

Canonical reserve/mode/built-in atomic-formula requirements apply unchanged.
The three active `.miz`/expectation pairs and their covered trace requirements
exercise the real source AST, resolver environment, seven set-terminal mode
expansions, and checker output. Existing direct test symbol references total 54
(48 output/extractor and six invalid-key references) across `support.rs`,
`binary_route_fixtures.rs`, and `long_chain.rs`, protecting exact payload,
source provenance, invalid-key fallback, active fixtures, and cross-route
isolation. Therefore this is move-only `design_drift` and no ZC0 test task is
needed. `spec_coverage_audit.md` remains unchanged because authority, behavior,
coverage credit, owner crate, and deferred status do not change. Forbidden
changes are route-set expansion, config/key/role/mode edits, chain
generalization, object/set coercion, assertion weakening, test/expectation
edits, and movement of adjacent local-object-mode, type-assertion, asserted-head,
or formula routes.

## Task 263ZC Move Result

Task 263ZC moved only the fifteen approved fragments totaling 176 lines into
the existing private `type_elaboration/binary_routes.rs`, preserving original
raw hashes `18b60fd0...`, `1a844717...`, `ed156484...`, `387e7f5e...`,
`684fa9a6...`, `745185c5...`, `8a480a24...`, `1567378c...`, `d8c1184a...`,
`6ed554fb...`, `2b4ffa33...`, `58399ffd...`, `543bd8cd...`, `d1365809...`,
and `5baa8351...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is
token-identical and the combined normalized hash is `8859b993...`. Adjacent
local-object-mode, type-assertion, asserted-head, formula, and other route
families remain in `runner.rs`.

The expanded 3,590-line private owner has raw hash `6f8f8c73...`, while
`runner.rs` is 6,427 lines with raw hash `d1f5b9bb...`; the 323-line phase facade
has raw hash `2e757879...`. `long_chain_config.rs` remains 82 lines with hash
`3b0e2638...`. Invalid-key constants remain leaf-private; configs, three test
output wrappers, and three extractors cross only the test facade. The normal
facade adds only three production detail routes, and `binary_routes` imports
the shared set-terminal table directly from its sibling.

The focused membership/equality/inequality filters each pass two tests, and all
272 crate unit tests pass. The raw and normalized 272-name list hashes, four
CLI byte hashes, active counts 96/4/188, plan 403/367, type coverage 235/223,
pass/fail 219/184, and 23 warnings/zero errors remain unchanged. Formatting,
all-target/all-feature Clippy with warnings denied, workspace tests, and diff
cleanliness pass. Task 263ZC is complete; fresh Task 263 inventory returns to
the local-object-mode long-chain binary route family. `spec_coverage_audit.md`
remains unchanged because no authority, behavior, coverage credit, owner crate,
or deferred status changed.

## Task 263ZD Pre-Move Inventory and Specification

Fresh dependency inventory selects the local-object-mode long-chain
membership, equality, and inequality binary routes as the next bounded family.
They form fifteen exact `runner.rs` fragments: invalid keys at 488-489 (2
lines, `c9abd07e...`), 490-491 (2, `76c6badc...`), and 492-493 (2,
`274e8231...`); configs at 3365-3385 (21, `da89ee91...`), 3387-3411 (25,
`24c13cf8...`), and 3413-3438 (26, `9bbfb507...`); production detail routes at
3721-3733 (13, `a3646c63...`), 3735-3747 (13, `ebfd9b1f...`), and 3749-3761
(13, `c392ea7b...`); test-only outputs at 4554-4564 (11, `29d99bb5...`),
4566-4576 (11, `432ad380...`), and 4578-4588 (11, `c1e39c32...`); and
extractors at 5605-5616 (12, `591afb95...`), 5618-5629 (12, `f1750caf...`),
and 5631-5642 (12, `cd31b66d...`). Total: 186 lines; combined raw hash
`073769aa...` and whitespace-normalized pre-move hash `de18e68c...`.

Task 263ZD mechanically appends only those fragments to existing private
`type_elaboration/binary_routes.rs`, importing the shared object-terminal table
directly from sibling `long_chain_config`. Only the three production detail
routes cross the normal phase facade. Configs, test-consumed outputs, and
extractors cross under `#[cfg(test)]`; invalid-key constants remain
leaf-private while config-derived runner test aliases retain their existing
names and values. No call site, name, config value, exact seven-expansion
object-terminal chain, key, payload, ordering, fallback, or fail-closed
behavior changes.

Canonical reserve/mode/built-in atomic-formula requirements apply unchanged.
The three active `.miz`/expectation pairs and their covered trace requirements
exercise the real source AST, resolver environment, seven object-terminal mode
expansions, and checker output. Existing direct test symbol references total 55
(49 output/extractor and six invalid-key references) across `support.rs`,
`binary_route_fixtures.rs`, and `long_chain.rs`, protecting exact payload,
source provenance, invalid-key fallback, active fixtures, and cross-route
isolation. Therefore this is move-only `design_drift` and no ZD0 test task is
needed. `spec_coverage_audit.md` remains unchanged because authority, behavior,
coverage credit, owner crate, and deferred status do not change. Forbidden
changes are route-set expansion, config/key/role/mode edits, chain
generalization, object/set coercion, assertion weakening, test/expectation
edits, and movement of adjacent type-assertion, asserted-head, or formula
routes.

## Task 263ZD Move Result

Task 263ZD moved only the fifteen approved fragments totaling 186 lines into
the existing private `type_elaboration/binary_routes.rs`, preserving original
raw hashes `c9abd07e...`, `76c6badc...`, `274e8231...`, `da89ee91...`,
`24c13cf8...`, `9bbfb507...`, `a3646c63...`, `ebfd9b1f...`, `c392ea7b...`,
`29d99bb5...`, `432ad380...`, `c1e39c32...`, `591afb95...`, `f1750caf...`,
and `cd31b66d...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is
token-identical and the combined normalized hash is `de18e68c...`. Adjacent
type-assertion, asserted-head, formula, and other route families remain in
`runner.rs`.

The expanded 3,791-line private owner has raw hash `3ce5e2f4...`, while
`runner.rs` is 6,246 lines with raw hash `e10f439e...`; the 333-line phase
facade has raw hash `e94c8b71...`. `long_chain_config.rs` remains 82 lines with
hash `3b0e2638...`. Invalid-key constants remain leaf-private; configs, three
test output wrappers, and three extractors cross only the test facade. The
normal facade adds only three production detail routes, and `binary_routes`
imports the shared object-terminal table directly from its sibling. Required
import cleanup only narrows generic binary source/output helpers that no longer
have a production consumer in `runner.rs`; the two binary source types remain
available normally for the production `output.rs` sibling.

The focused membership/equality/inequality filters each pass two tests, and all
272 crate unit tests pass. The raw and normalized 272-name list hashes, four
CLI byte hashes, active counts 96/4/188, plan 403/367, type coverage 235/223,
pass/fail 219/184, and 23 warnings/zero errors remain unchanged. Formatting,
all-target/all-feature Clippy with warnings denied, workspace tests, and diff
cleanliness pass. Task 263ZD is complete; fresh Task 263 inventory returns to
the remaining long-chain type-assertion/asserted-head route families.
`spec_coverage_audit.md` remains unchanged because no authority, behavior,
coverage credit, owner crate, or deferred status changed.

## Task 263ZE Pre-Move Inventory and Specification

Fresh dependency inventory selects only the local-mode long-chain reserved-
variable type-assertion route as the first bounded owner for a private
`type_elaboration/type_assertion_routes.rs` leaf. It forms five exact
`runner.rs` fragments: the invalid key at 505-506 (2 lines, `74d62809...`),
config at 3376-3388 (13, `e0f14b5b...`), production detail route at 4185-4198
(14, `79da5ff1...`), test-only output at 4860-4870 (11, `83d3b15e...`), and
extractor at 5970-5981 (12, `a9c40c0d...`). Total: 52 lines; combined raw hash
`5e321346...` and whitespace-normalized pre-move hash `2f3d7241...`.

Task 263ZE mechanically moves only those fragments into the new private leaf,
which imports the shared set-terminal table directly from sibling
`long_chain_config`. Only the production detail route crosses the normal phase
facade. The config, test-consumed output, and extractor cross under
`#[cfg(test)]`; the invalid-key constant remains leaf-private while a
config-derived runner test alias retains its existing name and value. No call
site, name, config value, exact seven-expansion set-terminal chain, asserted
builtin head, key, payload, ordering, fallback, or fail-closed behavior changes.

Canonical reserve/mode/type-assertion requirements apply unchanged. The one
active `.miz`/expectation pair and covered trace requirement exercise the real
source AST, resolver environment, seven set-terminal mode expansions, asserted
builtin-set head, and checker output. Existing direct test symbol references
total 69 across `support.rs`, `asserted_head_base.rs`, `long_chain.rs`, and
`remaining_bridges_and_nested_isolation.rs`, protecting exact payload, source
provenance, invalid-key fallback, active fixture, and cross-route isolation.
Therefore this is move-only `design_drift` and no ZE0 test task is needed.
`spec_coverage_audit.md` remains unchanged because authority, behavior,
coverage credit, owner crate, and deferred status do not change. Forbidden
changes are config/key/role/mode edits, chain or asserted-head generalization,
object/set coercion, assertion weakening, test/expectation edits, and movement
of any asserted-head or local-object-mode route.

## Task 263ZE Move Result

Task 263ZE moved only the five approved fragments totaling 52 lines into new
private `type_elaboration/type_assertion_routes.rs`, preserving original raw
hashes `74d62809...`, `e0f14b5b...`, `79da5ff1...`, `83d3b15e...`, and
`a9c40c0d...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is token-
identical and the combined normalized hash is `2f3d7241...`. Every asserted-
head route and the adjacent local-object-mode type-assertion route remain in
`runner.rs`.

The new 73-line private owner has raw hash `36549372...`; `runner.rs` is 6,197
lines with raw hash `a2c5aa11...`; and the 341-line phase facade has raw hash
`2d6c1c85...`. The unchanged 82-line `long_chain_config.rs` retains hash
`3b0e2638...`. The invalid-key constant remains leaf-private; the config,
test-consumed output, and extractor cross only the test facade. The normal
facade adds only the production detail route, and the owner imports the shared
set-terminal table directly from its sibling. The runner's test alias derives
the unchanged key from the moved config.

The focused filter passes two tests and all 272 crate unit tests pass. The raw
and normalized 272-name list hashes, four CLI byte hashes, active counts
96/4/188, plan 403/367, type coverage 235/223, pass/fail 219/184, and 23
warnings/zero errors remain unchanged. Formatting, all-target/all-feature
Clippy with warnings denied, workspace tests, diff cleanliness, and review-
only checks pass. Task 263ZE is complete; fresh Task 263 inventory returns to
the remaining long-chain asserted-head and local-object-mode type-assertion
route families. `spec_coverage_audit.md` remains unchanged because no
authority, behavior, coverage credit, owner crate, or deferred status changed.

## Task 263ZF Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, and real-consumer
inventory selects only the local-mode long-chain same-`ChainMode6` asserted-
head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 512-513 (2 lines, `e4633687...`), config at
3381-3393 (13, `1f16fdb8...`), production detail route at 4176-4187 (12,
`a027a240...`), test-only output at 4836-4844 (9, `42f4bfc7...`), and extractor
at 5934-5945 (12, `6b1f2ecb...`). Total: 48 lines; combined raw hash
`85282759...` and whitespace-normalized pre-move hash `5ed24905...`.

Task 263ZF mechanically moves only those fragments into the existing private
owner, which already imports the shared set-terminal table directly from
sibling `long_chain_config`. Only the production detail route crosses the
normal phase facade. The config, test-consumed output, and extractor cross
under `#[cfg(test)]`; the invalid-key constant remains leaf-private while a
config-derived runner test alias retains its existing name and value. No call
site, name, config value, exact seven-expansion set-terminal chain, same-mode
asserted-head relation, key, payload, ordering, fallback, or fail-closed
behavior changes.

Canonical reserved-theorem-variable and static type-assertion requirements
apply unchanged. The one active `.miz`/expectation pair and its dedicated
covered checker requirement exercise real source AST/resolver data, seven
set-terminal expansions, independent reserve/asserted `ChainMode6` source
sites, and real checker output. The four private test files retain 94 family-
name occurrences (`asserted_head_base.rs` 3, `asserted_head_fixtures.rs` 4,
`long_chain.rs` 70, and `remaining_bridges_and_nested_isolation.rs` 17),
protecting the exact payload, provenance, key fallback, active fixture,
corruption matrix, and cross-route isolation. Therefore this is move-only
`design_drift` and no ZF0 test task is needed. `spec_coverage_audit.md` remains
unchanged because authority, behavior, coverage credit, owner crate, and
deferred status do not change. Forbidden changes are config/key/role/mode
edits, asserted-head generalization, radix/multi-hop/local-object-mode route
moves, object/set coercion, assertion weakening, and test/expectation edits.

## Task 263ZF Move Result

Task 263ZF moved only the five approved fragments totaling 48 lines into the
existing private `type_elaboration/type_assertion_routes.rs`, preserving
original raw hashes `e4633687...`, `1f16fdb8...`, `a027a240...`, `42f4bfc7...`,
and `6b1f2ecb...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is token-
identical and the combined normalized hash is `5ed24905...`. Every radix,
multi-hop, local-object-mode, and other adjacent route remains in `runner.rs`.

The expanded 125-line private owner has raw hash `e3d046a9...`; `runner.rs` is
6,152 lines with raw hash `dd4cb898...`; and the 347-line phase facade has raw
hash `737c890e...`. The unchanged 82-line `long_chain_config.rs` retains hash
`3b0e2638...`. The invalid-key constant remains leaf-private; the config,
test-consumed output, and extractor cross only the test facade. The normal
facade adds only the production detail route, the owner retains its direct
sibling table import, and the runner's test alias derives the unchanged key
from the moved config. Production dispatch order and call identity are
unchanged.

The focused filter passes two tests and all 272 crate unit tests pass. The raw
and normalized 272-name list hashes, four CLI byte hashes, active counts
96/4/188, plan 403/367, type coverage 235/223, pass/fail 219/184, and 23
warnings/zero errors remain unchanged. Formatting, all-target/all-feature
Clippy with warnings denied, workspace tests, diff cleanliness, and review-
only checks pass. Task 263ZF is complete; fresh Task 263 inventory returns to
the radix, multi-hop, and local-object-mode long-chain asserted-head routes.
`spec_coverage_audit.md` remains unchanged because no authority, behavior,
coverage credit, owner crate, or deferred status changed.

## Task 263ZG Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, and real-consumer
inventory selects only the local-mode long-chain immediate-radix asserted-head
route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 519-520 (2 lines, `1c51cc95...`), config at
3386-3400 (15, `f573f4d7...`), production detail route at 4167-4178 (12,
`59f89066...`), test-only output at 4814-4822 (9, `66381eab...`), and extractor
at 5902-5913 (12, `c58a33b1...`). Total: 50 lines; combined raw hash
`9de63d06...` and whitespace-normalized pre-move hash `bfcb5927...`.

Task 263ZG mechanically moves only those fragments into the existing private
owner, which already imports the shared set-terminal table directly from
sibling `long_chain_config`. Only the production detail route crosses the
normal phase facade. The config, test-consumed output, and extractor cross
under `#[cfg(test)]`; the invalid-key constant remains leaf-private while a
config-derived runner test alias retains its existing name and value. No call
site, name, config value, exact seven-expansion set-terminal chain, immediate
`ChainMode6 -> ChainMode5` relation, key, payload, ordering, fallback, or fail-
closed behavior changes.

Canonical reserved-theorem-variable and static type-assertion requirements
apply unchanged. The one active `.miz`/expectation pair and its dedicated
covered Task 209 checker requirement exercise real source AST/resolver data,
seven set-terminal expansions, distinct reserve `ChainMode6` and asserted
`ChainMode5` source sites, the immediate bare radix edge, and real checker
output. The five private test files retain 73 family-name occurrences
(`support.rs` 1, `asserted_head_base.rs` 3, `asserted_head_fixtures.rs` 4,
`long_chain.rs` 48, and `remaining_bridges_and_nested_isolation.rs` 17),
protecting the exact payload, provenance, all-order corruption matrix, key
fallback, active fixture, and cross-route isolation. Therefore this is move-
only `design_drift` and no ZG0 test task is needed. `spec_coverage_audit.md`
remains unchanged because authority, behavior, coverage credit, owner crate,
and deferred status do not change. Forbidden changes are config/key/role/mode
edits, relation or chain generalization, same-mode/multi-hop/local-object-mode
route moves, object/set coercion, assertion weakening, and test/expectation
edits.

## Task 263ZG Move Result

Task 263ZG moved only the five approved fragments totaling 50 lines into the
existing private `type_elaboration/type_assertion_routes.rs`, preserving
original raw hashes `1c51cc95...`, `f573f4d7...`, `59f89066...`, `66381eab...`,
and `c58a33b1...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is token-
identical and the combined normalized hash is `bfcb5927...`. Every multi-hop,
local-object-mode, and other adjacent route remains in `runner.rs`.

The expanded 180-line private owner has raw hash `0a99d34d...`; `runner.rs` is
6,105 lines with raw hash `dd4c9b2a...`; and the 351-line phase facade has raw
hash `7e16d5dc...`. The unchanged 82-line `long_chain_config.rs` retains hash
`3b0e2638...`. The invalid-key constant remains leaf-private; the config,
test-consumed output, and extractor cross only the test facade. The normal
facade adds only the production detail route, the owner retains its direct
sibling table import, and the runner's test alias derives the unchanged key
from the moved config. Production dispatch order, the exact immediate radix
relation, and call identity are unchanged.

The focused filter passes two tests and all 272 crate unit tests pass. The raw
and normalized 272-name list hashes, four CLI byte hashes, active counts
96/4/188, plan 403/367, type coverage 235/223, pass/fail 219/184, and 23
warnings/zero errors remain unchanged. Formatting, all-target/all-feature
Clippy with warnings denied, workspace tests, diff cleanliness, and review-
only checks pass. Task 263ZG is complete; fresh Task 263 inventory returns to
the multi-hop and local-object-mode long-chain asserted-head routes.
`spec_coverage_audit.md` remains unchanged because no authority, behavior,
coverage credit, owner crate, or deferred status changed.

## Task 263ZH Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, and real-consumer
inventory selects only the local-mode long-chain two-hop asserted-head route as
the next bounded addition to private `type_elaboration/type_assertion_routes.rs`.
It forms five exact `runner.rs` fragments: the invalid key at 526-527 (2 lines,
`99d058e8...`), config at 3391-3406 (16, `2691b2e7...`), production detail route
at 4156-4167 (12, `15b0a146...`), test-only output at 4790-4798 (9,
`032f065d...`), and extractor at 5868-5879 (12, `c645fca8...`). Total: 51 lines;
combined raw hash `a9e3c846...` and whitespace-normalized pre-move hash
`b22e9463...`.

Task 263ZH mechanically moves only those fragments into the existing private
owner, retaining its direct sibling `long_chain_config` dependency. Only the
production detail route crosses the normal phase facade. The config, test-
consumed output, and extractor cross under `#[cfg(test)]`; the invalid-key
constant remains leaf-private while a config-derived runner test alias retains
its existing name and value. No call site, name, config value, exact seven-
expansion set-terminal chain, two-hop `ChainMode6 -> ChainMode5 -> ChainMode4`
relation, key, payload, ordering, fallback, or fail-closed behavior changes.

Canonical reserved-theorem-variable and static type-assertion requirements
apply unchanged. The one active `.miz`/expectation pair and its dedicated
covered Task 224 checker requirement exercise real source AST/resolver data,
seven set-terminal expansions, distinct reserve `ChainMode6` and asserted
`ChainMode4` source sites, the two exact bare relation edges, and real checker
output. `long_chain.rs` retains all 50 family-name occurrences protecting exact
payload, provenance, all-order corruption, key fallback, active fixture, and
cross-route isolation. Therefore this is move-only `design_drift` and no ZH0
test task is needed. `spec_coverage_audit.md` remains unchanged because
authority, behavior, coverage credit, owner crate, and deferred status do not
change. Forbidden changes are config/key/role/mode edits, relation or chain
generalization, immediate/deeper-hop/local-object-mode route moves, object/set
coercion, assertion weakening, and test/expectation edits.

## Task 263ZH Move Result

Task 263ZH moved only the five approved fragments totaling 51 lines into the
existing private `type_elaboration/type_assertion_routes.rs`, preserving
original raw hashes `99d058e8...`, `2691b2e7...`, `15b0a146...`, `032f065d...`,
and `c645fca8...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is token-
identical and the combined normalized hash is `b22e9463...`. The previously
moved immediate-radix route remains unchanged in `type_assertion_routes.rs`;
every deeper-hop, local-object-mode, and other adjacent route remains in
`runner.rs`.

The expanded private owner is 236 lines with raw hash `ce9630bc...`;
`runner.rs` is 6,057 lines with raw hash `d9c02f6a...`, and the phase facade is
355 lines with raw hash `c74a5326...`. The unchanged 82-line
`long_chain_config.rs` retains hash `3b0e2638...`. The invalid-key constant is
leaf-private; config, test-consumed output, and extractor cross only the test
facade, while the normal facade adds only the production detail route. The
owner retains its direct sibling table import, the runner test alias derives
the unchanged key from the moved config, and production dispatch order, exact
two-hop relation, and call identity are unchanged.

The focused two tests and all 272 crate unit tests pass. Raw/normalized
272-name list hashes, four CLI byte hashes, active counts 96/4/188, plan
403/367, type coverage 235/223, pass/fail 219/184, and 23 warnings/zero errors
remain unchanged. Formatting, all-target/all-feature Clippy with warnings
denied, workspace tests, diff cleanliness, and review-only checks pass. Task
263ZH is complete; fresh Task 263 inventory returns to the deeper-hop and
local-object-mode long-chain asserted-head routes. `spec_coverage_audit.md`
remains unchanged because no authority, behavior, coverage credit, owner
crate, or deferred status changed.

## Task 263ZI Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the local-mode long-chain three-hop
asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 533-534 (2 lines, `6265db24...`), config at
3396-3412 (17, `84937c90...`), production detail route at 4144-4156 (13,
`6393cffc...`), test-only output at 4765-4774 (10, `ee67b1d6...`), and extractor
at 5833-5844 (12, `bda2d7a2...`). Total: 54 lines; combined raw hash
`32c6f854...` and whitespace-normalized pre-move hash `0082cb9f...`.

Task 263ZI mechanically moves only those fragments into the existing private
owner, retaining its direct sibling `long_chain_config` dependency. Only the
production detail route crosses the normal phase facade. The config, test-
consumed output, and extractor cross under `#[cfg(test)]`; the invalid-key
constant remains leaf-private while a config-derived runner test alias retains
its existing name and value. No public API or call site, name, config value,
exact seven-expansion set-terminal chain, three-hop
`ChainMode6 -> ChainMode5 -> ChainMode4 -> ChainMode3` relation, key, payload,
ordering, fallback, or fail-closed behavior changes.

Canonical reserved-theorem-variable and static type-assertion requirements
apply unchanged. The one active `.miz`/expectation pair and its dedicated
covered Task 226 checker requirement exercise real source AST/resolver data,
seven set-terminal expansions, distinct reserve `ChainMode6` and asserted
`ChainMode3` source sites, the three exact bare relation edges, and real
checker output. `long_chain.rs` retains all 42 family-name occurrences
protecting exact payload, provenance, all-order corruption, key fallback,
active fixture, and cross-route isolation. Therefore this is move-only
`design_drift` and no ZI0 test task is needed. `spec_coverage_audit.md` remains
unchanged because authority, behavior, coverage credit, owner crate, and
deferred status do not change. Forbidden changes are config/key/role/mode
edits, relation or chain generalization, immediate/two-hop/four-or-deeper/
local-object-mode route moves, object/set coercion, assertion weakening, and
test/expectation edits.

## Task 263ZI Move Result

Task 263ZI moved only the five approved fragments totaling 54 lines into the
existing private `type_elaboration/type_assertion_routes.rs`, preserving
original raw hashes `6265db24...`, `84937c90...`, `6393cffc...`, `ee67b1d6...`,
and `bda2d7a2...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is token-
identical and the combined normalized hash is `0082cb9f...`. The previously
moved immediate/two-hop routes remain unchanged in `type_assertion_routes.rs`;
every four-or-deeper, local-object-mode, and other adjacent route remains in
`runner.rs`.

The expanded private owner is 295 lines with raw hash `f6dbf168...`;
`runner.rs` is 6,006 lines with raw hash `48b37dfe...`, and the phase facade is
359 lines with raw hash `b44f5910...`. The unchanged 82-line
`long_chain_config.rs` retains hash `3b0e2638...`. The invalid-key constant is
leaf-private; config, test-consumed output, and extractor cross only the test
facade, while the normal facade adds only the production detail route. The
owner retains its direct sibling table import, the runner test alias derives
the unchanged key from the moved config, and production dispatch order, exact
three-hop relation, and call identity are unchanged.

The focused two tests and all 272 crate unit tests pass. Raw/normalized
272-name list hashes, four CLI byte hashes, active counts 96/4/188, plan
403/367, type coverage 235/223, pass/fail 219/184, and 23 warnings/zero errors
remain unchanged. Formatting, all-target/all-feature Clippy with warnings
denied, workspace tests, diff cleanliness, and review-only checks pass. Task
263ZI is complete; fresh Task 263 inventory returns to the four-or-deeper and
local-object-mode long-chain asserted-head routes. `spec_coverage_audit.md`
remains unchanged because no authority, behavior, coverage credit, owner
crate, or deferred status changed.

## Task 263ZJ Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the local-mode long-chain four-hop
asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 540-541 (2 lines, `4b7810fc...`), config at
3401-3418 (18, `f6832e47...`), production detail route at 4131-4143 (13,
`4d5e0688...`), test-only output at 4738-4747 (10, `2066549a...`), and extractor
at 5795-5806 (12, `150e478c...`). Total: 55 lines; combined raw hash
`9a44e3fb...` and whitespace-normalized pre-move hash `23488d36...`.

Task 263ZJ mechanically moves only those fragments into the existing private
owner, retaining its direct sibling `long_chain_config` dependency. Only the
production detail route crosses the normal phase facade. The config, test-
consumed output, and extractor cross under `#[cfg(test)]`; the invalid-key
constant remains leaf-private while a config-derived runner test alias retains
its existing name and value. No public API or call site, name, config value,
exact seven-expansion set-terminal chain, four-hop
`ChainMode6 -> ChainMode5 -> ChainMode4 -> ChainMode3 -> ChainMode2` relation,
key, payload, ordering, fallback, or fail-closed behavior changes.

Canonical reserved-theorem-variable and static type-assertion requirements
apply unchanged. The one active `.miz`/expectation pair and its dedicated
covered Task 228 checker requirement exercise real source AST/resolver data,
seven set-terminal expansions, distinct reserve `ChainMode6` and asserted
`ChainMode2` source sites, the four exact bare relation edges, and real checker
output. `long_chain.rs` retains all 34 family-name occurrences protecting
exact payload, provenance, all-order corruption, key fallback, active fixture,
and cross-route isolation. Therefore this is move-only `design_drift` and no
ZJ0 test task is needed. `spec_coverage_audit.md` remains unchanged because
authority, behavior, coverage credit, owner crate, and deferred status do not
change. Forbidden changes are config/key/role/mode edits, relation or chain
generalization, immediate/two/three/five/six-hop/local-object-mode route moves,
object/set coercion, assertion weakening, and test/expectation edits.

## Task 263ZJ Move Result

Task 263ZJ moved only the five approved fragments totaling 55 lines into the
existing private `type_elaboration/type_assertion_routes.rs`, preserving
original raw hashes `4b7810fc...`, `f6832e47...`, `4d5e0688...`, `2066549a...`,
and `150e478c...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is token-
identical and the combined normalized hash is `23488d36...`. The previously
moved immediate/two/three-hop routes remain unchanged in
`type_assertion_routes.rs`; every five/six-hop, local-object-mode, and other
adjacent route remains in `runner.rs`.

The expanded private owner is 355 lines with raw hash `7dc607a4...`;
`runner.rs` is 5,954 lines with raw hash `db5857bd...`, and the phase facade is
363 lines with raw hash `51a3b0d4...`. The unchanged 82-line
`long_chain_config.rs` retains hash `3b0e2638...`. The invalid-key constant is
leaf-private; config, test-consumed output, and extractor cross only the test
facade, while the normal facade adds only the production detail route. The
owner retains its direct sibling table import, the runner test alias derives
the unchanged key from the moved config, and production dispatch order, exact
four-hop relation, and call identity are unchanged.

The focused two tests and all 272 crate unit tests pass. Raw/normalized
272-name list hashes, four CLI byte hashes, active counts 96/4/188, plan
403/367, type coverage 235/223, pass/fail 219/184, and 23 warnings/zero errors
remain unchanged. Formatting, all-target/all-feature Clippy with warnings
denied, workspace tests, diff cleanliness, and review-only checks pass. Task
263ZJ is complete; fresh Task 263 inventory returns to the five/six-hop and
local-object-mode long-chain asserted-head routes. `spec_coverage_audit.md`
remains unchanged because no authority, behavior, coverage credit, owner
crate, or deferred status changed.

## Task 263ZK Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the local-mode long-chain five-hop
asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 547-548 (2 lines, `1d5a5452...`), config at
3406-3424 (19, `abf23b93...`), production detail route at 4117-4129 (13,
`448393c6...`), test-only output at 4710-4719 (10, `81db1ea3...`), and extractor
at 5756-5767 (12, `432bb0d3...`). Total: 56 lines; combined raw hash
`cacef95c...` and whitespace-normalized pre-move hash `2266a3d0...`.

Task 263ZK mechanically moves only those fragments into the existing private
owner, retaining its direct sibling `long_chain_config` dependency. Only the
production detail route crosses the normal phase facade. The config, test-
consumed output, and extractor cross under `#[cfg(test)]`; the invalid-key
constant remains leaf-private while a config-derived runner test alias retains
its existing name and value. No public API or call site, name, config value,
exact seven-expansion set-terminal chain, five-hop
`ChainMode6 -> ChainMode5 -> ChainMode4 -> ChainMode3 -> ChainMode2 ->
ChainMode1` relation, key, payload, ordering, fallback, or fail-closed behavior
changes.

Canonical reserved-theorem-variable and static type-assertion requirements
apply unchanged. The one active `.miz`/expectation pair and its dedicated
covered Task 230 checker requirement exercise real source AST/resolver data,
seven set-terminal expansions, distinct reserve `ChainMode6` and asserted
`ChainMode1` source sites, the five exact bare relation edges, and real checker
output. `long_chain.rs` retains all 25 snake-case family-name occurrences protecting exact
payload, provenance, all-order corruption, key fallback, active fixture, and
cross-route isolation. Therefore this is move-only `design_drift` and no ZK0
test task is needed. `spec_coverage_audit.md` remains unchanged because
authority, behavior, coverage credit, owner crate, and deferred status do not
change. Forbidden changes are config/key/role/mode edits, relation or chain
generalization, immediate/two/three/four/six-hop/local-object-mode route moves,
object/set coercion, assertion weakening, and test/expectation edits.

## Task 263ZK Move Result

Task 263ZK moved only the five approved fragments totaling 56 lines into the
existing private `type_elaboration/type_assertion_routes.rs`, preserving
original raw hashes `1d5a5452...`, `abf23b93...`, `448393c6...`, `81db1ea3...`,
and `432bb0d3...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is token-
identical and the combined normalized hash is `2266a3d0...`. The previously
moved immediate/two/three/four-hop routes remain unchanged in
`type_assertion_routes.rs`; the six-hop, every local-object-mode, and every
other adjacent route remains in `runner.rs`.

The expanded private owner is 416 lines with raw hash `2395aed6...`;
`runner.rs` is 5,901 lines with raw hash `bbe6b7f2...`, and the phase facade is
367 lines with raw hash `9ca398f8...`. The unchanged 82-line
`long_chain_config.rs` retains hash `3b0e2638...`. The invalid-key constant is
leaf-private; config, test-consumed output, and extractor cross only the test
facade, while the normal facade adds only the production detail route. The
owner retains its direct sibling table import, the runner test alias derives
the unchanged key from the moved config, and production dispatch order, exact
five-hop relation, call identity, and terminal fail-closed fallback are
unchanged.

The focused two tests and all 272 crate unit tests pass. Raw/normalized
272-name list hashes, four CLI byte hashes, active counts 96/4/188, plan
403/367, type coverage 235/223, pass/fail 219/184, and 23 warnings/zero errors
remain unchanged. Formatting, all-target/all-feature Clippy with warnings
denied, workspace tests, diff cleanliness, and review-only checks pass. Task
263ZK is complete; fresh Task 263 inventory returns to the six-hop and local-
object-mode long-chain asserted-head routes. `spec_coverage_audit.md` remains
unchanged because no authority, behavior, coverage credit, owner crate, or
deferred status changed.

## Task 263ZL Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the local-mode long-chain six-hop
asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 554-555 (2 lines, `ec22ef78...`), config at
3411-3430 (20, `582d2c60...`), production detail route at 4102-4113 (12,
`f8349031...`), test-only output at 4681-4689 (9, `aa261362...`), and extractor
at 5716-5727 (12, `575ead8d...`). Total: 55 lines; combined raw hash
`7f677c2e...` and whitespace-normalized pre-move hash `b8fba0fe...`.

Task 263ZL mechanically moves only those fragments into the existing private
owner, retaining its direct sibling `long_chain_config` dependency. Only the
production detail route crosses the normal phase facade. The config, test-
consumed output, and extractor cross under `#[cfg(test)]`; the invalid-key
constant remains leaf-private while a config-derived runner test alias retains
its existing name and value. No public API or call site, name, config value,
exact seven-expansion set-terminal chain, full-distance six-hop
`ChainMode6 -> ChainMode5 -> ChainMode4 -> ChainMode3 -> ChainMode2 ->
ChainMode1 -> BaseMode` relation, key, payload, ordering, fallback, or fail-
closed behavior changes.

Canonical reserved-theorem-variable and static type-assertion requirements
apply unchanged. The one active `.miz`/expectation pair and its dedicated
covered Task 234 checker requirement exercise real source AST/resolver data,
seven set-terminal expansions, distinct reserve `ChainMode6` and asserted
`BaseMode` source sites, the six exact bare relation edges, and real checker
output. `long_chain.rs` retains all 18 snake-case family-name occurrences plus
the all-5,039-order finite corruption matrix, all 56 prior-owner isolation,
immutable output, key fallback, active fixture, and real sidecar. Therefore
this is move-only `design_drift` and no ZL0 test task is needed.
`spec_coverage_audit.md` remains unchanged because authority, behavior,
coverage credit, owner crate, and deferred status do not change. Forbidden
changes are config/key/role/mode edits, relation or chain generalization,
same-mode/immediate/two/three/four/five-hop/local-object-mode route moves,
object/set coercion, assertion weakening, and test/expectation edits.

## Task 263ZL Move Result

Task 263ZL moved only the five approved fragments totaling 55 lines into the
existing private `type_elaboration/type_assertion_routes.rs`, preserving
original raw hashes `ec22ef78...`, `582d2c60...`, `f8349031...`, `aa261362...`,
and `575ead8d...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is token-
identical and the combined normalized hash is `b8fba0fe...`. All previously
moved local-mode routes remain unchanged in `type_assertion_routes.rs`; every
local-object-mode and other adjacent route remains in `runner.rs`.

The expanded private owner is 476 lines with raw hash `095eab00...`;
`runner.rs` is 5,849 lines with raw hash `952a1d7f...`, and the phase facade is
369 lines with raw hash `2b473071...`. The unchanged 82-line
`long_chain_config.rs` retains hash `3b0e2638...`. The invalid-key constant is
leaf-private; config, test-consumed output, and extractor cross only the test
facade, while the normal facade adds only the production detail route. The
owner retains its direct sibling table import, the runner test alias derives
the unchanged key from the moved config, and production dispatch order, exact
six-hop relation, call identity, and terminal fail-closed fallback are
unchanged.

The first focused compile correctly failed because the move removed the last
`runner.rs` consumer of `SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS`. The repair
removed only that stale runner import and phase-facade re-export. Direct sibling
consumers in `binary_routes.rs` and `type_assertion_routes.rs` remain, while the
local-object table stays exposed for retained runner consumers. The focused two
tests then pass, as do all 272 crate unit tests. Raw/normalized 272-name list
hashes, four CLI byte hashes, active counts 96/4/188, plan 403/367, type
coverage 235/223, pass/fail 219/184, and 23 warnings/zero errors remain
unchanged. Formatting, all-target/all-feature Clippy with warnings denied,
workspace tests, diff cleanliness, and review-only checks pass. Task 263ZL is
complete; fresh Task 263 inventory returns only to the local-object-mode long-
chain asserted-head routes. `spec_coverage_audit.md` remains unchanged because
no authority, behavior, coverage credit, owner crate, or deferred status
changed.

## Task 263ZM Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the local-object-mode long-chain six-
hop asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 561-563 (3 lines, `ac31b964...`), config at
3416-3435 (20, `56bf0820...`), production detail route at 4086-4098 (13,
`8011c4b0...`), test-only output at 4652-4661 (10, `b24222d0...`), and extractor
at 5677-5688 (12, `dc57ecdb...`). Total: 58 lines; combined raw hash
`770ab2db...` and whitespace-normalized pre-move hash `a489a76f...`.

Task 263ZM mechanically moves only those fragments into the existing private
owner, adding its direct sibling `long_chain_config` local-object table
dependency. Only the production detail route crosses the normal phase facade.
The config, test-consumed output, and extractor cross under `#[cfg(test)]`; the
invalid-key constant remains leaf-private while a config-derived runner test
alias retains its existing name and value. No public API or call site, name,
config value, exact seven-expansion object-terminal chain, full-distance six-
hop `ChainObjectMode6 -> ChainObjectMode5 -> ChainObjectMode4 ->
ChainObjectMode3 -> ChainObjectMode2 -> ChainObjectMode1 -> BaseObjectMode`
relation, key, payload, ordering, fallback, or fail-closed behavior changes.

Canonical reserved-theorem-variable and static type-assertion requirements
apply unchanged. The one active `.miz`/expectation pair and its dedicated
covered Task 236 checker requirement exercise real source AST/resolver data,
seven object-terminal expansions, distinct reserve `ChainObjectMode6` and
asserted `BaseObjectMode` source sites, the six exact bare relation edges, and
real checker output without object/set coercion. `long_chain.rs` retains all 14
snake-case family-name occurrences plus the all-5,039-order finite corruption
matrix, all 57 prior-owner isolation, immutable output, key fallback, active
fixture, and real sidecar. Therefore this is move-only `design_drift` and no
ZM0 test task is needed. `spec_coverage_audit.md` remains unchanged because
authority, behavior, coverage credit, owner crate, and deferred status do not
change. Forbidden changes are config/key/role/mode edits, relation or chain
generalization, local-mode or other local-object-mode route moves, object/set
coercion, assertion weakening, and test/expectation edits.

## Task 263ZM Move Result

Task 263ZM moved only the five approved fragments totaling 58 lines into the
existing private `type_elaboration/type_assertion_routes.rs`, preserving
original raw hashes `ac31b964...`, `56bf0820...`, `8011c4b0...`, `b24222d0...`,
and `dc57ecdb...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is token-
identical and the combined normalized hash is `a489a76f...`. All previously
moved local-mode routes remain unchanged in `type_assertion_routes.rs`; every
other local-object-mode and adjacent route remains in `runner.rs`.

The expanded private owner is 541 lines with raw hash `04c02f75...`;
`runner.rs` is 5,794 lines with raw hash `721574ab...`, and the phase facade is
373 lines with raw hash `bf96abb3...`. The unchanged 82-line
`long_chain_config.rs` retains hash `3b0e2638...`. The invalid-key constant is
leaf-private; config, test-consumed output, and extractor cross only the test
facade, while the normal facade adds only the production detail route. The
owner imports the object-terminal table directly from its sibling, the runner
test alias derives the unchanged key from the moved config, and production
dispatch order, exact object-terminal six-hop relation, call identity, and
terminal fail-closed fallback are unchanged.

The focused two tests, all 272 crate unit tests, and the full relevant-crate
suite pass. Raw/normalized 272-name list hashes, four CLI byte hashes, active
counts 96/4/188, plan 403/367, type coverage 235/223, pass/fail 219/184, and 23
warnings/zero errors remain unchanged. Formatting, all-target/all-feature
Clippy with warnings denied, workspace tests, diff cleanliness, and review-
only checks pass. Task 263ZM is complete; fresh Task 263 inventory returns to
the remaining local-object-mode long-chain asserted-head routes.
`spec_coverage_audit.md` remains unchanged because no authority, behavior,
coverage credit, owner crate, or deferred status changed.

## Task 263ZN Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the local-object-mode long-chain five-
hop asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 568-570 (3 lines, `fde751f2...`), config at
3420-3438 (19, `a2e917f5...`), production detail route at 4069-4081 (13,
`b4e0cd1e...`), test-only output at 4621-4630 (10, `9c0fa75e...`), and extractor
at 5635-5646 (12, `4be72697...`). Total: 57 lines; combined raw hash
`a1e6e85b...` and whitespace-normalized pre-move hash `66a0a9c1...`.

Task 263ZN mechanically moves only those fragments into the existing private
owner, retaining its direct sibling `long_chain_config` object-table
dependency. Only the production detail route crosses the normal phase facade.
The config, test-consumed output, and extractor cross under `#[cfg(test)]`; the
invalid-key constant remains leaf-private while a config-derived runner test
alias retains its existing name and value. No public API or call site, name,
config value, exact seven-expansion object-terminal chain, five-hop
`ChainObjectMode6 -> ChainObjectMode5 -> ChainObjectMode4 ->
ChainObjectMode3 -> ChainObjectMode2 -> ChainObjectMode1` relation, terminal-
only `ChainObjectMode1 -> BaseObjectMode -> object` normalization, key,
payload, ordering, fallback, or fail-closed behavior changes.

Canonical reserved-theorem-variable and static type-assertion requirements
apply unchanged. The one active `.miz`/expectation pair and its dedicated
covered Task 231 checker requirement exercise real source AST/resolver data,
seven object-terminal expansions, distinct reserve `ChainObjectMode6` and
asserted `ChainObjectMode1` source sites, the five exact bare relation edges,
and real checker output without object/set coercion. `long_chain.rs` retains all
22 snake-case family-name occurrences plus the all-5,039-order finite
corruption matrix, all 55 prior-owner isolation, immutable output, key
fallback, active fixture, and real sidecar. Therefore this is move-only
`design_drift` and no ZN0 test task is needed. `spec_coverage_audit.md` remains
unchanged because authority, behavior, coverage credit, owner crate, and
deferred status do not change. Forbidden changes are config/key/role/mode
edits, relation or chain generalization, local-mode or other local-object-mode
route moves, object/set coercion, assertion weakening, and test/expectation
edits.

## Task 263ZN Move Result

Task 263ZN moved only the five approved fragments totaling 57 lines into the
existing private `type_elaboration/type_assertion_routes.rs`, preserving
original raw hashes `fde751f2...`, `a2e917f5...`, `b4e0cd1e...`, `9c0fa75e...`,
and `4be72697...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is token-
identical and the combined normalized hash is `66a0a9c1...`. All previously
moved local-mode and local-object-mode six-hop routes remain unchanged in
`type_assertion_routes.rs`; every other local-object-mode and adjacent route
remains in `runner.rs`.

The expanded private owner is 603 lines with raw hash `e9fb3b88...`;
`runner.rs` is 5,740 lines with raw hash `e35165d1...`, and the phase facade is
377 lines with raw hash `946dcebe...`. The unchanged 82-line
`long_chain_config.rs` retains hash `3b0e2638...`. The invalid-key constant is
leaf-private; config, test-consumed output, and extractor cross only the test
facade, while the normal facade adds only the production detail route. The
owner retains its direct object-terminal table import, the runner test alias
derives the unchanged key from the moved config, and production dispatch order,
exact object-terminal five-hop relation, call identity, and terminal fail-
closed fallback are unchanged.

The focused two tests, all 272 crate unit tests, and the full relevant-crate
suite pass. Raw/normalized 272-name list hashes, four CLI byte hashes, active
counts 96/4/188, plan 403/367, type coverage 235/223, pass/fail 219/184, and 23
warnings/zero errors remain unchanged. Formatting, all-target/all-feature
Clippy with warnings denied, workspace tests, diff cleanliness, and review-
only checks pass. Task 263ZN is complete; fresh Task 263 inventory returns to
the remaining local-object-mode long-chain asserted-head routes.
`spec_coverage_audit.md` remains unchanged because no authority, behavior,
coverage credit, owner crate, or deferred status changed.

## Task 263ZO Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the local-object-mode long-chain four-
hop asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 575-577 (3 lines, `c73a1ed8...`), config at
3475-3492 (18, `aa4574ef...`), production detail route at 4095-4107 (13,
`be90c9c8...`), test-only output at 4624-4633 (10, `b09aa3cd...`), and extractor
at 5633-5644 (12, `e89973e7...`). Total: 56 lines; combined raw hash
`2a5cb09a...` and whitespace-normalized pre-move hash `9452ed92...`.

Task 263ZO mechanically moves only those fragments into the existing private
owner, retaining its direct sibling `long_chain_config` object-table
dependency. Only the production detail route crosses the normal phase facade.
The config, test-consumed output, and extractor cross under `#[cfg(test)]`; the
invalid-key constant remains leaf-private while a config-derived runner test
alias retains its existing name and value. No public API or call site, name,
config value, exact seven-expansion object-terminal chain, four-hop
`ChainObjectMode6 -> ChainObjectMode5 -> ChainObjectMode4 ->
ChainObjectMode3 -> ChainObjectMode2` relation, terminal-only `ChainObjectMode2
-> ChainObjectMode1 -> BaseObjectMode -> object` normalization, key, payload,
ordering, fallback, or fail-closed behavior changes.

Canonical reserved-theorem-variable and static type-assertion requirements
apply unchanged. The one active `.miz`/expectation pair and its dedicated
covered Task 229 checker requirement exercise real source AST/resolver data,
seven object-terminal expansions, distinct reserve `ChainObjectMode6` and
asserted `ChainObjectMode2` source sites, the four exact bare relation edges,
and real checker output without object/set coercion. `long_chain.rs` retains all
28 snake-case family-name occurrences plus the all-5,039-order finite
corruption matrix, all 53 prior-owner isolation, immutable output, key
fallback, active fixture, and real sidecar. Therefore this is move-only
`design_drift` and no ZO0 test task is needed. `spec_coverage_audit.md` remains
unchanged because authority, behavior, coverage credit, owner crate, and
deferred status do not change. Forbidden changes are config/key/role/mode
edits, relation or chain generalization, local-mode or other local-object-mode
route moves, object/set coercion, assertion weakening, and test/expectation
edits.

## Task 263ZO Move Result

Task 263ZO moved only the five approved fragments totaling 56 lines into the
existing private `type_elaboration/type_assertion_routes.rs`, preserving
original raw hashes `c73a1ed8...`, `aa4574ef...`, `be90c9c8...`, `b09aa3cd...`,
and `e89973e7...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is token-
identical and the combined normalized hash is `9452ed92...`. All previously
moved local-mode and local-object-mode six-/five-hop routes remain unchanged in
`type_assertion_routes.rs`; every other local-object-mode and adjacent route
remains in `runner.rs`.

The expanded private owner is 664 lines with raw hash `9da1dffd...`;
`runner.rs` is 5,687 lines with raw hash `eb33ccce...`, and the phase facade is
381 lines with raw hash `4ca061cc...`. The unchanged 82-line
`long_chain_config.rs` retains hash `3b0e2638...`. The invalid-key constant is
leaf-private; config, test-consumed output, and extractor cross only the test
facade, while the normal facade adds only the production detail route. The
owner retains its direct object-terminal table import, the runner test alias
derives the unchanged key from the moved config, and production dispatch order,
exact object-terminal four-hop relation, call identity, and terminal fail-
closed fallback are unchanged.

The focused two tests, all 272 crate unit tests, and the full relevant-crate
suite pass. Raw/normalized 272-name list hashes, four CLI byte hashes, active
counts 96/4/188, plan 403/367, type coverage 235/223, pass/fail 219/184, and 23
warnings/zero errors remain unchanged. Formatting, all-target/all-feature
Clippy with warnings denied, workspace tests, diff cleanliness, and review-
only implementation checks pass. Task 263ZO is complete; fresh Task 263
inventory returns to the remaining local-object-mode long-chain asserted-head
routes. `spec_coverage_audit.md` remains unchanged because no authority,
behavior, coverage credit, owner crate, or deferred status changed.

## Task 263ZP Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the local-object-mode long-chain
three-hop asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 587-589 (3 lines, `787cf360...`), config at
3461-3477 (17, `94ceb3f4...`), production detail route at 4066-4078 (13,
`94b06181...`), test-only output at 4584-4593 (10, `bd960eb3...`), and extractor
at 5580-5591 (12, `45e07c6a...`). Total: 55 lines; combined raw hash
`4af642ff...` and whitespace-normalized pre-move hash `ecc4d42e...`.

Task 263ZP mechanically moves only those fragments into the existing private
owner, retaining its direct sibling `long_chain_config` object-table
dependency. Only the production detail route crosses the normal phase facade.
The config, test-consumed output, and extractor cross under `#[cfg(test)]`; the
invalid-key constant remains leaf-private while a config-derived runner test
alias retains its existing name and value. No public API or call site, name,
config value, exact seven-expansion object-terminal chain, three-hop
`ChainObjectMode6 -> ChainObjectMode5 -> ChainObjectMode4 ->
ChainObjectMode3` relation, terminal-only `ChainObjectMode3 -> ChainObjectMode2
-> ChainObjectMode1 -> BaseObjectMode -> object` normalization, key, payload,
ordering, fallback, or fail-closed behavior changes.

Canonical reserved-theorem-variable and static type-assertion requirements
apply unchanged. The one active `.miz`/expectation pair and its dedicated
covered Task 227 checker requirement exercise real source AST/resolver data,
seven object-terminal expansions, distinct reserve `ChainObjectMode6` and
asserted `ChainObjectMode3` source sites, the three exact bare relation edges,
and real checker output without object/set coercion. `long_chain.rs` retains all
36 snake-case family-name occurrences plus the all-5,039-order finite
corruption matrix, all 51 prior-owner isolation, immutable output, key
fallback, active fixture, and real sidecar. Therefore this is move-only
`design_drift` and no ZP0 test task is needed. `spec_coverage_audit.md` remains
unchanged because authority, behavior, coverage credit, owner crate, and
deferred status do not change. Forbidden changes are config/key/role/mode
edits, relation or chain generalization, local-mode or other local-object-mode
route moves, object/set coercion, assertion weakening, and test/expectation
edits.

## Task 263ZP Move Result

Task 263ZP moved only the five approved fragments totaling 55 lines into the
existing private `type_elaboration/type_assertion_routes.rs`, preserving
original raw hashes `787cf360...`, `94ceb3f4...`, `94b06181...`, `bd960eb3...`,
and `45e07c6a...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is token-
identical and the combined normalized hash is `ecc4d42e...`. All previously
moved local-mode and local-object-mode six-/five-/four-hop routes remain
unchanged in `type_assertion_routes.rs`; every other local-object-mode and
adjacent route remains in `runner.rs`.

The expanded private owner is 724 lines with raw hash `a3e7d1be...`;
`runner.rs` is 5,635 lines with raw hash `aea9e1af...`, and the phase facade is
385 lines with raw hash `76309099...`. The unchanged 82-line
`long_chain_config.rs` retains hash `3b0e2638...`. The invalid-key constant is
leaf-private; config, test-consumed output, and extractor cross only the test
facade, while the normal facade adds only the production detail route. The
owner retains its direct object-terminal table import, the runner test alias
derives the unchanged key from the moved config, and production dispatch order,
exact object-terminal three-hop relation, call identity, and terminal fail-
closed fallback are unchanged.

The focused two tests, all 272 crate unit tests, and the full relevant-crate
suite pass. Raw/normalized 272-name list hashes, four CLI byte hashes, active
counts 96/4/188, plan 403/367, type coverage 235/223, pass/fail 219/184, and 23
warnings/zero errors remain unchanged. Formatting, all-target/all-feature
Clippy with warnings denied, workspace tests, diff cleanliness, and review-
only implementation checks pass. Task 263ZP is complete; fresh Task 263
inventory returns to the remaining local-object-mode long-chain asserted-head
routes. `spec_coverage_audit.md` remains unchanged because no authority,
behavior, coverage credit, owner crate, or deferred status changed.

## Task 263ZQ Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the local-object-mode long-chain two-
hop asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 588-590 (3 lines, `afb54f75...`), config at
3448-3463 (16, `96edc075...`), production detail route at 4038-4050 (13,
`e88f6a56...`), test-only output at 4545-4554 (10, `d8d67f83...`), and extractor
at 5528-5539 (12, `09ea9384...`). Total: 54 lines; combined raw hash
`87f3069b...` and whitespace-normalized pre-move hash `18f90f83...`.

Task 263ZQ mechanically moves only those fragments into the existing private
owner, retaining its direct sibling `long_chain_config` object-table
dependency. Only the production detail route crosses the normal phase facade.
The config, test-consumed output, and extractor cross under `#[cfg(test)]`; the
invalid-key constant remains leaf-private while a config-derived runner test
alias retains its existing name and value. No public API or call site, name,
config value, exact seven-expansion object-terminal chain, two-hop
`ChainObjectMode6 -> ChainObjectMode5 -> ChainObjectMode4` relation, terminal-
only `ChainObjectMode4 -> ChainObjectMode3 -> ChainObjectMode2 ->
ChainObjectMode1 -> BaseObjectMode -> object` normalization, key, payload,
ordering, fallback, or fail-closed behavior changes.

Canonical reserved-theorem-variable and static type-assertion requirements
apply unchanged. The one active `.miz`/expectation pair and its dedicated
covered Task 225 checker requirement exercise real source AST/resolver data,
seven object-terminal expansions, distinct reserve `ChainObjectMode6` and
asserted `ChainObjectMode4` source sites, the two exact bare relation edges,
and real checker output without object/set coercion. `long_chain.rs` retains all
44 snake-case family-name occurrences plus the all-5,039-order finite
corruption matrix, all 49 prior-owner isolation, immutable output, key
fallback, active fixture, and real sidecar. Therefore this is move-only
`design_drift` and no ZQ0 test task is needed. `spec_coverage_audit.md` remains
unchanged because authority, behavior, coverage credit, owner crate, and
deferred status do not change. Forbidden changes are config/key/role/mode
edits, relation or chain generalization, local-mode or other local-object-mode
route moves, object/set coercion, assertion weakening, and test/expectation
edits.

`spec_coverage_audit.md` remains unchanged for Tasks 262N0-262Q because these
tasks preserve authority, behavior, coverage credit, owner crate, and deferred
status. Forbidden changes are accepted-shape expansion, route generalization,
config or result-role edits, payload/detail/diagnostic/order changes,
assertion weakening, test deletion or ignore, and checker/output movement.

## Task 263ZQ Move Result

Task 263ZQ moved only the five approved fragments totaling 54 lines into the
existing private `type_elaboration/type_assertion_routes.rs`, preserving
original raw hashes `afb54f75...`, `96edc075...`, `e88f6a56...`, `d8d67f83...`,
and `09ea9384...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is token-
identical and the combined normalized hash is `18f90f83...`. All previously
moved local-mode and local-object-mode six-/five-/four-/three-hop routes remain
unchanged in `type_assertion_routes.rs`; every other local-object-mode and
adjacent route remains in `runner.rs`.

The expanded private owner is 783 lines with raw hash `4d72d185...`;
`runner.rs` is 5,584 lines with raw hash `44a2b129...`, and the phase facade is
389 lines with raw hash `32d06bf1...`. The unchanged 82-line
`long_chain_config.rs` retains hash `3b0e2638...`. The invalid-key constant is
leaf-private; config, test-consumed output, and extractor cross only the test
facade, while the normal facade adds only the production detail route. The
owner retains its direct object-terminal table import, the runner test alias
derives the unchanged key from the moved config, and production dispatch order,
exact object-terminal two-hop relation, call identity, and terminal fail-closed
fallback are unchanged.

The focused two tests, all 272 crate unit tests, and the full relevant-crate
suite pass. Raw/normalized 272-name list hashes, four CLI byte hashes, active
counts 96/4/188, plan 403/367, type coverage 235/223, pass/fail 219/184, and 23
warnings/zero errors remain unchanged. Formatting, all-target/all-feature
Clippy with warnings denied, workspace tests, diff cleanliness, and review-
only implementation checks pass. Task 263ZQ is complete; fresh Task 263
inventory returns to the remaining local-object-mode long-chain asserted-head
routes. `spec_coverage_audit.md` remains unchanged because no authority,
behavior, coverage credit, owner crate, or deferred status changed.

## Task 263ZR Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the local-object-mode long-chain
immediate-radix asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 590-591 (2 lines, `85c28a03...`), config at
3436-3450 (15, `9c530a6d...`), production detail route at 4011-4023 (13,
`6906e0c0...`), test-only output at 4507-4516 (10, `56abaa93...`), and extractor
at 5477-5488 (12, `e0e40074...`). Total: 52 lines; combined raw hash
`a0b3d996...` and whitespace-normalized pre-move hash `a533b453...`.

Task 263ZR mechanically moves only those fragments into the existing private
owner, retaining its direct sibling `long_chain_config` object-table
dependency. Only the production detail route crosses the normal phase facade.
The config, test-consumed output, and extractor cross under `#[cfg(test)]`; the
invalid-key constant remains leaf-private while a config-derived runner test
alias retains its existing name and value. No public API or call site, name,
config value, exact seven-expansion object-terminal chain, immediate
`ChainObjectMode6 -> ChainObjectMode5` relation, terminal-only
`ChainObjectMode5 -> ChainObjectMode4 -> ChainObjectMode3 -> ChainObjectMode2
-> ChainObjectMode1 -> BaseObjectMode -> object` normalization, key, payload,
ordering, fallback, or fail-closed behavior changes.

Canonical reserved-theorem-variable and static type-assertion requirements
apply unchanged. The one active `.miz`/expectation pair and its dedicated
covered Task 210 checker requirement exercise real source AST/resolver data,
seven object-terminal expansions, distinct reserve `ChainObjectMode6` and
asserted `ChainObjectMode5` source sites, the exact bare immediate relation
edge, and real checker output without object/set coercion. `long_chain.rs`
retains all 43 snake-case family-name occurrences plus the all-5,039-order
finite corruption matrix, all 35 pre-existing owner isolation, immutable
output, key fallback, active fixture, and real sidecar. Therefore this is move-
only `design_drift` and no ZR0 test task is needed. `spec_coverage_audit.md`
remains unchanged because authority, behavior, coverage credit, owner crate,
and deferred status do not change. Forbidden changes are config/key/role/mode
edits, relation or chain generalization, local-mode or other local-object-mode
route moves, object/set coercion, assertion weakening, and test/expectation
edits.

## Task 263ZR Move Result

Task 263ZR moved only the five approved fragments totaling 52 lines into the
existing private `type_elaboration/type_assertion_routes.rs`, preserving
original raw hashes `85c28a03...`, `9c530a6d...`, `6906e0c0...`, `56abaa93...`,
and `e0e40074...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is token-
identical and the combined normalized hash is `a533b453...`. All previously
moved local-mode and local-object-mode multi-hop routes remain unchanged in
`type_assertion_routes.rs`; every other local-object-mode and adjacent route
remains in `runner.rs`.

The expanded private owner is 840 lines with raw hash `820ebd92...`;
`runner.rs` is 5,535 lines with raw hash `710da0a6...`, and the phase facade is
393 lines with raw hash `21abdde1...`. The unchanged 82-line
`long_chain_config.rs` retains hash `3b0e2638...`. The invalid-key constant is
leaf-private; config, test-consumed output, and extractor cross only the test
facade, while the normal facade adds only the production detail route. The
owner retains its direct object-terminal table import, the runner test alias
derives the unchanged key from the moved config, and production dispatch order,
exact object-terminal immediate-radix relation, call identity, and terminal
fail-closed fallback are unchanged.

The focused two tests, all 272 crate unit tests, and the full relevant-crate
suite pass. Raw/normalized 272-name list hashes, four CLI byte hashes, active
counts 96/4/188, plan 403/367, type coverage 235/223, pass/fail 219/184, and 23
warnings/zero errors remain unchanged. Formatting, all-target/all-feature
Clippy with warnings denied, workspace tests, diff cleanliness, and review-
only implementation checks pass. Task 263ZR is complete; fresh Task 263
inventory returns to the remaining local-object-mode long-chain asserted-head
routes. `spec_coverage_audit.md` remains unchanged because no authority,
behavior, coverage credit, owner crate, or deferred status changed.

## Task 263ZS Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the local-object-mode long-chain same-
mode asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 603-604 (2 lines, `f61c9584...`), config at
3441-3455 (15, `99a3d76e...`), production detail route at 4000-4011 (12,
`e9a8a538...`), test-only output at 4482-4490 (9, `4cc05280...`), and extractor
at 5441-5452 (12, `306510d5...`). Total: 50 lines; combined raw hash
`7a22a451...` and whitespace-normalized pre-move hash `3d08750b...`.

Task 263ZS mechanically moves only those fragments into the existing private
owner. Only the production detail route crosses the normal phase facade. The
config, test-consumed output, and extractor cross under `#[cfg(test)]`; the
invalid-key constant remains leaf-private while a config-derived runner test
alias retains its existing name and value. No public API or call site, name,
config value, exact seven-expansion object-terminal chain, same-symbol
`ChainObjectMode6` relation, terminal object normalization, key, payload,
ordering, fallback, or fail-closed behavior changes.

Canonical reserved-theorem-variable and static type-assertion requirements
apply unchanged. The active `.miz`/expectation pair and covered Task 200 checker
requirement exercise real source AST/resolver data, distinct reserve/asserted
sites resolving to the same `ChainObjectMode6` symbol, seven object-terminal
expansions, and real checker output without general reachability, widening,
`qua`, or object/set coercion. `long_chain.rs` retains all 61 snake-case family-
name occurrences, full-reverse and connected-deeper rejection, the exact
structural/provenance/corruption and immutable-output guards, active fixture,
and real sidecar. Therefore this is move-only `design_drift` and no ZS0 test
task is needed. `spec_coverage_audit.md` remains unchanged because authority,
behavior, coverage credit, owner crate, and deferred status do not change.
Forbidden changes are config/key/role/mode edits, relation or chain
generalization, other route moves, object/set coercion, assertion weakening,
and test/expectation edits.

## Task 263ZS Move Result

Task 263ZS moved only the five approved fragments totaling 50 lines into the
existing private `type_elaboration/type_assertion_routes.rs`, preserving
original raw hashes `f61c9584...`, `99a3d76e...`, `e9a8a538...`, `4cc05280...`,
and `306510d5...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is token-
identical and the combined normalized hash is `3d08750b...`. All previously
moved local-mode and local-object-mode routes remain unchanged in
`type_assertion_routes.rs`; every other local-object-mode and adjacent route
remains in `runner.rs`.

The expanded private owner is 895 lines with raw hash `1905d645...`;
`runner.rs` is 5,488 lines with raw hash `b893a626...`, and the phase facade is
397 lines with raw hash `3135dcb0...`. The unchanged 82-line
`long_chain_config.rs` retains hash `3b0e2638...`. The invalid-key constant is
leaf-private; config, test-consumed output, and extractor cross only the test
facade, while the normal facade adds only the production detail route. The
owner retains its direct object-terminal table import, the runner test alias
derives the unchanged key from the moved config, and production dispatch order,
exact object-terminal same-mode relation, call identity, terminal object
normalization, and fail-closed fallback are unchanged.

The focused two tests, all 272 crate unit tests, and the full relevant-crate
suite pass. Raw/normalized 272-name list hashes, four CLI byte hashes, active
counts 96/4/188, plan 403/367, type coverage 235/223, pass/fail 219/184, and 23
warnings/zero errors remain unchanged. Formatting, all-target/all-feature
Clippy with warnings denied, workspace tests, diff cleanliness, and review-only
implementation checks pass. Task 263ZS is complete; fresh Task 263 inventory
returns to the remaining local-object-mode long-chain asserted-head routes.
`spec_coverage_audit.md` remains unchanged because no authority, behavior,
coverage credit, owner crate, or deferred status changed.

## Task 263ZT Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the local-object-mode long-chain
reserved-variable builtin type-assertion route as the next bounded addition to
private `type_elaboration/type_assertion_routes.rs`. It forms five exact
`runner.rs` fragments: the invalid key at 610-611 (2 lines, `78b08029...`),
config at 3446-3459 (14, `b61e1cfe...`), production detail route at 3989-4002
(14, `1d970933...`), test-only output at 4459-4468 (10, `9b8192d2...`), and
extractor at 5407-5418 (12, `ee3fefe6...`). Total: 52 lines; combined raw hash
`7dc2d7ba...` and whitespace-normalized pre-move hash `a5a24f13...`.

Task 263ZT mechanically moves only those fragments into the existing private
owner, which imports the shared object-terminal table directly from sibling
`long_chain_config`. Only the production detail route crosses the normal phase
facade. The config, test-consumed output, and extractor cross under
`#[cfg(test)]`; the invalid-key constant remains leaf-private while a config-
derived runner test alias retains its existing name and value. No public API or
call site, name, config value, exact seven-expansion object-terminal chain,
builtin-object asserted head, key, payload, ordering, fallback, or fail-closed
behavior changes.

Canonical reserved-theorem-variable and static type-assertion requirements
apply unchanged. The active `.miz`/expectation pair and covered Task 179 checker
requirement exercise real source AST/resolver data, a `ChainObjectMode6`
reserved subject, an independent formula-side builtin `object` input, seven
object-terminal expansions, terminal builtin-object normalization, and real
checker output without general reachability, widening, `qua`, or object/set
coercion. `long_chain.rs` retains all 62 snake-case family-name occurrences,
exact structural/provenance/removal/corruption guards, immutable-output and
route-isolation coverage, active fixture, and real sidecar. The two focused
source/active tests pass before the move. Therefore this is move-only
`design_drift` and no ZT0 test task is needed. `spec_coverage_audit.md` remains
unchanged because authority, behavior, coverage credit, owner crate, and
deferred status do not change. Forbidden changes are config/key/role/mode
edits, chain or asserted-head generalization, other route moves, object/set
coercion, assertion weakening, and test/expectation edits.

## Task 263ZT Move Result

Task 263ZT moved only the five approved fragments totaling 52 lines into the
existing private `type_elaboration/type_assertion_routes.rs`, preserving
original raw hashes `78b08029...`, `b61e1cfe...`, `1d970933...`, `9b8192d2...`,
and `ee3fefe6...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is token-
identical and the combined normalized hash is `a5a24f13...`. All previously
moved type-assertion/asserted-head routes remain unchanged in the owner; every
other route remains in `runner.rs`.

The expanded private owner is 953 lines with raw hash `701e2c3f...`;
`runner.rs` is 5,437 lines with raw hash `9a1ea949...`, and the phase facade is
400 lines with raw hash `08cc2834...`. The unchanged 82-line
`long_chain_config.rs` retains hash `3b0e2638...`. The invalid-key constant is
leaf-private; config, test-consumed output, and extractor cross only the test
facade, while the normal facade adds only the production detail route. The
owner retains its direct object-terminal table import, and the now-stale runner
table import and phase-facade re-export were removed. The runner test alias
derives the unchanged key from the moved config; production dispatch order,
exact seven-expansion builtin-object relation, call identity, terminal object
normalization, and fail-closed fallback are unchanged.

The focused source/active tests, all 272 crate unit tests, the full relevant-
crate suite, and workspace tests pass. Raw/normalized 272-name list hashes,
four CLI byte hashes, active counts 96/4/188, plan 403/367, type coverage
235/223, pass/fail 219/184, and 23 warnings/zero errors remain unchanged.
Formatting, all-target/all-feature Clippy with warnings denied, diff
cleanliness, and review-only implementation checks pass. Task 263ZT is
complete; fresh Task 263 inventory returns to the remaining local-object-mode
type-assertion/asserted-head/formula route families. `spec_coverage_audit.md`
remains unchanged because no authority, behavior, coverage credit, owner
crate, or deferred status changed.

## Task 263ZU Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the direct local-object-mode reserved-
variable builtin type-assertion route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 705-707 (3 lines, `807489f0...`), config at
3432-3448 (17, `fc5e75dc...`), production detail route at 4007-4018 (12,
`f80bea53...`), test-only output at 4456-4464 (9, `eed40e5a...`), and extractor
at 5395-5406 (12, `ac4e4e34...`). Total: 53 lines; combined raw hash
`2eeb8849...` and whitespace-normalized pre-move hash `e62fac61...`.

Task 263ZU mechanically moves only those fragments into the existing private
owner. Only the production detail route crosses the normal phase facade. The
config, test-consumed output, and extractor cross under `#[cfg(test)]`; the
invalid-key constant remains leaf-private while a config-derived runner test
alias retains its existing name and value. No public API or call site, name,
config value, exact direct object-terminal expansion, builtin-object asserted
head, key, payload, ordering, fallback, or fail-closed behavior changes.

Canonical reserved-theorem-variable and static type-assertion requirements
apply unchanged. The active `.miz`/expectation pair and covered Task 145 checker
requirement exercise real source AST/resolver data, one exact bare
`LocalObjectModeTypeAssertion -> object` definition, a reserved subject, an
independent formula-side builtin `object` input, terminal builtin-object
normalization, `BindingId(0)` at ordinal 1, and real checker output without
general reachability, widening, `qua`, or object/set coercion. Existing tests
retain 65 direct extractor/output/invalid-key references across eight support,
source, active-fixture, long-chain, and isolation files, with exact source,
definition-label, expansion-corruption, immutable-output, active real sidecar,
and cross-route guards. The focused source/active tests pass before the move.
Therefore this is move-only `design_drift` and no ZU0 test task is needed.
`spec_coverage_audit.md` remains unchanged because authority, behavior,
coverage credit, owner crate, and deferred status do not change. Forbidden
changes are config/key/role/mode edits, route or asserted-head generalization,
other route moves, object/set coercion, assertion weakening, and test/
expectation edits.

## Task 263ZU Move Result

Task 263ZU moved only the five approved fragments totaling 53 lines into the
existing private `type_elaboration/type_assertion_routes.rs`, preserving
original raw hashes `807489f0...`, `fc5e75dc...`, `f80bea53...`, `eed40e5a...`,
and `ac4e4e34...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is token-
identical and the combined normalized hash is `e62fac61...`. All previously
moved routes remain unchanged in the owner; every other type-assertion and
asserted-head route remains in `runner.rs`.

The expanded private owner is 1,013 lines with raw hash `511425dc...`;
`runner.rs` is 5,386 lines with raw hash `6b33f91a...`, and the phase facade is
404 lines with raw hash `2120f54a...`. The invalid-key constant is leaf-private;
config, test-consumed output, and extractor cross only the test facade, while
the normal facade adds only the production detail route. The owner adds only
the mode-definition/radix imports required by the moved inline definition.
The runner test alias derives the unchanged key from the moved config;
production dispatch order, exact one-expansion builtin-object relation, call
identity, terminal object normalization, and fail-closed fallback are
unchanged.

The focused source/active tests, all 272 crate unit tests, the full relevant-
crate suite, and workspace tests pass. Raw/normalized 272-name list hashes,
four CLI byte hashes, active counts 96/4/188, plan 403/367, type coverage
235/223, pass/fail 219/184, and 23 warnings/zero errors remain unchanged.
Formatting, all-target/all-feature Clippy with warnings denied, diff
cleanliness, and review-only implementation checks pass. Task 263ZU is
complete; fresh Task 263 inventory returns to the remaining local-object-mode
type-assertion/asserted-head/formula route families. `spec_coverage_audit.md`
remains unchanged because no authority, behavior, coverage credit, owner
crate, or deferred status changed.

## Task 263ZV Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the chained local-object-mode
reserved-variable builtin type-assertion route as the next bounded addition to
private `type_elaboration/type_assertion_routes.rs`. It forms five exact
`runner.rs` fragments: the invalid key at 693-695 (3 lines, `98c1b75c...`),
config at 3183-3209 (27, `7a694885...`), production detail route at 3890-3903
(14, `479515b0...`), test-only output at 4346-4356 (11, `afd6acbb...`), and
extractor at 5266-5277 (12, `b751e7be...`). Total: 67 lines; combined raw hash
`13f33de7...` and whitespace-normalized pre-move hash `92f527a2...`.

Task 263ZV mechanically moves only those fragments into the existing private
owner. Only the production detail route crosses the normal phase facade. The
config, test-consumed output, and extractor cross under `#[cfg(test)]`; the
invalid-key constant remains leaf-private while a config-derived runner test
alias retains its existing name and value. No public API or call site, name,
config value, exact two-expansion object-terminal chain, builtin-object
asserted head, key, payload, ordering, fallback, or fail-closed behavior
changes.

Canonical builtin-type, reserved-theorem-variable, mode-chain, and static
type-assertion requirements apply unchanged. The active `.miz`/expectation
pair and covered Task 147 checker requirement exercise real source AST/resolver
data, exact `ChainObjectModeTypeAssertion -> BaseObjectModeTypeAssertion ->
object` definitions, a reserved subject, an independent formula-side builtin
`object` input, terminal builtin-object normalization, `BindingId(0)` at
subject ordinal 1, and real checker output without general reachability,
widening, `qua`, or object/set coercion. Existing tests retain 67 direct
extractor/output/invalid-key references across nine support, source, active-
fixture, long-chain, and isolation files, with exact source, definition-label,
both-expansion corruption, immutable-output, active real sidecar, and cross-
route guards. The focused source/active tests pass before the move. Therefore
this is move-only `design_drift` and no ZV0 test task is needed.
`spec_coverage_audit.md` remains unchanged because authority, behavior,
coverage credit, owner crate, and deferred status do not change. Forbidden
changes are config/key/role/mode edits, route or asserted-head generalization,
other route moves, object/set coercion, assertion weakening, and test/
expectation edits.

## Task 263ZV Move Result

Task 263ZV moved only the five approved fragments totaling 67 lines into the
existing private `type_elaboration/type_assertion_routes.rs`, preserving
original raw hashes `98c1b75c...`, `7a694885...`, `479515b0...`, `afd6acbb...`,
and `b751e7be...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is token-
identical, the combined raw hash is `13f33de7...`, and the combined normalized
hash is `92f527a2...`. All previously moved routes remain unchanged in the
owner; every other type-assertion and asserted-head route remains in
`runner.rs`.

The expanded private owner is 1,085 lines with raw hash `41caa325...`;
`runner.rs` is 5,323 lines with raw hash `b51bfae1...`, and the phase facade is
408 lines with raw hash `3bd1f0cd...`. The invalid-key constant is leaf-
private; config, test-consumed output, and extractor cross only the test
facade, while the normal facade adds only the production detail route. The
runner test alias derives the unchanged key from the moved config; production
dispatch position, exact two-expansion builtin-object relation, call identity,
terminal object normalization, and fail-closed fallback are unchanged.

The focused source/active tests, all 272 crate unit tests, the full relevant-
crate suite, and workspace tests pass. Raw/normalized 272-name list hashes,
four CLI byte hashes, active counts 96/4/188, plan 403/367, type coverage
235/223, pass/fail 219/184, and 23 warnings/zero errors remain unchanged.
Formatting, all-target/all-feature Clippy with warnings denied, diff
cleanliness, and review-only implementation checks pass. Task 263ZV is
complete; fresh Task 263 inventory returns to the remaining local-object-mode
type-assertion/asserted-head/formula route families. `spec_coverage_audit.md`
remains unchanged because no authority, behavior, coverage credit, owner
crate, or deferred status changed.

## Task 263ZW Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the two-edge local-object-mode
reserved-variable builtin type-assertion route as the next bounded addition to
private `type_elaboration/type_assertion_routes.rs`. It forms five exact
`runner.rs` fragments: the invalid key at 704-705 (2 lines, `1c780da5...`),
config at 3218-3249 (32, `977e0e8e...`), production detail route at 3881-3894
(14, `05bdafd9...`), test-only output at 4319-4329 (11, `e1765982...`), and
extractor at 5229-5240 (12, `c241f489...`). Total: 71 lines; combined raw hash
`b4862644...` and whitespace-normalized pre-move hash `f87b44d4...`.

Task 263ZW mechanically moves only those fragments into the existing private
owner. Only the production detail route crosses the normal phase facade. The
config, test-consumed output, and extractor cross under `#[cfg(test)]`; the
invalid-key constant remains leaf-private while a config-derived runner test
alias retains its existing name and value. No public API or call site, name,
config value, exact three-expansion object-terminal chain, builtin-object
asserted head, key, payload, ordering, fallback, or fail-closed behavior
changes.

Canonical builtin-type, reserved-theorem-variable, mode-chain, and static
type-assertion requirements apply unchanged. The active `.miz`/expectation
pair and covered Task 149 checker requirement exercise real source AST/resolver
data, exact `OuterTwoEdgeObjectModeTypeAssertion ->
MiddleTwoEdgeObjectModeTypeAssertion -> BaseTwoEdgeObjectModeTypeAssertion ->
object` definitions, a reserved subject, an independent formula-side builtin
`object` input, terminal builtin-object normalization, `BindingId(0)` at
subject ordinal 1, and real checker output without general reachability,
widening, `qua`, or object/set coercion. Existing tests retain 67 direct
extractor/output/invalid-key references across nine support, source, active-
fixture, long-chain, and isolation files, with exact source, definition-label,
all-three-expansion corruption, immutable-output, active real sidecar, and
cross-route guards. The focused source/active tests pass before the move.
Therefore this is move-only `design_drift` and no ZW0 test task is needed.
`spec_coverage_audit.md` remains unchanged because authority, behavior,
coverage credit, owner crate, and deferred status do not change. Forbidden
changes are config/key/role/mode edits, route or asserted-head generalization,
other route moves, object/set coercion, assertion weakening, and test/
expectation edits.

## Task 263ZW Move Result

Task 263ZW moved only the five approved fragments totaling 71 lines into the
existing private `type_elaboration/type_assertion_routes.rs`, preserving
original raw hashes `1c780da5...`, `977e0e8e...`, `05bdafd9...`, `e1765982...`,
and `c241f489...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is token-
identical, the combined raw hash is `b4862644...`, and the combined normalized
hash is `f87b44d4...`. All previously moved routes remain unchanged in the
owner; every other type-assertion and asserted-head route remains in
`runner.rs`.

The expanded private owner is 1,161 lines with raw hash `869e95b0...`;
`runner.rs` is 5,256 lines with raw hash `5189e88c...`, and the phase facade is
412 lines with raw hash `c1f79141...`. The invalid-key constant is leaf-
private; config, test-consumed output, and extractor cross only the test
facade, while the normal facade adds only the production detail route. The
runner test alias derives the unchanged key from the moved config; production
dispatch position, exact three-expansion builtin-object relation, call
identity, terminal object normalization, and fail-closed fallback are
unchanged.

The focused source/active tests, all 272 crate unit tests, the full relevant-
crate suite, and workspace tests pass. Raw/normalized 272-name list hashes,
four CLI byte hashes, active counts 96/4/188, plan 403/367, type coverage
235/223, pass/fail 219/184, and 23 warnings/zero errors remain unchanged.
Formatting, all-target/all-feature Clippy with warnings denied, diff
cleanliness, and review-only implementation checks pass. Task 263ZW is
complete; fresh Task 263 inventory returns to the remaining local-object-mode
type-assertion/asserted-head/formula route families. `spec_coverage_audit.md`
remains unchanged because no authority, behavior, coverage credit, owner
crate, or deferred status changed.

## Task 263ZX Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the three-edge local-object-mode
reserved-variable builtin type-assertion route as the next bounded addition to
private `type_elaboration/type_assertion_routes.rs`. It forms five exact
`runner.rs` fragments: the invalid key at 714-715 (2 lines, `c2f2ffca...`),
config at 3259-3301 (43, `b9016704...`), production detail route at 3869-3882
(14, `1f9c6902...`), test-only output at 4289-4299 (11, `0e10026b...`), and
extractor at 5188-5199 (12, `f273cd7a...`). Total: 82 lines; combined raw hash
`236c4a64...` and whitespace-normalized pre-move hash `f0d95b00...`.

Task 263ZX mechanically moves only those fragments into the existing private
owner. Only the production detail route crosses the normal phase facade. The
config, test-consumed output, and extractor cross under `#[cfg(test)]`; the
invalid-key constant remains leaf-private while a config-derived runner test
alias retains its existing name and value. No public API or call site, name,
config value, exact four-expansion object-terminal chain, builtin-object
asserted head, key, payload, ordering, fallback, or fail-closed behavior
changes.

Canonical builtin-type, reserved-theorem-variable, mode-chain, and static
type-assertion requirements apply unchanged. The active `.miz`/expectation
pair and covered Task 151 checker requirement exercise real source AST/resolver
data, exact `OuterThreeEdgeObjectModeTypeAssertion ->
MiddleThreeEdgeObjectModeTypeAssertion -> InnerThreeEdgeObjectModeTypeAssertion
-> BaseThreeEdgeObjectModeTypeAssertion -> object` definitions, a reserved
subject, an independent formula-side builtin `object` input, terminal builtin-
object normalization, `BindingId(0)` at subject ordinal 1, and real checker
output without general reachability, widening, `qua`, or object/set coercion.
Existing tests retain 64 direct extractor/output/invalid-key references across
eight support, source, active-fixture, long-chain, and isolation files, with
exact source, definition-label, all-four-expansion corruption, immutable-
output, active real sidecar, and cross-route guards. The focused source/active
tests pass before the move. Therefore this is move-only `design_drift` and no
ZX0 test task is needed. `spec_coverage_audit.md` remains unchanged because
authority, behavior, coverage credit, owner crate, and deferred status do not
change. Forbidden changes are config/key/role/mode edits, route or asserted-
head generalization, other route moves, object/set coercion, assertion
weakening, and test/expectation edits.

## Task 263ZX Move Result

Task 263ZX moved only the five approved fragments totaling 82 lines into the
existing private `type_elaboration/type_assertion_routes.rs`, preserving
original raw hashes `c2f2ffca...`, `b9016704...`, `1f9c6902...`, `0e10026b...`,
and `f273cd7a...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is token-
identical, the combined raw hash is `236c4a64...`, and the combined normalized
hash is `f0d95b00...`. All previously moved routes remain unchanged in the
owner; every other type-assertion and asserted-head route remains in
`runner.rs`.

The expanded private owner is 1,248 lines with raw hash `53b13b9b...`;
`runner.rs` is 5,178 lines with raw hash `39377f32...`, and the phase facade is
416 lines with raw hash `3a713a42...`. The invalid-key constant is leaf-
private; config, test-consumed output, and extractor cross only the test
facade, while the normal facade adds only the production detail route. The
runner test alias derives the unchanged key from the moved config; production
dispatch position, exact four-expansion builtin-object relation, call
identity, terminal object normalization, and fail-closed fallback are
unchanged.

The focused source/active tests, all 272 crate unit tests, the full relevant-
crate suite, and workspace tests pass. Raw/normalized 272-name list hashes,
four CLI byte hashes, active counts 96/4/188, plan 403/367, type coverage
235/223, pass/fail 219/184, and 23 warnings/zero errors remain unchanged.
Formatting, all-target/all-feature Clippy with warnings denied, diff
cleanliness, and review-only implementation checks pass. Task 263ZX is
complete; fresh Task 263 inventory returns to the remaining local-object-mode
type-assertion/asserted-head/formula route families. `spec_coverage_audit.md`
remains unchanged because no authority, behavior, coverage credit, owner
crate, or deferred status changed.

## Task 263ZY Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the four-edge local-object-mode
reserved-variable builtin type-assertion route as the next bounded addition to
private `type_elaboration/type_assertion_routes.rs`. It forms five exact
`runner.rs` fragments: the invalid key at 724-725 (2 lines, `38ec55aa...`),
config at 3305-3346 (42, `e665b971...`), production detail route at 3845-3858
(14, `547d8019...`), test-only output at 4247-4257 (11, `e36b7f6d...`), and
extractor at 5136-5147 (12, `1ee94ac5...`). Total: 81 lines; combined raw hash
`f0a97fef...` and whitespace-normalized pre-move hash `135373d6...`.

Task 263ZY mechanically moves only those fragments into the existing private
owner. Only the production detail route crosses the normal phase facade. The
config, test-consumed output, and extractor cross under `#[cfg(test)]`; the
invalid-key constant remains leaf-private while a config-derived runner test
alias retains its existing name and value. No public API or call site, name,
config value, exact five-expansion object-terminal chain, builtin-object
asserted head, key, payload, ordering, fallback, or fail-closed behavior
changes.

Canonical builtin-type, reserved-theorem-variable, mode-chain, and static
type-assertion requirements apply unchanged. The active `.miz`/expectation
pair and covered Task 153 checker requirement exercise real source AST/resolver
data, exact `TooDeepFourEdgeObjectModeTypeAssertion ->
OuterFourEdgeObjectModeTypeAssertion -> MiddleFourEdgeObjectModeTypeAssertion
-> InnerFourEdgeObjectModeTypeAssertion ->
BaseFourEdgeObjectModeTypeAssertion -> object` definitions, a reserved
subject, an independent formula-side builtin `object` input, terminal builtin-
object normalization, `BindingId(0)` at subject ordinal 1, and real checker
output without general reachability, widening, `qua`, or object/set coercion.
Existing tests retain 60 direct extractor/output/invalid-key references across
seven support, source, active-fixture, long-chain, and isolation files, with
exact source, definition-label, all-five-expansion corruption, immutable-
output, active real sidecar, and cross-route guards. The focused source/active
tests pass before the move. Therefore this is move-only `design_drift` and no
ZY0 test task is needed. `spec_coverage_audit.md` remains unchanged because
authority, behavior, coverage credit, owner crate, and deferred status do not
change. Forbidden changes are config/key/role/mode edits, route or asserted-
head generalization, other route moves, object/set coercion, assertion
weakening, and test/expectation edits.

## Task 263ZY Move Result

Task 263ZY moved only the five approved fragments totaling 81 lines into the
existing private `type_elaboration/type_assertion_routes.rs`, preserving
original raw hashes `38ec55aa...`, `e665b971...`, `547d8019...`, `e36b7f6d...`,
and `1ee94ac5...` as the pre-move oracle. After removing only required runner
visibility and formatting whitespace, every old/new fragment pair is token-
identical, the combined raw hash is `f0a97fef...`, and the combined normalized
hash is `135373d6...`. All previously moved routes remain unchanged in the
owner; every other type-assertion and asserted-head route remains in
`runner.rs`.

The expanded private owner is 1,334 lines with raw hash `defe8960...`;
`runner.rs` is 5,101 lines with raw hash `c337cb04...`, and the phase facade is
420 lines with raw hash `62b82681...`. The invalid-key constant is leaf-
private; config, test-consumed output, and extractor cross only the test
facade, while the normal facade adds only the production detail route. The
runner test alias derives the unchanged key from the moved config; production
dispatch position, exact five-expansion builtin-object relation, call
identity, terminal object normalization, and fail-closed fallback are
unchanged.

The focused source/active tests, all 272 crate unit tests, the full relevant-
crate suite, and workspace tests pass. Raw/normalized 272-name list hashes,
four CLI byte hashes, active counts 96/4/188, plan 403/367, type coverage
235/223, pass/fail 219/184, and 23 warnings/zero errors remain unchanged.
Formatting, all-target/all-feature Clippy with warnings denied, diff
cleanliness, and review-only implementation checks pass. Task 263ZY is
complete; fresh Task 263 inventory returns to the remaining local-object-mode
type-assertion/asserted-head/formula route families. `spec_coverage_audit.md`
remains unchanged because no authority, behavior, coverage credit, owner
crate, or deferred status changed.

## Task 263ZZ Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the direct local-object-mode same-
mode asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 644-645 (2 lines, `7e20cddf...`), config at
2040-2059 (20, `315d6705...`), production detail route at 3362-3373 (12,
`98767002...`), test-only output at 3848-3856 (9, `5faad673...`), and extractor
at 4643-4654 (12, `e4a9dc46...`). Total: 55 lines; combined raw hash
`2f87f6dd...` and whitespace-normalized pre-move hash `e5a22380...`.

Task 263ZZ mechanically moves only those fragments into the existing private
owner. Only the production detail route crosses the normal phase facade. The
config, test-consumed output, and extractor cross under `#[cfg(test)]`; the
invalid-key constant remains leaf-private while a config-derived runner test
alias retains its existing name and value. No public API or call site, name,
config value, exact one-expansion object-terminal same-mode relation, key,
payload, ordering, fallback, or fail-closed behavior changes.

Canonical builtin-type, reserved-theorem-variable, mode-expansion, and static
type-assertion requirements apply unchanged. The active `.miz`/expectation
pair and covered Task 183 checker requirement exercise real source AST/resolver
data, exact `LocalObjectModeAssertedHead -> object` definition, a reserved
subject, an independent formula-side reference to the same local mode, terminal
builtin-object normalization, `BindingId(0)` at subject ordinal 1, and real
checker output without general reachability, widening, `qua`, or object/set
coercion. Existing tests retain 60 direct extractor/output/invalid-key
references across five support, source, active-fixture, long-chain, and
isolation files, with exact source, definition-label/radix, corruption and
near-miss coverage, immutable output, active real sidecar, and cross-route
guards. The focused source/active tests pass before the move. Therefore this
is move-only `design_drift` and no ZZ0 test task is needed.
`spec_coverage_audit.md` remains unchanged because authority, behavior,
coverage credit, owner crate, and deferred status do not change. Forbidden
changes are config/key/role/mode edits, route or asserted-head generalization,
other route moves, object/set coercion, assertion weakening, and test/
expectation edits.

## Task 263ZZ Move Result

Task 263ZZ moved only the five approved fragments totaling 55 lines into the
existing private `type_elaboration/type_assertion_routes.rs` owner. The
invalid-key fragment remains leaf-private; the config, output, and extractor
remain test-only facade imports; and only the production detail route crosses
the normal phase facade. The config-derived runner test alias retains the same
name and value.

After removing the required `pub(in crate::runner)` visibility from the moved
fragments, every old/new fragment pair retains its exact raw hash:
`7e20cddf...`, `315d6705...`, `98767002...`, `5faad673...`, and
`e4a9dc46...`. Their combined raw hash remains `2f87f6dd...`, and the combined
whitespace-normalized hash remains `e5a22380...`. The item-scoped
`#[rustfmt::skip]` immediately above the moved config is outside this 55-line
oracle and only preserves the original multiline trailing-comma token form
under the owner's indentation; it has no runtime effect.

The post-move owner is 1,395 lines with SHA-256 `5db40505...`; `runner.rs` is
5,049 lines with `7ace5217...`; and the phase facade is 424 lines with
`639de742...`. The focused source and active-fixture tests, the 272-unit-test
crate suite, raw and normalized test-list hashes, and all four CLI report
hashes remain unchanged. Review-only test-sufficiency, implementation, and
source/documentation consistency checks have no finding; full workspace
format, Clippy, test, and diff gates pass. No API,
name, test, expectation, trace, diagnostic, key, payload, ordering, fallback,
or fail-closed behavior changed. `spec_coverage_audit.md` remains unchanged
because no authority, behavior, coverage credit, owner crate, or deferred
status changed.

## Task 263ZZA Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the chained local-object-mode same-
mode asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 697-698 (2 lines, `87ab7a13...`), config at
2903-2930 (28, `84bcf48a...`), production detail route at 3631-3642 (12,
`fcc6d9c8...`), test-only output at 4041-4049 (9, `f108ebaa...`), and extractor
at 4877-4888 (12, `f34d5bec...`). Total: 63 lines; combined raw hash
`c19bc3a5...` and whitespace-normalized pre-move hash `43acc3c2...`.

Task 263ZZA mechanically moves only those fragments into the existing private
owner. Only the production detail route crosses the normal phase facade. The
config, test-consumed output, and extractor cross under `#[cfg(test)]`; the
invalid-key constant remains leaf-private while a config-derived runner test
alias retains its existing name and value. No public API or call site, name,
config value, exact two-expansion object-terminal same-mode relation, key,
payload, ordering, fallback, or fail-closed behavior changes.

Canonical mode unfolding, builtin-object, reserved-theorem-variable, and
static type-assertion requirements apply unchanged. The active `.miz`/
expectation pair and covered Task 185 checker requirement exercise two ordered
real source-derived definitions, `BaseObjectModeAssertedHead -> object` and
`ChainObjectModeAssertedHead -> BaseObjectModeAssertedHead`, a reserved subject,
and an independent formula-side reference to the same outer mode. The route
retains distinct source sites/ranges, consumes exactly two expansions,
normalizes three known type entries to the terminal base-definition-RHS
builtin-object identity, resolves subject ordinal 1 to `BindingId(0)`, and
produces one inferred variable plus one fact/deferred-free checked type
assertion without general reachability, widening, `qua`, or object/set
coercion. Existing tests retain 63 direct extractor/output/invalid-key
references across six support, source, active-fixture, long-chain, and
isolation test files; including six definitions/internal calls in `runner.rs`,
the pre-move repository has 69 occurrences across seven source-and-test files.
The tests provide exact source, expansion, definition-label/radix,
corruption, near-miss, immutable-output, real-sidecar, and cross-route guards.
Both focused source and active-fixture tests pass before the move. Therefore
this is move-only `design_drift`; no ZZA0 test task is needed.
`spec_coverage_audit.md` remains unchanged because authority, behavior,
coverage credit, owner crate, and deferred status do not change. Forbidden
changes are config/key/role/mode edits, route or asserted-head generalization,
other route moves, object/set coercion, assertion weakening, and test/
expectation edits.

## Task 263ZZA Move Result

Task 263ZZA moved only the five approved fragments totaling 63 lines into the
existing private `type_elaboration/type_assertion_routes.rs` owner. The
invalid-key fragment remains leaf-private; the config, output, and extractor
remain test-only facade imports; and only the production detail route crosses
the normal phase facade. The config-derived runner test alias retains the same
name and value.

After removing the required `pub(in crate::runner)` visibility from the moved
fragments, every old/new fragment pair retains its exact raw hash:
`87ab7a13...`, `84bcf48a...`, `fcc6d9c8...`, `f108ebaa...`, and
`f34d5bec...`. Their combined raw hash remains `c19bc3a5...`, and the combined
whitespace-normalized hash remains `43acc3c2...`. The item-scoped
`#[rustfmt::skip]` immediately above the moved config is outside this 63-line
oracle and only preserves the original config token layout after adding the
required owner visibility; it has no runtime effect.

The post-move owner is 1,464 lines with SHA-256 `366eff9a...`; `runner.rs` is
4,989 lines with `9c01b80f...`; and the phase facade is 428 lines with
`03cff9d4...`. Both focused tests, the 272-unit-test crate suite, raw and
normalized test-list hashes, and all four CLI report hashes remain unchanged.
Review-only implementation and test-sufficiency checks found no source or test
issue; their sole completion-state documentation finding is repaired by this
paired update. Full workspace format, Clippy, test, and diff gates pass. No
API, name, test, expectation, trace, diagnostic, key, payload, ordering,
fallback, or fail-closed behavior changed. `spec_coverage_audit.md` remains
unchanged because no authority, behavior, coverage credit, owner crate, or
deferred status changed.

## Task 263ZZB Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the chained local-object-mode
immediate-radix asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 659-660 (2 lines, `9c789614...`), config at
2105-2132 (28, `dd489077...`), production detail route at 3348-3360 (13,
`fbf581af...`), test-only output at 3802-3811 (10, `16c15d04...`), and
extractor at 4583-4594 (12, `a452ccd2...`). Total: 65 lines; combined raw hash
`350810f3...` and whitespace-normalized pre-move hash `606b46b8...`.

Task 263ZZB mechanically moves only those fragments into the existing private
owner. Only the production detail route crosses the normal phase facade. The
config, test-consumed output, and extractor cross under `#[cfg(test)]`; the
invalid-key constant remains leaf-private while a config-derived runner test
alias retains its existing name and value. No public API or call site, name,
config value, exact two-expansion object-terminal immediate-radix relation,
key, payload, ordering, fallback, or fail-closed behavior changes.

Canonical mode unfolding, builtin-object, reserved-theorem-variable, and
static type-assertion requirements apply unchanged. The active `.miz`/
expectation pair and covered Task 202 checker requirement exercise two ordered
real source-derived definitions, `BaseObjectModeRadixAssertedHead -> object`
and `OuterObjectModeRadixAssertedHead -> BaseObjectModeRadixAssertedHead`, an
outer-mode reserved subject, and an independent formula-side reference to the
outer expansion's immediate radix. The route retains distinct outer/base
symbols and source sites/ranges, consumes exactly two expansions, normalizes
three known type entries to the terminal base-definition-RHS builtin-object
identity, resolves subject ordinal 1 to `BindingId(0)`, and produces one
inferred variable plus one fact/candidate/diagnostic/deferred-free checked type
assertion with zero expected constraints and without general reachability,
widening, `qua`, or object/set coercion. Existing tests retain 73 direct
extractor/output/invalid-key references across nine test files; including six
definitions/internal calls in `runner.rs`, the pre-move repository has 79
occurrences across ten source-and-test files. Exact source, expansion,
definition-label/radix, relation, corruption, near-miss, immutable-output,
real-sidecar, and cross-route guards are present. Both focused source and
active-fixture tests pass before the move. Therefore this is move-only
`design_drift`; no ZZB0 test task is needed. `spec_coverage_audit.md` remains
unchanged because authority, behavior, coverage credit, owner crate, and
deferred status do not change. Forbidden changes are config/key/role/mode/
relation edits, route or asserted-head generalization, other route moves,
object/set coercion, assertion weakening, and test/expectation edits.

## Task 263ZZB Move Result

Task 263ZZB moved only the five approved fragments totaling 65 lines into the
existing private `type_elaboration/type_assertion_routes.rs` owner. The
invalid-key fragment remains leaf-private; the config, output, and extractor
remain test-only facade imports; and only the production detail route crosses
the normal phase facade. The config-derived runner test alias retains the same
name and value.

After removing the required `pub(in crate::runner)` visibility from the moved
fragments, every old/new fragment pair retains its exact raw hash:
`9c789614...`, `dd489077...`, `fbf581af...`, `16c15d04...`, and
`a452ccd2...`. Their combined raw hash remains `350810f3...`, and the combined
whitespace-normalized hash remains `606b46b8...`. The item-scoped
`#[rustfmt::skip]` immediately above the moved config is outside this 65-line
oracle and only preserves the original config token layout after adding the
required owner visibility; it has no runtime effect.

The post-move owner is 1,535 lines with SHA-256 `7ae4fa4d...`; `runner.rs` is
4,927 lines with `171aa7c4...`; and the phase facade is 432 lines with
`dfa5b65a...`. Both focused tests, the 272-unit-test crate suite, raw and
normalized test-list hashes, and all four CLI report hashes remain unchanged.
Review-only implementation and test-sufficiency checks found no source or test
issue; their sole completion-state documentation finding is repaired by this
paired update. Full workspace format, Clippy, test, and diff gates pass. No
API, name, relation, test, expectation, trace, diagnostic, key, payload,
ordering, fallback, or fail-closed behavior changed. `spec_coverage_audit.md`
remains unchanged because no authority, behavior, coverage credit, owner
crate, or deferred status changed.

## Task 263ZZC Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the two-edge local-object-mode same-
mode asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 719-720 (2 lines, `cc5d93c2...`), config at
3083-3115 (33, `74710488...`), production detail route at 3634-3645 (12,
`c487e895...`), test-only output at 4005-4013 (9, `33561a0b...`), and extractor
at 4833-4844 (12, `694b6312...`). Total: 68 lines; combined raw hash
`d3f42ec4...` and whitespace-normalized pre-move hash `38599f34...`.

Task 263ZZC mechanically moves only those fragments into the existing private
owner. The orchestration call and its dispatch order stay in `runner.rs`. Only
the production detail route crosses the normal phase facade. The config, test-
consumed output, and extractor cross under `#[cfg(test)]`; the invalid-key
constant remains leaf-private while a config-derived runner test alias retains
its existing name and value. No public API or call site, helper name, config
value, exact three-expansion object-terminal same-mode relation, key, payload,
ordering, fallback, or fail-closed behavior changes.

Canonical mode unfolding, builtin-object, reserved-theorem-variable, and
static type-assertion requirements apply unchanged. The active `.miz`/
expectation pair and covered Task 187 checker requirement exercise three
ordered real source-derived definitions, `BaseTwoEdgeObjectModeAssertedHead ->
object`, `MiddleTwoEdgeObjectModeAssertedHead ->
BaseTwoEdgeObjectModeAssertedHead`, and `OuterTwoEdgeObjectModeAssertedHead ->
MiddleTwoEdgeObjectModeAssertedHead`, with independent reserve-side and formula-
side references to the same outer symbol at distinct sites/ranges. The route
consumes exactly three expansions, normalizes three known type entries to the
terminal base-definition-RHS builtin-object identity, resolves subject ordinal
1 to `BindingId(0)`, and produces one inferred variable plus one fact/
candidate/diagnostic/deferred-free checked type assertion with zero expected
constraints and without general reachability, widening, `qua`, or object/set
coercion. Existing tests retain 66 direct symbol references across seven test
files; including ten definitions/internal calls in `runner.rs`, the pre-move
repository has 76 occurrences across eight source-and-test files. Exact source,
expansion, definition-label/radix, relation, corruption, near-miss, immutable-
output, real-sidecar, and cross-route guards are present. Both focused source
and active-fixture tests pass before the move. Therefore this is move-only
`design_drift`; no ZZC0 test task is needed. `spec_coverage_audit.md` remains
unchanged because authority, behavior, coverage credit, owner crate, and
deferred status do not change. Forbidden changes are config/key/role/mode/
relation edits, route or asserted-head generalization, other route moves,
object/set coercion, assertion weakening, and test/expectation edits.

## Task 263ZZC Move Result

Task 263ZZC moved only the five approved fragments totaling 68 lines into the
existing private `type_elaboration/type_assertion_routes.rs` owner. The
orchestration call remains in place and byte/order-stable. The invalid-key
fragment remains leaf-private; the config, output, and extractor remain test-
only facade imports; and only the production detail route crosses the normal
phase facade. The config-derived runner test alias retains the same name and
value.

After removing the required `pub(in crate::runner)` visibility from the moved
fragments, every old/new fragment pair retains its exact raw hash:
`cc5d93c2...`, `74710488...`, `c487e895...`, `33561a0b...`, and
`694b6312...`. Their combined raw hash remains `d3f42ec4...`, and the combined
whitespace-normalized hash remains `38599f34...`. The item-scoped
`#[rustfmt::skip]` immediately above the moved config is outside this 68-line
oracle and only preserves the original config token layout after adding the
required owner visibility; it has no runtime effect.

The post-move owner is 1,609 lines with SHA-256 `2ebb1d54...`; `runner.rs` is
4,862 lines with `a05d72b2...`; and the phase facade is 436 lines with
`1b892834...`. Both focused tests, the 272-unit-test crate suite, raw hash
`5e41e4db...` and normalized hash `c0c2b80f...` for the test list, and all four
CLI report hashes remain unchanged. Review-only implementation and test-
sufficiency checks found no finding. Full workspace format, Clippy, test, and
diff gates pass. No API, name, relation, test, expectation, trace, diagnostic,
key, payload, ordering, fallback, or fail-closed behavior changed.
`spec_coverage_audit.md` remains unchanged because no authority, behavior,
coverage credit, owner crate, or deferred status changed.

## Task 263ZZD Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the two-edge local-object-mode
immediate-radix asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 700-701 (2 lines, `d4a69d79...`), config at
2679-2713 (35, `8325c905...`), production detail route at 3472-3484 (13,
`44b42bc2...`), test-only output at 3860-3869 (10, `c910581b...`), and
extractor at 4651-4662 (12, `dfb26d72...`). Total: 72 lines; combined raw hash
`10087773...` and whitespace-normalized pre-move hash `d24a1e53...`.

Task 263ZZD mechanically moves only those fragments into the existing private
owner. The orchestration call and dispatch order stay in `runner.rs`. Only the
production detail route crosses the normal phase facade. The config, test-
consumed output, and extractor cross under `#[cfg(test)]`; the invalid-key
constant remains leaf-private while a config-derived runner test alias retains
its existing name and value. No public API or call site, helper name, config
value, exact three-expansion object-terminal immediate-radix relation, key,
payload, ordering, fallback, or fail-closed behavior changes.

Canonical mode unfolding, builtin-object, reserved-theorem-variable, and
static type-assertion requirements apply unchanged. The active `.miz`/
expectation pair and covered Task 204 checker requirement exercise three
ordered real source-derived definitions: `BaseTwoEdgeObjectModeRadixAssertedHead
-> object`, `MiddleTwoEdgeObjectModeRadixAssertedHead ->
BaseTwoEdgeObjectModeRadixAssertedHead`, and
`OuterTwoEdgeObjectModeRadixAssertedHead ->
MiddleTwoEdgeObjectModeRadixAssertedHead`. The reserve subject retains the
outer symbol while the formula-side asserted type independently resolves the
middle symbol at a distinct site/range and must equal the outer expansion's
immediate radix. The route consumes exactly three expansions, normalizes three
known type entries to the terminal base-definition-RHS builtin-object identity,
resolves subject ordinal 1 to `BindingId(0)`, and produces one inferred
variable plus one fact/candidate/diagnostic/deferred-free checked type
assertion with zero expected constraints and without two-hop/general
reachability, widening, `qua`, or object/set coercion. Existing tests retain 75
direct symbol references across eight test files; including ten definitions/
internal calls in `runner.rs`, the pre-move repository has 85 occurrences
across nine source-and-test files. Exact source, expansion, definition-label/
radix, relation, corruption, near-miss, immutable-output, real-sidecar, and
cross-route guards are present. Both focused source and active-fixture tests
pass before the move. Therefore this is move-only `design_drift`; no ZZD0 test
task is needed. `spec_coverage_audit.md` remains unchanged because authority,
behavior, coverage credit, owner crate, and deferred status do not change.
Forbidden changes are config/key/role/mode/relation edits, route or asserted-
head generalization, other route moves, object/set coercion, assertion
weakening, and test/expectation edits.

## Task 263ZZD Move Result

Task 263ZZD moved only the five approved fragments totaling 72 lines into the
existing private `type_elaboration/type_assertion_routes.rs` owner. The
orchestration call remains in place and byte/order-stable. The invalid-key
fragment remains leaf-private; the config, output, and extractor remain test-
only facade imports; and only the production detail route crosses the normal
phase facade. The config-derived runner test alias retains the same name and
value.

After removing the required `pub(in crate::runner)` visibility from the moved
fragments, every old/new fragment pair retains its exact raw hash:
`d4a69d79...`, `8325c905...`, `44b42bc2...`, `c910581b...`, and
`dfb26d72...`. Their combined raw hash remains `10087773...`, and the combined
whitespace-normalized hash remains `d24a1e53...`. The item-scoped
`#[rustfmt::skip]` immediately above the moved config is outside this 72-line
oracle and only preserves the original config token layout after adding the
required owner visibility; it has no runtime effect.

The post-move owner is 1,687 lines with SHA-256 `4a98420b...`; `runner.rs` is
4,793 lines with `e3c01671...`; and the phase facade is 440 lines with
`d3243e97...`. Both focused tests, the 272-unit-test crate suite, raw hash
`5e41e4db...` and normalized hash `c0c2b80f...` for the test list, and all four
CLI report hashes remain unchanged. Review-only implementation and test-
sufficiency checks found no finding. Full workspace format, Clippy, test, and
diff gates pass. No API, name, relation, test, expectation, trace, diagnostic,
key, payload, ordering, fallback, or fail-closed behavior changed.
`spec_coverage_audit.md` remains unchanged because no authority, behavior,
coverage credit, owner crate, or deferred status changed.

## Task 263ZZE Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the two-edge local-object-mode two-
hop asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 678-679 (2 lines, `a8adcfca...`), config at
2185-2218 (34, `de8ce647...`), production detail route at 3290-3302 (13,
`77f965e5...`), test-only output at 3697-3706 (10, `23463041...`), and
extractor at 4452-4463 (12, `71947b90...`). Total: 71 lines; combined raw hash
`55b319a2...` and whitespace-normalized pre-move hash `af5eb98a...`.

Task 263ZZE mechanically moves only those fragments into the existing private
owner. The orchestration call and dispatch order stay in `runner.rs`. Only the
production detail route crosses the normal phase facade. The config, test-
consumed output, and extractor cross under `#[cfg(test)]`; the invalid-key
constant remains leaf-private while a config-derived runner test alias retains
its existing name and value. No public API or call site, helper name, config
value, exact three-expansion object-terminal two-hop relation, key, payload,
ordering, fallback, or fail-closed behavior changes.

Canonical mode unfolding, builtin-object, reserved-theorem-variable, and
static type-assertion requirements apply unchanged. The active `.miz`/
expectation pair and covered Task 212 checker requirement exercise three
ordered real source-derived definitions: `BaseTwoHopObjectModeAssertedHead ->
object`, `MiddleTwoHopObjectModeAssertedHead ->
BaseTwoHopObjectModeAssertedHead`, and `OuterTwoHopObjectModeAssertedHead ->
MiddleTwoHopObjectModeAssertedHead`. Independent raw reserve-subject Outer and
formula-side asserted Base inputs retain distinct symbols/sites/ranges. The
closed `BindingTwoHopRadix` relation explicitly validates the pairwise-distinct
Outer-to-Middle and Middle-to-Base bare links rather than treating generic
terminal traversal as relation evidence. The route consumes exactly three
expansions, normalizes three known type entries to the terminal base-definition-
RHS builtin-object identity, resolves subject ordinal 1 to `BindingId(0)`, and
produces one inferred variable plus one fact/candidate/diagnostic/deferred-free
checked type assertion with zero expected constraints and without general
reachability, widening, `qua`, or object/set coercion. Existing tests retain 40
direct symbol references across five test files; including ten definitions/
internal calls in `runner.rs`, the pre-move repository has 50 occurrences
across six source-and-test files. All five nonidentity definition orders,
structural/provenance/corruption guards, 37-owner isolation, immutable output,
and a real frontend/resolver sidecar are present. Both focused source and
active-fixture tests pass before the move. Therefore this is move-only
`design_drift`; no ZZE0 test task is needed. `spec_coverage_audit.md` remains
unchanged because authority, behavior, coverage credit, owner crate, and
deferred status do not change. Forbidden changes are config/key/role/mode/
relation edits, route or asserted-head generalization, other route moves,
object/set coercion, assertion weakening, and test/expectation edits.

## Task 263ZZE Move Result

Task 263ZZE moved only the five approved fragments totaling 71 lines into the
existing private `type_elaboration/type_assertion_routes.rs` owner. The
orchestration call remains in place and byte/order-stable. The invalid-key
fragment remains leaf-private; the config, output, and extractor remain test-
only facade imports; and only the production detail route crosses the normal
phase facade. The config-derived runner test alias retains the same name and
value.

After removing the required `pub(in crate::runner)` visibility from the moved
fragments, every old/new fragment pair retains its exact raw hash:
`a8adcfca...`, `de8ce647...`, `77f965e5...`, `23463041...`, and
`71947b90...`. Their combined raw hash remains `55b319a2...`, and the combined
whitespace-normalized hash remains `af5eb98a...`. The item-scoped
`#[rustfmt::skip]` immediately above the moved config is outside this 71-line
oracle and only preserves the original config token layout after adding the
required owner visibility; it has no runtime effect.

The post-move owner is 1,764 lines with SHA-256 `35de4952...`; `runner.rs` is
4,724 lines with `e62ee9af...`; and the phase facade is 444 lines with
`edc843d9...`. Both focused tests, the 272-unit-test crate suite, raw hash
`5e41e4db...` and normalized hash `c0c2b80f...` for the test list, and all four
CLI report hashes remain unchanged. Review-only implementation and test-
sufficiency checks found no source or test finding; their completion-doc drift
finding was repaired in the paired English/Japanese documents. Full workspace
format, Clippy, test, and diff gates pass. No API, name, relation, test,
expectation, trace, diagnostic, key, payload, ordering, fallback, or fail-
closed behavior changed. `spec_coverage_audit.md` remains unchanged because no
authority, behavior, coverage credit, owner crate, or deferred status changed.

## Task 263ZZF Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the three-edge local-object-mode two-
hop asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 686-688 (3 lines, `1992a9ee...`), config at
2227-2271 (45, `92b10e49...`), production detail route at 3272-3284 (13,
`1b9c1049...`), test-only output at 3662-3671 (10, `36138905...`), and
extractor at 4409-4420 (12, `bc67d644...`). Total: 83 lines; combined raw hash
`e7cc3312...` and whitespace-normalized pre-move hash `44bf94d5...`.

Task 263ZZF mechanically moves only those fragments into the existing private
owner. The orchestration call and dispatch order stay in `runner.rs`. Only the
production detail route crosses the normal phase facade. The config, test-
consumed output, and extractor cross under `#[cfg(test)]`; the invalid-key
constant remains leaf-private while a config-derived runner test alias retains
its existing name and value. No public API or call site, helper name, config
value, exact four-expansion object-terminal two-hop relation, key, payload,
ordering, fallback, or fail-closed behavior changes.

Canonical mode unfolding, builtin-object, reserved-theorem-variable, and
static type-assertion requirements apply unchanged. The active `.miz`/
expectation pair and covered Task 214 checker requirement exercise four
ordered real source-derived definitions: `BaseThreeEdgeObjectModeTwoHopAssertedHead
-> object`, `InnerThreeEdgeObjectModeTwoHopAssertedHead ->
BaseThreeEdgeObjectModeTwoHopAssertedHead`,
`MiddleThreeEdgeObjectModeTwoHopAssertedHead ->
InnerThreeEdgeObjectModeTwoHopAssertedHead`, and
`OuterThreeEdgeObjectModeTwoHopAssertedHead ->
MiddleThreeEdgeObjectModeTwoHopAssertedHead`. Independent raw reserve-subject
Outer and formula-side asserted Inner inputs retain distinct symbols/sites/
ranges. The closed `BindingTwoHopRadix` relation explicitly validates the
pairwise-distinct Outer-to-Middle and Middle-to-Inner bare links; the remaining
Inner-to-Base-to-object tail is terminal-normalization evidence only, never
generic relation evidence. The route consumes exactly four expansions,
normalizes three known type entries to the terminal base-definition-RHS
builtin-object identity, resolves subject ordinal 1 to `BindingId(0)`, and
produces one inferred variable plus one fact/candidate/diagnostic/deferred-free
checked type assertion with zero expected constraints and without general
reachability, widening, `qua`, or object/set coercion. Existing tests retain 39
direct symbol references across four test files; including ten definitions/
internal calls in `runner.rs`, the pre-move repository has 49 occurrences
across five source-and-test files. All 23 nonidentity definition orders,
structural/provenance/corruption guards, 39-owner isolation, immutable output,
and a real frontend/resolver sidecar are present. Both focused source and
active-fixture tests pass before the move. Therefore this is move-only
`design_drift`; no ZZF0 test task is needed. `spec_coverage_audit.md` remains
unchanged because authority, behavior, coverage credit, owner crate, and
deferred status do not change. Forbidden changes are config/key/role/mode/
relation edits, route or asserted-head generalization, other route moves,
object/set coercion, assertion weakening, and test/expectation edits.

## Task 263ZZF Move Result

Task 263ZZF moved only the five approved fragments totaling 83 lines into the
existing private `type_elaboration/type_assertion_routes.rs` owner. The
orchestration call remains in place and byte/order-stable. The invalid-key
fragment remains leaf-private; the config, output, and extractor remain test-
only facade imports; and only the production detail route crosses the normal
phase facade. The config-derived runner test alias retains the same name and
value.

After removing the required `pub(in crate::runner)` visibility from the moved
fragments, every old/new fragment pair retains its exact raw hash:
`1992a9ee...`, `92b10e49...`, `1b9c1049...`, `36138905...`, and
`bc67d644...`. Their combined raw hash remains `e7cc3312...`, and the combined
whitespace-normalized hash remains `44bf94d5...`. The item-scoped
`#[rustfmt::skip]` immediately above the moved config is outside this 83-line
oracle and only preserves the original config token layout after adding the
required owner visibility; it has no runtime effect.

The post-move owner is 1,853 lines with SHA-256 `cc72d6a6...`; `runner.rs` is
4,644 lines with `5136a010...`; and the phase facade is 448 lines with
`6de8b48e...`. Both focused tests, the 272-unit-test crate suite, raw hash
`5e41e4db...` and normalized hash `c0c2b80f...` for the test list, and all four
CLI report hashes remain unchanged. Review-only implementation and test-
sufficiency checks found no source or test finding; their completion-doc drift
finding was repaired in the paired English/Japanese documents. Full workspace
format, Clippy, test, and diff gates pass. No API, name, relation, test,
expectation, trace, diagnostic, key, payload, ordering, fallback, or fail-
closed behavior changed. `spec_coverage_audit.md` remains unchanged because no
authority, behavior, coverage credit, owner crate, or deferred status changed.

## Task 263ZZG Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the four-edge local-object-mode two-
hop asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 695-696 (2 lines, `875cc99e...`), config at
2274-2325 (52, `21f37ed8...`), production detail route at 3243-3255 (13,
`c38d179a...`), test-only output at 3616-3625 (10, `3acc53a2...`), and
extractor at 4355-4366 (12, `1ff744db...`). Total: 89 lines; combined raw hash
`c786476b...` and whitespace-normalized pre-move hash `70b18cc8...`.

Task 263ZZG mechanically moves only those fragments into the existing private
owner. The orchestration call and dispatch order stay in `runner.rs`. Only the
production detail route crosses the normal phase facade. The config, test-
consumed output, and extractor cross under `#[cfg(test)]`; the invalid-key
constant remains leaf-private while a config-derived runner test alias retains
its existing name and value. No public API or call site, helper name, config
value, exact five-expansion object-terminal two-hop relation, key, payload,
ordering, fallback, or fail-closed behavior changes.

Canonical mode unfolding, builtin-object, reserved-theorem-variable, and
static type-assertion requirements apply unchanged. The active `.miz`/
expectation pair and covered Task 216 checker requirement exercise five
ordered real source-derived definitions: `BaseFourEdgeObjectModeTwoHopAssertedHead
-> object`, `InnerFourEdgeObjectModeTwoHopAssertedHead ->
BaseFourEdgeObjectModeTwoHopAssertedHead`,
`MiddleFourEdgeObjectModeTwoHopAssertedHead ->
InnerFourEdgeObjectModeTwoHopAssertedHead`,
`OuterFourEdgeObjectModeTwoHopAssertedHead ->
MiddleFourEdgeObjectModeTwoHopAssertedHead`, and
`TooDeepFourEdgeObjectModeTwoHopAssertedHead ->
OuterFourEdgeObjectModeTwoHopAssertedHead`. Independent raw reserve-subject
TooDeep and formula-side asserted Middle inputs retain distinct symbols/sites/
ranges. The closed `BindingTwoHopRadix` relation explicitly validates the
pairwise-distinct TooDeep-to-Outer and Outer-to-Middle bare links; the remaining
Middle-to-Inner-to-Base-to-object tail is terminal-normalization evidence only,
never generic relation evidence. The route consumes exactly five expansions,
normalizes three known type entries to the terminal base-definition-RHS
builtin-object identity, resolves subject ordinal 1 to `BindingId(0)`, and
produces one inferred variable plus one fact/candidate/diagnostic/deferred-free
checked type assertion with zero expected constraints and without general
reachability, widening, `qua`, or object/set coercion. Existing tests retain 37
direct symbol references across three test files; including ten definitions/
internal calls in `runner.rs`, the pre-move repository has 47 occurrences
across four source-and-test files. All 119 nonidentity definition orders,
structural/provenance/corruption guards, 41-owner isolation, immutable output,
and a real frontend/resolver sidecar are present. Both focused source and
active-fixture tests pass before the move. Therefore this is move-only
`design_drift`; no ZZG0 test task is needed. `spec_coverage_audit.md` remains
unchanged because authority, behavior, coverage credit, owner crate, and
deferred status do not change. Forbidden changes are config/key/role/mode/
relation edits, route or asserted-head generalization, other route moves,
object/set coercion, assertion weakening, and test/expectation edits.

## Task 263ZZG Move Result

Task 263ZZG moved only the five approved fragments totaling 89 lines into the
existing private `type_elaboration/type_assertion_routes.rs` owner. The
orchestration call remains in place and byte/order-stable. The invalid-key
fragment remains leaf-private; the config, output, and extractor remain test-
only facade imports; and only the production detail route crosses the normal
phase facade. The config-derived runner test alias retains the same name and
value.

After removing the required `pub(in crate::runner)` visibility from the moved
fragments, every old/new fragment pair retains its exact raw hash:
`875cc99e...`, `21f37ed8...`, `c38d179a...`, `3acc53a2...`, and
`1ff744db...`. Their combined raw hash remains `c786476b...`, and the combined
whitespace-normalized hash remains `70b18cc8...`. The item-scoped
`#[rustfmt::skip]` immediately above the moved config is outside this 89-line
oracle and only preserves the original config token layout after adding the
required owner visibility; it has no runtime effect.

The post-move owner is 1,948 lines with SHA-256 `1ffac900...`; `runner.rs` is
4,558 lines with `cc6c99ea...`; and the phase facade is 452 lines with
`0058287b...`. Both focused tests, the 272-unit-test crate suite, raw hash
`5e41e4db...` and normalized hash `c0c2b80f...` for the test list, and all four
CLI report hashes remain unchanged. Review-only implementation and test-
sufficiency checks found no source or test finding; their completion-doc drift
finding was repaired in the paired English/Japanese documents. Full workspace
format, Clippy, test, and diff gates pass. No API, name, relation, test,
expectation, trace, diagnostic, key, payload, ordering, fallback, or fail-
closed behavior changed. `spec_coverage_audit.md` remains unchanged because no
authority, behavior, coverage credit, owner crate, or deferred status changed.

## Task 263ZZH Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the three-edge local-object-mode
three-hop asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 704-706 (3 lines, `200f40ee...`), config at
2318-2363 (46, `45c230ab...`), production detail route at 3209-3221 (13,
`43a51871...`), test-only output at 3565-3574 (10, `0bf5b9ec...`), and
extractor at 4295-4306 (12, `7a10d6e3...`). Total: 84 lines; combined raw hash
`da6e9082...` and whitespace-normalized pre-move hash `2858ac57...`.

Task 263ZZH mechanically moves only those fragments into the existing private
owner. The orchestration call and dispatch order stay in `runner.rs`. Only the
production detail route crosses the normal phase facade. The config, test-
consumed output, and extractor cross under `#[cfg(test)]`; the invalid-key
constant remains leaf-private while a config-derived runner test alias retains
its existing name and value. No public API or call site, helper name, config
value, exact four-expansion object-terminal three-hop relation, key, payload,
ordering, fallback, or fail-closed behavior changes.

Canonical mode unfolding, builtin-object, reserved-theorem-variable, and
static type-assertion requirements apply unchanged. The active `.miz`/
expectation pair and covered Task 218 checker requirement exercise four ordered
real source-derived definitions: `BaseThreeEdgeObjectModeThreeHopAssertedHead
-> object`, `InnerThreeEdgeObjectModeThreeHopAssertedHead ->
BaseThreeEdgeObjectModeThreeHopAssertedHead`,
`MiddleThreeEdgeObjectModeThreeHopAssertedHead ->
InnerThreeEdgeObjectModeThreeHopAssertedHead`, and
`OuterThreeEdgeObjectModeThreeHopAssertedHead ->
MiddleThreeEdgeObjectModeThreeHopAssertedHead`. Independent raw reserve-subject
Outer and formula-side asserted Base inputs retain distinct symbols/sites/
ranges. The closed `BindingThreeHopRadix` relation explicitly validates the
pairwise-distinct Outer-to-Middle, Middle-to-Inner, and Inner-to-Base bare
links; Base-to-object is terminal-normalization evidence only, never generic
relation evidence. The route consumes exactly four expansions, normalizes
three known type entries to the base-definition-RHS builtin-object identity,
resolves subject ordinal 1 to `BindingId(0)`, and produces one inferred
variable plus one fact/candidate/diagnostic/deferred-free checked type assertion
with zero expected constraints and without general reachability, widening,
`qua`, or object/set coercion. Existing tests retain 35 direct symbol references
across three test files; including ten definitions/internal calls in
`runner.rs`, the pre-move repository has 45 occurrences across four source-and-
test files. All 23 nonidentity definition orders, structural/provenance/
corruption guards, 43-owner isolation, immutable output, and a real frontend/
resolver sidecar are present. Both focused source and active-fixture tests pass
before the move. Therefore this is move-only `design_drift`; no ZZH0 test task
is needed. `spec_coverage_audit.md` remains unchanged because authority,
behavior, coverage credit, owner crate, and deferred status do not change.
Forbidden changes are config/key/role/mode/relation edits, route or asserted-
head generalization, other route moves, object/set coercion, assertion
weakening, and test/expectation edits.

## Task 263ZZH Move Result

Task 263ZZH moved only the five approved fragments totaling 84 lines into the
existing private `type_elaboration/type_assertion_routes.rs` owner. The
orchestration call remains in place and byte/order-stable. The invalid-key
fragment remains leaf-private; the config, output, and extractor remain test-
only facade imports; and only the production detail route crosses the normal
phase facade. The config-derived runner test alias retains the same name and
value.

After removing the required `pub(in crate::runner)` visibility from the moved
fragments, every old/new fragment pair retains its exact raw hash:
`200f40ee...`, `45c230ab...`, `43a51871...`, `0bf5b9ec...`, and
`7a10d6e3...`. Their combined raw hash remains `da6e9082...`, and the combined
whitespace-normalized hash remains `2858ac57...`. The item-scoped
`#[rustfmt::skip]` immediately above the moved config is outside this 84-line
oracle and only preserves the original config token layout after adding the
required owner visibility; it has no runtime effect.

The post-move owner is 2,038 lines with SHA-256 `839ec141...`; `runner.rs` is
4,477 lines with `caefe049...`; and the phase facade is 456 lines with
`e6fb91cd...`. Both focused tests, the 272-unit-test crate suite, raw hash
`5e41e4db...` and normalized hash `c0c2b80f...` for the test list, and all four
CLI report hashes remain unchanged. Review-only implementation and test-
sufficiency checks found no source or test finding; their completion-doc drift
finding was repaired in the paired English/Japanese documents. Full workspace
format, Clippy, test, and diff gates pass. No API, name, relation, test,
expectation, trace, diagnostic, key, payload, ordering, fallback, or fail-
closed behavior changed. `spec_coverage_audit.md` remains unchanged because no
authority, behavior, coverage credit, owner crate, or deferred status changed.

## Task 263ZZI Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the four-edge local-object-mode
three-hop asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 718-720 (3 lines, `edb310ad...`), config at
2366-2418 (53, `d0731bdf...`), production detail route at 3180-3192 (13,
`e3a909cd...`), test-only output at 3519-3528 (10, `ca4a50d5...`), and
extractor at 4240-4251 (12, `e1ef45a6...`). Total: 91 lines; combined raw hash
`532110f6...` and whitespace-normalized pre-move hash `5b0b96fa...`.

Task 263ZZI mechanically moves only those fragments into the existing private
owner. The orchestration call and dispatch order stay in `runner.rs`. Only the
production detail route crosses the normal phase facade. The config, test-
consumed output, and extractor cross under `#[cfg(test)]`; the invalid-key
constant remains leaf-private while a config-derived runner test alias retains
its existing name and value. No public API or call site, helper name, config
value, exact five-expansion object-terminal three-hop relation, key, payload,
ordering, fallback, or fail-closed behavior changes.

Canonical mode unfolding, builtin-object, reserved-theorem-variable, and
static type-assertion requirements apply unchanged. The active `.miz`/
expectation pair and covered Task 220 checker requirement exercise five ordered
real source-derived definitions: `BaseFourEdgeObjectModeThreeHopAssertedHead
-> object`, `InnerFourEdgeObjectModeThreeHopAssertedHead ->
BaseFourEdgeObjectModeThreeHopAssertedHead`,
`MiddleFourEdgeObjectModeThreeHopAssertedHead ->
InnerFourEdgeObjectModeThreeHopAssertedHead`,
`OuterFourEdgeObjectModeThreeHopAssertedHead ->
MiddleFourEdgeObjectModeThreeHopAssertedHead`, and
`TooDeepFourEdgeObjectModeThreeHopAssertedHead ->
OuterFourEdgeObjectModeThreeHopAssertedHead`. Independent raw reserve-subject
TooDeep and formula-side asserted Inner inputs retain distinct symbols/sites/
ranges. The closed `BindingThreeHopRadix` relation explicitly validates the
pairwise-distinct TooDeep-to-Outer, Outer-to-Middle, and Middle-to-Inner bare
links; Inner-to-Base-to-object is terminal-normalization evidence only, never
generic relation evidence. The route consumes exactly five expansions,
normalizes three known type entries to the base-definition-RHS builtin-object
identity, resolves subject ordinal 1 to `BindingId(0)`, and produces one
inferred variable plus one fact/candidate/diagnostic/deferred-free checked type
assertion with zero expected constraints and without general reachability,
widening, `qua`, or object/set coercion. Existing tests retain 33 direct symbol
references across three test files; including ten definitions/internal calls
in `runner.rs`, the pre-move repository has 43 occurrences across four source-
and-test files. All 119 nonidentity definition orders, separate unconnected-
deeper and connected sixth-edge guards, structural/provenance/corruption
guards, 45-owner isolation, focused Tasks 208 and 211-219 regressions,
immutable output, and a real frontend/resolver sidecar are present. Both
focused active-fixture and synthetic-exactness tests pass before the move.
Therefore this is move-only `design_drift`; no ZZI0 test task is needed.
`spec_coverage_audit.md` remains unchanged because authority, behavior,
coverage credit, owner crate, and deferred status do not change. Forbidden
changes are config/key/role/mode/relation edits, route or asserted-head
generalization, other route moves, object/set coercion, assertion weakening,
and test/expectation edits.

## Task 263ZZI Move Result

Task 263ZZI moved only the five approved fragments totaling 91 lines into the
existing private `type_elaboration/type_assertion_routes.rs` owner. The
orchestration call remains in place and byte/order-stable. The invalid-key
fragment remains leaf-private; the config, output, and extractor remain test-
only facade imports; and only the production detail route crosses the normal
phase facade. The config-derived runner test alias retains the same name and
value.

After removing the required `pub(in crate::runner)` visibility from the moved
fragments, every old/new fragment pair retains its exact raw hash:
`edb310ad...`, `d0731bdf...`, `e3a909cd...`, `ca4a50d5...`, and
`e1ef45a6...`. Their combined raw hash remains `532110f6...`, and the combined
whitespace-normalized hash remains `5b0b96fa...`. The item-scoped
`#[rustfmt::skip]` immediately above the moved config is outside this 91-line
oracle and only preserves the original config token layout after adding the
required owner visibility; it has no runtime effect.

The post-move owner is 2,135 lines with SHA-256 `009a2787...`; `runner.rs` is
4,389 lines with `ef501d02...`; and the phase facade is 460 lines with
`791d0685...`. Both focused tests, the 272-unit-test crate suite, raw hash
`5e41e4db...` and normalized hash `c0c2b80f...` for the test list, and all four
CLI report hashes remain unchanged. Review-only implementation and test-
sufficiency checks found no source or test finding; their completion-doc drift
finding was repaired in the paired English/Japanese documents. Full workspace
format, Clippy, test, and diff gates pass. No API, name, relation, test,
expectation, trace, diagnostic, key, payload, ordering, fallback, or fail-
closed behavior changed. `spec_coverage_audit.md` remains unchanged because no
authority, behavior, coverage credit, owner crate, or deferred status changed.

## Task 263ZZJ Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the four-edge local-object-mode four-
hop asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It forms five exact `runner.rs`
fragments: the invalid key at 719-721 (3 lines, `74c201e4...`), config at
2415-2468 (54, `11d9b1e0...`), production detail route at 3143-3155 (13,
`106f10db...`), test-only output including its attached `#[cfg(test)]` at
3465-3474 (10, `58454b88...`), and extractor at 4178-4189 (12,
`d18d2fca...`). Total: 92 lines; combined raw hash `a6b73ffc...` and
whitespace-normalized pre-move hash `61f5421e...`.

Task 263ZZJ mechanically moves only those fragments into the existing private
owner. The orchestration call and dispatch order stay in `runner.rs`. Only the
production detail route crosses the normal phase facade. The config, test-
consumed output, and extractor cross under `#[cfg(test)]`; the invalid-key
constant remains leaf-private while a config-derived runner test alias retains
its existing name and value. No public API or call site, helper name, config
value, exact five-expansion object-terminal four-hop relation, key, payload,
ordering, fallback, or fail-closed behavior changes.

Canonical reserved-variable closure, mode unfolding, builtin-object, and
static type-assertion requirements apply unchanged. The active `.miz`/
expectation pair and covered Task 222 checker requirement exercise five ordered
real source-derived definitions: `BaseFourEdgeObjectModeFourHopAssertedHead ->
object`, `InnerFourEdgeObjectModeFourHopAssertedHead ->
BaseFourEdgeObjectModeFourHopAssertedHead`,
`MiddleFourEdgeObjectModeFourHopAssertedHead ->
InnerFourEdgeObjectModeFourHopAssertedHead`,
`OuterFourEdgeObjectModeFourHopAssertedHead ->
MiddleFourEdgeObjectModeFourHopAssertedHead`, and
`TooDeepFourEdgeObjectModeFourHopAssertedHead ->
OuterFourEdgeObjectModeFourHopAssertedHead`. Independent raw reserve-subject
TooDeep and formula-side asserted Base inputs retain distinct symbols/sites/
ranges. The closed `BindingFourHopRadix` relation explicitly validates the
pairwise-distinct TooDeep-to-Outer, Outer-to-Middle, Middle-to-Inner, and Inner-
to-Base bare links; Base-to-object is terminal-normalization evidence only,
never generic relation evidence. The route consumes exactly five expansions,
normalizes three known type entries to the base-definition-RHS builtin-object
identity, resolves subject ordinal 1 to `BindingId(0)`, and produces one
inferred variable plus one fact/candidate/diagnostic/deferred-free checked type
assertion with zero expected constraints and without general reachability,
widening, `qua`, or object/set coercion. Existing tests retain 31 direct symbol
references across three test files; including ten definitions/internal calls
in `runner.rs`, the pre-move repository has 41 occurrences across four source-
and-test files. All 119 nonidentity definition orders, separate unconnected-
deeper and connected sixth-definition/fifth-link guards, structural/
provenance/corruption guards, 47-owner isolation, focused Task 208 and Tasks
211-221 regressions, immutable output, and a real frontend/resolver sidecar are
present. Both focused active-fixture and synthetic-exactness tests pass before
the move. Therefore this is move-only `design_drift`; no ZZJ0 test task is
needed. `spec_coverage_audit.md` remains unchanged because authority, behavior,
coverage credit, owner crate, and deferred status do not change. Forbidden
changes are config/key/role/mode/relation edits, route or asserted-head
generalization, other route moves, object/set coercion, assertion weakening,
and test/expectation edits.

## Task 263ZZJ Move Result

Task 263ZZJ moved only the five approved fragments totaling 92 lines into the
existing private `type_elaboration/type_assertion_routes.rs` owner. The
orchestration call remains in place and byte/order-stable. The invalid-key
fragment remains leaf-private; the config, output, and extractor remain test-
only facade imports; and only the production detail route crosses the normal
phase facade. The config-derived runner test alias retains the same name and
value.

After removing the required `pub(in crate::runner)` visibility from the moved
fragments, every old/new fragment pair retains its exact raw hash:
`74c201e4...`, `11d9b1e0...`, `106f10db...`, `58454b88...`, and
`d18d2fca...`. Their combined raw hash remains `a6b73ffc...`, and the combined
whitespace-normalized hash remains `61f5421e...`. The output fragment's
attached `#[cfg(test)]` is inside the 92-line oracle and preserves test-only
visibility. The item-scoped `#[rustfmt::skip]` immediately above the moved
config is outside this oracle and only preserves the original config token
layout after adding the required owner visibility; it has no runtime effect.

The post-move owner is 2,233 lines with SHA-256 `407a215c...`; `runner.rs` is
4,300 lines with `de2a5351...`; and the phase facade is 464 lines with
`a1424de3...`. Both focused tests, the 272-unit-test crate suite, raw hash
`5e41e4db...` and normalized hash `c0c2b80f...` for the test list, and all four
CLI report hashes remain unchanged. The pre-implementation review's semantic
`#[cfg(test)]` boundary finding was repaired before the move. Full workspace
format, Clippy, test, and diff gates pass. No API, name, relation, test,
expectation, trace, diagnostic, key, payload, ordering, fallback, or fail-
closed behavior changed. `spec_coverage_audit.md` remains unchanged because no
authority, behavior, coverage credit, owner crate, or deferred status changed.

## Task 263ZZK Pre-Move Inventory and Specification

Fresh authority, test, trace, expectation, design, source, API, and real-
producer/consumer inventory selects only the three-edge local-object-mode
immediate-radix asserted-head route as the next bounded addition to private
`type_elaboration/type_assertion_routes.rs`. It is the next physical unowned
local-object route after Task 263ZZJ and forms five exact `runner.rs` fragments:
the invalid key at 734-735 (2 lines, `65bca3b2...`), config at 2456-2499 (44,
`736e409a...`), production detail route at 3105-3117 (13, `dd971d62...`), test-
only output including its attached `#[cfg(test)]` at 3410-3419 (10,
`0d343a10...`), and extractor at 4115-4126 (12, `fc9ad737...`). Total: 81
lines; combined raw hash `92ee1ca1...` and whitespace-normalized pre-move hash
`07df31bc...`.

Task 263ZZK mechanically moves only those fragments into the existing private
owner. The orchestration call and dispatch order stay in `runner.rs`. Only the
production detail route crosses the normal phase facade. The config, test-
consumed output, and extractor cross under `#[cfg(test)]`; the invalid-key
constant remains leaf-private while a config-derived runner test alias retains
its existing name and value. No public API or call site, helper name, config
value, exact four-expansion object-terminal immediate-radix relation, key,
payload, ordering, fallback, or fail-closed behavior changes.

Canonical reserved-variable closure, mode unfolding, builtin-object, and
static type-assertion requirements apply unchanged. The active `.miz`/
expectation pair and covered Task 206 checker requirement exercise four ordered
real source-derived definitions: `BaseThreeEdgeObjectModeRadixAssertedHead ->
object`, `InnerThreeEdgeObjectModeRadixAssertedHead ->
BaseThreeEdgeObjectModeRadixAssertedHead`,
`MiddleThreeEdgeObjectModeRadixAssertedHead ->
InnerThreeEdgeObjectModeRadixAssertedHead`, and
`OuterThreeEdgeObjectModeRadixAssertedHead ->
MiddleThreeEdgeObjectModeRadixAssertedHead`. Independent raw reserve-subject
Outer and formula-side asserted Middle inputs retain distinct symbols/sites/
ranges. The closed `BindingImmediateRadix` relation validates only the exact
Outer-to-Middle bare link; Middle-to-Inner-to-Base-to-object is terminal-
normalization evidence only, never multi-hop or generic reachability evidence.
The route consumes exactly four expansions, normalizes three known type entries
to the base-definition-RHS builtin-object identity, resolves subject ordinal 1
to `BindingId(0)`, and produces one inferred variable plus one fact/candidate/
diagnostic/deferred-free checked type assertion with zero expected constraints
and without widening, `qua`, or object/set coercion. Existing tests retain 61
direct symbol references across seven test files; including ten definitions/
internal calls in `runner.rs`, the pre-move repository has 71 occurrences
across eight source-and-test files. All 23 nonidentity definition orders, exact
structural/provenance/corruption and per-definition near-miss guards, the
documented Task 206 bidirectional owner isolation plus all later cross-owner
regressions, immutable output, and a real frontend/resolver sidecar are
present. Both focused exhaustive-source and active-fixture tests pass before
the move. Therefore this is move-only `design_drift`; no ZZK0 test task is
needed. `spec_coverage_audit.md` remains unchanged because authority, behavior,
coverage credit, owner crate, and deferred status do not change. Forbidden
changes are config/key/role/mode/relation edits, route or asserted-head
generalization, other route moves, object/set coercion, assertion weakening,
and test/expectation edits.

## Task 263ZZK Move Result

Task 263ZZK moved only the five approved fragments totaling 81 lines into the
existing private `type_elaboration/type_assertion_routes.rs` owner. The
orchestration call and dispatch order remain in place and byte/order-stable.
The invalid-key fragment remains leaf-private; the config, output, and
extractor remain test-only facade consumers; and only the production detail
route crosses the normal facade. The output's attached `#[cfg(test)]` remains
inside the preservation oracle, while the formatting control remains outside
it.

After stripping only the required `pub(in crate::runner)` visibility, the five
post-move fragments retain their original hashes `65bca3b2...`, `736e409a...`,
`dd971d62...`, `0d343a10...`, and `fc9ad737...`; the combined raw hash remains
`92ee1ca1...`, and the pre-recorded normalized oracle remains `07df31bc...`.
The resulting source inventory is 4,222 lines / `cfd9724b...` for `runner.rs`,
468 / `5982f427...` for the phase facade, and 2,320 / `2f59862b...` for the
route owner.

Both focused route tests and all 272 crate unit tests pass. The raw and
normalized discovered-test hashes remain `5e41e4db...` and `c0c2b80f...`; the
plan, parse-only, declaration-symbol, and type-elaboration CLI hashes remain
`f3424007...`, `57d0fba9...`, `08b00a9f...`, and `1dadbeab...`. Formatting,
Clippy with denied warnings, workspace tests, and diff checks pass. No API,
test name, diagnostic, detail key, payload, ordering, fail-closed behavior,
authority, coverage credit, owner crate, or deferred status changed.
`spec_coverage_audit.md` therefore remains unchanged.

## Current Ownership

| Current area | Responsibility | Dependency direction | Audit decision |
|---|---|---|---|
| public report/result/status types and `run_*_corpus` functions | Stable public runner facade and corpus-level orchestration | plan/discovery to phase execution | Keep in `runner.rs`. |
| source/frontend and resolver staging | Source package preparation and cleanup, root/path/snapshot identity, frontend execution/result transport, common frontend diagnostic projection, and resolver shell/projection/symbol collection | shared by parse, declaration-symbol, and type-elaboration as applicable | Frontend staging moved in Task 258, declaration/type resolver collection in Task 260A, and common frontend diagnostic projection in Task 263B to private `shared.rs` with minimal parent-only visibility. |
| active-case admission and stable failure assembly | Tag/phase gates, expected-output matching, and deterministic failure diagnostics | phase-specific facade-to-owner transition | Tasks 259 and 260B moved parse-only and declaration case/failure boundaries. Task 263C moved type expected-key/failure projection to private `type_elaboration/result.rs`, and Task 263D moved type tag/runnable/gate admission to private `type_elaboration/admission.rs`; type case execution and actual-detail dispatch remain in `runner.rs` for fresh Task 263 inventory. |
| parse-only execution | Surface-AST snapshots and parse-only failure projection | shared frontend to parse-only result | Moved in Task 259 to private `parse_only.rs` with minimal parent-only visibility. |
| fixture import provider | Parser fixture lexical summaries and type import-summary adapters | parser/frontend seams shared by active phases | Moved in Task 261 to private `import_fixtures.rs`; later phases retain the same provider and adapter paths. |
| declaration-symbol observation | Consume the shared resolver result and assemble deterministic payload, expected-value, and failure projections | shared resolver output to declaration-symbol result | Moved in Task 260B to private `declaration_symbol.rs`; existing integration tests remain in `tests/metadata.rs`. |
| type-elaboration admission/execution | Lower-stage fail-closed gates and checker/core handoff dispatch | resolver output to source bridge | Task 263A moved generic checker-handoff assembly/validation to private `checker_handoff.rs`, Task 263C moved expected-key/failure projection to private `result.rs`, Task 263D moved active admission to private `admission.rs`, Tasks 263E-263F moved checker-output transports/builders, Tasks 263G-263I moved type-assertion/binary/shared-parenthesized validation, Tasks 263J-263M moved type-assertion/binary/parenthesized detail and payload-detail cores to private `output.rs`, Task 263N moved the cohesive parenthesized route owner to private `parenthesized_routes.rs`, Tasks 263O-263ZD moved the leading through both long-chain binary route owners to private `binary_routes.rs`, Task 263ZB moved the two shared long-chain definition tables to private `long_chain_config.rs`, and Tasks 263ZE-263ZZK started the type-assertion/asserted-head route owner through the direct, chained, two-edge, three-edge, four-edge, and long-chain builtin reserved-variable type assertions plus the direct, chained, and two-edge same-mode, chained, two-edge, and three-edge immediate-radix, two-edge, three-edge, and four-edge two-hop, the three-edge and four-edge three-hop, the four-edge four-hop, and long-chain same-mode, immediate-radix, six-, five-, four-, three-, and two-hop object-terminal routes; top-level case execution, dispatch, remaining local-object-mode type-assertion/asserted-head/formula configs and named wrappers, and other output consumers remain in `runner.rs`. The phase facade owns eleven private leaves. |
| source extraction | Exact source-shape recognition and real AST/resolver payload construction | syntax/resolver inputs to checker inputs | Tasks 262A-262B moved common source-AST primitives/projections and Task 262D moved the shared exact fixture-import projection to private `type_elaboration/source_ast.rs`; Tasks 262C/262E moved reserve type-expression/symbol projection, declaration segmentation, and local-mode expansion to private `type_elaboration/source_reserve.rs`; Tasks 262F-262Q moved standalone formula constants, shared exact numerals, builtin binary and type-assertion formulas, the shared imported-formula symbol resolver/provenance pair, imported predicate/functor, imported attribute assertion, set-enumeration, connective/quantifier families, and the shared, direct-binary, parenthesized, and type-assertion reserved-variable source substrate to private `type_elaboration/source_formula.rs`. Formula source extraction is complete; Tasks 263N-263ZZK colocated named extractors through both long-chain binary route families, all local-mode long-chain type-assertion/asserted-head routes, and the local-object-mode direct/chained/two-edge/three-edge/four-edge/long-chain builtin reserved-variable type assertions plus the direct, chained, and two-edge same-mode, chained, two-edge, and three-edge immediate-radix, two-edge, three-edge, and four-edge two-hop, the three-edge and four-edge three-hop, the four-edge four-hop, and long-chain same-mode, immediate-radix, six-, five-, four-, three-, and two-hop routes with their owners, while remaining local-object-mode type-assertion/asserted-head/formula route configs/wrappers plus checker/output consumers stay in `runner.rs` for Task 263 inventory. |
| payload validation and detail-key rendering | Exact checker/core output validation, expected/actual matching, deterministic keys, diagnostics | source bridge output to runner result | Tasks 263E-263I moved the three shared output transports/builders plus type-assertion/binary/shared-parenthesized validators and private helpers to private `type_elaboration/output.rs`; Tasks 263J-263M moved the type-assertion, binary, and shared parenthesized result/detail plus payload-detail cores there too; Tasks 263N-263ZZK moved the parenthesized through both long-chain binary configs, all local-mode long-chain type-assertion/asserted-head configs, and the local-object-mode direct/chained/two-edge/three-edge/four-edge/long-chain builtin reserved-variable type assertions plus the direct, chained, and two-edge same-mode, chained, two-edge, and three-edge immediate-radix, two-edge, three-edge, and four-edge two-hop, the three-edge and four-edge three-hop, the four-edge four-hop, and long-chain same-mode, immediate-radix, six-, five-, four-, three-, and two-hop configs and named detail/output wrappers to their route leaves. Remaining local-object-mode type-assertion/asserted-head/formula named wrappers/configs remain bounded work. No key or ordering edits. |
| fixture builders and corruption probes | AST/env/sidecar builders and finite negative matrices | test support to private production seams | Private test support/fragments only. |
| cross-owner isolation tests | Bidirectional route rejection and immutable/module guards | all supported source-bridge owners | Keep intact and move as a cohesive fragment. |

## Dependency Map

The permitted dependency direction is:

```text
public runner facade
  -> parse-only owner
     -> shared plan/admission/source/frontend/diagnostic staging
        -> fixture/import-summary owner (lexical provider)
  -> declaration-symbol owner
     -> shared plan/admission/source/frontend/diagnostic/resolver staging
        -> fixture/import-summary owner (lexical provider)
  -> type-elaboration owner
     -> active tag, runnable-admission, and gate validation
     -> shared plan/admission/source/frontend/diagnostic/resolver staging
        -> fixture/import-summary owner (lexical provider)
     -> fixture/import-summary owner (resolver adapter)
     -> source extraction
        -> common source-AST primitives
           -> fixture/import-summary owner (module-path projection)
        -> reserve type-expression, declaration, and local-mode projection
        -> standalone formula-constant, shared exact numeral, builtin binary/type-assertion,
           shared imported-symbol, imported predicate/functor, imported attribute,
           set-enumeration, connective/quantifier, and shared/direct-binary/
           parenthesized/type-assertion reserved-variable source projections
     -> checker-handoff assembly and readiness validation
     -> checker-output transports, builders, validation, and type-assertion detail projection
     -> expected-result and failure projection
     -> checker/core payload validation and deterministic actual-detail keys

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
| `src/runner/shared.rs` | Private source package preparation, frontend execution, common frontend diagnostic projection, admission support, and genuinely cross-phase helpers. |
| `src/runner/parse_only.rs` | Parse-only case execution, snapshots, and parse-only failure projection. |
| `src/runner/declaration_symbol.rs` | Declaration-symbol case execution, resolver observation, payload keys, and failure projection. |
| `src/runner/import_fixtures.rs` | Existing parser fixture summaries/adapters used by active phases. |
| `src/runner/type_elaboration.rs` and `src/runner/type_elaboration/` | Type-elaboration orchestration plus private source-extraction, checker-handoff, and payload-validation/detail/diagnostic leaves. |
| `src/runner/type_elaboration/binary_routes.rs` | Leading, multiple-reserve declaration, base membership/inequality, direct local-mode, direct local-object-mode, chained local-mode, chained local-object-mode, two-edge local-mode, two-edge local-object-mode, three-edge local-mode, three-edge local-object-mode, four-edge local-mode, four-edge local-object-mode, and both local-mode/local-object-mode long-chain membership/equality/inequality binary configs plus thin source/detail/test route wrappers. |
| `src/runner/type_elaboration/long_chain_config.rs` | Shared exact set-terminal and object-terminal seven-expansion definition tables used by long-chain binary, type-assertion, and asserted-head routes. |
| `src/runner/type_elaboration/type_assertion_routes.rs` | Active owner for reserved-variable type-assertion/asserted-head configs plus thin source/detail/test route wrappers; Tasks 263ZE-263ZZK own all local-mode long-chain builtin, same-mode, immediate-radix, two-hop, three-hop, four-hop, five-hop, and six-hop routes plus the local-object-mode direct/chained/two-edge/three-edge/four-edge/long-chain builtin reserved-variable type assertions and the direct, chained, and two-edge same-mode, chained, two-edge, and three-edge immediate-radix, two-edge, three-edge, and four-edge two-hop, the three-edge and four-edge three-hop, the four-edge four-hop, and long-chain same-mode, immediate-radix, six-, five-, four-, three-, and two-hop asserted-head routes. |
| `src/runner/type_elaboration/parenthesized_routes.rs` | Cohesive parenthesized reserved-variable configs plus thin source/detail/test route wrappers. |
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
| 262 | Complete through Tasks 262N0-262Q: moved every inventoried type-elaboration formula source-extraction leaf; checker/output consumers remain Task 263 work. |
| 262A | Complete: moved the five common exact source-AST primitives behind the private type-elaboration phase facade. |
| 262B | Complete: moved shared node-kind traversal and qualified-symbol spelling projections into the common source-AST leaf. |
| 262C | Complete: moved reserve type-expression, visible symbol/admission, and source-text projection into the private source-reserve leaf; retained declaration/mode callers for Task 262E. |
| 262D | Complete: moved the shared exact `parser.type_fixtures` import-item AST projection to the common source-AST leaf before its formula and reserve callers. |
| 262E | Complete: moved the bounded reserve declaration-segmentation and local-mode traversal/expansion family, retained handoff/formula ownership, and narrowed the three temporary Task 262C helpers. |
| 262F | Complete: moved only the standalone `thesis`/`contradiction` formula-constant transport, exact extractor, and dedicated node allowlist into the new private source-formula leaf, with facade aliases for only the two entries. |
| 262G | Complete: moved only the shared three-helper exact numeral AST projection into the private source-formula leaf while retaining all five caller families in `runner.rs`. |
| 262H0 | Complete: strengthened the existing builtin-binary unit matrix for config order, payload provenance, recovery, duplicate, and cardinality preservation without changing production or test count. |
| 262H | Complete: moved only the builtin equality/inequality/membership config, source transport, exact extractor, and dedicated allowlist into the private source-formula leaf. |
| 262I0 | Complete: strengthened the existing builtin type-assertion unit matrix for independently derived payload/checker provenance, recovery, duplicate, token-shape, and cardinality preservation without changing production or test count. |
| 262I | Complete: moved only the builtin type-assertion transport, exact extractor, and dedicated allowlist into the private source-formula leaf. |
| 262J0 | Complete: strengthened the existing imported predicate/functor matrix for independent payload/checker/import provenance plus recovery, duplicate, and structural-cardinality preservation without changing production or test count. |
| 262J1 | Complete: moved only the shared imported formula symbol resolver/provenance pair into the private source-formula leaf with all three callers unchanged. |
| 262J2 | Complete: moved only the imported predicate/functor transport, exact extractor, exact infix projection, and dedicated allowlist into the private source-formula leaf. |
| 262K0 | Complete: strengthened both existing imported attribute assertion variants for independent five-field payload/provenance/order preservation and bounded direct-rejection corruption coverage without changing production or test count. |
| 262K | Complete: moved only the imported attribute assertion transport, two-entry/shared extractor, and dedicated allowlist into the private source-formula leaf. |
| 262L0 | Complete: strengthened the existing exact set-enumeration matrix for independent eight-field grouping/punctuation/order preservation and bounded direct-rejection corruption coverage without changing production or test count. |
| 262L | Complete: moved only the set-enumeration transport, exact extractor, exact-set projection, and dedicated allowlist into the private source-formula leaf. |
| 262M0 | Complete: strengthened the existing exact connective/quantifier matrix for independent ten-field binder/output/diagnostic preservation and bounded direct-rejection corruption coverage without changing production or test count. |
| 262M | Complete: moved only the connective/quantifier transport, exact extractor, and dedicated allowlist into the private source-formula leaf. |
| 262N0 | Complete: strengthened the existing exact reserved-variable equality matrix for all config fields, independently derived formula/operand provenance, direct near-miss rejection, and 16 bounded default-off corruptions without changing production or test count. |
| 262N | Complete: moved the four normalized shared reserved-variable config/model, predicate, mode/identifier, and ordinal substrate fragments with minimal runner-scoped visibility. |
| 262O | Complete: moved only the direct reserved-variable binary source transport, generic extractor, and family allowlist into the private source-formula leaf; the temporary allowlist alias remains solely for Task 262P's retained parenthesized family. |
| 262P | Complete: moved only the parenthesized reserved-variable source enum/transport, generic extractor, single-parenthesized operand projection, and family allowlist; both binary allowlists are now leaf-private. |
| 262Q0 | Complete: strengthened the existing base reserved-variable type-assertion test for all ten source fields, exact config, direct rejection of all 15 near misses, and four bounded structural corruptions without changing test count or production. |
| 262Q | Complete: moved only the reserved-variable type-assertion source transport, generic extractor, and family allowlist after Q0; retained all 58 configs/wrappers and checker/output consumers. |
| 263 | Decomposed parent: move checker-handoff, payload-validation, detail-key, expected-output, and failure-diagnostic leaves in bounded dependency order. |
| 263A | Complete: moved the exact 506-line checker-handoff substrate to private `type_elaboration/checker_handoff.rs` with minimal runner-scoped visibility. |
| 263B | Complete: moved the exact 49-line common frontend diagnostic projection into existing private `shared.rs` with three parent-only entries. |
| 263C | Complete: moved the exact 24-line expected-result/failure-projection family into private `type_elaboration/result.rs` with two parent-only entries and exact-body/byte-stability preservation. |
| 263D | Complete: moved the exact four-fragment 50-line type active-admission gate into private `type_elaboration/admission.rs` with two parent-only entries and exact-body/byte-stability preservation. |
| 263E | Complete: moved the exact 33-line three-transport checker-output substrate into private `type_elaboration/output.rs` with runner-scoped field visibility and exact-body/byte-stability preservation. |
| 263F | Complete: moved the exact 277-line three-builder/output-projection producer family into existing private `type_elaboration/output.rs` with three parent-only builder entries and exact-body/byte-stability preservation. |
| 263G | Complete: moved only the exact 229-line type-assertion validator/private role helper/shared normalized-type predicate family into existing private `type_elaboration/output.rs`; validator and temporarily shared predicate are parent-only, the role helper is leaf-private, and all preservation gates pass. |
| 263H | Complete: moved only the exact 380-line binary-formula validator/source-projection/type-entry-helper family into existing private `type_elaboration/output.rs`; only the validator is parent-only, all helpers are leaf-private, and all preservation gates pass. |
| 263I | Complete: moved only the exact 67-line config-independent parenthesized-binary validator core into existing private `type_elaboration/output.rs` with one parent-only entry; all configs, named wrappers, detail, and call sites remain retained, and all preservation gates pass. |
| 263J | Complete: moved only the exact 46-line type-assertion result/detail core into existing private `type_elaboration/output.rs`; result projection is parent-only, collector leaf-private, direct validator/output aliases test-only, and all preservation gates pass. |
| 263K | Complete: moved only the exact 36-line binary-formula result/detail core into existing private `type_elaboration/output.rs`; both entries are parent-only, direct validator/output aliases test-only, all parenthesized/config/wrapper/call-site work retained, and all preservation gates pass. |
| 263L | Complete: moved only the exact 16-line shared parenthesized-binary output-detail core into existing private `type_elaboration/output.rs`; shared core parent-only, direct parenthesized validator/output and binary detail-collector aliases test-only, all payload/config/wrapper/call-site work retained, and all preservation gates pass. |
| 263M | Complete: moved only the exact 17-line parenthesized-binary payload-detail wrapper into existing private `type_elaboration/output.rs`; wrapper parent-only, direct builder/shared-detail aliases test-only, all config/named-wrapper/extractor/call-site work retained, and all preservation gates pass. |
| 263N | Complete: moved only the exact seven-fragment/720-line parenthesized config/named-route family into new private `type_elaboration/parenthesized_routes.rs`; normal facade exposes eight detail routes, the test facade exposes only retained test consumers, and all preservation gates pass. |
| 263O | Complete: moved only the exact eight-fragment/546-line leading direct-binary route family into new private `type_elaboration/binary_routes.rs`; normal facade exposes nine detail routes, test facade exposes only retained test consumers, and all preservation gates pass. |
| 263P | Complete: moved only the corrected exact five-fragment/313-line multiple-reserve declaration binary route family into existing private `type_elaboration/binary_routes.rs`; normal facade exposes five detail routes, test facade exposes only retained test consumers, and all preservation gates pass. |
| 263Q | Complete: moved only the exact five-fragment/116-line base reserved-variable membership/inequality route family into existing private `type_elaboration/binary_routes.rs`; normal facade exposes two detail routes, test facade exposes only retained test consumers, and all preservation gates pass. |
| 263R | Complete: moved only the exact ten-fragment/183-line direct local-mode membership/equality/inequality route family into existing private `type_elaboration/binary_routes.rs`; normal facade exposes three detail routes, test facade exposes only retained test consumers, and all preservation gates pass. |
| 263S | Complete: moved only the exact ten-fragment/190-line direct local-object-mode membership/equality/inequality route family into existing private `type_elaboration/binary_routes.rs`; normal facade exposes three detail routes, test facade exposes only retained test consumers, and all preservation gates pass. |
| 263T | Complete: moved only the exact fourteen-fragment/207-line chained local-mode membership/equality/inequality route family into existing private `type_elaboration/binary_routes.rs`; normal facade exposes three detail routes, test facade exposes only retained test consumers, and all preservation gates pass. |
| 263U | Complete: moved only the exact nine-fragment/229-line chained local-object-mode membership/equality/inequality route family into existing private `type_elaboration/binary_routes.rs`; normal facade exposes three detail routes and test facade exposes only retained test consumers. |
| 263V | Complete: moved only the exact fifteen-fragment/222-line two-edge local-mode membership/equality/inequality route family into existing private `type_elaboration/binary_routes.rs`; normal facade exposes three detail routes and test facade exposes only retained test consumers. |
| 263W | Complete: moved only the exact eleven-fragment/241-line two-edge local-object-mode membership/equality/inequality route family into existing private `type_elaboration/binary_routes.rs`; normal facade exposes three detail routes and test facade exposes only retained test consumers. |
| 263X | Complete: moved only the exact fifteen-fragment/242-line three-edge local-mode membership/equality/inequality route family into existing private `type_elaboration/binary_routes.rs`; normal facade exposes three detail routes and test facade exposes only retained test consumers. |
| 263Y | Complete: moved only the exact eleven-fragment/258-line three-edge local-object-mode membership/equality/inequality route family into existing private `type_elaboration/binary_routes.rs`; normal facade exposes three detail routes and test facade exposes only retained test consumers. |
| 263Z | Complete: moved only the exact fifteen-fragment/252-line four-edge local-mode membership/equality/inequality route family into existing private `type_elaboration/binary_routes.rs`; normal facade exposes three detail routes and test facade exposes only retained test consumers. |
| 263ZA | Complete: moved only the exact eleven-fragment/273-line four-edge local-object-mode membership/equality/inequality route family into existing private `type_elaboration/binary_routes.rs`; normal facade exposes three detail routes, test facade exposes only retained test consumers, and all preservation gates pass. |
| 263ZB | Complete prerequisite: moved only the exact two-fragment/74-line shared long-chain seven-expansion tables to new private `type_elaboration/long_chain_config.rs`; all 22 consumer configs/routes remain in place and all preservation gates pass. |
| 263ZC | Complete: moved only the exact fifteen-fragment/176-line local-mode long-chain membership/equality/inequality route family into existing private `type_elaboration/binary_routes.rs`; sibling table import and all preservation gates pass. |
| 263ZD | Complete: moved only the exact fifteen-fragment/186-line local-object-mode long-chain membership/equality/inequality route family into existing private `type_elaboration/binary_routes.rs`; sibling table import and all preservation gates pass. |
| 263ZE | Complete: moved only the exact five-fragment/52-line local-mode long-chain reserved-variable type-assertion route into new private `type_elaboration/type_assertion_routes.rs`; every asserted-head and local-object-mode route remains in place and all preservation gates pass. |
| 263ZF | Complete: moved only the exact five-fragment/48-line local-mode long-chain same-mode asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every radix, multi-hop, and local-object-mode route remains in place and all preservation gates pass. |
| 263ZG | Complete: moved only the exact five-fragment/50-line local-mode long-chain immediate-radix asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every multi-hop and local-object-mode route remains in place and all preservation gates pass. |
| 263ZH | Complete: moved only the exact five-fragment/51-line local-mode long-chain two-hop asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every deeper-hop and local-object-mode route remains in place and all preservation gates pass. |
| 263ZI | Complete: moved only the exact five-fragment/54-line local-mode long-chain three-hop asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every four-or-deeper and local-object-mode route remains in place and all preservation gates pass. |
| 263ZJ | Complete: moved only the exact five-fragment/55-line local-mode long-chain four-hop asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every five/six-hop and local-object-mode route remains in place and all preservation gates pass. |
| 263ZK | Complete: moved only the exact five-fragment/56-line local-mode long-chain five-hop asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every six-hop and local-object-mode route remains in place and all preservation gates pass. |
| 263ZL | Complete: moved only the exact five-fragment/55-line local-mode long-chain six-hop asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every local-object-mode route remains in place and all preservation gates pass. |
| 263ZM | Complete: moved only the exact five-fragment/58-line local-object-mode long-chain six-hop asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every other local-object-mode route remains in place and all preservation gates pass. |
| 263ZN | Complete: moved only the exact five-fragment/57-line local-object-mode long-chain five-hop asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every other local-object-mode route remains in place and all preservation gates pass. |
| 263ZO | Complete: moved only the exact five-fragment/56-line local-object-mode long-chain four-hop asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every other local-object-mode route remains in place and all preservation gates pass. |
| 263ZP | Complete: moved only the exact five-fragment/55-line local-object-mode long-chain three-hop asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every other local-object-mode route remains in place and all preservation gates pass. |
| 263ZQ | Complete: moved only the exact five-fragment/54-line local-object-mode long-chain two-hop asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every other local-object-mode route remains in place. |
| 263ZR | Complete: moved only the exact five-fragment/52-line local-object-mode long-chain immediate-radix asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every other local-object-mode route remains in place. |
| 263ZS | Complete: moved only the exact five-fragment/50-line local-object-mode long-chain same-mode asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every other local-object-mode route remains in place. |
| 263ZT | Complete: moved only the exact five-fragment/52-line local-object-mode long-chain reserved-variable builtin type-assertion route into existing private `type_elaboration/type_assertion_routes.rs`; every other route remains in place. |
| 263ZU | Complete: moved only the exact five-fragment/53-line direct local-object-mode reserved-variable builtin type-assertion route into existing private `type_elaboration/type_assertion_routes.rs`; every other route remains in place. |
| 263ZV | Complete: moved only the exact five-fragment/67-line chained local-object-mode reserved-variable builtin type-assertion route into existing private `type_elaboration/type_assertion_routes.rs`; every other route remains in place. |
| 263ZW | Complete: moved only the exact five-fragment/71-line two-edge local-object-mode reserved-variable builtin type-assertion route into existing private `type_elaboration/type_assertion_routes.rs`; every other route remains in place. |
| 263ZX | Complete: moved only the exact five-fragment/82-line three-edge local-object-mode reserved-variable builtin type-assertion route into existing private `type_elaboration/type_assertion_routes.rs`; every other route remains in place. |
| 263ZY | Complete: moved only the exact five-fragment/81-line four-edge local-object-mode reserved-variable builtin type-assertion route into existing private `type_elaboration/type_assertion_routes.rs`; every other route remains in place. |
| 263ZZ | Complete: moved only the exact five-fragment/55-line direct local-object-mode same-mode asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every other route remains in place. |
| 263ZZA | Complete: moved only the exact five-fragment/63-line chained local-object-mode same-mode asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every other route remains in place. |
| 263ZZB | Complete: moved only the exact five-fragment/65-line chained local-object-mode immediate-radix asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every other route remains in place. |
| 263ZZC | Complete: moved only the exact five-fragment/68-line two-edge local-object-mode same-mode asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every other route remains in place. |
| 263ZZD | Complete: moved only the exact five-fragment/72-line two-edge local-object-mode immediate-radix asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every other route remains in place. |
| 263ZZE | Complete: moved only the exact five-fragment/71-line two-edge local-object-mode two-hop asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every other route remains in place. |
| 263ZZF | Complete: moved only the exact five-fragment/83-line three-edge local-object-mode two-hop asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every other route remains in place. |
| 263ZZG | Complete: moved only the exact five-fragment/89-line four-edge local-object-mode two-hop asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every other route remains in place. |
| 263ZZH | Complete: moved only the exact five-fragment/84-line three-edge local-object-mode three-hop asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every other route remains in place. |
| 263ZZI | Complete: moved only the exact five-fragment/91-line four-edge local-object-mode three-hop asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every other route remains in place. |
| 263ZZJ | Complete: moved only the exact five-fragment/92-line four-edge local-object-mode four-hop asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every other route remains in place. |
| 263ZZK | Complete: moved only the exact five-fragment/81-line three-edge local-object-mode immediate-radix asserted-head route into existing private `type_elaboration/type_assertion_routes.rs`; every other route remains in place. |
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
| `test_gap` | Tasks 262H0, 262I0, 262J0, 262K0, 262L0, 262M0, 262N0, and 262Q0 repair bounded preservation-matrix gaps before their corresponding move-only tasks; no behavior or coverage credit changes. |
| `spec_gap`, `source_drift`, `test_expectation_drift` | None introduced or repaired by this series. |
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
