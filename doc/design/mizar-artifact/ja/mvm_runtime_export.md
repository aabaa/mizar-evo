# MVM R3 — 実行 Artifact と検証付きローダー

> 英語正本: [mvm_runtime_export.md](../en/mvm_runtime_export.md)。
> 設計契約。
> 現段階は設計案であり、実行形式や実装の承認ではない。

## 責務と順序

[仕様 §20.9.1](../../../spec/ja/20.algorithm_and_verification.md)
に従い、公開可能な計算アルゴリズムを元の本体・型・契約・検証証拠・
推移的実行依存・VM 版と結び付ける。公開宣言やハッシュだけでは
実行可能と認定しない。

1. R2（Core / Checker / VC / Proof）：元の**定義・本体**と受理済み義務。
2. R3（Artifact）：個々の呼出しに依存せず実行内容を正準公開。
3. R3（Build / Driver）：manifest と依存 Artifact の検証付き読込。
4. R4（MVM）：別途型付けされた真正な呼出しと結合し、正常復帰時だけ E5 を生成.
5. R6–R7（VC / Proof）：計算の受理と証明への変換。
6. R7 以降（Artifact）：計算結果の来歴を公開。実行内容の初回公開を
   この結果に依存させず、循環を防ぐ。

公開アルゴリズムから内部の private 補助関数を使えるが、
他パッケージから private 名を呼び出せない。依存 Artifact が
欠損・不正・旧版でもソースコードへフォールバックしない。
非 computable な public アルゴリズムは契約で利用でき、契約だけを使う
importer は runtime Artifact を読まなくてよい。ただし computable な
public アルゴリズムの公開には検証済み runtime Artifact が必須であり、
欠落はエラーであって契約のみの公開モードではない（§20.9.1）。
読み込みでは本体・契約・証拠・VM・版・全依存関係・公開範囲を
検証する。失敗は未検証の値や証明を生成しない。

R6-C が消費する R3 export 内容として、private helper を含む閉包内の
各 callee の型ガード・`requires`・完全な `ensures`・値返却／non-ghost
Pick／集合対列ループの分類を登録する。VC58 の同一パッケージ Open
目標生成にも必要となる。下記 R3-N1 の owner 決定に従い exporter が適用対象の
閉包一意性を証明し、importer は証拠との厳密な結合を検証する。

## Freeze 準備資料

Q1/Q3 は Artifact 担当（2026-11-20 11:00 JST）、Q3 は Build／Proof の合意も必要。
Q2 は RESERVED／担当未割当。2026-11-22 06:43 JST の owner 決定は
**N1：exporter が証明、N2：論理 import の信頼モデルを拡張**、N3 は本資料で準備する。
G9 の機構検討許可（06:34 JST）は残る問いの決定権を委譲しない。
以下は推奨案であり、形式・schema・API・owner 回答を確定しない。

### R3-Q1 — 実行表現（未決）

[R1-Q3](../../mizar-vc/ja/mvm_readiness.md) から移管された問い。R3 実装 freeze と R4 形式選択をブロックする。

| 選択肢 | 影響 |
|---|---|
| 抽出用 `RuntimeIr` を共有 | 消去・追跡モデルを共用できるが、[仕様と構築は計画段階](../../mizar-doc/ja/todo.md)。[§20.10](../../../spec/ja/20.algorithm_and_verification.md) の抽出は terminating 限定、MVM は partial も扱う。profile・意味・版の共同管理が必要で、R4 が抽出側の開発に依存する。 |
| checked Core/CFG から別の版付き Artifact 実行形式を生成 | 抽出から独立し MVM の範囲を扱えるが、命令・値の意味、正準 encoding、消去対応、互換性検証が別途必要。抽出での将来の再利用は自動ではない。 |

**owner 検討用の推奨：** 最初は狭い MVM 専用形式とし、別のソース lowering を作らず checked Core/CFG の事実を使う。
対応する操作・値・呼出し・消去と VM／形式の互換性を一緒に freeze する。

### R3-Q3 — 対応証拠と loader（未決）

R3 export をブロックする。N2 は import 系譜の信頼根拠を固定するが、実行内容の対応証明方式は選ばない。

| 選択肢 | 影響 |
|---|---|
| producer が対応を証明し認証済み公開で運ぶ | Core／VC／Proof が定義受理と変換・消去の関係を確立し、Artifact が実行 byte 列・全依存と結合。reader の完全性検証と Proof の権限・policy 検証は別。opaque 本体の再証明は不要だが真正な producer が必要。 |
| loader が対応を独立に再構成・検査 | producer の変換主張への依存を減らせるが、検査可能な関係・witness または元の checked 表現が必要。reader の信頼範囲と版の結合が増える。依存ソースへの fallback は不可、N2 の認証も必要。 |

**owner 検討用の推奨：** producer 側で対応を確立し、reader が検証する。独立した破損対照も維持する。
将来の freeze では、受理済み定義・本体・CFG・契約の identity、消去と実行の関係、命令・値の順序意味、
private helper／imported-public 依存閉包、VM／形式版、現在の snapshot／lock／policy 結合を規定する。
検証と実行は同じ不変 byte 列を使う。hash や署名だけでは対応を証明できない。
Artifact は正準 transport／manifest 版、Proof は受理・認証、Build／Driver は検証済み handle の消費を担う。

### R3-N3 — computability producer（未決）

§20.9.1 の computable-public export 適格性をブロックする。

| 選択肢 | 影響 |
|---|---|
| Frontend／Checker のみ | 型・可視性・操作を持つが、受理済み本体と実行依存の全対応は持たない。構文・型 flag だけでは export 可能と認定できない。 |
| Core／VC が定義・閉包を評価し Proof が受理、Artifact が消費 | checked 本体・呼出し・分類を private helper／imported export まで利用。完全で版に結合した評価が必要。Checker／Resolver が事実を供給し Artifact が公開を制御する。 |
| Artifact が命令に変換可能かで推論 | 形式に近いが、backend の対応範囲と言語の computability を混同し意味解析を重複させる。直列化成功だけでは不十分。 |

**owner 検討用の推奨：** Core／VC の評価を使い、意味上の computability・検証受理・現在の backend 対応を分離する。
producer／API の厳密な担当は owner freeze に残す。[§20.9.3](../../../spec/ja/20.algorithm_and_verification.md) の入力依存 runtime failure は定義を無効にせず、適格性は全入力の成功や全域性を保証しない。
未対応・不完全なら実行 handle を与えず、computable-public を黙って契約のみへ分類し直さない。
宣言全体と入力依存の computability、到達不能操作、template 実装、Pick enumerator の扱いは owner の判断材料であり、新しい言語基準を定めない。

### R3-Q2 — 順序の事実のみ（RESERVED）

R3 export と R4 同一モジュール pilot をブロックする。同一ファイルの計算例（§20.9.2／§20.13.4）と file-final emit（§23.4）が制約。
完全検証・別途認証された staged runtime を公開 commit 前に読むか、公開済み検証済み依存を待つか、定義ファイル内 private entry を直接評価するかは owner に残す。
[論理 transport 設計](./manifest.md#producer-transport-design) は commit 済み公開を認証するため、それだけでは commit 前の runtime 権限を与えない。
必要な事実は R2 の完全受理が得られる最初の時点、依存／group 順序、staging の寿命・currentness・失敗 rollback、循環のない readiness 根拠。
未検証 token や自身の将来の実行に依存する証明は不可。

### R3-N1/N2 — 決定済み入力と残る証拠

- **N1：exporter が閉包一意性を証明。** opaque private helper も含む適用対象の値返却 callee について、型ガード・`requires`・完全な `ensures`・trigger・証明または仕様どおりの免除を identity に結合する。
  importer は結合・依存・policy を検証し private 本体の代替証明を生成しない。全 algorithm に一意性を要求するのではなく R6 の trigger 規則を維持し、最終受理では実際の評価閉包を検証する。
- **N2：論理 import の信頼モデルを拡張。** [Proof 規則](../../mizar-proof/en/policy.md#trust-transfer-rule) と [Artifact 境界](./manifest.md#imported-promotion-authority-boundary) の importer が認可した権限、真正な受理結果だけの commit 済み公開、object／group／依存／policy の厳密な結合を使う。
  runtime には実行内容・VM 意味・対応も結合する。論理のみの import は runtime 不要。既存 transport は設計であり、runtime validator や受理済み R3 fixture の実装ではない。

### 後続への影響と次の freeze 条件

| 利用側 | 提案および N1/N2 決定の影響 |
|---|---|
| [R4](../../mizar-vc/ja/mvm_executor.md) | Q1 の意味、Q3 の検証済み byte 列・閉包、N3 適格性と別途型付けした真正な呼出しが必要。Q2 は同一モジュール pilot を引き続きブロック。実際の正常復帰による E5 だけでは Proof 受理にならない。 |
| [R6](../../mizar-vc/ja/mvm_computation_admission.md) | local A/B は準備可能、C は R3 export が必要。N1 の証明生成は exporter、D は N2 の下で全評価閉包と受理済み証明／免除を検証。computability や R4 復帰だけで受理を迂回しない。 |
| [R7](../../mizar-proof/ja/mvm_computation_import.md) | 真正な E5・R6・変換入力に N2 系譜と Q1/Q3 の実行内容・依存・VM identity を保持。replay は別途方程式／Kernel 根拠が必要。後の計算記録で最初の R3 export を認可しない。 |

AST-ARTIFACT 配下の Artifact 実装 freeze には、owner の Q1/Q2/Q3/N3 回答、真正で完全な [R2 定義証拠](../../mizar-core/ja/mvm_correspondence.md)、N1 exporter 証拠、N2 runtime 拡張契約が必要。
互換形式・manifest・loader 検証と英語正本の将来の正負テスト行列を freeze する。R2 accepted-set 準備には Open／Deferred が残り、完全な export 適格性ではない。
既存の [`idalgo`](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz) は Open VC であり実行の正例ではない。本資料は実装・runtime export・計算受理・coverage・言語規則の変更を許可しない。
