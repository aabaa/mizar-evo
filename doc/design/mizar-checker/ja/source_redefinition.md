# ソース再定義の target

正本は [English](../en/source_redefinition.md)。

## API と責任

`source_redefinition` は intrinsic な attribute・二項 predicate・単項 equals-functor の
宣言時 target を認証する。
`SourceOrdinaryRootHandoff` は Task278 が所有する宣言 root の前提入力であり、
`SourceRedefinitionHandoff` の三つの Task279 source-kind 規則がこれを消費する。
両者は元ソース graph・resolver identity・binding context・型正規化を利用する
immutable な sealed value とし、別の構文木を追加しない。
`SourceRedefinitionInput::from_source` は順序付き target、明示 label token または省略、
bound-target diagnostic anchor、notation relation、Pending coherence 入力を保持する。
`build_with_input` は fresh なソース導出との完全一致を検証して既存 handoff に封印する。

## Root identity と Pending request

候補 root は、source kind・notation・arity が一致する可視の先行 ordinary 宣言すべてとする。
順序付き intrinsic guard を Set parameter が厳密に狭化することを要求する。
attribute subject の semantic guard と resolver の明示 pattern arity は区別する。
functor の戻り型適合性は独立に検査し、使用箇所の選択結果・戻り型・宣言順で target を選ばない。
実際の symbol と source site で root 関係を保持する。
明示的な非修飾ローカル宣言 label は適格な ordinary root を指定する。
省略時は適格 root が一意であることを要求し、不正な label では省略時の推論に戻らない。
coherence request は実 owner に属する Pending のままとし、active refinement・使用可能な fact・
checked proof・等式・受理済み同値を生成しない。

## Relation と選択の transport

ordinary definition block の末尾には、裸の attribute synonym または同じ順序の formal loci を
使う二項 predicate antonym を一つ置ける。両 notation pattern と先行 ordinary root を認証し、
alias の使用は保留する。末尾の Object reserve と反射的等式 theorem は、単項 ordinary functor
の実 application を一つ含められる。ordinary-root handoff が collection・expansion・viability・
specificity の出力を再定義 binding より先に封印する。再定義 intake は bound root・ソース signature・
Pending refinement candidate を root の正規化 domain で認証する。Pending 入力から既存 selector に
渡す active-refinement set は空とし、公開結果は ordinary root の型を保持して evidence と inserted view
を持たない。comparison や priority edge を生成せず、theorem proof は未検査のままとする。
封印済み入力と内包する selection 出力は、同じ AST slot を通る。
最終 assembly は、再定義ソースにこの封印のない公開結果が渡されると拒否する。

## Transaction と境界

共通 handoff 一つを TypedAst に原子的に install し、ResolvedTypedAst で replay する。
型 ID は所有元の正規化 universe を保持し、宣言だけのソースに overload application や selected root を作らない。
import・template・nominal/attributed・修飾 target・refinement chain は初期境界の対象外とする。
既存の marked-Set/registration consumer は独立した前提を維持する。
実 type-elaboration Case が install 済み root・target・body/type・Pending request の対応を観測する。
各 definition block は parameter に続く ordinary 宣言または再定義をちょうど一つ含み、
ordinary 宣言には任意で alias が続く。宣言 label は互いに異なるものとする。
