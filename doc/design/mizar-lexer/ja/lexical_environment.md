# Module: lexical_environment

> Canonical language: English. English canonical version: [../en/lexical_environment.md](../en/lexical_environment.md).

## Purpose

このモジュールは、トークンの曖昧性解消(disambiguation)が参照する、アクティブ字句環境
(active lexical environment)を構築します。

現在の実装は、組み込みの予約テーブルと、インポートプレリュード(import prelude)で
指定されたモジュールがエクスポートする字句シンボルのサマリーとを結合します。
また、現在のモジュールの字句宣言のために、ソース位置に依存するレイヤーも提供します。
ローカル宣言は、その宣言項目が完了した後にだけ、インポート由来の環境を拡張します。

## Public API

実装済み API:

```rust
pub type ReservedWordTable = &'static [&'static str];
pub type ReservedSymbolTable = &'static [&'static str];

pub struct ActiveLexicalEnvironment {
    pub reserved_words: ReservedWordTable,
    pub reserved_symbols: ReservedSymbolTable,
    pub user_symbols: UserSymbolIndex,
    pub fingerprint: LexicalEnvironmentFingerprint,
}

pub struct ModuleLexicalSummary {
    pub module_id: ModuleId,
    pub exported_symbols: Vec<ExportedSymbolShape>,
    pub fingerprint: LexicalSummaryFingerprint,
}

pub struct ResolvedImport {
    pub module_id: ModuleId,
}

pub fn build_lexical_environment(
    imports: &[ResolvedImport],
    summaries: &[ModuleLexicalSummary],
) -> Result<ActiveLexicalEnvironment, LexicalEnvironmentError>;

pub struct LocalLexicalDeclarations {
    pub user_symbols: Vec<LocalUserSymbolDeclaration>,
    pub operator_declarations: Vec<LocalOperatorDeclaration>,
}

pub struct LocalUserSymbolDeclaration {
    pub spelling: String,
    pub symbol_id: SymbolId,
    pub source_module: ModuleId,
    pub export_rank: ExportRank,
    pub kind: UserSymbolKind,
    pub arity: UserSymbolArity,
    pub declared_at: SourceSpan,
    pub activation_start: SourcePos,
}

pub struct LocalOperatorDeclaration {
    pub spelling: String,
    pub source_module: ModuleId,
    pub declared_at: SourceSpan,
    pub activation_start: SourcePos,
    pub operator: Option<ExportedOperatorMetadata>,
}

pub struct ActiveOperatorMetadata {
    pub spelling: String,
    pub source_module: ModuleId,
    pub declared_at: SourceSpan,
    pub activation_start: SourcePos,
    pub operator: ExportedOperatorMetadata,
}

pub fn collect_local_lexical_declarations(
    raw: &RawTokenStream,
    current_module: ModuleId,
) -> LocalLexicalDeclarations;

pub fn is_constructor_name_spelling(value: &str) -> bool;
```

このモジュールは、最長一致(longest-match)の曖昧性解消で使う探索ヘルパーを公開します。

```rust
impl ActiveLexicalEnvironment {
    pub fn reserved_word(&self, spelling: &str) -> Option<&'static str>;
    pub fn reserved_symbol(&self, spelling: &str) -> Option<&'static str>;
    pub fn user_symbol(&self, spelling: &str) -> Option<&UserSymbolCandidate>;
    pub fn visible_user_symbols_at(
        &self,
        position: SourcePos,
        local_declarations: &LocalLexicalDeclarations,
    ) -> Vec<UserSymbolCandidate>;
    pub fn longest_user_symbol_at(&self, input: &str, start: usize) -> Vec<UserSymbolCandidate>;
    pub fn user_symbols_at(
        &self,
        spelling: &str,
        position: SourcePos,
        local_declarations: &LocalLexicalDeclarations,
    ) -> Vec<UserSymbolCandidate>;
    pub fn longest_user_symbol_at_position(
        &self,
        input: &str,
        start: usize,
        position: SourcePos,
        local_declarations: &LocalLexicalDeclarations,
    ) -> Vec<UserSymbolCandidate>;
    pub fn operator_metadata_at(
        &self,
        spelling: &str,
        position: SourcePos,
        local_declarations: &LocalLexicalDeclarations,
    ) -> Vec<ActiveOperatorMetadata>;
    pub fn visible_operator_metadata_at(
        &self,
        position: SourcePos,
        local_declarations: &LocalLexicalDeclarations,
    ) -> Vec<ActiveOperatorMetadata>;
}
```

## Data Model

`ExportedSymbolShape` は、完全な意味論的 IR ではなく、字句上の形状(shape)だけを保持します。

```rust
pub struct ExportedSymbolShape {
    pub spelling: String,
    pub symbol_id: SymbolId,
    pub source_module: ModuleId,
    pub export_rank: ExportRank,
    pub kind: UserSymbolKind,
    pub arity: UserSymbolArity,
}

pub struct UserSymbolCandidate {
    pub spelling: String,
    pub symbol_id: SymbolId,
    pub source_module: ModuleId,
    pub imported_module: ModuleId,
    pub import_ordinal: usize,
    pub export_rank: ExportRank,
    pub kind: UserSymbolKind,
    pub arity: UserSymbolArity,
}
```

`UserSymbolKind` は、可視な字句エントリのパーサー/リゾルバ上のカテゴリを記録します。
現在の言語仕様では、任意の `user_symbol` 記法は functor と述語にだけ直接許されます。
モード・属性・構造体のエントリは、通常の識別子、または `1-sorted`、`R-module`、
`C-star-algebraic` のような読みやすいハイフン区切り名である `constructor_name` の
綴りでなければなりません。構造体のセレクタは識別子であり、エクスポートされた
字句サマリーエントリとしては受け入れません。従来の汎用 `Constructor` 種別も
サマリー境界で拒否します。constructor-like な字句エントリは、`Mode`、`Attribute`、
`Structure` のいずれかの意味カテゴリで分類する必要があります。`UserSymbolArity` は
引数の個数の形状を、正確な個数・上下限のある範囲・下限のみの範囲として記録します。
これらはパーサー/リゾルバ向けのサマリーであり、完全な型シグネチャではありません。

アクティブ環境は以下を扱います。

- 識別子の形をした functor / predicate シンボルと constructor 名;
- 記号の形をした functor / predicate シンボル;
- `.` を含むシンボル;
- 同綴りでインポートされた候補に対する、インポート衝突(conflict)の検出;
- 診断のための安定した由来(provenance);
- 下流のパーサー/リゾルバフェーズのための、シンボル種別とアリティのメタデータ;
- 現在のモジュールの宣言の、範囲に依存した有効化(activation).

`ModuleLexicalSummary` は、生成側で正規化された生成物です。サマリーを作る構成要素は、字句環境ビルダーに渡す前に `exported_symbols` を決定的な順序に正規化しておく必要があります。正準順序は、少なくとも以下の字句上の同一性と由来に基づきます。

1. `spelling`
2. `source_module`
3. `symbol_id`
4. `kind`
5. `arity`
6. `export_rank`

`build_lexical_environment` はこの規約を前提にしており、サマリーの内部を並べ替えません。これにより環境のフィンガープリント(fingerprint)は、環境ビルダーがその場で選んだ順序ではなく、インポート元モジュールの正準的な字句サマリーに反応するようになります。

この生成側のサマリー順序は、`UserSymbolIndex` の内部で使うアクティブ候補の順序とは独立しています。サマリーがインポートされた後、同綴りの候補は、探索と診断の安定性のため、インポート序数(import ordinal)、エクスポート順位(export rank)、種別、アリティ、ソースモジュール、シンボル ID の順で整列されます。

## Algorithm

現在の実装は、すでに解決済みのインポートから、決定的な探索オブジェクトを構築します。

1. `ModuleLexicalSummary` を `ModuleId` でインデックス化します。同じモジュール ID のサマリーが複数渡された場合、Rust の値として完全に同一なら受け入れます。内容が異なる重複サマリーは構築エラーです。
2. 安定した FNV 方式のフィンガープリントに、バージョン文字列と、組み込みの予約語テーブル・予約記号テーブルを宣言順で書き込みます。
3. `ResolvedImport` をインポートプレリュード順で走査します。各インポートについて、対応する字句サマリーを必須とし、インポート序数・モジュール ID・サマリーのフィンガープリントをアクティブ環境のフィンガープリントに加えます。
4. サマリー内のエクスポートシンボルの形状は、インデックス化する前に綴りとアリティを検証します。functor と述語の綴りは有効な `user_symbol` でなければなりません。モード・属性・構造体の綴りは有効な `constructor_name` でなければなりません。セレクタと汎用 constructor のサマリーエントリは、セレクタが識別子のままであり、constructor-like なエントリは意味カテゴリで分類する必要があるため拒否します。すべてのエクスポート綴りは予約語と衝突してはいけません。予約特殊記号との完全一致も禁止しますが、仕様上の `.` 例外は functor にだけ許可します。アリティの形状は、最大値が最小値より小さくてはいけません。
5. エクスポートされた形状を `UserSymbolCandidate` に変換します。このとき、シンボルを定義・エクスポートした `source_module` と、現在のファイルがインポートした `imported_module` の両方に加えて、シンボル種別とアリティのメタデータを保持します。前者は由来に、後者はインポート衝突の診断に効きます。
6. 候補を `UserSymbolIndex` に挿入します。異なるインポートから同じ綴りが来た場合は `UserSymbolImportConflict` として拒否します。同じインポート内の同じ綴りはオーバーロード(overload)候補として保持でき、上記のアクティブ候補順序で安定化します。
7. 借用した予約テーブル、完成したユーザーシンボルインデックス、決定的なフィンガープリントを持つ `ActiveLexicalEnvironment` を返します。

`UserSymbolIndex` は、綴りの完全一致での探索・決定的な整列・衝突の診断のために、正準的な `BTreeMap<String, Vec<UserSymbolCandidate>>` を保持します。加えて、同じ綴りの集合を ASCII バイトトライ(trie)としても保持し、最長接頭辞(longest-prefix)探索を高速化します。`longest_user_symbol_at` は指定されたバイトオフセットからトライを辿り、最も深い終端ノードを記録し、その綴りの可視なインポート序数に属する候補を返します。したがって候補の探索は、インポートされたシンボルの総数ではなく、走査した綴りの長さと返却する候補数に比例します。公開された探索のセマンティクスは従来と同じです。

実装上の補足:

- `ModuleId` と `SymbolId` は `mizar-lexer` 内の軽量な文字列ニュータイプ(newtype)です。それ自体はモジュールの存在や意味論的な解決を意味しません。
- `ModuleLexicalSummary.exported_symbols` は、生成側で正規化済みであることを前提とします。整列とサマリーのフィンガープリントの安定性はサマリー生成側の責務であり、環境構築側の責務ではありません。
- `UserSymbolCandidate.source_module` は、字句サマリー由来の定義/エクスポートの由来を保持します。`imported_module` は、インポート衝突の診断のために、現在のファイルの解決済みインポートで指定されたモジュールを記録します。
- `UserSymbolCandidate.kind` と `UserSymbolCandidate.arity` は、アクティブ候補ごとに保持します。これにより後続のパーサー/リゾルバフェーズは、モジュールサマリーを作り直さずに、同綴りのオーバーロードを絞り込んだり区別したりできます。
- `.` は、予約特殊記号の衝突規則に対する仕様上の functor 専用の例外です。それ以外の予約記号綴りとの完全一致は拒否します。
- 異なるインポートから来た同綴りのユーザーシンボルは、環境構築上の衝突として拒否します。
- フィンガープリントは、プロセスごとに乱数化されるハッシュではなく内部の安定したバイトハッシャーを使い、シンボル種別とアリティのメタデータも含めます。
- トライは内部の高速化構造であり、フィンガープリントの計算やサマリーの正規化には影響しません。
- `collect_local_lexical_declarations` は浅い raw-token prepass を実行し、
  `pred`、`func`、`mode`、`attr`、`struct`、`synonym`、`antonym`、
  および演算子宣言について、現在のモジュールの有効化イベントをソース順に記録します。
  式、型、証明、完全な宣言本体はパースしません。
- ローカルの `pred` と `func` 宣言について、この prepass は、識別子形の呼び出し・前置・
  中置・後置パターンから直接の記法綴りを記録し、記号形または circumfix 記法パターンでは、
  区切りではない各記号片を記録します。`foo-bar` のような連続したハイフン区切りの
  predicate / functor 記法は、隣接部分が単純な 1 文字 locus の `x-y` operator 形で
  ない場合、1 つの user-symbol 綴りとして記録します。
- ローカルの `mode`、`attr`、`struct` 宣言について、この prepass は
  constructor-name 綴りだけを記録します。連続した読みやすいハイフン区切りの
  constructor 名は 1 つの綴りとして記録し、operator-like な記号名はローカルの
  constructor エントリとして導入しません。ローカルの `attr` 宣言では、
  prefix が数値、または浅い先行 `let` パラメータ scan で見つかった名前である場合に、
  先に属性の `param_prefix` 分割を適用します。この scan には、同じ `let`
  宣言内の後続 qualified segment と implicit 名も含めます。`n-dimensional`、
  `(row,col)-size`、`implicit-shaped` のような形では、constructor-name suffix
  (`dimensional`、`size`、`shaped`) だけを属性の綴りとして記録します。
  `Function of REAL, REAL` や `Function[REAL, REAL]` のような parameterized type
  expression 内の comma は、追加の属性パラメータ宣言として扱いません。一方で、
  後続の `X be set` のような explicit segment は、1 文字 uppercase の
  パラメータ名として寄与できます。`of` / `over` の型引数リストの後に別の
  explicit segment が続く場合、この scan は前方の型引数をパラメータとして扱わず、
  `g, k be set` のような comma-separated value-name list を含め、識別できる
  segment 境界からだけ再開します。この scan は `such` と `by` で停止するため、
  後続の条件や参照リストから偽の属性パラメータを導入しません。
- ローカルの `synonym` と `antonym` 宣言は、保守的な浅い分類を使います。alias 側
  または original 側に明確な operator-like 記法の手掛かりがある場合、その alias は
  述語 / functor 形式の記法として記録します。それ以外では、alias head が
  constructor-name 綴りである場合にだけ記録します。完全な意味論的 alias-family 分類は
  resolver の責務であり、この prepass は型情報を使って曖昧な word-only alias を任意の
  記号記法として解釈しません。
- ローカルのユーザーシンボル候補は、問い合わせ位置が宣言項目末尾の有効化オフセット以上の
  場合にだけ `longest_user_symbol_at_position` から見えます。定義に宣言所有の
  correctness/property clause が続く場合、この完了境界にはその trailing clause と
  その proof block も含まれます。そのため、宣言自身のヘッダー / 定義項、correctness
  trail、およびそれ以前のテキストは、その宣言が導入する綴りを見ることができません。
- 同じ綴りのローカル / インポートのエントリは、下流の resolver フェーズの
  overload candidate として結合されます。ローカルエントリは、インポート済みエントリを
  字句的にシャドウしません。
- `private` と `public` はローカル字句 prepass では無視されます。`algorithm`、
  `redefine`、inline `deffunc`、inline `defpred`、構造体 selector、および
  field/property 名は、ローカル lexer user-symbol 辞書エントリを導入しません。
- 演算子宣言は有効化イベントとして別に記録され、user-symbol 候補は導入しません。
  インポート由来の演算子メタデータは lexical byte offset `0` から有効として公開します。
  ローカルの演算子メタデータは、metadata を解析でき、query 位置で宣言が有効であり、
  かつ演算子宣言の spelling 位置で同じ spelling と対応 arity の active functor
  candidate が少なくとも 1 つ存在した場合にだけ公開します。これにより、不正な演算子宣言に
  lexer 診断を追加せずに forward reference 禁止を実装します。
- parser-facing な演算子メタデータは overload root の選択ではなく、spelling 単位の
  metadata です。同じ spelling の imported / local functor candidate は後続 resolver
  用の overload candidate として残し、`operator_metadata_at` は spelling / fixity /
  precedence entry を決定的に返します。後から有効になる activation point が先に考慮される順に
  並べます。互換しない same-spelling metadata の link-time conflict 診断は lexer の外に残ります。

## Non-Goals

このモジュールは以下を行いません。

- ソーステキストをパースする;
- インポート構文を解決する;
- 完全なモジュール IR を読み込む;
- 局所スコープの上書き(override)を判定する;
- シンボルの使用が型として正しいかを判定する;
- overload resolution result を選ぶ。

active な環境は、インポートされたモジュールの圧縮された `ModuleLexicalSummary` 射影（シンボルの綴り・種別・arity）のみを保持し、その定義・証明本体・完全なモジュール IR は決して保持しません（上記 Non-Goals を参照）。したがってこれは、常駐集合メモリモデルの「本体ではなくインターフェイスを保持する」規則の、レキサ層における表現です（spec [§12.6.3](../../../spec/ja/12.modules_and_namespaces.md#1263-メモリモデル)、アーキテクチャ [03.module_and_symbol_resolution.md](../../architecture/ja/03.module_and_symbol_resolution.md)）。インポートされた summary は、import closure 全体を先読みするのではなく、`LexicalSummaryProvider` のシームを通じて必要時に供給されます。

## Error Handling

ここでのエラーは環境構築の失敗であり、トークン化の失敗ではありません。

- 解決済みインポートに対応するモジュール字句サマリーがない;
- 同じモジュール ID に対して内容の異なる重複サマリーがある;
- エクスポートシンボルが、予約語または予約特殊記号と不正に衝突する;
- 異なるインポートがエクスポートする同綴りのユーザーシンボルが衝突する;
- ユーザーシンボルの綴りが不正;
- モード・属性・構造体エントリの constructor-name 綴りが不正;
- ユーザーシンボルのアリティ形状が不正。
- サポートされないセレクタまたは汎用 constructor の字句サマリー種別。

同じインポート元モジュール内の同綴りユーザーシンボルは、決定的な候補として表現できます。一方、異なるインポートから来た同綴りシンボルは衝突として拒否します。インポート順序とサマリー順序はエラーとして診断しませんが、決定的な入力規約の一部であり、環境のフィンガープリントに反映されます。

## Tests

テストでは以下を確認します。

- 予約テーブルが常に存在すること;
- インポートしたシンボルが可視になること;
- 異なるインポートから来た同綴りユーザーシンボルが決定的に拒否されること;
- 予約語との衝突が拒否されること;
- functor / predicate エントリは自由な記法を受け入れ、mode / attribute / structure
  エントリは constructor name を要求すること;
- selector と汎用 constructor のサマリーエントリが拒否されること;
- ローカルの読みやすいハイフン区切り constructor 名が 1 つの綴りとして記録されること;
- ローカルのパラメータ付き属性宣言は、`param_prefix` を含む綴りではなく
  constructor-name suffix を記録すること;
- 決定的な入力順序の下で環境のフィンガープリントが安定すること;
- 識別子の形・記号の形をしたシンボルに対する最長一致クエリに答えられること;
- 多数のインポートシンボルと綴りの重なりがあっても、トライに基づく探索が最長一致の動作を保つこと;
- 同綴りのオーバーロード候補について、種別/アリティのメタデータが保持されること;
- 種別またはアリティのメタデータが変わると、環境のフィンガープリントが変わること。
- 現在のモジュールのローカル宣言は、宣言項目より前では非アクティブであり、
  自身のヘッダー / 定義項でも非アクティブで、後続のソース位置でだけアクティブになること;
- 同じ綴りのローカル / インポート候補がどちらも保持されること;
- `private` / `public` がローカル字句有効化に影響しないこと;
- 演算子宣言、`deffunc`、`defpred`、`algorithm`、`redefine` がローカル
  user-symbol エントリを導入しないこと;
- parser-facing な演算子メタデータ query が、import 由来 metadata、local の before-use
  metadata、宣言後の使用だけでの有効化、`private` / `public` の no-op visibility、
  forward reference の拒否、および同綴り overload candidate の保持をカバーすること;
- synonym / antonym の prepass 有効化は `for` より前の alias pattern から派生し、
  `for` より後の original pattern からは派生しないこと。
