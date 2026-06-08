# AI親和性強化勧告の精査メモ

> ステータス: 検討用ワーキングノート（非規範 / non-normative）。
> 対象: 「Mizar evo の AI 親和性を高めるため、AI agent 操作面の仕様を追加すべき」という外部勧告の妥当性評価。
> 作成日: 2026-06-08。

## 0. 結論（先に）

- 勧告の **方向性は妥当かつ必要**。現行仕様は「AI が読める成果物（passive AI friendliness）」は強いが、「AI が安全に編集・検証・再試行する手順（active AI operability）」が未定義であり、これは §1 で掲げる三本柱の一つ **AI-readiness** を能動側で未達のまま残している。
- ただし勧告が提案する **packaging（規範的な「第25章 AI Agent Interface」を今すぐ追加し、JSON wire schema や OAuth scope まで凍結する）は時期尚早かつ一部冗長**。
- 推奨は **「概念枠組みと安全境界ポリシーは今採用、wire-format の凍結と MCP 束縛・ベンチマークは下層アーティファクト安定後に遅延導入」** の二段構え。理由は §5–§7。

## 1. 勧告の要旨

AI 親和性の核心は「AI に証明の正しさを委ねる」のではなく、正しさは verifier / ATP / kernel が担保し、AI は **検索・編集・計画・説明** に使う設計にすること。勧告はこれを以下に分解する。

- passive（読める）/ active（操作できる）の二段階に分け、後者の標準手順が欠けていると指摘。
- 提案要素: library 横断 `theorem-index.jsonl` / `symbol-index.jsonl`、`mizar context obligation`（文脈抽出＋トークン予算）、JSON patch protocol（`verify.patch` の dry-run → `apply.patch`）、proof-repair 状態機械、edit の安全度分類（Green/Yellow/Red）、MCP の Resources/Tools/Prompts 束縛と authorization scope、proof witness 形式の確定、診断への `recovery_actions`、AI 用 dataset export とベンチマーク。
- まとめて規範的な **「第25章 AI Agent Interface and Context Protocol」** を新設すべき、と提言。

## 2. 現行仕様の実地確認

勧告が引用する現行仕様の記述を実地検証した（[doc/spec/](../spec/) / [doc/design/](../design/)）。事実関係はおおむね正確。

### 2.1 既に存在し勧告も「強み」と認める土台

| 要素 | 所在 | 補足 |
|---|---|---|
| `*.mizir.json`（inferred_type / resolved_symbol / inserted_coercions / active_thesis / overload_resolution 等） | [§23.5.1](../spec/en/23.package_management_and_build_system.md) | 意味解釈済みプログラムの外出し。AI 入力として AST 以上の価値 |
| `missing_facts.json`（goal / triage / 候補 lemma / score / FQN / retry） | §21, §22, §23 | `by`-clause 補完の primary input |
| `used_axioms`（kernel が証明証明書から抽出） | §23.5.1 | citation の精密化 `refine` / `minimize` の基盤 |
| `explain-type/attribute/overload/qua/failure` API | [§23.7.8](../spec/en/23.package_management_and_build_system.md) | **既に「incremental query 向け・machine-readable・AI context window を溢れさせない」と明記** |
| bounded local subgraph 抽出 | §23.7.5 | full graph ではなく局所部分グラフを返す設計が既にある |
| 安定診断コード＋semantic name＋machine-readable fix＋triage 分類 | [§22](../spec/en/22.error_handling_and_diagnostics.md) | tools は numeric code を key にする方針も明記 |
| reasoning boundary / minimal trusted kernel | [architecture/08](../design/architecture/en/08.reasoning_boundary.md) | soundness を kernel が独立担保。AI 提案を安全に受理する前提 |

### 2.2 欠落（勧告の指摘は正しい）

- **MCP**: [§1.4](../spec/en/01.introduction.md) に「MCP を採用しライブラリ構造・定理検索・検証状態を AI agent に公開する」と **意図表明があるのみ**。束縛仕様（URI scheme / tool 一覧 / scope）は存在しない。
- **patch protocol / `verify.patch` / AI agent interface**: §1 以外に出現せず、**実体仕様なし**。
- **proof_witness**: 形式は tentative / under consideration（higher-order instantiation・term sharing・variable scoping 未確定）。
- **theorem 検索用機械可読インデックス**: ドキュメント生成に [`search.json`（§24）](../spec/en/24.documentation_generation.md) はあるが、FQN・kind・一行要約・signature のみの **HTML 全文検索用の薄いもの**。勧告の canonical/normalized/FOL/symbols を持つ `theorem-index.jsonl` とは別物で、これは未整備。

## 3. 必要性の評価

- **必要（直す価値あり）**: active operability の不在は、AI-readiness 柱の能動側を未達のまま残す。素材（artifact）は揃っているのに、AI が「どの API を・どの粒度の patch で・どの条件なら採用してよいか」の **契約（tooling contract）** がないため、各 IDE / agent が場当たり的に実装することになり、安全境界も保証されない。
- **語の設計を崩さない点が良い**: 勧告は構文追加ではなく、安定構文・FQN・kernel soundness を保ったまま artifact / context / patch / verify を厚くする方向。§1 の「disciplined expressivity」「minimal trusted kernel」と整合する。

## 4. 新規性 vs 既存の重複（切り分け）

勧告の提案には **既存機能の再パッケージにすぎないもの** と **真に新規のもの** が混在する。ここを区別しないと「第25章」が冗長になる。

- **ほぼ既存（増分のみ）**:
  - 診断の `recovery_actions` → §22 の machine-readable fix / candidates / triage の構造化拡張にすぎない。
  - `mizar context obligation` → §23.7.5 の bounded extraction を obligation 単位で束ねた CLI 表面。能力としては新規でない。
  - citation の `refine` / `minimize`、`used_axioms` → 既存。
- **真に新規（仕様の空白）**:
  1. **patch 提案＋ dry-run 検証プロトコル**（`verify.patch` を `apply.patch` の前に必須化）。
  2. **edit 安全度分類 Green/Yellow/Red**（axiom 追加・定理文の弱化・proof_witness 直接編集・`require_kernel_certificates=false` 化を Red として AI 単独禁止）。
  3. **MCP 束縛と authorization scope**（`mizar:read/explain/verify/write/refine/publish/unsafe-edit`）。
  4. **library 横断 theorem/symbol index**（symbolic 検索＋自然言語検索の二系統）。
  5. **proof-repair 状態機械**と **dataset export / ベンチマーク**。

## 5. 懸念点（時期尚早リスク）

1. **アーティファクトが固まる前に契約を凍結する危険**。tooling contract は、それが包む下層形式（`*.mizir.json`、proof_witness、obligation ID）に **追従すべきで先行すべきでない**。proof_witness は明示的に tentative。ここで patch の JSON schema や proof witness 形状を規範化すると、後で破壊的変更が要る。
2. **実装初期段階**。現実装はまだ verifier 本体・artifact 生成が未成熟で、検証器が無いと patch dry-run も index も実体を持てない。
3. **編集をまたいだ stable anchor の仕様が未確立**。`VcId` の決定的割り当て自体は architecture 側に規定がある（[07.vc_generation.md:445-448](../design/architecture/en/07.vc_generation.md#L445-L448): sorted seed list ＋ per-seed expansion index から決定的に採番）。問題はその先で、patch protocol が依存する `obligation_id: vc-42` は **インクリメンタル編集をまたいでも同一 obligation を指す安定アンカー**である必要があるが、現行の index ベース採番は obligation の挿入・削除でずれるため、この cross-edit 安定性は未規定。これ自体を先に仕様化する必要がある。
4. **文書配置の問題**。勧告自身が「これは言語仕様ではなく tooling contract」と認めている。にもかかわらず言語リファレンスに「第25章」として入れると関心が混在する。**[doc/design/architecture/](../design/architecture/)（12.diagnostics_and_lsp や 09.atp_interface の隣）または独立の tooling spec が適切**。
5. **重量過多の部分**: ローカル stdio 開発用ツールに OAuth 2.1 scope は過剰。remote server 限定で十分。ベンチマークは std library / corpus 前提なので現状作れない。

## 6. 推奨アクションと優先順位

**今すぐ採用（安価・高価値・下層に非依存）**

- **安全境界ポリシーの明文化**: edit を Green/Yellow/Red に分類し、Red（axiom 追加 / 定理文の弱化・変更 / requires・ensures の弱化 / proof_status・proof_witness の手編集 / kernel 設定の緩和）を AI 単独禁止と規定。[architecture/08 reasoning_boundary](../design/architecture/en/08.reasoning_boundary.md) と `require_kernel_certificates` ポリシーの自然な延長。
- **MCP scope の語彙だけ先に定義**（`read/explain/verify/write/refine/publish/unsafe-edit`）。MCP 束縛本体なしでも権限ポリシーとして機能する。
- **passive 側の薄い穴埋め**: `search.json` を一般化し、per-library の機械可読 index を export。素材は **既存 artifact（`*.mizir.json` の `exports` の FQN・source_range、obligations の `used_axioms`）／source comments／documentation extraction から得られるもの、または追加抽出すべきもの**（head/all symbols・theorem statement surface・doc などは現状 mizir.json の明示フィールドではなく、抽出経路の整備が要る）を組み合わせる。FOL/normalized 形は後回し。

**設計文書として枠組みを記述（規範化はしない）**

- patch proposal / `verify.patch` の dry-run loop と proof-repair 状態機械を **roadmap / 非規範 annex** として記述。インターフェースの意図は固定しつつ、wire schema は「下層安定後に凍結」と明記。
- 前提条件として **編集をまたいだ stable obligation anchor の要件**を先に起票（`VcId` の決定的採番は既存だが、cross-edit 安定アンカーは別途規定が要る）。

**遅延（下層安定・実装進捗待ち）**

- proof_witness 形式の確定（binder_id ベースの per-axiom scoping は方向として妥当）。
- MCP 束縛本体（URI scheme / Resources・Tools・Prompts）と OAuth scope（remote のみ）。
- theorem-index の FOL/TPTP・embedding 系統、dataset export、AI ベンチマーク（corpus 整備後）。

## 7. まとめ

勧告の **診断（passive 強・active 欠落）と方向（言語は崩さず tooling contract を厚くする）は正しく、採用すべき**。一方で **「規範的第25章を即時新設し schema/OAuth まで凍結」は時期尚早**で、(a) 既存機能との冗長、(b) 下層アーティファクト未確定、(c) 言語仕様への関心混在、という問題がある。

したがって本プロジェクトの取り込み方は:

1. 安全境界（Green/Yellow/Red）と scope 語彙は **今、ポリシーとして明文化**。
2. patch / context / repair-loop は **architecture 配下の非規範設計文書**として枠組みを記述し、wire-format は遅延凍結。
3. MCP 束縛・proof_witness 確定・ベンチマークは **下層安定後**に段階導入。

この形なら、構文を AI 向けに崩さず、Mizar らしい安定構文・FQN・kernel soundness を保ったまま AI-readiness 柱の能動側を埋められる。
