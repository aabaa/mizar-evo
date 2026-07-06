# カーネル健全性論証と実装前監査

> 正文は英語版:
> [../en/soundness_argument.md](../en/soundness_argument.md)。

## 目的

本書は、信頼される `mizar-kernel` 受理境界(パイプライン第 14 相)の健全性
論証であり、証明書フォーマットと検査意味論に対する実装前監査の記録である。
以下を一箇所に集約する。

- 受理前にカーネルが検査する不変条件の完全な列挙;
- 「replay」の厳密な意味論: カーネルが何を再計算し、何を決して信頼しないか;
- 拒否カテゴリの分類と、各カテゴリが防ぐ攻撃;
- 代入・α変換・節整形式性の検査で見落とされうるエッジケースと、その現状の
  扱い;
- 重大度付きの監査所見(探索側の不健全性が受理側に漏れうる経路候補を含む)。

本書は、訂正後の formula/substitution evidence 経路(architecture 15
「Post-Closeout Correction」、タスク 23-29)を通常の受理契約として監査し、
レガシーの resolution-trace 証明書は移行/監査用の在庫としてのみ扱う。
[15.kernel_certificate_format.md](../../architecture/en/15.kernel_certificate_format.md)、
[16.substitution_and_binding.md](../../architecture/en/16.substitution_and_binding.md)、
[08.reasoning_boundary.md](../../architecture/en/08.reasoning_boundary.md)、
[19.failure_semantics.md](../../architecture/en/19.failure_semantics.md)、
[20.test_strategy.md](../../architecture/en/20.test_strategy.md) と本クレートの
モジュール仕様を精査・相互照合した監査成果物である。新たな受理挙動は導入
しない。ギャップを発見した箇所は、アーキテクチャ文書を修正した(下記に記録)
か、後続タスク向けの所見として登録した。

## 信頼モデル

証明受理のための trusted computing base は次のとおり。

- `mizar-kernel` のソース(パーサ、各チェッカー、SAT エンコーディング、
  `sat_checker` ラッパーを含む);
- 監査済みの依存 `batsat = "=0.6.0"`(`bit-vec 0.5.1` を含む)。
  [sat_dependency_audit.md](../en/sat_dependency_audit.md) に従い固定・
  ラップされる;
- カーネルが独立に再検査する binder 契約を持つ `mizar-core` のデータ形状
  (カーネルは形状を消費し、妥当性の主張は消費しない);
- `mizar-session` の基盤型。

それ以外はすべて受理に関して untrusted である: ATP バックエンドとそのログ、
`mizar-atp` の変換、`mizar-vc` の生成・discharge evidence、resolver/checker の
状態、キャッシュ、アーティファクト、そして `KernelEvidence` バイト列を組み
立てた evidence producer。呼び出し側が与えたフィールドは、それ単体では決して
受理材料にならない。(a) 再導出して比較する、(b) 呼び出し側の不変コンテキスト
と照合する、(c) 無視または拒否する、のいずれかである。

一つだけ、構成上信頼される入力がある。それは第 14 相オーケストレータが
生成しなければならない不変検査コンテキスト(`FormulaEvidenceContext`、期待
対象 VC フィンガープリント、ポリシー、リミット)である。本方式全体の健全性
は、オーケストレータがこのコンテキストを、evidence バイト列を運んだのと同じ
チャネルからではなく、カーネル受理済みの依存アーティファクトと正準 VC
同一性からのみ構成することに依存する。所見 F2 を参照。

## 検査される不変条件

カーネルは、以下の不変条件がすべて成立する場合にのみ evidence を受理する。
識別子は `tests/certificates/` 配下の reject-first コーパスと所見の節から
参照される。

### E. エンベロープ・構造不変条件(`formula_evidence`、タスク 25)

- E1. ドメインセパレータは `MIZAR_KERNEL_EVIDENCE\0`。schema/encoding
  version はサポート対象であること。未知の値は
  `unsupported_certificate_format` として拒否。
- E2. セクションは固定の v1 順(symbol manifest、variable manifest、formula
  evidence、substitutions、provenance、final goal)。各セクションは length-
  framed で、バイト範囲は正確に消費され、trailing bytes は拒否。
- E3. 各セクション内の id は一意かつソート済み。重複・未ソートは拒否。参照は
  同一 evidence 内で宣言された id(imported fact は呼び出し側コンテキストの
  エントリ)のみ解決される。
- E4. すべてのリスト数、バイト長、term サイズ、再帰深度、ノード数は割り当て
  前に決定的リミットで検査される(`resource_exhaustion`)。
- E5. canonical hash 入力は検証済みエンベロープバイトそのもの。producer 供給
  の trusted hash フィールドは存在しない。

### B. 対象・プロファイル・コンテキスト束縛不変条件

- B1. evidence の `target_vc` は呼び出し側の期待対象 VC フィンガープリントと
  一致すること(不一致は `context_mismatch`)。
- B2. evidence の `kernel_profile` はチェッカー構成のプロファイルにサポート
  され、かつ等しいこと(`unsupported_certificate_format`)。
- B3. final goal のフィンガープリントは goal formula tree から再計算されて
  一致すること。その provenance は対象 VC と goal フィンガープリントを束縛
  すること(`missing_provenance`)。
- B4. final goal の polarity は、呼び出し側の不変コンテキストがこの VC に
  要求する検査種別と一致すること。証明義務の証明受理には refutation
  polarity(goal を偽として主張)が必要であり、それ以外の polarity での受理
  は `context_mismatch`(architecture 15「Goal Polarity Is Bound By The
  Target Obligation」; 所見 F1)。`mizar-kernel` task 30 が
  `check_kernel_evidence` で実装済みである。Accepted consistency check は
  `ConsistencyCheck` として運ばれ、downstream proof policy では non-selectable
  diagnostic evidence である。
- B5. 呼び出し側コンテキストの欠如、または evidence が束縛を主張する
  コンテキスト provenance フィンガープリントの不一致は
  `missing_provenance`。

### F. 論理式不変条件

- F1. すべての論理式はサポートされる文法(正規化 `clause::Atom` 上の
  `Atom`/`Not`/`And`/`Or`)でパースされること。空の連言/選言、不正 term、
  未知シンボル、manifest 非互換変数は拒否。
- F2. Atom の同一性は正準かつ単射なバイトエンコーディングによる。表示名、
  ソースパス、割り当て順序は一切関与しない。
- F3. 各論理式の tree フィンガープリントはカーネルが再計算し、記録値と一致
  すること(安定同一性の束縛であり、受理主張ではない)。
- F4. symbol/variable manifest は構造検証のみを許可し、シンボル検索、
  オーバーロード解決、ソース読み込みを決して誘発しない。

### P. Provenance・ソース束縛不変条件

- P1. すべての formula entry は `source_class` に形状が一致するソース束縛を
  ちょうど一つ持ち、対象 VC と formula フィンガープリントを束縛する
  provenance entry をちょうど一つ参照すること。
- P2. imported axiom/theorem entry は完全な同一性 5 つ組(package id、module
  path、exported item id、statement fingerprint、required proof status)を
  持ち、呼び出し側コンテキストの `FormulaImportedFactEvidence` と厳密一致
  すること。欠如・同一性不一致・フィンガープリント不一致は
  `unresolved_symbol`。
- P3. 証明ステータス強度は `kernel_verified > discharged_builtin >
  externally_attested_policy_permitted` の順。要求より弱いステータスで受理
  された evidence は `unresolved_symbol`。externally attested な import は
  さらにプロファイルポリシーで制御され、結果に taint を付す。
- P4. local hypothesis、cited premise、generated VC fact の束縛は、単に
  well-shaped であるだけでなく、呼び出し側の不変コンテキスト同一性に対して
  検証可能でなければならない(architecture 15「Context Identity Covers
  Non-Imported Source Bindings」; 所見 F2)。訂正後 checker は SAT encoding 前に
  task-28 context-identity payload を要求し、すべての非 import formula entry を
  不変な source/id、formula-id、formula-fingerprint row と照合し、欠落・stale・
  ambiguous な identity を `missing_provenance` として拒否する。
- P5. `used_axioms` は、source class が accepted imported axiom/theorem で
  ある受理済み formula evidence のみから導出される。バックエンド報告の
  used-axiom リストは決して信頼されない。

### S. 代入・α変換・freshness 不変条件

- S1. 代入ペイロードは明示的なコンテキスト evidence である。参照された
  ペイロードの欠如は `missing_provenance` であり、カーネルは source/target
  term の差分からペイロードを推測しない。
- S2. binder context は各エントリ自身の `binder_context_encoding` から v1
  文法でデコードされる。未知 version/role、切り詰め、重複フレーム、非正準
  順序、フレーム/term 非互換は `invalid_substitution` として拒否。
- S3. replay は capture-avoiding である。挿入された actual term の自由変数が
  束縛されてはならない。正当化する freshness witness のない衝突は
  `invalid_substitution` であり、暗黙のリネームは決して行わない。
- S4. α同値は正規化 binder 構造と安定 id で決定される。リネームはスコープ毎
  に単射であること。自由変数が束縛されず、束縛変数がスコープ外へ逃げない
  こと。
- S5. freshness witness は完全に再計算される: avoided set は source binder
  body の自由変数(元の束縛変数を除く)+ 挿入 actual の自由変数から再構築
  され、witness の生成 id はその集合に含まれないこと、決定的カウンタは
  manifest 由来の候補ストリームにおけるその id の位置と一致すること。
  不一致は `invalid_substitution`。
- S6. 自由変数副条件は、正規化された target binder スタックから記録パスで
  再計算される。記録された capture set は自己証明的ではなく、再計算集合と
  厳密一致すること。
- S7. 同時写像意味論: 複数の replacement は formal variable id をキーとする
  一つの写像として適用され、replacement の actual term は同一ペイロード内の
  他エントリで再書き換えされない(逐次合成の曖昧さなし)。
- S8. formal variable は source formula に出現すること。未サポートの payload
  kind・role・非 root rewrite path は(仕様化まで)`invalid_substitution`
  として fail-closed に拒否。

### I. 具体化・SAT エンコーディング不変条件(`sat_encoding`、タスク 26)

- I1. 具体化された論理式は、検査済み source formula と検査済み代入から
  カーネルが導出する。呼び出し側供給の instantiated formula、SAT clause、
  resolution trace、backend proof method、ログは trusted payload として
  無視または拒否される。
- I2. SAT 変数割り当ては決定的: atom 変数はソート済み正準 atom バイト順、
  Tseitin 補助変数は決定的走査順。等価な呼び出し順は同一の正準 SAT バイトを
  生成する。
- I3. エンコードされた問題が主張するのは正確に: 全 premise formula、全導出
  instantiation、および B4 で検査された polarity での standalone goal。goal
  は premise として重ねて主張されず、`used_axioms` にも寄与しない。
- I4. 正準 SAT バイトは診断/検査トレース成果物であり trusted input ではない。
  エンコード済み問題のフィールドはモジュール外では読み取り専用。

### C. 信頼 SAT 検査不変条件(`sat_checker`、タスク 27)

- C1. 受理証拠はカーネル導出問題に対する `SatCheckResult::Unsat` のみ。
  `Sat` は非受理証拠であり、ソルバーエラー、未サポート節、リミット失敗は
  決定的に拒否される。
- C2. `batsat` のヒューリスティック・乱数オプションはすべて監査済みの決定的
  値に固定され、呼び出し側に公開されない。proof 生成、モデル列挙、DIMACS、
  コールバック、プロセス/ネットワーク面は到達不能。
- C3. サイズリミット(変数、節、リテラル、節幅、正準バイト)はソルバー構築
  前に強制される。正確な conflict/propagation 予算は未サポートで、要求は
  拒否される(所見 F3)。

### R. 結果・オーケストレーション不変条件(`checker`、タスク 28)

- R1. `accepted` は先行するすべての不変条件クラスの成立を要する。どの
  サブ検査の失敗も入力全体を拒否し、修復も代替パイプラインもない。
- R2. バッチ結果は対象 VC フィンガープリント順、同値なら呼び出し側入力順。
  ワーカー完了順は結果にも拒否順序にも影響しない。
- R3. externally attested import 由来のポリシー taint は結果に伝播し、無条件
  の `kernel_verified` に洗浄されることは決してない。
- R4. 決定的ステップ予算は replay を `timeout` として停止し、サイズ/メモリ
  予算は `resource_exhaustion`。いずれも非受理であり、義務は未検証のまま
  残る。

### L. レガシー経路不変条件

- L1. レガシー `Certificate` / `resolution_trace` バイトは v1 ドメイン
  セパレータを共有せず、タスク 25 パーサでは
  `unsupported_certificate_format` として拒否される。
- L2. `KernelCheckPolicy.allow_legacy_certificate_audit` の既定は `false`。
  監査モードは検査のための replay を行えるが、それでも
  `unsupported_certificate_format` の監査レコード付き `Rejected` を返し、
  trusted `final_goal`、`used_axioms`、witness、キャッシュ昇格、
  `kernel_verified` を決して生成しない。
- L3. 監査 replay 内でもレガシー不変条件は成立する: 正準節エンコーディング
  (ソート済み・重複なしリテラル)、両親で逆極性の pivot、resolvent の
  再計算と比較、親参照は imported clause・generated clause・厳密により早い
  step のみ(前方参照・自己参照は malformed)、final goal は検査済みの正準
  空節に解決されること。

### D. 決定性・資源不変条件

- D1. 同一の evidence バイト、コンテキスト、リミット、ポリシーは、
  プラットフォームとワーカー数を問わずバイト同一の結果と拒否順序を生成する。
- D2. いかなる受理/拒否経路でも、wall-clock、乱数状態、環境、ファイル
  システム、キャッシュ、アーティファクト、グローバル可変状態を読まない。
- D3. すべての予算は対応する割り当てや再帰の前に検査される。

## Replay の意味論

「replay」は純関数

```text
(evidence_bytes, immutable_context, limits, policy)
  -> KernelCheckResult (accepted | rejected + 安定拒否レコード)
```

である。カーネルが再計算する(したがってフィールドとして信頼しない)もの:

| 記録フィールド | カーネルの動作 |
|---|---|
| formula フィンガープリント、goal フィンガープリント | パース済みツリーから再計算し比較 |
| entry hash 入力、正準 evidence hash | 検証済みバイトのみから導出 |
| 代入の target term | capture-avoiding replay で再導出し構造比較 |
| freshness witness(avoided set、カウンタ) | 正規化 evidence から完全再計算 |
| 自由変数 capture set | 記録パスで再計算 |
| instantiated formula | 検査済み formula + 代入から導出 |
| SAT 問題(変数、節) | 決定的に導出。入力からは決して読まない |
| UNSAT 結果 | ラップされた in-process チェッカーで再計算 |
| レガシー resolvent、cluster/reduction コミットメント(監査モード) | 再計算し比較 |

カーネルが呼び出し側の不変コンテキスト(オーケストレータ入力としてのみ信頼
され、producer 入力としては決して信頼されない)と照合するもの: 対象 VC
フィンガープリント、要求検査種別 / goal polarity、imported fact の同一性と
証明ステータス、コンテキスト provenance フィンガープリント、ポリシーゲート、
リミット。

カーネルがいかなる経路でも行わないこと: 証明探索、premise の選択・最小化、
代入の発明、オーバーロード解決、cluster 探索、registration 活性化、暗黙の
coercion 挿入、fallback 推論、代替エンコーディング、ATP/SAT 子プロセス、
バックエンド報告の成功による受理、不正 evidence のヒューリスティック修復。

## 拒否分類と防がれる攻撃

| 安定 detail | カテゴリ | 防がれる攻撃・失敗 |
|---|---|---|
| `unsupported_certificate_format` | certificate | schema/profile ダウングレード攻撃; レガシー resolution-trace 経路、backend proof method、SMT オブジェクト、ログの通常受理への持ち込み |
| `malformed_certificate` / `malformed_witness_data` | certificate | パーサ混乱: 重複 id、未ソートリスト、trailing bytes、二つの読み手が evidence 内容について不一致になりうる非正準エンコーディング |
| `context_mismatch` | certificate | 妥当な証明書の別 VC への再生; goal polarity の混同(B4); profile/コンテキストの継ぎ接ぎ |
| `missing_provenance` | kernel | premise 注入: 不変コンテキストに対して由来を検証できない formula や代入; 欠落ペイロードの推論による「修復」 |
| `unresolved_symbol` | kernel | 依存スライスの非同期化: statement fingerprint・同一性・受理証明ステータスがカーネル受理済みアーティファクトと一致しない imported theorem の引用; 証明ステータスの洗浄(externally attested の kernel-verified 偽装) |
| `invalid_substitution` | kernel | 変数捕獲、binder 衝突、α リネーム偽造、stale/偽造 freshness witness、偽造 capture set、未サポートペイロード形状の黙認 |
| `invalid_sat_refutation` | kernel | 導出問題が充足可能なのに反駁を主張; カーネル導出 SAT 素材の破壊; UNSAT ラッパー結果なしの受理 |
| `invalid_sat_proof`(レガシー) | kernel | 偽造 resolution step、誤 pivot、前方/循環導出、監査 replay での非空 final clause |
| `invalid_cluster_trace` | kernel | 隠れた推移的 cluster 展開、偽造 reduction コミットメント、guard evidence 不一致 |
| `timeout` / `resource_exhaustion` | いずれか | 非停止やメモリ爆発の暗黙受理化; いずれも義務を未検証のまま残す |

## 精査したエッジケース

各ケースはモジュール仕様と照合済み。処置は `covered`(不変条件が明示的に
扱う)、`fail-closed`(より豊かな仕様化まで保守的規則で拒否)、`finding`
(次節参照)のいずれか。

1. **攻撃者が形作った variable manifest 経由の fresh id 選択。** freshness の
   候補ストリームは producer 制御の manifest 由来だが、S5 が avoided set を
   binder body と挿入 actual から再計算するため、衝突する id は fresh と
   認定されえない。binder スコープ外の変数との衝突は無害(スコープ内の出現
   は定義上 body の自由変数であり avoided に入る)。`covered`。
2. **シャドーイング混同。** シャドーされた binder は別個の安定 id を使う
   (architecture 16)。一つの `binder_id` を二つの binder ノードで再利用する
   term は拒否(S2)。捕獲判定は表示名を決して参照しない。`covered`。
3. **逐次 vs 同時代入。** 別の replacement の actual term 内に現れる formal
   variable は再書き換えされない(S7)ため、順序依存の結果は認定されえない。
   `covered`。
4. **代入の連鎖 / 循環参照する導出。** 代入レコードは formula evidence entry
   のみを source にでき、導出式への参照は存在しないため、訂正後経路に連鎖も
   循環もない。レガシー監査 replay は前方/自己親参照を拒否(L3)。`covered`。
5. **premise としての goal の持ち込み。** final goal は standalone であり
   premise として主張されない(I3)。しかし producer は goal の式を local
   hypothesis や VC fact とラベルした premise entry に複製できる。task-31
   context identity verification は、その row が呼び出し側の不変コンテキストに
   存在しない限り、非 import 束縛(P4)として拒否する。`covered`。
6. **goal polarity の混同。** goal を真として主張し UNSAT を得ることは、
   premise が goal を反駁することの証明であり、それを goal の証明として
   受理すれば不健全。B4 が polarity を義務の検査種別に束縛する。`finding`
   F1(architecture 15 を修正済み)。
7. **矛盾した premise 集合。** 真正でカーネル検査可能な premise が矛盾して
   いれば任意の goal が導出できる。これは import された library に相対的に
   論理的に健全であり、局所検出は不能。緩和は provenance(P2)と imported
   fact 自体の上流受理。`covered`(信頼モデルによる。大域的整合性の残余
   仮定あり)。
8. **α変種 atom の別 SAT 変数化。** term 内の binder 名だけが異なる atom が
   異なるバイトにエンコードされるのは上流の正規化が失敗した場合のみ。
   manifest 検証済みの正規化エンコーディングにより α 同値 atom はバイト
   同一。正規化に失敗した producer は完全性を失うだけで健全性は失わない。
   `covered`。
9. **Tseitin 極性の誤り。** エンコーディングは完全に仕様化(演算子毎の
   ゲート節、決定的走査)。これはカーネルコードであり、evidence の信頼では
   なく決定性・ミューテーションテストで担保。`covered`。
10. **重複/未ソート節リテラル、恒真節(レガシー)。** 正準節エンコーディング
    は重複を拒否。恒真の扱いはプロファイル明示(marker か拒否)。節への
    `tautology` 誤ラベルは premise を弱めるだけ(不完全化であり不健全化では
    ない)。`covered`(レガシーのみ)。
11. **フィンガープリント衝突。** 現行アルゴリズムは正準バイトそのもの
    (恒等)で衝突なし。将来の digest アルゴリズムは衝突耐性が必須で、
    さもなくば imported fact 同一性が偽装可能。制約を architecture 15 に
    記録済み。`finding` F5(文書修正で解決)。
12. **imported statement fingerprint と formula-tree fingerprint。** v1 規則
    は等値を要求するため、(arch-18 の豊かな statement fingerprint を持つ)
    現実の imported statement は projection の仕様化まで引用不能。健全
    (fail-closed)だが完全性のブロッカー。`finding` F6。
13. **ゼロフレーム・未使用 binder フレーム。** 明示的 v1 エンコーディング
    経由でのみ受理。空バイト、フレーム欠落、未使用フレームは拒否
    (`invalid_substitution`)。`fail-closed`。
14. **捕獲なしの binder 下置換。** タスク 11 では拒否、タスク 12 の規則と
    witness による場合のみ意味論的に受理。`fail-closed`。
15. **局所略記クロージャ(`captured_free_variable`)。** payload kind と
    role は予約され、定義サイトクロージャと型ガード evidence の仕様化まで
    拒否。`fail-closed`。
16. **サイズリミット内でのソルバー非停止。** `batsat` は正確なステップ予算を
    公開せず、小さいが困難な導出問題が信頼ラッパー内で無制限の時間を消費
    しうる。不健全にはならない(UNSAT なしの受理はない)が、信頼基盤内の
    可用性ギャップ。`finding` F3。
17. **evidence とコンテキストの同一チャネル構成。** 第 14 相の呼び出し側が
    evidence producer に `FormulaEvidenceContext` も供給させると P2/P3 は
    崩壊する。コンテキストはオーケストレータのカーネル受理済み
    アーティファクトから来なければならない。`finding` F2(信頼モデルに
    文書化。強制は統合作業)。
18. **バッチのタイブレーク。** 同一対象フィンガープリントは呼び出し側入力順
    を保持。シャッフル構成は決定性テストでカバー。`covered`。

## 監査所見

重大度: **High** = もっともらしい不健全受理経路または信頼境界の穴;
**Medium** = 健全性隣接の曖昧さ・ドリフト・信頼基盤内の可用性ギャップ;
**Low** = 文書・整合性の負債。

- **F1(High、修正済み)。goal polarity の受理意味論が未規定だった。**
  `sat_encoding.md` は `AssertFalseForRefutation` と
  `AssertTrueForConsistency` の両方を定義し、architecture 15 は final_goal
  が「反証または証明すべき対象式」を記録するとだけ述べていた。証明義務の
  *証明*受理に refutation polarity が必要だとはどこにも書かれていなかった。
  producer は premise が `¬goal` を含意する VC に consistency polarity を
  選んで UNSAT を得られる — つまり反駁された goal を証明済みとして認定
  できる。本変更で修正: architecture 15(en/ja)は goal polarity を呼び出し
  側不変コンテキストの検査種別に束縛し、不一致を `context_mismatch` と
  した。コーパス: `fail_certificate_sat_goal_polarity_mismatch_001`。
  Checker-side B4 acceptance binding は `mizar-kernel` task 30 が実装し、
  `final_goal.polarity` の fail-fast rejection と、accepted consistency check を
  proof obligation として trust しない proof-policy guard を含む。
  producer-side `mizar-vc` handoff の declaration/rejection gap は `mizar-vc`
  task 27 で閉じた。
- **F2(High、修正済み)。非 import ソース束縛は仕様上のコンテキストから
  検証不能だった。** `FormulaEvidenceContext` は imported axiom/theorem のみを
  運ぶ。local hypothesis・cited premise・generated VC fact のエントリは
  非ゼロ id と*不透明な producer 所有* provenance ペイロードを束縛する —
  仕様上のカーネルは形状と対象束縛は検査できるが、実際の VC の local
  context や generated fact 集合への所属は検査できない。したがって
  (`mizar-vc` とは別の untrusted チャネルである)ATP 側 producer は任意の
  式 — goal 自身を含む — を local hypothesis とラベルできる(エッジ
  ケース 5)。Step 1 の producer/consumer 対で修正済み: architecture 15(en/ja)は、非 import
  ソース束縛の受理前にコンテキスト同一性がそれらを覆うことを要求し、検証
  データが存在するまで fail-closed とした。producer-side schema は現在、
  canonical formula-envelope hash と task-28 `context_identity_hash()` を分離する。
  context payload は各 local/VC-fact row を target VC と不透明な `mizar-vc` canonical
  formula-envelope handoff hash に束縛する。Kernel はその canonical handoff hash を
  `ParsedKernelEvidence::canonical_hash_input()` から再計算してはならない。parser hash
  input は binary evidence envelope であり、`mizar-vc` handoff renderer ではないためである。
  task 31 は受理前に target/row membership を検証し、documented line grammar から task-28
  context-identity hash を再計算する。
  回帰 coverage は、有効な local/cited/generated rows、identity 欠落、stale な
  target/hash/row payload、duplicate rows、matching immutable row を持たない
  local hypothesis としてラベルされた goal、task-28 golden context-identity line
  grammar を含む。
  コーパス:
  `fail_certificate_symbols_unverifiable_local_hypothesis_001`。
- **F3(Medium、設計上の deferral)。信頼 SAT ラッパーに正確なソルバー
  ステップ予算がない。** `sat_checker.md` は `batsat` 0.6.0 が安定した
  conflict/propagation 予算を公開しないことを記録しており、solve 時間を
  守るのはサイズリミットのみ。不健全受理は生じえないが、信頼基盤内の
  可用性ギャップ(solve 中に checker `timeout` が発火できない)。deferral を
  明示的に維持し、決定的予算 API を公開する依存が現れたら再訪する。
  コーパスは強制可能な側をカバー: `fail_certificate_resources_*`。
- **F4(Medium、修正済み)。architecture 15 の evidence フィールド一覧が
  実装済み v1 エンベロープからドリフト。** `KernelEvidence` のスケッチは
  `imported_axioms` / `imported_theorems` をトップレベルセクションとして
  列挙し、`symbol_manifest`、`variable_manifest`、`provenance` を欠いて
  いた。一方タスク 25 エンベロープに imported fact セクションはない
  (imported fact はソース束縛+呼び出し側コンテキスト)。本変更で整合
  (en/ja)。
- **F5(Medium、修正済み)。フィンガープリントの衝突耐性要件が未記載
  だった。** 現行のフィンガープリントアルゴリズムは正準バイトそのものだが、
  後から弱い digest が登録されることを防ぐ記述がなく、その場合 imported
  fact 同一性が偽装可能になる。制約を architecture 15 に追加(en/ja)。
- **F6(Medium、報告)。fingerprint 等値規則が imported fact の実用性を
  ブロック。** imported statement fingerprint に命題論理 formula-tree
  fingerprint との等値を要求するため、現実的な imported statement
  (arch-18 の豊かな式に対する statement fingerprint)は source-formula
  projection の仕様化まで引用できない。fail-closed で健全だが、import を
  引用する ATP-bound VC が受理されるには kernel/`mizar-vc` 対のスキーマ
  タスクが必要。
- **F7(Medium、`mizar-test` task 21 で解決済み)。訂正後経路の拒否語彙。**
  required-soundness-case レジストリは audit-mode resolution replay 向けの
  legacy `soundness.certificate.invalid_sat_proof` を保持し、訂正後経路向けに
  `soundness.certificate.invalid_sat_refutation`、
  `soundness.certificate.context_mismatch`、
  `soundness.certificate.missing_provenance`、
  `soundness.certificate.unsupported_legacy_certificate` を追加した。既存
  corrected-path certificate sidecar は payload や rejection behavior を変えず、
  `domain = "certificate"` と新 stable key を使う。これにより architecture 20 の
  invalid SAT refutation と normal policy 下の unsupported legacy-certificate
  coverage は required-case registry で固定された。
- **F8(Low、`mizar-test` task 22 で解決済み)。ディレクトリ命名の
  ドリフト。** architecture 20 は現在、certificate と kernel-evidence corpus
  の正準 root として `tests/certificates/` を列挙し、実装済み
  `mizar-test` レイアウトと本コーパスに一致している。退役済みの監査 draft 名
  `tests/kernel_evidence/` は、必要な場合の歴史的文脈に限って残す。
- **F9(Low、報告)。レガシー恒真 marker の意味論がプロファイル依存かつ
  希薄。** 誤ラベルは premise を弱めるだけで健全性の穴ではないが、タスク 29
  後続でレガシー経路とともに確定または退役させるべき。

## クレート TODO への影響(指摘のみ; 改訂は後続タスク)

- `doc/design/mizar-kernel/en/todo.md`: 候補新タスク — (a) 訂正後検査
  サービスへの B4 goal-polarity 束縛は task 30 で実装済み; (b) 非 import ソース束縛の
  コンテキスト同一性検証(F2)は task 31 で実装済みであり、不透明な `mizar-vc`
  canonical formula-envelope handoff hash と task-28 `context_identity_hash()` を
  運ぶ immutable context payload を使う; (c) ソルバーステップ予算
  deferral の再訪(F3); (d) fingerprint 等値規則を解除する imported
  statement projection の仕様化(F6、`mizar-vc` と対)。
- `doc/design/mizar-vc/en/todo.md`: producer-side goal-polarity declaration
  と consistency-polarity rejection は task 27 で解決済み。local/VC-fact 検証に
  カーネルが必要とする producer-side context-identity payload 生成は task 28 で
  解決済み。
- `doc/design/mizar-test/en/`: required soundness-case レジストリと
  layout/expectation 文書は訂正後経路 reason を含むようになった(F7)。
  コーパスルート命名 drift(F8)は task 22 で解決済み。

## 制約と前提

- 本書は architecture 15 post-closeout correction 時点の監査ベースラインを
  記録する。後続のスキーマタスクは、検査意味論を変更する同一変更内で不変
  条件の列挙を更新しなければならない。
- `tests/certificates/` 配下の reject-first コーパスは sidecar の notes から
  本書の不変条件 id を参照する。不変条件の改名は同一変更でそれらの notes の
  更新を要する。
- 本書のいかなる記述もモジュール仕様の禁止事項を弱めない。本書とモジュール
  仕様が食い違う場合、より厳しい記述が優先し、食い違いは修正すべき文書
  バグである。
