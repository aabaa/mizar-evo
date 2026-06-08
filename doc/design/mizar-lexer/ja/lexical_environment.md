# Module: lexical_environment

> Canonical language: English. English canonical version: [../en/lexical_environment.md](../en/lexical_environment.md).

## Purpose

このモジュールは、トークンの曖昧性解消(disambiguation)が参照する、ファイル単位のアクティブ字句環境(active lexical environment)を構築します。

この環境は、組み込みの予約テーブルと、インポートプレリュード(import prelude)で指定されたモジュールがエクスポートする字句シンボルのサマリーとを結合します。インポートはファイル先頭のプレリュードに限定されるため、この環境はソースファイル本体の全体で安定しています。

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
```

このモジュールは、最長一致(longest-match)の曖昧性解消で使う探索ヘルパーを公開します。

```rust
impl ActiveLexicalEnvironment {
    pub fn reserved_word(&self, spelling: &str) -> Option<&'static str>;
    pub fn reserved_symbol(&self, spelling: &str) -> Option<&'static str>;
    pub fn user_symbol(&self, spelling: &str) -> Option<&UserSymbolCandidate>;
    pub fn longest_user_symbol_at(&self, input: &str, start: usize) -> Vec<UserSymbolCandidate>;
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

`UserSymbolKind` は、可視なシンボルのパーサー/リゾルバ上のカテゴリを記録します。カテゴリは、関手(functor)、述語(predicate)、モード(mode)、属性(attribute)、構造(structure)、セレクタ(selector)、構成子(constructor)です。`UserSymbolArity` は引数の個数の形状を、正確な個数・上下限のある範囲・下限のみの範囲として記録します。これらはパーサー/リゾルバ向けのサマリーであり、完全な型シグネチャではありません。

アクティブ環境は以下を扱います。

- 識別子の形をしたシンボル;
- 記号の形をしたシンボル;
- `.` を含むシンボル;
- 同綴りでインポートされた候補に対する、インポート衝突(conflict)の検出;
- 診断のための安定した由来(provenance);
- 下流のパーサー/リゾルバフェーズのための、シンボル種別とアリティのメタデータ.

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
4. サマリー内のエクスポートシンボルの形状は、インデックス化する前に綴りとアリティを検証します。綴りはユーザーシンボルの綴りでなければならず、予約語と衝突してはいけません。予約特殊記号との完全一致も原則として禁止しますが、仕様上の例外である `.` だけは許可します。アリティの形状は、最大値が最小値より小さくてはいけません。
5. エクスポートされた形状を `UserSymbolCandidate` に変換します。このとき、シンボルを定義・エクスポートした `source_module` と、現在のファイルがインポートした `imported_module` の両方に加えて、シンボル種別とアリティのメタデータを保持します。前者は由来に、後者はインポート衝突の診断に効きます。
6. 候補を `UserSymbolIndex` に挿入します。異なるインポートから同じ綴りが来た場合は `UserSymbolImportConflict` として拒否します。同じインポート内の同じ綴りはオーバーロード(overload)候補として保持でき、上記のアクティブ候補順序で安定化します。
7. 借用した予約テーブル、完成したユーザーシンボルインデックス、決定的なフィンガープリントを持つ `ActiveLexicalEnvironment` を返します。

`UserSymbolIndex` は、綴りの完全一致での探索・決定的な整列・衝突の診断のために、正準的な `BTreeMap<String, Vec<UserSymbolCandidate>>` を保持します。加えて、同じ綴りの集合を ASCII バイトトライ(trie)としても保持し、最長接頭辞(longest-prefix)探索を高速化します。`longest_user_symbol_at` は指定されたバイトオフセットからトライを辿り、最も深い終端ノードを記録し、その綴りの可視なインポート序数に属する候補を返します。したがって候補の探索は、インポートされたシンボルの総数ではなく、走査した綴りの長さと返却する候補数に比例します。公開された探索のセマンティクスは従来と同じです。

実装上の補足:

- `ModuleId` と `SymbolId` は `mizar-lexer` 内の軽量な文字列ニュータイプ(newtype)です。それ自体はモジュールの存在や意味論的な解決を意味しません。
- `ModuleLexicalSummary.exported_symbols` は、生成側で正規化済みであることを前提とします。整列とサマリーのフィンガープリントの安定性はサマリー生成側の責務であり、環境構築側の責務ではありません。
- `UserSymbolCandidate.source_module` は、字句サマリー由来の定義/エクスポートの由来を保持します。`imported_module` は、インポート衝突の診断のために、現在のファイルの解決済みインポートで指定されたモジュールを記録します。
- `UserSymbolCandidate.kind` と `UserSymbolCandidate.arity` は、アクティブ候補ごとに保持します。これにより後続のパーサー/リゾルバフェーズは、モジュールサマリーを作り直さずに、同綴りのオーバーロードを絞り込んだり区別したりできます。
- `.` は、予約特殊記号の衝突規則に対する仕様上の例外です。それ以外の予約記号綴りとの完全一致は拒否します。
- 異なるインポートから来た同綴りのユーザーシンボルは、環境構築上の衝突として拒否します。
- フィンガープリントは、プロセスごとに乱数化されるハッシュではなく内部の安定したバイトハッシャーを使い、シンボル種別とアリティのメタデータも含めます。
- トライは内部の高速化構造であり、フィンガープリントの計算やサマリーの正規化には影響しません。

## Non-Goals

このモジュールは以下を行いません。

- ソーステキストをパースする;
- インポート構文を解決する;
- 完全なモジュール IR を読み込む;
- 局所スコープの上書き(override)を判定する;
- シンボルの使用が型として正しいかを判定する;
- オーバーロードの勝者を選ぶ.

active な環境は、インポートされたモジュールの圧縮された `ModuleLexicalSummary` 射影（シンボルの綴り・種別・arity）のみを保持し、その定義・証明本体・完全なモジュール IR は決して保持しません（上記 Non-Goals を参照）。したがってこれは、常駐集合メモリモデルの「本体ではなくインターフェイスを保持する」規則の、レキサ層における表現です（spec [§12.6.3](../../../spec/ja/12.modules_and_namespaces.md#1263-メモリモデル)、アーキテクチャ [03.module_and_symbol_resolution.md](../../architecture/ja/03.module_and_symbol_resolution.md)）。インポートされた summary は、import closure 全体を先読みするのではなく、`LexicalSummaryProvider` のシームを通じて必要時に供給されます。

## Error Handling

ここでのエラーは環境構築の失敗であり、トークン化の失敗ではありません。

- 解決済みインポートに対応するモジュール字句サマリーがない;
- 同じモジュール ID に対して内容の異なる重複サマリーがある;
- エクスポートシンボルが、予約語または予約特殊記号と不正に衝突する;
- 異なるインポートがエクスポートする同綴りのユーザーシンボルが衝突する;
- ユーザーシンボルの綴りが不正;
- ユーザーシンボルのアリティ形状が不正。

同じインポート元モジュール内の同綴りユーザーシンボルは、決定的な候補として表現できます。一方、異なるインポートから来た同綴りシンボルは衝突として拒否します。インポート順序とサマリー順序はエラーとして診断しませんが、決定的な入力規約の一部であり、環境のフィンガープリントに反映されます。

## Tests

テストでは以下を確認します。

- 予約テーブルが常に存在すること;
- インポートしたシンボルが可視になること;
- 異なるインポートから来た同綴りユーザーシンボルが決定的に拒否されること;
- 予約語との衝突が拒否されること;
- 決定的な入力順序の下で環境のフィンガープリントが安定すること;
- 識別子の形・記号の形をしたシンボルに対する最長一致クエリに答えられること;
- 多数のインポートシンボルと綴りの重なりがあっても、トライに基づく探索が最長一致の動作を保つこと;
- 同綴りのオーバーロード候補について、種別/アリティのメタデータが保持されること;
- 種別またはアリティのメタデータが変わると、環境のフィンガープリントが変わること。
