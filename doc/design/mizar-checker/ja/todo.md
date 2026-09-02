# mizar-checker TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。
> 2026-09-02 圧縮（batch CPT-02、規則は
> [../../documentation_compaction_rules.md](../../documentation_compaction_rules.md)）:
> ステータス文書の言語方針（2026-09-01 承認）に基づき、完了タスク本文の正本は
> 英語版および英語アーカイブ
> [../../archive/checker_todo_sections.md](../../archive/checker_todo_sections.md)
> に一本化した。以下には全見出し・登録済み redirect 行・未完了作業のみ残る。
> 各タスクの詳細の正本は [../../task_contracts/ja/](../../task_contracts/ja/)
> 配下の対応契約文書（例: 258B3M2B2B3E の owner partition 等、旧JA本文のみが
> 載せていた従属的詳細も契約が保持する）。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

モジュール仕様はまだ存在しない。各仕様は、それを引用する実装タスクより前に、
専用の仕様タスクが（英語と日本語を同じ変更で）執筆する。モジュール名は
[internal 07](../../internal/ja/07.crate_module_layout.md) の最小分割に従う。
この crate はアーキテクチャ 04、05、16、17、18、19 を精緻化する。

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| typed_ast | `typed_ast.md`（task 2） | `src/typed_ast.rs` | [x] |
| binding_env | `binding_env.md`（task 4） | `src/binding_env.rs` | [x] |
| type_checker | `type_checker.md`（task 6） | `src/type_checker.rs` | [~] |
| registration_resolution | `registration_resolution.md`（task 13） | `src/registration_resolution.rs` | [~] |
| cluster_trace | `cluster_trace.md`（task 15） | `src/cluster_trace.rs` | [~] |
| overload_resolution | `overload_resolution.md`（task 21） | `src/overload_resolution.rs` | [~] |
| resolved_typed_ast | `resolved_typed_ast.md`（task 27） | `src/resolved_typed_ast.rs` | [~] |

`mizar-checker` はパイプライン phase 6-8 を実装する。入力は `ResolvedAst` と
`SymbolEnv`、出力は `TypedAst`、`ResolutionTrace`、`ResolvedTypedAst` で
ある。phase に対応する 3 つの波で構築する: 型検査（phase 6）、再生可能な
trace を伴う cluster/registration 解決（phase 7）、オーバーロード解決
（phase 8）。soft type は意味論的メタデータであり、すべての事実は論理述語
または registration 由来の事実として説明可能でなければならず、どの波も
証明探索を行わない。

依存順序: `typed_ast` データ → `binding_env` / `type_checker`（第 1 波）→
`registration_resolution` / `cluster_trace`（第 2 波）→
`overload_resolution` / `resolved_typed_ast`（第 3 波）。

以下の各タスクは意図的に小さくしてある — 1 つのモジュール仕様、または
1 モジュールの 1 挙動スライス — 。これにより、crate の残りを抱え込まずに
1 タスクを単独で実装・テスト・コミットまで自律的に完遂できる。

## crate の前提条件

production crate は `mizar-session` と `mizar-resolve` に依存し、
`mizar-syntax` へは推移的にだけ到達する。Task 258B1 は parser-shaped
corruption fixture 用に direct な test-only `mizar-syntax` dev-dependency を
追加するが、production source は syntax-free のままである。第 1 波は
`mizar-resolve` task 14 と 20（名前解決、`SymbolEnv` 骨格）を必要とし、
以後の波は `mizar-resolve` task 21 のシグネチャ増分と、対応する
`mizar-parser` の定義文法タスク（23-31）とともに成長する。アーキテクチャ:
[04.type_and_registration_resolution.md](../../architecture/ja/04.type_and_registration_resolution.md)、
[05.overload_resolution.md](../../architecture/ja/05.overload_resolution.md)、
[16.substitution_and_binding.md](../../architecture/ja/16.substitution_and_binding.md)、
[17.cluster_trace_format.md](../../architecture/ja/17.cluster_trace_format.md)。
crate 所有権: [internal 07](../../internal/ja/07.crate_module_layout.md)。

## 解決済みおよび保留中の決定

- **TypedAst の arena 表現: task 3 で解決済み。** `TypedAst` は dense local
  id を持つ同質な `TypedNodeKind` arena を使い、現在の `mizar-syntax`
  compatibility view と `mizar-resolve` arena style を鏡映する。task 3 は
  node-kind storage のための direct `mizar-syntax` dependency を追加せず、
  checker-local な source-shape projection を使う。`ResolvedTypedAst` は task 28
  で同じ決定を再訪する。
- **registration の活性化ゲート: task 19 で解決済み。** ローカル
  registration は、その証明義務が設定済み verifier ポリシーに受理される
  まで自動推論に影響してはならない（アーキテクチャ 04 の制約）。phase
  11-14 がまだ存在しないため、task 19 は暫定ポリシーを実装する。
  生成された義務は pending / unverified status として記録し、explicit な
  accepted verifier/artifact status input が利用可能になるまで registration は
  active database に入らない。
  トップレベルに登録済み。`mizar-vc`/`mizar-proof` 着地時に再訪する。
- **trace スキーマ準拠: 解決済み。**
  [17.cluster_trace_format.md](../../architecture/ja/17.cluster_trace_format.md)
  が `ResolutionTrace` の正準スキーマである。`cluster_trace.md` はそれを
  精緻化するのであって、分岐させない。
- **diagnostics レコード: `mizar-resolve` の決定に従う**（`mizar-diagnostics`
  採用時期）。resolver が採用したレコードを checker も使う。トップレベルに
  登録済み。
- **constructor の property 値供給源: task 35 で解決済み。** デフォルトの
  構造体 constructor は field のみを受け取り、`property` 値は第 7 章の
  property implementation からのみ来る。task 35 は spec 05/07 を英日で
  更新し、reject-first の inactive `advanced_semantics` seed と traceability を
  追加し、checker/core source semantics は変更しない。

## 順序付きタスク一覧

各タスクの後で `cargo test -p mizar-checker` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### 第 1 波: 型検査（phase 6）

1. **crate の足場と lint 方針のガード。** [x]
2. **仕様: `typed_ast.md`。** [x]
3. **`typed_ast` データ形状の実装。** [x]
4. **仕様: `binding_env.md`。** [x]
5. **束縛環境とコンテキストの構築。** [x]
6. **仕様: `type_checker.md`。** [x]
7. **型式の正規化。** [x]
8. **宣言とローカル束縛の検査。** [x]
9. **項と論理式の型推論。** [x]
10. **coercion 候補、sethood、non-emptiness、narrowing 義務。** [x]
11. **型事実の記録とクエリ。** [x]
12. **stage `type_elaboration` のコーパスランナー。** [x]

### 第 2 波: cluster と registration の解決（phase 7）

13. **仕様: `registration_resolution.md`。** [x]
14. **registration 索引。** [x]
15. **仕様: `cluster_trace.md`。** [x]
16. **trace 記録付き cluster 解決閉包。** [x]
17. **cluster ループ検出と有界飽和。** [x]
18. **reduction の適用。** [x]
19. **pending registration の検証と活性化ゲート。** [x]
20. **attribute 付き型使用の existential ゲート。** [x]

### 第 3 波: オーバーロード解決（phase 8）

21. **仕様: `overload_resolution.md`。** [x]
22. **候補サイトの収集。** [x]
23. **template 展開。** [x]
24. **viability フィルタリング。** [x]
25. **specificity グラフの構築。** [x]
26. **根の選択、refinement の結合、view の挿入。** [x]
27. **仕様: `resolved_typed_ast.md`。** [x]
28. **`ResolvedTypedAst` の組み立て。** [x]

### 強化と横断フォローアップ

29. **stage `formula_statement` / `advanced_semantics` の deferred corpus obligation。** [x]
30. **決定性スイート。** [x]
31. **公開 enum の前方互換性ポリシー。** [x]
32. **ソース/仕様対応監査。** [x]
33. **二言語ドキュメント同期監査。** [x]
34. **module 境界リファクタリング gate。** [x]

### 第 4 波: 意味論監査フォローアップ（2026-07-03）

[semantic_spec_audit.md](./semantic_spec_audit.md) は checker が担当する仕様章
(03、05-08、13、14、17-19)を監査し、所見 SSA-001 から SSA-020 と 16 件の
adversarial rejection corpus を記録した。以下のタスクは全所見を、担当タスク
か明示的な処置(disposition)のいずれかに変換する。AGENTS.md が
`doc/spec/en/` を設計文書・コードより上位に置くため、spec 決定タスク
(35-44)が先行する: 仕様が決めていない挙動を checker が実装してはならない。
各 spec タスクは監査の提案解決策から選択し(またはより良い解決策を記録し)、
`doc/spec/en/` と `doc/spec/ja/` を同一変更で更新し、決定が新しい拒否を
生む場合は reject-first corpus seed を追加または活性化し、
`tests/coverage/spec_trace.toml` を更新する。

所見の処置(全 SSA id はタスクまたは記録済みの理由に対応する):

| 所見 | 処置 |
|---|---|
| SSA-001 | task 35 |
| SSA-002, SSA-011, SSA-012 | task 36 |
| SSA-003, SSA-010, SSA-016, SSA-019 | task 37 |
| SSA-004 | task 38 |
| SSA-005 | task 39 |
| SSA-006 | task 40 |
| SSA-007, SSA-008, SSA-020 | task 41 |
| SSA-009 | task 42 |
| SSA-013, SSA-014 | task 43 |
| SSA-015, SSA-017 | task 44 |
| SSA-018 | タスク化しない: greedy `of`/`over` parse は決定的かつ文書化済み(spec 19.6.4)。scope 感度 lint は将来の diagnostics 採用 wave に属し、そこで記録する |
| corpus seeds | task 49 が必要な runner、parser support、declaration-symbol support、source-to-checker payload extraction 到着時に、監査 fixture 16 件、task-35 constructor-property seed、task-36 duplicate-coverage seed、task-37 ordinary/template-derived equivalent-root seed、task-38 functorial-`for` guard seed、task-39 property-overlap coherence seed、task-44 omitted-`reconsider` / ambiguous-redefinition-target seed を活性化する。task-37 same-return signature-conflict seed は Resolver Task 31 が `declaration_symbol` で単独活性化し、task 49 はその member を exact 24-fixture set と reconciliation/deduplication するだけである |

35. **Spec 決定: constructor property 引数と extensionality(SSA-001)。** [x]
36. **Spec 決定: structure member 同一性・upcast path・非循環性(SSA-002, SSA-011, SSA-012)。** [x]
37. **Spec 決定: オーバーロード tie-break と tie の曖昧性(SSA-003, SSA-010, SSA-016, SSA-019)。** [x]
38. **Spec 決定: functorial cluster `for T` の意味論(SSA-004)。** [x]
39. **Spec 決定: property implementation の coherence(SSA-005)。** [x]
40. **Spec 契約: registration activation のタイミング(SSA-006)。** [x]
41. **Spec 明確化: closure 停止性・矛盾検出サイト・`attr(args)`(SSA-007, SSA-008, SSA-020)。** [x]
42. **Spec 明確化: reduction 決定性のシグネチャ(SSA-009)。** [x]
43. **Spec 明確化: 依存 mode の sethood と built-in inhabitation(SSA-013, SSA-014)。** [x]
44. **Spec 明確化: `reconsider` の discharge と曖昧な redefinition target(SSA-015, SSA-017)。** [x]
45. **Checker 整合: オーバーロード tie-break の実装。** [x]
46. **Checker 整合: closure の矛盾検出と停止性規則。** [x]
47. **Checker 整合: existential gate と activation 契約。** [x]
48. **Reserve source declaration producer seam。** [x]
49. **監査 corpus の活性化と task-29 record の改訂。** [ ]
    - `advanced_semantics`/`formula_statement` runner、property-implementation
      parser support、source-to-checker payload 抽出(mizar-test runner 成長 +
      MC-G020/MC-G021/MC-G023/MC-G027、および task-39 seed については
      MC-G030/property-implementation payload extraction)が到着したら、意味論監査 fixture
      16 件、task-35 constructor-property seed、task-36 duplicate-coverage
      seed、task-37 ordinary/template-derived equivalent-root ambiguity seed、
      task-38 functorial-`for` guard seed、task-39 property-overlap coherence
      seed、task-44 omitted-`reconsider` / ambiguous-redefinition-target seed を活性化する。
      exact scopeは
      [payload_family_decomposition.md](./payload_family_decomposition.md)の
      24-fixture reconciliation setである。same-return memberはresolver Task 31が
      `declaration_symbol`でsole activationし、Task 49は残り23件をactivateした後に
      24件全体をreconcile/deduplicateする。same-signature/different-return fixtureは
      set外で既にactiveであり、再activationせずunchanged controlとして保つ。task-29 の
      deferred corpus record を監査由来の requirement id を指す(または
      置き換えられる)よう改訂する。
    - 受け入れ条件: `mizar-test` plan が fixture を active と表示し plan
      error が 0 件。deferred record が二重計上されない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: 完了済みtasks 35-44、parser Tasks 47-48、resolver Task 31、完了済み
      checker Task 247、checker Tasks 248-264/269-279（blocked-reserved Task
      274のexternal accepted-status gateとexternal scheme/theorem-role Gate S1を
      含む）、mizar-test Task-10 increments `MT10-FS`/`MT10-AS`。Tasks 266-268
      だけでは不十分。参照:
      [payload_family_decomposition.md](./payload_family_decomposition.md)、
      semantic_spec_audit.md「Adversarial Corpus」。

50. **Source-derived attributed reserve evidence-gap bridge.** [x]
51. **Source-derived local mode reserve expansion-gap bridge.** [x]
52. **Source-derived local structure reserve evidence-gap bridge.** [x]
53. **Source-derived attributed local structure reserve evidence-gap bridge.** [x]
54. **Source-derived attributed local mode reserve expansion-gap bridge.** [x]
55. **Source-derived bare local mode expansion bridge.** [x]
56. **Source-derived local mode expansion chain bridge.** [x]
57. **Source-derived local mode structure-RHS evidence-gap bridge.** [x]
58. **Source-derived local mode attributed-builtin RHS evidence-gap bridge.** [x]
59. **Source-derived attributed local mode reserve evidence-gap bridge.** [x]
60. **Source-derived attributed local mode structure-RHS evidence-gap bridge.** [x]
61. **Source-derived attributed local mode attributed-builtin-RHS evidence-gap bridge.** [x]
62. **Source-derived local mode structure-RHS chain evidence-gap bridge を追加する。** [x]
63. **Source-derived local mode attributed-builtin-RHS chain evidence-gap bridge を追加する。** [x]
64. **Source-derived attributed local mode bare-builtin chain evidence-gap bridge を追加する。** [x]
65. **Source-derived attributed local mode structure-RHS chain evidence-gap bridge を追加する。** [x]
66. **Source-derived attributed local mode attributed-builtin-RHS chain evidence-gap bridge を追加する。** [x]
67. **Source-derived structure-qualified attribute gap boundary を追加する。** [x]
68. **Source-derived argument-bearing mode reserve gap boundary を追加する。** [x]
69. **Source-derived argument-bearing structure reserve gap boundary を追加する。** [x]
70. **Source-derived bracket-form local mode reserve gap boundary を追加する。** [x]
71. **Source-derived bracket-form local structure reserve gap boundary を追加する。** [x]
72. **Source-derived two-edge bare local mode chain bridge を追加する。** [x]
73. **Source-derived three-edge bare local mode chain bridge を追加する。** [x]
74. **Source-derived structural bare local mode chain bridge を追加する。** [x]
75. **Source-derived local mode forward-reference active-range boundary を追加する。** [x]
76. **Source-derived local structure forward-reference active-range boundary を追加する。** [x]
77. **Source-derived local attribute forward-reference active-range boundary を追加する。** [x]
78. **Source-derived imported structure reserve extraction-gap boundary を追加する。** [x]
79. **Source-derived imported mode reserve extraction-gap boundary を追加する。** [x]
80. **Source-derived imported attribute reserve extraction-gap boundary を追加する。** [x]
81. **Source-derived argument-bearing local attribute reserve extraction-gap boundary を追加する。** [x]
82. **Source-derived imported mode reserve provenance bridge を追加する。** [x]
83. **Source-derived imported structure reserve provenance bridge を追加する。** [x]
84. **Source-derived imported attribute reserve provenance bridge を追加する。** [x]
85. **Source-derived imported non-empty attribute reserve provenance bridge を追加する。** [x]
116. **Source-derived imported positive empty attribute reserve provenance bridge を追加する。** [x]
171. **Source-derived imported negative empty object reserve provenance bridge を追加する。** [x]
86. **Source-derived theorem formula extraction-gap boundary を追加する。** [x]
115. **Exact source-derived formula statement recovery checker bridge を追加する。** [x]
117. **Source-derived formula constant kind checker bridge を追加する。** [x]
118. **Builtin binary theorem exact-token guard を厳密化する。** [x]
119. **Exact source-derived reserved-variable equality checker bridge を追加する。** [x]
120. **Exact source-derived reserved-variable membership checker bridge を追加する。** [x]
121. **Exact source-derived reserved-variable inequality checker bridge を追加する。** [x]
122. **Checker の reflexive type-assertion admissibility と exact reserved-variable source bridge を追加する。** [x]
123. **Exact source-derived distinct reserved-variable equality checker bridge を追加する。** [x]
124. **Exact source-derived multiple-reserve-declaration equality checker bridge を追加する。** [x]
125. **Exact source-derived heterogeneous-reserve membership checker bridge を追加する。** [x]
126. **Exact direct-local-mode reserved-variable equality checker bridge を追加する。** [x]
127. **Exact one-edge local-mode-chain reserved-variable equality checker bridge を追加する。** [x]
128. **Exact direct local-object-mode reserved-variable equality checker bridge を追加する。** [x]
129. **Exact one-edge local-object-mode-chain reserved-variable equality checker bridge を追加する。** [x]
130. **Exact direct-local-mode reserved-variable inequality checker bridge を追加する。** [x]
131. **Exact direct-local-object-mode reserved-variable inequality checker bridge を追加する。** [x]
132. **Exact one-edge local-mode-chain reserved-variable inequality checker bridge を追加する。** [x]
133. **Exact one-edge local-object-mode-chain reserved-variable inequality checker bridge を追加する。** [x]
134. **Exact two-edge local-mode-chain reserved-variable equality checker bridge を追加する。** [x]
135. **Exact two-edge local-object-mode-chain reserved-variable equality checker bridge を追加する。** [x]
136. **Exact two-edge local-mode-chain reserved-variable inequality checker bridge を追加する。** [x]
137. **Exact two-edge local-object-mode-chain reserved-variable inequality checker bridge を追加する。** [x]
138. **Exact direct-local-mode reserved-variable normalized-reflexive type assertion checker bridge を追加する。** [x]
139. **Exact direct-local-mode left reserved-variable membership checker bridge を追加する。** [x]
140. **Exact direct-local-object-mode left reserved-variable membership checker bridge を追加する。** [x]
141. **Exact one-edge local-mode-chain left reserved-variable membership checker bridge を追加する。** [x]
142. **Exact one-edge local-object-mode-chain left reserved-variable membership checker bridge を追加する。** [x]
143. **Exact two-edge local-mode-chain left reserved-variable membership checker bridge を追加する。** [x]
144. **Exact two-edge local-object-mode-chain left reserved-variable membership checker bridge を追加する。** [x]
145. **Exact direct local-object-mode reserved-variable normalized-reflexive type assertion checker bridge を追加する。** [x]
146. **Exact one-edge local-mode-chain reserved-variable normalized-reflexive type assertion checker bridge を追加する。** [x]
147. **Exact one-edge local-object-mode-chain reserved-variable normalized-reflexive type assertion checker bridge を追加する。** [x]
148. **Exact two-edge local-mode-chain reserved-variable normalized-reflexive type assertion checker bridge を追加する。** [x]
149. **Exact two-edge local-object-mode-chain reserved-variable normalized-reflexive type assertion checker bridge を追加する。** [x]
150. **Exact three-edge local-mode-chain reserved-variable normalized-reflexive type assertion checker bridge を追加する。** [x]
151. **Exact three-edge local-object-mode-chain reserved-variable normalized-reflexive type assertion checker bridge を追加する。** [x]
152. **Exact four-edge local-mode-chain reserved-variable normalized-reflexive type assertion checker bridge を追加する。** [x]
153. **Exact four-edge local-object-mode-chain reserved-variable normalized-reflexive type assertion checker bridge を追加する。** [x]
154. **Exact three-edge local-mode-chain reserved-variable equality checker bridge を追加する。** [x]
155. **Exact three-edge local-object-mode-chain reserved-variable equality checker bridge を追加する。** [x]
156. **Exact three-edge local-mode-chain reserved-variable inequality checker bridge を追加する。** [x]
157. **Exact three-edge local-object-mode-chain reserved-variable inequality checker bridge を追加する。** [x]
158. **Exact three-edge local-mode-chain left reserved-variable membership checker bridge を追加する。** [x]
159. **Exact distinct-binding shared-reserve membership checker bridge を追加する。** [x]
160. **Exact distinct-binding shared-reserve inequality checker bridge を追加する。** [x]
161. **Exact multiple-reserve-declaration inequality checker bridge を追加する。** [x]
162. **Exact multiple-reserve-declaration membership checker bridge を追加する。** [x]
87. **Source-derived term formula extraction-gap boundary を追加する。** [x]
88. **Source-derived proof skeleton extraction-gap boundary を追加する。** [x]
89. **Source-derived statement proof extraction-gap boundary を追加する。** [x]
90. **Source-derived predicate/functor definition extraction-gap boundary を追加する。** [x]
91. **Source-derived attribute definition extraction-gap boundary を追加する。** [x]
92. **Source-derived mode/structure definition extraction-gap boundary を追加する。** [x]
93. **Source-derived proof-local declaration extraction-gap boundary を追加する。** [x]
94. **Source-derived proof-local inline definition extraction-gap boundary を追加する。** [x]
95. **Source-derived registration block extraction-gap boundary を追加する。** [x]
96. **Source-derived redefinition / notation extraction-gap boundary を追加する。** [x]
97. **Source-derived imported TypeCaseStruct reserve provenance bridge を追加する。** [x]
98. **Source-derived imported predicate/functor term-formula extraction-gap boundary を追加する。** [x]
99. **Source-derived formula connective/quantifier extraction-gap boundary を追加する。** [x]
112. **Exact source-derived formula connective/quantifier shell checker bridge を追加する。** [x]
100. **Source-derived builtin membership formula extraction-gap boundary を追加する。** [x]
101. **Source-derived builtin inequality formula extraction-gap boundary を追加する。** [x]
102. **Source-derived builtin type assertion formula extraction-gap boundary を追加する。** [x]
103. **Source-derived imported attribute assertion formula extraction-gap boundary を追加する。** [x]
104. **Source-derived attribute-level non-empty imported attribute assertion formula extraction-gap boundary を追加する。** [x]
114. **Exact source-derived attribute-level non-empty imported attribute assertion theorem checker bridge を追加する。** [x]
105. **Source-derived set-enumeration formula extraction-gap boundary を追加する。** [x]
111. **Exact source-derived set-enumeration theorem checker bridge を追加する。** [x]
106. **Source-derived builtin equality theorem term/formula checker bridge を追加する。** [x]
108. **Source-derived builtin membership theorem term/formula checker bridge を追加する。** [x]
110. **Source-derived imported predicate/functor theorem checker bridge を追加する。** [x]
163. **Exact three-edge local-object-mode membership checker bridge を追加する。** [x]
164. **Exact four-edge local-mode membership checker bridge を追加する。** [x]
165. **Exact four-edge local-object-mode membership checker bridge を追加する。** [x]
166. **Exact four-edge local-mode equality checker bridge を追加する。** [x]
167. **Exact four-edge local-object-mode equality checker bridge を追加する。** [x]
168. **Exact four-edge local-mode inequality checker bridge を追加する。** [x]
169. **Exact four-edge local-object-mode inequality checker bridge を追加する。** [x]
172. **Exact local-mode long-chain equality checker bridge を追加する。** [x]
173. **Exact local-mode long-chain inequality checker bridge を追加する。** [x]
174. **Exact local-mode long-chain membership checker bridge を追加する。** [x]
175. **Exact local-mode long-chain type assertion checker bridge を追加する。** [x]
176. **Exact local-object-mode long-chain equality checker bridge を追加する。** [x]
177. **Exact local-object-mode long-chain inequality checker bridge を追加する。** [x]
178. **Exact local-object-mode long-chain membership checker bridge を追加する。** [x]
179. **Exact local-object-mode long-chain type assertion checker bridge を追加する。** [x]
180. **Exact source-derived contradiction formula-constant checker bridge を追加する。** [x]
181. **Exact imported attributed-reserve routing を repair する。** [x]
182. **Exact formula-side local-mode asserted-head checker bridge を追加する。** [x]
183. **Object-terminal formula-side local-mode asserted-head checker bridge を追加する。** [x]
184. **One-edge formula-side local-mode asserted-head checker bridge を追加する。** [x]
185. **One-edge object-terminal formula-side local-mode asserted-head checker bridge を追加する。** [x]
186. **Two-edge formula-side local-mode asserted-head checker bridge を追加する。** [x]
187. **Two-edge object-terminal formula-side local-mode asserted-head checker bridge を追加する。** [x]
188. **Exact builtin-object reserved-variable equality checker bridge を追加する。** [x]
189. **Exact builtin-object reserved-variable type-assertion checker bridge を追加する。** [x]
190. **Exact builtin-object reserved-variable inequality checker bridge を追加する。** [x]
191. **Exact distinct-binding shared-builtin-object equality checker bridge を追加する。** [x]
192. **Exact distinct-binding shared-builtin-object inequality checker bridge を追加する。** [x]
193. **Exact multiple-reserve-declaration builtin-object equality checker bridge を追加する。** [x]
194. **Exact multiple-reserve-declaration builtin-object inequality checker bridge を追加する。** [x]
195. **Exact three-edge formula-side local-mode asserted-head checker bridge を追加する。** [x]
196. **Exact three-edge object-terminal formula-side local-mode asserted-head checker bridge を追加する。** [x]
197. **Exact four-edge formula-side local-mode asserted-head checker bridge を追加する。** [x]
198. **Exact four-edge object-terminal formula-side local-mode asserted-head checker bridge を追加する。** [x]
199. **Exact seven-expansion set-terminal formula-side local-mode asserted-head checker bridge を追加する。** [x]
200. **Exact seven-expansion object-terminal formula-side local-mode asserted-head checker bridge を追加する。** [x]
201. **Exact one-edge formula-side immediate-radix local-mode asserted-head checker bridge を追加する。** [x]
202. **Exact one-edge object-terminal formula-side immediate-radix local-mode asserted-head checker bridge を追加する。** [x]
203. **Exact two-edge set-terminal formula-side immediate-radix local-mode asserted-head checker bridge を追加する。** [x]
204. **Exact two-edge object-terminal formula-side immediate-radix local-mode asserted-head checker bridge を追加する。** [x]
205. **Exact three-edge set-terminal formula-side immediate-radix local-mode asserted-head checker bridge を追加する。** [x]
206. **Exact three-edge object-terminal formula-side immediate-radix local-mode asserted-head checker bridge を追加する。** [x]
207. **Exact four-edge set-terminal formula-side immediate-radix local-mode asserted-head checker bridge を追加する。** [x]
208. **Exact four-edge object-terminal formula-side immediate-radix local-mode asserted-head checker bridge を追加する。** [x]
209. **Exact seven-expansion set-terminal formula-side immediate-radix local-mode asserted-head checker bridge を追加する。** [x]
210. **Exact seven-expansion object-terminal formula-side immediate-radix local-mode asserted-head checker bridge を追加する。** [x]
211. **Exact two-edge set-terminal formula-side two-hop local-mode asserted-head checker bridge を追加する。** [x]
212. **Exact two-edge object-terminal formula-side two-hop local-mode asserted-head checker bridge を追加する。** [x]
213. **Exact three-edge set-terminal formula-side two-hop local-mode asserted-head checker bridge を追加する。** [x]
214. **Exact three-edge object-terminal formula-side two-hop local-mode asserted-head checker bridge を追加する。** [x]
215. **Exact four-edge set-terminal formula-side two-hop local-mode asserted-head checker bridge を追加する。** [x]
216. **Exact four-edge object-terminal formula-side two-hop local-mode asserted-head checker bridge を追加する。** [x]
217. **Exact three-edge set-terminal formula-side three-hop local-mode asserted-head checker bridge を追加する。** [x]
218. **Exact three-edge object-terminal formula-side three-hop local-mode asserted-head checker bridge を追加する。** [x]
219. [x] **Exact four-edge set-terminal three-hop asserted head を bridge する。**
220. [x] **Exact four-edge object-terminal three-hop asserted head を bridge する。**
221. [x] **Exact four-edge set-terminal full-distance four-hop asserted head を bridge する。**
222. [x] **Exact four-edge object-terminal full-distance four-hop asserted head を bridge する。**
223. [x] **Exact transparent single-parenthesized reserved-variable equality を bridge する。**
224. [x] **exact seven-expansion set-terminal two-hop asserted head を bridge する。**
225. [x] **exact seven-expansion object-terminal two-hop asserted head を bridge する。**
226. [x] **exact seven-expansion set-terminal three-hop asserted head を bridge する。**
227. [x] **exact seven-expansion object-terminal three-hop asserted head を bridge する。**
228. [x] **exact seven-expansion set-terminal four-hop asserted head を bridge する。**
229. [x] **exact seven-expansion object-terminal four-hop asserted head を bridge する。**
230. [x] **exact seven-expansion set-terminal five-hop asserted head を bridge する。**
231. [x] **exact seven-expansion object-terminal five-hop asserted head を bridge する。**
233. [x] **exact parenthesized builtin-object reserved-variable equality を bridge する。**
234. [x] **Exact seven-expansion set-terminal full-distance six-hop asserted head を bridge する。**
236. [x] **Exact seven-expansion object-terminal full-distance six-hop asserted head を bridge する。**

## 推奨検証

各タスクの後で実行する:

```text
cargo test -p mizar-checker
cargo clippy -p mizar-checker --all-targets -- -D warnings
```

resolver 境界やコーパスに触れるタスクでは追加で実行する:

```text
cargo test -p mizar-resolve
cargo test -p mizar-test
```

テストが通ったらここでタスクにチェックを付ける。

## 備考

- checker が所有するのは soft type の事実、再生可能な registration 効果、
  オーバーロードの最終決定のみ: 証明探索、ATP の前提選択、任意の一階推論は
  行わない。
- ここで `VcId` を割り当てることは決してない。phase 6-8 は
  `InitialObligationId` を発行し、`mizar-vc` が後で正確に 1 回変換する。
- 各波の網羅性は `mizar-resolve` のシグネチャ増分とパーサーの定義文法
  タスクが律速する。resolver がまだ収集できない宣言種別を検査しない。
- 依存スライスと fingerprint の統合（アーキテクチャ 18）は `mizar-cache`
  とともに到来する。checker はスライスが計算可能であり続けるよう、
  ソース単位の寄与追跡を正確に保つだけでよい。

## Task 241 Active Addendum

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 242 Active Addendum

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 243 Active Addendum

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 244 Active Addendum

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 245 Active Addendum

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 246 Active Addendum

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Tasks 266-268 Final Checker Handoff Queue

Completion evidence: [central Task-247 historical contract](../../task_contracts/ja/247.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Tasks 248-264/269-279 STEP 5 ソースペイロードproducer queue

完全なauthority slice、dependency、consumer、gate、negative scope、exit
criteriaは[payload_family_decomposition.md](./payload_family_decomposition.md)
をcanonicalとする。以下の各unchecked rowは将来の1 nonempty logical taskかつ
1 commitである。

- [x] **Task 248:** source item/declaration site/binding-context producer。
  syntax-free producer、immutable `TypedAst`/`ResolvedTypedAst` handoff、exact
  reserve/definition-parameter shadowing consumer、recovery boundary、corruption
  matrix、bounded trace rowは完了した。term-use selection、type result、RHS/
  formula/proof semantics、accepted factへのcreditは与えない。
- [x] **Task 249:** type head/application/argument producer。public
  syntax-free flat producer、input-only legacy binding-environment seam、
  immutable `TypedAst`/clone-only `ResolvedTypedAst` handoff、exact broad
  10/13/6 consumer、Task-248 2/2/0 co-consumer、corruption/determinism matrix、
  runner-only readiness detail、bounded trace row、paired docs、count/hash
  verificationはcomplete。normalization、evidence、term/`qua` selection、
  accepted fact/declaration/proof、downstream IRはpublishしない。
- [x] **Task 250:** attribute-chain/qualification/provenance producer。public
  syntax-free five-table producer、immutable `TypedAst`/clone-only
  `ResolvedTypedAst` handoff、exact real consumer 4件とaggregate
  4/4/0・4/4/1/1/1 oracle、written polarity/qualifier/group/actual preservation、
  synthetic prefix extractor、corruption/determinism matrix、bounded
  trace/expectation progression、paired documentationはcomplete。semantic
  attribute instance、arity/admissibility、term result、evidence request/result、
  accepted fact/declaration/proof、downstream IRはpublishしない。
- [x] **Task 251:** evidence-query request/dependency-fact-reference producer。
  paired crate-plan contractを実装し、exact Task-249 broad route +
  Task-84/85がdense syntax-free request/response-reference tableを通してmissing request
  10件（mode-expansion 5 / structure-inhabitation 3 / attributed 2）をemitする。
  `TypedAst`がownし、
  `ResolvedTypedAst`がclone-preserveする。production-path testはsupplied inputを
  accepted evidenceと扱わずrequested/missing/rejected/suppliedを区別する。
  checker testはexact association/corruption/cardinality/atomicityをcoverし、
  evidence resultを捏造しない。
- [x] **Task 252:** primary term producer。paired crate-plan contractをpublic
  3-table syntax-free contract、exact 3-route 7/4/2 real consumer oracle、
  transparent-parenthesis rule、numeric-request-only boundary、synthetic
  constant/`it` dependency coverage、final ownership、test、bounded covered
  trace rowまで実装した。real `it` ownerと
  local-constant binding productionはTasks 260/264/269に残す。post-freeze
  contract correctionは各reference ordinalを先行して完了したbinding rowから
  deriveし、reachableな`Ambiguous` rejectionのためduplicate-priority binding
  groupを保持し、このproducerで`Resolver`がstructurally unreachableと記録する。
- [x] **Task 253:** functor/inline-functor application producer。public
  5-table `source_application` contractをexact imported caseと同じdefinition
  blockの後続definiens consumer、aggregate application/wrapper/candidate/
  argument/request 2/1/2/3/4 oracle、Task-252 term/reference/numeric-request
  3/1/2 slice、Task-248 `DefinitionParameter` shadow-handoff再利用、
  transparent cross-family origin ownership、individual candidate-reference
  boundary、synthetic schema、corruption matrix、final ownershipまで実装した。
  inline identity/formal/capture/substitutionはTask 270、direct template
  transportはTask 277、ordinary/template candidate collection/viability/
  winnerはTask 278に残り、Task 253はTask-252 primary termを複製しない。
- [x] **Task 254:** structure constructor/selector/update term producer。
  public seven-table syntax-free handoff、Task-248 context reuse、
  Task-252/253/254 ownership/fingerprint matrix、exact
  5/0/3/9/2/10/26 + 8/0/8 consumer、arena-key class 5個、bounded
  fixture/trace rowとreciprocal backlink、corruption/determinism/final
  ownership coverage、measured 413/377・243/231 oracleはcompleteである。
  structure/member/view semanticsはTask 263 ownershipに残す。
- [x] **Task 255:** set/comprehension/choice/`qua` term producer。public
  6-table syntax-free `source_set_term` transaction、exact local-definition
  consumer、4/0/1/3/4/7 + Task-252 4/0/4 oracle、one-way
  Task-252/253/254/255 child ownership、conditional fingerprint、one-shot
  final handoff、bounded fixture/trace row、frozen producer/extractor/
  corruption/install-order matrixはcompleteである。comprehension binder
  identity/captureはTask 257、condition付きcomprehension formula ownershipは
  Tasks 256-257、semantic result/sethood/nonemptiness/widening decisionはdeferred
  のままとする。
- [x] **Task 256:** atomic formula producer。public 8-table syntax-free
  transaction、private exact 8-route consumer、exact
  `8/0/1/1/1/2/13/11` aggregate、Task-252 `16/0/16`、Task-253
  `1/1/1/2/2`、Task-255 `2/0/0/0/4/2`、conditional
  Task-253/254/255 fingerprint、unresolved input request 11件、final immutable
  handoff、reciprocal trace increment、review済みreal/synthetic/exclusion/
  corruption/install matrixは完了した。既存semantic routeと全outcome/detail
  fieldは不変である。
- [ ] **Task 257:** composite/quantified formula、binder、predicate-chain、
  conditioned-comprehension umbrella。
  - [x] **Task 257A:** exact implication/universal/negation/contradiction treeと
    explicit unused binder 1件。public seven-table transaction、`2/1/4`
    binding extension、private exact consumer、final ownership、reciprocal
    trace row、review済みreal/synthetic/corruption/isolation matrixは完了した。
    preflight-corrected real rangeをcanonical `.miz`/semantic detail vector
    不変のまま保持する。
  - [ ] **Task 257B:** broader connective/quantifier shape、implicit binder、
    bound use/capture。
    - [x] **Task 257B1:** explicit universal-to-atomic compositionとbinderが
      selectするbound use 2件。exact 79-byte pass consumer、第2 exact Task-257
      composite profile、Task-252/256 dependency、`1/2` formula-composition
      transaction、final ownership、bounded trace rowはsemantic truth/theorem
      acceptanceなしでcomplete。
    - [x] **Task 257B2:** Task 257B1後のexact conjunction/disjunction/`iff`/
      repetition/executable formula grouping transport。connective
      truth/theorem acceptanceは含めない。
    - [x] **Task 257B3:** Tasks 257B1/B2後のexact existential/restricted/
      nested quantification、implicit reserved-binder shadowing、scoped use
      6件。semantic truth/closure/capture result/theorem creditなしでfrozen
      source-to-final-handoff transportを実装済み。
  - [ ] **Task 257C:** separately frozen Task-256/255 extension後の
    predicate-chain/conditioned-comprehension composition。
    - [x] **Task 257C1:** Task 256をpredicate-chain segment、polarity token、
      shared-boundary transportで拡張する。
      - [x] syntax-free 9-table contract、exact consumer、test、trace
        projection、ownership、semantic deferralをfreeze。
      - [x] fresh preflight後、別logical task/commitでfrozen contractを実装。
    - [x] Task 255のcondition-bearing comprehension transportを別の
      documentation/implementation pairとしてfreeze/実装する。
    - [ ] predicate-chain/conditioned-comprehension formula compositionは
      後続の別Task-257C sliceだけで追加する。
      - [x] **Task 257C2 prerequisite:** exact independent
        condition-to-atomic-formula associationをsemanticsなしでfreeze。
      - [x] **Task 257C2 implementation:** separate Task-256C1とfresh
        preflight後、frozen condition-formula associationだけをimplement。
      - [x] **Task 257C3 prerequisite:** Task 257C2後、predicate-chain
        conjunction/segment-negation compositionをseparately freeze。
      - [x] **Task 257C3 implementation:** fresh post-documentation
        preflight後にfrozen predicate-chain compositionだけを実装。
- [ ] **Task 258:** general theorem-owner/statement-semantic/assumption/
  visibility-scoped input-fact producer。accepted theorem factをpublishしない。
  - [x] **Task 258A:** exact reserved-variable equality theorem owner、
    statement shell、implicit reserved-type-guard input、unverified
    proposition candidate。
    - [x] exact 81-byte future `MT10-FS` source、resolver owner/label
      provenance、Task-252/256 lower profile、syntax-free `1/1/1/1/1`
      transaction、typed/resolved ownership、empty-semantic boundary、owned
      BindingEnv/fingerprint、production/named test-only seamでのTask-248
      exclusion、tests、trace non-activation、exit criteriaをfreeze。
    - [x] dedicated documentation commitとfresh parser/resolver/lower-API/
      count/hash preflight後にfrozen Task-258A transportだけを実装。
  - [ ] **Task 258B:** explicit assumption/conclusion/witness、local
    label/citation input、composite theorem root、nested statement context、
    broader visibility。Tasks 269-272はproof-local binding/closure/
    reconsider intent/proof skeleton/justification semanticsを保持する。
    - [x] **Task 258B1 prerequisite:** exact 139-byte nested
      equality-statement source、theorem owner 1件、statement/context/guard/
      candidate各4件、proof binding context 3件、local proof-step label 1件、
      resolved citation 1件、replayable resolver projection/reference/result、
      sole keyed node 68を持つtwo-pass 77-node/root-76 resolver AST、
      Task-252/256 dependency、typed/resolved ownership、test-only syntax
      dev-dependency、empty-semantic boundary、tests、non-activationをfreeze。
    - [x] **Task 258B1 implementation:** dedicated documentation commitと
      fresh parser/resolver/lower-API/count/hash preflight後にfrozen nested
      conclusion/local-label transportだけを実装した。checker 4本とrunner 5本で
      bounded `source_drift`/`test_gap`を閉じ、semantic/corpus activation
      gateはすべてdeferredのままとした。
    - [x] **Task 258B2 prerequisite:** exact 113-byte unlabeled single-
      assumption source、55-node/root-54 parser shape、theorem/assumption/
      conclusion `1/3/3/3/3` profile、Task-48 `2/1/0`、Task-252
      `6/6/0`、Task-256 `3/0/0/0/0/0/0/6/6`、base-only typed/final
      ownership、empty-semantic boundary、tests、non-activationをfreeze。
    - [x] **Task 258B2 implementation:** dedicated documentation commitと
      fresh parser/resolver/lower-API/count/hash preflight後にfrozen
      single-assumption transportだけをimplementした。checker 4本/runner
      5本がbounded `source_drift`/`test_gap`をcloseし、semantic/corpus
      activationは追加していない。
    - [x] **Task 258B3 prerequisite:** exact 104-byte unnamed witness
      source、49-node/root-48 parser identity、theorem-only resolver
      provenance、Task-48 `2/1/0`、Task-252 `5/5/0`、Task-256
      `2/0/0/0/0/0/0/4/4`、formula-only base `1/2/2/2/2`、one-row
      witness companion、paired typed/final ownership、tests、
      non-activationをfreeze。
    - [x] **Task 258B3 implementation:** documentation commit/fresh
      preflight後にfrozen paired witness transportだけをimplementした。
      checker 4本/runner 5本がbounded `source_drift`/`test_gap`をcloseし、
      semantics/corpus activationは追加していない。
    - [ ] **Tasks 258B3N/M:** B3後/B4前にnamed-witness transportと
      multiple/other witness-term transportをseparately freezeする。
      abbreviation/substitution/type-obligation/goal semanticsをinferしない。
    - [ ] **Tasks 258B4-B5:** composite theorem rootとbroader
      imported/outer/inner visibility profileをseparately freezeし、Tasks
      269-272 semanticsを吸収しない。
- [x] **Task 259:** predicate-definition/initial-obligation intake producer。
- [x] **Task 260:** functor-definition/initial-obligation intake producer。
- [x] **Task 261:** attribute-definition producer。
- [x] **Task 262:** mode-definition producer。
- [x] **Task 263:** structure/inheritance/constructor-definition producer。
- [x] **Task 264:** property-implementation producer。parser Task 48に依存。
- [ ] **Task 269:** proof-local declaration/binding producer。
- [ ] **Task 270:** inline-definition closure/capture/substitution-request producer。
- [ ] **Task 271:** `reconsider` intent/coercion/evidence-request producer。
  parser Task 47に依存。
- [ ] **Task 272:** non-Task-180 proof-skeleton/justification producer。
- [ ] **Task 273:** registration-item/correctness/initial-obligation intake producer。
- [ ] **Task 274 (blocked-reserved):** accepted verifier/artifact-status import/
  activation adapter。canonical authorityがupstream owner/schema/authentication
  rule/testsを命名するまで実行不能。
- [ ] **Task 275:** source-derived cluster-closure trace producer。
- [ ] **Task 276:** source-derived reduction/normalization trace producer。
- [ ] **Task 277:** direct template role/actual/guard producer。missing
  scheme/theorem roleはexecutable task外のexternal Gate S1で、Task 49はS1にも
  gateされる。
- [ ] **Task 278:** ordinary/template overload input-to-selection producer。
- [ ] **Task 279:** redefinition/notation target/coherence/refinement producer。
  dependency cycleなしにTask 278 ordinary-root resultをconsumeする。

各taskはfamilyを適用可能な`TypedAst`/`ResolvedTypedAst` tableまでtransactionally
projectし、実`mizar-test` Task-10 caseがconsumeする。未消費DTO、placeholder
runner、docs-only implementation commitはproducer taskを満たさない。

## Task 257B2 frozen-contract addendum

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 257B3 frozen-contract addendum

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 257C1 frozen-contract addendum

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 255C1 frozen-contract ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 257C2 frozen-contract ledger

- [x] unchanged 191-byte source/hash、direct condition-wrapperからinner
  equalityへのrelation、exact range、built-in equality identity、imported
  `++` provenance exclusionをfreeze。
- [x] Task-252 `4/0/4`、Task-253 `1/0/1/2/2`、Task-255
  `1/0/1/1/1/1/2`、Task-256 `1/0/0/0/0/0/0/2/2`、condition-formula
  association 1件をfreeze。
- [x] Task-257B APIを変更しないdedicated immutable handoff/producer/table、
  dependency fingerprint 4件、deterministic debug、typed/resolved ownership、
  validation、rollback、compatibility boundaryをfreeze。
- [x] existing fixture/sidecar reuse、future trace row 1件、exact test
  matrix、unchanged 419-case/pass-fail/active count、projected plan
  `419/386`/type `252/240`、semantic deferral、audit impactをfreeze。
- [x] missing contractを`design_drift`、implementationをbounded
  `source_drift`/`test_gap`、committed Task-256 condition-container
  rejectionをseparate authority-backed `source_drift`、stale Task-255C1
  umbrella checkboxをresolved `design_drift`、origin driftをreport-only
  `repo_metadata_conflict`にclassify。
- [x] separate Task-256C1 condition-container compatibility prerequisiteを
  freeze/reviewし、documentation-only commitを作る。
- [x] Task-256C1を両install order/strict arbitrary-overlap rejection込みで
  own commitにimplement/verify。
- [ ] Task-256C1後、fresh parser/resolver/API、both-install-order、
  count/test-list/production/CLI-hash preflightを行い、Task 257C2だけを
  separate logical task/commitでimplement。

## Checker Task 256C1 frozen-contract ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 256C1 implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 257C2 implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 257C3 frozen-contract ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 257C3 implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3 frozen-contract ledger

- [x] final-LF 104-byte source/hash、49-node/root-48 parser range、
  public/exported theorem provenance `[2,1]`、resolver companionなしをfreeze。
- [x] Task-48 `2/1/0`、Task-252 `5/5/0`、Task-256
  `2/0/0/0/0/0/0/4/4`、formula-only base `1/2/2/2/2`、atomic edgeからの
  exact term-2 exclusionをfreeze。
- [x] public witness ID/input/row/table/handoff/producer/error、
  fingerprint 2件、deterministic debug、direct binding context、unnamed
  primary target 1件をfreeze。
- [x] dense base IDs 0/1 + global ordinals 0/2、witness
  source/within-take ordinals 1/0、paired unique `[0,1,2]` validationをfreeze。
- [x] typed/final pair-only ownership、全profile/source-family exclusion、
  rollback/replay、checker tests 4本、runner tests 5本、empty semanticsを
  freeze。
- [x] 全fixture、sidecar、expectation、trace row/status/count、active route、
  source、test list、hashをlibraries `346/379`、runner 30 paths /
  36,479 linesで不変に保持。
- [x] closed contractを`design_drift`、future codeをbounded
  `source_drift`、testsを`test_gap`と分類し、blocking protocol
  disagreementなし。
- [x] 本dedicated documentation commitとfresh parser/resolver/lower-API/
  count/hash preflight後にTask 258B3だけをimplementした。librariesは
  `350/384`、changed hash/lineはimplementation resultで再測定済み。
- [x] fresh-inventoryし、Task 258B3N named-primary witness transportだけを
  freeze。exact 107-byte/51-node source、`1 witness / 1 name` table、B3
  compatibility、no binding/semantics、checker/runner tests 4/5本、
  unchanged baselineを確定。
- [x] dedicated documentation commitとfresh parser/resolver/lower/count/hash
  preflight後、Task 258B3Nだけをimplement。exact dense witness-name
  transport、checker tests 4本、runner tests 5本がpassし、libraryは
  `354/389`。semantic/corpus activationはない。
- [x] broad Task 258B3Mをexact reserved-variable B3M1と
  non-reserved-variable/other-term B3M2へ分解。
- [x] Task 258B3M1だけをfreeze: exact 113-byte/56-node mixed
  two-witness source、Task-252 `6/6/0`、base/witness/name
  `1/2/2/2/2` + `2/1`、shared source ordinal 1、dense ordinals 0/1、
  no new API/semantics、tests 4/5本、unchanged baseline。
- [x] documentation commitとfresh parser/resolver/lower/count/hash
  preflight後にfrozen Task 258B3M1だけをimplement。
- [x] Task 258B3M2をexact unnamed-numeral B3M2Aとremaining other-term
  B3M2BへdecomposeしてからTask 258B4を選ぶ。
- [x] Task 258B3M2Aだけをfreeze: final-LF 107-byte/49-node source、
  Task-252 `5/4/1`、base/witness/name `1/2/2/2/2` + `1/0`、numeric
  request ownership、new API/semanticsなし、tests 4/5本、unchanged
  baselines。
- [x] documentation commitとfresh parser/resolver/lower/count/hash
  preflight後にfrozen Task 258B3M2Aだけをimplement。
- [x] Task 258B3M2Bをexact parenthesized B3M2B1とremaining
  authority-valid B3M2B2へdecompose。
- [x] B3M2B1 documentation prerequisiteをfreezeし、implementationと
  B3M2B2をseparate taskに維持。
- [x] frozen B3M2B1をdocs commit/fresh preflight後にimplement。
- [x] Task 258B3M2B2をexact nested-parenthesized B3M2B2Aとremaining
  authority-valid B3M2B2Bへdecompose。
- [x] Task 258B3M2B2Aだけをfreezeし、implementationをseparateに維持。
- [x] documentation commitとfresh parser/resolver/lower/count/hash
  preflight後にfrozen Task 258B3M2B2Aをimplement。
- [ ] B3M2B2B remaining witness-term shapesをfreeze/implementしてから
  Task 258B4を選ぶ。

Completion evidence: [central Task-258B3N historical contract](../../task_contracts/ja/258B3N.md#completion-evidence)。

## Checker Task 258B3M1 documentation ledger

Completion evidence: [central Task-258B3M1 historical contract](../../task_contracts/ja/258B3M1.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2A documentation ledger

Completion evidence: [central Task-258B3M2A historical contract](../../task_contracts/ja/258B3M2A.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B1 frozen-contract ledger

Completion evidence: [central Task-258B3M2B1 historical contract](../../task_contracts/ja/258B3M2B1.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2A frozen-contract ledger

Completion evidence: [central Task-258B3M2B2A historical contract](../../task_contracts/ja/258B3M2B2A.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B1P frozen lower-prerequisite ledger

Completion evidence: [central Task-258B3M2B2B1P historical contract](../../task_contracts/ja/258B3M2B2B1P.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B1A frozen-contract ledger

Completion evidence: [central Task-258B3M2B2B1A historical contract](../../task_contracts/ja/258B3M2B2B1A.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B1B1P frozen-prerequisite ledger

Completion evidence: [central Task-258B3M2B2B1B1P historical contract](../../task_contracts/ja/258B3M2B2B1B1P.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B1B1 frozen-contract ledger

Completion evidence: [central Task-258B3M2B2B1B1 historical contract](../../task_contracts/ja/258B3M2B2B1B1.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B2P frozen-prerequisite ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B2P implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B2A frozen-contract ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B2A implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B2BP frozen-contract ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B2BP implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B2B frozen-contract ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B2B implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B2CP frozen-prerequisite ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B2C frozen-contract ledger

Completion evidence: [central Task-258B3M2B2B2C historical contract](../../task_contracts/ja/258B3M2B2B2C.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B3P frozen-contract ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B3P implementation-closure ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B3A frozen-contract ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B3A implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B3B frozen-contract ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 258B3M2B2B3B implementation completion

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B3C frozen-contract ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B3C implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B3D frozen-contract ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B3D implementation ledger

- [x] frozen checker 3 + runner 4 Rust consumersだけをimplementし、両
  `source_set_term.rs` ownersをpreserve。
- [x] exact checker 4 + runner 5 testsと
  `32/70/44/72/62/21` field matricesをimplement。
- [x] exact source/resolver/owner/graph、all-family isolation、
  replay/rollback/clone、empty semanticsをcover。
- [x] test-sufficiency reviewを**NO FINDINGS**でcomplete。
- [x] focused `4/4 + 5/5`、checker/runner packages
  `406+15` / `466+3/14/137/2/21`、format、full ClippyをPASS。
- [x] final module sizes、production/test hashes、unchanged 5 CLI hashes、
  authority/trace/active/semantic no-opをrecord。
- [x] independent implementation reviewを**NO FINDINGS**でcomplete。
- [x] stale implementation-review stateのMedium `design_drift`、
  24-order wordingのLow、EN qua-edge table wordingのLowをfix。
- [x] final source/documentation consistency、bilingual、documentation/
  boundary repeatsを**NO FINDINGS**でcomplete。
- [x] checker/runner packages、format、full Clippy、full workspace tests、
  5 CLI/count/hash final rerunsをPASS。
- [x] independent final read-only quality reviewを**NO FINDINGS**、全9
  hard gates PASS、score capなし、valid `100/100`
  （`20/20/15/15/10/10/5/5`）でcomplete。
- [x] metadata CLI warnings/errors `23/0`とlarge repeated-test diff review
  volumeをnonblocking residualとしてrecord。
- [ ] exact synchronized implementation scopeだけをstageしcached diffを
  inspect。
- [ ] implementation commit 1件をcreate。
- [ ] clean post-commit/stash invariantsをverifyし、next dependency-minimal
  taskをfresh inventory。

## Checker Task 258B3M2B2B3E documentation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B3E implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B4A documentation prerequisite

Completion evidence: [central Task-258B4A historical contract](../../task_contracts/ja/258B4A.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B4B documentation prerequisite

Completion evidence: [central Task-258B4B historical contract](../../task_contracts/ja/258B4B.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B4C documentation prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B4C lower-stage prerequisite ledger

Completion evidence: [central Task-258B4C historical contract](../../task_contracts/ja/258B4C.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B5A frozen-contract documentation prerequisite

Completion evidence: [central Task-258B5A historical contract](../../task_contracts/ja/258B5A.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B5B frozen-contract documentation prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B5B lower-stage prerequisite

Completion evidence: [central Task-258B5B historical contract](../../task_contracts/ja/258B5B.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B5C frozen-contract documentation prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## B5C R-032A preflight overlay

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## B5C R-032B lint-policy preflight overlay

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 258B5C active implementation

- [x] private declaration-symbol consumerでexact spec-derived fail case 2件と
  covered trace row 2件を追加し、public codeをemptyに維持。
- [x] omitted metadata count consumerのfour `5 -> 7` assertionを修正し、
  `test_expectation_drift`/scope `design_drift`に分類。
- [x] 全checker source/API/semantic resultをno-opに保ち、R-G007内の
  confinement requirement 2件だけをclose。
- [x] findings-free test/implementation/source-documentation reviewと
  focused/crate/workspace/count/hash verification gateを完了。
- [x] final qualityを**NO FINDINGS**、全9 hard gates PASS、score capなし、
  valid `100/100`（`20/20/15/15/10/10/5/5`）で完了。
- [ ] task-only commit、post-commit invariant、next-task fresh inventoryを
  完了。

## Checker Task 259 Frozen-Contract Documentation Prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 248 Two-Parameter Profile-Extension Documentation Prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 259 Frozen-Contract Correction Prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 259 active implementation

Completion evidence: [central Task-260 historical contract](../../task_contracts/ja/260.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 249R definition-return documentation prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 249R active implementation

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 260 active implementation

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 261 frozen-contract documentation prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 261 implementation

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 262 frozen-contract documentation prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 263 preflight lower prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 249S standalone structure-member type prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 263 structure-definition intake

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 264 lower-prerequisite sequence

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 249PI property-type composition prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 269A named-witness binding slice

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 269B mixed-witness binding increment

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 269CP isolated proof-`let` lower prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 269C binding-only proof-`let` transaction

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 269CT proof-`let` source-type prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 269GP proof-`given` lower prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 269GS canonical `given` scope reconciliation

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 269G proof-`given` binding consumer

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 269GT proof-`given` source-type consumer

Completion evidence: [central Task-269GT historical contract](../../task_contracts/ja/269GT.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 269GUP proof-`given` use-profile binding prerequisite

Completion evidence: [central Task-269GUP historical contract](../../task_contracts/ja/269GUP.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 269GUPT source-type prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 269GU later-use term/reference prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 269GCP Given-condition lower prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 269GC Given-condition binding consumer

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 269GCT Given-condition source-type consumer

Completion evidence: [central Task-269GCT historical contract](../../task_contracts/ja/269GCT.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 269GCU given-condition term/reference consumer

Completion evidence: [central Task-269GCU historical contract](../../task_contracts/ja/269GCU.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Checker Task 269SDP descendant/set lower prerequisite

- [x] clean GCU commit、origin `0/19` report-only、stash不変を確認する。
- [x] GCU stale statusを`design_drift`、missing lower/selector/testsを
  `design_drift`/`source_drift`/`test_gap`、Ch.4/15 `set`矛盾をSDPには
  nonblocking・captureにはblockingな`spec_gap`として分類する。
- [x] 180-byte source、68-node Surface、shell/resolver、range/hash、private
  ABI/debug、4 files/tests、zero-credit境界、exitを凍結する。
- [x] EN/JA review NO FINDINGS、docs 9 gates `100/100`、42 Markdownを
  prerequisite `f468b0163bb00726dca9b356f48790c73bb1fe98`としてcommit。
- [x] fresh preflight後lower-only 4 files/4 testsを実装し、focused testsと
  test/implementation reviewをNO FINDINGSで完了。
- [x] source/docs再review、full verification、final qualityを全9 hard
  gates PASS、score capなしの`100/100`で完了する。
- [ ] exact staging/implementation commit後にdescendant context/binding
  taskをfresh-selectする。

Completion evidence: [central Task-269SDP historical contract](../../task_contracts/ja/269SDP.md#completion-evidence)。

## Checker Task 269SDC descendant context/binding prerequisite

Completion evidence: [central Task-269SDC historical contract](../../task_contracts/ja/269SDC.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 269SDT

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 269SDU Descendant Given Occurrence/Reference

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 277A Direct Parser-Origin Template Transport

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 277B-L Template Type-Parameter Association

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 277C Fraenkel structural composition

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 257C4A Fraenkel generator binding context

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 257C4B Fraenkel generator bound-use transport

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 257C4C0 nested Fraenkel capture test intent

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 257C4C1 explicit-import lexical admission

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 257C4C3 nested binder/mapper-use transport

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 257C4C4 nested mapper primary transport

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 257C4C5 nested capture-identity receipt

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 257C4C6 nested capture-identity installation

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 257C4C7 two-capture prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 257C4C8 normalized nested capture graph

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 33C opaque capture-graph owner receipt

- [x] User-selected checker-owned scalar receipt、by-value C4C8/Task33R dependency、exact
  public API、graph-dependency→owner-dependency→association precedenceをpaired
  [Task33C contract](../../task_contracts/ja/CHECKER-FRAENKEL-CAPTURE-GRAPH-OWNER-33C.md)へfreezeする。
- [x] Rust edit前にindependent specification/equivalenceとbilingual/boundary reviewを
  **NO FINDINGS**まで完了する。
- [x] Frozen handoff/error/producer、checker test 4件、public-enum guard、private real-fixture
  probe 1件だけをimplementする。
- [x] C4C4/C4C5/C4C6、Typed/Resolved/Core ownership、parameter/order/GeneratedOrigin
  deferral、active route、diagnostic、protected artifact、semantic coverage、Task277B zero creditを
  preserveする。
- [x] Post-source review、verification、final-quality hard gateを**NO FINDINGS**、`9/9`
  PASS、valid uncapped `100/100`で完了する。
- [ ] Exact task-only commit、clean postcommit proof、fresh inventoryを完了する。

## Task 264C property carrier identity transport

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

## Task 264D equals selector identity association

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/checker_todo_sections.md).

