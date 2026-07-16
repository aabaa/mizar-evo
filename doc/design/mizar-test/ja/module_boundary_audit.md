# Module-Boundary Audit: mizar-test Runner

> 正本は英語です。英語版:
> [../en/module_boundary_audit.md](../en/module_boundary_audit.md)。

## Task 248 Gate

Task 248 は source move の前に active runner 実装を監査する。この maintenance
series は source layout と reviewability の `design_drift` を修復するものであり、
Mizar language behavior、runner admission、public API、diagnostic、detail key、
payload、ordering、expectation meaning、traceability credit は変更しない。

authority order は `doc/spec/en` > `.miz` tests > `spec_trace.toml` >
expectations > design > source のままである。Chapter 03、04、07、13、14、16
と既存 executable intent は runner への input であり、この refactor の変更対象
ではない。[harness.md](./harness.md)、[minimal_crate.md](./minimal_crate.md)、
[expectation_schema.md](./expectation_schema.md)、
[internal 07](../../internal/ja/07.crate_module_layout.md) が derived harness と
ownership boundary を定義する。

## Baseline

Task 248 inventory 時点:

- `src/runner.rs` は 111,262 行。
- pre-test prefix は 17,142 行目で終わり、public runner facade、private phase
  helper、137 個の `#[cfg(test)]` helper attribute を含む。
- private `mod tests` は 17,143 行目から始まり、約 94,120 行。
- private module は `#[test]` attribute 272 個を持つ。direct scope が 244 個、
  既存 nested task module が 28 個。
- direct test は parse-only import-provider test 1 個と type-elaboration の
  source extraction、payload、fixture、corruption、cross-owner isolation family。
- declaration-symbol runner test は `tests/metadata.rs` が integration owner。
  move すべき private declaration-symbol test は存在しない。
- active type-elaboration runner は 188 cases、metadata plan は 403 cases /
  367 requirements、type-elaboration coverage は 235 / 223、pass/fail は
  219 / 184、unit-test count は 272。

## Task 249 Move Result

Task 249 は inline module を private `#[cfg(test)]`、
`#[rustfmt::skip] mod tests;` に置換し、body を byte-for-byte で
`src/runner/tests.rs` へ移動した。formatter guard により、この move-only task
中に newly top-level となった test import と body が reorder/reflow されない。
runner file は 17,144 行、test module は 94,118 行となった。exact extracted
body hash は
`ab658ad10bcbb2d415778f6289cbb9ae2bed48e21c19b5496fa8f676309d3b69`、
sorted 272-test list は
`5e41e4dbfcc303322c246a612de61926a628957a168589b45864d0a5070bb07e`
のままである。module privacy、qualified test name、public API、active-runner
count、diagnostic、payload、ordering、fail-closed behavior は不変である。

## Task 250 Move Result

Task 250 は wrapper module を作らず `src/runner/tests/support.rs` を
root-include した。6,546 行の fragment は 17 import group と、連続した shared
environment、fixture specification、AST builder、corruption、range、id support
（non-test function 201 個、type/constant item 24 個）を含む。exact moved hash は
`b880b4605345b1156f125292134d62aff91a32799b5f5834fe7d2a1e5de068a8`。
retained 87,572 行は byte-identical のままで hash は
`197f2d6dc31da2130674954667383bb9aec502a613f3e5b1c33bf0299ea2959b`、
結果の 87,574 行の `tests.rs` hash は
`7d85a8ecd4dffcb0475afc53693e581af661ccbb01b44eab974e030abb046a66`
である。272 tests はすべて同じ sorted-name hash で `runner::tests` に残る。

## Task 251 Move Result

Task 251 は wrapper module を作らず `src/runner/tests/parse_only.rs` を
root-include した。byte-identical な 111 行の fragment は
`parse_only_provider_resolves_every_stub_and_deduplicates_fixture_summaries`
だけを含み、hash は
`3cddce85155b72597cfc4c2ea5841dbf3fe5f88d0c8123d98ba9cb958f90a3a8`。
separator blank を含む retained 87,463 行は byte-identical のままで hash は
`010f86378bca27c0620998c0de0242d6376fb8b3c37c002d0ca430fb01f7e35c`、
結果の 87,464 行の `tests.rs` hash は
`16480c65416a611c732153360775f10180f609b012027b0a0a970cff1f5a3d84`
である。fully qualified test name と sorted 272-test list は不変である。

## Task 252 Move Result

Task 252 は wrapper module を作らず
`src/runner/tests/type_elaboration/source_extraction.rs` を root-include した。
byte-identical な 3,680 行の fragment は baseline reserve extraction、
local-mode expansion-chain extraction、real declaration-checked
`ResolvedTypedAst` handoff の 3 tests を含み、hash は
`aa9a16c3ed36439ac8c5a4756e3818d6e5f0abd7e076e2e2df8b46487e88c358`。
Task 253 前の separator を含む retained 83,784 行は byte-identical のままで
hash は
`2d9ef7d8369c4d654af3bd91598d306c8a9777c9d0981454ce9396095c8a6d79`、
結果の 83,785 行の `tests.rs` hash は
`16f3d6ceb1e75655ea39825f0294896393e676d0a7391bb2a409e14b3b904d22`
である。3 fully qualified names と sorted 272-test list は不変である。

## Task 253A Move Result

Task 253A は wrapper module を作らず
`src/runner/tests/type_elaboration/reserved_binary.rs` を root-include した。
byte-identical な 9,982 行の fragment は、監査済み Task 189/246 ownership
exception を含む先頭の baseline reserved-variable / binary-formula bridge
23 tests を含み、hash は
`88f1a793e139ea808c823fd68956d0dc9863735905ae3fb34e214efa86a11d8e`。
Task 254 前の separator を含む retained 73,803 行は byte-identical のままで
hash は
`faf592952a4c871b840b6a1cbbb977ca3f1bbddc98def4f99d54c1a900fdcb06`、
結果の 73,804 行の `tests.rs` hash は
`97d05a3dc35774246af301ad7b4dc6601d2ab85ca669bebfdbcfa140767d150f`。
23 fully qualified names、元の order 位置、canonical raw/secondary normalized
272-test list hash はすべて不変である。Task 253 は Task 253B まで pending。

## Task 254 Move Result

Task 254 は wrapper module を作らず
`src/runner/tests/type_elaboration/mode_chain.rs` を root-include した。
byte-identical な 10,232 行の fragment は監査済み non-long-chain
local-mode/object-mode 26 tests を含み、hash は
`2989031d64871c726f325a5d5bd2ebb4ed4b9a078b83bab3c4f04f456cf3225f`。
Task 253B 前の separator を含む retained 63,572 行は byte-identical のままで
hash は
`6725980d7842af5c398f58139ce371ac64d8912ba744f4417ac20c88165d5d81`、
結果の 63,573 行の `tests.rs` hash は
`7e5d0f5735c551be19ac13b2dc96732bf4a9f3cd7088317beb22c760e0d03b68`。
26 fully qualified names、元の order 位置、Task 253B boundary、両 272-test list
hash はすべて不変である。

## Task 253B Move Result

Task 253B は wrapper module を作らず
`src/runner/tests/type_elaboration/reserved_direct.rs` を root-include した。
byte-identical な 284 行の fragment は監査済み direct reserved-variable
membership/inequality 2 tests を含み、hash は
`c65a5f27463950979368bc702e36f42fa0398884029cff450b54b31095f30e4e`。
Task 255 前の separator を含む retained 63,289 行は byte-identical のままで
hash は
`fffe06106cca615e370bb4c2da222da5a4bc21a264cadb5ae8c2d79ed7fdbcce`、
結果の 63,290 行の `tests.rs` hash は
`c90905d94abd1a43c0d65d4abffe8bc970262eee2d64e22da1db4024d614bbf4`。
2 fully qualified names、元の order 位置、Task 255 boundary、両 272-test list
hash はすべて不変である。parent Task 253 は完了。

## Task 255A Move Result

Task 255A は wrapper module を作らず
`src/runner/tests/type_elaboration/asserted_head_base.rs` を root-include した。
byte-identical な 6,653 行の fragment は監査済み先頭 source test 12件と専用
Task 205 isolation helper 2件を含み、hash は
`9ecea3c52ae64b83d6d5de9b825307f31c7d331e3ba29d78bb69cd931709d020`。
Task 255B 前の separator を含む retained 56,637 行は byte-identical のままで
hash は
`d9f772962e590f49d188ca1d0cbe8cf5863b7dd84bb9e73606f878f33036007a`、
結果の 56,638 行の `tests.rs` hash は
`535968a53524b3741d9adeed0ee6e42f2e4c45184af285a2bae077810b3bd682`。
12 fully qualified names、helper ownership、元の order 位置、Task 255B
boundary、両 272-test list hash はすべて不変である。parent Task 255 は pending。

## Task 255B Move Result

Task 255B は wrapper module を作らず
`src/runner/tests/type_elaboration/asserted_head_four_edge_radix.rs` を
root-include した。byte-identical な 3,303 行の fragment は監査済み four-edge
radix source test 2件と専用 Task 208/207 helper function 4件を含み、hash は
`5fcc4240fff400bda08e3d6678a61f3db444f0a8c6c055802d7ba7bea961092e`。
Task 255C 前の separator を含む retained 53,335 行は byte-identical のままで
hash は
`16d36bc1978973931a673a7620c569c70b021fe4ed210e19540a0ee8fa7c7d9d`、
結果の 53,336 行の `tests.rs` hash は
`78594f98a92a30445d251cf0fb394e5537ecab73cf9b8e9c67c357e4a0135389`。
2 fully qualified names、4 helper owner、元の order 位置、Task 255C boundary、
両 272-test list hash はすべて不変である。parent Task 255 は pending。

## Task 255C Move Result

Task 255C は wrapper module を作らず
`src/runner/tests/type_elaboration/asserted_head_three_edge_object_radix.rs` を
root-include した。byte-identical な 1,278 行の fragment は監査済み three-edge
object-radix source test 1件と専用 Task 206 helper function 2件を含み、hash は
`c5c1b04ab663fe3557e24c86b551352d6d1c54c5511870ba224edb7538f95442`。
Task 255D 前の separator を含む retained 52,058 行は byte-identical のままで
hash は
`e841b80390d879d910bfc50a34547ef56b8b2ab40c6c4b9681e8b07f707dc12b`、
結果の 52,059 行の `tests.rs` hash は
`23caa0585a96be2db997295fccad436de5bfefdbe033fdd4516ca8e30dacea9f`。
fully qualified name、2 helper owner、元の order 位置、Task 255D boundary、両
272-test list hash はすべて不変である。parent Task 255 は pending。

## Task 255D Move Result

Task 255D は wrapper module を作らず
`src/runner/tests/type_elaboration/asserted_head_two_edge_object_radix.rs` を
root-include した。byte-identical な 1,046 行の fragment は監査済み two-edge
object-radix source test 1件と専用 Task 204 helper function を含み、hash は
`e20a04ba33ffc1f344da0aa990795576b7096eb6a016a69d730d0d29377349f4`。
Task 255E 前の separator を含む retained 51,013 行は byte-identical のままで
hash は
`16d6ec2333861ac9d78d3694efe76a71bb9a9830f16def60c4a425fb7da63dc7`、
結果の 51,014 行の `tests.rs` hash は
`68bf3cf08b26a449f46aee00d7fe8f716d1663ac9aeb7005b311f4f7c6c15906`。
fully qualified name、helper owner、元の order 位置、Task 255E boundary、両
272-test list hash はすべて不変である。parent Task 255 は pending。

## Task 255E Move Result

Task 255E は wrapper module を作らず
`src/runner/tests/type_elaboration/asserted_head_type_assertion.rs` を
root-include した。byte-identical な 7,649 行の fragment は監査済み最後の
non-long-chain type-assertion/asserted-head source test 16件を含み、helper
function は含まない。hash は
`27bb8b3f17cabfce79ec9e32e390fbad3c9356c845dab4c7fb53dfd9f3b5160a`。
最初の active fixture 前の separator を含む retained 43,365 行は
byte-identical のままで hash は
`b0465c9378a8f0151e0c58ba4986876f3de163ceb5918b7ceb49db4462b6d1c3`、
結果の 43,366 行の `tests.rs` hash は
`75fc0ff2b4a48362a1184185ea1315c0d8dab90b9b5a9b45a3fafe13b14d7278`。
16 fully qualified names、元の order 位置、assertion、直後の active-fixture
boundary、両 272-test list hash はすべて不変である。parent Task 255 は完了。

## Task 256 Move Result

Task 256 は wrapper module を作らず
`src/runner/tests/type_elaboration/long_chain.rs` を root-include した。
byte-identical な 20,977 行の fragment は監査済み long-chain source/active
seven-expansion test 44件をすべて含む。12件の `next_permutation` function は
test-local finite guard として nested のままであり、module-level helper や
無関係な item は移動していない。fragment hash は
`c4bcb161ac7bbb03593beff0fd55c6fbf8bc1960618a92263d127856e709d8b0`。
retained 22,389 行は byte-identical のままで hash は
`d737b5160458533039c7535423cffa03265deacb719d167e486897a612d7afbf`、
結果の 22,390 行の `tests.rs` hash は
`603263b325a00d45a41ec3087dafab05ab4ebe448fe3be70a7c0d107f907df8d`。
44 fully qualified names、元の order 位置、finite guard、assertion、両
272-test list hash はすべて不変である。直前の four-edge equality test と
直後の four-edge inequality test は `tests.rs` に残り、Task 257 の nested
Task 216-222 module も fragment 外のままである。Task 256 は完了し、Task 257
が次である。

## Task 257A Move Result

Task 257 の fresh inventory により、残る test 141件を contiguous かつ
order-preserving な8 family に分割する。Task 257A は binary-route test 18件、
Task 257B は builtin-object reserve fixture 3件、Task 257C は専用 trace intent
を持つ Task 180 standalone contradiction formula-constant fixture 1件だけを
分離する。Task 257D は distinct/multiple/heterogeneous reserve fixture 11件、
Task 257E は mode-chain fixture 26件、Task 257F は active
reserve/asserted-head/type-assertion fixture 35件と interleaved owner-route
isolation guard 4件、Task 257G は source-gap/equality test 3件、Task 257H は
root source/active bridge fixture 9件、root synthetic/route-isolation test
3件、既存 Task 216-222 module 内 nested test 28件を含む。8 count の合計は
残る root test 113件と nested test 28件に一致する。parent Task 257 は 257H
まで pending。

Task 257A は wrapper module を作らず
`src/runner/tests/type_elaboration/binary_route_fixtures.rs` を root-include
した。byte-identical な 2,960 行の fragment は監査済み binary/parenthesized
active-fixture/route-isolation test 18件を含み、module-level helper や無関係な
item は含まない。hash は
`b00af949465486166f8a5d012dce6b02345aad29b2e576c4b574cf1c6ea23eee`。
Task 257B 前の separator を含む retained 19,430 行は byte-identical のままで
hash は
`d07c5006c01b8975342d95a5fff8c447106c38e8754ddaac2f87be442c7d07a5`、
結果の 19,431 行の `tests.rs` hash は
`e2f877ddf29c6f9e2e22225e97ff4294d7e27affda04145f78a950e567022e5e`。
18 fully qualified names、元の order 位置、assertion、Task 257B boundary、両
272-test list hash はすべて不変である。Task 257A は完了し、parent Task 257
は pending。

## Task 257B Move Result

Task 257B は wrapper module を作らず
`src/runner/tests/type_elaboration/reserve_object_fixtures.rs` を
root-include した。byte-identical な 156 行の fragment は監査済み Task
188/190/189 builtin-object reserve equality/inequality/type-assertion active
fixture 3件だけを含む。hash は
`9cfb91fad7f537fbe790ac8e8206e383b0068a8bdcb14158c554219702d9446f`。
Task 257C 前の separator を含む retained 19,275 行は byte-identical のままで
hash は
`c4459d3170895c98e4d6018ae491adce8889f12351a9a4b834c8669e84eb285d`、
結果の 19,276 行の `tests.rs` hash は
`509d784ce5f2b23c98675fdfcb74324dfede166204067c8c3bdd0a1339ba6d18`。
3 fully qualified names、元の order 位置、assertion、両 272-test list hash は
すべて不変である。Task 180 contradiction fixture は Task 257C の先頭 item
として `tests.rs` に残る。Task 257B は完了し、parent Task 257 は pending。

## Task 257C Move Result

Task 257C は wrapper module を作らず
`src/runner/tests/type_elaboration/formula_constant_fixture.rs` を
root-include した。byte-identical な 53 行の fragment は監査済み Task 180
standalone contradiction active fixture と exact checked
`FormulaKind::Contradiction` payload assertion だけを含む。hash は
`986b9120d84a487093c4ce3392a11eba03d65441cfb66d09ec9c34bc72dc03c5`。
Task 257D 前の separator を含む retained 19,223 行は byte-identical のままで
hash は
`e271687874a614c317a3d0a6a7ff3da5b1081235c9ec18233ddefc91167122a0`、
結果の 19,224 行の `tests.rs` hash は
`a8140de0a533cb4e2f3d4093155d14f188abcef707094a2b10fe5dda469958ad`。
fully qualified name、元の order 位置、assertion、両 reserve-family boundary、
両 272-test list hash はすべて不変である。Task 257C は完了し、Task 257D が
次であり、parent Task 257 は pending。

## Task 257D Move Result

Task 257D は wrapper module を作らず
`src/runner/tests/type_elaboration/reserve_fixtures.rs` を root-include した。
byte-identical な 739 行の fragment は監査済み distinct、multiple-declaration、
heterogeneous reserve active fixture 11件を含み、module-level helper や無関係な
item は含まない。hash は
`24b4811f26418afe9de5efbf0cf3d7ea54be329ddf1255f89bafc38546301b40`。
Task 257E 前の separator を含む retained 18,485 行は byte-identical のままで
hash は
`5dfbf14737caf47e36f7a0c6bb6a1cab58bea8d608da41c0d74cf1fd58eeda4f`、
結果の 18,486 行の `tests.rs` hash は
`4e40491533df5102655f803e899c032d20adbcaf4c68c6e4980867da87849cf0`。
11 fully qualified names、元の order 位置、assertion、Task 257E mode-chain
boundary、両 272-test list hash はすべて不変である。Task 257D は完了し、
parent Task 257 は pending。

## Task 257E Move Result

Task 257E は wrapper module を作らず
`src/runner/tests/type_elaboration/mode_chain_fixtures.rs` を root-include
した。byte-identical な 1,578 行の fragment は監査済み non-long-chain
local-mode/object-mode active membership/equality/inequality fixture 26件を
含み、helper や無関係な item は含まない。hash は
`9e3c1a6e11b01dc257982002379d884f9de24ec5093982d7604e9a988dc2e593`。
Task 257F 前の separator を含む retained 16,908 行は byte-identical のままで
hash は
`dd144c50d0b24adfc690e99f160e5ab73362f6b972107ac71ff6bed0513a3774`、
結果の 16,909 行の `tests.rs` hash は
`cacc1dd5a5fcd2e14526bac47e277d900b0c0f9b56c6cc1bee2b7ea2e7229c3f`。
26 fully qualified names、元の order 位置、assertion、Task 257F boundary、両
272-test list hash はすべて不変である。Task 257E は完了し、parent Task 257
は pending。

## Task 257F Move Result

Task 257F は wrapper module を作らず
`src/runner/tests/type_elaboration/asserted_head_fixtures.rs` を root-include
した。byte-identical な 3,374 行の fragment は監査済み active
reserve/asserted-head/type-assertion fixture 35件と interleaved two-hop
owner-route isolation guard 4件を含む。helper、無関係な item、long-chain test
は含まない。hash は
`19623c52e34c57fc664f01370139ce253a834513c47fe0f6b7b2563f7684bf26`。
Task 257G 前の separator を含む retained 13,535 行は byte-identical のままで
hash は
`4c19658998190c21cbd8a72efa112e29659664d55a7c5b3040ef54ec7cbbb3e8`、
結果の 13,536 行の `tests.rs` hash は
`9e3bb0de8742d0371e4e686815ba70337b8c278a1e26799069baef8758e093ec`。
39 fully qualified names、元の order 位置、expansion/payload/prior-owner
rejection assertion、Task 257G boundary、両 272-test list hash はすべて不変で
ある。Task 257F は完了し、parent Task 257 は pending。

## Task 257G Move Result

Task 257G は wrapper module を作らず
`src/runner/tests/type_elaboration/source_gap_and_equality.rs` を root-include
した。byte-identical な 2,923 行の fragment は source-reserve gap/evidence
fail-closed test と four-edge local-mode equality source/active pair だけを含む。
hash は
`7726ee451322c547406da5c5b3800be2527685df41ca2de4dc60d47644164487`。
retained 10,613 行は byte-identical のままで hash は
`2ebb32f99fa9001d0a5d303deb5f477a369074b54b3b71ba2ea690aa3f38e49c`、
結果の 10,614 行の `tests.rs` hash は
`b1a22962fefb7a2cc54aa37ff5f601c9995bce432d78fa499cfca8e6c35423bf`。
3 fully qualified names、元の order 位置、detail-key/fail-closed assertion、
直後の `long_chain.rs` include、Task 257H start、両 272-test list hash はすべて
不変である。Task 257G は完了し、parent Task 257 は pending。

## Task 257H Move Result

Task 257H は wrapper module を作らず
`src/runner/tests/type_elaboration/remaining_bridges_and_nested_isolation.rs`
を root-include した。byte-identical な 10,578 行の fragment は、最後の root
source/active bridge fixture 9件、root synthetic/route-isolation test 3件、既存
Task 216-222 module 7個の内側にある nested test 28件をすべて含む。hash は
`96a64963bc06ec3f6f076d00296ebb48450611fb6a512d5f16283c2999e43d50`。
retained 36行は byte-identical のままで hash は
`a3cba5854fc315b6c9c3dd20be2fdeaf7a5e972cb7a626299d2dcb2bb6c56f06`、
結果の37行の `tests.rs` hash は
`0e9b7e861a13fe593435ee8169c28658b5290f054789a3e2f73b896fa2b39061`。
40 fully qualified names、元の order 位置、nested module name 7件、bridge/
isolation assertion、両 canonical 272-test list hash はすべて不変である。
Task 257H と parent Task 257 は完了し、private test layout は安定した。次は
Task 258 である。

## Task 258 Move Result

Task 258 は private owner `src/runner/shared.rs` を作成し、cohesive な
source/frontend staging fragment 2個を `runner.rs` から移動した。元の30行の
frontend execution/result fragment hash は
`7d03c8561f87b95d5b777beba830998f44c0cd1cbe72a245c29573a64fa1b0f6`、
元の89行の package/root/path/snapshot fragment hash は
`34fd4b86829394b95f5ae3125c5bf2f010b0ca0357254ea93446e50e7f384672`。
direct dependency import を含む結果の138行の `shared.rs` hash は
`11a52bf7fb0e729ac680df33dfa4b7fd65b9fdd922ee9aca6e9ba4a96d7f8f56`、
17,022行の `runner.rs` hash は
`dde9e23dfb8092be02f3b1139b59dbfddcbb8e55c0c21eac7ad70e1f1fcbda04`。
`run_frontend`、`FrontendRun` とその field、root normalization、共有 module-
path projection だけが parent-only `pub(super)` visibility を使い、package
preparation/cleanup、temp naming、snapshot identity は `shared.rs` private の
ままである。direct import により owner は facade namespace から独立し、明示
import した parent-owned `ParseOnlyImportProvider` だけを generalize や早期移動
せず、Task 261 までの一時 parent dependency として保持する。public surface hash
`0cb48ae8ac2ccdf14595112df24b8a4c083a989a631580e9044707aa514a267e`、
4 CLI byte hash、両 272-test list hash、count、payload、diagnostic、ordering、
fail-closed behavior はすべて不変である。Task 258 は完了し、次は Task 259。

## Task 259 Move Result

Task 259 は private owner `src/runner/parse_only.rs` を作成し、cohesive な
parse-only fragment 3個を `runner.rs` から移動した。元の51行の case-execution
fragment hash は
`6ff68ec8610c9e5ded44f69369850e11d7adfbaf1685f540398fd465d58f4361`、
元の24行の failure-projection fragment hash は
`2504fbeae49d240c8897f50f00303124ab7c0c3d4bde56393a316dc2419d4275`、
元の32行の Surface-AST snapshot comparison fragment hash は
`e8e1698aa3af9e86e80baf03f799af89490782e3202c20ab22a58011f6d65176`。
direct dependency import を含む結果の121行の `parse_only.rs` hash は
`d1c1dd0f0c322f3bd4a6e829e66bf6aeaf0dc01b46d60dd177a7fe8e4619ae5a`、
16,913行の `runner.rs` hash は
`5579a126eccfbbb937e36149d74a940e146619254c1bb8301dca57d191cdfec9`。
case runner と failure projection だけが parent-only `pub(super)` visibility を
使い、snapshot comparison は private のままである。この owner は sibling
`shared::run_frontend` を直接呼び、Task 263 までの明示的な一時 parent diagnostic
dependency として `assertion_diagnostic_codes` と `frontend_error_code` だけを
保持する。fixture import provider と adapter は Task 261 のため `runner.rs` に
残し、fixture ownership を早期移動していない。public surface hash
`0cb48ae8ac2ccdf14595112df24b8a4c083a989a631580e9044707aa514a267e`、
4 CLI byte hash、両 272-test list hash、count、payload、diagnostic、ordering、
fail-closed behavior はすべて不変である。Task 259 は完了し、次は Task 260。

## Task 260A Move Result

fresh Task 260 review は元の production-helper task を、cross-phase resolver
collection leaf の Task 260A と、その declaration-symbol caller の Task 260B という
独立 ownership move 2個へ分割した。Task 260A は shared leaf を先に移動した。元の
29行の `ResolverSymbolCollection` と shell/projection/collection fragment hash は
`b7f13156c77bfc75d5f6a4f1682fe752b4fe9dfd12b3c7c0cd3913cef44458e0`、
元の18行の resolver module-id/diagnostic projection fragment hash は
`d1bed7b1c59ab13e997a72ed492fdfdabf38466a9921c0254be64934846e1c61`、
元の9行の diagnostic-class key fragment hash は
`363ae1321d663c1d597cdf033c449fe0226c87672e2eefd3bf92b819458cb0e0`。
結果の203行の `shared.rs` hash は
`0cd2eb09c043e564470b4003a34dfc4f9e89cb695b1d2df1404b76dd7e8bc299`、
16,851行の `runner.rs` hash は
`72340a9aeca93ec338375b8bfc51beeb47a2499325faf452733c3e1dec48bbab`。
`resolver_symbol_collection`、その result type、既存 declaration/type consumer が
使う result field 3個だけが parent-only `pub(super)` visibility を持ち、module
identity と diagnostic projection は `shared.rs` private のままである。両 caller は
移動も変更もしていない。public surface hash
`0cb48ae8ac2ccdf14595112df24b8a4c083a989a631580e9044707aa514a267e`、
4 CLI byte hash、両 272-test list hash、count、payload、diagnostic、ordering、
fail-closed behavior はすべて不変である。Task 260A は完了し、次は Task 260B。

## Task 260B Move Result

Task 260B は、Task 260A が shared resolver prerequisite を移動した後に private owner
`src/runner/declaration_symbol.rs` を作成した。元の37行の case-execution fragment
hash は
`b58aebc17cd350c5107775b9027d78037b32e0bb1d72782e101746dd6c2d318f`、
元の36行の observation fragment hash は
`8e9bb3e70c1368aa1882bf623b13664ea12129ffc9f6f44a079148f5eee29631`、
元の125行の payload encoding/classification/expected-value projection fragment
hash は
`02df2d29157e2469ca8139178dec9cabd199d25fdfa554749d999556b2b05376`、
元の19行の failure-diagnostic fragment hash は
`3b366648f438663e7412c2e567bb307ff7245b92739f9bbed38a16fd8862573e`。
direct dependency import を含む結果の231行の `declaration_symbol.rs` hash は
`cf29e362d3109fc8a45e366c8abaa9f98baae7329f83c3556fe8452ec3347232`、
16,632行の `runner.rs` hash は
`a6e9d547d68e18e1de2d22ce4393552cf760e8f6b8081fe608f8ffdcab67005d`。
case runner と failure projection だけが parent-only `pub(super)` visibility を
使い、observation、payload encoding/classification、expected-value projection は
private のままである。この owner は `shared::run_frontend` と
`shared::resolver_symbol_collection` を直接 consume する。唯一の一時 parent
dependency `frontend_detail_keys` は child module から parent private item に access
できるため plain private visibility を維持し、Task 263 で common diagnostic family
を移動する。既存 `tests/metadata.rs` integration ownership は不変。public surface
hash `0cb48ae8ac2ccdf14595112df24b8a4c083a989a631580e9044707aa514a267e`、
4 CLI byte hash、両 272-test list hash、count、payload、diagnostic、ordering、
fail-closed behavior はすべて不変である。Tasks 260A-260B は完了し、次は Task 261。

## Task 261 Move Result

Task 261 は単一の `parser.type_fixtures` vocabulary を共有する family 2個を移動し、
private owner `src/runner/import_fixtures.rs` を作成した。元の161行の
type-elaboration import-summary adapter fragment hash は
`98d9ebc8ff104583bca469f66a89c5f90dfd91085f811012fc06d173b6224d8b`、
元の167行の lexical-summary provider/15-symbol vocabulary fragment hash は
`3097dc061f34ef0d08482aa785f7827b38b17a8b15dbc8f9fc0e7ca876a49c34`。
direct dependency import を含む結果の349行の `import_fixtures.rs` hash は
`bb2d10572184600c2121ae680ff936586a8b525eaea7e2a358f1d3b4305bc04d`、
16,293行の `runner.rs` hash は
`5e878da91e11b7d69709e94dfc9ad851e298fe7b46ed111c174696c2e2b12363`。
`ParseOnlyImportProvider`、type import-summary adapter entry、Task 262 parent
caller がまだ consume する module-path projection だけが parent-only `pub(super)`
visibility を使い、vocabulary、environment clone、imported module discovery、
symbol-kind mapping は private のままである。parent private alias を維持し、
`shared.rs` と既存 test support の import を test edit なしで保持した。stub の
order/span identity、per-module deduplication、fingerprint、exact 15-symbol の
kind/arity/operator/rank order、resolver symbol/provenance order、diagnostic は不変。
public surface hash
`0cb48ae8ac2ccdf14595112df24b8a4c083a989a631580e9044707aa514a267e`、
4 CLI byte hash、両 272-test list hash、count、payload、diagnostic、ordering、
fail-closed behavior はすべて不変である。Task 261 は完了し、次は Task 262。

## Current Ownership

| Current area | Responsibility | Dependency direction | Audit decision |
|---|---|---|---|
| public report/result/status type と `run_*_corpus` function | stable public runner facade と corpus-level orchestration | plan/discovery から phase execution | `runner.rs` に残す。 |
| source/frontend と resolver staging | source package preparation/cleanup、root/path/snapshot identity、frontend execution/result transport、resolver shell/projection/symbol collection | 適用範囲で parse、declaration-symbol、type-elaboration が共有 | frontend staging は Task 258、declaration/type resolver leaf は Task 260A で最小 parent-only visibility の private `shared.rs` へ移動済み。 |
| active-case admission と stable failure assembly | tag/phase gate、expected-output matching、deterministic failure diagnostic | phase-specific facade-to-owner transition | Tasks 259/260B で parse-only/declaration case/failure boundary は移動済み。残る type boundary は Tasks 262-263 まで `runner.rs` に保持。 |
| parse-only execution | Surface-AST snapshot と parse-only failure projection | shared frontend から parse-only result | Task 259 で最小 parent-only visibility の private `parse_only.rs` へ移動済み。 |
| fixture import provider | parser fixture lexical summary と type import-summary adapter | active phase が共有する parser/frontend seam | Task 261 で private `import_fixtures.rs` へ移動済み。後段 phase は同じ provider/adapter path を維持。 |
| declaration-symbol observation | shared resolver result を consume し、deterministic payload、expected-value、failure projection を組み立てる | shared resolver output から declaration-symbol result | Task 260B で private `declaration_symbol.rs` へ移動済み。既存 integration test は `tests/metadata.rs` に残す。 |
| type-elaboration admission/execution | lower-stage fail-closed gate と checker/core handoff dispatch | resolver output から source bridge | private type-elaboration owner。 |
| source extraction | exact source-shape recognition と real AST/resolver payload construction | syntax/resolver input から checker input | private type-elaboration leaf owner。caller より先に移す。 |
| payload validation と detail-key rendering | exact checker/core output validation、expected/actual matching、deterministic key、diagnostic | source bridge output から runner result | private type-elaboration leaf owner。key/order は編集しない。 |
| fixture builder と corruption probe | AST/env/sidecar builder と finite negative matrix | test support から private production seam | private test support/fragment のみ。 |
| cross-owner isolation test | bidirectional route rejection と immutable/module guard | 全 supported source-bridge owner | cohesive fragment として保持して移す。 |

## Dependency Map

許可する dependency direction:

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
     -> checker/core payload validation
     -> deterministic detail keys and failure diagnostics

private runner::tests
  -> shared test support and fixture builders
  -> the same private phase seams
```

leaf helper は caller より先に移す。phase module は shared staging に依存してよいが、
parse-only と declaration-symbol は checker/core payload validation に依存しては
ならない。metadata `plan` は payload-free のままにする。

## Target Source Layout

fresh inventory で family がまだ大きすぎると判明した場合、leaf split をさらに
小さくしてよい。ただし empty/synthetic owner module は禁止する。

| Target path | Ownership |
|---|---|
| `src/runner.rs` | public facade、public report/result/status type、public active-case iterator、top-level corpus orchestration のみ。 |
| `src/runner/shared.rs` | private source package preparation、frontend execution、admission support、真に cross-phase な helper。 |
| `src/runner/parse_only.rs` | parse-only case execution、snapshot、parse-only failure projection。 |
| `src/runner/declaration_symbol.rs` | declaration-symbol case execution、resolver observation、payload key、failure projection。 |
| `src/runner/import_fixtures.rs` | active phase が使う既存 parser fixture summary/adapter。 |
| `src/runner/type_elaboration.rs` と `src/runner/type_elaboration/` | type-elaboration orchestration と private source-extraction / payload-validation/detail/diagnostic leaf。 |
| `src/runner/tests.rs` | 単一 private `runner::tests` module と root-level `include!` declaration。 |
| `src/runner/tests/support.rs` | shared test import、builder、environment、id、corruption helper。 |
| `src/runner/tests/parse_only.rs` | nonempty parse-only private test family。 |
| `src/runner/tests/type_elaboration/*.rs` | nonempty cohesive source-extraction、reserved/binary、mode-chain、asserted-head、long-chain、isolation family。 |
| `tests/metadata.rs` | 既存 declaration-symbol integration-test owner。後の独立 inventory が nonempty move を正当化しない限り不変。 |

test fragment は new wrapper module を作らず、`runner::tests` root へ直接 include
する。これにより既存 qualified test name と Task 216-222 の nested module name を
保持する。discovered test list を変える child-module split は禁止する。

Task 253 の fresh inventory により、reserved/binary owner を Task 254 の
mode-chain block の前後にある 2 個の物理的かつ order-preserving な fragment へ
分割する。Task 253A は先頭の baseline reserve/binary 23-test block である。
Task 189 の reserved-object type-assertion test は baseline reserved-object bridge
boundary に埋め込まれ、その owner であるため、この block に保持する。Task 246
の parenthesized two-edge local-mode equality も parenthesized binary route に属する
ため、この block に保持する。この分類によって両 test を広い asserted-head または
mode-chain family へ移管するものではない。253A 後も Task 253 は pending とし、
Task 254 で間の local-mode/object-mode chain block を移動後、Task 253B で直後の
direct reserved-variable membership/inequality test を移動する。Task 255 は direct
reserved-variable type-assertion test から始まる。この順序で source/discovery order
を保持する。

Task 254 の fresh inventory により、この間の block を 26 complete tests に固定
する。これは direct から four-edge までの non-long-chain set/object membership、
equality、inequality family で、long-chain test や non-test helper/item を含まない。
block 後の separator を retained とし、その直後から Task 253B が始まる。Task 254
はこの contiguous block だけを
`src/runner/tests/type_elaboration/mode_chain.rs` へ移動する。

Task 253B の fresh inventory により、次の block を 2 complete direct
reserved-variable tests（membership と inequality）に固定する。non-test
helper/item は含まない。直後の separator を retained とし、その後の direct
reserved-variable type-assertion test から Task 255 が始まる。Task 253B はこの
284-line block だけを
`src/runner/tests/type_elaboration/reserved_direct.rs` へ移動し、この move の完了で
parent Task 253 を完了する。

Task 255 の fresh inventory により、non-long-chain type-assertion/asserted-head
source family を 5 個の物理的かつ order-preserving な block へ分割する。Task 255A
は 6,653 行で、three-edge set-side radix owner までの先頭 source test 12件と、
それらだけが consume する専用 Task 205 isolation helper 2件を含む。Task 255B は
3,303 行で、four-edge set/object radix test 2件と専用 Task 208/207 helper を含む。
Task 255C/255D はそれぞれ single three-edge/two-edge object-radix source test と
専用 Task 206/204 helper を一緒に保持する。Task 255E は最後の contiguous source
test 16件を含み、active-fixture block 前で停止する。各 block 間の separator は
`tests.rs` に残す。これら専用 helper item は owner test と一緒に移動し、standalone
active-fixture/cross-owner-isolation test は後続の fresh inventory に残す。Task 255
subtask は long-chain test を含まない。parent Task 255 は Task 255E まで pending。

## Ordered Move Tasks

| Task | Bounded action |
|---|---|
| 248 | paired audit を追加し、paired crate plan と preservation matrix を更新。source move なし。 |
| 249 | 完了: inline private `mod tests` body 全体を `src/runner/tests.rs` へ機械的に移動。 |
| 250 | 完了: nonempty shared test support を root-included support fragment へ移動。 |
| 251 | 完了: nonempty parse-only private test family を root-included fragment へ移動。 |
| 252 | 完了: baseline type-elaboration source-extraction / real handoff test を移動。 |
| 253A | 完了: 先頭の baseline reserved-variable/binary-formula 23-test block を移動。Task 253 は pending のまま。 |
| 254 | 完了: 26-test non-long-chain local-mode/object-mode chain bridge block を移動し、直後の Task 253B boundary を保持。 |
| 253B | 完了: 2 direct reserved-variable membership/inequality test を `reserved_direct.rs` へ移動し、直後の Task 255 boundary を保持して Task 253 を完了。 |
| 255A | 完了: 先頭 type-assertion/asserted-head source test 12件と専用 Task 205 helper 2件を `asserted_head_base.rs` へ移動。 |
| 255B | 完了: four-edge radix source test 2件と専用 Task 208/207 helper を `asserted_head_four_edge_radix.rs` へ移動。 |
| 255C | 完了: three-edge object-radix source test と専用 Task 206 helper を移動。 |
| 255D | 完了: two-edge object-radix source test と専用 Task 204 helper を移動。 |
| 255E | 完了: 最後の non-long-chain source test 16件を移動し、active-fixture boundary を保持して Task 255 を完了。 |
| 256 | 完了: long-chain source/active bridge test 44件と test-local finite guard 12件を `long_chain.rs` へ移動し、両隣の four-edge boundary を保持。 |
| 257 | 完了: inventory 済みの残る fixture、bridge-gap、corruption、isolation 8 family を Task 257H までにすべて移動。 |
| 257A | 完了: 先頭 binary/parenthesized fixture/route-isolation test 18件を `binary_route_fixtures.rs` へ移動し、Task 257B separator を保持。 |
| 257B | 完了: builtin-object reserve active fixture 3件を `reserve_object_fixtures.rs` へ移動し、Task 257C separator を保持。 |
| 257C | 完了: Task 180 standalone contradiction fixture だけを `formula_constant_fixture.rs` へ移動し、両 reserve-family boundary を保持。 |
| 257D | 完了: distinct/multiple/heterogeneous reserve fixture 11件を `reserve_fixtures.rs` へ移動し、Task 257E separator を保持。 |
| 257E | 完了: non-long-chain active mode-chain fixture test 26件を `mode_chain_fixtures.rs` へ移動し、Task 257F separator を保持。 |
| 257F | 完了: active reserve/asserted-head/type-assertion fixture 35件と interleaved owner-route isolation guard 4件を `asserted_head_fixtures.rs` へ移動し、Task 257G separator を保持。 |
| 257G | 完了: source-gap/four-edge-equality test 3件を `source_gap_and_equality.rs` へ移動し、直後の long-chain include と Task 257H boundary を保持。 |
| 257H | 完了: 最後の root bridge fixture 9件、root isolation test 3件、nested test 28件を `remaining_bridges_and_nested_isolation.rs` へ移動し、Task 216-222 module を保持して Task 257 を完了。 |
| 258 | 完了: test layout 安定後、shared source/frontend staging helper を private `shared.rs` へ移動。 |
| 259 | 完了: parse-only case execution、Surface-AST snapshot comparison、failure projection を private `parse_only.rs` へ移動。 |
| 260A | 完了: cross-phase resolver shell/projection/symbol collection leaf を declaration/type caller より先に private `shared.rs` へ移動。 |
| 260B | 完了: 既存 declaration-symbol case/observation/payload/expectation/failure helper を private `declaration_symbol.rs` へ移動。integration test は不変。 |
| 261 | 完了: lexical provider、exact fixture vocabulary、type import-summary adapter を private `import_fixtures.rs` へ移動。 |
| 262 | type-elaboration source-extraction leaf を移動。 |
| 263 | payload validation、detail key、expected output、failure diagnostic leaf を移動。 |
| 264 | paired source-layout inventory、path table、todo/plan state、ownership guard を closeout。 |

各 source-moving task は nonempty でなければならない。fresh inventory により
smaller family が必要なら編集前に bounded subtask を追加し、no-op commit は
作らない。

## Preservation Matrix

| Surface | Required invariant |
|---|---|
| public API | `mizar_test::runner` re-export、signature、enum attribute、CLI behavior は不変。 |
| tests | function name、fully qualified discovered name、nested module name、discovery order/set、272 tests は不変。 |
| corpus/trace | active runner 188、plan 403/367、type 235/223、pass/fail 219/184、backlink、requirement、expectation meaning は不変。 |
| diagnostics | code、stable detail key、fallback key、text、source identity、ordering は byte-for-byte 不変。 |
| payloads | key、value、shape、provenance、source range、binding identity、deterministic ordering、immutable output は不変。 |
| fail-closed behavior | unsupported、malformed、ambiguous、imported-gap、evidence-gap、lower-stage case は同じ boundary で reject。 |
| authority | move の都合だけで `doc/spec`、`.miz`、expectation、traceability を編集しない。 |

各 move の前後で test 実行に加え、
`cargo test -p mizar-test --lib -- --list` の exact sorted test line を
capture/compare する。`: test` suffix を含む canonical raw-list oracle は 272 行で、
hash は
`5e41e4dbfcc303322c246a612de61926a628957a168589b45864d0a5070bb07e`。
suffix を除いた normalized-name list は secondary oracle に限り、Task 253A 前の
hash は
`c0c2b80f8b4e6c84cd25d77573fda722c4d1846fed168cd4a478781cdb42775e`。

## Classification And Coverage-Audit Impact

| Class | Result |
|---|---|
| `design_drift` | active。source layout が phase/ownership review boundary を隠している。Tasks 249-264 は behavior 変更なしで修復する。 |
| `spec_gap`、`test_gap`、`source_drift`、`test_expectation_drift` | この series が導入または修復するものはない。 |
| `source_undocumented_behavior`、`boundary_violation` | new finding なし。既存 runner behavior は paired harness plan と上位 authority に従う。 |
| `repo_metadata_conflict` | finding なし。 |

`doc/design/spec_coverage_audit.md` は変更しない。この series は specification
chapter coverage、design mapping、traceability status、owner crate、follow-up
ownership、deferred rationale を変更しない。

## Per-Task Review And Verification

各 source move で review-only により visibility drift、test-discovery drift、
owner-boundary drift、source/documentation inconsistency、accidental behavior
change を確認する。required command:

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

active CLI preservation count は parse-only 96、declaration-symbol 4、
type-elaboration 188。

## Exit Criteria

`runner.rs` が public facade/top-level orchestration のみに限定され、各 private
owner の visibility が最小で、preservation matrix が通り、paired source layout、
crate plan、todo、harness path table、bilingual/ownership guard document が同期し、
全 verification が green のときだけ series complete とする。Task 264 後にだけ
fresh Step 5 inventory を再開する。Steps 6/7 は既存 dependency gate 成立まで
deferred のままである。
