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

## Task 262A Move Result

Task 262 の fresh inventory により、reserve/formula extractor owner の双方より先に
移す common source-AST leaf が確認された。Task 262A は private
`src/runner/type_elaboration.rs` phase facade と private
`src/runner/type_elaboration/source_ast.rs` leaf を作り、exact
compilation-item-list recognizer、structural-child projection、direct-token
projection、checker-site projection、recursive recovery predicate を移動した。
元 fragment hash は順に
`84bf7a706ff2295e0087484fda11f210a6f363f4bfa386567004b1b291abcb1b`、
`5684a8ad7fa11893580921465265d7343a6cd1d9824ad5a9b9b6443153380981`、
`3c7621566d18a891f2390be433cf292ee67affcebbc2ea591ee09ffddb1bc5d3`、
`a12398685131398da0a9a3a0200d1b7e988be6d1e12ea7dc6a9fe9019eab7bb4`、
`9aa975ae84b5bed868095e19969b5db18f59a113d96963f21656f2358fb87326`。
結果の63行の `source_ast.rs` hash は
`e785097028171a78e3f8764618ac4bced422756b4c1a985e72de3138ae46a1ed`、
6行の facade hash は
`a5d786f3fce6b7d6b5661918e4fb46a3116b41f33fe307adebed4ddefe2e3efa`、
16,240行の `runner.rs` hash は
`01990093ec8ac5b2360bf174e8b1d13b21550f599c3b51ab3fd0e02725762bd9`。
Rust は child の `pub(super)` item を grandparent へ re-export できないため、
leaf function は runner subtree 限定の明示的な `pub(in crate::runner)` scope を使い、private
phase facade は `pub(super)` で re-export する。private `runner.rs` alias により
全 production/test caller を維持する。function body、traversal/filter order、
recovery recursion、typed-site identity、exact-shape rejection、fail-closed behavior
は不変。Task 262A は完了し、parent Task 262 は残る bounded source-extraction
family のため open のまま。

## Task 262B Move Result

Task 262A 後の fresh inventory により、formula/reserve extractor が共有する
AST-only projection がさらに2個確認された。Task 262B は private recursive helper
を含む preorder node-kind collector と exact qualified-symbol spelling projection を
`source_ast.rs` へ移動した。元 fragment hash は
`e06bf8e9c5252a3bfefea3ff16804414fe813cbc903cd0afcdfd0d237a1185c5`
と
`8ab94eafe97e9b28c7a236efd3071834b36ba02a2fce1988d721123f19272f7e`。
結果の113行の `source_ast.rs` hash は
`d9bff4c1c4bbeb2bd988502db2bff2a7370dbf9b61d7e817e6e82077878da78a`、
6行の facade hash は
`f89edc2b2dcd2065c9445aae9a7b05084750d7689f8a2029c4e78050a7c797c9`、
16,193行の `runner.rs` hash は
`fea5c857a73a6f7429c2517b187b354fa321c0e6be14ffd64922eb10d57c42d0`。
caller-facing function 2個は parent-only facade re-export を介した runner
subtree 限定 `pub(in crate::runner)` visibility を維持し、recursive collector は
`source_ast.rs` private のまま。traversal order、token/path validation、spelling
assembly、全 caller、全 test import は不変。visible-symbol resolution、source-text
assembly、range merge、reserve/mode extraction は Task 262C のため `runner.rs` に
保持する。Task 262B は完了し、parent Task 262 は open のまま。

## Task 262C Move Result

Task 262B 後の fresh inventory により、reserve type-expression/symbol-projection
family を declaration segmentation と mode expansion から分離した。Task 262C は
private `src/runner/type_elaboration/source_reserve.rs` leaf を作り、
`SourceTypeExpression` transport、exact builtin/symbol type-expression projection、
visible attribute/type-head resolution、local/imported-fixture admission、recursive
source-text assembly を移動した。元の8行 transport fragment hash は
`6b95aec82269efe807537832e551e0bac37480cb653ad02cd3492e7ccd304afe`、
元の266行 type-expression/resolution fragment hash は
`918d2e22b0c18555cc0bffe1c2721f1563bc22427e7902959e7b7dcb56328f0e`、
元の89行 provenance/source-text fragment hash は
`8b06c6b116d5f420a40a645a274516051052a56908a2974c3e25efa43af80e2a`。
direct dependency import を含む結果の370行の `source_reserve.rs` hash は
`16b9a05842b3db5c22468d9674526bd7efc6739572d933ebf57e6ba0b69e34fb`、
14行の facade hash は
`e768f927bbf7263a7930f2ae73dcc8787b4df29f019f9f81ed50ec799f5d1f9d`、
15,834行の `runner.rs` hash は
`0574cd3bbdbf4df09c02a2a9be07af07b9732c5dc1d5036feb9919641c3a6007`。
transport と4 field、extraction entry は parent-only facade re-export 経由で
runner-subtree-only visibility を維持する。visible resolver entry 2個は、leaf 外の
caller が既存 private preservation test だけなので、test-only facade/runner alias
を使う。保持した Task 262E declaration/mode caller がまだ consume する3 helper、
`source_reserve_symbol_head_kind`、`is_imported_fixture_reserve_attribute`、
`imported_fixture_reserve_attribute_spelling` も同じ temporary scope を使い、Task
262E で caller を移した時点で narrow しなければならない。他の type-head、
attribute、admission、source-text helper は leaf-private のまま。exact AST shape/
recovery rejection、local-before-imported ambiguity handling、symbol kind/provenance
admission、attribute polarity/order、spelling/range、fail-closed behavior は不変。
formula-only resolution、range merge、reserve declaration segmentation、local-mode
traversal/expansion は Task 262E と後続 bounded inventory のため `runner.rs` に保持する。
Task 262C は完了し、parent Task 262 は open のまま。

## Task 262D Move Result

fresh dependency inventory により、exact `parser.type_fixtures` import-item
recognizer が formula caller 2個と保持した reserve caller で共有されることを確認した。
Task 262D は reserve family より先に、この common source-AST prerequisite を
`src/runner/type_elaboration/source_ast.rs` へ移動した。元の28行 fragment hash は
`d137915a4bac8d6922ea86d34975b07004b4cef389a5ea9d008fb955f3f83bdc`。
結果の147行の `source_ast.rs` hash は
`baf131e5f82846df2286ad68c6e8bad9d2642af2ce530f7b8c7362900ef2aa9e`、
15行の facade hash は
`10db1015db9d0a653f511ffaa5a48a2a708b9c1b3d254a194894f44430ff384f`、
15,803行の `runner.rs` hash は
`4dfc36f6f8f204b705688c5762d42281be949ce7c7eae2751e12d1aeb84c13d6`。
不変の370行の `source_reserve.rs` hash は
`16b9a05842b3db5c22468d9674526bd7efc6739572d933ebf57e6ba0b69e34fb`。
`source_ast` は fixture owner に runner-subtree visibility で残す
`import_fixtures::module_path_spelling` を直接 consume し、child-to-parent runner
helper dependency は残らない。recognizer は runner-subtree-only
`pub(in crate::runner)` visibility、parent-only facade re-export、3 caller 用 private
`runner.rs` alias を使う。この visibility と rustfmt signature wrapping 以外は本体を
変更していない。ImportItem/ImportAliasDecl/path cardinality、direct-token filtering、
recovery rejection、exact module spelling、caller order、fail-closed behavior は不変。
Task 262D は完了し、parent Task 262 は open、次は Task 262E。

## Task 262E Pre-Move Inventory and Specification

fresh inventory では、`runner.rs` に残る reserve family の配置を
`design_drift` と分類する。exact source-derived declaration/local-mode
payload producer は既存 private owner
`type_elaboration/source_reserve.rs` と cohesive であり、現在の parent 配置は
その ownership を表現していない。language、test、expectation、trace、metadata
間の不一致は見つからなかった。移動対象は `SourceReserveExtraction` から
`extract_builtin_reserve_segment` までの連続 1,074 行で、hash は
`31f8e27a1835ea31e6d65ff67acbfa8fcc040fc588df7f24453ff848e0bd690b`、
さらに独立した 10 行の `merge_optional_range` helper で、hash は
`aa186a9105816e62352473111ffe3b9958a332086e9d1fc459c024fbc2cfac5c`。

move では runner-subtree transport boundary を1つ維持する。
`SourceReserveExtraction` と `bridge`/`mode_expansions` field、および既存の
test-only accessor である。加えて runner-subtree helper boundary を4つ維持する:
`extract_builtin_source_reserve_declarations`、
`extract_builtin_source_reserve_declarations_after_node_guard`、
`source_mode_symbol_spelling`、`mode_definition_pattern_spelling`。private phase
facade はこの5 boundary item だけを parent へ re-export し、`runner.rs` は
unchanged caller 用の private alias を保持する。それ以外の moved item は
leaf-private のまま、または leaf-private になる。残る caller の移動後、
Task 262C の temporary helper
`source_reserve_symbol_head_kind`、
`is_imported_fixture_reserve_attribute`、
`imported_fixture_reserve_attribute_spelling` は leaf-private にする。

`source_reserve` は Task 262D fixture-import recognizer を含む common
source-AST projection を直接 consume し、child-to-parent dependency なしで
自身の Task 262C type-expression/symbol projection を引き続き consume する。
`SourceReserveHandoff`、`source_module_binding_env`、formula-only imported
term/formula resolution、checker handoff/validation、後続 orchestration は
`runner.rs` に保持する。preservation matrix は exact AST/import shape、node
allowlist、recovery rejection、traversal budget/order、dependency/provenance
admission、attribute polarity/order、spelling/range、payload contents、
diagnostic/detail key、fail-closed behavior である。この task は test body/name、
public API、spec/trace/expectation artifact、harness count、specification-coverage
credit を変更しない。

## Task 262E Move Result

Task 262E では inventory 済みの両 fragment を既存 private owner
`src/runner/type_elaboration/source_reserve.rs` へ移動した。rustfmt と最小の
import/visibility adjustment 後、`runner.rs` は 14,718 行、hash は
`f38352151d71474b676fb3c2a50e313c33f6de6dad5a09097c28aa9de729ce62`、
phase facade は 16 行、hash は
`07c19a11381d002cd3a6503470df6e1e63d09a2b435350608b1cc8fe1724a50a`、
`source_reserve.rs` は 1,474 行、hash は
`88cf0cf08de2e61b2e6342aacc36ee01e20e00606d0c51f4bf7b5c64495253db`。
unchanged の `source_ast.rs` は 147 行、hash
`baf131e5f82846df2286ad68c6e8bad9d2642af2ce530f7b8c7362900ef2aa9e`
を維持する。

extraction transport、その2 field/test-only accessor、2 extraction entry、
2 spelling helper は runner-subtree visibility、parent-only facade re-export、
private `runner.rs` alias を使う。Task 262C の temporary helper 3個は
leaf-private になった。`source_reserve` は common source-AST primitive と
fixture-import recognizer を直接 consume し、child-to-parent source dependency
は導入していない。`SourceReserveHandoff`、module binding environment、
formula-only imported symbol resolver、checker handoff/validation、orchestration
は `runner.rs` に保持する。

moved declaration/import gate、node allowlist、recovery check、traversal budget、
dependency order、expansion provenance、segment/range assembly、fail-closed branch
は、必要な visibility と rustfmt wrapping 以外は不変。unit test 272件は全成功し、
sorted raw/normalized test-list hash は
`5e41e4dbfcc303322c246a612de61926a628957a168589b45864d0a5070bb07e` と
`c0c2b80f8b4e6c84cd25d77573fda722c4d1846fed168cd4a478781cdb42775e`
を維持する。Task 262E は完了。parent Task 262 は残る formula-extraction family
の fresh inventory のため open のまま。behavior、test、trace credit、owner crate
が不変なので `spec_coverage_audit.md` 更新は不要。

## Task 262F Pre-Move Inventory and Specification

fresh inventory では、最初に残る formula family の配置を同じ
`design_drift` と分類する。exact standalone formula-constant AST projection は
cohesive private source-extraction leaf だが、その transport、extraction、node
policy は `runner.rs` 内で分離している。Task 262F は private
`src/runner/type_elaboration/source_formula.rs` を作り、3 fragment だけを移動する:
6 行の `SourceFormulaStatement` transport、hash
`8ab3f277e5a8e0dabe1caacf76e5f54d81804c3619209bf94ac88ed01ebbc5e7`、
84 行の `thesis`/`contradiction` entry と common exact extractor family、hash
`eb1927127ca995ad3e9f090cb04aaf2b0326aac240b58dcbc14cfb731666061c`、
12 行の dedicated theorem-node allowlist、hash
`acc01a4adb0ee02529a2fce8d8f0772c944f1b606f108bbde4e4096cc143c840`。

transport とその2 field は runner-subtree-only visibility を使い、caller は
type alias なしで inferred return payload を consume する。2 extraction entry は
runner-subtree-only visibility、parent-only facade re-export、private `runner.rs`
alias を使う。common exact extractor と node allowlist は leaf-private のまま。new leaf は既存 common
source-AST の token、structural-child、recovery、node-kind、site projection を
直接 consume する。real consumer である `source_formula_statement_output`、
`source_contradiction_formula_output`、それらの detail-key path、module binding
environment、checker inference は caller body 不変で `runner.rs` に保持する。

これは move-only task である。exact theorem label/token/cardinality、
formula-expression/constant kind と spelling、recovery/node rejection、real AST
site/range、`FormulaKind`、deferred reason、checker output、diagnostic/detail-key
ordering、fail-closed behavior を維持する。reserved-variable/binary、
builtin/imported formula、set-enumeration、connective/quantifier extraction、
formula semantics、child graph、theorem acceptance、fact、proof、CoreIr、
ControlFlowIr、VC は対象外。既存 exact、near-miss、corruption、active-fixture、
detail-key、route-isolation test を preservation matrix とし、
spec/test/trace/expectation と specification-coverage credit は変更しない。

## Task 262F Move Result

Task 262F は private
`src/runner/type_elaboration/source_formula.rs` leaf を作り、inventory 済みの
3 fragment をすべて移動した。rustfmt と最小 visibility/import adjustment 後、
`runner.rs` は 14,615 行、hash は
`b0d19f08a642b8b29e0f6c74e063b35909c3a9fbac30f9c1ee713de9fefa57f2`、
phase facade は 20 行、hash は
`59f458f5336f60be419c9d8e86b4a2dbed8f01dcc7ddc087cc437a25e72f3e7a`、
new leaf は 116 行、hash は
`d13b2ca47ad8c1580f38f363fac79881b304bcc5425e557ec7bdc6bd7a8264c2`。
unchanged の 147 行 `source_ast.rs` と 1,474 行 `source_reserve.rs` は hash
`baf131e5f82846df2286ad68c6e8bad9d2642af2ce530f7b8c7362900ef2aa9e` と
`88cf0cf08de2e61b2e6342aacc36ee01e20e00606d0c51f4bf7b5c64495253db`
を維持する。

transport と field は2 entry の return type を通じた runner-subtree visibility
だけを持つ。compiler-confirmed minimal facade/private alias surface は2 entry のみ。
common exact extractor と node allowlist は leaf-private で、leaf は
`source_ast` に直接依存する。両 checker consumer body、detail-key path、module
binding environment は `runner.rs` に保持した。必要な visibility、import、
rustfmt wrapping 以外、3 moved body は不変。

unit test 272件は全成功。sorted raw/normalized test-list hash は
`5e41e4dbfcc303322c246a612de61926a628957a168589b45864d0a5070bb07e` と
`c0c2b80f8b4e6c84cd25d77573fda722c4d1846fed168cd4a478781cdb42775e`
を維持する。Task 262F は完了し、parent Task 262 は残る reserved/binary、
builtin/imported、enumeration、connective/quantifier formula family の fresh
inventory のため open のまま。behavior、test、trace credit、owner crate が
不変なので `spec_coverage_audit.md` 更新は不要。

## Task 262G Pre-Move Inventory and Specification

fresh inventory では shared exact numeral projection を同じ `design_drift` と
分類する。その3つの AST-only helper は formula-source policy だが、まだ
`runner.rs` に残る。47行の contiguous fragment の hash は
`b415692ed2ee250be1bd4b66bfe90d21cc5cb444124eb249cca8890d1d488631`。
`exact_numeral_term_operand` は builtin binary、builtin type-assertion、imported
predicate/functor、imported attribute、set-enumeration extractor に7 retained
call site を持つ。`exact_numeral_term_node_or_expression` は imported infix-functor
projection に2 retained call site を持ち、common
`exact_numeral_term_node` recognizer は他の2 helper だけから呼ばれる。
connective/quantifier と standalone constant family はこの prerequisite を
consume しない。

Task 262G はその fragment だけを既存 private
`src/runner/type_elaboration/source_formula.rs` leaf へ移動する。operand と
node-or-expression entry は runner-subtree-only visibility、parent-only facade
re-export、private `runner.rs` alias を持ち、numeral-node recognizer は
leaf-private に保つ。leaf は `SurfaceAst`、`SurfaceNodeId`、`SurfaceNodeKind`、
`SourceRange` と、既存 common source-AST の token、structural-child、recovery
projection を引き続き直接 consume する。5 caller family、その transport/config、
resolver use、node allowlist、checker consumer、detail key、diagnostic、failure
assembly は body 不変で `runner.rs` に保持する。

これは move-only prerequisite である。exact `TermExpression` wrapper と
single-child cardinality、recovery rejection、`NumeralTerm` kind、各 caller が
要求する direct token spelling (`1`/`2`)、empty structural-child requirement、
returned real node identity/range、caller order、fail-closed behavior を維持する。
既存 spec-derived `.miz` source、trace/expectation row、exact/near-miss/corruption
matrix、active fixture、bidirectional route isolation を preservation matrix とする。
test-first 追加は不適切で、spec、test、trace、expectation、API、public surface、
`spec_coverage_audit.md` の変更は禁止する。formula-family move、helper の
rename/dedup/generalization、semantic payload change、theorem acceptance、fact、
proof、CoreIr、ControlFlowIr、VC は対象外。

## Task 262G Move Result

Task 262G は inventory 済み3-helper fragment を既存 private
`source_formula.rs` leaf へ移動した。必要な visibility qualifier を除くと、moved
47行は original hash
`b415692ed2ee250be1bd4b66bfe90d21cc5cb444124eb249cca8890d1d488631`
を維持する。`runner.rs` は 14,569 行、hash
`f3858539557d392e1d85fcf98bbfac615ef2564c1b3b9475c522994e7a6d94d4`、
phase facade は 21 行、hash
`702a81c671cc435d8dd1c1c4e1444070823372340308e319eeaf8790a0fcb8db`、
source-formula leaf は 165 行、hash
`ffbb81c4b76339f26c23423785e1139260d92426b6b56fc9295c0065635ab3f6`。
unchanged の 147 行 source-AST と 1,474 行 source-reserve leaf は hash
`baf131e5f82846df2286ad68c6e8bad9d2642af2ce530f7b8c7362900ef2aa9e` と
`88cf0cf08de2e61b2e6342aacc36ee01e20e00606d0c51f4bf7b5c64495253db`
を維持する。

exact numeral-node recognizer は leaf-private。operand と node-or-expression
entry だけが parent-only facade を横断し、7+2 retained caller site はすべて
original order で `runner.rs` に残る。caller body、transport/config、resolver
dependency、node allowlist、checker consumer、detail key、diagnostic、failure
assembly は不変。focused preservation と active type 188件は全成功。unit test
272件と relevant-crate suite は全成功し、sorted raw/normalized test-list hash は
`5e41e4dbfcc303322c246a612de61926a628957a168589b45864d0a5070bb07e` と
`c0c2b80f8b4e6c84cd25d77573fda722c4d1846fed168cd4a478781cdb42775e`
を維持する。

Task 262G は完了。parent Task 262 は残る reserved/binary、builtin/imported、
enumeration、connective/quantifier formula family の fresh bounded inventory の
ため open のまま。behavior、test、trace credit、owner crate が不変なので
`spec_coverage_audit.md` 更新は不要。

## Task 262H Pre-Move Inventory and Specification

fresh inventory では builtin equality/inequality/membership formula family を
同じ `design_drift` と分類する。3つの cohesive fragment が `runner.rs` 内で
分離している: 43行 config/source transport、hash
`cd7bf9a595ba8d6b73c1cafa567da306092c1953e92e9695c3bf67c5c653336d`、
84行 exact extractor、hash
`ce691c4917fc00c8b4fe0799f02f8e252e4cf005d3a3a1082ae01c8c0e35bc3c`、
17行 dedicated node allowlist、hash
`979560644d3d5827e2abbb016d2b5ea5da22a21cf71f3c35feca89404f3b29d8`。
3 config は `TermFormulaPayloadBoundary`、`BuiltinInequalityPayloadBoundary`、
`BuiltinMembershipPayloadBoundary` だけを exact operator、numeral spelling、
`FormulaKind` へ対応付ける。production extractor の caller は
`source_builtin_binary_term_formula_detail_keys` 1個。private preservation matrix
も config constant とその label、left、operator、right field を読み、
status-prefixed near miss を reject する。

Task 262H はこの3 fragment だけを既存 private `source_formula.rs` leaf へ移動する。
source transport と全 field、extraction entry、config type、test が consume する
4 config field、config constant は runner-subtree-only visibility を持つ。
config の `formula_kind` field、extractor implementation、node allowlist は可能な
限り leaf-private。facade は extraction entry を unconditional に expose し、config
constant は facade re-export と `runner.rs` import の両方に `#[cfg(test)]` を使う。
inferred transport/config type に parent alias は不要。leaf は `FormulaKind` を
直接 import し、既存 local exact-numeral helper と common source-AST の token、
structural-child、recovery、node-kind、site projection を consume する。
production checker/detail consumer と private test code は `runner.rs` とその test
subtree に byte-for-byte 保持する。

Task 262H は move-only task のまま。3-entry config order/value、exact theorem
label/token/cardinality、single formula-expression/two-term shape、operator/numeral
spelling、recovery/node rejection、real AST site/range、formula kind、checker
payload/detail-key ordering、fail-closed behavior を維持する。canonical `.miz`
source とその spec/trace/expectation intent、Task 262H0 で strengthen した
exact/near-miss/corruption matrix、active case 3件、route isolation を
preservation matrix とする。Task 262H 自体は test を追加せず、spec、trace、
expectation、public API、`spec_coverage_audit.md` を変更しない。config edit、
rename/dedup/generalization、reserved-variable formula、builtin type assertion、
imported/enumeration/connective formula、checker/detail move、semantic checking、
fact、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は対象外。

## Task 262H0 Test-Gap Inventory and Specification

Task 262H の test-sufficiency review は production move 前に独立 `test_gap` を
分類した。既存 test は3 active source が期待する fail-closed detail key へ到達し、
status、wrong label、wrong operator、wrong right numeral、extra root を reject
することを証明する。一方、extracted formula/left/right site/range、各 config が
生む `FormulaKind`、3-entry config order を直接 assert しない。また wrong left
numeral、recovered theorem/formula、duplicate theorem、duplicate formula
expression、extra operand cardinality rejection が欠ける。そのため、move が
source provenance/config mapping を壊しても同じ final detail key を維持できる。

Task 262H0 は Task 262H に先行する test-only repair である。new test を追加せず、
既存 `source_reserve_bridge_reports_gap_or_evidence_detail_for_unsupported_shapes`
を拡張する。3 config を canonical order で列挙し、exact label/left/operator/right
value を assert、各 exact payload を extract して `FormulaKind`、formula/operand
`TypedSiteRef`、hard source-derived range を比較する。recovered label/operator
token、duplicate theorem、duplicate formula expression、extra term expression の
bounded synthetic builder state と、各 config の wrong-left case を加え、すべて
既存 payload-extraction boundary で fail させる。test support はこの matrix に
必要な最小 corruption flag/builder だけを追加してよい。

Task 262H0 では production source、`.miz`、expectation、trace、spec、public API、
diagnostic、payload behavior、test name、test count を変更しない。既存 behavior
が test subject であり新 intent ではない。assertion は strengthen だけを許す。
focused test、unit 272件と unchanged list hash、active type 188件、relevant-crate
test、workspace fmt/Clippy/test、diff check を必須とする。この test-only task は
move-only Task 262H より前に別 commit とする。coverage credit、owner crate、
authority artifact が不変なので `spec_coverage_audit.md` 更新は不要。

## Task 262H0 Test Repair Result

Task 262H0 は test の追加・rename なしで既存 test を strengthen した。canonical
config loop は3 entry 全件を order、exact label/left/operator/right value、resulting
`FormulaKind`、独立計算した formula/operand range と対応する real AST site まで
固定する。各 config に wrong-left rejection も追加した。5つの bounded
corruption は recovered theorem label、recovered formula operator、duplicate
theorem、duplicate formula expression、extra operand cardinality を cover する。
synthetic duplicate/extra node は独立 ownership の allowlisted node kind を使い、
early node-policy guard ではなく意図した extractor cardinality branch へ到達する。

default/status-bearing builder は従来の exact token kind/text、child order、range、
offset、root construction を維持する。production extractor の import は private
test subtree だけ。`support.rs` は 6,655 行、hash
`5db1b0dc66f8149050d04f3f487c7e9efb201b990e871e8766cafbfca77b7d97`、
`source_gap_and_equality.rs` は 3,067 行、hash
`0178a217c935d42d4f229a30e3875989ac1aa9ae6bcd56057e931b7b05d7660a`。
production source、`.miz`、spec、trace、expectation、public API、diagnostic、
payload behavior、test name、test count は不変。

focused test と unit test 272件、active type 188件は全成功し、raw/normalized
test-list hash は
`5e41e4dbfcc303322c246a612de61926a628957a168589b45864d0a5070bb07e` と
`c0c2b80f8b4e6c84cd25d77573fda722c4d1846fed168cd4a478781cdb42775e`
を維持する。Task 262H0 は完了し、Task 262H で後続 production move も完了した。
test intent、coverage credit、owner crate が不変なので
`spec_coverage_audit.md` 更新は不要。

## Task 262H Move Result

Task 262H は inventory 済み builtin-binary 3 fragment を既存 private
`source_formula.rs` leaf へ移動した。review normalization により、config/transport、
extractor、allowlist body は必要な runner-subtree visibility と rustfmt whitespace
だけを除けば HEAD と同一。3 config entry は exact order、label、operator、
numeral spelling、`FormulaKind` value を維持する。

`runner.rs` は 14,430 行、hash
`c0f358ac368f31c560f204df8e89e8885144366c9871f288a0306fa84e2ae981`、
24行 phase facade は hash
`d3b9de31b1bf6c2b68d4bafd088c7b88addab6db083a5b5adff93e581f1981d4`、
310行 source-formula leaf は hash
`32978c9783b913439e9f8e94d326789c13aefff4d5e8326c669cb1a7d9745d6c`。
unchanged の147行 source-AST と1,474行 source-reserve leaf は hash
`baf131e5f82846df2286ad68c6e8bad9d2642af2ce530f7b8c7362900ef2aa9e` と
`88cf0cf08de2e61b2e6342aacc36ee01e20e00606d0c51f4bf7b5c64495253db`
を維持する。

extraction entry は private facade を unconditional に横断する。config constant
は `#[cfg(test)]` のときだけ横断し、config type と test-consumed field 4個は
runner-subtree visibility、`formula_kind` と node allowlist は leaf-private。
inferred transport/field は facade type alias なしで runner-subtree-visible。
production detail/checker caller と H0 test/support code は byte-for-byte 不変。
unit test 272件と relevant-crate test、active type 188件は全成功し、
raw/normalized test-list hash は
`5e41e4dbfcc303322c246a612de61926a628957a168589b45864d0a5070bb07e` と
`c0c2b80f8b4e6c84cd25d77573fda722c4d1846fed168cd4a478781cdb42775e`
を維持する。

Task 262H は完了。parent Task 262 は reserved-variable/binary、builtin
type-assertion、imported、set-enumeration、connective/quantifier formula family の
fresh bounded inventory のため open のまま。behavior、authority、coverage
credit、owner crate が不変なので `spec_coverage_audit.md` 更新は不要。

## Task 262I Pre-Move Inventory and Specification

clean HEAD `628b3272` の fresh inventory により、builtin type-assertion formula
family を `runner.rs` の連続する3 fragment として分離した。

- lines 1,649-1,656 の8行 `SourceBuiltinTypeAssertionFormula` transport、hash
  `88bc334c400dd92327d5fdc25e90efef1560cc097f5f2ecd6a5a822883082da4`。
- lines 12,988-13,069 の82行
  `extract_source_builtin_type_assertion_formula` entry、hash
  `c4d2a3911147e0ceefdb6d4f0b767e19ea829cc66e8f52d67fb5c146e2b3540d`。
- lines 13,686-13,701 の16行 dedicated node allowlist、hash
  `1e7c125594df441e775eac25259e0dd5c3a1081896ac461a5c441fb53748a844`。

この family は Chapter 14.2.3 type-assertion shape と active
`fail_type_elaboration_builtin_type_assertion_formula_gap_001.miz` sidecar の
exact source-derived slice を実装する。trace/expectation は real numeral term、
type-assertion formula、builtin-set type payload の後、numeric-type/partial-formula
diagnostic で fail closed することを要求する。production checker/detail caller は
`runner.rs` に保持する。

Task 262I はこの3 fragment だけを既存 private `source_formula.rs` leaf へ移動する。
extractor は既存 lower-level `source_reserve.rs` leaf の `SourceTypeExpression` と
`extract_builtin_source_type_expression` を直接 consume する。これにより acyclic
`source_formula -> source_reserve` projection dependency を確立し、
`source_reserve` からの reverse dependency はない。extraction entry は phase
facade を unconditional に横断する。inferred transport と consume される6 field
は runner-subtree visibility、allowlist は leaf-private とし、config、test-only
re-export、facade type alias は不要。

Task 262I は move-only とする。theorem label/token、recovery policy、structural
cardinality、numeral spelling、asserted builtin-set constraint、payload site/range、
checker output、detail key、ordering、fail-closed behavior は不変。helper rename、
deduplication、generalization、semantic edit、test rewrite は行わない。exact active
source、trace、expectation、Task 262I0 matrix、route-isolation case、active type
188件、272-test list を preservation oracle とする。coverage credit、owner crate、
deferred status は不変なので `spec_coverage_audit.md` 更新は不要。

## Task 262I0 Test-Gap Inventory and Specification

Task 262I test-sufficiency review により、production move 前に修復すべき独立
`test_gap` が見つかった。既存 positive unit test は checker kind/status を確認し、
formula/subject site と asserted range を同じ extraction payload の値へ比較するが、
formula、subject、asserted-type の `TypedSiteRef` と source range を独立固定せず、
`payload.asserted_type_site` と payload-level builtin-set spelling/head/empty
attributes も直接確認していない。

negative matrix は wrong label、status prefix、wrong numeral、builtin `object`、
attributed `set`、extra reserve/root content を reject するが、exact theorem label
または `is` token の recovery、duplicate theorem/formula-expression、extra formula
child、negation/wrong direct formula token、extra assertion operand を検査しない。
これらは Task 262I が保持すべき exact recovery、singleton、token、two-operand
contract の branch である。

Task 262I0 は test-only repair とする。既存 shared synthetic builder に bounded
builtin type-assertion corruption shape を追加し、既存
`source_reserve_bridge_reports_gap_or_evidence_detail_for_unsupported_shapes` test を
strengthen する。fixed theorem label、numeral、`is`、`set` spelling から expected
source offset を独立導出し、formula/subject/asserted type の source/node kind と
exact site/range、payload spelling/head/attributes、checker handoff を assertion する。
各 recovery/duplicate/token/cardinality corruption は extraction なし、かつ既存
payload extraction-gap detail key のままでなければならない。

Task 262I0 は test を追加せず、production source、`.miz`、expectation、trace、
specification、public API、diagnostic、payload behavior、test name/count を変更しない。
move-only Task 262I より前の独立 commit とする。preservation coverage だけの修復
なので `spec_coverage_audit.md` は変更しない。

## Task 262I0 Test Repair Result

Task 262I0 は test を追加・rename せず既存 test を strengthen した。positive
matrix は exact label、subject、`is`、`set` spelling から formula、numeral-subject、
asserted-type range を導出し、node kind/range で各 expected site を独立選択し、
extraction payload の全 field を固定する。checker type-entry cardinality を2件に
固定し、asserted-type site owner の entry が厳密に1件、subject term entry が
subject site owner であることを要求し、checked formula と normalized asserted
type を独立 source expectation に anchor する。

bounded corruption builder は default-off の recovery、duplicate、token-shape、
structural-cardinality control だけを追加する。negative matrix は recovered exact
label、recovered `is`、duplicate theorem、duplicate formula expression、extra
formula child、negation、extra assertion operand を検査する。全 case は extraction
なしで、既存
`type_elaboration.external_dependency.ast_payload_extraction` detail key を維持する。

`support.rs` は6,765行、hash
`757e507c998c0a0acdc6334b3d9ea1c68a0dbe9b87bb0eb623fca93e49942b4b`、
`source_gap_and_equality.rs` は3,250行、hash
`ed70cdc2536d6f44362c56b303cedee4ac0c666809abc4c1189b283963ce4b90`。
production source、`.miz`、specification、trace、expectation、public API、
diagnostic、payload behavior、test name/count は不変。

focused test、relevant-crate test、unit test 272件、active type 188件、format、
all-target/all-feature Clippy、workspace test は全成功。plan/count 403/367、type
coverage 235/223、pass/fail 219/184、raw/normalized test-list hash と4 CLI hash は
不変。Task 262I0 は完了し、move-only Task 262I で後続 production move も完了した。
behavior、test intent、coverage credit、owner crate が不変なので
`spec_coverage_audit.md` 更新は不要。

## Task 262I Move Result

Task 262I は inventory 済み builtin type-assertion 3 fragment を既存 private
`source_formula.rs` leaf へ移動した。review normalization により transport、
extractor、allowlist body は必要な runner-subtree visibility だけを除けば HEAD
`1b113e8b` と byte-equivalent。strengthen 済み Task 262I0 test/support file と
production checker/detail consumer はすべて byte-for-byte 不変。

`runner.rs` は14,320行、hash
`7d347e8a932ec5a4115540a6e6822b0ee23a6e41e919300ec56c04e5511303e4`、
24行 phase facade は hash
`61b5b82055f4f726d3b5209e2e6b57a176d0acaac5fbef9e1614780460306270`、
423行 source-formula leaf は hash
`a055d6e2220961f5445bbf4b5394b2ffc72738160dbd228af399e267241ec43d`。
unchanged の147行 source-AST と1,474行 source-reserve leaf は hash
`baf131e5f82846df2286ad68c6e8bad9d2642af2ce530f7b8c7362900ef2aa9e` と
`88cf0cf08de2e61b2e6342aacc36ee01e20e00606d0c51f4bf7b5c64495253db`
を維持する。

extraction entry は private phase facade を unconditional に横断する。inferred
transport と runner が consume する6 field は facade type alias なしの
runner-subtree visibility、dedicated allowlist は leaf-private。
`source_formula` は既存 `source_reserve` の `SourceTypeExpression` と builtin
type-expression projection を直接 consume し、reverse import がないため dependency
は acyclic。checker payload construction、detail rendering、route ordering、top-level
orchestration は `runner.rs` に保持する。

focused preservation test、relevant-crate test、unit test 272件、active type 188件は
全成功。plan/count 403/367、type coverage 235/223、pass/fail 219/184、
raw/normalized test-list と4 CLI hash は不変。format、all-target/all-feature
Clippy、workspace test、diff cleanliness も成功。Task 262I は完了した。behavior、
authority、coverage credit、owner crate、deferred status を変更しないため
`spec_coverage_audit.md` は不変。parent Task 262 は残る formula family の fresh
bounded inventory のため open のまま。

## Tasks 262J1-J2 Pre-Move Inventory and Specification

clean HEAD `fdce5d8a` の fresh inventory により、imported predicate/functor
formula work を `runner.rs` の5 fragment に分離した。

- lines 1,648-1,662 の15行 transport、hash
  `474b345cfa983e95fcce895a08a56c89a51bd1d3b8cf542b0fbacb16c42fe76e`。
- lines 12,978-13,104 の127行 family extractor、hash
  `6b967aff4d407f448cd8fd72aac205e88824c327f0048bb325786ef9a73e8bd4`。
- lines 13,486-13,546 の61行 exact infix transport/helper、hash
  `9b6b8d4f5fd417f6654f4232448514a279f006309c1308219514024bee4421b2`。
- lines 13,593-13,615 の23行 dedicated allowlist、hash
  `2daf39d17bde7186fe4a7fff4ad7fe6270ffc7a71e6ec1bdb44dbc2ba03fdafa`。
- lines 13,706-13,748 の43行 shared imported symbol resolver/provenance pair、
  hash `fc4914d1c4a557f1401db035032c22e84430faf0ac9355b8d3a1cf3716761955`。

exact active sidecar は imported visibility/conflict、import-prelude、
parenthesized infix term surface、predicate application syntax に従う。real imported
`divides` predicate と `++` functor symbol、numeral site 3個、infix functor site
1個、predicate formula 1個を渡した後、missing numeric/signature/
predicate-signature payload と partial formula checking で fail closed する。

shared resolver はこの family の predicate/functor resolution と、保持する
imported-attribute extractor の計3 caller を持つ。Task 262J1 は resolver/provenance
pair だけを先に移動する。resolver entry は runner-subtree visibility と
unconditional parent-facade alias、provenance predicate は leaf-private とし、全
caller は不変。

J1 後の Task 262J2 は transport、exact family extractor、exact infix projection、
dedicated allowlist だけを移動する。extractor は facade を unconditional に横断し、
transport と12 field は facade type alias なしの runner-subtree visibility、infix
transport/helper と allowlist は leaf-private。checker/detail/orchestration caller と
imported-attribute family は `runner.rs` に保持する。両 task は move-only で、
rename、deduplication、generalization、accepted-shape/symbol-admission change、
operator metadata change、diagnostic/detail/order change、test、authority edit を禁止。
dependency は `source_formula -> source_ast` と既存 `source_formula -> source_reserve`
edgeにより acyclic のまま。

## Task 262J0 Test-Gap Inventory and Specification

J1/J2 test-sufficiency review により独立 `test_gap` を確認した。現 positive matrix
は extractor-returned site で checker term/formula を探索し、imported symbol は
module path だけを確認する。formula、outer numeral、infix term、両 infix operand の
site/range、12 transport field、exact symbol kind/spelling/module/contribution
provenance、checker ordering を独立固定しない。

既存 negative matrix は source near-miss 12件と symbol-env case 6件を持つが、
direct extractor assertion、bounded recovery/duplicate、predicate segment/head
cardinality、parenthesized/infix cardinality、imported-contribution provenance
corruption を持たない。Task 262J0 は test-only repair とし、既存
builder/environment support に default-off bounded corruption control を追加し、
既存 test を independently derived source expectation と exact checker handoff/order
で strengthen する。全 negative case は extraction なし、かつ既存 extraction-gap
detail key を維持しなければならない。

Task 262J0 は test を追加せず、production source、`.miz`、expectation、trace、
specification、public API、diagnostic、payload behavior、test name/count を変更しない。
move-only J1/J2 より前の独立 commit とする。coverage credit、owner crate、follow-up
ownership、deferred rationale は不変なので `spec_coverage_audit.md` は変更しない。

## Task 262J0 Test Repair Result

Task 262J0 は test の追加・rename なしで既存 imported predicate/functor test を
strengthen した。positive matrix は formula、outer numeral、infix term、infix の両
operand、predicate formula の site/range を source spelling から独立導出する。12
extraction transport field 全て、imported predicate/functor の exact kind、spelling、
module、contribution provenance、checker term order、checked formula/term site handoff
を固定した。

default-off bounded corruption builder は recovered label/functor、duplicate
theorem/formula、formula/segment/head cardinality、parenthesized/infix cardinality、
imported-contribution provenance を網羅する。既存 source near miss 12件、structural
corruption 11件、既存 symbol-environment case 6件、isolated local-contribution case
は全て direct extraction なし、かつ不変の
`type_elaboration.external_dependency.ast_payload_extraction` detail key となる。

`support.rs` は6,943行、hash
`68e90fa32900462fbeac2065209d183762d85e4e32ddbe16d261680d564eed98`、
`source_gap_and_equality.rs` は3,525行、hash
`69e2a9f82e83d95247f5ec1d88244b38a071db1a09bcae34ed4772401b35924d`。
production source、`.miz`、specification、trace、expectation、public API、diagnostic、
payload behavior、test name/count は不変。

focused test、relevant-crate test、unit test 272件、active type case 188件は成功。
plan/count は403/367、type coverage 235/223、pass/fail 219/184、raw/normalized
test-list と4 CLI hash は不変。format、all-target/all-feature Clippy、workspace test、
diff cleanliness も成功した。Task 262J0 は完了し、move-only Task 262J1 が次。
behavior、test intent、coverage credit、owner crate、deferred status は不変なので
`spec_coverage_audit.md` は変更しない。

## Task 262J1 Move Result

Task 262J1 は inventory 済み shared imported formula symbol resolver/provenance pair
だけを既存 private `source_formula.rs` leaf へ移動した。resolver entry に必要な
`pub(in crate::runner)` visibility を除いて正規化した43行 moved body は baseline
hash `fc4914d1c4a557f1401db035032c22e84430faf0ac9355b8d3a1cf3716761955`
を保持する。predicate、functor、imported-attribute の caller body と順序は不変。

resolver entry だけが unconditional parent-only alias で private phase facade を
横断し、provenance predicate は leaf-private のまま。`ContributionKind` と
`NamespacePath` は implementation と共に移動し、runner-owned `SymbolEnv`、
`SymbolKind`、`ResolverSymbolId` use は `runner.rs` に保持した。既存の
`source_formula -> source_ast` と `source_formula -> source_reserve` dependency は
reverse edge なしで acyclic のまま。

`runner.rs` は14,277行、hash
`8d4e3ec02e275e3a5e69f3599285270cc176496b52321af72e29e063ca10fade`、
25行 phase facade は hash
`a969e263beb6eee47cbd111ff3efc25ef71122af1e7c7a8ae32a63c5c75dbd25`、
467行 source-formula leaf は hash
`eb6ef963457cf16625e00b03fc81795ff89772e253f5c0b3a45a7c592e324bcf`。
test、authority artifact、checker/detail consumer、public API、diagnostic、payload、
ordering、fail-closed behavior は不変。

focused preservation test、relevant-crate test、unit test 272件、active type case
188件は成功。plan/count は403/367、type coverage 235/223、pass/fail 219/184、
raw/normalized test-list と4 CLI hash は不変。format、all-target/all-feature Clippy、
workspace test、diff cleanliness も成功した。Task 262J1 は完了し、move-only Task
262J2 が次。behavior、authority、coverage credit、owner crate、deferred status は
不変なので `spec_coverage_audit.md` は変更しない。

## Task 262J2 Move Result

Task 262J2 は inventory 済み imported predicate/functor fragment 4個だけを既存
private `source_formula.rs` leaf へ移動した。必要な runner-subtree visibility だけを
除いて正規化すると、15行 transport、127行 extractor、61行 exact infix
projection、23行 allowlist は baseline hash
`474b345cfa983e95fcce895a08a56c89a51bd1d3b8cf542b0fbacb16c42fe76e`、
`6b967aff4d407f448cd8fd72aac205e88824c327f0048bb325786ef9a73e8bd4`、
`9b6b8d4f5fd417f6654f4232448514a279f006309c1308219514024bee4421b2`、
`2daf39d17bde7186fe4a7fff4ad7fe6270ffc7a71e6ec1bdb44dbc2ba03fdafa`
を保持する。

extractor だけが private phase facade を unconditional に横断する。transport と12
field は facade type alias なしの runner-subtree visibility、exact infix
transport/helper と dedicated allowlist は leaf-private。checker/detail/orchestration
caller と imported-attribute family は `runner.rs` で不変、moved extractor は Task
262J1 の leaf-owned resolver と direct source-AST projection を再利用する。sole
external caller が leaf 内へ移動したため未使用となった
`exact_numeral_term_node_or_expression` facade/runner alias は削除したが、
implementation と visibility は不変。

`runner.rs` は14,047行、hash
`9e47a64eedd35ae7e66629bdfefdaa39a86389d5002925af3887a2b7282222d0`、
25行 phase facade は hash
`2fad12f17b75a9ec51e97132846fbe926abeeeffb9f8c32eb78df93d0eab1330`、
698行 source-formula leaf は hash
`a4d3fbe9708eade5d3b6ca3db965f3fd119aff8723c30d6ed6fbf9ccd982f049`。
test、authority artifact、public API、diagnostic、payload、ordering、accepted shape、
fail-closed behavior は不変。

focused preservation test、relevant-crate test、unit test 272件、active type case
188件は成功。plan/count は403/367、type coverage 235/223、pass/fail 219/184、
raw/normalized test-list と4 CLI hash は不変。format、all-target/all-feature Clippy、
workspace test、diff cleanliness も成功した。Task 262J2 は完了し、parent Task 262
は残る formula family の fresh bounded inventory のため open のまま。behavior、
authority、coverage credit、owner crate、deferred status は不変なので
`spec_coverage_audit.md` は変更しない。

## Tasks 262K0-K Pre-Move Inventory and Specification

clean HEAD `9625d0a1` の fresh inventory により、exact imported attribute
assertion family を `runner.rs` の3 fragment に分離した。

- lines 1,649-1,656 の8行 five-field transport、hash
  `f6b78fea06f451c61eac5286ea41b8f85e33bfa80d4b392cfd68d65e9117f5ca`。
- lines 12,963-13,103 の141行 exact `empty`/`non empty` two-entry/shared-shape
  extractor、hash
  `a7aa82e3b3a97cbdcf2b7506920bda40cf7d4ddeef2feb5a1124c5d7e3b93c05`。
- lines 13,388-13,408 の21行 dedicated node allowlist、hash
  `3f13f99cd6fe64cd8baddceefdeed904e4b118d2132c6cecd06a2fe7187f0e76`。

exact active bridge は positive `empty` と attribute-level `non empty` を区別し、
後者は formula-level negation ではない。両者は `parser.type_fixtures` を import し、
imported `empty` attribute を要求し、real source-derived numeral 1個と attribute-
assertion formula 1個を checker へ渡した後、missing numeric および
formula/attribute semantic payload で fail closed する。Chapter 14 と canonical
harness はこの polarity boundary を保持し、broader attribute semantics を deferred
のままにする。

Task 262K は K0 後にこの3 fragment だけを移動する。exact extractor entry 2個は
runner-subtree visibility と unconditional parent-facade alias、shared-shape extractor
と allowlist は leaf-private。transport と5 field は runner-subtree visibility とし、
retained checker helper が transport を引数型として明示するため、transport も
parent-only type re-export で facade を横断する。これにより consumer の変更・複製を
避ける。checker/detail/orchestration consumer は `runner.rs` に保持する。moved family
は leaf-owned imported-symbol resolver、exact numeral projection、source-AST projection
を再利用する。last external caller の移動後、
K は `exact_compilation_item_list`、`is_exact_parser_type_fixtures_import`、
`qualified_symbol_spelling`、`resolve_imported_fixture_term_formula_symbol` の obsolete
runner/facade alias と runner-only `SymbolKind` import だけも削除する。leaf
implementation/visibility は不変で、new reverse dependency はない。

## Task 262K0 Test-Gap Inventory and Specification

K test-sufficiency review により独立 `test_gap` を確認した。現 positive matrix 2個は
extractor-returned site で checker term/formula を探索し、imported provenance は
module path だけを確認する。formula/subject site/range、5 transport field、exact
`AttributeRef` polarity、symbol kind/spelling/module/contribution provenance、singleton
checker ordering と formula-to-subject handoff を独立固定しない。

既存 source/environment near miss は rendered gap detail だけを確認し、direct
extractor rejection、recovery、duplicate theorem/formula expression、formula/
assertion/attribute-chain/attribute-ref/qualified-symbol/numeral cardinality corruption、
duplicate/mismatched `non`、isolated imported-contribution-kind corruption を持たない。
Task 262K0 は test-only repair とし、既存 support に default-off family-specific
corruption control を追加し、両 variant の既存 test を independently derived source
expectation、exact provenance/order、direct `None`、不変の
`type_elaboration.external_dependency.ast_payload_extraction` detail key で strengthen
する。

Task 262K0 は test を追加せず、production source、`.miz`、expectation、trace、
specification、public API、diagnostic、payload behavior、test name/count を変更しない。
move-only K より前の独立 commit とする。coverage credit、owner crate、follow-up
ownership、deferred rationale は不変なので `spec_coverage_audit.md` は変更しない。

## Task 262K0 Test Repair Result

Task 262K0 は test の追加・rename なしで既存 imported attribute assertion variant
2個を strengthen した。各 positive matrix は exact source spelling から formula/
subject range を導出し、`IsAssertion`/`NumeralTerm` site を独立選択し、5 transport
field 全て、direct `AttributeRef` polarity（`[]` と `["non"]`）、exact imported
attribute kind/spelling/module/contribution provenance を固定する。singleton checker
term/formula order と formula-to-subject handoff はこの independent site に固定した。

default-off bounded builder は両 variant の recovered label/attribute symbol、duplicate
theorem/formula expression、formula/assertion/attribute-chain/attribute-ref/qualified-
symbol/numeral cardinality、unexpected/duplicate `non` を網羅する。既存 source/
environment near miss と corruption case 22件は全て direct extraction なし、かつ不変の
`type_elaboration.external_dependency.ast_payload_extraction` detail key となる。shared
boolean environment builder により、otherwise-identical ImportedSource control の
extraction 成功後に LocalSource contribution twin が拒否されることも固定した。

`support.rs` は7,146行、hash
`46340ae9aa4ac03b7e5e458a515814bea4db86de177625c97b57762d894a8025`、
`source_gap_and_equality.rs` は3,974行、hash
`101fb755532276a12ce2202f297c318ad77249eab9aa27ce2670fe59e08ab47c`。
production source、`.miz`、specification、trace、expectation、public API、diagnostic、
payload behavior、test name/count は不変。

focused test、relevant-crate test、unit test 272件、active type case 188件は成功。
plan/count は403/367、type coverage 235/223、pass/fail 219/184、raw/normalized
test-list と4 CLI hash は不変。format、all-target/all-feature Clippy、workspace test、
diff cleanliness も成功した。Task 262K0 は完了し、move-only Task 262K が次。
behavior、test intent、coverage credit、owner crate、deferred status は不変なので
`spec_coverage_audit.md` は変更しない。

## Task 262K Move Result

Task 262K は inventory 済み imported attribute assertion fragment 3個だけを既存
private `source_formula.rs` leaf へ移動した。必要な runner-subtree visibility だけを
除いて正規化すると、8行 transport、141行 two-entry/shared extractor、21行
allowlist は baseline hash
`f6b78fea06f451c61eac5286ea41b8f85e33bfa80d4b392cfd68d65e9117f5ca`、
`a7aa82e3b3a97cbdcf2b7506920bda40cf7d4ddeef2feb5a1124c5d7e3b93c05`、
`3f13f99cd6fe64cd8baddceefdeed904e4b118d2132c6cecd06a2fe7187f0e76`
を保持する。

exact extractor entry 2個は private phase facade を unconditional に横断し、shared
shape extractor と allowlist は leaf-private。transport と5 field は runner-subtree
visibility、transport は unchanged checker helper が引数型として明示するため1個の
parent-only type re-export を持つ。checker/detail/orchestration consumer と順序は
`runner.rs` で byte-identical のまま。

moved family は leaf-owned imported-symbol resolver、exact numeral projection、
source-AST projection を直接再利用する。final external caller の移動後、
`exact_compilation_item_list`、`is_exact_parser_type_fixtures_import`、
`qualified_symbol_spelling`、`resolve_imported_fixture_term_formula_symbol` の obsolete
facade/runner alias と runner の unused `SymbolKind` import だけを削除した。leaf
implementation/visibility は不変で、dependency は acyclic のまま。

`runner.rs` は13,874行、hash
`d03812923d461dc718cb4236ee5568dfa03ac07e3bfb0f5995627d46f345b2c6`、
26行 phase facade は hash
`8e5b39254a2ca468d62db55d3ba7a69bdfaea5248881d5a5c62ca8d3eed526dd`、
871行 source-formula leaf は hash
`f1a6888ca7c10bfbf1a8a868261e34d31fa74003512250cdbe5b117e018f19de`。
test、authority artifact、public API、diagnostic、payload、polarity、ordering、
accepted shape、fail-closed behavior は不変。

focused preservation test、relevant-crate test、unit test 272件、active type case
188件は成功。plan/count は403/367、type coverage 235/223、pass/fail 219/184、
raw/normalized test-list と4 CLI hash は不変。format、all-target/all-feature Clippy、
workspace test、diff cleanliness も成功した。Task 262K は完了し、parent Task 262 は
残る formula family の fresh bounded inventory のため open のまま。behavior、
authority、coverage credit、owner crate、deferred status は不変なので
`spec_coverage_audit.md` は変更しない。

## Tasks 262L0-L Pre-Move Inventory and Specification

clean HEAD `be7a2c6e` の fresh inventory により、exact set-enumeration formula
family を `runner.rs` の4 fragment に分離した。

- lines 1,649-1,659 の11行 eight-field transport、hash
  `5aa3f3e859cc0313f935e80011ef7be4e05299a0763f97de572eccc500fd71c8`。
- lines 12,954-13,010 の57行 exact extractor、hash
  `f05ab26f14f3d28e2f721575ca7a53c74fae9dfeebb0779906fd0a6d45b7fc99`。
- lines 13,148-13,190 の43行 private exact-set transport/projection、hash
  `45c155d6556740807b395b0e1a8114094db074ac6768ee7d892b7e0eb2d26036`。
- lines 13,237-13,251 の15行 dedicated node allowlist、hash
  `461650cdedc2f56cdf072e95e1ef0243bc7be1a3c7323e0628c652ad562b6dd1`。

exact active bridge は
`SetEnumerationPayloadBoundary: {1, 2} = {1, 2}` だけを認識し、real source AST
から numeral item 4個、set-enumeration term 2個、equality formula 1個を投影した後、
missing numeric type payload、missing set result-type payload、partial formula checking
で fail closed する。canonical Chapter 13、exact `.miz`、trace row、expectation、
harness は一致し、broader set-enumeration extraction/semantics は deferred のまま。

Task 262L は L0 後にこの4 fragment だけを移動する。eight-field transport と全 field
は runner-subtree visibility、exact extraction entry だけは unconditional parent-facade
alias とする。exact-set transport、projection helper、allowlist は leaf-private。
checker/detail consumer `source_set_enumeration_formula_output` は `runner.rs` に
byte-identical で保持する。この consumer は transport type を名指ししないため、
facade type re-export は不要。moved family は leaf-owned exact numeral/source-AST
projection を直接再利用し、reverse dependency を導入しない。last external exact-
numeral caller の移動後、L は obsolete `exact_numeral_term_operand` facade alias と
runner import だけを削除し、leaf implementation/visibility は変更しない。

## Task 262L0 Test-Gap Inventory and Specification

L test-sufficiency review により独立 `test_gap` を確認した。positive matrix は8
transport field を独立固定する一方、left/right item vector を結合し、checker term
6個を unordered search で探し、formula handoff を extractor-returned site と比較する。
そのため 2+2 item grouping、両方の exact punctuation triple、deterministic checker
six-term output order と対応する exact term-kind order、formula の independently
derived site と `[left_set, right_set]` handoff を独立固定しない。

既存 near-miss matrix は rendered extraction-gap detail だけを確認し、extractor を
direct call せず、left item mismatch 2個を結合する。isolated four-position numeral
near miss と、formula-expression/formula/operand、term-wrapper/set/item、punctuation、
numeral-child の allowlisted kind/cardinality corruption がない。Task 262L0 は
test-only repair とし、既存 support に default-off family-specific corruption control
を追加し、既存 test を independent grouping/punctuation/order expectation、direct
`None`、不変の `type_elaboration.external_dependency.ast_payload_extraction` detail key
で strengthen する。

Task 262L0 は test を追加せず、production source、`.miz`、expectation、trace、
specification、public API、diagnostic、payload behavior、test name/count を変更しない。
move-only L より前の独立 commit とする。coverage credit、owner crate、follow-up
ownership、deferred rationale は不変なので `spec_coverage_audit.md` は変更しない。

## Task 262L0 Test Repair Result

Task 262L0 は test の追加・rename なしで既存 exact set-enumeration matrix を
strengthen した。positive path は separate 2+2 item group と両 punctuation triple を
固定し、deterministic six-site/six-kind checker output、equality formula site と ordered
set term を independently derived source site に固定する。8 transport field は全て
independent に固定したまま。

item-spelling near miss 4件は left-first、left-second、right-first、right-second を
個別に isolate する。全既存 source near miss は不変の gap detail を確認する前に
extractor を direct call する。default-off/allowlisted corruption variant 11件は formula-
expression cardinality/kind、formula child/kind/operand cardinality、term-wrapper
kind/cardinality、set kind/punctuation/item cardinality、numeral-child cardinality を
独立に網羅し、各 case は direct extraction なし、かつ不変の
`type_elaboration.external_dependency.ast_payload_extraction` detail key となる。

`support.rs` は7,330行、hash
`451611d56191b98685fc27fd9a87eec36090f7b1dba11aa3a7a7f8e8d9e801e6`、
`source_gap_and_equality.rs` は4,079行、hash
`e1836ed29e9b6593970047b5e68f746def70cbd86f9fd98b11aad7841459afb7`。
production source、`.miz`、specification、trace、expectation、public API、diagnostic、
payload behavior、test name/count は不変。

focused test、relevant-crate test、unit test 272件、active type case 188件は成功。
plan/count は403/367、type coverage 235/223、pass/fail 219/184、raw/normalized
test-list と4 CLI hash は不変。format、all-target/all-feature Clippy、workspace test、
diff cleanliness も成功した。Task 262L0 は完了し、move-only Task 262L が次。
behavior、test intent、coverage credit、owner crate、deferred status は不変なので
`spec_coverage_audit.md` は変更しない。

## Task 262L Move Result

Task 262L は inventory 済み set-enumeration fragment 4個だけを既存 private
`source_formula.rs` leaf へ移動した。required runner-subtree visibility と wrapped
public extraction signature だけを normalize すると、11行 transport、57行 extractor、
43行 exact-set projection、15行 allowlist は baseline hash
`5aa3f3e859cc0313f935e80011ef7be4e05299a0763f97de572eccc500fd71c8`、
`f05ab26f14f3d28e2f721575ca7a53c74fae9dfeebb0779906fd0a6d45b7fc99`、
`45c155d6556740807b395b0e1a8114094db074ac6768ee7d892b7e0eb2d26036`、
`461650cdedc2f56cdf072e95e1ef0243bc7be1a3c7323e0628c652ad562b6dd1`
をそれぞれ維持する。

transport と8 field は runner-subtree visibility、exact extractor だけが phase facade
を越える。exact-set transport/helper/allowlist は leaf-private のままで、transport
type alias は追加しない。保持した `source_set_enumeration_formula_output` checker/
detail consumer は hash
`710f25b9f406aad51eeb99c105abd79f9477e0c18b60ea3f27124a1b81330355`
で HEAD と byte-identical。final external caller の移動後、obsolete
`exact_numeral_term_operand` facade alias/runner import だけを削除し、leaf
implementation/visibility/body は不変。

`runner.rs` は13,744行、hash
`2fa77cd1126d591f37c13e2e7c0fb2522a3e9a269ecb81dbb26f86ffcd93f234`、
25行 phase facade は hash
`8aca34293b02fad31567ec4b3d2865e8c8fac95c333d060718885d462c19b8af`、
1,003行 source-formula leaf は hash
`4bbe60d38ca7af3a320ab97c8b4f6e2aa61abd50dc41c68c6431e0fb7684af01`。
test、authority artifact、public API、diagnostic、payload、ordering、accepted shape、
fail-closed behavior は不変。

focused preservation test、relevant-crate test、unit test 272件、active type case 188件は
成功。plan/count は403/367、type coverage 235/223、pass/fail 219/184、raw/
normalized test-list と4 CLI hash は不変。format、all-target/all-feature Clippy、
workspace test、diff cleanliness も成功した。Task 262L は完了し、parent Task 262 は
残る formula family の fresh bounded inventory のため open のまま。behavior、
authority、coverage credit、owner crate、deferred status は不変なので
`spec_coverage_audit.md` は変更しない。

## Tasks 262M0-M Pre-Move Inventory and Specification

clean HEAD `334b83e2` の fresh inventory により、exact formula connective/
quantifier family を `runner.rs` の3 fragment に分離した。

- lines 1,649-1,661 の13行 ten-field transport、hash
  `98f4a9a771cebc18faa43d1b266dd78f931f00a7d9435c7f6606cfd807a6e424`。
- lines 12,942-13,076 の135行 exact extractor、hash
  `a64080512c757a0a8f85357ec5a086285d7139257bd816bdbb85a5ae19bcd56f`。
- lines 13,123-13,140 の18行 dedicated node allowlist、hash
  `80c39e182da04f34e2598f0670fcba4c17785dbea34373ef7d7847e3488cce1f`。

exact active bridge は contradiction premise が universal `set`-typed binder を
含み、その body が negated contradiction である形だけを認識する。real source AST
から contradiction constant 2個と implication/quantified/negation shell を投影し、
missing formula/quantifier payload で fail closed する。canonical Chapter 14、exact
`.miz`、trace row、expectation、harness は一致し、broader connective、binder、child-
formula、theorem semantics は deferred のまま。

Task 262M は M0 後にこの3 fragment だけを移動する。ten-field transport と全 field
は runner-subtree visibility、exact extraction entry だけは unconditional parent-
facade alias とし、allowlist は leaf-private。checker/detail consumer
`source_formula_connective_quantifier_output` は `runner.rs` に byte-identical で保持する。
consumer は transport type を名指ししないため facade type re-export は不要。moved
extractor は leaf-owned source-AST projection と `source_reserve` builtin type-
expression projection を直接再利用し、dependency direction は acyclic のまま。move
後は production runner で unused となる `SurfaceFormulaConnective`、
`SurfaceFormulaConstant`、`SurfaceFormulaPrefixOperator`、`SurfaceQuantifierKind` import
4個だけを削除し、test-support import は独立して保持する。他 runner caller が必要な
`extract_builtin_source_type_expression`、`TypeHeadInput`、全 source-AST facade alias
は保持する。

## Task 262M0 Test-Gap Inventory and Specification

M test-sufficiency review により独立 `test_gap` を確認した。positive matrix は10
transport field と5 shell state を独立固定する一方、extractor-returned site による
unordered search で checker formula を探す。deterministic five-site/five-kind output
order と complete diagnostic provenance を固定せず、2 contradiction-constant formula
diagnostic だけを source anchor し、implication/quantified/negation diagnostic key/range
pair を確認しない。binder segment/type-expression/head shape と direct `x being` / `set`
token も independently assert しない。

既存 near-miss matrix は rendered extraction-gap detail だけを確認し、extractor を
direct call しない。formula-expression、implication/repetition/token/operand、premise
constant、universal token/child、binder segment/token/child、negation token/child、body
constant、recovered inner node の allowlisted corruption がない。attributed-set binder
は `AttributeChain`、`QualifiedSymbol`、`PathSegment` が current family allowlist を
必ず失敗するため、別の non-allowlisted near miss とする。M0 は production boundary
を広げず、後段 attributes-empty guard の isolate を主張しない。Task 262M0 は test-
only repair とし、既存 support に default-off family-specific allowlisted corruption
control と別の attributed-set near miss を追加し、既存 test を independent binder、
output-order/state、diagnostic key/range、direct `None`、不変の
`type_elaboration.external_dependency.ast_payload_extraction` detail-key assertion で
strengthen する。default-off corruption は family allowlist と preceding guard を保持し、
rejection を isolate する。

Task 262M0 は test を追加せず、production source、`.miz`、expectation、trace、
specification、public API、diagnostic、payload behavior、test name/count を変更しない。
move-only M より前の独立 commit とする。coverage credit、owner crate、follow-up
ownership、deferred rationale は不変なので `spec_coverage_audit.md` は変更しない。

## Task 262M0 Test Repair Result

Task 262M0 は test の追加・rename なしで既存 exact connective/quantifier matrix を
strengthen した。positive path は binder segment/type-expression/type-head range と
direct `x`、`being`、`set` token を固定する。10 transport field は全て independent
に anchor したまま。実測 deterministic checker order は body contradiction、
negation、quantified shell、implication、premise contradiction で、5 entry 全ての
exact site/kind/context/partial status/deferred reason を固定する。formula-payload 4件と
quantifier-payload 1件の diagnostic key/range pair は complete multiset として固定する。

全既存 connective/quantifier near miss は不変の detail key より先に direct extractor
rejection を確認する。attributed-set binder は production を広げず、後段 attributes-
empty guard の isolate を主張しない explicit non-allowlisted near miss とする。
default-off/allowlisted corruption 18件は theorem/formula-expression shape、implication
repetition/token/operand、premise kind/token、universal token/child、binder segment
kind/token/child、negation token/child、body kind/token、descendant recovery を独立に
網羅する。各 case は direct extraction なし、かつ不変の
`type_elaboration.external_dependency.ast_payload_extraction` detail key となる。

`support.rs` は7,551行、hash
`7315c2d22d5d0e7dbf27c2086e34f3177e6b1fba6c57f3e9db0cd51660081af0`、
`source_gap_and_equality.rs` は4,260行、hash
`dd39dcbaf71644d6e6a9d0035fb9d838925e6d2db0892b58009c53e495fe6369`。
production source、`.miz`、specification、trace、expectation、public API、diagnostic、
payload behavior、test name/count は不変。

focused test、relevant-crate test、unit test 272件、active type case 188件は成功。
plan/count は403/367、type coverage 235/223、pass/fail 219/184、raw/normalized
test-list と4 CLI hash は不変。format、all-target/all-feature Clippy、workspace test、
diff cleanliness も成功した。Task 262M0 は完了し、move-only Task 262M が次。
behavior、test intent、coverage credit、owner crate、deferred status は不変なので
`spec_coverage_audit.md` は変更しない。

## Task 262M Move Result

Task 262M は inventory 済み formula connective/quantifier fragment 3個だけを既存
private `source_formula.rs` leaf へ移動した。required runner-subtree visibility だけを
除去すると、13行 transport、135行 extractor、18行 allowlist は baseline hash
`98f4a9a771cebc18faa43d1b266dd78f931f00a7d9435c7f6606cfd807a6e424`、
`a64080512c757a0a8f85357ec5a086285d7139257bd816bdbb85a5ae19bcd56f`、
`80c39e182da04f34e2598f0670fcba4c17785dbea34373ef7d7847e3488cce1f`
を維持する。

transport と10 field は runner-subtree visibility、exact extractor だけが phase facade
を越え、transport type alias は追加せず、allowlist は leaf-private のまま。保持した
`source_formula_connective_quantifier_output` checker/detail consumer は hash
`7bc5d0899674fda17899b4c78463ac1d83e9ed8ad99196a4b0bb2eaf11f844f0`
で HEAD と byte-identical。production runner で unused となった syntax-enum import
4個だけを削除し、test support、`TypeHeadInput`、builtin type-expression extractor、
still-used source-AST facade alias は全て不変。dependency direction は acyclic。

`runner.rs` は13,573行、hash
`1ea8e97e9f87e92bbcdd5b9e17e8a1d829b46f34f14c1a53d983529ece9ce58f`、
26行 phase facade は hash
`1eb16a6815df883433ef6de6e7814cba7102e5962c8b5425ac875caba0c5fb69`、
1,173行 source-formula leaf は hash
`d418905106d5b6313fe62644c4145c83428c056880f2f9b2d74cc2eb2d00760d`。
test、authority artifact、public API、diagnostic、payload、ordering、accepted shape、
fail-closed behavior は不変。

focused preservation test、relevant-crate test、unit test 272件、active type case 188件は
成功。plan/count は403/367、type coverage 235/223、pass/fail 219/184、raw/
normalized test-list と4 CLI hash は不変。format、all-target/all-feature Clippy、
workspace test、diff cleanliness も成功した。Task 262M は完了し、parent Task 262 は
残る formula family の fresh bounded inventory のため open のまま。behavior、
authority、coverage credit、owner crate、deferred status は不変なので
`spec_coverage_audit.md` は変更しない。

## Tasks 262N0-262Q Reserved-Variable Formula Fresh Inventory

Task 262M 後の fresh inventory は、残る source-formula code を 1つの shared
reserved-variable source model/substrate と、その後の 3 bounded extraction owner
（direct binary、parenthesized binary、type assertion）に分類する。checker-output
transport、builder、validator、detail key、diagnostic は Task 263 のままにする。
concrete config と thin named route wrapper は、source extraction を Task 263 が
consume する result-role / invalid-detail-key contract に結合せず移動できる後続
inventory まで、paired のまま `runner.rs` に保持する。

shared source substrate は reserved-variable config/model の type definition/schema
（concrete config value/static ではない）、builtin/mode projection predicate、
asserted-head relation check、exact mode-definition/expansion check、shared identifier
operand projection、source-use ordinal calculation から成る。single-parenthesized
operand projection は Task 262P、3 family-specific node allowlist はそれぞれ Tasks
262O-262Q の generic core と一緒に保持する。`runner.rs:12974` までとした最初の
family 候補範囲は `source_module_binding_env` の開始を含むため review で却下され、
reserved-variable helper/allowlist region 全体の clean endpoint は 12969 行目であり、
各 task は割り当てられた subfragment だけを選ぶ。projection predicate を model type
definition より先に移すと runner-owned source configuration への ownership direction
が逆転する。したがって Task 262N はそれらの definition と shared substrate を一緒に
移し、保持 validator が必要とする alias だけを parent-only で公開する。concrete
config value/static と thin named wrapper は Task 263 contract boundary の後続 inventory
まで paired のまま `runner.rs` に保持する。

独立 test review はこの move 前に bounded `test_gap` を確認した。既存 exact equality
bridge は real checker handoff と多数の detail-key failure を証明するが、全 config
field、formula と 2 operand の site/range、既存 near-miss matrix に対する direct
extractor rejection、allowlisted expression/predicate/term kind/cardinality corruption
を独立固定していない。Task 262N0 は test-only とし、既存 equality test とその
default-off private AST builder にこれらの preservation assertion を追加する。test
追加、production source、`.miz`、expectation、trace、specification、public API、
behavior、coverage credit の変更は行わない。N0 の review/verification 後だけ Task
262N へ進む。続く Tasks 262O/262P/262Q は direct-binary、parenthesized-binary、
type-assertion source core をそれぞれ移し、262Q 前には fresh test-sufficiency review
を行う。

## Task 262N0 Test Repair Result

Task 262N0 は test の追加・rename なしで既存 exact reserved-variable equality test
を強化した。formula と 2 operand の site/range を AST から独立導出して固定し、全
binary config field、保持する 13 near miss の direct rejection、16 default-off
corruption rejection を固定した。corruption matrix は formula-expression/predicate
kind/cardinality、左右 term-expression/reference kind/cardinality、左右/operator
recovery を独立 cover する。default path は従来の node/token sequence、kind、range、
order、ID を維持する。

production source、`.miz`、expectation、trace、specification、API、diagnostic、
payload、ordering、coverage artifact は変更していない。focused exact test、unit test
272件、active type case 188件、relevant-crate test、workspace test は成功した。
plan/count 403/367、type coverage 235/223、pass/fail 219/184 は不変で、raw/normalized
test-list と 4 CLI hash も不変。format、all-target/all-feature Clippy、diff cleanliness
も成功した。Task 262N0 は完了し、次は move-only Task 262N。authority、behavior、
coverage credit、owner crate、deferred status は不変なので
`spec_coverage_audit.md` は変更しない。

## Task 262N Pre-Move Inventory and Specification

Task 262N は `runner.rs` から既存 private
`type_elaboration/source_formula.rs` leaf へ次の 4 source-substrate fragment だけを
移動する。

- reserved-variable config/model type definition/method 136行、hash
  `2c3ebcfe343f60ddae3bb2124f4f15f942c0f8236f54b42954ed4453766e2ac1`。
- builtin/mode projection predicate、asserted-head relation check、terminal-mode
  traversal 562行、hash
  `ffe1ae491ff3b7548171410a840e4ea6ea5edbdf69cee595b6c18b74e4612da6`。
- exact mode-definition/expansion check と shared direct identifier operand
  projection 115行、hash
  `eb5d150c267e2f7e3a1259ecb75b7e985caab81aba973be0b71ed15522d8cfcb`。
- source binding/use ordinal validation 50行、hash
  `4bdef09433003048b6b439f7dd2ee3bc154fa8c3cc63082aedae7a8bcb44b3a0`。

移動する 863行は、保持 concrete config、validator、後続 generic core が必要とする
最小 `pub(in crate::runner)` visibility 以外 byte-preserve する。
`source_mode_expansion_for_spelling` は leaf-private のまま。phase facade は移動した
model schema/helper のうち `runner.rs` が保持 consume するものだけを re-export する。
Task 262N は concrete config value/static、thin named wrapper、source transport、
generic extractor、single-parenthesized operand projection、family allowlist、checker/
output transport/body、detail key、diagnostic を移動・編集しない。rename、deduplication、
generalization、semantic cleanup を禁止する。

完了条件は visibility-only prefix を除いた normalized moved-fragment equivalence、
strengthened N0 test、全 direct/parenthesized/asserted-head/type-assertion/long-chain/
cross-owner isolation test、272-name raw/normalized test list 不変、active type 188件、
plan/count、coverage、pass/fail、4 CLI hash 不変、format、Clippy、relevant-crate/
workspace test、diff cleanliness、visibility/dependency direction/behavior/paired
source-doc consistency の no-findings review。

## Task 262N Move Result

Task 262N は inventory した 4 reserved-variable source-substrate fragment だけを
既存 private `source_formula.rs` leaf へ移動した。runner-scoped visibility qualifier
を除くと、model、predicate、mode/identifier、ordinal fragment の hash はそれぞれ
`2c3ebcfe343f60dd...`、`ffe1ae491ff3b754...`、`eb5d150c267e2f7...`、
`4bdef09433003048...` を維持する。review により `spelling` と `input_head` は
leaf-private に戻し、`source_mode_expansion_for_spelling` も leaf-private のまま。
他の runner-scoped type/field/method/helper/facade alias には保持 consumer がある。
formula leaf だけが使う reserve spelling projection 2個は production facade alias を
削除し、private test が引き続き使う 1 spelling helper は `#[cfg(test)]` のみ公開する。

concrete config value/static と thin named wrapper、source transport、generic direct/
parenthesized/type-assertion core、single-parenthesized operand projection、3 family
allowlist、checker/output transport/body、detail key、diagnostic は `runner.rs` に保持。
dependency direction は acyclic で、`source_formula` は sibling source-AST/source-
reserve leaf に依存し、保持 runner code は 35行 phase facade 経由だけで consume する。

結果の `runner.rs` は 12,717行、hash
`2a20df9e786bac81e30a60fdd1824b44fc87dbd38eeb20ba97bdeb3862a0a33a`、
35行 facade は hash
`65d8c6a8bbd1421f827888d9444502c41ae7f2e7e69c1eb15928ea34f347b2e2`、
2,044行 source-formula leaf は hash
`8fabf38e9dea88b7fc1387508ce21a6d29080659af1148fb694c2da74c8aae49`。
focused N0 test、unit test 272件、relevant-crate test、workspace test、active type
188件は成功した。plan/count 403/367、type coverage 235/223、pass/fail 219/184、
raw/normalized test-list と 4 CLI hash は不変。format、all-target/all-feature Clippy、
diff cleanliness も成功した。Task 262N は完了し、次は Task 262O。authority、behavior、
coverage credit、owner crate、deferred status は不変なので
`spec_coverage_audit.md` は変更しない。

## Task 262O Pre-Move Inventory and Specification

Task 262O は `runner.rs` から既存 private `source_formula.rs` leaf へ次の 3 direct
reserved-variable binary source-core fragment だけを移動する。

- source transport 16行、hash
  `d7c400d8c6c6d101c40159a3f76b910a27113a6f9092c4b6672ef4bd1e41a303`。
- generic direct-binary extractor 126行、hash
  `9f7e12badc208e4a7686bcabffb0da648748b9be7b672a2706f846690c42f4c3`。
- direct-binary node allowlist 19行、hash
  `8b6d0b2e43a4346121e3b571246210b16d487a635a618c5ff66eeefe05fb1a77`。

161行は最小 runner-scoped visibility だけを追加し normalized equivalence で移動する。
source transport/extractor は保持 named route wrapper と checker/output validator が
引き続き consume する。binary allowlist は保持 parenthesized allowlist が delegate
するため一時 parent-only alias を持ち、parenthesized core を移す Task 262P でその
alias を削除する。Task 262O は formula leaf に direct reserve-extraction dependency
を追加するが、concrete config/wrapper、parenthesized/type-assertion transport/core/
allowlist、single-parenthesized operand projection、checker/output transport/body、
detail key、diagnostic は移動しない。

Task 262N0 と既存 direct binary route/source/corruption/isolation matrix は十分であり、
新規 test は不要。完了条件は 3 normalized hash、最小 visibility、fail-closed shape/
order と payload provenance 維持、272-name list と active 188件不変、plan/count/
coverage/pass-fail/CLI hash 不変、全 Rust verification、diff cleanliness、
implementation/source-doc no-findings review。rename、deduplication、generalization、
semantic cleanup を禁止する。

## Task 262O Move Result

Task 262O は inventory した direct reserved-variable binary source-core fragment
3個だけを既存 private `source_formula.rs` leaf へ移動した。transport と extractor は
追加した runner-scoped visibility を除くと original hash を維持する。allowlist は同じ
visibility normalization と rustfmt による signature wrapping だけを戻すと original
hash を維持する。formula leaf は sibling reserve-extraction entry を直接 consume し、
phase facade は transport、extractor、および一時的に binary allowlist を保持 runner
consumer へ公開する。最後の alias は保持 parenthesized allowlist だけが使用し、Task
262P で削除しなければならない。

結果の `runner.rs` は 12,558行、hash
`25eff814585b074fc137f87f8da8172dadef3aa02b703bab1b35b5287156c920`、
38行 facade は hash
`5083cf8a6bcc49144c0f8f594b1a1a4d30007a1d4c2da840b8bda136c0d2dce4`、
2,209行 source-formula leaf は hash
`88132f00f4f925c9293142310660b495e688f6a1d65659e88ec1dcc51ea83c14`。
concrete config/wrapper、parenthesized/type-assertion transport/core/allowlist、
single-parenthesized operand projection、checker/output/detail/diagnostic code は
`runner.rs` に不変で残る。

unit test 272件と active type case 188件は全成功。plan/count は 403/367、type
coverage は 235/223、pass/fail は 219/184 のまま。raw/normalized test-list と4つの
CLI hash も不変。formatting、all-target/all-feature Clippy、workspace test、diff
cleanliness は成功した。Task 262O は完了し、次は Task 262P。authority、behavior、
coverage credit、owner crate、deferred status は不変なので
`spec_coverage_audit.md` は変更しない。

## Task 262P Pre-Move Inventory and Specification

Task 262P は `runner.rs` から既存 private `source_formula.rs` leaf へ次の4つの
parenthesized reserved-variable binary source-core fragment だけを移動する。

- source-side wrapper enum/transport 13行、hash
  `9574d330441d576284cfabaadcb9963efb1bf4ff441a1c88bff631a68706ab00`。
- generic parenthesized extractor 184行、hash
  `a252111f84228774ed187e4bfc22ddaa4f297171f7f23dad4e16e0971745f080`。
- exact single-parenthesized identifier projection 52行、hash
  `b776ca58fb0873f1bb050d15b9ab90a878b02809f980a00c6f05433ceb57cab2`。
- parenthesized node allowlist 6行、hash
  `c45e2f1d15cd1dfc503df711dd58615291f909faff37433d49bca8a741f71157`。

255行は enum、source transport と保持 consumer が使う4 field、generic extractor
だけに最小 runner-scoped visibility を追加し normalized equivalence で移動する。
single-parenthesized projection と parenthesized allowlist は leaf-private になる。
後者の移動により Task 262O の direct-binary allowlist 用 temporary facade alias と
runner import を削除し、その direct allowlist 自体も leaf-private visibility へ戻す。
formula leaf から common source-AST と sibling reserve-extraction leaf への dependency
は acyclic のまま。

named route wrapper 8個、concrete config、source-output transport、checker/output
conversion/validation、detail key、diagnostic、test は `runner.rs` に不変で残す。
既存 exact left/right-parenthesized active slice とその direct source、wrapper/range/
provenance corruption、near-miss、cross-route isolation、immutable-output、real frontend/
resolver sidecar coverage は十分で、test-only prerequisite や新規 test は不要。
完了条件は4 normalized hash、temporary allowlist alias と visibility 削除、wrapper
side/site/range と inner operand ordering 維持、fail-closed behavior と272-name/active
188件 inventory 不変、plan/count/coverage/pass-fail/CLI hash 不変、全 Rust
verification、diff cleanliness、implementation/source-doc no-findings review。rename、
deduplication、generalization、semantic cleanup、checker/output move を禁止する。

## Task 262P Move Result

Task 262P は inventory した parenthesized reserved-variable binary source-core
fragment 4個だけを既存 private `source_formula.rs` leaf へ移動した。追加した
runner-scoped visibility を除くと enum/transport と generic extractor は hash
`9574d330...`、`a252111f...` を維持し、leaf-private single-parenthesized projection
と allowlist は hash `b776ca58...`、`c45e2f1d...` をそのまま維持する。direct-
binary/parenthesized allowlist は両方とも leaf-private。Task 262O の temporary facade
alias、runner import、direct-allowlist visibility はすべて削除した。

結果の `runner.rs` は 12,300行、hash
`563bb974845d95da52e723f1c3e853b79beb55c02e283e1cd10707589d1e5b70`、
39行 facade は hash
`5082a9a6a52c72ed8c95482b425823161bad64b5d75cfb8f14b4143110745c6f`、
2,466行 source-formula leaf は hash
`a09c2c1d757f00c3e27ddb993d78f5aeed06dd08ef0f20aa27c7b080334c9c28`。
named wrapper/config 8個、output transport、checker conversion/validation、detail key、
diagnostic、test は `runner.rs` に残り、moved helper により未使用となった
`SurfaceNodeId` import だけを削除した。

unit test 272件と active type case 188件は全成功。plan/count は 403/367、type
coverage は 235/223、pass/fail は 219/184 のまま。raw/normalized test-list と4つの
CLI hash も不変。formatting、all-target/all-feature Clippy、workspace test、diff
cleanliness は成功した。Task 262P は完了し、次は Task 262Q。authority、behavior、
coverage credit、owner crate、deferred status は不変なので
`spec_coverage_audit.md` は変更しない。

## Task 262Q Pre-Move Inventory and Specification

fresh test-sufficiency/source inventory は Task 262Q の残る reserved-variable type-
assertion source-core fragment を次の3つだけと特定する。

- source transport 13行、hash
  `1a8d06350de32059528b6af1240457874a323a24cb17cbedce128f560c50b00e`。
- generic type-assertion extractor 121行、hash
  `9334dbda0d88f8efbd75a7597471f08777df7f651761c132af4672034bcdf89e`。
- type-assertion node allowlist 18行、hash
  `2fd9587c78d740ffa0893baac5dfc18031ff43296e76bfa69819c2e2ba6b41d1`。

152行は `runner.rs` から既存 private `source_formula.rs` leaf へ normalized
equivalence で移動する。source transport とその10 field、generic extractor だけに
runner-scoped visibility を追加し、allowlist は leaf-private のまま。formula leaf の
既存 common source-AST、source-reserve、mode-expansion、exact identifier dependency
だけを使うため dependency direction は acyclic のまま。

concrete config/named route wrapper 58個、source-output transport、checker/output
conversion/validation、detail key、diagnostic、test は `runner.rs` に不変で残す。
既存 base/object、local-mode、asserted-head、two-through-six-hop、long-chain、exact/
near-miss、source/range/ordinal/head/provenance corruption、immutable-output、cross-route
isolation、real frontend/resolver coverage は paired active `.miz` slice 58件と対応する
unit-test name 137件を含む。fresh review は実装前にこの matrix が十分か確認し、bounded
preservation gap があれば別 test-only prerequisite task/commit を 262Q より先に行う。

完了条件は3 normalized hash、最小 visibility、accepted shape と exact asserted-head
relation 維持、fail-closed behavior と272-name/active 188件 inventory 不変、plan/
count/coverage/pass-fail/CLI hash 不変、全 Rust verification、diff cleanliness、
implementation/source-doc no-findings review。chain generalization、config/wrapper split、
rename、deduplication、semantic cleanup、checker/output move を禁止する。

## Task 262Q0 Test Repair Specification

必須 fresh review は Task 262Q の前に bounded `test_gap` を確認した。既存 base
reserved-variable type-assertion test は synthetic AST から real source extractor/checker
handoff に到達するが、source transport 10 field と config identity をすべて独立固定
せず、15 near miss を aggregate detail key だけで観測し、既に model 済みの
structural corruption 4個を identifier-subject generic extractor に適用していない。
broader active family が real frontend/resolver sidecar を担うが、move 後の generic-core
regression を別 extraction route が隠す可能性は残る。

Task 262Q0 は test-only。既存 private identifier type-assertion AST builder の
corruption argument を default-off wrapper から利用可能にし、
`source_reserved_variable_type_assertion_bridge_checks_reflexive_admissibility` だけを
強化する。positive assertion は formula/subject/asserted-type site/range を AST から
独立導出し、exact config identity と全 config field、reserve、spelling、ordinal、
asserted type、distinct-range payload を固定する。既存 near miss はすべて named
extractor が直接 `None` を返すことを aggregate extraction-gap key より先に確認する。
identifier route の bounded corruption 4個—recovered `is`、duplicate formula
expression、extra formula child、extra assertion operand—にも同じ direct/aggregate
rejection assertion を行う。

Q0 は test を追加せず、production source、`.miz`、expectation、trace、specification、
public API、behavior、diagnostic、coverage credit を変更しない。test name/count hash、
active case、repository count、CLI output は不変でなければならない。Task 262Q は Q0
review と全 verification の後だけ開始できる。

## Task 262Q0 Test Repair Result

Task 262Q0 は既存 base reflexive-admissibility test とその private default-off AST
builder だけを強化した。test は source transport 10 field を AST から導出して固定し、
named extractor route を通じて config 9 value、reserve payload、spelling、ordinal、
asserted type、distinct source range を固定する。既存 near miss 15件はすべて aggregate
gap check より先に named extractor で直接 reject する。recovered `is`、duplicate
formula expression、extra formula child、extra assertion operand corruption にも同じ
direct/aggregate rejection を追加した。

production source、`.miz`、expectation、trace、specification、public API、behavior、
diagnostic、coverage credit、test name、test count は変更していない。unit test 272件と
active type case 188件は全成功。plan/count は 403/367、type coverage は 235/223、
pass/fail は 219/184 のまま。raw/normalized test-list と4つの CLI hash も不変。
formatting、all-target/all-feature Clippy、workspace test、diff cleanliness は成功した。
Task 262Q0 は完了し、次は move-only Task 262Q。authority、behavior、coverage credit、
owner crate、deferred status は不変なので `spec_coverage_audit.md` は変更しない。

## Task 262Q Move Result

Task 262Q は inventory した reserved-variable type-assertion source-core fragment 3個
だけを既存 private `source_formula.rs` leaf へ移動した。追加した runner-scoped
visibility を除くと transport/generic extractor は hash `1a8d0635...`、
`9334dbda...` を維持し、leaf-private allowlist は hash `2fd9587c...` をそのまま
維持する。transport、その10 field、generic extractor だけを runner-scoped とした。
concrete config/named wrapper 58個と output/checker/validation/detail/diagnostic code は
`runner.rs` に不変で残る。

結果の `runner.rs` は 12,144行、hash
`0454931d868a11b6cdfd90b845b8b091f2cd636add4fc8fb6c7aaf43a64cd6e4`、
40行 facade は hash
`a9f7b768ad32e6c51337f3b764db5243a80fc6cf2c16a7d97e57d1e99ef3a770`、
2,621行 source-formula leaf は hash
`a7ffd9dad1e60a7e7890e494e9abc5bafb38e2f9cb11f62d14a03f617fe32b21`。
moved core により obsolete となった import/facade alias は削除した。private test だけが
使う alias 6個は、その direct unit test が使う reserve-extraction guard entry を含め
明示的に `#[cfg(test)]` とした。

unit test 272件と active type case 188件は全成功。plan/count は 403/367、type
coverage は 235/223、pass/fail は 219/184 のまま。raw/normalized test-list と4つの
CLI hash も不変。formatting、all-target/all-feature Clippy、workspace test、diff
cleanliness は成功した。Task 262Q は完了し、次は Task 263。authority、behavior、
coverage credit、owner crate、deferred status は不変なので
`spec_coverage_audit.md` は変更しない。

## Task 263A Pre-Move Inventory and Specification

fresh Task 263 inventory は、保持された checker-handoff substrate を最初の acyclic
bounded family と分類する。正確な `runner.rs:11542-12047` fragment は506行、hash
`95532967e13e1ab39b4ebc23c3403ffe15e57b5a73bda2810d915ccf170175f0`。
`source_module_binding_env` から `typing_for_type_entry` までで、empty module
binding environment、`SourceReserveHandoff` transport、reserve declaration から
`TypedAst`/`ResolvedTypedAst` への assembly、handoff validation、bounded Core
context readiness check、test-only complete handoff entry を所有する。

Task 263A はこの fragment を新しい private
`type_elaboration/checker_handoff.rs` へ機械的に移動する。この leaf は checker、Core、
resolver、session、syntax input と、test-only entry 用の sibling
`SourceReserveExtraction` transport だけに依存する。concrete route config、named
source extractor、detail key、expected-output projection、failure diagnostic、top-level
orchestration は参照しない。これにより後続 Task 263 family より先に
`source_reserve -> checker_handoff -> retained checker/output and orchestration
consumer` という acyclic direction を確立する。

consumer が `runner.rs` に残る間、runner-scoped visibility は
`source_module_binding_env`、`SourceReserveHandoff` とその4 field、
`assemble_source_reserve_checker_handoff`、`assert_source_reserve_handoff`、
`assert_source_reserve_core_summary_readiness`、
`assert_source_reserve_core_context_readiness` に限定する。test-only
`assemble_source_checker_handoff` は `#[cfg(test)]` の場合だけ expose する。
resolved/typed assembly と type-entry projection helper は leaf-private のままにする。

これは move-only `design_drift` で、test prerequisite はない。既存
`source_extraction.rs` direct handoff test、generic output validator/corruption matrix、
unit test 272件、active type case 188件を preservation matrix とする。config、named
wrapper、source extraction、payload、detail key、diagnostic、ordering、fail-closed
behavior、public API、test、authority artifact は変更しない。behavior、coverage
credit、owner crate、deferred status は変わらないため
`spec_coverage_audit.md` は変更しない。

## Task 263A Move Result

Task 263A は inventory 済み checker-handoff substrate を新しい private
`type_elaboration/checker_handoff.rs` leaf へ移動した。review 済み runner-scoped
visibility だけを除去し、元 separator newline を復元すると、moved fragment は exact
hash `95532967e13e1ab39b4ebc23c3403ffe15e57b5a73bda2810d915ccf170175f0`
を維持する。body、control-flow branch、validation string、payload、ordering は変更して
いない。resolved/typed assembly と type-entry helper は leaf-private のまま、complete
handoff entry と sibling source-reserve dependency は `#[cfg(test)]` の場合だけ存在する。

結果の `runner.rs` は11,617行、hash
`4c0aa87165f31fe66816666f1fc33f47b64643e7d644d30db21e8e8f4eb4ed8b`、
46行 phase facade は hash
`daf8415255a5af402436c792414c5fd635b32c5cf397deaff051efbfb16d7ece`、
550行 checker-handoff leaf は hash
`a7cf9bcd076dbc68098ddecbab6c58eca988ecdd8ea378324bad44a32cf5288b`。
`runner.rs` から moved-only import だけを削除した。既存
`SourceReserveDeclarationBridge` test namespace alias は不変 corruption test のため
明示的 `#[cfg(test)]` のまま保持した。

direct handoff test、unit test 272件、active type case 188件は全成功。plan/count は
403/367、type coverage は235/223、pass/fail は219/184 のまま。raw/normalized
test-list hash と4つの CLI byte hash は不変。formatting、all-target/all-feature
Clippy、workspace test、diff cleanliness は成功した。Task 263A は完了し、fresh
Task 263 inventory で次の bounded family を選ぶ。authority、behavior、coverage
credit、owner crate、deferred status は不変なので `spec_coverage_audit.md` は変更しない。

## Task 263B Pre-Move Inventory and Specification

fresh inventory は common frontend diagnostic projection を次の acyclic bounded family
として選ぶ。正確な `runner.rs` fragment 3個で構成する: line 78 の1行 recovery-tag
constant、lines 794-800 の7行 `frontend_detail_keys` fragment は hash
`394797911f19bd3904b4f66d8beed648d418bec9c6f172218f7e8912d21d2038`、lines
11528-11568 の41行 diagnostic-code/assertion/error fragment は hash
`ea3f9ffb0862e0a37575de150b82a3d654000778e87fa5abd0d9d41a40ff50a3`。
recovery-tag の full hash は
`381e1d7f0e9ab985a0ce5436a8b6e19f63ca169da43f54c35fcfb42d68972b04`。
source order で連結すると49行、hash
`0a4d80ff40dbf1d936ea0f5a965047e1a5f3a961812ede65deca56a8866a4ba5`。

Task 263B はこの fragment を、`FrontendRun` を所有し `FrontendDiagnostic` と
`TestCase` を直接 import 済みの既存 private `runner/shared.rs` へ機械的に移動し、
frontend import に `DiagnosticCode` を追加する。
recovery tag と `frontend_diagnostic_code` は leaf-private のままにする。
`frontend_detail_keys`、`assertion_diagnostic_codes`、`frontend_error_code` だけを
parent-only entry とする。parse-only と declaration-symbol は shared sibling entry 3個を
直接 import し、`runner.rs` は保持 type consumer のため shared detail-key entry を
import して now-unused `DiagnosticCode`/`FrontendDiagnostic` import を削除する。
child-to-parent/checker dependency なしで `shared frontend/diagnostic projection ->
phase consumer` を確立する。

これは move-only `design_drift` で、Task 263B0 test prerequisite はない。recovery-tag
case を含む active parse matrix、declaration/type lower-stage case、active-runner byte-
stability/repository execution test、4つの CLI projection が code mapping、key prefix、
ordering、fallback behavior を保持する。test、expectation、public API、diagnostic、
payload、source behavior、authority artifact は変更しない。source file は追加しないため
paired Source Inventory file list は不変。coverage credit、owner crate、deferred status
は変わらないため `spec_coverage_audit.md` は変更しない。

## Task 263B Move Result

Task 263B は正確な common frontend diagnostic family を既存 private `shared.rs` へ
移動した。review 済み `pub(super)` modifier 3個を除去し、whitespace と rustfmt の
optional trailing signature comma を正規化すると、旧/moved family はともに hash
`f7b793a4a93ec14cb24869c5de1e8b87ad35c79012185308c7ebaaf06d2f994b`
となる。recovery tag と fallback mapper は leaf-private のまま。parse-only と
declaration-symbol は shared entry を直接 import し、保持 type consumer は runner
owner 経由で `frontend_detail_keys` だけを import する。

結果の `runner.rs` は11,566行、hash
`6cc0b8a7a70f4298761df02f1d8be755ba22416625cffd8e8fcf6d8660dc5f59`、
260行 `shared.rs` は hash
`1c5f780fbb0df10faf8f363594e5b19fbd7eb19abc852ece67308559141689b8`。
diagnostic string、match arm、syntax/non-syntax branch、iteration order、prefix、
wildcard fallback、frontend-error formatting は変更していない。

unit test 272件、active parse 96件、declaration-symbol 4件、type-elaboration 188件は
全成功。plan/count は403/367、type coverage は235/223、pass/fail は219/184 のまま。
raw/normalized test-list hash と4つの CLI byte hash は不変。formatting、all-target/
all-feature Clippy、workspace test、diff cleanliness は成功した。Task 263B は完了し、
fresh Task 263 inventory で次の bounded family を選ぶ。file、authority、behavior、
coverage credit、owner crate、deferred status は不変なので Source Inventory と
`spec_coverage_audit.md` の変更は不要。

## Task 263C Pre-Move Inventory and Specification

fresh inventory は `runner.rs:11512-11535` の正確な24行 expected-result/failure-
projection family を選ぶ。`expected_type_elaboration_detail_keys` と
`type_elaboration_failure_diagnostic` を含み、raw hash は
`b9efaec531ff58c52d028b413f8ea644640a5f0aeccaf57da3682cd7c5d1317c`。
direct dependency は `TestCase`、`ValidationDiagnostic`、stable public
`TypeElaborationCaseResult` DTO だけ。

Task 263C はこの family を新しい private `type_elaboration/result.rs` へ機械的に
移動する。両 function を type-elaboration facade 経由の parent-only entry とし、他の
export は作らない。stable runner result DTO への leaf dependency は明示的な facade-
contract edge とする。payload list を stable key より優先する順序、failure
code/key/text、expected/actual formatting、vector order を正確に維持する。

`run_type_elaboration_case` は large retained actual-detail dispatcher に依存するため
この task では `runner.rs` に残す。今移動すると result-leaf から parent-private への
reverse edge を作るか、後続 detail/output family と混ざる。generic output validator
も current output/config/source-helper dependency graph の separate bounded inventory が
必要なため pending のままにする。

これは move-only `design_drift` で、Task 263C0 prerequisite はない。stable-detail
fallback test、active type case 188件、repository/CLI byte-stability、normalized exact-
body equivalence により result matching/failure assembly を保持する。test、expectation、
diagnostic、API、payload、behavior、authority artifact は変更しない。新しい source
path は move と同時に paired Source Inventory へ追加する。coverage credit、owner
crate、deferred status は変わらないため `spec_coverage_audit.md` は変更しない。

## Task 263C Move Result

Task 263C は正確な24行 expected-result/failure-projection family を新しい private
`type_elaboration/result.rs` へ移動した。必要な
`pub(in crate::runner)` visibility qualifier 2個だけを除去すると、旧/moved body は
ともに hash
`b9efaec531ff58c52d028b413f8ea644640a5f0aeccaf57da3682cd7c5d1317c`
となる。facade はこの2 entry だけを parent-only expose する。case execution と
actual-detail dispatcher は `runner.rs` に残すため、reverse dependency を導入せず
public API も変更しない。

結果の `runner.rs` は11,541行、hash
`2e6bc713114f726af47de08d7ceb622f9d0f79282d00994be458f7f35e0c435e`、
50行 `type_elaboration.rs` facade は hash
`44634b3b24f645bbb49ea66c1569cf251c8f11db505c94de252877e9112c02cc`、
新しい29行 `result.rs` は hash
`608b458dd0d7491d7af1d6ef9261e468ec548b39966ecfa8acbc81bd8b7bd4c2`。
payload-list precedence、stable-key fallback、failure code/key/text、expected/actual
formatting、vector order、fail-closed caller flow は不変。

unit test 272件、active parse 96件、declaration-symbol 4件、type-elaboration
188件は全成功。plan/count は403/367、type coverage は235/223、pass/fail は
219/184 のまま。raw/normalized test-list hash と4つの CLI byte hash は不変。
paired Source Inventory に新しい leaf を追加した。Task 263C は完了し、fresh Task
263 inventory で次の bounded validation/detail family を選ぶ。authority、behavior、
coverage credit、owner crate、deferred status は不変なので
`spec_coverage_audit.md` の変更は不要。

## Task 263D Pre-Move Inventory and Specification

fresh inventory は type-elaboration active-admission family を正確な4 fragment として
選ぶ。1行 `ACTIVE_TYPE_ELABORATION_TAG` constant
(`4629969fa68b61384e96b345b2a646d786b6f843ca5ad128fa17723d473d68ac`)、
13行 runnable predicate
(`5303e0c27405121d8aeefb7c6e2163dfcd288419c44b6e44779b1df4e0a41c9d`)、
6行 tag predicate
(`c91740986c91df19297de24f8c6f7441fed4886de246e18c65b5361e4a1fdd5b`)、
30行 gate validator
(`b0cb1652b4046473ce2bc12285ac09a69411c08d32b6a7144a501a9f27818945`)
である。source order で連結した正確な50行の hash は
`937c032b2504225dbe5e79f5526545d969929afbd8dbfc9c09faf4cc5ad7a429`。

Task 263D はこの family を新しい private `type_elaboration/admission.rs` へ機械的に
移動する。tag constant と tag predicate は leaf-private のままにする。
`is_active_type_elaboration` と `validate_active_type_elaboration_tags` だけを
type-elaboration facade 経由の parent-only entry とする。public
`active_type_elaboration_cases` iterator と corpus-level orchestration は
`runner.rs` に残す。direct dependency は `ValidationDiagnostic`、`ExpectedOutcome`、
`PipelinePhase`、`TestCase`、`TestPlan`、`Stage` だけで、source、checker、output、
parent DTO dependency はない。

これは move-only `design_drift` で、Task 263D0 prerequisite はない。既存の non-type、
wrong-phase、public-diagnostic-code gate test が gate branch、diagnostic code、silent-
skip rejection を直接保持する。normalized exact-body equivalence と repository/report/
CLI byte-stability が detail key、text、order、active case 188件の iteration behavior を
保持する。test、
expectation、diagnostic、API、payload、behavior、authority artifact は変更しない。
新しい source path は move と同時に paired Source Inventory へ追加する。coverage
credit、owner crate、deferred status は変わらないため
`spec_coverage_audit.md` は変更しない。

## Task 263D Move Result

Task 263D は正確な4 fragment/50行 type active-admission family を新しい private
`type_elaboration/admission.rs` へ移動した。必要な
`pub(in crate::runner)` visibility qualifier 2個だけを除去し、ASCII whitespace を
fold し、rustfmt の trailing `TestPlan` signature comma だけを正規化すると、旧/moved
family はともに hash
`ea1a50947f895bcbc5bcca417432b3860369174677ea9b8b4b7626ca651157c4`
となる。tag constant/tag predicate は leaf-private のまま、facade は runnable
predicate と gate validator だけを parent-only expose する。public iterator と corpus
orchestration は `runner.rs` に残す。

結果の `runner.rs` は11,490行、hash
`5d58dcfe62d1d724a731f5421ad6547d7e8e7757581297efe7b6a000adec2230`、
52行 `type_elaboration.rs` facade は hash
`b06293cc471453df1bb373a53b51cbba2d8b3991ec5206c5b0ecd719047839e7`、
新しい60行 `admission.rs` は hash
`b5261a23dae29eb656ba6f414a622a4cc40501dabd0fcf457fedf53b23aba150`。
admission branch、diagnostic code/key/text、case内 diagnostic order、silent-skip
rejection は不変。

focused gate test 3件、unit test 272件、active parse 96件、declaration-symbol 4件、
type-elaboration 188件は全成功。plan/count は403/367、type coverage は235/223、
pass/fail は219/184 のまま。raw/normalized test-list hash と4つの CLI byte hash は
不変。paired Source Inventory に新しい leaf を追加した。Task 263D は完了し、fresh
Task 263 inventory で次の bounded validation/detail family を選ぶ。authority、
behavior、coverage credit、owner crate、deferred status は不変なので
`spec_coverage_audit.md` の変更は不要。

## Task 263E Pre-Move Inventory and Specification

fresh dependency inventory は `runner.rs:5361-5393` の正確な連続33行 checker-output
transport substrate を選ぶ。binary-formula、parenthesized-binary-formula、type-
assertion output struct 3個と22 field で、raw family hash は
`e5da36674f0779384d90fa35a7f42ee209dfbca2049efe76b2893c0b36705ce0`。
production/test の type reference 153個、named output/validator test reference 346個を
持つ real transport であり、empty/synthetic owner ではない。

Task 263E はこの3 transport だけを新しい private
`type_elaboration/output.rs` へ機械的に移動する。3 type と22 field は、保持 builder、
validator、named output helper、既存 corruption test が必要とする最小
`pub(in crate::runner)` visibility とし、type-elaboration facade は3 type だけを
parent-only re-export する。direct dependency は sibling source payload transport と
checker handoff、および checker/session typed input/inference output である。builder、
validator、detail projection、named wrapper、config、全 call site は separate bounded
task のため `runner.rs` に残す。

これは後続 output-owner task の move-only `design_drift` prerequisite で、Task 263E0
prerequisite はない。既存 field-level payload/corruption matrix と normalized exact-body
equivalence が全 field/type/order/debug shape を保持する。272-test list、active case
188件、repository/report/CLI byte-stability、全 gate が API、payload、diagnostic、order、
fail-closed behavior を保持する。test、expectation、semantic helper、source behavior、
authority artifact は変更しない。新しい source path は move と同時に paired Source
Inventory へ追加する。coverage credit、owner crate、deferred status は変わらないため
`spec_coverage_audit.md` は変更しない。

## Task 263E Move Result

Task 263E は正確な33行3 transport checker-output substrate を新しい private
`type_elaboration/output.rs` へ移動した。3 type と22 field に必要な25個の
`pub(in crate::runner)` qualifier だけを除去すると、moved lines 11-43 は元の raw
hash を byte-for-byte 再現する:
`e5da36674f0779384d90fa35a7f42ee209dfbca2049efe76b2893c0b36705ce0`。
facade は3 transport だけを parent-only re-export する。builder、validator、detail
projection、named wrapper、config、call site は `runner.rs` に残し、now-unused parent
`SourceRange`/`SourceReserveHandoff` import だけを削除した。

結果の `runner.rs` は11,457行、hash
`d43d0f6a62cff726fffc88ebe2452932371626a71a9e13aa9bae09eb8168708e`、
57行 `type_elaboration.rs` facade は hash
`0c068fd8a7bca6f7d0194e06cda9723eb0bfe8d39b1bc3d3c6553c5a6cb61c86`、
新しい43行 `output.rs` は hash
`bb056c40bdafeb2d3f60821da8cf4fa908045b16dc0230defbff85bc27bdb350`。
derive、field type/order、debug shape、payload、fail-closed behavior は不変。

focused output test 4件、unit test 272件、active parse 96件、declaration-symbol 4件、
type-elaboration 188件は全成功。plan/count は403/367、type coverage は235/223、
pass/fail は219/184 のまま。raw/normalized test-list hash と4つの CLI byte hash は
不変。paired Source Inventory に新しい leaf を追加した。Task 263E は完了し、fresh
Task 263 inventory で次の bounded builder/validator/detail family を選ぶ。authority、
behavior、coverage credit、owner crate、deferred status は不変なので
`spec_coverage_audit.md` の変更は不要。

## Task 263F Pre-Move Inventory and Specification

fresh dependency inventory は `runner.rs:8441-8701` の連続 checker-output builder 3個
（261行、hash
`cb4396e080d9f31f79e57feebfd5de5badad92f3aedfdf358b0eb277eb416b25`）と、それら
だけが使う `runner.rs:9473-9488` の16行 `source_reserved_type_projection` helper
（hash `c450e8588af637f3f3a8dc04f522ef988dc470a54b4d005001c4ba5f102f33b0`）
を選ぶ。source order で連結した正確な277行 producer family の hash は
`b4939bbe52118a6b6e1d268bff26c6fa11e2994e14e0bb0b4e7215e94a41efaa`。

Task 263F はこの family を既存 private `type_elaboration/output.rs` transport owner へ
機械的に移動する。type-assertion、binary-formula、parenthesized-binary builder entry
を type-elaboration facade 経由の parent-only とし、projection helper は leaf-private
のままにする。direct dependency は Task 263E output transport、sibling source
payload/config transport、sibling checker-handoff assembly、resolver symbol、checker
binding/type/formula input API である。保持 validator、detail projection、named wrapper、
active orchestration、public result DTO への dependency はない。

これは move-only `design_drift` で、Task 263F0 prerequisite はない。既存 source-
output、field-provenance、lookup-ordinal、checker-payload、corruption、active fixture
matrix が3 builder と fail-closed branch を実行する。exact-body equivalence、272-test、
active case 188件、repository/report/CLI byte-stability、全 gate が construction order、
error string、source range、binding identity、input、payload、failure boundary を保持する。
test、expectation、validator、detail key、config、semantic behavior、authority artifact は
変更しない。既存 `output.rs` owner の拡張なので Source Inventory は不変。coverage
credit、owner crate、deferred status は変わらないため
`spec_coverage_audit.md` は変更しない。

## Task 263F Move Result

Task 263F は正確な3 builder/sole projection-helper producer family を既存 private
`type_elaboration/output.rs` へ移動した。必要な3個の
`pub(in crate::runner)` builder qualifier だけを除去すると、moved lines 51-311 は
hash `cb4396e080d9f31f79e57feebfd5de5badad92f3aedfdf358b0eb277eb416b25`、
private helper lines 313-328 は hash
`c450e8588af637f3f3a8dc04f522ef988dc470a54b4d005001c4ba5f102f33b0`、
source-order combination は
`b4939bbe52118a6b6e1d268bff26c6fa11e2994e14e0bb0b4e7215e94a41efaa`
を維持する。facade は3 builder だけを parent-only expose する。validator、detail
projection、named wrapper、config、call site は `runner.rs` に残す。

結果の `runner.rs` は11,180行、hash
`cfefc3b316fe7d9ff33153475ed42540fcf8605a16ad11132f4380c7ca0350a7`、
60行 `type_elaboration.rs` facade は hash
`c673946fddb223a2ae566073205bffaac56ce34ccbb393ae0e755ad6d5c15658`、
328行 `output.rs` は hash
`41a151db0d3e6fc4ba45c04989e1bbf577cfc4a8ae55ba9d570998794c90bbcd`。
construction order、error text、source range、binding identity、checker input、payload、
fail-closed flow は不変。

focused builder test 4件、unit test 272件、active parse 96件、declaration-symbol 4件、
type-elaboration 188件は全成功。初回 full-crate run では無関係な `/tmp` fixture path
missing が一時発生したが、exact failing route-isolation test と full crate rerun は両方
成功した。plan/count は403/367、type coverage は235/223、pass/fail は219/184 の
まま。raw/normalized test-list hash と4つの CLI byte hash は不変。Task 263F は完了し、
fresh Task 263 inventory で次の bounded validator/detail family を選ぶ。path、authority、
behavior、coverage credit、owner crate、deferred status は変わらないため Source
Inventory と `spec_coverage_audit.md` は不変。

## Task 263G Pre-Move Inventory and Specification

fresh dependency inventory は `runner.rs:8443-8656` の正確な type-assertion output
validator と private role-entry helper（214行、hash
`17ad7203816094ef55580f9356388510e6164cdc2f4a38412639d496db1b623c`）、および
`runner.rs:9197-9211` の shared normalized-builtin-type predicate（15行、hash
`c1e417207bcc04654fdeb3fee13a00985a5aff63181298d1b65d149d3d6f15aa`）を選ぶ。
source order で連結した正確な229行 family の hash は
`b6557af65c99430f112772b665c36a3545bdb39f48541e1c817f06eadfc0b10f`。

Task 263G はこの family を既存 private `type_elaboration/output.rs` へ機械的に移動する。
type-assertion validator は parent-only、role-entry helper は leaf-private とし、
normalized-type predicate は保持 binary-formula validator に既存 call site が2個あるため
一時的に parent-only とする。dependency は Tasks 263E/F output/checker-handoff owner、
exact source-formula predicate、checker typed-output API だけである。binary/parenthesized
validator、detail-key projection、named wrapper、config、call site、orchestration は
`runner.rs` に残す。

これは move-only `design_drift` で、Task 263G0 prerequisite はない。production
detail-result path と既存10 test module の212 direct validator assertion が exact success、
provenance、lookup ordinal、checker count/identity、canonical source、corruption rejection、
route isolation、fail-closed behavior を固定する。exact-body equivalence、272-test、
active case 188件、repository/report/CLI byte-stability、全 gate により、全 error string、
comparison、ordering decision、detail key、payload、failure boundary を保持する。test、
expectation、config、wrapper、validator logic、semantic behavior、authority artifact は
変更しない。既存 `output.rs` path を拡張し、coverage credit、owner crate、deferred
status は変わらないため Source Inventory と `spec_coverage_audit.md` は不変。

## Task 263G Move Result

Task 263G は正確な type-assertion validator、private role-entry helper、shared
normalized-type predicate を既存 private `type_elaboration/output.rs` へ移動した。必要な
2個の `pub(in crate::runner)` qualifier だけを除去すると、moved lines 337-550 は
hash `17ad7203816094ef55580f9356388510e6164cdc2f4a38412639d496db1b623c`、
lines 552-566 は hash
`c1e417207bcc04654fdeb3fee13a00985a5aff63181298d1b65d149d3d6f15aa`、
source-order combination は
`b6557af65c99430f112772b665c36a3545bdb39f48541e1c817f06eadfc0b10f`
を維持する。facade は validator と一時 shared predicate だけを parent-only expose し、
role helper は leaf-private のまま。保持 binary validator は元の predicate call 2個だけを
引き続き持つ。

結果の `runner.rs` は10,948行、hash
`97247c5bedcee1baebaec2f5caae2d332dea5de246e18671992db4ddbc64e2aa`、
61行 `type_elaboration.rs` facade は hash
`c36560ef2972e383d2a0d59aa1021fb8341d0bfbf8c79ebded0e1dbc16d2df0c`、
566行 `output.rs` は hash
`01c75f7906b759308c9c52f36768dbd46b1d3f8fd462507bc448f538601224d5`。
全 validation branch、error string、comparison、checker lookup、normalized-type identity
check、canonical-source check、fail-closed return は不変。

focused type-assertion test 47件、unit test 272件、active parse 96件、
declaration-symbol 4件、type-elaboration 188件は全成功。plan/count は403/367、type
coverage は235/223、pass/fail は219/184 のまま。272行 raw/normalized test-list hash と
4つの CLI byte hash は不変。Task 263G は完了し、fresh Task 263 inventory で次の
bounded binary/parenthesized-validator または detail family を選ぶ。path、authority、
behavior、coverage credit、owner crate、deferred status は変わらないため Source
Inventory と `spec_coverage_audit.md` は不変。

## Task 263H Pre-Move Inventory and Specification

fresh dependency inventory は `runner.rs:8442-8779` の正確な binary-formula output
validator（338行、hash
`528876adb6cda98d2030df317d2589733799917682f9bdcf0d04f1333ff46ddf`）、
`runner.rs:8937-8953` の source-type projection predicate（17行、hash
`4317df8d93687b6357cc2f5943cd4c1b248fc69e2092c02586588c00bfa40170`）、
`runner.rs:8955-8979` の type-entry validator（25行、hash
`135354e0b3aa68dbd5435a869134722b2617b0e65faea16810ff9a3ad657f43e`）
を選ぶ。source order で連結した正確な380行 family の hash は
`76fcab1f8c068b9b0ee0bd552b106e9a23cce794e7ff0f9134120e2285de7836`。

Task 263H はこの family を既存 private `type_elaboration/output.rs` へ機械的に移動する。
binary validator は保持 production detail/parenthesized-validator consumer のため
parent-only、helper predicate 2個は leaf-private とする。Task 263G normalized-type
predicate の call 2個も一緒に移るため、その predicate は一時 parent-only から
leaf-private に狭めて facade から除去する。parenthesized validator、detail projection、
named wrapper、config、call site、orchestration は `runner.rs` に残す。

これは move-only `design_drift` で、Task 263H0 prerequisite はない。production
detail path、保持 parenthesized consumer、既存11 test module の104 direct validator
assertion が exact success、binding/provenance/ordinal identity、expected/result constraint、
checker count/order、semantic type sharing、canonical source、corruption rejection、route
isolation、fail-closed behavior を固定する。exact-body equivalence、272-test、active case
188件、repository/report/CLI byte-stability、全 gate により、全 error string、comparison、
ordering decision、detail key、payload、failure boundary を保持する。test、expectation、
config、wrapper、validator logic、semantic behavior、authority artifact は変更しない。
既存 `output.rs` path を拡張し、coverage credit、owner crate、deferred status は変わらない
ため Source Inventory と `spec_coverage_audit.md` は不変。

## Task 263H Move Result

Task 263H は正確な binary-formula validator と private helper 2個を既存 private
`type_elaboration/output.rs` へ移動した。必要な validator の
`pub(in crate::runner)` qualifier だけを除去すると、moved lines 570-907 は hash
`528876adb6cda98d2030df317d2589733799917682f9bdcf0d04f1333ff46ddf`、
lines 909-925 は hash
`4317df8d93687b6357cc2f5943cd4c1b248fc69e2092c02586588c00bfa40170`、
lines 927-951 は hash
`135354e0b3aa68dbd5435a869134722b2617b0e65faea16810ff9a3ad657f43e`、
source-order combination は
`76fcab1f8c068b9b0ee0bd552b106e9a23cce794e7ff0f9134120e2285de7836`
を維持する。validator だけ parent-only。helper 2個と normalized-type predicate は
leaf-private で、一時 normalized predicate facade alias は除去した。parenthesized
validator、detail projection、config、wrapper、call site は `runner.rs` に残す。

結果の `runner.rs` は10,558行、hash
`2440c1f2cce788ed4f58437338124639f36327b88572105b4b3a80c4e4679446`、
62行 `type_elaboration.rs` facade は hash
`1ff372989d8ccce83ce68645ac054e245ec4c85f90cf1c2919fb56fac3c8216f`、
951行 `output.rs` は hash
`2fe4650c4be3c5560ab991278dcc701e32581c75b8ab7429c90d95ccc86a9689`。
全 validation branch、error string、collection order、lookup、expected/result constraint、
semantic identity check、canonical-source choice、fail-closed return は不変。

focused reserved-variable test 123件、unit test 272件、active parse 96件、
declaration-symbol 4件、type-elaboration 188件は全成功。plan/count は403/367、type
coverage は235/223、pass/fail は219/184 のまま。272行 raw/normalized test-list hash と
4つの CLI byte hash は不変。Task 263H は完了し、fresh Task 263 inventory で次の
bounded parenthesized-validator または detail family を選ぶ。path、authority、behavior、
coverage credit、owner crate、deferred status は変わらないため Source Inventory と
`spec_coverage_audit.md` は不変。

## Task 263I Pre-Move Inventory and Specification

fresh dependency inventory は config-independent shared parenthesized-binary validator core
`runner.rs:8523-8589` だけを選ぶ（67行、hash
`2de635a7524ac3734edb15c9d953dab6fc73b4800f5d3804866c0cffba7e5d88`）。
これは sole parenthesized wrapper/range/typed-output isolation predicate で、Task 263H
binary validator を直接呼ぶ。

Task 263I はこの exact core を既存 private `type_elaboration/output.rs` へ機械的に移動し、
保持 named test wrapper 8個と production detail consumer のため parent-only entry 1個を
設ける。concrete config、named validator、detail projection、output wrapper、call site、
orchestration は `runner.rs` に残し、同時移動による config-independent core boundary
越境を避ける。

これは move-only `design_drift` で、Task 263I0 prerequisite はない。named wrapper 8個は
既存2 test module に16 direct assertion を持ち、active/report detail path が production
consumer を実行する。left/right wrapper side、config identity、source/copied wrapper
site/range equality、distinct typed site、source-id/range containment、term/type entry/formula
からの exclusion、corruption rejection、route isolation、fail-closed behavior を固定する。
exact-body equivalence、272-test、active case 188件、repository/report/CLI byte-stability、
全 gate により、全 error string、comparison、ordering decision、detail key、payload、
failure boundary を保持する。test、expectation、config、wrapper logic、semantic behavior、
authority artifact は変更しない。既存 `output.rs` path を拡張し、coverage credit、owner
crate、deferred status は変わらないため Source Inventory と `spec_coverage_audit.md` は
不変。

## Task 263I Move Result

Task 263I は正確な config-independent parenthesized-binary validator core だけを既存
private `type_elaboration/output.rs` へ移動した。必要な `pub(in crate::runner)` qualifier
を除去すると、moved lines 954-1020 は hash
`2de635a7524ac3734edb15c9d953dab6fc73b4800f5d3804866c0cffba7e5d88`
を維持する。facade はこの validator 1個だけを parent-only expose する。named
validator 8個、concrete config、detail projection、output wrapper、call site はすべて
`runner.rs` に残し、leaf は generic config type だけを import する。

結果の `runner.rs` は10,491行、hash
`3d75554d7cc1c45b5cdbab06ce27a30bd660cb01a4cd5e9311157442c5a43205`、
63行 `type_elaboration.rs` facade は hash
`dfd15b3390d53dd6c84decf0babb117077e53ce400b4325126757faff3061453`、
1,020行 `output.rs` は hash
`0c18a5d1244da77a85d73368d622dda2699b95463b015e980cd98604b79a6a16`。
wrapper-side selection、pointer identity、source/copy site/range check、containment、
typed-output exclusion、error text、fail-closed return は不変。

focused parenthesized test 25件、unit test 272件、active parse 96件、
declaration-symbol 4件、type-elaboration 188件は全成功。plan/count は403/367、type
coverage は235/223、pass/fail は219/184 のまま。272行 raw/normalized test-list hash と
4つの CLI byte hash は不変。Task 263I は完了し、fresh Task 263 inventory で次の
bounded detail または config/named-wrapper family を選ぶ。path、authority、behavior、
coverage credit、owner crate、deferred status は変わらないため Source Inventory と
`spec_coverage_audit.md` は不変。

## Task 263J Pre-Move Inventory and Specification

fresh dependency inventory は `runner.rs:6973-7018` の正確な type-assertion
result/detail core を選ぶ（46行、hash
`3d4f7e8ce9ff1b60d0960e33fb8e1689fb4862a1730cf3144137e720db053fb8`）。
parent 向け result projection は既存 consumer 125個を持ち、output diagnostic collector
は選択 core 内だけで使われる。

Task 263J はこの family を既存 private `type_elaboration/output.rs` へ機械的に移動する。
result projection は保持 named detail wrapper のため parent-only、collector は Task 263G
validator/output transport の隣で leaf-private とする。binary/parenthesized detail core、
config、named wrapper、output wrapper、call site、orchestration は `runner.rs` に残す。

これは move-only `design_drift` で、Task 263J0 prerequisite はない。既存8 test module
の direct matrix と active/report consumer が validator-first rejection、invalid-key
fallback、binding/declaration/formula diagnostic collection、checker prefix、canonical
iteration、sort/dedup、empty success、corruption rejection、fail-closed behavior を固定する。
exact-body equivalence、272-test、active case 188件、repository/report/CLI byte-stability、
全 gate により、全 key、fallback、ordering decision、payload、failure boundary を保持する。
test、expectation、config、wrapper logic、semantic behavior、authority artifact は変更しない。
既存 `output.rs` path を拡張し、coverage credit、owner crate、deferred status は変わらない
ため Source Inventory と `spec_coverage_audit.md` は不変。

## Task 263J Move Result

Task 263J は正確な type-assertion result/detail core だけを既存 private
`type_elaboration/output.rs` へ移動した。必要な `pub(in crate::runner)` qualifier を
除去すると、moved lines 536-581 は hash
`3d4f7e8ce9ff1b60d0960e33fb8e1689fb4862a1730cf3144137e720db053fb8`
を維持する。result projection は parent-only、diagnostic collector は leaf-private。
production detail consumer は result projection だけを使うため、facade/runner の direct
validator/output type alias は test-only に gate した。binary/parenthesized detail、config、
named/output wrapper、call site は `runner.rs` に残す。

結果の `runner.rs` は10,444行、hash
`66bda6fe475617e30298b8dfb9384b92d55a033a23ee11726ada2e8ba9e6a8c2`、
68行 `type_elaboration.rs` facade は hash
`5a2412bfbf81a7505ccc03d68a12266a9ce5ec238247ed2c583c5cf08666ec4a`、
1,067行 `output.rs` は hash
`0afb49bbd16b8eb320e70d6997818302290cf1352fefe0b2c7ad3a3a2e9be1df`。
validator-first rejection、fallback selection、diagnostic source/prefix、canonical
iteration、sort/dedup、empty success、fail-closed behavior は不変。

focused type-assertion test 47件、unit test 272件、active parse 96件、
declaration-symbol 4件、type-elaboration 188件は全成功。plan/count は403/367、type
coverage は235/223、pass/fail は219/184 のまま。272行 raw/normalized test-list hash と
4つの CLI byte hash は不変。Task 263J は完了し、fresh Task 263 inventory で次の
bounded binary/parenthesized detail または config/named-wrapper family を選ぶ。path、
authority、behavior、coverage credit、owner crate、deferred status は変わらないため
Source Inventory と `spec_coverage_audit.md` は不変。

## Task 263K 移動前 Inventory と仕様

fresh dependency inventory は `runner.rs:6973-7008` の正確な binary-formula
result/detail core（36行、hash
`be8659f6d1bd22caba5270f0ea180521a90375e8b37c8f1a7b9e8f0cb4068b37`）を選ぶ。
result projection は保持 production consumer 52個を持ち、その diagnostic collector
は保持 shared parenthesized-detail core からも使われる。shared-support import 2個を
除き、test module 6個が両 entry を直接145回参照する。

Task 263K はこの2 function だけを既存 private `type_elaboration/output.rs` へ機械的に
移動する。保持 runner consumer が両方を呼ぶため、両 entry は parent-only となる。
production collector が binary validator と同じ leaf へ移動した後、facade と runner
の direct validator/output type alias は test-only gate する。parenthesized detail、
全 config と named/output wrapper、全 call site は `runner.rs` に保持する。

これは move-only `design_drift` であり、Task 263K0 prerequisite は不要。既存 direct
result/output matrix は validator-first rejection、設定済み invalid-key fallback、
declaration/formula diagnostic source、checker-key prefix、canonical iteration、
sort/dedup、empty success を cover する。preservation matrix は必要な visibility を
除く exact function-body equivalence、stable key/diagnostic payload order 不変、272 test
の raw/normalized list 不変、active type 188件と plan/count byte 不変、`.miz`、
expectation、trace、spec、API、config、wrapper、call-site edit なしとする。

authority inventory に canonical contradiction はない。`doc/spec/en`、既存 `.miz`
corpus、`spec_trace.toml`、expectation は従来 intent を保持し、`harness.md` と
`expectation_schema.md` は active runner と deterministic detail contract を引き続き
定義し、source は修正対象の derived layout にすぎない。既存 `output.rs` path、
authority、behavior、credit、owner crate、deferred status は変わらないため、Source
Inventory と `spec_coverage_audit.md` は変更しない。

## Task 263K Move Result

Task 263K は正確な binary-formula result/detail core だけを既存 private
`type_elaboration/output.rs` へ移動した。必要な2個の `pub(in crate::runner)` qualifier
を除去すると、moved lines 957-992 は hash
`be8659f6d1bd22caba5270f0ea180521a90375e8b37c8f1a7b9e8f0cb4068b37` を維持する。
両 projection は parent-only。最初の non-test crate build により、validator alias に
加えて direct binary output-type alias も test-only になったことを検出したため、paired
inventory を精密化し、facade/runner の両 alias を `#[cfg(test)]` にした。
parenthesized detail、全 config と named/output wrapper、全 call site は `runner.rs` に
保持する。

結果の `runner.rs` は10,411行、hash
`bc7d9d3dc7536c8311eb9b7c5c6131657114ad1b3bdc2f5a3b13149642ccc1b3`、69行
`type_elaboration.rs` facade は hash
`3411dfac21ea4872bdbea24466a64c7cdaafc27c54828b397913f483ed00e2e7`、1,104行
`output.rs` は hash
`94a0aa92cacfacf2ef32bc0b5b8e336f7340c19a3bcc4ef505052e568b3b69e9`。
builder-error fallback、validator-first rejection、設定済み invalid-key fallback、
declaration/formula diagnostic source、checker-key prefix、canonical iteration、
sort/dedup、empty success、fail-closed behavior は不変。

focused source-reserved-variable test 4件、unit test 272件、relevant crate integration
test、active parse 96件、declaration-symbol 4件、type-elaboration 188件は全成功。
plan/count は403/367、type coverage は235/223、pass/fail は219/184、warning/error は
23/0 のまま。raw/normalized test-list hash と4つの CLI byte hash は不変。format、
warning deny の all-target/all-feature Clippy、workspace test、diff cleanliness は成功。
Task 263K は完了し、fresh Task 263 inventory は次の bounded parenthesized-detail または
config/named-wrapper family を選ぶ。path、authority、behavior、coverage credit、owner
crate、deferred status は変わらないため Source Inventory と
`spec_coverage_audit.md` は不変。

## Task 263L 移動前 Inventory と仕様

fresh dependency inventory は `runner.rs:7065-7080` の正確な shared
parenthesized-binary output-detail core（16行、hash
`700b2283f7a6ea7b61c97ec59a27166404a72eccdce8f8e7aa7c681dd9003e47`）を選ぶ。
sole production caller は parenthesized active route 8個すべてを担う保持 payload-detail
wrapper。保持 test-only named wrapper 8個も同じ core を呼び、`reserved_binary.rs` に
direct assertion 26個がある。

Task 263L はこの shared core だけを既存 private `type_elaboration/output.rs` へ機械的に
移動する。保持 payload/named-wrapper caller のため parent-only となる。parenthesized
validator と同じ leaf へ移動した後、facade と runner の direct parenthesized
validator/output-type alias と direct binary detail-collector alias は test-only gate
する。payload-detail wrapper、config 8個、全 named detail/validator/output wrapper、
全 call site は `runner.rs` に保持する。

これは move-only `design_drift` であり、Task 263L0 prerequisite は不要。既存 active/
direct matrix は保持 caller の builder fallback、validator-first wrapper rejection、
設定済み invalid-key fallback、nested binary diagnostic projection、left/right wrapper
identity、fail-closed behavior を cover する。preservation matrix は必要な visibility を
除く exact function-body equivalence、272 test の raw/normalized list 不変、active/CLI
byte 不変、`.miz`、expectation、trace、spec、API、config、wrapper、payload-detail、
call-site edit なしとする。

authority inventory に canonical contradiction はない。既存 `output.rs` path/owner は
不変で、authority、behavior、coverage credit、deferred status は変わらないため、
Source Inventory と `spec_coverage_audit.md` は変更しない。

## Task 263L Move Result

Task 263L は正確な shared parenthesized-binary output-detail core だけを既存 private
`type_elaboration/output.rs` へ移動した。必要な `pub(in crate::runner)` qualifier を
除去すると、moved lines 1106-1121 は hash
`700b2283f7a6ea7b61c97ec59a27166404a72eccdce8f8e7aa7c681dd9003e47` を維持する。
shared core は parent-only。最初の non-test build で、parenthesized core の移動後は
direct binary detail-collector alias も test-only になったことを検出したため、paired
inventory を精密化し、その alias と direct parenthesized validator/output-type alias
を `#[cfg(test)]` にした。payload-detail wrapper、config 8個、named detail/validator/
output wrapper、全 call site は `runner.rs` に保持する。

結果の `runner.rs` は10,395行、hash
`46338bc436d6fac02ed5ecd33ef454bed44e4ea8ed55427723e0781be0fadd44`、70行
`type_elaboration.rs` facade は hash
`720cecb3656838d7b2362db0c8c37a5fbc836d9e5b40e7713aa418ebe42b2576`、1,121行
`output.rs` は hash
`c07eec9a8e118462998ac9d99e0c983ed140bf1197c3bfd3125a0ed2a34c70c3`。
builder fallback は保持 caller に残り、validator-first rejection、設定済み fallback、
nested binary detail projection、left/right wrapper identity、fail-closed behavior は
moved core で不変。

focused parenthesized test 25件、unit test 272件、relevant crate integration test、
active parse 96件、declaration-symbol 4件、type-elaboration 188件は全成功。plan/count は
403/367、type coverage は235/223、pass/fail は219/184、warning/error は23/0 のまま。
raw/normalized test-list hash と4つの CLI byte hash は不変。format、warning deny の
all-target/all-feature Clippy、workspace test、diff cleanliness は成功。Task 263L は
完了し、fresh Task 263 inventory は残る payload-detail または config/named-wrapper
family を選ぶ。path、authority、behavior、coverage credit、owner crate、deferred
status は変わらないため Source Inventory と `spec_coverage_audit.md` は不変。

## Task 263M 移動前 Inventory と仕様

fresh dependency inventory は `runner.rs:5508-5524` の正確な parenthesized-binary
payload-detail wrapper（17行、hash
`5807184d2ce9cfa8f7fb5a9be4d8401b8a538a335f28d07768a251840a169605`）を選ぶ。
production route wrapper 8個と `reserved_binary.rs` の direct assertion 8個がこの
entry を呼ぶ。

Task 263M はこの wrapper だけを既存 private `type_elaboration/output.rs` へ機械的に
移動する。保持 route/test caller のため parent-only となる。builder と shared output-
detail core は production で leaf-internal になるため、facade/runner alias は test-only
となる。config 8個、全 named route/detail/validator/output wrapper、named extractor、
全 call site は `runner.rs` に保持する。

これは move-only `design_drift` であり、Task 263M0 prerequisite は不要。既存 active/
direct matrix は builder-error fallback、設定済み invalid-key selection、validator-first
rejection、nested binary diagnostic、side identity、fail-closed behavior を cover する。
preservation matrix は必要な visibility を除く exact function-body equivalence、272 test
list と active/CLI byte 不変、`.miz`、expectation、trace、spec、API、config、wrapper、
extractor、call-site edit なしとする。

authority inventory に canonical contradiction はない。既存 `output.rs` path/owner は
不変で、authority、behavior、coverage credit、deferred status は変わらないため、
Source Inventory と `spec_coverage_audit.md` は変更しない。

## Task 263M Move Result

Task 263M は正確な parenthesized-binary payload-detail wrapper だけを既存 private
`type_elaboration/output.rs` へ移動した。必要な `pub(in crate::runner)` qualifier を
除去すると、moved lines 1123-1139 は hash
`5807184d2ce9cfa8f7fb5a9be4d8401b8a538a335f28d07768a251840a169605` を維持する。
wrapper は parent-only。direct parenthesized builder/shared-detail alias は test-only と
なり、config、named route/detail/validator/output wrapper、named extractor、全 call site
は `runner.rs` に保持する。

結果の `runner.rs` は10,377行、hash
`ee5ce9753442a91cea9642c32941f0bda71f05c956ad13b49d36d90d17639e35`、71行
`type_elaboration.rs` facade は hash
`62bd63a6aaaac7fbf83f8783b90bfa4546dfab99308b4ff420fd66803ebc9678`、1,139行
`output.rs` は hash
`55fd0eae01f417d011a3800d532f65eed1a2fd76d60d7387e9630fe3d9c92e57`。
builder-error fallback、設定済み invalid-key selection、success delegation、nested
binary diagnostic、side identity、fail-closed behavior は不変。

focused parenthesized test 25件、unit test 272件、relevant crate integration test、
active parse 96件、declaration-symbol 4件、type-elaboration 188件は全成功。plan/count は
403/367、type coverage は235/223、pass/fail は219/184、warning/error は23/0 のまま。
raw/normalized test-list hash と4つの CLI byte hash は不変。format、warning deny の
all-target/all-feature Clippy、workspace test、diff cleanliness は成功。Task 263M は
完了し、fresh Task 263 inventory は残る config/named-wrapper family を選ぶ。path、
authority、behavior、coverage credit、owner crate、deferred status は変わらないため
Source Inventory と `spec_coverage_audit.md` は不変。

## Task 263N 移動前 Inventory と仕様

fresh dependency inventory は `runner.rs` の正確な7 fragmentからなる cohesive private
parenthesized route owner を選ぶ。invalid key 220-235（16行、`f0a67ec1...`）、config
8個 3099-3298（200、`d374247d...`）、production detail route 8個 5374-5506
（133、`683e4c79...`）、test-only named detail wrapper 6960-7046（87、
`08f628be...`）、output wrapper 7058-7142（85、`9139389e...`）、validator wrapper
8322-8408（87、`87d26ecb...`）、source extractor 8個 8819-8930（112、
`95dce665...`）で合計720行。

Task 263N はこれらだけを新規 private `type_elaboration/parenthesized_routes.rs` へ機械的に
移動する。config と thin source/detail/test wrapper を同じownerに置き、source/output
ownershipの逆依存を避ける。normal phase facadeを跨ぐのはproduction detail route 8個
だけで、configとtest消費wrapper/extractorは`#[cfg(test)]`で跨ぐ。call site、name、
config value、key、payload、ordering、fallback、fail-closed behaviorは変更しない。
既存active routeと`reserved_binary.rs`/`binary_route_fixtures.rs`のmatrixをoracleとする。

これはmove-only `design_drift`でN0は不要。新規real pathはmoveと同時にpaired Source
Inventoryへ追加し、paired target layoutには追加済みとする。authority、behavior、
coverage credit、owner crate、deferred statusは
変わらないため`spec_coverage_audit.md`は不変。

## Task 263N 移動結果

移動した7 fragment は normalized hash `f0a67ec1...`、`d374247d...`、
`683e4c79...`、`08f628be...`、`9139389e...`、`87d26ecb...`、
`95dce665...` を正確に維持し、combined normalized hash は `93a45180...`。
新規 private owner は745行/raw hash `490cc42b...`、`runner.rs` は9,721行/raw
hash `9cb5f972...`。invalid-key constant は leaf-private のまま。既存 runner test
名はconfig由来test-only aliasで解決し、config、named test wrapper 24個、extractor
8個はtest facadeだけを跨ぐ。normal facadeがexposeするのはproduction detail route
8個だけ。

focused parenthesized test 25個とcrate unit test 272個は全成功。raw/normalized
272-name list hash、4 CLI byte hash、active count 96/4/188、plan 403/367、type
coverage 235/223、pass/fail 219/184、warning 23/error 0は不変。format、warning deny
all-target/all-feature Clippy、workspace test、diff cleanlinessは成功。Task 263Nは完了し、
fresh Task 263 inventoryは残るnon-parenthesized route-owner familyへ戻る。paired Source
Inventoryは新しいreal pathを含み、authority、behavior、coverage credit、owner crate、
deferred statusは変わらないため`spec_coverage_audit.md`は不変。

## Task 263O 移動前 Inventory と仕様

fresh dependency inventory はleading direct-binary ownerを選ぶ。対象はreserved-variable
equality、reserved-object-variable equality/inequality、distinct reserved-object-variable
equality/inequality、distinct reserved-variable equality/membership/inequality、heterogeneous
reserve membership。`runner.rs` の正確な8 fragment、invalid key 6個 150-161（12行、
`d3c61a92...`）、invalid key 3個 287-292（6、`6c3ab931...`）、先頭config 5個
3131-3244（114、`aca11227...`）、distinct config 3個 3287-3359（73、
`7febfe4a...`）、heterogeneous config 3484-3507（24、`abe7d7f1...`）、production
detail route 9個 5214-5322（109、`3d564030...`）、test-only output wrapper 9個
6678-6768（91、`475ab5d7...`）、source extractor 9個 8262-8378（117、
`5499a8cb...`）で合計546行、combined hash `f2271cc0...`。

Task 263O はこれらだけを新規 private `type_elaboration/binary_routes.rs` へ機械的に
移動する。leaf は既存 `source_formula` config/extractor substrate と `output`
builder/detail coreを直接consumeし、両siblingからnew leafへの依存はないためacyclic。
normal phase facadeを跨ぐのはproduction detail route 9個だけ。config、test消費output、
extractorは`#[cfg(test)]`だけを跨ぎ、invalid-key constantはleaf-privateのまま、config由来
runner test aliasで既存name/valueを維持する。call site、name、config value、key、payload、
ordering、fallback、fail-closed behaviorは変更しない。

`reserved_binary.rs`、`binary_route_fixtures.rs`、`reserve_fixtures.rs`、
`reserve_object_fixtures.rs`、shared test supportの既存direct occurrence 187件
（output/extractor reference 162件とinvalid-key reference 25件）がsource exactness、
checker payload、invalid-key fallback、active real fixture、route isolationを覆う。よって
move-only `design_drift`でO0 test taskは不要。new real pathはmoveと同時にpaired Source
Inventory/target layoutへ追加する。authority、behavior、coverage credit、owner crate、
deferred statusは変わらないため`spec_coverage_audit.md`は不変。route-set expansion、
direct-family regrouping、config/key/role edit、wrapper generalization、assertion weakening、
test/expectation edit、later binary/type-assertion/formula route moveは禁止。

## Task 263O 移動結果

移動した8 fragment はoriginal raw hash `d3c61a92...`、`6c3ab931...`、
`aca11227...`、`7febfe4a...`、`abe7d7f1...`、`3d564030...`、`475ab5d7...`、
`5499a8cb...` をpre-move oracleとして維持する。必要なrunner visibilityとformat空白だけを
除くと全old/new fragment pairはtoken-identicalで、combined normalized hashは
`86bf7cad...`。新規private ownerは559行/raw hash `c4546956...`、`runner.rs`は
9,234行/raw hash `8a55c57d...`。invalid-key constantはleaf-privateのまま。既存runner
test名はconfig由来test-only aliasで解決し、config、test output wrapper 9個、extractor
9個はtest facadeだけを跨ぐ。normal facadeがexposeするのはproduction detail route
9個だけ。

selected-family focused filterとcrate unit test 272個は全成功。raw/normalized 272-name
list hash、4 CLI byte hash、active count 96/4/188、plan 403/367、type coverage
235/223、pass/fail 219/184、warning 23/error 0は不変。format、warning deny
all-target/all-feature Clippy、workspace test、diff cleanlinessは成功。Task 263Oは完了し、
fresh Task 263 inventoryは後続direct-binary route-owner familyへ戻る。paired Source
Inventory/target layoutは新しいreal pathを含み、authority、behavior、coverage credit、
owner crate、deferred statusは変わらないため`spec_coverage_audit.md`は不変。

## Task 263P 移動前 Inventory と仕様

fresh dependency inventoryはmultiple-reserve declaration binary route 5個、set
equality/inequality/membershipとobject equality/inequalityを選ぶ。`runner.rs`の正確な
5 fragment、invalid key 324-333（10行、`c1091c1b...`）、config 5個 3214-3337
（124、`85224887...`）、production detail route 5個 5522-5583（62、
`518d4e55...`）、test-only output wrapper 5個 6805-6856（52、`1af7a5ab...`）、
source extractor 5個 8360-8424（65、`55bb8ec4...`）で合計313行、combined
hash `790eba84...`。

Task 263Pはこれらだけを既存private `type_elaboration/binary_routes.rs`へ機械的に
追記する。dependencyはTask 263Oの`source_formula`/`output`へのacyclic graphを維持。
normal phase facadeを跨ぐのはproduction detail route 5個だけ。config、test消費output、
extractorは`#[cfg(test)]`だけを跨ぎ、invalid-key constantはleaf-privateのまま、config由来
runner test aliasで既存name/valueを維持する。call site、name、config value、key、payload、
ordering、fallback、fail-closed behaviorは変更しない。

`reserved_binary.rs`、`binary_route_fixtures.rs`、`reserve_fixtures.rs`、shared test
supportの既存direct occurrence 104件（output/extractor reference 96件、invalid-key
reference 8件）がsource exactness、checker payload、invalid-key fallback、active real
fixture、route isolationを覆う。よってmove-only `design_drift`でP0 test taskは不要。
new source pathはなく、paired target layoutはowner拡張を記録する。authority、behavior、
coverage credit、owner crate、deferred statusは変わらないため`spec_coverage_audit.md`は
不変。route-set expansion、config/key/role edit、wrapper generalization、assertion
weakening、test/expectation edit、base/mode-chain/type-assertion/formula route moveは禁止。

## Task 263P 移動結果

移動した5 fragmentは訂正済みoriginal raw hash `c1091c1b...`、`85224887...`、
`518d4e55...`、`1af7a5ab...`、`55bb8ec4...`をpre-move oracleとして維持する。
最初のcompile-mode verificationでdraftのoutput範囲6805-6857が次の未移動routeの
`#[cfg(test)]`を含むと判明したため、完了前に6805-6856へ訂正し、attributeを元routeへ
戻し、最初の移動extractorをnormal leaf visibilityへ戻した。必要なrunner visibilityと
format空白だけを除くと全訂正済みold/new fragment pairはtoken-identicalで、combined
normalized hashは`340d2658...`。

拡張後private ownerは872行/raw hash `883042d7...`、`runner.rs`は8,956行/raw hash
`48ba9d05...`。invalid-key constantはleaf-privateのまま。既存runner test名はconfig由来
test-only aliasで解決し、config、test output wrapper 5個、extractor 5個はtest facade
だけを跨ぐ。normal facadeが追加するのはproduction detail route 5個だけで、phaseは
private leaf 9個を維持する。

focused multiple-reserve test 10個とcrate unit test 272個は全成功。raw/normalized
272-name list hash、4 CLI byte hash、active count 96/4/188、plan 403/367、type
coverage 235/223、pass/fail 219/184、warning 23/error 0は不変。format、warning deny
all-target/all-feature Clippy、workspace test、diff cleanlinessは成功。Task 263Pは完了し、
fresh Task 263 inventoryは残るbase/mode-chain binary route-owner familyへ戻る。new source
pathはなく、authority、behavior、coverage credit、owner crate、deferred statusは
変わらないため`spec_coverage_audit.md`は不変。

## Task 263Q 移動前 Inventory と仕様

fresh dependency inventoryはbase reserved-variable membership/inequality binary routeを
選ぶ。`runner.rs`の正確な5 fragment、invalid key 361-364（4行、`5d41a022...`）、
config 2個 3197-3238（42、`aa8213c1...`）、production detail route 2個
5423-5446（24、`81da3344...`）、test-only output wrapper 2個 6644-6663
（20、`ae5f0131...`）、source extractor 2個 8147-8172（26、`1b44be5a...`）
で合計116行、combined hash `ec7a766a...`。

Task 263Qはこれらだけを既存private `type_elaboration/binary_routes.rs`へ機械的に
追記する。dependencyはTasks 263O-263Pの`source_formula`/`output`へのacyclic graphを
維持。normal phase facadeを跨ぐのはproduction detail route 2個だけ。config、test消費
output、extractorは`#[cfg(test)]`だけを跨ぎ、invalid-key constantはleaf-privateのまま、
config由来runner test aliasで既存name/valueを維持する。call site、name、config value、
key、payload、ordering、fallback、fail-closed behaviorは変更しない。

`reserved_direct.rs`、`reserved_binary.rs`、`binary_route_fixtures.rs`、
`asserted_head_fixtures.rs`、shared test supportの既存direct occurrence 38件
（output/extractor reference 34件、invalid-key reference 4件）がsource exactness、checker
payload、invalid-key fallback、active real fixture、route isolationを覆う。よってmove-only
`design_drift`でQ0 test taskは不要。new source pathはなく、paired target layoutはowner
拡張を記録する。authority、behavior、coverage credit、owner crate、deferred statusは
変わらないため`spec_coverage_audit.md`は不変。route-set expansion、config/key/role
edit、wrapper generalization、assertion weakening、test/expectation edit、mode-chain/
type-assertion/formula route moveは禁止。

## Task 263Q 移動結果

Task 263Qは承認済みの5 fragment、合計116行だけを既存private
`type_elaboration/binary_routes.rs`へ移動し、original raw hash
`5d41a022...`、`aa8213c1...`、`81da3344...`、`ae5f0131...`、
`1b44be5a...`をpre-move oracleとして維持した。必要なrunner visibilityとformat空白
だけを除くと全old/new fragment pairはtoken-identicalで、combined normalized hashは
`e8c45cf1...`。次のtype-assertion output attribute/bodyと次のextractorは`runner.rs`に
残り、後続routeはboundaryを跨いでいない。

拡張後private ownerは988行/raw hash `087967cc...`、`runner.rs`は8,851行/raw hash
`a039be76...`。invalid-key constantはleaf-privateのまま。既存runner test名はconfig由来
test-only aliasで解決し、config、test output wrapper 2個、extractor 2個はtest facade
だけを跨ぐ。normal facadeが追加するのはproduction detail route 2個だけで、phaseは
private leaf 9個を維持し、dependency graphはacyclicのまま。

membership/inequalityのfocused filter 2本はそれぞれ33件/31件成功し、crate unit test
272件も全成功。raw/normalized 272-name list hash、4 CLI byte hash、active count
96/4/188、plan 403/367、type coverage 235/223、pass/fail 219/184、warning 23/error 0は
不変。format、warning deny all-target/all-feature Clippy、workspace test、diff cleanlinessは
成功。Task 263Qは完了し、fresh Task 263 inventoryは残るmode-chain binary route-owner
familyへ戻る。new source pathはなく、authority、behavior、coverage credit、owner crate、
deferred statusは変わらないため`spec_coverage_audit.md`は不変。

## Task 263R 移動前 Inventory と仕様

fresh dependency inventoryはdirect local-mode membership/equality/inequality binary routeを
次のbounded familyとして選ぶ。`runner.rs`の正確な10 fragment、membership invalid key
255-256（2行、`c4db5ce6...`）、equality/inequality invalid key 276-279（4、
`70a954f2...`）、membership config 3204-3231（28、`77ebd7a7...`）、
equality/inequality config 3626-3675（50、`81a2369d...`）、membership production
detail route 4910-4920（11、`6545f96f...`）、equality/inequality detail route
5043-5065（23、`74305b0b...`）、membership test-only output 6179-6187（9、
`a0c62cc0...`）、equality/inequality test-only output 6292-6310（19、
`0367ba53...`）、membership extractor 7600-7611（12、`508569dd...`）、
equality/inequality extractor 7730-7754（25、`c1e52d0c...`）で合計183行、
combined raw hash `16bcea2e...`。

Task 263Rはこれらだけを既存private `type_elaboration/binary_routes.rs`へ機械的に
追記する。leafは既存`source_formula`/`output` ownerだけに依存し続ける。normal phase
facadeを跨ぐのはproduction detail route 3個だけ。config、test消費output、extractorは
`#[cfg(test)]`だけを跨ぎ、invalid-key constantはleaf-privateのまま、config由来runner
test aliasで既存name/valueを維持する。call site、name、config value、mode-definition
chain、key、payload、ordering、fallback、fail-closed behaviorは変更しない。

`support.rs`、`binary_route_fixtures.rs`、`mode_chain.rs`、
`mode_chain_fixtures.rs`の既存direct test occurrence 52件がexact real source expansion、
checker payload、invalid-key fallback、active fixture、route isolationを覆う。active
`.miz`/expectation pair 3組とcovered trace requirementがcanonical reserved variableとatomic
equality/inequality/membership intentおよびexact source-derived checker seamを直接保持する。
よってmove-only `design_drift`でR0 test taskは不要。new source pathはなく、paired target
layoutはowner拡張を記録する。authority、behavior、coverage credit、owner crate、deferred
statusは変わらないため`spec_coverage_audit.md`は不変。route-set expansion、config/key/
role/mode edit、wrapper generalization、assertion weakening、test/expectation edit、object-mode/
deeper-chain/type-assertion/formula route moveは禁止。

## Task 263R 移動結果

Task 263Rは承認済みの10 fragment、合計183行だけを既存private
`type_elaboration/binary_routes.rs`へ移動し、original raw hash `c4db5ce6...`、
`70a954f2...`、`77ebd7a7...`、`81a2369d...`、`6545f96f...`、
`74305b0b...`、`a0c62cc0...`、`0367ba53...`、`508569dd...`、
`c1e52d0c...`をpre-move oracleとして維持した。必要なrunner visibilityとformat空白
だけを除くと全old/new fragment pairはtoken-identicalで、combined normalized hashは
`be8e0e9b...`。隣接chained membership、local-object inequality、全deeper-chain config、
route、output attribute/body、extractorは`runner.rs`に残る。

拡張後private ownerは1,181行/raw hash `70feaa70...`、`runner.rs`は8,681行/raw hash
`7131c8b7...`。invalid-key constantはleaf-privateのまま。既存runner test名はconfig由来
test-only aliasで解決し、config、test output wrapper 3個、extractor 3個はtest facade
だけを跨ぐ。normal facadeが追加するのはproduction detail route 3個だけで、phaseは
private leaf 9個を維持し、dependency graphはacyclicのまま。

membership/equality/inequalityのfocused filterはそれぞれ10件/12件/10件成功し、crate
unit test 272件も全成功。raw/normalized 272-name list hash、4 CLI byte hash、active count
96/4/188、plan 403/367、type coverage 235/223、pass/fail 219/184、warning 23/error 0は
不変。format、warning deny all-target/all-feature Clippy、workspace test、diff cleanlinessは
成功。Task 263Rは完了し、fresh Task 263 inventoryは残るobject-mode/deeper-chain binary
route-owner familyへ戻る。new source pathはなく、authority、behavior、coverage credit、
owner crate、deferred statusは変わらないため`spec_coverage_audit.md`は不変。

## Task 263S 移動前 Inventory と仕様

fresh dependency inventoryはdirect local-object-mode membership/equality/inequality binary
routeを次のbounded familyとして選ぶ。`runner.rs`の正確な10 fragment、membership/
inequality invalid key 293-296（4行、`0c2d5a85...`）、equality invalid key 383-384
（2、`759fc61a...`）、membership/inequality config 3583-3636（54、
`bf587e0d...`）、equality config 4819-4843（25、`ff4ef313...`）、membership/
inequality production detail route 4953-4977（25、`08141211...`）、equality detail
route 5274-5285（12、`7c4207cd...`）、membership/inequality test-only output
6170-6190（21、`d67627c1...`）、equality test-only output 6443-6452（10、
`1b1d490e...`）、membership/inequality extractor 7573-7597（25、`889aa420...`）、
equality extractor 7885-7896（12、`3cfd12b2...`）で合計190行、combined raw hash
`3e39b474...`。

Task 263Sはこれらだけを既存private `type_elaboration/binary_routes.rs`へ機械的に
追記する。leafは既存`source_formula`/`output` ownerだけに依存し続ける。normal phase
facadeを跨ぐのはproduction detail route 3個だけ。config、test消費output、extractorは
`#[cfg(test)]`だけを跨ぎ、invalid-key constantはleaf-privateのまま、config由来runner
test aliasで既存name/valueを維持する。call site、name、config value、mode-definition
chain、key、payload、ordering、fallback、fail-closed behaviorは変更しない。

`support.rs`、`binary_route_fixtures.rs`、`mode_chain.rs`、
`mode_chain_fixtures.rs`の既存direct test occurrence 52件（output/extractor reference 44件、
invalid-key reference 8件）がexact real source expansion、checker payload、invalid-key
fallback、active fixture、route isolationを覆う。active `.miz`/expectation pair 3組とcovered
trace requirementがcanonical reserved variableとatomic equality/inequality/membership intent
およびexact source-derived checker seamを直接保持する。よってmove-only `design_drift`で
S0 test taskは不要。new source pathはなく、paired target layoutはowner拡張を記録する。
authority、behavior、coverage credit、owner crate、deferred statusは変わらないため
`spec_coverage_audit.md`は不変。route-set expansion、config/key/role/mode edit、wrapper
generalization、assertion weakening、test/expectation edit、chained/deeper-chain/
type-assertion/formula route moveは禁止。

## Task 263S 移動結果

Task 263Sは承認済みの10 fragment、合計190行だけを既存private
`type_elaboration/binary_routes.rs`へ移動し、original raw hash `0c2d5a85...`、
`759fc61a...`、`bf587e0d...`、`ff4ef313...`、`08141211...`、
`7c4207cd...`、`d67627c1...`、`1b1d490e...`、`889aa420...`、
`3cfd12b2...`をpre-move oracleとして維持した。必要なrunner visibilityとformat空白
だけを除くと全old/new fragment pairはtoken-identicalで、combined normalized hashは
`e0590337...`。隣接chained familyと次のtype-assertion detail、output attribute/body、
extractorは`runner.rs`に残る。

拡張後private ownerは1,380行/raw hash `2b7e1aef...`、`runner.rs`は8,504行/raw hash
`f5080dee...`。invalid-key constantはleaf-privateのまま。既存runner test名はconfig由来
test-only aliasで解決し、config、test output wrapper 3個、extractor 3個はtest facade
だけを跨ぐ。normal facadeが追加するのはproduction detail route 3個だけで、phaseは
private leaf 9個を維持し、dependency graphはacyclicのまま。

membership/equality/inequalityのfocused filterは各10件成功し、crate unit test 272件も
全成功。raw/normalized 272-name list hash、4 CLI byte hash、active count 96/4/188、plan
403/367、type coverage 235/223、pass/fail 219/184、warning 23/error 0は不変。format、
warning deny all-target/all-feature Clippy、workspace test、diff cleanlinessは成功。Task
263Sは完了し、fresh Task 263 inventoryは残るchained/deeper-chain binary route-owner
familyへ戻る。new source pathはなく、authority、behavior、coverage credit、owner crate、
deferred statusは変わらないため`spec_coverage_audit.md`は不変。

## Task 263T 移動前 Inventory と仕様

fresh dependency inventoryはchained local-mode membership/equality/inequality binary routeを
次のbounded familyとして選ぶ。`runner.rs`の正確な14 fragment、membership invalid key
297-298（2行、`3547f56d...`）、equality/inequality invalid key 314-317（4、
`a33a4243...`）、membership/equality/inequality config 3234-3268（35、
`9266cead...`）、3598-3629（32、`ff54a0ed...`）、4672-4703（32、
`b624f397...`）、production detail route 3個 4779-4790（12、`77d10775...`）、
4887-4898（12、`fd4ddd74...`）、5142-5153（12、`603f4e69...`）、test-only
output 3個 5973-5982（10、`5214fdac...`）、6065-6074（10、`e26f53b0...`）、
6282-6291（10、`dd43dd7a...`）、source extractor 3個 7331-7342（12、
`77c8abb7...`）、7435-7446（12、`54f042db...`）、7682-7693（12、
`92c2a218...`）で合計207行、combined raw hash `dd7a8b0c...`。

Task 263Tはこれらだけを既存private `type_elaboration/binary_routes.rs`へ機械的に
追記する。leafは既存`source_formula`/`output` ownerだけに依存し続ける。normal phase
facadeを跨ぐのはproduction detail route 3個だけ。config、test消費output、extractorは
`#[cfg(test)]`だけを跨ぎ、invalid-key constantはleaf-privateのまま、config由来runner
test aliasで既存name/valueを維持する。call site、name、config value、2-definition chain、
key、payload、ordering、fallback、fail-closed behaviorは変更しない。

`support.rs`、`binary_route_fixtures.rs`、`mode_chain.rs`、
`mode_chain_fixtures.rs`の既存direct test occurrence 50件（output/extractor reference 43件、
invalid-key reference 7件）がexact real two-expansion source chain、checker payload、
invalid-key fallback、active fixture、route isolationを覆う。active `.miz`/expectation pair
3組とcovered trace requirementがcanonical reserved-variable atomic-formula intentとexact
source-derived checker seamを保持する。よってmove-only `design_drift`でT0 test taskは
不要。new source pathはなく、paired target layoutはowner拡張を記録する。authority、
behavior、coverage credit、owner crate、deferred statusは変わらないため
`spec_coverage_audit.md`は不変。route-set expansion、config/key/role/mode edit、chain
generalization、assertion weakening、test/expectation edit、chained object-mode/deeper-chain/
type-assertion/formula route moveは禁止。

## Task 263T 移動結果

Task 263Tは承認済みの14 fragment、合計207行だけを既存private
`type_elaboration/binary_routes.rs`へ移動し、original raw hash `3547f56d...`、
`a33a4243...`、`9266cead...`、`ff54a0ed...`、`b624f397...`、
`77d10775...`、`fd4ddd74...`、`603f4e69...`、`5214fdac...`、
`e26f53b0...`、`dd43dd7a...`、`77c8abb7...`、`54f042db...`、
`92c2a218...`をpre-move oracleとして維持した。必要なrunner visibilityとformat空白
だけを除くと全old/new fragment pairはtoken-identicalで、combined normalized hashは
`aa98a27d...`。隣接chained-object、two-/three-/deeper-edge、type-assertion、他route
familyは`runner.rs`に残る。

拡張後private ownerは1,600行/raw hash `03d9236d...`、`runner.rs`は8,306行/raw hash
`3f73039e...`。invalid-key constantはleaf-privateのまま。既存runner test名はconfig由来
test-only aliasで解決し、config、test output wrapper 3個、extractor 3個はtest facade
だけを跨ぐ。normal facadeが追加するのはproduction detail route 3個だけで、phaseは
private leaf 9個を維持し、dependency graphはacyclicのまま。

membership/equality/inequalityのfocused filterは各2件成功し、crate unit test 272件も
全成功。raw/normalized 272-name list hash、4 CLI byte hash、active count 96/4/188、plan
403/367、type coverage 235/223、pass/fail 219/184、warning 23/error 0は不変。format、
warning deny all-target/all-feature Clippy、workspace test、diff cleanlinessは成功。Task
263Tは完了し、fresh Task 263 inventoryは残るchained object-mode/deeper-chain binary
route-owner familyへ戻る。new source pathはなく、authority、behavior、coverage credit、
owner crate、deferred statusは変わらないため`spec_coverage_audit.md`は不変。

## Task 263U 移動前 Inventory と仕様

fresh dependency inventoryはchained local-object-mode membership/equality/inequality binary
routeを次のbounded familyとして選ぶ。`runner.rs`の正確な9 fragment、invalid key 3個
331-338（8行、`972beff3...`）、membership config 3537-3575（39、`71bb150f...`）、
equality/inequality config 4618-4690（73、`32f853aa...`）、membership production
detail route 4773-4785（13、`84c8bd3d...`）、equality/inequality detail route
5029-5054（26、`4fc8b564...`）、membership test-only output 5916-5926（11、
`5b884de2...`）、equality/inequality test-only output 6134-6155（22、
`7c165117...`）、membership extractor 7250-7261（12、`c84f51e1...`）、
equality/inequality extractor 7497-7521（25、`2240a58d...`）で合計229行、combined
raw hash `ae0066dd...`。

Task 263Uはこれらだけを既存private `type_elaboration/binary_routes.rs`へ機械的に
追記する。leafは既存`source_formula`/`output` ownerだけに依存し続ける。normal phase
facadeを跨ぐのはproduction detail route 3個だけ。config、test消費output、extractorは
`#[cfg(test)]`だけを跨ぎ、invalid-key constantはleaf-privateのまま、config由来runner
test aliasで既存name/valueを維持する。call site、name、config value、2-definition
object chain、key、payload、ordering、fallback、fail-closed behaviorは変更しない。

`support.rs`、`binary_route_fixtures.rs`、`mode_chain.rs`、
`mode_chain_fixtures.rs`の既存direct test occurrence 48件（output/extractor reference 41件、
invalid-key reference 7件）がexact real two-expansion object-mode source chain、checker
payload、invalid-key fallback、active fixture、route isolationを覆う。active `.miz`/
expectation pair 3組とcovered trace requirementがcanonical reserved-variable atomic-formula
intentとexact source-derived checker seamを保持する。よってmove-only `design_drift`でU0
test taskは不要。new source pathはなく、paired target layoutはowner拡張を記録する。
authority、behavior、coverage credit、owner crate、deferred statusは変わらないため
`spec_coverage_audit.md`は不変。route-set expansion、config/key/role/mode edit、chain
generalization、assertion weakening、test/expectation edit、deeper-chain/type-assertion/formula
route moveは禁止。

## Task 263U 移動結果

Task 263Uは承認済み9 fragment、合計229行だけを既存private
`type_elaboration/binary_routes.rs`へ移動し、移動前oracleのoriginal raw hash
`972beff3...`、`71bb150f...`、`32f853aa...`、`84c8bd3d...`、
`4fc8b564...`、`5b884de2...`、`7c165117...`、`c84f51e1...`、
`2240a58d...`を維持した。必要なrunner visibilityとformatting whitespaceだけを
除くと全old/new fragment pairはtoken-identicalで、combined normalized hashは
`a6b1bb6b...`。隣接するtwo-/three-/four-edge、long/deeper-chain、type-assertion、
formula、他route familyは`runner.rs`に残る。

拡張後private ownerは1,838行、raw hash `4e4c0125...`、`runner.rs`は8,090行、
raw hash `687c85be...`、phase facadeは235行、raw hash `8980cdd9...`。
invalid-key constantはleaf-privateのまま。既存runner test nameはconfig由来test-only
aliasで解決し、config、test output wrapper 3個、extractor 3個はtest facadeだけを
跨ぐ。normal facadeが追加するのはproduction detail route 3個だけで、phaseは
引き続きprivate leaf 9個を所有しdependency graphはacyclic。

focused membership/equality/inequality filterは各2 test、crate unit test 272件は全成功。
raw/normalized 272-name list hash、4 CLI byte hash、active count 96/4/188、plan
403/367、type coverage 235/223、pass/fail 219/184、warning/error 23/0は不変。
formatting、warning deny all-target/all-feature Clippy、workspace test、diff cleanlinessは
成功。Task 263Uは完了し、fresh Task 263 inventoryはdeeper-chain binary route-owner
familyへ戻る。new source pathはなく、authority、behavior、coverage credit、owner crate、
deferred statusは変わらないため`spec_coverage_audit.md`は不変。

## Task 263V 移動前 Inventory と仕様

fresh dependency inventoryはtwo-edge local-mode membership/equality/inequality binary
routeを次のbounded familyとして選ぶ。`runner.rs`の正確な15 fragment、membership/
equality/inequality invalid key 339-340（2行、`f02fb8e4...`）、352-353（2、
`ac20181b...`）、422-423（2、`a40e0c6f...`）、config 3262-3301（40、
`54b49166...`）、3550-3586（37、`0694dde7...`）、4469-4505（37、
`30030132...`）、production detail route 4591-4602（12、`bc4a798e...`）、
4672-4683（12、`b6bb868b...`）、4874-4885（12、`815c915b...`）、test-only
output 5705-5714（10、`d4bb53d3...`）、5774-5783（10、`65190120...`）、
5946-5955（10、`99a8c9c1...`）、source extractor 6995-7006（12、
`a17900f5...`）、7073-7084（12、`f77cfcd9...`）、7268-7279（12、
`fbe87d76...`）で合計222行、combined raw hash `f680fb91...`。

Task 263Vはこれらだけを既存private `type_elaboration/binary_routes.rs`へ機械的に
追記する。leafは既存`source_formula`/`output` ownerだけに依存し続ける。normal phase
facadeを跨ぐのはproduction detail route 3個だけ。config、test消費output、extractorは
`#[cfg(test)]`だけを跨ぎ、invalid-key constantはleaf-privateのまま、config由来runner
test aliasで既存name/valueを維持する。call site、name、config value、正確な
3-definition chain、key、payload、ordering、fallback、fail-closed behaviorは変更しない。

`support.rs`、`binary_route_fixtures.rs`、`mode_chain.rs`、
`mode_chain_fixtures.rs`、`reserved_binary.rs`の既存direct test occurrence 54件
（output/extractor reference 46件、invalid-key reference 7件、config reference 1件）が
exact real three-expansion source chain、checker payload、invalid-key fallback、active
fixture、cross-route isolation、direct/parenthesized owner boundaryを覆う。active `.miz`/
expectation pair 3組とcovered trace requirementがcanonical reserved-variable atomic-formula
intentとexact source-derived checker seamを保持する。よってmove-only `design_drift`でV0
test taskは不要。new source pathはなく、paired target layoutはowner拡張を記録する。
authority、behavior、coverage credit、owner crate、deferred statusは変わらないため
`spec_coverage_audit.md`は不変。route-set expansion、config/key/role/mode edit、chain
generalization、assertion weakening、test/expectation edit、object-mode、three-/four-edge、
long-chain、type-assertion、formula route moveは禁止。

## Task 263V 移動結果

Task 263Vは承認済み15 fragment、合計222行だけを既存private
`type_elaboration/binary_routes.rs`へ移動し、移動前oracleのoriginal raw hash
`f02fb8e4...`、`ac20181b...`、`a40e0c6f...`、`54b49166...`、
`0694dde7...`、`30030132...`、`bc4a798e...`、`b6bb868b...`、
`815c915b...`、`d4bb53d3...`、`65190120...`、`99a8c9c1...`、
`a17900f5...`、`f77cfcd9...`、`fbe87d76...`を維持した。必要なrunner
visibilityとformatting whitespaceだけを除くと全old/new fragment pairはtoken-identical
で、combined normalized hashは`53865bd6...`。隣接するobject-mode、three-/four-edge、
long-chain、type-assertion、formula、他route familyは`runner.rs`に残る。

拡張後private ownerは2,073行、raw hash `024f1b74...`、`runner.rs`は7,877行、
raw hash `e609ff69...`、phase facadeは247行、raw hash `8d12176a...`。
invalid-key constantはleaf-privateのまま。既存runner test nameと保持direct
parenthesized corruption consumerはconfig由来test-only aliasで解決する。config、test
output wrapper 3個、extractor 3個はtest facadeだけを跨ぎ、normal facadeが追加するのは
production detail route 3個だけ。phaseは引き続きprivate leaf 9個を所有しdependency
graphはacyclic。

focused membership/equality/inequality filterはparenthesized boundary test 2件を含め
2/4/2 test、crate unit test 272件は全成功。raw/normalized 272-name list hash、4 CLI byte
hash、active count 96/4/188、plan 403/367、type coverage 235/223、pass/fail 219/184、
warning/error 23/0は不変。formatting、warning deny all-target/all-feature Clippy、workspace
test、diff cleanlinessは成功。Task 263Vは完了し、fresh Task 263 inventoryはtwo-edge
object-modeと残るthree-/four-edge/long-chain binary route-owner familyへ戻る。new source
pathはなく、authority、behavior、coverage credit、owner crate、deferred statusは
変わらないため`spec_coverage_audit.md`は不変。

## Task 263W 移動前 Inventory と仕様

fresh dependency inventoryはtwo-edge local-object-mode membership/equality/inequality
binary routeを次のbounded familyとして選ぶ。`runner.rs`の正確な11 fragment、membership
invalid key 369-370（2行、`d572e286...`）、inequality/equality invalid key 439-442
（4、`d571dc2e...`）、config 3479-3522（44、`2f964b21...`）、4405-4447
（43、`9438d880...`）、4449-4487（39、`246a2852...`）、membership production
detail route 4543-4555（13、`7277fccf...`）、inequality/equality detail route
4746-4771（26、`041e760f...`）、membership test-only output 5610-5620（11、
`1231694c...`）、inequality/equality test-only output 5783-5804（22、
`2d5ae89e...`）、membership extractor 6873-6884（12、`82ab31ea...`）、
inequality/equality extractor 7068-7092（25、`63fa9c8a...`）で合計241行、combined
raw hash `a57c6acd...`。

Task 263Wはこれらだけを既存private `type_elaboration/binary_routes.rs`へ機械的に
追記する。leafは既存`source_formula`/`output` ownerだけに依存し続ける。normal phase
facadeを跨ぐのはproduction detail route 3個だけ。config、test消費output、extractorは
`#[cfg(test)]`だけを跨ぎ、invalid-key constantはleaf-privateのまま、config由来runner
test aliasで既存name/valueを維持する。call site、name、config value、正確な
3-definition object chain、key、payload、ordering、fallback、fail-closed behaviorは
変更しない。

`support.rs`、`binary_route_fixtures.rs`、`mode_chain.rs`、
`mode_chain_fixtures.rs`の既存direct test occurrence 50件（output/extractor reference
43件、invalid-key reference 7件）がexact real three-expansion object-mode source chain、
checker payload、invalid-key fallback、active fixture、cross-route isolation、object/set
identity boundaryを覆う。active `.miz`/expectation pair 3組とcovered trace requirementが
canonical reserved-variable atomic-formula intentとexact source-derived checker seamを保持
する。よってmove-only `design_drift`でW0 test taskは不要。new source pathはなく、
paired target layoutはowner拡張を記録する。authority、behavior、coverage credit、owner
crate、deferred statusは変わらないため`spec_coverage_audit.md`は不変。route-set
expansion、config/key/role/mode edit、chain generalization、object/set coercion、assertion
weakening、test/expectation edit、three-/four-edge、long-chain、type-assertion、formula
route moveは禁止。

## Task 263W 移動結果

Task 263Wは承認済み11 fragment、合計241行だけを既存private
`type_elaboration/binary_routes.rs`へ移動し、移動前oracleのoriginal raw hash
`d572e286...`、`d571dc2e...`、`2f964b21...`、`9438d880...`、
`246a2852...`、`7277fccf...`、`041e760f...`、`1231694c...`、
`2d5ae89e...`、`82ab31ea...`、`63fa9c8a...`を維持した。必要なrunner
visibilityとformatting whitespaceだけを除くと全old/new fragment pairはtoken-identical
で、combined normalized hashは`0e58ae98...`。隣接するthree-/four-edge、long-chain、
type-assertion、formula、他route familyは`runner.rs`に残る。

拡張後private ownerは2,324行、raw hash `9ef34cf7...`、`runner.rs`は7,649行、
raw hash `394ebbe8...`、phase facadeは259行、raw hash `361f6e9c...`。
invalid-key constantはleaf-privateのまま。既存runner test nameはconfig由来test-only
aliasで解決し、config、test output wrapper 3個、extractor 3個はtest facadeだけを跨ぐ。
normal facadeが追加するのはproduction detail route 3個だけで、phaseは引き続きprivate
leaf 9個を所有しdependency graphはacyclic。

focused membership/equality/inequality filterは各2 test、crate unit test 272件は全成功。
raw/normalized 272-name list hash、4 CLI byte hash、active count 96/4/188、plan
403/367、type coverage 235/223、pass/fail 219/184、warning/error 23/0は不変。
formatting、warning deny all-target/all-feature Clippy、workspace test、diff cleanlinessは
成功。Task 263Wは完了し、fresh Task 263 inventoryは残るthree-/four-edge/long-chain
binary route-owner familyへ戻る。new source pathはなく、authority、behavior、coverage
credit、owner crate、deferred statusは変わらないため`spec_coverage_audit.md`は不変。

Tasks 262N0-262Q は authority、behavior、coverage credit、owner crate、deferred
status を維持するため `spec_coverage_audit.md` は変更しない。accepted-shape expansion、
route generalization、config/result-role edit、payload/detail/diagnostic/order change、
assertion weakening、test deletion/ignore、checker/output move を禁止する。

## Current Ownership

| Current area | Responsibility | Dependency direction | Audit decision |
|---|---|---|---|
| public report/result/status type と `run_*_corpus` function | stable public runner facade と corpus-level orchestration | plan/discovery から phase execution | `runner.rs` に残す。 |
| source/frontend と resolver staging | source package preparation/cleanup、root/path/snapshot identity、frontend execution/result transport、common frontend diagnostic projection、resolver shell/projection/symbol collection | 適用範囲で parse、declaration-symbol、type-elaboration が共有 | frontend staging は Task 258、declaration/type resolver collection は Task 260A、common frontend diagnostic projection は Task 263B で最小 parent-only visibility の private `shared.rs` へ移動済み。 |
| active-case admission と stable failure assembly | tag/phase gate、expected-output matching、deterministic failure diagnostic | phase-specific facade-to-owner transition | Tasks 259/260B で parse-only/declaration case/failure boundary は移動済み。Task 263C で type expected-key/failure projection を private `type_elaboration/result.rs`、Task 263D で type tag/runnable/gate admission を private `type_elaboration/admission.rs` へ移動し、type case execution/actual-detail dispatch は fresh Task 263 inventory のため `runner.rs` に保持。 |
| parse-only execution | Surface-AST snapshot と parse-only failure projection | shared frontend から parse-only result | Task 259 で最小 parent-only visibility の private `parse_only.rs` へ移動済み。 |
| fixture import provider | parser fixture lexical summary と type import-summary adapter | active phase が共有する parser/frontend seam | Task 261 で private `import_fixtures.rs` へ移動済み。後段 phase は同じ provider/adapter path を維持。 |
| declaration-symbol observation | shared resolver result を consume し、deterministic payload、expected-value、failure projection を組み立てる | shared resolver output から declaration-symbol result | Task 260B で private `declaration_symbol.rs` へ移動済み。既存 integration test は `tests/metadata.rs` に残す。 |
| type-elaboration admission/execution | lower-stage fail-closed gate と checker/core handoff dispatch | resolver output から source bridge | Task 263A で generic checker-handoff assembly/validation を private `checker_handoff.rs`、Task 263C で expected-key/failure projection を private `result.rs`、Task 263D で active admission を private `admission.rs`、Tasks 263E-263F で checker-output transport/builder、Tasks 263G-263I で type-assertion/binary/shared-parenthesized validation、Tasks 263J-263M で type-assertion/binary/parenthesized detail/payload-detail core を private `output.rs`、Task 263Nでcohesive parenthesized route ownerをprivate `parenthesized_routes.rs`、Tasks 263O-263Wでleading/multiple-reserve/base membership/inequality/direct local-mode/direct local-object-mode/chained local-mode/chained local-object-mode/two-edge local-mode/two-edge local-object-mode binary route ownerをprivate `binary_routes.rs`へ移動。top-level case execution/dispatch、three-/four-edge/long-chain binary、type-assertion/formula config/named wrapper、他のoutput consumerは`runner.rs`に残り、phase facadeはprivate leaf 9個を所有する。 |
| source extraction | exact source-shape recognition と real AST/resolver payload construction | syntax/resolver input から checker input | Tasks 262A-262B で common source-AST primitive/projection、Task 262D で shared exact fixture-import projection を private `type_elaboration/source_ast.rs`、Tasks 262C/262E で reserve type-expression/symbol projection、declaration segmentation、local-mode expansion を private `type_elaboration/source_reserve.rs`、Tasks 262F-262Q で standalone formula constant、shared exact numeral、builtin binary/type-assertion formula、shared imported-formula symbol resolver/provenance pair、imported predicate/functor、imported attribute assertion、set-enumeration、connective/quantifier family、shared/direct-binary/parenthesized/type-assertion reserved-variable source substrate を private `type_elaboration/source_formula.rs` へ移動済み。formula source extractionは完了し、Tasks 263N-263Wでparenthesized/leading direct-binary/multiple-reserve/base membership/inequality/direct local-mode/direct local-object-mode/chained local-mode/chained local-object-mode/two-edge local-mode/two-edge local-object-mode named extractorをroute ownerと同居させた。three-/four-edge、long-chain、later route config/wrapperとchecker/output consumerはTask 263 inventoryのため`runner.rs`に残す。 |
| payload validation と detail-key rendering | exact checker/core output validation、expected/actual matching、deterministic key、diagnostic | source bridge output から runner result | Tasks 263E-263I で shared output transport/builder 3個と type-assertion/binary/shared-parenthesized validator/private helper を private `type_elaboration/output.rs` へ移動。Tasks 263J-263Mでtype-assertion/binary/shared parenthesized result/detailとpayload-detail core、Tasks 263N-263Wでparenthesized/leading direct-binary/multiple-reserve/base membership/inequality/direct local-mode/direct local-object-mode/chained local-mode/chained local-object-mode/two-edge local-mode/two-edge local-object-mode configとnamed detail/output wrapperを各route leafへ移動。later three-/four-edge、long-chain named wrapper/configは後続bounded workに残す。key/order は編集しない。 |
| fixture builder と corruption probe | AST/env/sidecar builder と finite negative matrix | test support から private production seam | private test support/fragment のみ。 |
| cross-owner isolation test | bidirectional route rejection と immutable/module guard | 全 supported source-bridge owner | cohesive fragment として保持して移す。 |

## Dependency Map

許可する dependency direction:

```text
public runner facade
  -> parse-only owner
     -> shared plan/admission/source/frontend/diagnostic staging
        -> fixture/import-summary owner (lexical provider)
  -> declaration-symbol owner
     -> shared plan/admission/source/frontend/diagnostic/resolver staging
        -> fixture/import-summary owner (lexical provider)
  -> type-elaboration owner
     -> active tag、runnable-admission、gate validation
     -> shared plan/admission/source/frontend/diagnostic/resolver staging
        -> fixture/import-summary owner (lexical provider)
     -> fixture/import-summary owner (resolver adapter)
     -> source extraction
        -> common source-AST primitives
           -> fixture/import-summary owner (module-path projection)
        -> reserve type-expression, declaration, and local-mode projection
        -> standalone formula-constant, shared exact numeral, builtin binary/type-assertion,
           shared imported-symbol, imported predicate/functor, imported attribute,
           set-enumeration、connective/quantifier、shared/direct-binary/parenthesized/
           type-assertion reserved-variable source projection
     -> checker-handoff assembly と readiness validation
     -> checker-output transport、builder、validation、type-assertion detail projection
     -> expected-result と failure projection
     -> checker/core payload validation と deterministic actual-detail keys

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
| `src/runner/shared.rs` | private source package preparation、frontend execution、common frontend diagnostic projection、admission support、真に cross-phase な helper。 |
| `src/runner/parse_only.rs` | parse-only case execution、snapshot、parse-only failure projection。 |
| `src/runner/declaration_symbol.rs` | declaration-symbol case execution、resolver observation、payload key、failure projection。 |
| `src/runner/import_fixtures.rs` | active phase が使う既存 parser fixture summary/adapter。 |
| `src/runner/type_elaboration.rs` と `src/runner/type_elaboration/` | type-elaboration orchestration と private source-extraction、checker-handoff、payload-validation/detail/diagnostic leaf。 |
| `src/runner/type_elaboration/binary_routes.rs` | leading、multiple-reserve declaration、base membership/inequality、direct local-mode、direct local-object-mode、chained local-mode、chained local-object-mode、two-edge local-mode、two-edge local-object-mode membership/equality/inequality binary config と thin source/detail/test route wrapper。 |
| `src/runner/type_elaboration/parenthesized_routes.rs` | cohesive parenthesized reserved-variable config と thin source/detail/test route wrapper。 |
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
| 262 | Tasks 262N0-262Q までで完了。inventory 済み type-elaboration formula source-extraction leaf をすべて移動し、checker/output consumer は Task 263 に残す。 |
| 262A | 完了: common exact source-AST primitive 5個を private type-elaboration phase facade 配下へ移動。 |
| 262B | 完了: shared node-kind traversal と qualified-symbol spelling projection を common source-AST leaf へ移動。 |
| 262C | 完了: reserve type-expression、visible symbol/admission、source-text projection を private source-reserve leaf へ移動し、declaration/mode caller は Task 262E のため保持。 |
| 262D | 完了: shared exact `parser.type_fixtures` import-item AST projection を formula/reserve caller より先に common source-AST leaf へ移動。 |
| 262E | 完了: bounded reserve declaration-segmentation/local-mode traversal/expansion family を移動し、handoff/formula ownership を保持して Task 262C temporary helper 3個を narrow。 |
| 262F | 完了: standalone `thesis`/`contradiction` formula-constant transport、exact extractor、dedicated node allowlist だけを new private source-formula leaf へ移動し、2 entry だけを facade alias にした。 |
| 262G | 完了: shared 3-helper exact numeral AST projection だけを private source-formula leaf へ移動し、5 caller family はすべて `runner.rs` に保持。 |
| 262H0 | 完了: production/test count 不変で既存 builtin-binary unit matrix に config order、payload provenance、recovery、duplicate、cardinality preservation を追加。 |
| 262H | 完了: builtin equality/inequality/membership config、source transport、exact extractor、dedicated allowlist だけを private source-formula leaf へ移動。 |
| 262I0 | 完了: production/test count 不変で既存 builtin type-assertion unit matrix に independently derived payload/checker provenance、recovery、duplicate、token-shape、cardinality preservation を追加。 |
| 262I | 完了: builtin type-assertion transport、exact extractor、dedicated allowlist だけを private source-formula leaf へ移動。 |
| 262J0 | 完了: production/test count 不変で既存 imported predicate/functor matrix に independent payload/checker/import provenance、recovery、duplicate、structural-cardinality preservation を追加。 |
| 262J1 | 完了: shared imported formula symbol resolver/provenance pair だけを3 caller 不変で private source-formula leaf へ移動。 |
| 262J2 | 完了: imported predicate/functor transport、exact extractor、exact infix projection、dedicated allowlist だけを private source-formula leaf へ移動。 |
| 262K0 | 完了: production/test count 不変で既存 imported attribute assertion variant 2個に independent five-field payload/provenance/order preservation と bounded direct-rejection corruption coverage を追加。 |
| 262K | 完了: imported attribute assertion transport、two-entry/shared extractor、dedicated allowlist だけを private source-formula leaf へ移動。 |
| 262L0 | 完了: production/test count 不変で既存 exact set-enumeration matrix に independent eight-field grouping/punctuation/order preservation と bounded direct-rejection corruption coverage を追加。 |
| 262L | 完了: set-enumeration transport、exact extractor、exact-set projection、dedicated allowlist だけを private source-formula leaf へ移動。 |
| 262M0 | 完了: production/test count 不変で既存 exact connective/quantifier matrix に independent ten-field binder/output/diagnostic preservation と bounded direct-rejection corruption coverage を追加。 |
| 262M | 完了: connective/quantifier transport、exact extractor、dedicated allowlist だけを private source-formula leaf へ移動。 |
| 262N0 | 完了: production/test count 不変で既存 exact reserved-variable equality matrix に全 config field、独立導出 formula/operand provenance、direct near-miss rejection、16 bounded default-off corruption を追加。 |
| 262N | 完了: 4 normalized shared reserved-variable config/model、predicate、mode/identifier、ordinal substrate fragment を最小 runner-scoped visibility で移動。 |
| 262O | 完了: direct reserved-variable binary source transport、generic extractor、family allowlist だけを private source-formula leaf へ移動。temporary allowlist alias は Task 262P の保持 parenthesized family 専用。 |
| 262P | 完了: parenthesized reserved-variable source enum/transport、generic extractor、single-parenthesized operand projection、family allowlist だけを移動し、binary allowlist 2個は両方 leaf-private。 |
| 262Q0 | 完了: test count/production 不変で既存 base reserved-variable type-assertion test に source field 10個、exact config、near miss 15件の direct rejection、bounded structural corruption 4個を追加。 |
| 262Q | 完了: Q0 後に reserved-variable type-assertion source transport、generic extractor、family allowlist だけを移動し、config/wrapper 58個と checker/output consumer を保持。 |
| 263 | 分割済み parent: checker-handoff、payload-validation、detail-key、expected-output、failure-diagnostic leaf を bounded dependency order で移動。 |
| 263A | 完了: 正確な506行 checker-handoff substrate を最小 runner-scoped visibility で private `type_elaboration/checker_handoff.rs` へ移動。 |
| 263B | 完了: 正確な49行 common frontend diagnostic projection を parent-only entry 3個で既存 private `shared.rs` へ移動。 |
| 263C | 完了: 正確な24行 expected-result/failure-projection family を parent-only entry 2個で private `type_elaboration/result.rs` へ移動し、exact-body/byte-stability を維持。 |
| 263D | 完了: 正確な4 fragment/50行 type active-admission gate を parent-only entry 2個で private `type_elaboration/admission.rs` へ移動し、exact-body/byte-stability を維持。 |
| 263E | 完了: 正確な33行3 transport checker-output substrate を runner-scoped field visibility で private `type_elaboration/output.rs` へ移動し、exact-body/byte-stability を維持。 |
| 263F | 完了: 正確な277行3 builder/output-projection producer family を parent-only builder entry 3個で既存 private `type_elaboration/output.rs` へ移動し、exact-body/byte-stability を維持。 |
| 263G | 完了: 正確な229行 type-assertion validator/private role helper/shared normalized-type predicate family だけを既存 private `type_elaboration/output.rs` へ移動。validator と一時 shared predicate は parent-only、role helper は leaf-private、全 preservation gate 成功。 |
| 263H | 完了: 正確な380行 binary-formula validator/source-projection/type-entry-helper family だけを既存 private `type_elaboration/output.rs` へ移動。validator だけ parent-only、全 helper は leaf-private、全 preservation gate 成功。 |
| 263I | 完了: 正確な67行 config-independent parenthesized-binary validator core だけを parent-only entry 1個で既存 private `type_elaboration/output.rs` へ移動。全 config、named wrapper、detail、call site を保持し、全 preservation gate 成功。 |
| 263J | 完了: 正確な46行 type-assertion result/detail core だけを既存 private `type_elaboration/output.rs` へ移動。result projection は parent-only、collector は leaf-private、direct validator/output alias は test-only、全 preservation gate 成功。 |
| 263K | 完了: 正確な36行 binary-formula result/detail core だけを既存 private `type_elaboration/output.rs` へ移動。両 entry は parent-only、direct validator/output alias は test-only、全 parenthesized/config/wrapper/call-site work を保持し、全 preservation gate 成功。 |
| 263L | 完了: 正確な16行 shared parenthesized-binary output-detail core だけを既存 private `type_elaboration/output.rs` へ移動。shared core は parent-only、direct parenthesized validator/output と binary detail-collector alias は test-only、全 payload/config/wrapper/call-site work を保持し、全 preservation gate 成功。 |
| 263M | 完了: 正確な17行 parenthesized-binary payload-detail wrapper だけを既存 private `type_elaboration/output.rs` へ移動。wrapper は parent-only、direct builder/shared-detail alias は test-only、全 config/named-wrapper/extractor/call-site work を保持し、全 preservation gate 成功。 |
| 263N | 完了: 正確な7 fragment/720行 parenthesized config/named-route family だけを新規 private `type_elaboration/parenthesized_routes.rs` へ移動。normal facade はdetail route 8個、test facadeは保持test consumerだけを exposeし、全 preservation gate成功。 |
| 263O | 完了: 正確な8 fragment/546行 leading direct-binary route family だけを新規 private `type_elaboration/binary_routes.rs` へ移動。normal facade はdetail route 9個、test facadeは保持test consumerだけを exposeし、全preservation gate成功。 |
| 263P | 完了: 訂正済み正確な5 fragment/313行 multiple-reserve declaration binary route family だけを既存 private `type_elaboration/binary_routes.rs` へ移動。normal facade はdetail route 5個、test facadeは保持test consumerだけを exposeし、全preservation gate成功。 |
| 263Q | 完了: 正確な5 fragment/116行 base reserved-variable membership/inequality route family だけを既存 private `type_elaboration/binary_routes.rs` へ移動。normal facade はdetail route 2個、test facadeは保持test consumerだけを exposeし、全preservation gate成功。 |
| 263R | 完了: 正確な10 fragment/183行 direct local-mode membership/equality/inequality route family だけを既存 private `type_elaboration/binary_routes.rs` へ移動。normal facade はdetail route 3個、test facadeは保持test consumerだけを exposeし、全preservation gate成功。 |
| 263S | 完了: 正確な10 fragment/190行 direct local-object-mode membership/equality/inequality route family だけを既存 private `type_elaboration/binary_routes.rs` へ移動。normal facade はdetail route 3個、test facadeは保持test consumerだけを exposeし、全preservation gate成功。 |
| 263T | 完了: 正確な14 fragment/207行 chained local-mode membership/equality/inequality route family だけを既存 private `type_elaboration/binary_routes.rs` へ移動。normal facade はdetail route 3個、test facadeは保持test consumerだけを exposeし、全preservation gate成功。 |
| 263U | 完了: 正確な9 fragment/229行 chained local-object-mode membership/equality/inequality route family だけを既存 private `type_elaboration/binary_routes.rs` へ移動。normal facade はdetail route 3個、test facadeは保持test consumerだけを expose。 |
| 263V | 完了: 正確な15 fragment/222行 two-edge local-mode membership/equality/inequality route family だけを既存 private `type_elaboration/binary_routes.rs` へ移動。normal facade はdetail route 3個、test facadeは保持test consumerだけを expose。 |
| 263W | 完了: 正確な11 fragment/241行 two-edge local-object-mode membership/equality/inequality route family だけを既存 private `type_elaboration/binary_routes.rs` へ移動。normal facade はdetail route 3個、test facadeは保持test consumerだけを expose。 |
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
| `test_gap` | Tasks 262H0/262I0/262J0/262K0/262L0/262M0/262N0/262Q0 で対応する move-only task 前の bounded preservation-matrix gap を修復する。behavior/coverage credit は不変。 |
| `spec_gap`、`source_drift`、`test_expectation_drift` | この series が導入または修復するものはない。 |
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
