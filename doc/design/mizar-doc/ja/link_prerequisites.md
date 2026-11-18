# Doc10 採用済み Producer と Allocation 前提 interface
正本は英語: [English companion](../en/link_prerequisites.md)。
状態: 前提 interface は採用済みで、[gate-1 import/closure input](#authenticated-ordered-imports-handoff) を別途供給する。[gate-2 direct-by input](#authenticated-direct-by-handoff) も別途供給する。検証済み allocation、診断、resolver 統合は引き続き gate で保留する。[Spec24.2](../../../spec/en/24.documentation_generation.md#242-cross-referencing-and-hyperlinks) は Doc9 と Doc13 href interface を固定しており、この採用はそれらを再検討しない。[共有 link 診断](../../mizar-diagnostics/ja/documentation_links.md) は別途の未採用・未割当の提案のままとする。

## Authenticated Import And Dependency Inputs
producer の責任は、解決済み import 選択を保持する Session/Artifact publication 境界に置く。Doc consumer は package/version を選択せず、import を再構築しない。producer は各 source module の完全識別と publication hash、結合済み module/publication 識別からなる意味上の順序の import vector、明示的な coverage（利用不可、不完全、完全（完全な空を含む））を供給する。格納順の sort と dependency interface hash ではその vector を確立できない。[Doc4 入力](../en/doc_build.md#index-inputs-and-later-completion) は変更しない。
producer は、選択 package publication を根とする明示的な有限 documentation-closure inventory も供給する。内容は package tuple、選択 publication hash、index location、各 index の module/export coverage である。完全性は対象 namespace と必要な全 dependency edge を明示し、参照の省略は不在の証拠としない。cycle はこの有限 inventory 内だけで許可し、discovery を認めない。source や network への fallback はない。
index consumer は利用前に既存の validating reader を通じて各参照先 index を認証し、schema 互換性、publication/package binding、完全 module/export 識別を検証済み publication に照合する。各 import endpoint はその認証済み inventory に結合しなければならない。path、FQN、canonical 順序、interface hash だけでは不十分。未対応 schema、integrity、stale/currentness、binding の失敗は下位 reader の拒否を維持し、coverage 不在は利用不可/不完全のままとする。
handoff は各照会 namespace の利用不可、不完全、明示的に完全な空を区別して保持する。現 module の一意な一致は import がなくても確定的であり、それ以外の非修飾探索成功には完全な順序付き import と必要な target 入力を要求する。FQN 探索は結合済み closure だけを使う。重複経路は来歴を保持しつつ同一の完全 target をまとめ、異なる target は決して統合しない。ここでは直列化 field や schema revision を選択しない。

### Authenticated Ordered Imports Handoff
`authenticate_doc_imports` は独立したメモリ内 gate-1 handoff であり、raw request の `import_order` と `dependency_indexes` は従来通り未認証である。root plan を現在の registry/request/workspace に対して再検証し、明示された有限 `(package, publication_hash, artifact_root)` inventory だけを既存 export/summary reader で読む。root は検証済み publication index の場所であり、新たな serialized DocIndex 形式ではない。
`AuthenticatedDocImports` は選択 publication、完全 identity の export/summary inventory、真正な `PublishedImportView` producer を非公開で保持する。source module/publication/hash と元の Resolver 来歴は検証済み publication に一致しなければならない。import target を inventory の完全 identity と publication hash に結合し、publication 結合が曖昧なら勝者を選ばない。
`DocImportCoverage` は拡張可能な `#[non_exhaustive]` の unavailable/incomplete/complete 状態である。module ごとの coverage と有限 closure coverage を別に保持する。producer、summary、endpoint の欠落は完全空にならず、完全空には真正な完全 producer と有限 dependency scope の検査が必要である。
この API に link lookup、direct-by/UsedBy、allocation、診断、render 統合は含まれず、[後続 readiness gate](#ordered-readiness-gates-and-decisions) は未完了のままである。

## Artifact-Owned Direct-By Producer
Artifact 所有者は、元の権威ある compilation/publication process から構文上の定理間直接 `by` citation を射影し、引用元 publication と index と同じ完全 export 識別に結合しなければならない。既存の [artifact evidence](../../../spec/en/23.package_management_and_build_system.md#2351-mizirjson-artifact-format)、export、`used_axioms` だけではこの graph を供給せず、Doc consumer は source、名前、proof evidence から復元できない。
各 citation record は、引用元と引用先の完全 module/publication/export 識別と occurrence 識別を保持する。occurrence 識別は引用元定理、保持された `by` occurrence ordinal、その occurrence 内の引用先 target ordinal であり、両 ordinal は元の遭遇順とする。認証済み source provenance/map が利用可能なら保持し、range を捏造しない。反復引用と自己引用も record として残し、package 間 endpoint は供給済み closure に対して認証しなければならない。ordinal は構文 occurrence を区別し、proof/evidence dependency を区別するものではない。
coverage は、選択した有限 closure 内でどの定理本体/証明を調べたか、その範囲が完全、不完全、利用不可のどれかを明示的に列挙する。完全な空には直接 edge のない権威ある完全 coverage が必要であり、空の record vector だけでは足りない。対応付け不能な endpoint は影響範囲の coverage を不完全にし、黙って捨てず occurrence 来歴を保持する。不在/未調査の定理や package から網羅的な UsedBy を主張できない。
consumer は対応付け済みの直接 edge だけを反転し、occurrence 順序を別に保持しながら spec24.2 の表示用重複除去/順序を適用する。完全性は明示した documentation closure に相対的であり、あり得る全外部依存元を対象としない。producer 射影、認証、schema 互換性、coverage 検証には別途所有者の作業が必要であり、interface 採用は artifact field や証明受理の変更を導入しない。

### Authenticated Direct-By Handoff
`authenticate_doc_direct_by` は現在有効な `AuthenticatedDocImports` と元の [Artifact 射影](../../mizar-artifact/ja/verified_artifact.md#in-memory-direct-by-projection) を受け取り、source を読まず dependency を探索しない。`AuthenticatedDocDirectBy` は不変 producer と元の occurrence/examination を保持し、反転・表示・link resolution は行わない。
各 source module/publication/hash を選択 manifest、captured SourceId を利用可能な認証済み import producer と結合し、調査済み/引用元/対応済み引用先 theorem を正確な格納 export identity に照合する。重複・scope 外の source producer と既知 endpoint の矛盾は拒否する。引用先 module が不在なら occurrence を incomplete のまま残す。完全な module coverage は全 local public theorem export の調査も要求し、reexport に local body を追加しない。producer 不在は unavailable、scope 欠落や unmapped owner/target は完全空にしない。
module ごと・全体の citation coverage は `DocImportCoverage` を再利用し、有限 import handoff の `closure_coverage()` と独立する。caller は両状態を保持する。供給 module の完全な citation 調査は欠けた dependency input の調査を確立しない。射影の serialized schema はなく、既存 validating reader の拒否は gate 1 に残す。Driver の真正 capture は元の compilation 構文と利用可能な意味的 mapping を保持する。構造上供給した callback だけでは真正 producer witness としない。

## Validated Page, Anchor And Section Allocations
render allocation 所有者は、選択 output root と完全 module/publication 識別に結合した table に加え、item 識別、正確な権威ある label/FQN id、optional alias を供給する。href 解決前に Doc consumer は、供給された完全な destination 集合に page-path 衝突がなく、同一 page 上で異なる target が一つの完全 id を共有していないという証拠を要求する。label 利用不可は item-anchor readiness を止め、完全 id の重複はその page を止める。識別の勝者、suffix、slug、推測 path で衝突を修復しない。
検証では、path が宣言 root に相対であること、destination が照会 target を覆うこと、正確な FQN fragment が結合済み export と一致すること、optional alias が spec24.2 の full/section/alias 衝突規則を満たすことを確認する。安全でない optional alias は完全 anchor を変えずに抑制する。directory spelling、filesystem case 方針、dedicated-item/source route、output publication は Doc13/Doc23 の決定であり、この interface 採用はその候補を採用しない。
生成仕様 producer は論理 section id と page destination を持つ明示的な version-bound section inventory を供給し、allocation validator は version binding、一意性、destination/fragment の存在を確認する。完全と判明した index に一致 section がなければ未解決とし、利用不可/部分的な index 入力から完全探索を確立できない。不在/未知の section を module anchor にしない。採用済み interface を超える section grammar は加えない。
検証は論理 path/fragment を渡し、推測した href や事前正規化した href を渡さない。採用済み [Doc13 href interface](../en/render.md#candidate-destinations-and-page-content) だけを適用し、正確な DOM id を保ちつつ path segment/完全 fragment ごとに UTF-8 を percent-encode する。allocation 完全性と href encoding は別の義務であり、正しい encoding では不在 destination を認証できない。

## Ordered Readiness Gates And Decisions
上記の前提 interface 定義は採用済みである。残る前提条件群を順に解決する。interface 採用だけでは producer、認証、coverage、allocation、診断、実装の gate のどれも充足しない:
1. [認証済み import/closure handoff](#authenticated-ordered-imports-handoff) は明示的な coverage と拒否検証付きの順序付き import と有限 dependency index を供給する。この input gate だけを充足し、後続 gate は充足しない。
2. [認証済み direct-by handoff](#authenticated-direct-by-handoff) は Artifact 所有の射影と有限 scope/endpoint 検証を供給し、反復/自己/package 間 occurrence と unavailable/incomplete/complete-empty coverage を evidence fallback なしで保持する。この producer gate だけを充足し、後続 gate はすべて開いたままとする。
3. それらの識別に対する page/full-anchor/alias/section allocation を供給・検証し、衝突、label 不在、version 不一致、採用済みの非 ASCII/percent/hash href encoding を witness で示す。
4. location support と profile 処理を含む [正確な共有診断](../../mizar-diagnostics/ja/documentation_links.md) を別途採用・割当し、同位置/非連続/未対応付け occurrence と scope だけの不完全性を witness で示す。
5. 別途 dispatch された Doc10 readiness/implementation unit だけが、これらの認証済み入力と採用済み診断を統合できる。Doc13 render、schema/publication 作業、coverage/owner/milestone credit はこの定義の範囲外である。
採用済みの内部/来歴の決定: publication-bound producer の配置、明示的 coverage inventory、citation occurrence ordinal、validation handoff は完成済み提案から維持した。新たな言語仕様の選択は不要であり、ここに未選択のものは残さない。数値診断と診断採用 metadata はその所有者で明示的に UNSELECTED のままであり、code/schema の割当は行わない。Doc9 の歴史的な②の代案は有効化しない。
