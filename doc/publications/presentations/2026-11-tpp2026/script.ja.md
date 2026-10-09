# TPP 2026 発表原稿（日本語）

> **Status:** working script, aligned with `slides.md` frame numbers  
> **用途:** スライド（英語版 `tpp2026.pdf`、日本語版 `tpp2026_ja.pdf`）を日本語で話すための原稿。フレーム番号は両版で共通。内容の正典は `draft.ja.md`。  
> **記法:** 各フレームの冒頭に目安時刻。文末の〔事実〕〔仮説〕〔構想〕は主張レベル。無印は事実（既存システム・公開ベンチマーク・Mizar Evo 仕様・main branch の実装状況）。  
> **分量:** 30分の配分案。具体例の説明速度をリハーサルで確認し、`[deep dive]` は時間が押したら飛ばす。

---

## Part 0. Introduction（0–2分）

### 0.1 Title（0:00）

Mizar は、一階述語論理と集合論の上に、人間が読める数学の言葉で証明を書き、機械で検査する処理系です。ライブラリ MML は50年の蓄積があります。
Mizar Evolution、略して Mizar Evo は、その言語と開発基盤を設計し直すプロジェクトです。
現行 Mizar の課題を整理し、新仕様でどのように対応するかを説明します。まず課題と設計指針を1枚で対応させ、§1–§6 でそれぞれの具体例を見ます。
仕様と実装、今後の計画を区別してお話しします。

### 0.2 How To Read The Claims [deep dive]（0:45）

無印は事実、「research hypothesis」は Mizar Evo が検証しようとしている主張、「future direction」は日程も設計も確定していない目標です。
コード例には Białystok の資料と同じく、MML の正確な引用・仕様例・スケッチのラベルが付いています。
仕様と実装は別物です。終盤に一枚、その境界を示します。

### 0.3 Current Mizar: Six Challenges And Design Principles（1:00）

現行 Mizar の課題と新仕様の設計指針を、同じ行に並べました。この6行が、本編の §1 から §6 に対応します。
§1 は基盤です。論理と MML の継承と処理系の刷新を両立し、一階論理、集合論、小さな kernel を保ちます。
§2 は記述です。暗黙の型や演算選択を追いやすくし、数学的な抽象化を継承しながら選択を明示します。
§3 は汎用化です。定義、定理、scheme の共通化を template で統一します。§4 は計算で、algorithm の契約、不変条件、停止性を検査します。
§5 は依存管理、配布、差分検証、IDE といった開発基盤です。§6 は検査と自動化です。ATP の活用の流れを示し、evidence をインスタンス化して SAT で検査する仕組みを説明します。
論理基盤と読みやすい言語は継承すべき強みです。実装済みの範囲は、末尾の状況説明で区別します。〔課題と仕様〕

## Part 1. Logical Foundation（2–4分）

### 1.1 Keep The Foundation, Rebuild The Tools（2:00）

数学の層は保存する。その下と周りの基盤は作り直す。これが Mizar Evo の方針です。
証明の基盤として、一階論理と Tarski–Grothendieck 集合論を保ちます。soft type、mode、attribute、registration、structure、宣言的証明も保ちます。
証明探索と検査を分離し、受理を小規模な kernel に集約します。論理の継承と、小さな信頼基盤を両立する設計です。検査の具体的な仕組みは §6 で説明します。
数学的な記述は §2、汎用化は §3、計算は §4、開発基盤は §5 で扱います。〔仕様〕

### 1.2 Current Mizar: Functions In Set Theory（3:00）

今度は Mizar の基礎、集合論側です。MML からの正確な引用を二つ見せます。
一つ目、FUNCT_1。関数適用 `f.x` は、「x が dom f に入っていれば [x, it] が f に属する」という、定義された集合論的関係です。
二つ目、FUNCT_2。`Function of X,Y` は、部分関数の上に `quasi_total` という属性を付けた soft type です。
ここでは関数は普通の一階の対象、つまり対の集合です。適用は論理に組み込まれているのではなく、定義されています。証明の基盤を高階にする必要はどこにもありません。
代償もあります。素朴に集合論を書けば、定義域・グラフ・関数性・所属関係が全部表に出てきます。〔事実〕

## Part 2. Readable Mathematics（4–10分）

### 2.1 Preserve And Extend Mathematical Writing（4:00）

Mizar の50年の答えは、この配管を言語の中に隠すことでした。
著者が実際に書くのは、`let f be Function of X, Y;`、`let x be Element of X;`、そして `f.x` です。同じ集合論的な関数ですが、対や定義域は書きません。
soft type、mode、attribute、registration、scheme、宣言的証明。これらは単なる糖衣構文ではありません。一階の集合論を、人間が数学として読み書きできる層へ持ち上げるための言語設計です。
Mizar はこの方向に50年歩いてきました。Mizar Evo はこの道を歩き続けます。〔事実と解釈〕
新仕様では、この記述を継承・拡張し、型、登録、オーバーロードの暗黙の選択を明示・追跡可能にする方針です。〔仕様〕

### 2.2 Registrations: Give Automatic Chains Names（5:00）

registration は、型の事実を自動的に伝播させる仕組みです。empty な集合は finite、finite なら countable という規則を連鎖的に使えます。
新仕様では登録項目にラベルを必須にします。EmptyImpliesFinite が働いて finite を得て、FiniteImpliesCountable が働いて countable を得た、という経路を記録します。
自動適用は保ちます。毎回 by で全部の規則を指定する必要はありません。どの規則を使い、どの事実が不足したかを名前で追えることが、トレーサビリティの改善です。〔仕様〕

### 2.3 Structures: Stored Fields And Derived Properties（6:30）

構造の宣言では、field と property を分けます。carrier と add は格納するデータで、構成子の引数になります。zero は標準的な値なので property として宣言します。
宣言だけでは値は決まりません。別の実装で、means なら存在・一意性を証明し、equals なら値を表す項を与えます。数学のデータと、そこから決まる値を区別する設計です。〔仕様〕

### 2.3a Inheritance Can Be Declared Later（7:30）

構造の宣言後に inherit を書けます。型の定義と関係の実装を分ける点で、Rust の trait 実装に近い構成です。
AddLoopStr を LoopStr として使うとき、親の binop を子の add、親の property unit を子の property zero に対応付けます。役割をリネームして同じ理論を使えます。
親ごとに一つの inherit 宣言です。型が同じなら証明は不要で、型を狭める場合は coherence を証明します。〔仕様〕

### 2.3b Checked Diamonds And Reused Group Theorems（8:45）

AddLoopStr から Magma には、LoopStr を経由する経路と AddMagma を経由する経路があります。メンバーの元の宣言と経路を追い、共有を検査します。共有する部分と異なるビューを明確にします。
Group で「単位元を右から掛けても x」と証明しておけば、環の加法ビューでその定理を使えます。R qua AddLoopStr を選ぶと、binop と unit は add と zero に対応し、「x に零を足しても x」になります。
選択した環の加法ビューが Group の前提を満たす必要があります。乗法側は一般には monoid で、同じ Group の定理をそのまま使えるとはしません。〔仕様に基づくスケッチ〕

## Part 3. Generic Mathematics（10–14分）

### 3.1 Current MML: Result Types For The Sum Of Functions（10:00）

functor の例として、関数の和を見ます。意味は、各点で値を足すことです。
現行 MML の VALUED_1 では、点ごとの加法を定義した上で、実数、複素数などの結果型に応じた再定義と登録を追加しています。〔事実〕
冒頭の2行は別々の登録ブロックの抜粋です。f1、f2 が複素数値なら和も複素数値、実数値なら和も実数値だと、型の事実をそれぞれ与えます。

### 3.1a Template: One Body And Concrete Result Types（11:00）

新仕様では、加法構造 T と添字集合 I をパラメータにし、T.carrier の値を持つ関数の和を一つの本体で定義します。
表の実数の例では f+g、複素数の例では u+v と書いています。同じ本体から、結果型はそれぞれ Function of I,R.carrier、Function of I,C.carrier に具体化します。各構造の加法が閉じていることと、必要な登録が前提です。存在・一意性の証明は省略しています。〔仕様に基づくスケッチ〕
synonym で Add の別名を中置の + にします。ここでは AddMagma への継承経路が一意で、関数の宣言型から型引数 T と添字集合 I が一意に決まるため、呼び出しで引数を省略する例にしています。
明示形と推論の条件は脚注にあります。値域集合だけで演算を選ぶわけではなく、継承経路が曖昧な場合は qua のビューを明示します。〔仕様〕
現行の定義は異なる定義域の共通部分も扱います。本例は共通の非空定義域に限定して、型の共通化を説明しています。

### 3.2 Schemes Become Ordinary Templates [deep dive]（13:00）

帰納法を例にすると、`let P be pred(Nat);` を持つ定理 `NatInduction[P]` になります。述語パラメータは、代入する述語ごとに一階の定理の族を生成します。
適用は `defpred` の後に `by NatInduction[P], Base, Step` と明示します。関数子パラメータは schema レベルの記号であって集合ではないので、論理は一階のままです。
`PermProduct[T]` や `qua` によるビュー選択は Białystok の資料と Backup 4 にあります。〔仕様〕

## Part 4. Verified Computation（14–20分）

### 4.1 Computation: Verified Algorithms（14:00）

algorithm の検査は Hoare 論理によります。requires が成り立つ入力で手続きを実行したとき、戻れば ensures が成り立つ、という契約です。停止性は次の terminating で扱います。
代入、if/else、while、for、return といった擬似コードに近い制御構造なので、アルゴリズムを読みやすく記述できます。
契約やループ不変条件から一階の証明義務を生成します。アルゴリズム自体の検証と、検証済み手続きによる証明支援を同じ枠組みに結び付けます。〔仕様〕

### 4.2 Algorithms: Contracts, Proofs, Computation（15:00）

仕様の例、ユークリッドの互除法です。requires、ensures、ループの invariant と decreasing があり、Gcd は数学側の関数子なので循環はありません。
こうした注釈は Dafny の invariant と decreases に類似しています。Evo のキーワードは decreasing です。類似例の出典を脚注とノートに置いています。
冒頭の EuclidGcdDef は定義ラベルで、euclid_gcd は関数として呼ぶ名前です。
terminating は、requires を満たすすべての入力で停止するという主張です。書くだけで信頼されるのではなく、不変条件と停止性の測度から生成する証明義務を検査します。
検証後は数学的な functor に昇格し、具体入力での計算だけでなく、論理式や証明の中でも使えます。〔仕様〕

### 4.2a Termination And Functor Promotion（17:00）

既定は部分正当性で、戻れば契約を満たすという意味です。terminating を検証すれば、requires の下で全域性が得られ、functor として使えます。
検証済みで定義的な断片に入る algorithm には定義方程式も与えます。一方、可変状態とループを持つ互除法では、本体を定義として展開せず、全域性と契約の公理を使います。
例の proof fragment は、正の自然数 a,b について euclid_gcd(a,b)=Gcd(a,b) を契約から示しています。by の後は EuclidGcdDef という定義ラベルで、検証済みの昇格公理を引用します。by computation は別に仕様化されていますが、MVM 実行とコード抽出は今後の課題です。〔仕様／実装状況〕

### 4.3 Euclid: What Must Be Proved?（18:00）

何を証明するのかを、1回の反復で見ます。更新前は x、y、y は正。r を x mod y と置き、更新後は x が旧 y、y が r になります。
最初は x=a、y=b なので、事前条件から不変条件が成立します。反復中は Gcd(x,y)=Gcd(y,r) という補題を使って、元の入力との GCD の一致を保ちます。旧 y は正、r は自然数なので、型と非負性も保たれます。
停止性には 0<=r<y を使います。新しい y は旧 y より小さい自然数です。終了時は y=0 なので、不変条件と Gcd(x,0)=x から、返す x が入力の GCD だと分かります。
こうした補題をライブラリから用い、各義務を定理と同じ枠組みで検査します。SAT 単独が剰余や GCD を理解するわけではありません。最後の y:=r を y:=x に変えれば、測度は減りません。これは仕様に基づく検査の説明例で、実行結果ではありません。〔仕様に基づく説明〕

## Part 5. Development Infrastructure（20–23分）

### 5.1 Environment: What Must An Author Import?（20:00）

現行 Mizar の環境部を見ます。ALGSTR_0 を短縮したものです。vocabularies は記号、notations は記法、constructors は構成子、registrations は型推論などで自動適用する事実、theorems は引用する定理を取り込みます。
特に notations と definitions は article を並べる順序にも意味があり、取り込む対象だけでなく、その順序も著者が管理します。
同じ STRUCT_0 が複数の一覧に出ます。新しい概念を使いたい著者は、それがどの article にあり、どの役割を取り込む必要があるかを判断します。記号が読めても、必要な記法や登録が揃ったとは限りません。〔現行環境の説明〕

### 5.1a Imports And Dependency Tracking（21:00）

新仕様では冒頭の import でモジュールの公開項目を取り込みます。例の function と algebra.structure.sorted は、仕様の例に基づくモジュールです。元の完全修飾名と、登録の解決トレースで、どの項目が実際に使われたか追跡する方針です。
依存指紋で検証済みの成果を再利用し、package と lock file でバージョンを固定します。IDE からは解決した名前やトレースを確認する設計です。
旧環境部の各一覧を一対一で置換する翻訳例ではありません。数学的な記述を継承しながら、依存と暗黙の処理を説明できる開発基盤へ進む、という設計です。〔仕様〕

### 5.2 The Whole Picture（22:00）

全体像を一枚で。
上に豊かな数学。structure、template、algorithm、module。下に小さな一階の基盤。周りに現代的な基盤、ATP 層、kernel、成果物、バージョン管理されたライブラリ、IDE、AI、出版。
数学の層は保存し、核は小さく保ち、基盤は作り直す。この話の内容はすべて、この三つの層のどれかです。〔仕様〕

## Part 6. Checking And Automation（23–28分）

### 6.1 Automation: Separate Search From Checking（23:00）

Sledgehammer と似た流れを、ソースで見ます。Isabelle では Q a を示したいところで sledgehammer を呼び、たとえば metis allPQ pa という内部で検証する証明を提案してもらいます。この例は仕組みを示すスケッチです。
Mizar Evo では、すべての x について P(x) なら Q(x) という前提 AllPQ と P(a) を引用して、thus Q(a) by AllPQ, Pa と書きます。
共通点は、ゴールと前提から外部で探索し、最後は内側の信頼できる仕組みで受理することです。prover の成功報告だけでは証明を受理しません。
Isabelle の Metis は内部で再証明します。Mizar Evo は元の論理式と置換を受け取り、instance 生成と SAT 検査を行います。Mizar の前提は引用前提と局所仮定で、Sledgehammer の theory context からの前提選択とは範囲が異なります。〔アーキテクチャ仕様に基づく説明〕

### 6.2 Using ATPs: Search, Check, And Reuse（24:00）

ATP をどこで活用するか、流れ図で示します。
ゴールと利用できる前提を ATP に渡して探索し、得られた候補 evidence を kernel で検査します。受理した成果をライブラリに保存し、再利用します。
未解決の場合は、補題や方針を修正して再探索します。LLM はこの提案や修復を支援できます。この図は設計意図であり、反復の費用対効果を示した実験結果ではありません。
現在の ATP 入力は引用した前提と局所仮定です。受理の根拠である evidence の検査を、次のページで具体的に見ます。〔仕様／設計意図〕

### 6.3 Resolution Tree And Extracted Substitutions（24:45）

前提 F1 は P なら Q、F2 は Q なら R、H は P(a)、ゴールは R(a) です。ゴールの否定を加えて反駁します。
木の最初は Q(x) と Q(y) を単一化し、x:=y として、not P(y) または R(y) を得ます。次に P(y) と P(a) を単一化し、y:=a として R(a) を得ます。not R(a) と合わせると空節です。
途中の x:=y をそのまま保存するのではなく、枝に沿って y:=a と合成します。元の F1 に x:=a、F2 に y:=a を回収し、出所と対象に結び付けます。
これは外部ログを使う非信頼の候補生成案です。現行の独立した instance finder も、同じ形の evidence を生成できます。ログの各推論を kernel に再生させるわけではありません。〔設計スケッチ〕

### 6.3a Saved Evidence To Concrete SAT Clauses（26:15）

上から、保存する evidence、kernel の前処理、SAT solver に渡す節列です。元の式と合成した置換を出所・対象・束縛文脈と照合し、capture avoidance を検査します。その後、二つの含意を a でインスタンス化し、P(a) とゴールの否定を加えます。
P(a)、Q(a)、R(a) を命題変数1、2、3にします。二つの含意を表す補助変数4、5を導入し、これらが真という単位節を加えます。こうした Tseitin 符号化は現行エンコーダと同じ構成です。
下の DIMACS では5変数・10節をすべて示しています。負の整数は否定、0 は節の終わりなので、1行に複数の節があります。p と二つの含意は r を強制し、not r と矛盾します。
small kernel は外部の Resolution の各ステップを replay しません。検査済み evidence から自分で生成した節について、信頼する SAT 検査器で UNSAT を確認します。ここで示した節列は符号化の具体例で、実装の実行ダンプではありません。〔アーキテクチャ仕様に基づく設計スケッチ〕

## Closing. Status And Roadmap（28–30分）

### Where The Project Stands (October 2026)（28:00）

仕様と実装を分けておきます。2026年10月時点です。
仕様は24章と付録。英語が正典です。
main branch に実装済みなのは、Rust のフロントエンド、alpha コーパス上の名前解決と型検査、検証条件生成と決定的な discharge、ATP 問題の符号化と候補 evidence、SAT に基づく kernel の evidence 検査、キャッシュ・指紋・ビルドスケジューリングの各マイルストーンです。
進行中は、ソースから検証済み成果物までの end-to-end 統合、LSP サーバ、ドキュメント生成です。
後回しは、MVM の実行、コード抽出、ライブラリ全体の前提選択、MML の移行です。
外部 prover を使った end-to-end の結果は、この講演では主張しません。〔実装状況、講演日に再確認〕

### Roadmap（28:45）

2026年は仕様を仕上げ、kernel までの Rust パイプライン、template 処理、alpha の end-to-end 実行を完成させます。
2027年は代表的な MML article を移行し、native hammer のベースラインを作り、MizAR と同じ単位、top-level theorem でベンチマークします。
2028年以降は移行の拡大、学習ベースの前提選択、LLM による失敗回復、MVM と抽出です。暗号と量子は将来構想のままです。〔計画／構想〕

### Closing（29:30）

Keep the foundation small. Keep the mathematics readable. Modernize everything else.
基盤を小さく保ち、数学的記述の可読性を維持し、周辺基盤を現代化する。
冒頭の6つの課題に対し、template、algorithm、開発基盤、evidence と SAT 検査という形で対応を示しました。
ありがとうございました。MML の移行、記述の利便性、自動化の評価について、ご意見をいただければ幸いです。

## 想定問答（Backup の使いどころ）

- **「58.4% と 68.8% は結局どちらが強いのか」** → 評価単位・前提・予算・成功判定が異なるため、設計の優劣を判定できない。Backup 1–2, 11–12。
- **「一階の方が ATP に有利なのか」** → 本講演はその優位性を根拠にしない。符号化と再構成は接続の仕組みとして説明し、性能は別に評価する。Backup 13–17。
- **「template は本当に一階のままか」** → 仕様 18 章：述語・関数子パラメータは schema レベル、インスタンス化ごとに一階義務。Backup 4。
- **「algorithm は結局 tactic 言語ではないのか」** → 契約付きで検証対象になる点が違う。tactic 的用途も含むが、それ自体が仕様化・検証される。MVM・抽出は未実装。
- **「kernel は何を信頼しているのか」** → 本編 6.3。evidence の対応と代入を検査し、kernel が生成した SAT 問題の UNSAT を確認する。詳細は Backup 5。
- **「どこまで動くのか」** → 末尾の実装状況の通り。end-to-end と外部 prover の結果は未主張。講演日に `doc/design/todo.md` で再確認。
