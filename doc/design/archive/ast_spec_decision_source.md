# Historical AST specification decision source

Original recorded reference: `doc/spec/temp/algorithm_on_AST.md`.
Retained excerpt: the current local `doc/spec/tmp/algorithm_on_AST.md`,
from the agreed expr/match table through Appendix B. Retention date: 2026-10-13.
The source text below is quoted verbatim in its original language.
The exact source version used by the completed AST-SPEC tasks is not
independently established by this local copy.
This frozen historical quotation is not current language authority,
sequencing authority or implementation evidence. Current requirements belong
to `doc/spec/en/`.

## 合意済み：expr of T と match

構文木は、組み込みの型 `expr of T` の対象として扱います。`T` はその木が表す式の値の型で、`val t : ghost const T` の `T` と同じものです。たとえば実数の式の木は `expr of Real` で、論理式の木は値が真偽値なので `expr of boolean` です。

| 項目 | 合意した内容 |
| --- | --- |
| 表記 | `expr` を小文字の予約語にする。`set` や `object` と同じ組み込みの型 |
| 構文木の作り方 | func や pred の適用は、オーバーロード解決後の定義をラベルとし、引数の木を子に持つ節点になる。節点の型は `expr of R` で、`R` はその定義の戻り値の型（pred なら `boolean`）。ラベルである定義は型に現れない（`sin p` の節点の型は `expr of Real` であり、`expr of sin` という型はない）。論理結合子（&、or、not、implies、iff）の適用も同じく節点になり、型は expr of boolean。引数の列は意味に必要な暗黙の値引数を明示した仮引数順とする。量化子、`the`、集合の内包表記は、中を見ない葉にする（不透明な葉）。葉は精緻化後の意味上の自由変数を宣言 UID で記録する（仮決定 8）。`x \|-> e` は正式の Binder で、内部では `Abs(κ, A, B, b)`（束縛変数の宣言 κ、定義域 A、共域 B、本体 b）として反射し、束縛関係を保つ。通常の適用節点とも不透明な葉とも区別する（仮決定 23）。各節点は解決済みの記号、元の型、変数の宣言元、解釈に必要な文脈を不変に保持する |
| 境界の規則 | `=` は常に対象の同一性。`expr of T` の対象では、本体でも仕様位置でも構造的同一性（構成子の種類、解決済みの記号、リテラルの値、不透明な葉が保存する精緻化済みの元の式、順序付き部分構造、変数の宣言 UID を含む不変な構造の一致。付録 A.1 S1）で、α 同値や意味の一致に読み替えない。意味は組み込みの ghost 関数 `val` で明示する：`t : expr of T` に対し `val t : ghost const T`（t の保持文脈に適合する現在の論理環境の下での意味）。論理式の木 p : expr of boolean では val p の型は ghost const boolean で、p の表す命題は `val p = true`。`val t` の型は ghost const T なので仕様位置（`requires`、`ensures`、`assert`、`invariant`、`decreasing`）でだけ使え、本体では使えない（`expr_size` と同じ扱い、§20.5.3）。実行の条件分岐や返値を意味値から作る経路（`if val p`、`return val t`、ghost の一要素列の走査）は置かず、数値が要る範囲には意味との一致を契約で保証する実行可能な評価 algorithm をライブラリで定義する（例：`eval_nat(t : expr of Nat) -> Nat ensures result = val t`）。位置で `=` を読み替える規則は置かない（仮決定 13） |
| 組み立ての規則 | R1：`expr of T` が期待される位置の `T` 型の式は構文木になる。R2：すでに構文である部分式はそのまま埋め込む。R3：構文が期待される位置では func と pred の適用は節点になり、構文でない引数は R1 で反射し、構文の引数は R2 で埋め込む。algorithm の適用は実行する。昇格した terminating algorithm も実行する（仮決定 9） |
| 節点の型 | 節点ごとに型を記録する。`t is expr of Real` や型を指定したパターン変数で判定できる。型が広がる方向は含め、属性は含めない。expr of T は T について共変で、T の対象が S の対象でもあるなら expr of T の対象は expr of S の対象でもある。ただし型の拡大は元の型・宣言 UID・保持文脈を変えない（`expr of Nat` を `expr of Real` として扱っても、自然数の変数節点に負の実数は付値できない）（仮決定 15） |
| 束縛 | 一点付値は `val_at(t, x, v)`、その許可は ghost 述語 `valid_at(t, x, v)` で表す（`val(t, x, v)` の綴りは採らない）。`for u being Real holds f.u = val_at(t, x, u)` のように書き、「x を u と読んだときの t の値」を表す。`x` は文脈で同一性が定まる変数節点に限り（`requires x is variable` と保持文脈の整合）、`valid_at(t, x, v)` は v が x の元の型・属性を満たし、更新後の環境でも依存と適用条件が保たれることを表す。`val_at` の戻り型は x に依存しない型に限る（`Element of {x}` のような型は `expr of Real` 等へ広げてから使う）（仮決定 16） |
| Binder x \|-> e | 表面文法は Identifier \|-> Term の一種類。項の中で最も弱く右結合（x \|-> y \|-> e は x \|-> (y \|-> e)、x \|-> e \| X は x \|-> (e \| X)）。頭部に型・領域の句は置かず、変数の型は外側の let・reserve と期待型で定め、曖昧なら Binder 全体への qua 注記で関数型を示す。定義域は期待される認定済み関数型の入力集合、なければ宣言型の認定された集合担体の順で決め、どちらもなければ拒否する。assume から定義域を抽出しない。通常の論理位置では基礎ライブラリの関数構成と一意性により関数対象（dom g = A、rng g ⊆ B、g(u) = E(u)）に変換し、反射では同じ意味を持つ Binder 節点にする。旧 MML の定数有限列 n \|-> a は (Seg n) --> a へ移行する（仮決定 23） |
| `expr_children(t)` | 子の並び（`FinSequence of expr of object`）を返す非 ghost の実行可能な純粋操作。本体でも仕様位置でも使える。変数節点と変数に依存しない原子的な定数節点では空列、通常の適用節点では意味上の暗黙引数を補った仮引数順の引数の木（重複を保持）、不透明な葉では記録された自由変数の節点列（精緻化後の初出順、UID で重複を除く）、Binder `Abs(κ, A, B, b)` では `[A, B, Var(κ), b]` の順（重複を保持）。各子は元の型と必要な文脈を保持し、取得のたびに UID を変えない。宣言型・属性・環境・証拠・キャッシュは走査辺に含めない（仮決定 21） |
| `for c in expr_children(t)` | 通常の列の走査。任意の ghost 有限列を非 ghost の変数に束縛する規則は置かない。子の列を変数に保持したり長さを計算したりすることを特別に禁じる規則も置かない（仮決定 21） |
| expr\_size(t) | ghost の正確な自然数。expr\_size(t) = 1 + Σ expr\_size(expr\_children(t)\[i\])（子の各位置について。共有を展開した出現数で、DAG の共有やキャッシュで値は変わらない）。変数節点と原子的定数は 1、不透明な葉は 1 + \|FV\|、Binder は 2 + size A + size B + size b。c in rng expr\_children(t) implies expr\_size(c) < expr\_size(t) を停止性の補題として使い、MVM が再帰ごとに実サイズを計算することは要求しない。固定幅のオーバーフローや丸めを値として採用しない（仮決定 19） |
| 意味規則と信頼上の位置づけ | 記号ごとの意味の等式（例：val (sin p) = sin (val p)）、命題の真理条件（例：val (a <= b) = true iff val a <= val b）、反射との一致、型に適合する更新と非出現の性質を、適用条件を伴う一階論理の図式として生成する。R1 による通常の項 e の純反射は val t = e、命題 P の純反射は val t = true iff P。R2 で挿入する木は、その構文対象ではなく意味を合成する（例：p : expr of Real を埋め込んだ sin p の意味は sin (val p)）。内部の環境意味論では `Val(Var(κ), ρ) = ρ(κ)`、`Val(Node_f(p1, …, pn), ρ) = f(Val(p1, ρ), …)`。Binder は `D = Val(A, ρ)`、`K = Val(B, ρ)` として `Val(Abs(κ, A, B, b), ρ) = g`、`dom g = D`、`rng g c= K`、`for u st u in D holds g.u = Val(b, ρ[κ := u])`。本体の更新前の適合環境を要求する公開 val\_at を、一般の Binder の意味規則へ無条件に用いない。不透明な葉は保存された元の項・論理式の意味と、UID による捕獲回避の代入補題に接続する。構造の定義・型規則・表現不変条件・導出補題、検証済みの契約、MVM の個別実行証拠は信頼上の役割が異なるため、付録 A.1〜A.3 で区別し採用条件を示す。構造上の子の有限性は構造再帰と expr\_size の根拠であって、意味規則全体の無矛盾性や任意の定義展開の停止性の根拠ではない |
| `match` | 対象を `expr of T` 型に限る。検証条件の e = π は = の通常の意味で木の等式になり、特別扱いは要らない（仮決定 13）。節点の形は開いた集合なので網羅性の証明はなく、網羅性は `otherwise` でだけ扱う（仮決定 10）。パターンで捕捉した名前はその分岐内だけで有効な不変の局所束縛で、型が一意に決まる場合は前置の var 宣言を要しない（コード例はこの形）。Binder のパターン `case x \|-> body` は、対象 `Abs(κ, A, B, b)` の格納済みの `Var(κ)`、本体 b、入力型、領域 A、外部文脈を捕捉する（パターンの綴りと元の名前は一致しなくてよい。照合は構文の種類による。新しい UID の生成や改名はしない）。共域 B は捕捉元ごとの再構成文脈 C の既定値として保持する。AST 構築位置の `x \|-> e'` で x が捕捉済みの変数なら、保存された領域と UID で閉じて再構成し（領域を再推論・拡大しない）、共域は C の既定値か `(x \|-> e') qua expr of Function of A, B'` の注記で指定する。R2 は自由 UID を保存し、内側の束縛点の衝突だけを α 改名で処理する（仮決定 17、23） |

`match` を構文木に限るのは、健全性の問題を封じるためです。今の仕様では、実行時の照合は字句で行うのに、検証条件の否定仮定 `¬∃p,q. e = p + q` は意味で扱います。実数は全て `r + 0` と書けるのでこの仮定は常に偽となり、`otherwise` 節で何でも証明できてしまいます。

## 合意済み：部分的な algorithm と fail

仕様書（EN と JA）に反映済みです（コミット 8737e8ba）。一部は単位 3 で修正します（反映計画）。

| 項目 | 内容 | 反映先 |
| --- | --- | --- |
| `fail` 文 | `fail ["メッセージ" {, 式}];`。実行時の検証失敗を起こし、その経路には検証条件を作らない。`terminating` の algorithm と ghost の文脈では使えない。捕捉する構文はない | §2.4、§20.2.6、付録 A |
| `otherwise` を省いた `match` | 部分的な algorithm でだけ許し、`otherwise fail "no case matched", e; end;` の省略形とする | §20.2.5 |
| 評価述語と評価公理 | 部分的な algorithm にも関数記号 f を与える。`returns_f(x, r)` は完全な入力 x で有限回の実行により r を正常に返したことを表す。完全な入力には構文対象・通常の実行値・必要な保持文脈・再構成文脈 C の暗黙引数を含め、実行と論理で一致させる。基本の生成規則は `returns_f(x, r) implies f(x) = r` と、検証済みの契約 `P(x) & returns_f(x, r) implies Q(x, r)` の二つ（付録 A.3 E1・E2）。単値性 `returns_f(x, r1) & returns_f(x, r2) implies r1 = r2` は前者から導かれ、実行意味論の単値性はこの関数記号との接続を正当化する前提でもある（E3）。未返却の引数での f(x) は未規定の値。`f(x) = r` から returns\_f や ensures を逆向きに導かない。失敗・タイムアウト・資源不足の試行は新しい正常返却の証拠を与えず、抽象的な正常返却の不存在も証明しない（仮決定 4） | §20.13.1 |
| 戻ることの根拠 | MVM の正常返却、本体内の呼び出しから正常に戻った経路（部分正当性）、正常返却の存在を示す定理、の 3 つに限る。単なる停止と区別する | §20.13.1 |
| 部分的な algorithm の再帰 | `decreasing` なしで再帰でき、戻った再帰呼び出しの `ensures` を使える。評価の長さについての帰納法で健全 | §20.13.4 |
| エラーコード | E0331（実行時の失敗）、E0503（失敗が許されない場所での `fail`）を追加し、E0501 を更新。E0504（論理式中の部分的な呼び出し）は単位 3 で削除（仮決定 4） | §22 |
| terminating | 前提を満たす全入力で正常返却することを保証する（P(x) implies ex r st returns\_f(x, r)）。直接の fail、暗黙の失敗、子呼び出しや反復範囲の評価の発散・失敗も対象。部分的な algorithm を呼ぶ場合は、その呼び出しの前提と正常返却を示す | §20 |

## 仮決定：by の後の algorithm の扱い

algorithm の引数は常に利用者が明示的に渡します。

整理は次のとおりです。

| 書き方 | 意味 |
| --- | --- |
| `by computation(alg(引数), …)` | 列挙した呼び出しと主張に現れる呼び出しを MVM が評価する。正常返却と requires の正当化を根拠に、ensures の実例全体と、通常の数学的な呼び出しとして表せる値の等式を公開候補にする。各候補は付録 A.4 の認定変換と残存検査を通過してから前提に加える。returns や構文引数を含む値の等式は内部の適用証拠として保持する（仮決定 2、4、20）。構文木を取る algorithm はこの列挙で評価を指示し、構文木を取らない algorithm は呼び出しを主張に書く。資源オプションは computation(steps: …, replay: …) の名前付き引数（仮決定 11） |
| `expr of boolean` の位置のラベルと `thesis` | ラベルはその主張を、`thesis` は現在の目標を表す。使ったラベルは引用したものとして扱う。thesis は thus と hence の直下でだけ書ける（仮決定 3） |
| 昇格した algorithm の公理の引用 | func と同じく定義ラベルで引用する。`terminating algorithm FactorialDef: factorial(n) -> Nat` と書き、公理は `by FactorialDef` で引用する。ラベルは `terminating` の algorithm にだけ付けられ、省略できる。名前だけの引用はなく、`by` の後のラベルは引用、computation(…) の中の適用は計算と見た目で区別できる（仮決定 1） |

明示的に引数を渡す方針では、定理の `st` の仮定も引数として書き直す必要があり、少し冗長になります。暗黙の受け渡しをなくしたことの代償です。

## コード例

本体の中で実行した呼び出しの ensures は文脈にあるので（§20.13.1）、algorithm を名前で引用する書き方はありません。ensures の中で構文木の意味を取るところは `val` で明示しています（仮決定 13）。algorithm の名前は動的な手続きであることが分かるように動詞で始め（differentiate、differentiate\_expr、check\_occurs、try\_link、try\_prove\_order\_chain、step\_collatz）、名詞の func、形容詞の pred と区別します（付録 D の推奨規則）。

### 大小関係の連鎖

次の例は、連言に含まれる二本の不等式を一回つないで目標を得る手続きで、任意長の連鎖の閉包は計算しません。false は目標の反証を意味しないので、入口名を `try_prove_order_chain` とします。パターンで捕捉する変数は分岐ローカルの束縛で、`expr of Real` として型検査されます（型注記の文法は未確定）。

```mizar
definition
  let p, q, g be expr of boolean;

  terminating algorithm try_link(p, q, g) -> boolean
    ensures (result = true & val p = true & val q = true) implies val g = true
  do
    match p do
      case a <= b do
        match q do
          case c <= d do
            match g do
              case e <= f do
                if a = e & b = c & d = f do     :: 木の一致（= の通常の意味）
                  return true by XXREAL_0:2;    :: 実数の <= の推移律と val の公理
                end;
              end;
              otherwise end;
            end;
          end;
          otherwise end;
        end;
      end;
      otherwise end;
    end;
    return false;
  end;
end;

definition
  let h, g be expr of boolean;

  terminating algorithm try_prove_order_chain(h, g) -> boolean
    ensures (result = true & val h = true) implies val g = true
  do
    const hs := split_conjuncts(h);    :: 連言を再帰的にほどいた列。正常返却と val h = true implies 各 val(hs.(i)) = true を保証する
    for i = 1 to len hs do
      for j = 1 to len hs do
        if try_link(hs.(i), hs.(j), g) = true do
          :: 実行した split_conjuncts と try_link の ensures は文脈にある（§20.13.1、仮決定 1）
          return true;
        end;
      end;
    end;
    return false;
  end;
end;
```

```mizar
theorem: for x, y, z being Real st x <= y & y <= z holds x <= z
proof
  let x, y, z be Real;
  assume A1: x <= y & y <= z;
  thus x <= z by computation(try_prove_order_chain(A1, thesis));
end;

:: 三本をつなぐ場合は中間の主張を明示する（一度に渡すと try_link が二本しかつながないので false になる）。
:: x, y, z, w はこの証明位置で実数として固定されている。A3 は thus/hence の直下ではないので thesis を使わない
A1: x <= y & y <= z;
A2: z <= w;
A3: x <= z by computation(try_prove_order_chain(A1, x <= z));
thus x <= w by computation(try_prove_order_chain(A3 & A2, thesis));
```

### sin/cos を含む微分

外部入口の木の型は `expr of Function of REAL, REAL` で、値は全実数上の関数対象です。関数は数学表記の Binder `u |-> (sin u) * u` で書き、Binder の変数と評価点を別名にします（仮決定 23）。通常の論理位置では Binder は関数対象に変換され、反射では束縛関係を保った節点になります。内部の補助手続き `differentiate_expr` の契約は `val_at`・`valid_at`（仮決定 16）で書け、要求仕様 D1〜D3 とコード案で示します。補題名は FDIFF\_1、FDIFF\_2、SIN\_COS、VALUED\_1 の該当定理を指す候補で、正確な対応は仕様反映時に確認します。

**D1（入力と更新の許可）** `t : expr of Real`、`x : expr of object` で、`requires x is variable & for u being Real holds valid_at(t, x, u)`（t と x の保持文脈は整合する）。各適合する外部環境 ρ でこの前提を示し、一つの環境での成立を別の環境や文脈へ無条件に拡張しない。`x is variable` と `t is expr of Real` だけからは導かない。D1 の下で、実数関数 Fρ(t, x)(u) = Val(t, ρ\[x := u\]) を略記として定める（表面記法では `z |-> val_at(t, x, z)`。z は `reserve` 等で実数型が決まる契約用の束縛名）。

**D2（正常返却と結果の全実数での適用条件）** r を正常に返したとき、全ての実数 u について `valid_at(r, x, u)` が成り立ち、Fρ(t, x) が u で微分可能で、その導関数の値が Val(r, ρ\[x := u\]) に等しい。結果側の `valid_at` は導関数を再び全実数上の関数対象として閉じるために必要で、入力側だけで済ませない。

**D3（結果の型とスコープ）** 結果は入力から保持した元の外部文脈と x の下で型付けされ、入力の文脈にない自由 UID を追加しない。再帰呼び出しの一時変数名（dp など）は R2 で埋め込まれ、新しい自由変数として木に混入しない。構文対象の型付け・保持文脈・構成と再構成の規則で検証し、専用の述語は足さない。

```mizar
reserve z for Real;

algorithm differentiate_expr(t, x) -> expr of Real
  requires x is variable
    & for u being Real holds valid_at(t, x, u)
  ensures for u being Real holds
    valid_at(result, x, u)
    & (z |-> val_at(t, x, z)) is_differentiable_in u
    & diff((z |-> val_at(t, x, z)), u) = val_at(result, x, u)
do
  if t = x do return 1; end;
  if check_occurs(x, t) = false do return 0; end;
  match t do
    case p + q do
      const dp := differentiate_expr(p, x);
      const dq := differentiate_expr(q, x);
      return dp + dq;
    end;
    case p * q do
      const dp := differentiate_expr(p, x);
      const dq := differentiate_expr(q, x);
      return dp * q + p * dq;
    end;
    case sin p do
      const dp := differentiate_expr(p, x);
      return (cos p) * dp;
    end;
    case cos p do
      const dp := differentiate_expr(p, x);
      return -(sin p) * dp;
    end;
  end;   :: otherwise なし: 未対応の形は実行時に失敗する
end;
```

各返却には、和・積・合成・sin/cos の微分則と構文の意味の対応を使った証明が要ります。`t = x` の分岐は変数の意味等式と D1 から恒等関数の微分に、`check_occurs` の false の分岐は各適合環境での更新不変性から定数関数に帰着します。和・積・sin・cos の分岐は、解決済みの記号の型から各子について D1 を示し、再帰呼び出しの D2・D3 と微分則で結果を証明します（意味等式は内部の更新後環境で使い、子の型が x に依存する場合に公開の val\_at を無条件に当てはめません）。otherwise のない match で未対応の形に達すれば部分的 algorithm の規則で失敗します。外部入口は入力・結果とも全実数上の関数の木とし、本体は Binder のパターンで捕捉し、捕捉した UID で閉じて再構成します（仮決定 23）。

```mizar
algorithm differentiate(f) -> expr of Function of REAL, REAL
  ensures for u being Real holds
    (val f) is_differentiable_in u
    & diff(val f, u) = (val result).u
do
  match f do
    case x |-> body do
      const derivative := differentiate_expr(body, x);
      return x |-> derivative;
    end;
  end;
end;
```

入口と補助手続きの接続は四段で正当化します。本体を開く（Binder の型付けから、捕捉した領域の意味が REAL に等しく、外部環境を全実数で拡張した環境で本体が適合することを得る。領域の式が字面で REAL と一致することは要求せず、variable 属性だけを根拠にしない）、本体の型と D1 を示す（全入力についての値域証明による検証済みの型ビューで本体を `expr of Real` として扱い、本体の保持文脈で全実数への `valid_at(body, x, u)` を示す）、結果を閉じる（正常返却で得た derivative と結果側の valid\_at を使い、元の UID で閉じて `expr of Function of REAL, REAL` にする。他の自由 UID は捕獲しない）、契約を接続する（入力と結果の各点の意味等式を D2 に当てて外部契約を得る）。利用側は、定理の証明の中で `by computation(differentiate(…))` と書きます（仮決定 2）。評価が完了すると、ensures に引数と結果を代入し、Binder と val の認定された意味の変換を適用したもの（木も UID も val\_at も残らない関数の等式）が文脈に入ります。

```mizar
reserve u for Real;

theorem: for x being Real holds diff((u |-> (sin u) * u), x) = (cos x) * x + sin x
proof
  let x be Real;
  :: 主張の u |-> (sin u) * u は関数対象に変換される（仮決定 23。定義域は reserve の Real の担体 REAL、共域は本体の型から REAL）。
  :: computation が differentiate を評価し、結果の木 u |-> ((cos u) * 1) * u + (sin u) * 1 について ensures の実例が
  :: 変換・検査されて文脈に入る。点 x での評価と算術の簡約で thesis が出る
  thus thesis by computation(differentiate(u |-> (sin u) * u)), VALUED_1;
end;
```

differentiate\_expr が使う check\_occurs です。表面契約は「false を返したなら、許可された各更新で意味が変わらない」を `val`・`val_at`・`valid_at` で述べ、契約の読み方（仮決定 16）により各適合環境について保証します（一つの環境だけを調べて証明済みとはしない）。この具体的な構造走査の実装については、より強い内部補題「false なら UID(x) not in FV(t)」を構造帰納で併せて証明し、接続条件から表面契約を導きます。両者は同じ強さではなく、`x - x` に意味上の不変性だけで false を返す別の実装は表面契約を満たし得ても非出現までは得られません（本実装は `x - x` に true を返してよく、true から意味上の依存は結論しません）。FV は意味上の暗黙引数、Binder の A・B と本体の束縛、不透明な葉の依存を含む内部の数学的な記法で、新しい組み込み名ではありません（旧契約 `result = false implies ex c st for v holds val(t, x, v) = c` は一点付値しか述べず、Binder をまたぐ合成ができないので採らない）。

```text
表面契約:
  check_occurs(x : expr of object, t : expr of object) -> boolean
    requires x is variable   [t と x の保持文脈は整合する]
    ensures result = false implies
      (for v being object st valid_at(t, x, v) holds val t = val_at(t, x, v))

内部補題（この実装について構造帰納で証明する）:
  returns_check_occurs(x, t, false) implies UID(x) not in FV(t)

  UID(x) not in FV(t), Adm(ρ1), Adm(ρ2), ρ1 and ρ2 agree except at UID(x)
    imply Val(t, ρ1) = Val(t, ρ2)

  FV(t) ⊆ ⋃ { FV(c) | c in rng expr_children(t) }   （変数節点以外。Binder では等号にならない）
```

```mizar
terminating algorithm check_occurs(x, t) -> boolean
  requires x is variable
  ensures result = false implies
    (for v being object st valid_at(t, x, v)
       holds val t = val_at(t, x, v))
  decreasing expr_size(t)
do
  if t = x do return true; end;
  for c in expr_children(t) do
    :: 検証用の不変条件：処理済みの全ての子 c で UID(x) not in FV(c)
    if check_occurs(x, c) = true do return true; end;
  end;
  return false;
end;
```

最後の `return false` の証明には、各子への再帰の停止性（尺度の補題。再帰先は全て expr\_children(t) の要素）と正常返却後の契約、処理済みの添字集合について非出現を保ちループ通過時に全添字を含むことを要求する不変条件（現在の要素が列に含まれるという条件だけで全要素の処理を代用しない）、非変数節点について FV(t) ⊆ ⋃ FV(c) の包含、の三点が要ります。コメントの不変条件は証明義務であって、公開の FV 関数や新しい文法ではありません。Binder に対して「直接の子の現在の値が x に依存しなければ親も依存しない」という規則は使えません。`y |-> y * x` の本体を開いて y に 0 を与えると子の値は x を変えても 0 ですが、Binder 全体は x の値 v に応じて `u |-> u * v` になります。これは走査の誤動作ではなく、一点付値の不変性だけでは親への合成ができないことを示す例です。

`(cos p) * dp` のうち、`cos p` と `*` は節点になり（R3）、`dp` には実行した再帰呼び出しの結果が埋め込まれます（R2）。未対応の記号でも微分変数に依存しなければ定数分岐を使えます。失敗するのは `exp x` のように微分変数に依存する未対応の式で、診断には対象の式、微分変数、呼び出しの経路を示します。Binder の入れ子を言語が許すことは、differentiate が任意の入れ子を処理する保証ではありません。

### コラッツ（部分正当性の例）

仕様書の §20.13.4 に入っている例です。末尾の二つの定理は、単位 3 で by computation の形に書き換える予定です（仮決定 4）。

```mizar
definition
  let n be Nat;

  terminating algorithm step_collatz(n) -> Nat
    ensures n >= 1 implies result >= 1   :: 再帰呼び出しの requires n >= 1 を契約から満たすために要る
  do
    if n mod 2 = 0 do return n div 2; end;
    return 3 * n + 1;
  end;
end;

definition
  let n be Nat;

  :: terminating ではない: 全ての n >= 1 での停止性は未解決問題
  algorithm count_collatz_steps(n) -> Nat
    requires n >= 1
    ensures iterate[Nat, step_collatz](result, n) = 1
  do
    if n = 1 do return 0 by iterate; end;
    return count_collatz_steps(step_collatz(n)) + 1 by IterateShift;
  end;
end;

:: 単位 3 の後の形（提案、仮決定 4）。部分的な algorithm の呼び出しも主張に書け、by computation で評価する
theorem CollatzSteps27: count_collatz_steps(27) = 111 by computation;

:: 評価が完了すると count_collatz_steps(27) = 111 と ensures の実例 iterate(111, 27) = 1 が文脈に入る
theorem Collatz27Reaches1: iterate[Nat, step_collatz](count_collatz_steps(27), 27) = 1
  by computation;
```

## 反映計画

単位 1 は反映済みです。設計判断 1〜23 は 2026-10-09 に確定し、付録 A・B のレビューに基づいて規則の適用条件と検証根拠を補正しました。単位 2 以降の仕様化では、下記の先行成果物を揃え、付録の図式を対象文法・構文形・基礎ライブラリの認定と対応付けます。各単位で EN と JA を同時に更新し、差分の確認を得てからコミットします。基礎ライブラリとの初期接続は標準の定義 ID と証明の固定した対応表で始め、任意の利用者定義 Binder の登録機構、汎用の環境・置換 API、一般的な AST 再構成関数は必須項目にしません。

| 単位 | 内容 | 主な反映先 | 状態 |
| --- | --- | --- | --- |
| 1 | 部分的な algorithm と `fail`、単値性、正常返却、`terminating` からの部分的な子呼び出し | §2.4、§16.5.1、§20、§22、付録 A | 反映済み（8737e8ba）。一部を単位 3 で修正。2026-10-09 の追補（単値性、正常返却、子呼び出し）は未反映で、現在の実装と照合する |
| 2 | `expr of T` と `match`、`val`、`val_at`・`valid_at` と環境に基づく付値、属性 variable、境界の規則（仮決定 18、20）、正式の Binder `x \|-> e`（文法と優先順位、let・reserve・期待型・qua による型と領域、基礎ライブラリとの認定表、関数対象への変換、環境、三つの解釈モード、照合・再構成文脈 C・R2。仮決定 23）、実行可能な `expr_children` と列挙規則、通常の列走査の `for … in`、ghost の `expr_size` の定義と尺度の補題（仮決定 19、21）、組み込み ghost 関数の改名と整理、旧 `n \|-> a` の移行。Binder を含まない部分（2a）を先に、Binder（2b）は受け入れ条件 G1〜G7・S1〜S6・V1〜V18 と付録関連の K1〜K12 で検証しながら進める。規則の区分・採用条件は付録 A.1〜A.3、診断は付録 B に従う | §2.4、§3.4、§13、§14、§19、§20、§21.7.1、§22、付録 A | 確定、着手可 |
| 3 | `by` の後の algorithm（computation(alg(引数)) への適用の統合、ラベルと `thesis`、定義ラベル案、括弧付きの引用、公開する全ての取り込み事実の検査とそのエラーコード（仮決定 20。付録 A.4 の規則・処理順・適用条件と付録 B の診断を基にし、K1〜K12 で確認する）） | §16.5.1、§20、§21、§22.4、§23.5.1、§23.6.1 | 確定、着手可 |
| 4 | 例と推奨規則（algorithm 名は動詞で始めるという命名規則、テンプレートの例、外に出す algorithm を val の契約のラッパーで包む書き方、契約が関数について述べる algorithm は関数値の木を取るという規則（仮決定 22）、新しい Binder は共通の関数抽象と通常の演算への展開から検討するという推奨規則（仮決定 23）、`split_conjuncts`、`check_occurs`、診断、必要な括弧と型注記、旧有限列の移行例、微分と大小関係の例） | §18.8.4、§20.12、付録 D、sample\_codes | 未着手 |
| 5 | MVM の信頼モデル（`by computation(replay: true)`、`computation_replay`、状態 `"computed"`、構造操作の決定性、共有とキャッシュ、正常返却の再現性、資源上限） | §20.9.2、§21.7.1、§23 | 未着手 |
| 6 | 設計文書とタスク（architecture 15 の線形算術の節、todo.md、実装タスク） | doc/design | 未着手。architecture 15 と todo.md には古い案（式の木と `Eval`）が決定として残っている。todo.md は自律エージェントの入口なので、1 段落の訂正だけ単位 2 より先に入れる |

単位 3 で直す予定の箇所は次の 6 つです。

- §16.5.1 の表の「部分的なアルゴリズム」の行。評価公理は計算の事実なしには働かないので、名前だけの引用を許したのは誤りでした
- §20.13.1 の「`f` を引用すると評価公理が引用される」という文
- §20.13.4 のコラッツの例の `by Collatz27, count_collatz_steps`
- 論理式中の `f(c) = v` を `returns_f` と読む規則と E0504（仮決定 4：読み替え規則を廃止し、部分的な algorithm にも関数記号を与えて呼び出しを論理式に書けるようにする。E0504 は削除）
- §20.3 の「空でないことを静的に示せなければ MVM が実行時に失敗させる」という記述（仮決定 6：部分的な algorithm では暗黙の `fail`、`terminating` では E0503）
- 境界で拒否する対象、診断位置、新しいエラーコードの確定（仮決定 20。未確認の番号はこの文書では割り当てない）

### 正式仕様の章に先立って揃える表

この文書は設計判断の記録であり、言語全体の文法・型検査規則を網羅した仕様書そのものではありません。付録 A・B は、その記録を規則の適用条件・信頼上の役割・処理順・診断まで具体化したものです。次の三つの表は対象資料との照合が必要なので、仕様化の先行成果物として作成します。記載のない構文や認定の対応を、実装者が推測して補うことはしません。

| 先行成果物 | 固定する内容 | 照合先・反映単位 |
| --- | --- | --- |
| 文法の集約 | `expr of T`、algorithm の契約句、match/case/otherwise、for、Binder と全体への qua、computation の引数を生成規則と既存非終端記号への接続表にする。新しい Binder の規則と既存文法との境界を明示する | 対象仕様書の §13・§14 など。単位 2・3 |
| 構文形と精緻化の分類表 | 全ての項・論理式の形を構造上の分類に対応付け、意味上の暗黙引数と型・属性・適用条件を区別する。保持する条件と宣言元・依存関係を記録する規則を各構文形について定め、付録 A.2・A.4 の生成規則に接続する | 対象文法・型検査規則。単位 2a・2b。付録の一般則だけから未分類の形を反射・変換可能としない |
| 基礎ライブラリの認定表 | 対象 MML バージョンを固定し、`Real`／`REAL`、`Nat`／`NAT`、`Function of`・`PartFunc of`・適用・dom・rng・Seg・`-->`、関数構成の scheme、付録 A.4 で利用する boolean の定義・二値性の根拠を定義 ID と証明に対応付ける | 使用する規則に先立って単位 2a に必要な共通部分を固定し、単位 2b の最初の成果物として Binder に関する対応を補う。単位 3 も同じ表を参照する。綴りの一致だけで認定せず、認定未成立の規則は発行・適用しない |

## 決定の一覧（2026-10-09 確定）

2026-10-06 のレビューの推奨案を仮に採用して上の表とコード例に反映し、2026-10-09 に全項目を確定した。同日の追加決定（Binder の文法・型・領域、val\_at・valid\_at による付値、子の列挙と尺度、照合・再構成・R2）は 8、13〜17、19、21〜23 に反映している。同日の付録レビューでは、これらの設計判断を維持した上で、意味規則の型・スコープ条件、認定変換の手順、検証してから採用する規則の区分を補正した。番号は維持し、本文中の「仮決定 n」はこの一覧の番号を指す。

- [x] 1\. 昇格した algorithm の公理は (B) 定義ラベルで引用する。`terminating algorithm FactorialDef: factorial(n) -> Nat` と書き、`by FactorialDef` で引用する。ラベルは `terminating` にだけ付けられ、省略できる。algorithm を名前だけで引用する書き方はなくす。本体の中で実行した呼び出しの ensures は文脈に入るので引用不要（§16.5.1 に func の定義ラベルが入った 2ad38231 と整合）
- [x] 2\. 適用の形 `by alg(引数)` は独立した構文としては置かず、構文木の algorithm の評価は `by computation(alg(引数), steps: …, replay: …)` で指示する。`by computation` は、主張に現れる呼び出しと引数に列挙された呼び出しを評価し、ensures の実例全体と、通常の数学的な呼び出しとして表せる値の等式を公開候補とし、仮決定 20・付録 A.4 の検査後に文脈に加える。構文を含む値の等式と returns は内部の適用証拠に保持する。列挙した呼び出しの引数は R1 で反射され、requires は証明義務になり、取り込む ensures は仮決定 20 の検査を受ける。構文木を取らない algorithm は、terminating でも部分的でも、呼び出しを主張に書けばよい（`theorem factorial(5) = 120 by computation`、仮決定 4）。ラベルは引用、computation の中の適用は計算、と見た目で区別できる（2026-10-08 に改訂。旧：適用の形を独立した構文とし、構文木を取らない部分的な algorithm にも認める）
- [x] 3\. `expr of boolean` の位置にラベルと `thesis` を書ける。`thesis` は `thus` と `hence` の直下に限る
- [x] 4\. 部分的な algorithm にも関数記号 f を与える（2026-10-08 に改訂）。基本の生成規則は `returns_f(x, r) implies f(x) = r` と、本体の部分正当性を検証してから採用する契約の評価公理 `P(x) & returns_f(x, r) implies Q(x, r)` の二つ（付録 A.3 E1・E2）。単値性 E3 は E1 と等式の推移性から導かれる一方、実行意味論が完全な入力に対して単値であることは E1 の接続を正当化する前提である。E1〜E5 の全てで、必要な保持文脈と C を含む暗黙引数も同じ完全な論理入力に含める。構文対象の契約と terminating の保証は付録 A.2 M8 の環境適合性・InputOK・前提の下で読む。評価が完了しない引数に対する f(x) は公理が何も語らない未規定の値で、全域な関数記号しか持たない一階論理の標準的な扱い（正常返却の単値性と型の適合、未返却時を含む関数記号の導入条件を検証して正当化する。単値性を公理に追加するだけで実行との不一致を解消するものではない）。これにより部分的な algorithm の呼び出しは論理式に書け、主張に現れる呼び出しは `by computation` で評価できる。正常返却と requires を確認すると returns\_f(c, v) を内部の証拠にでき、`f(c) = v` と ensures の実例 Q(c, v) は公開候補全体の変換・検査を通過して文脈に入る（例：`theorem count_collatz_steps(27) = 111 by computation`、`theorem iterate[Nat, step_collatz](count_collatz_steps(27), 27) = 1 by computation`）。論理式中の `f(c) = v` を `returns_f` と読み替える規則と E0504 は廃止のまま（等式は本物の等式で代入できる）。旧案（2026-10-07）：関数記号を与えず、呼び出しを論理式に書くこと自体を禁止し、値は ensures 経由でだけ取り出す。「111 を返す」という事実を定理に書けないのが代償だったが、懸念していた「代入できない等式」は読み替え規則の問題で、本物の関数記号なら生じないため撤回
- [x] 5\. 括弧付きの引用 `by alg[T, P, Q](引数)` と、演算子の記号（`<=` など）を pred の実引数として渡すことは、単位 4 の例が必要とするまで先送り
- [x] 6\. §20.3 の「空でないことを静的に示せなければ MVM が実行時に失敗させる」という記述を塗る。部分的な algorithm では暗黙の `fail`、`terminating` では E0503 相当のエラーにする（レビュー指摘 3）
- [x] 7\. 記号を問わない節点の先頭の記号を調べる機能、節点を組み立て直す関数、struct の値を分解する `match` は、必要になった時点で検討する
- [x] 8\. 量化子、`the`、内包表記は当面、内部の構文を通常の match で分解しない不透明な葉とする。精緻化（記号解決と意味上の暗黙引数の明示化。任意の定義展開ではない）後の意味上の自由変数を宣言 UID で記録し、`expr_children` は初出順で重複を除いた変数節点の列を返す。宣言型・属性・適用条件の依存は適合環境の条件として保持し、値の式の自由変数と区別する（`the Element of {x}` の選択対象の x は自由変数に含め、`y : Element of A(x)` の型前提の x は含めない）。`check_occurs` はライブラリのまま正しく、葉の意味は保存された元の項・論理式へ接続する。自由出現への更新は、宣言 UID に基づく捕獲回避の代入とその意味保存の補題で正当化し、公開の置換 API や不透明な内部を分解する match は追加しない（付録 A.2 M3・M6、レビュー指摘 1）。正式の Binder（`x |-> e`、仮決定 23）は通常の適用節点とも不透明な葉とも区別し、領域 A、共域 B、束縛変数、本体を順に子とする（2026-10-09 改訂、同日の追加決定で精密化）
- [x] 9\. 構文が期待される位置では algorithm の適用は常に実行する。昇格した `terminating` algorithm も同じで、節点にはならない（レビュー指摘 2）
- [x] 10\. `exhaustiveness_proof` と予約語 `exhaustive` を削る。構文木の節点の形は開いた集合なので、網羅性は `otherwise` でだけ扱う（レビュー指摘 4）
- [x] 11\. 資源オプション steps と replay は `computation(…)` の名前付き引数として渡す（例：`by computation(differentiate(u |-> (sin u) * u), steps: 100000)`）。その computation で評価する全ての呼び出しにかかる（レビュー指摘 8。単位 5。2026-10-08 に仮決定 2 と統合）
- [x] 12\. `by` の引数は、その位置のスコープにある変数（proof の `let` や `assume` で固定した変数、reserve の変数）を参照できる。proof ブロックのない一行の `by` から主張の量化子の内側の変数は参照できない。新しいスコープ規則は加えず、let の定数が証明の残りで使えるという既存の規則をそのまま適用する（レビュー指摘 7。2026-10-08 に改訂。旧：定理の変数を使うときは reserve を書く。reserve の変数が正当化にしか現れず出所が読めない書き方を許していた）
- [x] 13\. 意味は組み込みの ghost 関数 `val` で明示する。`t : expr of T` に対し `val t : ghost const T` で、t の保持文脈に適合する現在の論理環境の下での意味。論理式の木 p : expr of boolean では val p の型は ghost const boolean で、p の表す命題は `val p = true`。`=` は常に対象の同一性で、`expr of T` の対象では構造的同一性（構成子の種類、解決済みの記号、リテラルの値、不透明な葉が保存する精緻化済みの元の式、順序付き部分構造、変数の宣言 UID を含む不変な構造の一致。付録 A.1 S1）。α 同値や意味の一致へ読み替えず（Binder ID を変えた α 改名は別の木で、意味の等しさは Binder の意味の規則で扱う）、位置で `=` や式を意味に読み替える規則も置かない。共有状態・キャッシュ・格納先は同一性の一部にしない。`val` の型は ghost const T なので本体では使えない（`expr_size` と同じ扱い。予約語は増えない）。`val`・`val_at`・`valid_at` は契約・assert・不変条件・証明で使え、実行の条件分岐・通常の返値・反復回数や列要素を意味値から作る用途には使えず、ghost の有限列を for で走査して持ち出す例外も置かない。数値が要る範囲には、対応する構文と通常の実行可能な演算だけを用いて意味との一致を契約で保証する評価 algorithm をライブラリで定義する（例：自然数リテラルと加算だけの閉じた式を計算する `eval_nat(t : expr of Nat) -> Nat ensures result = val t`。本体はリテラルのペイロードを通常の自然数として読み子を再帰して計算し、`return val t` とは書かない。開いた式は自由変数の値を通常の引数で受け取る）。有限で閉じた木や Nat 型だけを根拠に汎用の val を実行可能にできるとはしない（命題の真偽で 0 か 1 になる自然数の評価は真偽判定を要求する）。命題の木の検査手続きは `result = true implies val p = true` を保証し、完全な決定手続きを証明した範囲にだけ同値の契約を与える。match の検証条件の特別扱いと、本体で意味を条件に使ったときの専用エラーは不要になる。名前は `eval` を避けた（第 20 章で「評価」は MVM の実行を指す）。単位 2 で §20.5.3 の組み込み ghost 関数の節に追加する（旧：束縛は val の中の変数節点にだけ効く。仮決定 16 の付値に置き換えた。2026-10-09 に精密化）
- [x] 14\. 名前は「式と値」の対にする（2026-10-07 確定）。型は `expr of T`、意味関数は `val`。一階論理の伝統では expression が項と論理式の総称、value が項の値と論理式の真理値の総称なので、項と論理式の区別を崩さない。組み込み関数も型と語を揃え、`term_size` を `expr_size`、`term_args` を `expr_children` に改名する（§20.5.3、単位 2）。構文木について「値」と言うときは `val` の値だけを指し、構文木そのものは `expr of T` の対象と呼ぶ。`expr` は予約語なので識別子に使えない。不採用：`eval`（本体で実行できると誤読され、決定不能な論理式の真理値を計算するように読める）、`interp`（インタプリタの連想）、`sem` と `den`（意味が伝わらない）、`meaning` と `denote`（長い、動詞）、`term of T`（項と論理式を混同する）、`syntax of T`（`val` と対にならない）、記号や括弧の記法、`is true`、ghost の接頭辞（構文や特殊規則が増える）。一点付値と更新の許可の名前は val\_at、valid\_at とし、組み込み名であって新しい予約語にはしない（val(t, x, v) の綴りは併存させない）。利用者向けの汎用環境型、置換 API、Binder 専用の走査文は追加しない（2026-10-09 追記）
- [x] 15\. `expr of T` は T について共変とする。T の対象が S の対象でもあるなら、`expr of T` の対象は `expr of S` の対象でもある。`differentiate_expr` が `expr of Real` の木を `expr of object` を取る `check_occurs` に渡せるのはこの規則による。ただし型の拡大は元の変数型、宣言 UID、保持文脈を変更しない。`expr of Nat` を `expr of Real` として扱っても、自然数変数への負の実数の付値は `valid_at` を満たさない。`val_at` の返値型は、更新対象の UID に依存しない型へ必要に応じて広げる（2026-10-07 確定、2026-10-09 改訂）
- [x] 16\. 付値は型に適合する環境の下で定義する。意味論のメタ記法として、環境 ρ の下での意味 `Val(t, ρ)` を `Val(Var(i), ρ) = ρ(i)`、`Val(Node_f(p1, …, pn), ρ) = f(Val(p1, ρ), …)`、`Val(Abs(i, A, B, body), ρ) = g`（D = Val(A, ρ)、K = Val(B, ρ) として dom g = D、rng g ⊆ K、for u in D: g(u) = Val(body, ρ\[i := u\])。D が空なら空関数で、本体の適合環境の存在を要求しない。一般の Binder の意味に公開 val\_at を無条件に用いず、内部 Val で拡張環境を表す。付録 A.2 M4）で定める。`val t` は現在の適合する文脈での意味。一点付値は `val_at[R](t : expr of R, x : expr of object, v : object) -> ghost const R`、許可は ghost 述語 `valid_at(t, x, v)` で、`Adm_Γ(ρ) implies (valid_at(t, x, v) iff Adm_Γ(ρ[x := v]))`。Γ は t と x の共通の保持文脈で、更新後の適合には v が x の元の型・属性を満たすことと、x に依存する他の宣言・適用条件の維持を含む（型ビューに用いた必要前提も含める。更新先の単独の型だけでは足りない。共変性で expr of Nat を expr of Real として渡しても負の実数は付値できない。`y in {x}` の依存があれば y を固定したまま x を自由に変えられるとは限らない）。実引数 v が式なら更新前の環境で解釈し、更新は局所的な数学上の操作で副作用ではない。`val_at` の戻り型 R は x に依存しない型に限り、`Element of {x}` のような型は `expr of Real` や `expr of object` へ広げてから使う（依存型の代入結果を返す機構は加えない。広げても元の制約と保持文脈は消えず、valid\_at は元の制約で検査する）。一般の節点の意味等式を各子の公開 val\_at へ機械的に分解せず、内部証明は更新後の環境 ρ' で `Val(Node_f(p, q), ρ') = f(Val(p, ρ'), Val(q, ρ'))` を使う。開いた構文対象の契約は、保持文脈に適合する任意の環境について `Adm_Γ(ρ) & InputOK & P & returns_alg implies Q` の形で検証・適用し（InputOK は通常の実行値と論理上の変数の対応を表すメタ条件）、一つの環境での前提を全環境へ一般化せず、文脈が変わる地点（Binder の本体へ入るときなど）では環境の対応と前提を改めて示す。失敗・タイムアウトでは事後条件を取り込まない。照合から付値や適合環境の存在は得ない（空領域の Binder の本体も構文として扱えるが要素は取り出せない）。`valid_at` の保持条件からの展開は通常の論理式への等価変換であって、その式の証明ではない。val\_at の使用には現在の適合性に加え更新条件を局所文脈で正当化する。制限付き量化や含意の前件に valid\_at が現れるときはそのガードを保持し、全更新が許可されるとは扱わない（付録 A.4 T5・T6）。`valid_at` は環境や値を生成する述語ではなく、適合環境のない文脈で空虚に成立する契約から具体的な値は取り出せない。不透明な葉の意味は元の式と適合する環境に基づき、葉の自由変数に x が含まれなければ x の付値変更で意味が変わらない。不採用：`expr_subst`（葉への置換と捕獲回避の規則が要る）、任意の object を無条件に付値する旧読み、`val(t, x, v)` の綴り（2026-10-07 確定、2026-10-09 改訂）
- [x] 17\. 組み込みの属性 `variable` を維持する。`x is variable` は x が変数節点であることを表す実行可能な構造上の性質で、契約と本体で使える。内部の定義は `t is variable iff ex κ st t = Var(κ)` とし、R1 で変数を反射した節点が variable である性質を得る（κ はメタ記法。公開の UID 型は追加しない。付録 A.1 S4）。Binder の領域、外部文脈、捕捉・再構成の権利はこの属性だけから得ない。変数節点は宣言の UID と元の型を保持し、現在の数学的な値が等しくても UID は等しくならず、表示名や局所番号だけで同一視しない。Binder パターンは格納済みの UID、入力型、領域、本体の文脈を保って捕捉し、既定の共域は捕捉元ごとの再構成文脈 C に保持する（共域を変数宣言の属性にはしない。同じ κ を別の共域へ再構成して再捕捉すれば別の C）。C を使う補助手続きには精緻化後の暗黙引数として渡し、契約・returns・論理入力にも同じ引数を含める（実行側だけに保存しない）。変数節点の AST 等式だけから C の一致を推論せず、C の対応を持たない AST から共域を推測して再構成しない。捕捉本体は、保持した全入力についての値域証明とその前提の下で、必要な検証済みの型ビュー（例：本体を `expr of Real` として扱う）を利用できるが、一点の意味の確認や前提を失った文脈では使えない。読み取りによる UID の変更や、束縛変数の実値の生成を行わない。C の実装上の表現は比較できるが、元の共域を既定値として保持する意味、完全な論理入力への対応、変更しない再構成の構造的一致（受け入れ条件 V7）を保存する。返却位置・qua の期待型や本体型だけから再推論して元の共域を失う案は同値な実装省略ではなく、採用するなら決定と受け入れ条件の変更として別途扱う（2026-10-07 確定、2026-10-09 改訂）
- [x] 18\. 境界の規則。公開する主張・文脈に構文木の対象（AST の型・構成子・参照）を残さない。algorithm の外では `expr` 型の対象を宣言して構造を操作する経路を設けず、`expr_` 関数と `match` も使えない。expr 型の対象が外に現れるのは `by computation(alg(…))` の引数として R1 で作られるものだけ。数学表記の Binder は関数対象へ変換して主張に用い、`expr` 引数の func を主張へ持ち込む特例は置かない（仮決定 23）。構文に由来する数値や判定結果が公開されること自体は許す。外へ取り込む事実は検証済みの契約と正常返却によって正当化し、構文引数を単にその数学的な値に置き換える変換は認めない（x の木と x + 0 の木で加算判定の結果が異なることは許されるが、それを実数上の同じ関数の値として公開することはできない）。構文木の algorithm の結果の木は ensures の result に代入されて初めて意味を持つ証人で、それ自体を事実として定理に書く経路はない（構文木を取らない algorithm の結果は仮決定 4 により f(c) = v と書ける）（2026-10-07 確定、2026-10-09 改訂）
- [x] 19\. `expr_size` は ghost に留め、正確な自然数値 `expr_size(t) = 1 + Σ expr_size(expr_children(t)[i])`（子の各位置について。同じ子が複数位置にあればそれぞれ数える）と定義する。変数節点と原子的定数は 1、不透明な葉は 1 + |FV(e)|、Binder は 2 + size A + size B + size b。共有を展開した構造上の出現数であって、メモリ上の共有節点数や格納容量ではなく、共有の有無で値が変わらない（t(n+1) = f(t(n), t(n)) は DAG なら n+1 節点だがサイズは 2^(n+1) − 1。展開木を実体化せずに計算できる）。各子は親より真に小さい（`c in rng expr_children(t) implies expr_size(c) < expr_size(t)`）。再帰する木の algorithm の停止性はこの一般の補題で検証でき、MVM が検証済みの `decreasing expr_size(t)` を再帰ごとに再評価することは要求しない。具体的な値が要る処理では正確なサイズを計算・取得し、不変な節点のキャッシュや DAG の動的計画法を使える（各節点と各子の位置を一度ずつ処理。巨大な整数の演算費用は別）。固定幅のオーバーフロー・飽和・丸めを値として採用せず、資源上限に達したら未完了として扱う。ghost は仕様位置の区分であって計算費用の不在や構造情報の秘匿を意味しない（`expr_children` が実行可能なので利用者がサイズを通常の algorithm で計算することも妨げない）。契約にも書け、`ensures expr_size(result) < expr_size(t)` を呼び出し側の `decreasing` に使う algorithm どうしの合成ができ、その契約を by で外へ出すには、付録 A.4 T7 により正確な数の比較へ変換し、expr 型の項が残らないことを確認する（仮決定 20）。ここで必要なのは木の構造と UID が具体的に定まることであり、FV が空であることや自由変数の数学的な値は必要ない。当初の「`expr_children` は通常の関数とし、契約に書かないことを付録 D の推奨規則とする」は仮決定 20・21 で置き換えた（2026-10-07 確定、2026-10-08、2026-10-09 に改訂）
- [x] 20\. 取り込みの検査。`by computation` は、引数を型検査して型・スコープを保った木を作り、呼び出しを MVM で評価して正常返却を確認し、requires を正当化して契約に引数と結果を代入し（その契約自身の帰結で同じ呼び出しの requires や正常返却を循環的に正当化しない）、認定した意味保存の変換（構造的な val の変換と、対象・方向・停止性を定めた Binder の変換。任意の means を無制限に展開しない）で通常の式に直し、公開する全ての事実を検査してから文脈に加える。検査対象は ensures の実例だけでなく、公開候補の f(c) = r、量化変数の型、返値や列・集合の中に残る構文対象。object へ型を広げたり列に包んだりした AST も対象。構文木が残る公開候補は全体を拒否し、残った項、候補の種類、候補内の位置（契約の節、等式の辺、型・列・集合の内部など）を報告する。ensures の実例は全体を変換・検査し、木が残る部分だけを黙って捨てない。T8 は公開候補の選別規則であって、ensures の部分式を消す書き換えではない。`returns_f(c, r)` や構文木を含む内部の f(c) = r は契約を適用する証拠として保持するだけでよく、公開する値の等式は通常の数学的な呼び出し・引数・返値で表せるものに限る。`expr_size` の計算結果のように構文に由来する数が公開されることは許される。この検査は公開形式の制約であって、内部の生成公理や MVM の意味的な誤りを防ぐ仕組みではない。変換は付録 A.4 の規則集合・処理順・適用条件に従い、構造再帰、保持条件の有限な展開、捕獲回避の代入、通常の数学的な Binder の関数構成を区別する。有限の AST であることだけを、全ての定義展開や任意順の書き換えが停止する理由にしない。未認定の形や条件の不足、変換・検査の資源切れは診断して候補を取り込まない。既に確認した内部の正常返却証拠を、その後の取り込み失敗により否定することもない。不採用：契約に含まれる関数名（expr\_size、expr\_children）を見て引用を止める案、expr\_size と expr\_children を契約に書けないとする案（algorithm 内で構文的な契約を合成できなくなる）、本体での利用位置を制御条件に限る案（2026-10-08、2026-10-09 改訂）
- [x] 21\. `expr_args` は `expr_children` に改名する（葉に対して自由変数の列を返すことも含めて「子」が実態に合う）。`expr_children(t)` は非 ghost の実行可能な純粋操作として `FinSequence of expr of object` を返し、本体と仕様位置で使える。要素型を expr of object とすることは元の型やスコープを失わせない。列挙規則：変数節点 `Var(κ)` と変数に依存しない原子的定数は空列、通常の適用 `Node_f(a1, …, an)` は意味上の暗黙引数を明示した仮引数順で `[a1, …, an]`（重複を保持）、不透明な葉は精緻化後の初出順で UID により重複を除いた自由変数の節点列、Binder `Abs(κ, A, B, b)` は `[A, B, Var(κ), b]` の順（重複を保持。A と B は外側の文脈、Var(κ) と b は κ の宣言で拡張した文脈を保持する）。宣言型・属性・宣言文脈への参照、変数から所属 Binder への参照、環境と値、証拠・親参照・位置情報、キャッシュは走査辺に含めない。構造上の子の関係は有限かつ非循環で、DAG としての共有は許す。取得・照合は格納済みの UID をそのまま使い、取得のたびに開き直したり改名したりしない（`t1 = t2 implies expr_children(t1) = expr_children(t2)`）。新しい Binder の生成と捕獲回避の ID は完全な論理入力から決定的に選び、実行時刻や呼び出しごとのグローバルカウンタだけによらない。`for c in expr_children(t)` は通常の列走査で、任意の ghost 有限列を非 ghost の変数へ束縛する規則は置かない（`<val p>` のような一要素列を経由して非計算的な意味を実行値へ持ち出せてしまう）。子の列の保持・長さの計算・蓄積を特別に禁じる規則も置かない：守るべきは外部境界（仮決定 18、20）であって本体の規律ではなく、algorithm が AST を調べて計算する以上、構文に由来する数値や判定結果が外に出ることは許容する。非変数節点について `FV(t) ⊆ ⋃ FV(c)` を保証する（Binder では宣言の子 Var(κ) を含むため等号にならず、この過剰な出現情報を許容する）。不採用：`expr_children` を ghost にして ghost 有限列を for でだけ実行値に取り出す案（上の穴。専用の走査文に限っても構文情報の保持を防ぐ効果はなく、特例が増えるだけ）、match に記号を問わないパターン（node(cs)、\_(cs)、f(cs)）を加える案（名前が直感に反し可読性を損なう）、`children of t` のような専用の句（2026-10-08、2026-10-09 改訂）
- [x] 22\. 契約が関数について述べる algorithm は、関数対象を値とする木を外部入口に取る。最初の微分例 `differentiate` は入力と正常返却結果をともに `expr of Function of REAL, REAL` とし、`for u being Real holds (val f) is_differentiable_in u & diff(val f, u) = (val result).u` を保証する（一般の `PartFunc of REAL, REAL` に対して結果の定義域内だけを述べる契約にはしない）。利用者は数学表記の Binder `u |-> e` で関数を書く（仮決定 23）。func と pred は一階論理の記号で対象ではないので節点のラベルにしかならず、`expr of` の型には現れない。理由：`sin x * x` を `expr of Real` として反射すると木が関数であることを型が忘れ、関数を `for g st for u holds g.u = val_at(t, x, u)` で受け直す必要が生じて、微分変数の出所、入れ子の量化子、g や t が全域でない場合の扱いが問題になった。内部の `differentiate_expr(t : expr of Real, x : expr of object) -> expr of Real` は全実数への更新適合性 `for u being Real holds valid_at(t, x, u)` を前提とし、正常返却時に結果側の `valid_at(result, x, u)`、`(z |-> val_at(t, x, z)) is_differentiable_in u`、`diff((z |-> val_at(t, x, z)), u) = val_at(result, x, u)` とスコープの保存を保証する（D1〜D3）。外部の型だけで Binder の問題が消えるわけではなく、内部で本体を処理する手続きには付値・スコープの規則（仮決定 16、17）が要る。一般の部分関数を扱う別の algorithm は妨げないが、必要な定義域の条件をこの例の型から省略・一般化しない。点ごとの木だけを外部入口にして利用者に関数の存在を毎回書かせる形は推奨しない。付録 D の推奨規則に加える（2026-10-08、2026-10-09 改訂）
- [x] 23\. `x |-> e` を正式の Binder とする。表面文法は `Identifier |-> Term` の一種類で、項の中で最も弱く右結合（`x |-> y |-> e` は `x |-> (y |-> e)`、`x |-> e | X` は `x |-> (e | X)`、`(x |-> e).a` は適用、`(x |-> e) | X` は制限、`(x |-> e) qua T` は全体への型注記）。通常の演算のオペランドにするときは括弧で囲む。頭部に専用の型・領域句（`x being T |->`、`x in A |->`、末尾の where）を追加せず、変数の型は既存の `let`・`reserve` の宣言型と期待型で定め、曖昧なら Binder 全体への `qua` 注記で関数型を示す。通常の数学位置では本体だけで有効な新しい束縛 UID を導入し、外側の同名変数の値・個別の仮定・旧 UID を引き継がない（`let x be Real; assume x = 0; set f = x |-> x` は恒等関数で、零関数に簡約しない）。定義域は、期待される認定済み `Function of A, B`／`PartFunc of A, B`（反射位置では `expr of` のその値の型）の入力集合 A を優先し、なければ宣言型 T の認定された正確な集合担体 Carrier(T)、どちらでも決まらなければ拒否して注記を求める。期待型から A を選んだときは A の全要素が T を満たすことを検査し、一点の所属や `x in A` の仮定では代用しない。整合しなければ拒否し、別の定義域へ切り替えるフォールバックはしない。`assume` や等式から定義域を自動抽出しない。`PartFunc of A, B` の期待型で新規に構成する場合も実際の定義域を A 全体に固定する。型名から集合名を作る規則は置かず（`Real` と `REAL` の対応は基礎ライブラリの定義による認定）、`object` や `set` 全体を無条件に定義域にせず、`Element of A` の空集合時の意味は保つ。空の A を許し（結果は空関数）、要素を捏造しない。共域 B は外側で定まる集合とし、期待型、なければ本体型の認定済み担体から決め、本体の像集合の自動構成はしない。各 u in A について本体の型・適用条件と E(u) in B を検査し（値域証明に使った前提は保存する）、`(x |-> e) | X` は定義域全体で構成してから制限する。通常の論理位置では基礎ライブラリの関数構成の scheme と関数の等しさにより関数対象（dom g = A、rng g ⊆ B、g(u) = E(u)）へ変換し、反射では同じ意味を持つ Binder 節点 `Abs(κ, A, B, b)` にする。両経路の意味の一致を接続条件とする。構文の認識は言語、`Function`・`PartFunc`・適用・dom・rng は基礎ライブラリの定義で、同じ綴りの利用者定義に特別な意味を与えず、基礎定義や証明が使えなければ公理を生成せず型検査エラーにする。型の合う入れ子を許す。解釈モードは「新規導入（数学項・R1）」「パターン捕捉（`case x |-> body`）」「再構成（頭部が捕捉済み変数）」の三つで、構築位置を確定してから頭部を解決し、型推論の失敗で切り替えない。再構成は保存された入力型・領域・UID を保持し（領域を再推論・拡大しない）、既定の共域は捕捉元ごとの再構成文脈 C から得て、`(x |-> e') qua expr of Function of A, B'` で変更できる（Function の期待型には保存領域と A0 の集合としての等しさ、PartFunc には包含を証明する。実際の定義域 X ⊆ REAL の Binder を `expr of PartFunc of REAL, REAL` で受け取っても定義域は X のまま）。R2 は自由 UID を保存して埋め込み、綴りによる置換はせず、内側の束縛点の衝突だけを所属出現ごと α 改名する（保持文脈を対応付けられない木の挿入はスコープエラー）。他の Binder（総和、積分、極限）は共通の関数抽象と通常の演算への展開（例：`sum i in I of e` → `Sum(I, (i |-> e))`）から検討し、任意のユーザー定義 Binder の採用は未決（Wiedijk の提案を参考）。旧 MML の定数有限列 `n |-> a`（FINSEQ\_2:def 2）は `(Seg n) --> a` へ移行し、期待型で読み分ける互換規則は置かない（用語・記号の全面的な見直しの中で扱う）。C の実装上の表現を変更する場合も、既定共域の保持、完全な論理入力との対応、変更しない再構成の構造的一致を保存する（仮決定 17、受け入れ条件 V7）。不採用：`|->` を expr 引数の通常の func として定義する案、val で意味を定めた expr 引数の func だけを許す案、カーネルの項構成子としての lambda、変換算法 abstract、専用の `being`／`in` 頭部、`assume` からの領域抽出、像集合の自動推論、期待型による旧記法の読み分け（2026-10-08、2026-10-09 改訂・追加決定）

## 仕様化時に確認する最小例

以下は単位 2〜4 の受け入れ条件で、2026-10-09 の追加決定で確定した 31 項目（文法・型・領域 G1〜G7、子・尺度・同一性 S1〜S6、付値・捕獲回避・実行と公開 V1〜V18）です。実行して確認した結果ではなく、受理を要求する例は必要な基礎定義と証明（`Real` と `REAL` の対応など）が利用可能な文脈を、拒否を要求する例は追加の仮定で型義務を空虚にしていない通常の文脈を前提とします。「期待型」は型検査器への入力を表すメタ記法です。

| 番号 | 例・条件 | 必要な結果 |
| --- | --- | --- |
| G1 | `let x be Real; assume x = 0; set f = x \|-> x;`（期待型なし） | 受理。dom f = REAL、f.u = u。内外の x の ID は異なり、f.1 = 0 へ簡約しない |
| G2 | `let A be Subset of REAL; let x be Real; set f = (x \|-> x + 1) qua PartFunc of A, REAL;` | 受理。dom f = A（A = {} なら f = {}）。部分関数型でもより小さい定義域を選ばない |
| G3 | `let x be Real; set A = {0, REAL}; assume x in A; set f = (x \|-> x + 1) qua Function of A, REAL;` | 拒否。REAL は Real の対象でなく、A の全要素が宣言型を満たす義務を x in A で代用しない |
| G4 | `let A be set; let x be object; assume x in A; set f = x \|-> 0;` | 拒否（定義域を確定できない）。object 全体の担体を仮定せず、仮定から A を選ばない |
| G5 | `let x be Element of {}; set f = x \|-> x;` | 拒否（型名の引数 {} を担体にしない）。`(x \|-> x) qua Function of {}, {}` なら受理し f = {} |
| G6 | `let x, y be Real; set f = x \|-> ((y \|-> x + y).0);` | 受理。各定義域は REAL、f.u = u。本体の x は外側 Binder の新 ID、y は内側の新 ID |
| G7 | `let x be Real; let y be object; set f = x \|-> y;` | 拒否（出力集合を確定できず、{y} を自動生成しない）。`qua Function of REAL, {y}` なら受理 |
| S1 | `Abs(κ, REAL, REAL, Var(κ))` | 子は \[REAL, REAL, Var(κ), Var(κ)\]、長さ 4、サイズ 5。重複を保持し、本体をスコープ情報なしの閉じた木として扱わない |
| S2 | 不透明な葉 `the Element of {x}`、精緻化後の出現順が x, y, x の葉 | 前者の子は \[Var(x)\]、サイズ 2。後者は \[Var(x), Var(y)\]。選択対象の x を型前提として捨てない |
| S3 | `Var(y)`、y の宣言型が `Element of A(x)` | 子は空列、FV は {y}。x だけが異なり y が同じで適合する二環境で意味は同じ。適合しない更新をこの性質だけで許さない |
| S4 | 仮引数順 (S, a) で表面は f(a) の適用（S は暗黙引数） | 子は \[S, a\]。S を隠して \[a\] とする実装は拒否。同じ実引数が複数位置なら重複を保持 |
| S5 | t0 = a、t(n+1) = f(t(n), t(n)) を共有あり・なしで表す | 同じ構造は等しく、サイズは 2^(n+1) − 1。固定幅を超えても丸めない。展開木を実体化しない |
| S6 | 同じ Binder への反復した取得・照合、同じ完全入力での生成、別文脈で同じ局所番号の変数 | UID と子の等式が保たれ、生成の返値は単値。局所番号だけで同一視しない。取得時の改名やキャッシュ更新で等式が変わる実装は拒否 |
| V1 | 自然数変数を expr of Real または expr of object として渡し、負の実数へ更新 | 許可しない（元の型条件で valid\_at を満たさない） |
| V2 | t の型が Element of {x} で、更新前の精密型を val\_at の返値型にする | 拒否。x に依存しない上位型を使っても更新適合性は別途検査 |
| V3 | 親の返値型は固定、子の精密型が x に依存 | 内部の更新後環境での合成は許し、各子の公開 val\_at を型検査なしで生成しない |
| V4 | y ∈ {x} を要する文脈で y を固定し x だけを変更 | x 単独の型だけでは許可しない。更新後の依存条件を要求 |
| V5 | 空領域の Binder を照合し Var(κ) と本体を列に保持 | 構文操作として受理。要素・適合環境・具体的な val を照合だけから得ない。Binder 全体が空関数であることは外側で示せる |
| V6 | 同名の新規 Binder の下へ外側変数を含む既存 AST を R2 で挿入 | 外側の自由 UID を保存し、表示名の一致で捕獲しない |
| V7 | 捕捉した Binder を変更せず再構成、または同じ捕捉 ID の Binder を内側に重ねる | 往復は構造的に同じ木。衝突は束縛点と所属出現を改名し、自由 UID は変えない |
| V8 | 捕捉 Binder の共域を期待型で変更、より広い入力集合の PartFunc で再構成 | 入力型・実際の領域・UID を保ち、値域と期待型適合を証明。領域を拡大しない。既定共域 REAL の C と {0} の C′ は別入力 |
| V9 | 同じ局所番号を持つ別文脈の変数を挿入 | 宣言元の対応がなければ同一視しない。自由変数を供給できない挿入はスコープエラー |
| V10 | check\_occurs が false | 全適合環境で許可された更新に対し意味不変。構造帰納では非出現も示す。true から依存を結論しない |
| V11 | 本体 y \* x を y = 0 の一点で確認 | それだけでは y \|-> y \* x の x に関する不変性を証明済みとしない |
| V12 | `if val p`、`return val t`、ghost 一要素列 \<val p> の非 ghost 走査 | 意味値を実行値へ持ち出す経路として拒否 |
| V13 | 閉じた自然数式を実行可能なリテラル取得・加算で評価 | 正常返却と result = val t を証明すれば通常の自然数を使える。有限性や Nat 型だけで万能な評価器を認めない |
| V14 | 実数上の Binder を differentiate に渡す | 入力・結果の全実数での適合性と微分の契約を証明。狭い領域の部分関数を無条件に渡さない |
| V15 | 実行値の引数を変える、前提が別環境で成り立たないまま契約を再利用 | 同じ構文入力だけを理由に転用しない。完全な実行入力との対応と前提を確認 |
| V16 | object への型拡大や列・集合に包んだ AST を契約や値の等式として公開 | 公開候補全体の検査で拒否。AST が消えた数値や主張は正常返却と契約を根拠に公開できる |
| V17 | 未対応の実行形、fail、実行資源切れ、正常返却の証拠がない処理。または正常返却後の変換・検査の未完了 | 実行が正常返却を確認できなければ、その試行から新しい returns の証拠や事後条件を得たと扱わない。取り込み段階だけが未完了なら既に得た内部証拠を否定せず、当該公開候補を取り込まない。いずれも正常返却の不存在を結論せず、尺度に不正確な整数を代用しない |
| V18 | 本体の元の型が object で、全入力で値が REAL に属する証明を保持 | 必要な文脈・前提の下で expr of Real として扱い微分ヘルパーへ接続できる。型ビューは木・UID・元の型を変えず、一点の確認や前提を失った文脈では使えない |

### 付録 A・B の受け入れ条件（K1〜K12）

以下は今回のレビュー補正を検査する例です。G・S・V の既存の受け入れ条件を置き換えません。この表と上の受け入れ条件の S 番号は、付録 A.1 の規則番号とは別の系列です。

| 番号 | 条件・入力 | 要求する結果と参照 |
| --- | --- | --- |
| K1 | 保存された精緻化済みの元の式が異なる二つの不透明な葉。両方とも FV と子列は空 | 子列の一致だけで AST を同一視しない。リテラルの値と不透明な葉の元の式を含めて比較する（付録 A.1 S1） |
| K2 | 通常の項 e の純反射、量化式を含む命題 P の純反射、p : expr of Real を R2 で埋め込んだ sin p | 項は val t = e、命題は val t = true iff P。混合構築の意味は sin (val p) であり、構文対象 p を実数引数として残さない（M2・M3、T2・T3） |
| K3 | 空領域の Binder、および本体の元の型が束縛 UID に依存する Binder | 全体の意味を内部 Val と拡張環境で定める。空領域では空関数を得るが、本体の現在の適合環境や公開 val\_at の形成条件を捏造しない（M4、T4、B3） |
| K4 | 内外に同じ表示名の変数がある Binder と、不透明な葉を含む本体への一点付値 | 元の宣言 UID と変換先変数を対応付け、自由出現だけを捕獲回避で置換する。旧束縛の同名変数を誤って更新しない（M6、T4・T5） |
| K5 | 自然数変数への負の実数の更新。または `for v being object st valid_at(t, x, v) holds ...` | valid\_at を述語として書くことは許す。条件を展開しただけで更新可能とはせず、val\_at の具体的使用に必要な条件を要求する。制限付き量化ではガードを保持してその下で変換し、全ての v に対する valid\_at の証明は要求しない（T5・T6、B3） |
| K6 | 型ビューまたは依存する宣言・適用条件の前提を失った文脈で意味規則を適用 | 保持された必要条件を展開対象に含め、不足を診断する。AST の型拡大や Binder の照合で前提が消えたと扱わない（M5・M8、T6、B3・B8） |
| K7 | 命題の木 p、q について val p = val q を公開する候補 | 認定済みの標準 boolean の二値性と真理条件を用いられる場合、対応する命題の同値へ変換する。認定・変換規則が適用できない形は AST を残して公開せず診断する（T1、B8） |
| K8 | 数学的な値が与えられていないが宣言 UID と構造が具体的な Var(x) | expr\_size は 1、子列の長さは 0 と計算できる。FV が空であることを要求せず、構造比較は共有やキャッシュに依存しない。再構成も、必要な C と形成条件を伴う同じ完全入力についてこの性質を満たす（S1〜S3、T7） |
| K9 | 部分正当性が未検証の ensures、または正常返却の保証が未検証の terminating 宣言 | 宣言だけから E2・E4 を発行しない。E3 を別の独立公理として追加して実行との不一致を隠さない。C を含む完全な入力と、必要な環境・InputOK・前提を揃える（E1〜E5、B7） |
| K10 | MVM の実行、正確なサイズ計算、意味変換、証明義務の検証、残存検査のいずれかで資源上限に達する | 実際の段階と未完了の操作を報告する。不正確な値や未検証の公開事実を生成しない。取り込みだけが未完了なら、既に得た内部の正常返却証拠を否定しない（E5、A.4 の処理手順、B6・B8） |
| K11 | 内部の returns または構文引数を含む値の等式と、構文を含む ensures の実例 | T8 により内部の適用証拠を公開候補から区別する。ensures の一部分を消して残りだけを公開する操作には使わない。object へ広げた AST、型・列・集合の内部も候補全体で検査する（T8、B8） |
| K12 | terminating または ghost の文脈の fail、terminating の otherwise 欠如、直接の子でない木への再帰 | 前二者は実行前に拒否する。最後は他の型・契約・正常返却の義務を満たした上で、選んだ尺度の厳密な減少を証明できれば許し、子でないことだけでは拒否しない（B5・B7） |

## 付録 A　構造・意味・実行の規則と、取り込み変換の信頼上の位置付け

この付録は、決定の一覧と合意済みの表を基に、仕様書（§20.5.3、architecture 15 の信頼基盤の一覧、§22.4 の取り込み）へ移す規則と、その採用条件を整理したものです。定義、型規則、表現不変条件、導出補題、検証済みの契約、個別の実行証拠を区別します。既存の決定の単なる転載ではなく、レビューを受けて、構造的等式に含める情報、各規則の前提、取り込み変換の処理順と停止の根拠を具体化しています。出典欄の決定番号は、その規則の基礎となった決定を示します。

記法は決定 16 のメタ記法（`Var(κ)`、`Node_f(…)`、`Opaque(e)`、`Abs(κ, A, B, b)`、`Val(t, ρ)`、`Adm_Γ(ρ)`、`FV(t)`）を使い、利用者向けの新しい構文や API ではありません。C は、捕捉に付随する再構成文脈を表します。カーネルへ渡す論理式は、必要に応じて記号ごと・節点ごとの一階のスキーマとして生成します。メタ記法で述べた型規則や表現不変条件を、そのまま独立した公理として無条件に追加する趣旨ではありません。

導出補題は、記載した定義と前提から正当化します。検証済みの契約は本体の証明義務を通過してから採用し、個別の実行証拠は、対応する完全な入力で正常返却したことを確認する経路から得ます。生成公理という形でカーネルへ渡す場合も、これらの採用条件は省略できません。表現の実装、規則の生成、検証結果と本体の対応、MVM からの事実の取り込みは、それぞれの接続の正しさを要します。A.4 の公開前検査は、この接続全体の正しさを代わりに保証するものではありません。

### A.1　構造の定義・型規則・不変条件と導出補題

以下は、型とスコープが適合する構文対象についての規則です。A・B・b は AST の部分構造であり、その数学的な意味と区別します。

| 番号 | 区分と規則 | 採用条件・信頼上の位置付け | 出典 |
| --- | --- | --- | --- |
| S1 | **構造的同一性の定義。** `t1 = t2` iff 構造を定める全ての不変ペイロードが一致する。比較対象には、節点の種類、解決済みの記号、数値等のリテラルの値、不透明な葉が保存する精緻化済みの元の式、順序付き部分構造、変数の宣言 UID を含む。帰結として `t1 = t2 implies expr_children(t1) = expr_children(t2)` および `t1 = t2 implies expr_size(t1) = expr_size(t2)` を得る | 子列や FV が一致するだけでは同じ木としない。例えば、異なる閉じた式を保存した二つの不透明な葉は、子列がともに空でも同じ木ではない。共有状態、キャッシュ、格納先、診断用のソース位置は等式の一部にしない。UID を変更した α 改名や意味の一致を自動的に構造的等式とせず、`Var(κ)` の等式から付随する C の一致を導かない。同一性判定の実装はこの定義に対応させる | 13、17、21 |
| S2 | **子列挙の定義と実行操作との接続。** `expr_children(Var(κ)) = <>`。変数に依存しない原子的定数 c では `expr_children(c) = <>`。`expr_children(Node_f(a1, …, an)) = <a1, …, an>`。`expr_children(Opaque(e)) = <Var(κ1), …, Var(κk)>`。`expr_children(Abs(κ, A, B, b)) = <A, B, Var(κ), b>` | 通常適用は意味上の暗黙引数を明示した仮引数順で、重複を保持する。不透明な葉は精緻化済みの式の FV を初出順に並べ、UID で重複を除く。Binder は記載順で重複を保持する。格納済みの UID と子の元の型・保持文脈を返し、取得時に改名しない。A・B は外側、`Var(κ)`・b は κ の宣言を加えた文脈を持つ。実行可能な列挙処理をこの定義に対応させる | 8、21、23 |
| S3 | **尺度の定義と減少の導出補題。** 正確な自然数として `expr_size(t) = 1 + Σ_i expr_size(expr_children(t).i)` と定める。S7 の有限性の下でサイズは正となり、`c in rng expr_children(t) implies expr_size(c) < expr_size(t)` を導く | 和は子の各位置について取り、共有を展開した出現数を数える。共有・キャッシュで値を変えず、固定幅のオーバーフローや丸めを値として使わない。具体的な数値を求める処理の正確さと、停止性の検証で使う一般の減少補題を区別する。後者は再帰のたびの MVM による実サイズ計算を要求しない | 19、21 |
| S4 | **属性の定義。** `t is variable iff ex κ st t = Var(κ)`。R1 で変数 x を反射した木は `Var(κ_x)` であり variable である | iff を採用する。κ は宣言元が定まる UID を表すメタ変数であり、この属性は変数節点であることだけを判定する。Binder の領域、再構成文脈 C、その変数を閉じる権利はこの属性だけから得ない | 17 |
| S5 | **型規則。** 型付けされた通常適用について `Node_f(…) : expr of R`（R は引数を適用した f の戻り型、pred と論理結合子は boolean）。形成条件を満たす Binder は `Abs(κ, A, B, b) : expr of Function of A₀, B₀`。`t : expr of T` かつ T の対象が S の対象なら `t : expr of S` | A₀・B₀ は AST の A・B が表す外側の数学的な集合項であり、適合環境 ρ の下ではそれぞれ `Val(A,ρ)`・`Val(B,ρ)` と解釈される。A・B は当該 κ に依存せず、全ての `u in A₀` について入力の宣言型が成立し、本体の型・適用条件が成立し、その値が B₀ に属することを証明する。必要な基礎ライブラリの認定を使い、任意の未検証の Abs にこの型を与えない。空領域では要素を仮定しない。型の拡大は元の型・UID・保持文脈を変更しない | 15、23 |
| S6 | **FV の内部定義と包含の導出補題。** `FV(Var(κ)) = {κ}`、変数に依存しない原子的定数の FV は空集合、`FV(Node_f(a1, …, an)) = FV(a1) ∪ … ∪ FV(an)`、`FV(Opaque(e))` は記録した集合、`FV(Abs(κ, A, B, b)) = FV(A) ∪ FV(B) ∪ (FV(b) \\ {κ})`。これらと S2 から、変数節点以外で `FV(t) ⊆ ⋃ { FV(c) : c in rng expr_children(t) }` を導く | 意味に必要な暗黙引数、量化範囲、選択の対象を FV に含める。変数の宣言型・属性・文脈の依存全体へ FV を拡張せず、それらの適合性は Adm に保持する。不透明な葉の FV 記録処理をこの定義に対応させる。包含は一般に等号ではなく、Binder の宣言の子による過剰な出現情報を許す | 8、16、21 |
| S7 | **表現・形成の不変条件。** 構造上の子の関係は有限かつ非循環とする。DAG による部分木の共有を許す | 構築・変換・読み込みによってこの条件を保つ。宣言文脈、変数を所有する Binder、環境、証明情報、親、キャッシュへの参照を構造上の子の辺に加えない。S2 の意味に必要な値引数は除外しない。この条件は構造帰納と S3 の根拠であり、意味の規則の無矛盾性、任意の定義展開、A.4 の変換処理全体の停止性を単独で保証しない | 20、21 |

### A.2　意味の定義・反射との対応・契約の解釈

以下の `val t` は、t の保持文脈に適合する現在の論理環境 ρ での `Val(t,ρ)` を表します。意味の式を用いる際には、当該文脈、型、必要な適用条件を保持します。開いた本体を構文対象として得たことから、その本体の適合環境の存在は推論しません。

通常の数学的な**項**と**論理式**を区別します。元の項 e の意味を `⟦e⟧ρ`、元の論理式 P の真理条件を `⟦P⟧ρ` と書きます。これらは説明用のメタ記法です。項についての等式と、論理式についての `val t = true iff P` を使い分け、FOL の論理式をそのまま等号の右辺に置きません。

| 番号 | 区分と規則 | 採用条件・意味保存の根拠 | 出典 |
| --- | --- | --- | --- |
| M1 | **解決済みの記号に対する意味の定義。** func f について `Val(Node_f(p1,…,pn),ρ) = f(Val(p1,ρ),…,Val(pn,ρ))`。pred P について `Val(Node_P(p1,…,pn),ρ) = true iff P(Val(p1,ρ),…,Val(pn,ρ))`。論理結合子について、例えば `Val(p & q,ρ) = true iff (Val(p,ρ) = true & Val(q,ρ) = true)`、`Val(not p,ρ) = true iff not (Val(p,ρ) = true)`。or・implies・iff も対応する真理条件で定める | 引数は意味上の暗黙引数を含めて完全化され、解決済みの定義と対応するものとする。引数・結果の型と適用条件を満たす環境で使用する。認定された原子的定数は零引数の記号として扱える。これを任意の利用者定義の無制限な展開規則にはしない | 13、16、21 |
| M2 | **純粋な R1 と元の数学式の対応、および R2 の保存。** 通常の項 e そのものを R1 で反射した t について `Val(t,ρ) = ⟦e⟧ρ`。論理式 P については `Val(t,ρ) = true iff ⟦P⟧ρ`。変数の場合は `Val(Var(κ),ρ) = ρ(κ)`。R2 が既存の AST q を埋め込むときは、その箇所に q 自身を置き、意味も `Val(q,ρ)` を引き継ぐ | 元式の記号・UID・型付け文脈と、反射した木との対応を保持する。R2 を含む構築式では M1〜M4 を組み合わせて意味を求め、元の構築コードを無条件に数学式と同一視しない。例えば q が AST なら、q を埋め込んだ sin 節点の意味は `sin(Val(q,ρ))` であり、AST q 自身を数学的な実数とする `sin q` ではない | 13、16、23 |
| M3 | **不透明な葉と元式の対応。** 元の項 e には `Val(Opaque(e),ρ) = ⟦e⟧ρ`、元の論理式 P には `Val(Opaque(P),ρ) = true iff ⟦P⟧ρ` | 精緻化した元式、意味上の自由 UID、型付けに必要な文脈を保持する。量化子を含む論理式も、その通常の論理式としての意味を使用する。量化の内部を通常の AST 子として分解しないことと、その真理条件を無視することを混同しない。この対応は認められた反射対象について用い、無制限の自己反射や自己評価を新たに正当化するものではない | 8、13、16 |
| M4 | **Binder の意味の定義と二つの表現の対応。** `Val(Abs(κ,A,B,b),ρ) = gρ`。`Aρ = Val(A,ρ)`、`Bρ = Val(B,ρ)` とすると、`dom gρ = Aρ`、`rng gρ ⊆ Bρ`、各 `u ∈ Aρ` について `gρ.u = Val(b,ρ[κ↦u])` | 外側の適合環境で Aρ・Bρ は集合であり、各入力 u に対して拡張環境の適合性と本体の値域条件を示す。存在・一意性は認定された基礎ライブラリの関数構成と関数の等しさに接続する。通常の数学位置の Binder と、同じ束縛を反射した節点は同じ関数対象を表す。ここでは内部の `Val` を用い、外側に κ の現在値がないまま公開 `val_at` を呼ばない | 16、23 |
| M5 | **一点更新の許可の定義。** x が UID i の変数節点で、t と x の共通の保持文脈 Γ が整合するとき、`Adm_Γ(ρ) implies (valid_atρ(t,x,v) iff Adm_Γ(ρ[i↦v]))` | v は更新前の環境で解釈した値とする。更新後には、x の元の型・属性、依存する宣言・適用条件、使用する型ビューの必要な前提を満たさなければならない。共変性や型ビューによって既存の条件を弱めない。適合する現在環境が前提であり、この述語から環境や値の存在を生成しない。通常の論理式への展開には、後述の有限な条件記録とその対応の証明を要する | 15、16、17 |
| M6 | **一点更新後の意味の定義とその帰結。** `Adm_Γ(ρ)`、`valid_atρ(t,x,v)` および公開の戻り型 R が i に依存しないという条件の下で、`val_at[R](t,x,v) = Val(t,ρ[i↦v])` | 変数について、対象の UID が i なら値は v、異なる UID j なら値は `ρ(j)`。通常適用は更新後の環境で M1、Binder は同じ環境で M4、不透明な葉は同じ環境で M3 を用いる。子の精密型が i に依存する場合も、内部の `Val` による合成は使える。各子の公開 `val_at` へ展開する場合は、その子ごとに適用条件と固定した戻り型を確認する。通常の式への代入として表す場合は、下記の UID に関する代入補題を用いる | 16 |
| M7 | **非出現からの意味不変性の接続補題。** `κ not in FV(t)`、`Adm_Γ(ρ1)`、`Adm_Γ(ρ2)`、および ρ1・ρ2 が κ 以外で一致することから、`Val(t,ρ1) = Val(t,ρ2)` を得る | 値の式の自由変数と、型の前提の依存を区別する。意味に必要な暗黙引数、不透明な葉の依存、Binder の束縛について M1〜M4 との対応を示す。更新後の適合性を省略する根拠にはせず、意味不変性から構文上の非出現を逆向きに推論しない | 8、16、21 |
| M8 | **開いた構文対象を含む契約の解釈。** 各環境 ρ について `Adm_Γ(ρ) & InputOK(args,ρ) & P_ρ(args) & returns_alg(args,result) implies Q_ρ(args,result)` | args は構文対象、通常の実行値、必要な保持文脈、使用する C 等を含む完全な論理入力。InputOK は、通常の実行値と論理上の値との必要な対応を表す。前提を示した一つの環境から全環境へ一般化せず、適用する環境ごとに対応と前提を保持する。空の本体文脈でこの式が空虚に成立しても、無条件の Q や本体の値は得られない。本体の検証を経て契約を採用することは E2 による | 16、17 |

**M4 の空領域と依存する本体型。** Aρ が空なら gρ は空関数であり、本体の適合環境の存在を要求しません。本体の元の型が κ に依存する場合も、拡張環境でその本体を解釈して値域条件を示します。公開の `val_at` を別途用いるときは、必要な固定型への検証済みの型ビューと、その前提を保持します。M4 全体を、これらの条件を省いた公開 `val_at` の式へ置き換えません。

**M5 の有限な条件記録。** Γ の数学的な適合条件を通常の論理式へ出力するには、型検査・精緻化で必要とした条件の有限な記録 `KΓ = <K1,…,Kn>` と、`Adm_Γ(ρ) iff (K1(ρ) & … & Kn(ρ))` という対応を確認できるものとします。条件は元の宣言型・属性、領域、意味上の暗黙引数の適合、必要な適用条件、および型ビューに残る前提を含みます。全ての周辺の仮定や定義を無制限に集めるものではありません。条件中の量化式は有限な式として保持し、集合の全要素を列挙しません。

どの表層の型・属性からどの条件を記録するかという総表、および対象とする MML の定義 ID は、単位 2 の型検査・認定表と接続して固定します。この付録はその未知の部分を推測して補いません。A.4 の T6 は、必要な条件の記録と対応が利用可能な実例について適用します。展開できないことを、その更新が数学的に不可能であることと同一視しません。

**M6 と通常の式への代入との接続。** 次は、対象となる通常の項・論理式の代入補題を表すメタ記法です。e の変数は宣言 UID で識別し、項の場合には、両環境の適合性の下で

```text
⟦e[i := w]⟧ρ = ⟦e⟧(ρ[i ↦ ⟦w⟧ρ])
```

が成り立ちます。論理式 P については、対応する二つの真理条件の同値を用います。代入は自由出現だけに作用し、束縛点・型・領域の必要な参照を区別して捕獲を避けます。w は更新前の環境で解釈され、他の UID の値は固定します。表示名の一致による一括置換や、代入した w の内部への同じ代入の再適用は行いません。

この補題を M2・M3 の元式との対応へ接続すれば、不透明な葉に i が出現する場合も更新後の意味を述べられます。非出現の場合の不変性だけを、その代わりにはしません。適用対象の元式には反射範囲・型・スコープの条件があり、任意の自己評価を含む式についてこの対応を無条件に追加しません。A.4 の変換は、内部の UID 対応を使ってこの代入を実現し、利用者向けの置換 API を追加しません。

### A.3　algorithm の契約・全域化・実行証拠

以下の E1〜E5 に共通して、入力 x（具体的な呼び出しでは c）は、構文対象、通常の実行値、必要な保持文脈、使用する再構成文脈 C 等を含む、精緻化後の完全な論理入力を表します。暗黙引数は実行側と契約・返却関係・生成する関数の側で一致させます。`Var(κ)` の AST 等式だけを理由に C を置換しません。

`returns_f(x,r)` は、この入力から有限の実行によって r を正常に返すことを表します。関数記号の等式から、逆向きに正常返却を推論する規則は置きません。構文対象を含む契約では、下表の P・Q を無条件の式として使わず、M8 に従い、各環境 ρ における `Adm_Γ(ρ)`、`InputOK(x,ρ)`、`P_ρ(x)` を保持します。

| 番号 | 区分と規則 | 採用条件・信頼上の位置付け | 出典 |
| --- | --- | --- | --- |
| E1 | **正常返却のグラフと全域関数記号の接続。** `returns_f(x,r) implies f(x) = r` | 正常返却の結果の型と、完全な入力に対する実行の単値性を正当化した上で関数記号を導入する。宣言された戻り型を全入力で保証するなら、未返却入力でもその型の値が存在することを型形成規則または証明で正当化する。依存する戻り型のこの条件が P の下でしか成立しない場合、P の外までその型を断言する全域関数として導入しない。正常返却しない入力での値は未規定とし、等式だけから returns や ensures を得ない。C や保持文脈を完全な論理入力から落として全域化してはならない | 4、17、21 |
| E2 | **検証済みの部分正当性の契約。** 通常の形は `P(x) & returns_f(x,r) implies Q(x,r)`。構文対象を含む場合は、各 ρ について `Adm_Γ(ρ) & InputOK(x,ρ) & P_ρ(x) & returns_f(x,r) implies Q_ρ(x,r)` | ensures の宣言だけで発行せず、本体の検証を通して採用する。部分的な再帰は正常返却した有限の実行に関する帰納で扱えるが、呼び出し前提と正常に戻った経路を要する。M8 の環境条件や InputOK を落とさず、一つの環境で示した前提を全環境に一般化しない | 4、16 |
| E3 | **実行との接続条件、および E1 からの帰結。** `returns_f(x,r1) & returns_f(x,r2) implies r1 = r2` | 実行意味論では、同じ完全な入力から正常に返る結果が一致することを要求し、E1 の導入を正当化する条件とする。E1 導入後、この論理式自体は等式の推移性から導けるため、独立した追加公理にはしない。宣言 UID の生成、構造操作、再構成も完全な入力に対して決定的でなければならない | 4、17、21 |
| E4 | **検証済みの正常返却保証と昇格した契約。** 通常の形は `P(x) implies ex r st returns_f(x,r)`。この保証と E1・E2 から `P(x) implies Q(x,f(x))` を得て、定義ラベルで引用できる | terminating の宣言だけで発行せず、前提を満たす全入力で正常返却することを検証する。直接・暗黙の fail、部分的な子呼び出し、反復範囲の評価も対象とする。構文対象を含む場合、存在保証も昇格した契約も、各 ρ における `Adm_Γ(ρ) & InputOK(x,ρ) & P_ρ(x)` を前提とする。ラベルの引用でこの条件や公開境界を外してはならない | 1、4、6、16 |
| E5 | **個別の実行証拠。** `by computation` が完全な入力 c での正常返却 v を確認したとき、内部に `returns_f(c,v)` を得る | これは任意の呼び出しの返却を仮定する公理ではなく、対象の本体・入力・結果の対応を確認する経路から採用する事実である。MVM の報告を信頼する経路と replay 等の詳細は単位 5 で定め、その正しさは公開形式の検査だけでは保証されない。requires 等は契約を利用する前提として正当化する。fail・タイムアウト・資源切れは、この呼び出しの新しい正常返却証拠を生成しない。ただしタイムアウト等から数学的な正常返却の不存在を結論しない。公開には E1・E2 と A.4 を通し、内部の returns をそのまま公開しない | 2、4、11、20 |

E4 の正常返却保証は、任意の有限な資源上限の下で MVM が必ず計算を終えるという保証ではありません。資源上限に達した実行から不正確な値や未確認の返却事実を補うことはせず、別に正当化された論理上の保証と、その実行で得られた証拠を区別します。

### A.4　取り込み候補の選別、認定した意味変換、公開前検査

`by computation(alg(c))` は、完全な入力 c での正常返却 r を確認し、契約の利用に必要な前提を正当化してから、ensures に引数と結果を代入した実例を扱います。同じ呼び出しの契約の帰結を使って、その呼び出し自身の前提や正常返却を循環的に正当化しません。構文対象を含む場合は、M8 の現在環境の適合性、InputOK、契約の前提を保持します。

本節は、**適用条件付きの有限な変換手順**を定めます。全ての表層構文・型・数学式に対する変換完備性や、任意の規則適用順での停止を主張しません。規則は T1〜T7 と本節の boolean の正規化に限定し、T8 を候補の選別に使います。論理式の部分式を辿る処理、UID を保つ変数対応、必要な α 改名は、この手順の内部操作です。任意の `means` の展開、未知の記号を同じ綴りの定義で補うこと、AST 自身をその数学的な値に置き換える操作は含めません。

#### 有限な変換対象と、内部の UID 対応

各取り込み候補について、次を処理開始時に固定します。

1. 引数・結果を代入した有限な候補式と、その正当化に用いる保持条件・局所ガード。
2. その式が参照する具体的な AST と、S2 の子を辿って得られる有限で非循環な構造。
3. T2・T3 に使える、元式と AST の対応の記録。元式は記号・束縛を解決した有限な項または論理式とし、後述する終端的な数学式として出力できることを確認する。
4. T6 に使える、M5 の有限な条件記録と適合性との対応、および型ビュー等に必要な証明の前提。
5. 各記号の意味、標準 boolean、Binder の関数構成について使用する認定済みの定義・規則と、必要な有限の証明。

元式の来歴、宣言文脈、C、証明の参照を、全て AST の子として再帰的に展開しません。有限な記録を越える定義展開や、未分類の式・未認定の定義 ID が必要になった場合は、この手順で推測して補わず、対応する診断を出して公開を拒否します。

内部の対応写像 σ は、AST の自由 UID を、出力先のスコープで型付けされた通常の数学的な項へ対応付けます。初期の対応は現在の宣言との対応から得て、Binder に入るときは新しい束縛変数への対応を追加します。これは説明・実装用のメタ記法であり、公開する環境型ではありません。

σ の各値は、AST や未変換の意味関数を含まない有限な項として固定します。複数の UID の対応は同時に適用し、一つの対応に埋め込んだ項の内部を、別の対応によってもう一度置換しません。特定の UID だけを更新するときも、他の UID の対応は変更しません。自由変数を出力先の宣言へ対応付けられない場合、同じ綴りの変数や仮の値で補ってはなりません。

以下で「終端的な数学式」とは、名前と束縛が解決され、型・適用条件が正当化され、出力する部分に AST の値・参照・構成子・内部 UID・未変換の `val` 系や `expr_` 操作を含まない通常の項または論理式をいいます。型拡大や列・集合への包装によって残る AST も、この条件の検査対象です。通常の数学的な関数記号の定義はここでは展開しません。通常の Binder 表記が残る場合は、認定された関数構成への一方向の変換を行い、反射して AST に戻しません。

T2・T3 の元式と、T6 の条件記録は、この終端的な形へ、UID の対応と認定された通常の Binder の変換だけで出力できる実例を対象とします。元式や条件の中から、新たな反射・評価要求を再導入して同じ規則群へ戻す処理は行いません。この条件を満たす記録がない場合は、B8 で未対応の元式・条件を報告します。この限定は取り込み変換の適用範囲であり、M3 や M5 の数学的な意味の定義を、展開できた場合だけに変更するものではありません。

#### T1〜T8

| 番号 | 規則・処理 | 根拠 | 適用条件 |
| --- | --- | --- | --- |
| T1 | **解決済みの通常節点。** func の節点の意味は、各子を同じ UID 対応の下で数学的な項へ翻訳し、その結果を f に渡す。pred と論理結合子の節点は、`val t = true` に対応する真理条件へ翻訳する。認定された定数・リテラルは対応する通常の項へ出力する。boolean の比較は下表の正規化を併用する | M1 | 出力可能な通常の数学記号との対応、引数の型・適用条件、暗黙引数、結果の型を確認する。反射・意味評価・MVM の実行を再び要求する内部記号を、通常の f として通さない。子の意味は内部の `Val` で合成し、依存する子に公開 `val_at` を無条件に生成しない。子が必要な項の形へ翻訳できなければ、論理式を項の位置へ押し込まず拒否する |
| T2 | **純粋な R1 の元式、または変数の対応。** 純粋な R1 の項 e に対応する `val t` は、σ を適用した元の項へ翻訳する。元式が命題 P の場合は、`val t = true` の真理条件を σ 適用後の P にする。変数節点 `Var(κ)` の意味は `σ(κ)` とする | M2 | 元式との対応が、現在の木・UID・文脈に対して有効で、終端的な数学式として出力できること。R2 を含む構築コード全体を、そのまま元の数学式として使わない。元の綴りの変数が出力先で見えることだけを、UID の対応の代わりにしない |
| T3 | **不透明な葉。** 項の葉の意味は、σ を適用した元の項へ翻訳する。論理式の葉については、意味が true であることを、σ 適用後の元の論理式にする | M3、M6 の代入補題 | 元式・自由 UID・型付け条件との対応が有効で、終端的な数学式として出力できること。項と論理式を混同せず、出力した元式を再反射したり、その定義を再帰的に開いたりしない |
| T4 | **Binder。** `Abs(κ,A,B,b)` の意味を、外側の σ で翻訳した領域 A'・共域 B' と、新しい束縛変数 z の下で翻訳した本体 E による通常の関数対象へ変換する。具体的な手順は下記による | M4 | 外側の適合性、全ての領域要素についての本体の型・適用条件・値域条件を保持する。公開 `val_at` に必要な現在の κ の値を要求せず、内部の環境拡張と UID 対応で本体を扱う |
| T5 | **公開の一点付値。** `val_at(t,x,v)` は、まず v を更新前の対応・環境で終端的な項 v' にし、x の UID i の対応だけを v' に更新して、t の意味を翻訳する。他の UID の対応は固定する | M6、UID に関する代入補題 | 現在の `Adm_Γ`、変数節点とスコープの適合性、戻り型の i からの独立性を確認する。更新後の適合性は、局所ガードまたは既に正当化した前提によって示す。表示名への後付けの一括代入や、他の対応項の内部の再置換はしない |
| T6 | **更新許可の論理式への展開。** `valid_at(t,x,v)` を、記録済みの Γ の条件を、一点更新後の対応で解釈した有限な連言へ変換する | M5 | 現在の `Adm_Γ` と、有限な条件記録との対応が必要。元の変数型・属性、依存宣言・適用条件、使用する型ビューの必要な前提を落とさない。展開は同値な式への変換であり、その連言の証明ではない |
| T7 | **具体的な構造の計算。** `expr_size(t)`、`len expr_children(t)` を正確な数へ、AST の構造的等式 `t = t'` を判定結果へ変換する | S1〜S3、S7 | t・t' の構造と必要な宣言情報が、実行の入力・結果として具体化していること。`FV(t) = {}` は要求せず、`Var(κ)` のような開いた木も対象にできる。未知の AST を推測せず、意味値の一致を構造的一致に使わない。資源不足では不正確な値を代用しない |
| T8 | **公開候補の選別。** `returns_alg(c,r)` と、構文対象を含む内部の `alg(c) = r` は正当化の証拠として保持し、直接の公開候補にはしない。通常の数学的な呼び出し・引数・返値で表せる値の等式と、ensures の実例全体を公開検査へ渡す | E1〜E5、決定20 | 完全な論理入力を基準とし、C 等の暗黙引数を落として別の関数の等式を作らない。これは個別の証拠の選別であって、ensures の内部から AST を含む部分だけを捨てる規則ではない |

#### T4：Binder の本体を開いて翻訳する手順

1. A と B は外側の σ で翻訳し、A' と B' を得ます。Binder 自身の κ に依存しないという形成条件を保持します。
2. 出力先の有効な宣言、σ の値、本体の自由変数を捕獲しない新しい束縛変数 z を導入します。元の表示名を使える場合でも、新しい宣言との対応を明示し、名前の一致を根拠にしません。
3. 本体は `σ[κ↦z]` の下で翻訳します。内部意味は `Val(b,ρ[κ↦u])` に対応し、各 `u ∈ A'` の下で型・適用条件・値域条件を検証します。依存する元の型や検証済み型ビューの前提も、この文脈で扱います。
4. E が通常の項として得られたとき、概念上の表面表記は次です。これは意味変換の結果の模式表示であって、新しい Binder 頭部の文法ではありません。

```text
(z |-> E) qua Function of A', B'
```

5. この通常の Binder を、認定された基礎ライブラリの構成と一意性により関数対象へ変換します。A' が空でも許され、結果は空関数です。κ の実値や本体の適合環境の存在を、変換の前提として外側に生成しません。全入力の義務が空虚であることは、未対応の構文を型検査なしに通す理由にはしません。

元の AST の UID は変更しません。ここで新しくするのは出力先の束縛宣言であり、元の κ との対応を翻訳中に保持します。必要な意味保存の証明は M4 と捕獲回避の代入の対応によります。対象の基礎定義や構成の証明を利用できない場合は、未証明の関数を補わず診断します。

#### T1 に付随する boolean 値の正規化

標準 boolean の定義が認定され、`true <> false` と、各 boolean 値が true または false であることを使用できる場合に限り、次の規則を使います。B・C は型検査済みの boolean 値の項、`Truth(B)` は「B が true である」真理条件への内部翻訳を表すメタ記法であり、公開する関数ではありません。

| 元の位置 | 出力する論理式 |
| --- | --- |
| `B = true` | `Truth(B)` |
| `B = false` | `not Truth(B)` |
| `B <> true` | `not Truth(B)` |
| `B <> false` | `Truth(B)` |
| `B = C` | `Truth(B) iff Truth(C)` |
| `B <> C` | `not (Truth(B) iff Truth(C))` |

この表は、少なくとも一方に除去対象の boolean の `val` または `val_at` がある比較に適用します。`Truth(val p)` は T1〜T4 による p の真理条件、`Truth(val_at(p,x,v))` は T5 の更新を反映した真理条件にします。B が既に終端的な普通の boolean 項 b なら、`Truth(B)` は単に `b = true` として終了し、その比較へ同じ表を再適用しません。

これにより、例えば `val p = val q` は、p と q の真理条件の同値にできます。論理式そのものを、項を要求する場所へ挿入する規則ではありません。任意の関数の boolean 引数位置や Binder の返値位置で通常の boolean 項が必要な場合は、その項としての翻訳が得られることを別途必要とします。この規則だけで対応しない形を補わず、AST が残れば B8 で拒否します。必要な二値性の認定がない場合も、未証明の変換を行いません。

#### T5・T6 のガードと代入値

T6 は、更新後の条件を記述する有限な論理式を生成するだけです。例えば自然数変数への負の実数の更新は、条件を展開しただけでは許可されません。T5 の使用には、その更新条件の証明が必要です。

一方、`for v being object st valid_at(t,x,v) holds Q` のような契約では、制限部分を展開した条件を保持して Q の内部を変換します。全ての v についてその条件が無条件に成り立つと証明する必要はありません。含意等の局所ガードも同様に、適用する部分式で使用できる条件として保持します。ガードを外して帰結だけを公開する規則は置きません。

v の翻訳結果は、更新前の環境で解釈する項として固定します。例えば v 自身に x の現在値が現れても、更新後の x に読み替えません。また、他の UID に対応付けた項が同じ表示名を含むことを理由に、その項まで変更しません。直接 `e[x:=v]` と略記できる場合も、この UID・スコープと一点更新の条件を満たす捕獲回避の代入を意味します。

#### 処理段階と停止の根拠

実装は、少なくとも次の段階と依存方向を保ちます。規則を任意の順序で反復する汎用書き換え系としては定めません。

1. **証拠と候補の確定。** 完全な入力・正常返却・前提を確認し、有限な実例を作り、T8 で公開候補と内部証拠を分けます。ensures は実例全体を一つの候補として扱います。
2. **有限な記録の確認。** 元式、Γ の条件記録、UID の対応、型ビューの前提、使用する認定規則を確認します。T2・T3・T6 の出力が終端的な数学式として得られるという適用条件を満たさない場合、別の反射・評価要求を自動的に追加せず診断します。
3. **候補式の走査。** 有限な論理式を辿り、束縛と局所ガードを保持します。T5 の v は更新前に先に翻訳します。boolean の比較には前記の規則を一度適用し、各真理条件を翻訳します。T6 は記録済み条件への一回の展開と対応の適用、T7 は具体的な構造の計算として処理します。
4. **AST の意味の翻訳。** 元式との終端的な対応が使える場合は T2、該当する不透明な葉には T3 を使って終了します。それ以外の認定済みの通常節点には T1、Binder には T4、変数には UID 対応を使います。T5 は同じ t を異なる対応でこの翻訳へ渡す入口であり、t の翻訳から同じ T5 の入口へ戻しません。未分類の節点は拒否します。
5. **通常の Binder の変換と公開前検査。** 得られた通常の Binder を認定済みの関数構成へ一方向に変換し、候補全体を検査します。出力した数学式を再反射したり、通常の定義を無制限に開いたりして前の段階へ戻しません。

停止の根拠は、処理ごとに区別します。

- T1・T4 の AST に対する再帰は、S2 の構造上の子へ進むため S3・S7 によって整礎です。T2・T3 は有限な元式への終端的な出力であり、取り出した式を再び AST の意味変換へ戻さないという適用条件を使います。
- T5 の引数 v は、有限な入力式の真の部分式として先に処理します。その後は t に対する上記の構造再帰と、有限な UID 対応の適用です。代入した項の内部へ同じ代入を再適用しません。式が複製される場合も、有限な変換結果の複製であり、新しい評価要求を作りません。
- T6 は、固定した有限な条件列の有限な式への対応の適用です。宣言・型・証明の参照を辿って新しい条件を際限なく発見する処理ではありません。この根拠を AST の S7 から導きません。
- boolean の正規化は、対象となる比較から各真理条件の有限な翻訳へ進みます。終端的な boolean 項の比較には戻しません。通常の Binder の変換も、有限な Binder 出現と認定済みの構成・証明に対する一方向の処理です。
- T7 は具体的な有限構造の正確な計算です。共有やキャッシュを使ってよいですが、資源切れを近似値や未証明の事実で埋めません。

これらは、対象範囲・記録・認定済みの変換を満たす実例についての根拠です。数学的な側条件一般を自動で証明できることや、無制限な証明探索の停止を主張しません。必要な証明を確認できない場合や資源上限に達した場合は、その段階に対応する診断を出し、未確認の結果を公開しません。S7 だけで全ての元式の展開・文脈展開・自己評価が停止すると説明しません。

#### 公開候補全体の検査と、残る実装上の接続

変換後の各公開候補について、式全体、量化変数の型、関数の完全な引数と返値、型注記、列・集合などの内部まで検査します。AST の型・値・構成子・参照、内部 UID や C、未変換の `val`・`val_at`・`valid_at`・`expr_children`・`expr_size` 等が残る場合は、候補全体を拒否し、残った項とその位置を報告します。単なる綴りではなく解決済みの識別情報と値・型の情報で検査し、object への型拡大で回避できないようにします。

ensures の実例のうち変換できた部分だけを黙って残すことはしません。T8 で内部証拠を公開候補にしない処理と、契約の部分を捨てる処理を区別します。元式が終端的な数学式であることの事前確認を通していても、公開候補全体の最終検査は省略しません。

全表層構文の分類、型・属性から記録する条件の総表、標準 boolean と Binder の基礎ライブラリについて認定する具体的な定義 ID、および既存仕様の診断番号との対応は、対象実装へ接続して固定する必要があります。この付録は、それらを推測した万能の変換器を定義しません。A.2 の意味との対応、A.3 の実行証拠、各変換の証明、公開境界の検査は別々に正当化し、公開形式の検査だけで内部公理や MVM 全体の正しさが保証されるとは扱いません。

## 付録 B　診断の一覧

本文で「拒否する」「エラーにする」とした場面を、原因と検出する段階ごとにまとめたものです。B1〜B8 はこの付録内の分類 ID であり、実装のエラーコードではありません。分類名と既存のエラーコードとの対応は、仕様書側の分類・番号表に合わせて確定します。本付録では新しいエラーコード番号を割り当てず、本文が既に参照している E0503 は該当する既存診断との対応確認に用います。「報告」は診断に含める情報を示します。

「条件を示せない」は、必要な証明が得られていないことを表し、その条件が偽であるという証明を表しません。型検査から生成される証明義務の失敗、許されない位置の構文、MVM の実行失敗、検証・意味変換の未完了を区別します。失敗・未完了を理由に、正常返却や事後条件の新しい証拠を生成してはなりません。

| 区分 | 状況 | 検出する段階 | 報告 | 出典 |
| --- | --- | --- | --- | --- |
| B1 文法 | `by alg(引数)` の旧形、`exhaustive`・`exhaustiveness_proof`、Binder 頭部の専用の `being`／`in`、`children of t` など、廃止・不採用の構文を使用する | 構文解析 | 使用位置、受理しない構文、対応する現在の書き方。Binder の型・領域には既存の宣言・期待型・全体への `qua` 注記を案内する | 2、10、21、23 |
| B1 文法 | algorithm の名前だけを `by` の引用に使う、部分的な algorithm に昇格用の定義ラベルを付ける、許可された `thus`／`hence` の直下以外で `thesis` を引数に使う | 構文・位置の検査、名前解決 | 使用位置と宣言元。計算の適用と定義ラベルの引用を区別し、該当する利用条件を示す | 1〜3 |
| B2 型（Binder） | 新規 Binder の定義域を確定できない。期待型の入力集合も、宣言型の認定された正確な集合担体もない（G4、G5） | 型検査 | Binder の位置、変数とその宣言型、不足する期待型。Binder 全体への `qua` で関数型を示すよう求める。`assume` から領域を自動抽出しない | 23 |
| B2 型（Binder） | 出力集合を確定できない（G7） | 型検査 | Binder の位置、本体の型、不足する期待型。像集合を自動構成して補わない | 23 |
| B2 型（Binder） | 期待型の入力集合の全要素が入力の宣言型を満たすことを示せない（G3）、または全ての入力で本体の型・適用条件と出力集合への所属を示せない | 型検査が生成する証明義務の検証 | 期待型、宣言型、選んだ入力集合・出力集合、不足する条件。一点の所属を全入力の保証にせず、別の定義域へフォールバックしない | 23 |
| B2 型（Binder） | 必要な基礎ライブラリの認定がない。型の担体、関数型、関数構成の scheme または対応する証明が認定表にない | 型検査・基礎定義との接続 | 解決された定義と、必要な認定・証明。同じ綴りの別定義や未証明の生成公理で補わない | 23 |
| B2 型（Binder） | 数学項・R1 の新規導入、パターン捕捉、AST の再構成という解釈モードを確定できない。再構成の頭部に、再構成文脈 C との対応を持たない AST 変数を使う | 型検査・名前解決 | 構築位置、頭部の名前と宣言元、欠けている型または C との対応。型推論の失敗で別のモードへ切り替えない | 17、23 |
| B2 型（Binder） | 再構成の期待型が保存領域と適合しない。Function の期待型が要求する領域の等しさ、PartFunc の期待型が要求する領域の包含、または新しい共域への値域条件を示せない（V8） | 型検査が生成する証明義務の検証 | 保存領域、期待型、既定・指定共域、不足する証明。保存領域を拡大・再推論しない | 17、23 |
| B3 型（付値） | `val_at` の戻り型が更新対象の変数に依存する（V2） | 型検査 | 対象の式、戻り型、依存する変数と宣言元。既存の型規則で正当化できる場合に限り、依存しない上位型の候補を示す | 16 |
| B3 型（付値） | `val_at`・`valid_at` の x が変数節点であることを示せない、または t と x の保持文脈が整合しない | 型検査・必要な証明義務の検証 | 対象の式、t と x の型・保持文脈、対応しない宣言。`variable` 属性だけで領域や適合環境を補わない | 16、17 |
| B3 型（付値） | `val`・`valid_at`・`val_at` の使用に必要な現在の論理環境の適合性を正当化できない。特に、空領域の Binder を照合したことだけから、領域の要素・適合環境・本体の具体的な意味値を得ようとする（V5） | 型検査が生成する証明義務の検証 | 使用位置、対象の式、必要な保持文脈と不足する適合条件。構文の取得と具体的な意味値の取得を区別する | 16、17 |
| B3 型（付値） | `val_at(t, x, v)` の使用に必要な `valid_at(t, x, v)` を正当化できない。元の型・属性に合わない更新、他の宣言や適用条件の依存を壊す更新を含む（V1、V4） | 型検査が生成する証明義務の検証 | 対象の式、更新対象と更新値、不足する元の型・属性・依存条件、および該当する宣言元。共変性や型ビューで元の制約を消さない | 15、16 |
| B3 型（付値） | 検証済みの型ビューに必要な値域証明・前提を正当化できない（V18）、または一つの環境で成立した前提を別の環境・文脈へ根拠なく持ち出す（V15） | 検証 | 型ビューまたは呼び出しの位置、必要な前提、環境・文脈の対応の不足。一点の値の確認を全入力の証明として扱わない | 16、17 |
| B4 スコープ | R2 で埋め込む木の保持文脈を埋め込み先に対応付けられない、または必要な自由変数を供給できない（V9） | 型検査・スコープ検査 | 挿入位置、対応を欠く変数の名前と宣言位置、必要に応じて UID。表示名や局所番号の一致だけで対応付けない | 23 |
| B4 スコープ | `by` の引数から、その位置のスコープにない主張の量化変数を参照する | 型検査・名前解決 | 参照位置、変数と束縛位置。必要に応じて proof 内で `let`・`assume` を使い、その位置で利用できる変数により呼び出しを記述するよう案内する | 12 |
| B5 ghost と実行の境界 | `val`・`val_at`・`valid_at`・`expr_size` を実行値の計算に使用する。`if val p`、通常の `return val t`、ghost の列の非 ghost の `for` 走査などを含む（V12） | 型検査・ghost の利用位置の検査 | 使用位置、ghost の式、実行値へ持ち出そうとした経路。既存の ghost 違反として扱い、契約・assert・不変条件・decreasing・証明での正当な使用とは区別する | 13、19、21 |
| B5 ghost と実行の境界 | ghost の文脈で `fail` を使用する | 構文の利用位置の静的検査 | `fail` の位置と、それを許さない ghost の文脈。MVM の実行失敗として扱わない | 部分的な algorithm と fail の表 |
| B5 ghost と実行の境界 | algorithm の外で `expr` 型の対象を宣言する、`expr_` 関数や `match` によって構造を操作する | 構文解析・型検査 | 使用位置と境界を越える構文対象・操作。`by computation` の引数として許される R1 の反射と混同しない | 18 |
| B6 実行 | 部分的な algorithm が明示・暗黙の `fail` に達する、または `otherwise` のない `match` で未対応の形に達する | MVM | 対象の式、入力、利用者向けの失敗メッセージと呼び出しの経路。微分なら微分変数も示す。この失敗から正常返却・事後条件の新しい証拠を生成しない（V17） | 4、6 |
| B6 実行 | MVM の `steps` 上限を超える、または実行中の資源が不足する | MVM | 呼び出しの経路、未完了の操作、実際に達した資源上限の種類と上限。正確な数値の計算を近似値で置き換えて続行しない | 11、19 |
| B7 検証条件 | `terminating` の algorithm 内で明示的な `fail` を使用する、または `otherwise` を省いた `match` を使用する | 構文の利用位置の静的検査 | 禁止された文の位置と、外側の `terminating` 宣言。正常返却の証明を試すだけでは禁止を解除しない。既存の E0503 等との対応は仕様書の番号表で確認する | 部分的な algorithm と fail の表、6、10 |
| B7 検証条件 | 正常返却する経路で ensures を示せず、本体の部分正当性の検証が完了していない | 検証 | 返却位置、事後条件と不足する証明義務。失敗経路に事後条件の証明を要求せず、未検証の契約を E2 として採用しない | 4、A.3 E2 |
| B7 検証条件 | 全域関数記号の導入に必要な結果の型・単値性を正当化できない。全入力で宣言する戻り型の値の存在が、未返却入力では保証されない場合を含む | 型形成・検証 | 宣言、完全な入力、戻り型と不足する導入条件。requires の下でのみ成立する型の条件を、その外へ無条件に一般化しない | 4、A.3 E1・E3 |
| B7 検証条件 | `terminating` の algorithm が、前提を満たす全入力で正常返却することを示せない。部分的な子呼び出しの前提と正常返却、非空性、反復範囲の評価を含む | 検証 | 対象の経路・呼び出しと不足する義務。単なる停止、値の等式、正常返却を区別する | 4、6 |
| B7 検証条件 | 宣言された `decreasing` について再帰先の尺度が現在より小さいことを示せない、またはループ不変条件が保てない | 検証 | 呼び出しまたはループの位置、尺度・不変条件、不足する証明。`expr_children(t)` の子であることを唯一の減少証明にしない | 19、21 |
| B8 取り込み | `by computation` の呼び出しの requires を正当化できない、正常返却の証拠がない、または契約自身の帰結で同じ呼び出しの前提・正常返却を循環的に正当化する | 取り込みの前提・証拠の検査 | 呼び出し、不足する前提または正常返却の証拠、循環している依存。値の等式だけを正常返却の証拠にしない | 2、4、20 |
| B8 取り込み | A.4 の認定意味変換の適用条件を正当化できない。T4 で構成する Binder の型・領域・捕獲回避の条件、T5 に必要な更新適合性などを含む | 取り込み時の意味変換・型検査 | 公開候補内の対象位置、適用しようとした規則と不足する条件。変換を適用する局所的な論理文脈の仮定・量化条件も区別して示す | 16、20、23、A.4 |
| B8 取り込み | 対象の構文・論理式の形に認定された変換がない、変換が循環する、または認定された規則による変換が完了しない | 取り込み時の意味変換 | 未変換の部分、規則を適用できない理由、循環または未完了の操作。任意の定義展開や構文引数の意味への素の置換で補わない | 18、20、A.4 |
| B8 取り込み（検証との共通資源診断） | 宣言・契約の証明義務の検証、具体的な構造量の計算、意味変換、公開候補の残存検査の途中で資源上限へ達する。尺度の計算を含む | 実際に未完了となった段階（検証・具体値の計算・取り込み時の意味変換・公開候補の残存検査） | 対象の宣言・式または公開候補、処理段階、未完了の操作、資源上限の種類と上限。MVM の実行失敗と区別し、近似値や未確認の変換結果を使わない | 19、20、A.4 |
| B8 取り込み | 認定意味変換後の公開候補に、構文対象の型・構成子・UID・参照、AST を含む意味関数・構造操作が残る。object への型拡大や列・集合の内部を含む（V16） | 公開候補全体の検査 | 残った項、公開候補の種類、候補内の位置。ensures の節、値の等式の辺、量化変数の型、返値・列・集合の内部を含めて特定する。候補全体を拒否し、残った部分だけを黙って捨てない | 18、20 |
| B8 取り込み | 命題木の意味や変換途中の boolean 表現が、認定された規則で通常の論理式または通常の boolean 値の項へ直せずに残る | 取り込み時の意味変換・公開候補全体の検査 | 残った部分、その使用位置、必要な変換の形。`val p = true` 以外の位置を根拠なく同じ規則で処理せず、候補全体を拒否する | 13、20、A.4 |

`valid_at(t, x, v)` を適切な文脈で述語として書くこと、その否定を述べること、偽になり得る更新を契約の条件で除外すること自体はエラーではありません。B3 が拒否するのは、使用自体に必要な構文・文脈の条件が満たされない場合、または `val_at` 等を使うために必要な証明が不足する場合です。空領域の Binder の子を構文として取得・保持する操作や、適合環境を仮定した全称的な契約を、具体的な値の取り出しと混同して拒否しません。

型・付値・スコープの診断では、変数名だけでなく宣言元と必要な条件を示します。UID は同名の別宣言を区別する補助情報として使用できますが、UID の数値だけを報告して利用者に原因の推測を求めません。

B8 の検査対象は ensures の実例だけではなく、外部へ公開する全ての候補です。意味変換が失敗した場合は、未変換の AST が残ったという最終状態だけでなく、最初に適用できなかった認定規則または未完了の操作を報告します。通常の数学的な boolean 値や論理式、構文に由来していても AST を残さず正当化された数値・判定結果を、その由来だけを理由に拒否しません。

<!-- End of retained source quotation. -->
