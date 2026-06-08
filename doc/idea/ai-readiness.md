# AI親和性を高める方策

AI親和性の核心は **「AIに証明の正しさを委ねる」のではなく、AIを“検索・編集・計画・説明”に使い、正しさは Mizar evo verifier / ATP / kernel が担保する設計にすること**です。

Mizar evo の現在仕様は、この方向にかなり近いです。ただし、まだ **AIが外部から安全に操作するための protocol 仕様** が足りません。つまり、現在は「AIが読みやすい成果物」はあるが、「AIが編集・検証・再試行する標準手順」は未完成、という評価です。

---

## 1. まず方向性を二段階に分けるべきです

AI親和性には、実は二種類あります。

第一は **passive AI friendliness**、つまり AI が Mizar evo のコードや検証結果を読めることです。

第二は **active AI operability**、つまり AI が Mizar evo プロジェクトに対して、小さな変更を提案し、検証し、失敗理由を読み、再修正できることです。

現在の Mizar evo 仕様は、前者にはかなり強いです。仕様上、verifier は `*.mizir.json` に inferred type、resolved symbol、inserted coercions、active thesis、overload resolution summary、proof obligations などを出力する設計になっており、これは IDE と AI tool の主要入力だと明記されています。([GitHub][1])

一方で、後者、つまり **AI agent がどの API を呼び、どの単位で patch を作り、どの条件なら採用してよいか** は、まだ独立した仕様章としては薄いです。

したがって、深掘りすべき本線は次です。

> Mizar evo を「AIが読める形式言語」から、
> **AIが安全に操作できる形式数学環境** にする。

---

## 2. 現在仕様の強み：すでに土台はかなりある

現在仕様で特に良いのは、AIが必要とする暗黙情報を verifier artifact として外に出す設計です。

Mizar 系の難しさは、表面構文だけを見ても、AIには次の情報が分からないことです。

```text
この + はどの functor か
この型はどこまで attribute closure されているか
どの qua coercion が暗黙挿入されたか
現在の thesis は何か
この proof obligation はどの lemma を必要としているか
overload の候補は何個あったか
なぜこの候補が選ばれたか
```

Mizar evo はここにかなり正面から対応しています。`*.mizir.json` の `expressions` には `expr_id`, `source_range`, `inferred_type`, `resolved_symbol`, `template_args`, `inserted_coercions`, `active_thesis`, `overload_resolution` が含まれる設計です。([GitHub][1])

これは非常に重要です。AIにとって、これは「Mizarソースの AST」以上の価値があります。なぜなら、AIが欲しいのは単なる構文木ではなく、**verifier が見ている意味解釈済みのプログラム**だからです。

また、未証明 obligation について `missing_facts.json` を出し、goal、triage、候補 lemma、score、FQN、source、conclusion、retry_succeeded を持たせる設計になっています。仕様上、これは AI-assisted `by`-clause completion の primary input とされています。([GitHub][1])

さらに、証明成功時には `used_axioms` を記録し、それを `mizar refine` や `mizar minimize` に使って citation を精密化する設計もあります。これはAI生成証明の「後片付け」に非常に効きます。([GitHub][1])

このあたりは、かなり良いです。

---

## 3. ただし “missing_facts” だけでは AI theorem search として足りない

現在の `missing_facts.json` は、失敗した proof obligation に対して候補 lemma を出すものです。これは証明修正には効きます。

しかし、AIがより能動的に証明を組み立てるには、**library-wide theorem index** が必要です。

つまり、各 theorem / definition / registration について、次のような検索用メタデータを公式 artifact として出した方がよいです。

```json
{
  "schema_version": "1.0",
  "kind": "theorem",
  "fqn": "std.nat.add_comm",
  "label": "add_comm",
  "module": "std.nat",
  "source_range": "120:1-122:4",

  "statement": {
    "surface": "for a, b being Nat holds a + b = b + a",
    "canonical": "for a: std.nat.Nat, b: std.nat.Nat holds std.nat.add(a,b) = std.nat.add(b,a)",
    "fol": "... optional normalized FOL/TPTP form ..."
  },

  "symbols": {
    "head_symbols": ["std.nat.add", "="],
    "all_symbols": ["std.nat.Nat", "std.nat.add", "="],
    "types": ["std.nat.Nat"]
  },

  "dependencies": {
    "used_axioms": ["std.nat.add_assoc", "std.nat.zero_add"],
    "imports": ["std.nat.core"]
  },

  "search_text": {
    "doc": "Commutativity of addition on natural numbers.",
    "keywords": ["addition", "commutative", "Nat"],
    "latex": "a + b = b + a"
  },

  "trust": {
    "proof_status": "kernel_verified",
    "package": "std",
    "version": "1.0.0"
  }
}
```

現在の documentation generator は、source comments、item labels、type signatures、theorem statements、`exports`、`@latex` を使って HTML を生成する設計になっています。([GitHub][2]) しかし、AI向けには HTML だけでなく、**`theorem-index.jsonl`** のような検索専用 artifact が必要です。

特に重要なのは、statement を複数の形で持つことです。

```text
surface       人間が書いたMizar表記
canonical     FQNを展開した安定表記
normalized    交換・結合・binder名などを正規化した検索用表記
fol/tptp      ATPやsymbolic search用の論理表現
embedding     自然言語検索用のテキスト
```

これがないと、AIは「`+` の可換性」を探したいときに、表面記法・synonym・overload・module alias に振り回されます。

---

## 4. AI操作の中心は “patch protocol” にすべきです

AIにソース全体を書き換えさせるのは危険です。
Mizar evo で本当に必要なのは、**小さな検証可能 edit** を標準化することです。

たとえば、AIがやるべき操作は次のような粒度です。

```text
by clause に lemma を追加する
ambiguous overload に qua を挿入する
@proof_hint を追加する
assert を一つ挿入して goal を分割する
import を一つ追加する
bulk citation を refine する
不要 citation を削除する
```

これらを自由文ではなく、JSON patch として扱います。

例：

```json
{
  "schema_version": "1.0",
  "patch_id": "patch-20260605-001",
  "target": {
    "file": "src/foo.miz",
    "source_hash": "sha256:a3f4...",
    "obligation_id": "vc-42"
  },
  "edit": {
    "kind": "replace_by_clause",
    "source_range": "35:20-35:45",
    "old_text": "by add_assoc;",
    "new_text": "by add_assoc, add_comm, add_0_right;"
  },
  "intent": {
    "diagnostic_code": "E0301",
    "triage": "missing_lemma",
    "expected_to_close": ["vc-42"]
  }
}
```

これに対して、verifier は次のような応答を返します。

```json
{
  "schema_version": "1.0",
  "patch_id": "patch-20260605-001",
  "status": "accepted_by_verifier",
  "changed_obligations": [
    {
      "obligation_id": "vc-42",
      "old_status": "open",
      "new_status": "kernel_verified",
      "used_axioms": [
        "std.nat.add_comm",
        "std.nat.add_0_right"
      ]
    }
  ],
  "new_diagnostics": [],
  "artifact_hashes": {
    "mizir": "sha256:...",
    "proof_witness": "sha256:..."
  }
}
```

逆に失敗したら、

```json
{
  "schema_version": "1.0",
  "patch_id": "patch-20260605-001",
  "status": "rejected_by_verifier",
  "new_diagnostics": [
    {
      "code": "E0201",
      "semantic_name": "overload.ambiguous",
      "source_range": "35:29-35:34",
      "explanation_ref": "build/diagnostic-explanation/foo.json#E0201-35-29",
      "suggested_next_actions": [
        {
          "kind": "insert_qua",
          "target_range": "35:29-35:34",
          "candidates": ["AddStr", "MulStr"]
        }
      ]
    }
  ]
}
```

この形にすると、AIは「文章を生成する存在」ではなく、**検証器に対して小さな仮説を投げる proof-repair loop の一部**になります。

ここが一番大事です。

---

## 5. MCPでは “Resources / Tools / Prompts” に分けると自然です

Mizar evo 仕様では、MCP integration により library structure、theorem search、verification status を AI agents に標準 context として公開すると述べています。([GitHub][3])

これを具体化するなら、MCP の三分類に沿わせるのが自然です。MCP では、Tools はモデルが外部システムを呼ぶための機能で、各 tool は名前と schema を持ちます。([Model Context Protocol][4]) Resources は、ファイルやDB schemaのような context data を URI で公開する仕組みです。([Model Context Protocol][5]) Prompts はユーザーが明示的に選べる reusable prompt template です。([Model Context Protocol][6])

Mizar evo では、こう切るとよいです。

### MCP Resources：読み取り専用の文脈

```text
mizar://source/src/foo.miz
mizar://artifact/src/foo.mizir.json
mizar://obligation/src/foo.miz#vc-42
mizar://diagnostic/src/foo.miz#E0201-35-29
mizar://symbol/std.nat.add_comm
mizar://theorem/std.nat.add_comm
mizar://module/std.nat
mizar://cluster-path/std.set.empty_to_countable
mizar://proof-witness/src/foo.miz#vc-42
```

Resources は原則 read-only です。AIはこれを読んで、必要な最小文脈だけを取得します。

### MCP Tools：検証器や検索器を呼ぶ操作

```text
mizar.search.theorem
mizar.search.symbol
mizar.explain.type
mizar.explain.overload
mizar.explain.failure
mizar.context.obligation
mizar.suggest.by_clause
mizar.verify.patch
mizar.apply.patch
mizar.refine.citations
mizar.minimize.citations
mizar.show.vcs
```

特に重要なのは `mizar.verify.patch` です。
`apply.patch` よりも先に、必ず dry-run 的な `verify.patch` を通すべきです。

### MCP Prompts：人間が選べる作業テンプレート

```text
prove current obligation
fix overload ambiguity
explain this proof failure
split goal into assert steps
refine citations
write doc comment for theorem
```

この分離にすると、AI agent は次のように動けます。

```text
1. resources/read で obligation を読む
2. mizar.search.theorem で候補補題を探す
3. mizar.explain.failure で失敗理由を読む
4. mizar.verify.patch で小さな edit を検証する
5. 成功した patch だけをユーザーに提示する
```

これが「AIが安全に操作できるMizar」の基本形です。

---

## 6. `mizar context` が最重要コマンドになる

AIに巨大な library 全体を渡すのは無理です。
したがって、AI向けには **context extraction** が必要です。

現在仕様には local subgraph extraction の考え方があり、AI tools と IDE には full global graph ではなく local subgraph を返すとされています。([GitHub][1]) これは非常に正しいです。

この考え方を theorem proving 全体に広げるべきです。

たとえば：

```bash
mizar context obligation --file src/foo.miz --id vc-42 --budget 8000 --json
```

出力：

```json
{
  "schema_version": "1.0",
  "context_kind": "obligation",
  "budget": {
    "requested_tokens": 8000,
    "estimated_tokens": 6420
  },
  "target": {
    "file": "src/foo.miz",
    "obligation_id": "vc-42",
    "source_range": "35:3-35:30",
    "goal": "s + i = i * (i + 1) div 2",
    "hypotheses": [
      "i <= n",
      "s = i * (i - 1) div 2"
    ]
  },
  "local_context": {
    "nearby_bindings": [],
    "local_lemmas": [],
    "imports": ["std.nat", "std.arith.div"]
  },
  "candidate_facts": [
    {
      "fqn": "std.nat.Nat_triangle_step",
      "rank": 1,
      "reason": "matches conclusion head and symbols",
      "statement": "..."
    }
  ],
  "diagnostics": [
    {
      "code": "E0301",
      "triage": "missing_lemma"
    }
  ],
  "allowed_patch_kinds": [
    "replace_by_clause",
    "insert_assert",
    "insert_proof_hint"
  ]
}
```

この `context` は、AIの入力そのものになります。

重要なのは、context に **許可される patch kind** まで含めることです。AIに何でも編集させず、「今は by clause 修正まで」「今は theorem statement の変更は禁止」などを明示します。

---

## 7. 証明修正 loop は状態機械として定義するとよい

AI proof repair は、自然言語対話ではなく状態機械にした方が安定します。

```text
OPEN_OBLIGATION
  ↓ context extraction
CANDIDATE_FACTS
  ↓ patch proposal
PATCH_DRY_RUN
  ↓ verifier
PATCH_ACCEPTED or PATCH_REJECTED
  ↓ if accepted
KERNEL_VERIFIED
  ↓ citation refinement
REFINED
  ↓ human review / commit
DONE
```

これを仕様化します。

各状態で AI が使ってよい操作を制限します。

| 状態                | AIができること                 | 禁止すべきこと             |
| ----------------- | ------------------------ | ------------------- |
| `OPEN_OBLIGATION` | context取得、補題検索           | ソース変更               |
| `CANDIDATE_FACTS` | patch案作成                 | theorem statement変更 |
| `PATCH_DRY_RUN`   | verifierに検証依頼            | 直接commit            |
| `PATCH_REJECTED`  | diagnosticを読んで再提案        | 同じpatchの無限再試行       |
| `KERNEL_VERIFIED` | citation refine          | 証明済みartifactの改ざん    |
| `DONE`            | human-readable summary生成 | proof validity の自称  |

AIの失敗で一番多いのは「目標式を弱める」「定理文を変える」「axiomを足す」「定義を都合よく変える」です。
だから、AI agent mode では edit kind を分類すべきです。

---

## 8. edit kind を安全度で分類する

私は、Mizar evo では AI edit を Green / Yellow / Red に分けるべきだと思います。

### Green：原則自動提案してよい

```text
by clause に lemma を追加
by clause から未使用 citation を削除
qua を挿入
@show_type / @show_resolution / @show_thesis を挿入
@proof_hint を挿入
コメント・doc comment を追加
mizar refine の適用
```

これらは意味を大きく変えにくいです。もちろん最終的には verifier が確認します。

### Yellow：検証成功しても人間確認が望ましい

```text
assert を追加して証明を分割
local lemma を追加
import を追加
registration を追加
cluster を追加
algorithm invariant を追加
```

これらは証明構造や探索空間を変えます。検証が通れば論理的にはよいですが、ライブラリ設計上の影響があるので人間確認が必要です。

### Red：AI単独では禁止

```text
axiom を追加
theorem statement を弱める
definition を変更
requires / ensures を弱める
proof_status を手で変更
proof_witness を直接編集
kernel設定を緩める
require_kernel_certificates = false にする
```

特に `axiom` 追加や theorem statement の変更は、AI proof repair では禁じるべきです。

Mizar evo の設計思想では、LLMやATPを使いつつも、soundness は minimal trusted kernel で守るという方針が明記されています。([GitHub][3]) したがって、AIが kernel の外側で何を提案してもよいが、**kernel の検査対象や信頼境界をAIが緩めることは禁止**、という線引きが必要です。

---

## 9. proof witness 仕様は早めに固めた方がよい

現在仕様では `proof_witness` が `*.mizir.json` から参照される外部ファイルとして示されていますが、その形式は tentative / under consideration と書かれています。特に higher-order instantiation、term sharing、variable scoping の扱いはまだ確定していないとされています。([GitHub][1])

ここはAI親和性にも効きます。

なぜなら、AIが「なぜこの lemma が効いたのか」を学ぶには、単に `used_axioms` だけでなく、

```text
どの axiom が
どの変数代入で
どの goal に
どの順番で
使われたか
```

が必要だからです。

推奨仕様はこうです。

```json
{
  "schema_version": "1.0",
  "obligation_id": "vc-42",
  "kernel": {
    "kernel_checked": true,
    "kernel_version": "0.1.0",
    "checker": "sat-substitution"
  },
  "goal_fingerprint": "sha256:...",
  "steps": [
    {
      "step_id": "s1",
      "kind": "axiom_instance",
      "axiom_fqn": "std.nat.Nat_triangle_step",
      "axiom_fingerprint": "sha256:...",
      "instantiation": [
        {
          "binder_id": "n",
          "term": "i",
          "term_fingerprint": "sha256:..."
        }
      ]
    }
  ],
  "used_axioms": [
    "std.nat.Nat_triangle_step"
  ]
}
```

ポイントは、変数名ではなく **binder_id / scoped variable** を使うことです。
証明証跡で `?n` のような名前だけを使うと、別 axiom の `?n` と衝突します。現在仕様でも、per-axiom binding にする必要があると述べられています。([GitHub][1])

この proof witness が安定すれば、AI用 training data も作りやすくなります。

---

## 10. diagnostics は “AI-readable recovery data” まで持たせるべき

現在仕様では、diagnostic は stable code を持ち、数字コードは再利用しない方針です。tools は message text ではなく numeric code を見るべき、とされています。([GitHub][7]) これは非常に良いです。

さらに、AI向けには diagnostic に `recovery_actions` を持たせるとよいです。

例：

```json
{
  "code": "E0201",
  "semantic_name": "overload.ambiguous",
  "source_range": "42:10-42:18",
  "message": "ambiguous overload",
  "machine_explanation": {
    "kind": "ambiguous_overload",
    "candidates": [
      {
        "fqn": "std.algebra.add_group.Product",
        "required_view": "AddStr"
      },
      {
        "fqn": "std.algebra.mul_monoid.Product",
        "required_view": "MulStr"
      }
    ]
  },
  "recovery_actions": [
    {
      "kind": "insert_qua",
      "range": "42:10-42:18",
      "replacement_template": "${expr} qua MulStr"
    },
    {
      "kind": "qualify_symbol",
      "replacement_template": "std.algebra.mul_monoid.Product(${args})"
    }
  ]
}
```

現在の explanation API は `explain-type`, `explain-attribute`, `explain-overload`, `explain-qua`, `explain-failure` を提供し、`explain-failure` は competing candidates、missing registrations、suggested fix を返す設計です。([GitHub][1]) これをさらに patch protocol と直結させるべきです。

つまり、

```text
diagnostic
  -> explanation
  -> recovery_action
  -> patch proposal
  -> verify.patch
```

という一本の流れを仕様化します。

---

## 11. theorem search は “自然言語検索 + symbolic search” の二系統が必要

Mizar evo の AI 検索では、embedding だけに頼るべきではありません。

形式数学では、自然言語的に近い theorem が論理的には無関係だったり、逆に名前が遠い theorem が証明には必須だったりします。

したがって theorem search は二系統にします。

### A. Symbolic search

```text
goal の head symbol
goal に出る FQN
型
attribute
conclusion pattern
rewrite direction
hypothesis shape
```

で検索する。

例：

```bash
mizar search theorem \
  --symbols std.nat.add,std.nat.div \
  --conclusion-head "=" \
  --type std.nat.Nat \
  --json
```

### B. Semantic / natural language search

```text
"triangle number induction step"
"commutativity of natural number addition"
"subgroup of abelian group is abelian"
```

で検索する。

このとき、検索対象テキストは source comment だけでは不足します。
検索用に合成された説明文が必要です。

```json
{
  "embedding_text": [
    "Theorem std.nat.Nat_triangle_step.",
    "For natural number n, the triangular sum formula advances from n to n+1.",
    "Statement: ...",
    "Uses symbols: addition, multiplication, division by 2, Nat."
  ]
}
```

これにより、AIは「名前を知らないが意味は分かる」補題を探せます。

---

## 12. import と namespace の設計はAIにかなり良い

Mizar evo の import は prelude に限定され、ファイル全体の active lexical environment が import prelude から一度構築される設計です。([GitHub][8])

これはAIには非常に有利です。途中で import が増える言語だと、AIは「この位置で何が見えているか」を毎回追わなければなりません。Mizar evo では、ファイル先頭の import closure を読めばよい。

また、module の physical file path と logical namespace が一対一対応し、FQN が file location から導かれる設計も良いです。([GitHub][8])

この方向は維持すべきです。

追加で欲しいのは、AI向けの import suggestion API です。

```bash
mizar search import --symbol Group --json
mizar search import --theorem add_comm --json
mizar suggest import --for-obligation vc-42 --json
```

出力：

```json
{
  "symbol": "add_comm",
  "candidates": [
    {
      "module": "std.nat",
      "fqn": "std.nat.add_comm",
      "already_visible": false,
      "import_statement": "import std.nat;",
      "risk": "low"
    }
  ]
}
```

AIが勝手に広い prelude を import しすぎると探索空間が膨らむので、`risk` や `added_visible_symbols_count` も出すとよいです。

---

## 13. overload / cluster explanation は Mizar evo の独自価値になる

Mizar evo の強みは、単に theorem proving をすることではなく、Mizar 的な soft type / attribute / cluster / registration の世界を説明できることです。

現在仕様では、cluster resolution graph、registration subsumption DAG、path view、neighborhood view、explanation extractor が定義されています。特に、AI tools と IDE には full graph ではなく bounded local subgraph を返す設計になっています。([GitHub][1])

これは非常に良いです。

AIに対しては、たとえば次のような説明が必要です。

```text
この expression は Ring としても AddStr としても MulStr としても見える。
今回の Product は AddStr 版と MulStr 版の両方が候補になった。
継承パスが二つあり、暗黙 upcast では決められない。
したがって `qua MulStr` を入れれば解決する。
```

現在仕様の overload resolution では、candidate set、viable candidate、best match、ambiguity が明確に定義され、複数経路では `qua` が必要という設計です。([GitHub][9])

これはAIにとって、非常に「修正しやすい」エラーです。

だから、Mizar evo のAI支援は最初から巨大な theorem proving を狙うよりも、まず次を高精度化すると良いです。

```text
overload ambiguity repair
qua insertion
attribute / cluster missing explanation
by clause completion
unused citation refinement
```

この4つは、AI支援の費用対効果が高いです。

---

## 14. MCP security と Mizar soundness を接続する

MCP の tool はモデルが外部操作を呼ぶための仕組みですが、公式仕様でも human-in-the-loop、入力検証、アクセス制御、rate limit、output sanitization、tool call timeout、audit log が重要とされています。([Model Context Protocol][4])

Mizar evo の場合、さらに形式数学特有の security policy が必要です。

たとえば MCP scope をこう分けます。

```text
mizar:read          source/artifact/theorem の読み取り
mizar:explain       explanation API の呼び出し
mizar:verify        verify.patch の実行
mizar:write         patch の適用
mizar:refine        citation refinement
mizar:publish       package publication
mizar:unsafe-edit   axiom/definition/theorem statement の変更
```

`mizar:unsafe-edit` は通常のAI agentには渡さない。
`mizar:publish` も人間承認必須にする。

MCP authorization は OAuth 2.1 を前提にした protected resource / scope の設計を持っています。([Model Context Protocol][10]) Mizar evo の MCP server でも、少なくとも remote server ではこの scope 設計に合わせるべきです。

ローカル開発用には stdio transport でもよいですが、その場合でも `--ai-safe-mode` のような明示的な制限が欲しいです。

```bash
mizar mcp-server --safe-mode \
  --allow read,explain,verify,refine \
  --deny unsafe-edit,publish
```

---

## 15. AI用 benchmark を仕様化すべきです

AI親和性を主張するには、評価指標が必要です。

Mizar evo には次の benchmark suite があると良いです。

### Proof repair benchmark

既存の通る証明から意図的に citation を抜く。

```text
original: thus thesis by A, B, C;
broken:   thus thesis by A;
expected: add B, C
```

評価：

```text
top-1 repair success rate
top-5 repair success rate
平均 verify.patch 回数
新規エラー発生率
修正後 used_axioms minimality
```

### Overload repair benchmark

意図的に `qua` を抜く。

```text
Product(R)
```

期待修正：

```text
Product(R qua MulStr)
```

評価：

```text
正しい view を選べた率
不要 qua 挿入率
説明の正確性
```

### Context budget benchmark

巨大な module に対して、必要最小 context だけで修正できるかを見る。

```text
8k tokens context
16k tokens context
32k tokens context
```

評価：

```text
context size
repair success
retrieved relevant theorem recall
irrelevant theorem contamination
```

この benchmark があると、AI親和性が「雰囲気」ではなく、継続的に改善可能な engineering target になります。

---

## 16. 追加仕様章としては “25. AI Agent Interface” が欲しい

前回も述べましたが、深掘り後の提案として、次の章を入れるのが最も筋が良いです。

```text
25. AI Agent Interface and Context Protocol

25.1 Design Principles
25.2 Trust Boundary and Safe Edit Classes
25.3 Context Budget Model
25.4 AI Resource URI Scheme
25.5 Theorem and Symbol Index Artifacts
25.6 Obligation Context Extraction
25.7 Patch Proposal Format
25.8 Patch Verification API
25.9 Proof Repair State Machine
25.10 MCP Binding
25.11 Authorization Scopes
25.12 Dataset Export Format
25.13 Benchmarks and Metrics
```

この章は、言語仕様というより **tooling contract** です。
Mizar evo のAI親和性を決定的に上げるのは、たぶん構文をさらに増やすことではなく、この tooling contract です。

---

## 17. 優先順位をつけるならこうです

私なら、実装・仕様化の優先順位は次にします。

### Phase 1：read-only AI context

```text
theorem-index.jsonl
symbol-index.jsonl
mizar context obligation
mizar search theorem
mizar explain failure --json
```

まずAIが安全に読む・探す・説明するだけにする。

### Phase 2：dry-run patch verification

```text
patch schema
mizar verify patch
replace_by_clause
insert_qua
insert_proof_hint
```

まだ source を直接変更しない。
patch を検証して、成功/失敗を返すだけ。

### Phase 3：safe patch application

```text
mizar apply patch
mizar refine
mizar minimize
AI-safe edit classes
```

Green edit のみ自動適用可能にする。

### Phase 4：MCP server

```text
mizar mcp-server
Resources
Tools
Prompts
authorization scopes
audit log
```

MCPに乗せる。
この段階で IDE / ChatGPT / Claude / Cursor / VS Code などが統一的に触れる。

### Phase 5：training data export

```text
goal -> used_axioms
failed_goal -> successful_patch
overload_error -> qua_fix
bulk_citation -> refined_citation
expression -> inferred_type
surface_symbol -> resolved_fqn
```

これで Mizar evo 専用の retrieval model / repair model / premise selector を育てられる。

---

## 18. 最終評価

深掘り後の評価はこうです。

**Mizar evo の現在仕様は、AI親和性の“素材”はかなり揃っています。**
特に `*.mizir.json`、`missing_facts.json`、`used_axioms`、stable diagnostic code、explanation API、LSP code action、cluster graph explanation は強いです。

ただし、まだ足りないのは **AI agent の操作面の正式仕様** です。

現状は、

```text
AIが読む artifact: かなりある
AIが理解する diagnostic: かなりある
AIが検索する index: まだ不足
AIが編集する patch protocol: まだ不足
AIが検証ループを回す API: まだ不足
AIが安全に操作する権限モデル: まだ不足
AI評価 benchmark: まだ不足
```

という状態です。

したがって、今後の方向性としては、

> 言語をAI向けに崩すのではなく、
> **Mizarらしい安定構文・FQN・kernel soundnessを維持したまま、
> AIが使える artifact / context / patch / verify protocol を厚くする**

のが最善だと思います。

この方向で進めると、Mizar evo は単なる「AIで補助できる形式言語」ではなく、**AI agent が安全に探索・修正・検証できる形式数学プラットフォーム**になります。

[1]: https://raw.githubusercontent.com/aabaa/mizar-evo/main/doc/spec/en/23.package_management_and_build_system.md "raw.githubusercontent.com"
[2]: https://raw.githubusercontent.com/aabaa/mizar-evo/main/doc/spec/en/24.documentation_generation.md "raw.githubusercontent.com"
[3]: https://raw.githubusercontent.com/aabaa/mizar-evo/main/doc/spec/en/01.introduction.md "raw.githubusercontent.com"
[4]: https://modelcontextprotocol.io/specification/draft/server/tools "Tools - Model Context Protocol"
[5]: https://modelcontextprotocol.io/specification/draft/server/resources "Resources - Model Context Protocol"
[6]: https://modelcontextprotocol.io/specification/draft/server/prompts "Prompts - Model Context Protocol"
[7]: https://raw.githubusercontent.com/aabaa/mizar-evo/main/doc/spec/en/22.error_handling_and_diagnostics.md "raw.githubusercontent.com"
[8]: https://raw.githubusercontent.com/aabaa/mizar-evo/main/doc/spec/en/12.modules_and_namespaces.md "raw.githubusercontent.com"
[9]: https://raw.githubusercontent.com/aabaa/mizar-evo/main/doc/spec/en/19.overload_resolution.md "raw.githubusercontent.com"
[10]: https://modelcontextprotocol.io/specification/draft/basic/authorization "Authorization - Model Context Protocol"
