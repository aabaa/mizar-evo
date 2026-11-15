# Navigation features
正本: [英語版](../en/navigation.md)。英語版と同期した日本語訳。

## Responsibility

navigation は [architecture12 の機能別入力](../../architecture/en/12.diagnostics_and_lsp.md#request-sources) に従い、定義、参照、document symbol、semantic token を提供する。[internal03 の LSP Bridge](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lsp-bridge) に従い、LSP は editor 向け変換を所有し、名前解決、型検査、証明受理、artifact の変更は行わない。

## Feature sources

定義は [公開 metadata reader](metadata.md#published-inputs) を通じて解決済み symbol metadata と export summary を読む。producer が所有する identity と source range を保つ。任意または欠落した metadata は、不足する symbol の名前解決を許可しない。

参照は選択した [LspSnapshot](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lsp-snapshot) の、artifact と source snapshot から構築された symbol index を読む。この仕様は読む入力元だけを定め、symbol index の構造や新しい artifact projection を定義しない。

document symbol は現在の buffer の parsed outline を読む。その buffer に対して fresh な場合だけ artifact metadata で outline を補強する。stale artifact は現在の outline の補強には使わない。

semantic token は現在の buffer の lexer/parser 出力を読み、fresh な場合だけ解決済み symbol class を加える。fresh な semantic artifact がなくても syntax-only token を利用できる。現在の buffer の token に stale な semantic class を適用しない。

## Snapshot association and freshness

[source association](snapshot.md#source-association) と [freshness](snapshot.md#freshness-and-replacement) をそのまま適用する。各 request は一つの immutable な source/metadata association を保持し、置換時にも進行中の read 内で新旧入力を混ぜない。native range は対応する元 source を用いて [range 変換](range.md) する。

保持された semantic read は `stale_artifact` になり得る。古いという理由だけで隠さず、明示的に stale とする。元の source association を保ち、range を現在の buffer の range と呼び替えず、target を検証できない current delivery を抑止する。現在の buffer の outline/token の補強には引き続き fresh な metadata が必要である。

semantic 入力が欠ける場合は、利用可能な syntax-only 機能へ段階的に縮退し、semantic な事実、空の成功 snapshot、予定済み作業を捏造しない。`pending` には実際に予定済みまたは実行中の検証が必要である。syntax-only overlay は semantic metadata も coherent な full snapshot も供給せず、等しい build id や保持 handle は currentness を確立しない。既存の [retention](snapshot.md#currentness-and-retention) 規則を保つ。

## Projection boundary

metadata reader の [projection 境界](metadata.md#projection-boundary) に従い、既存の公開 `VerifiedArtifact` 式 metadata と module summary を、snapshot data および上記の現在の buffer の構文入力とともに読む。生の compiler IR は読まず、compiler phase を通じて不足する semantic metadata を再構成しない。

schema、summary の形、format、validating reader は Artifact が引き続き所有する。navigation は metadata field、index 表現、公開 diagnostic code/name/alias、producer/reader/snapshot の挙動を追加しない。

## Implementation boundary

task14 が固定した read の境界に従い、task15 は metadata producer の着地に合わせて navigation を増分で実装する。registry/atomic activation、診断公開、hover はそれぞれ別の owner が引き続き所有する。

task15 の syntax-only token 増分 は、現在の open-buffer の正確な text を既存 lexer の前処理と shell 分類へ渡し、`textDocument/semanticTokens/full` を提供する。legend は `keyword`、`operator`、`number`、`comment`、modifier は空で、役割が不明な identifier は省略する。raw LF、CRLF、単独 CR で区切った各行の UTF-16 token を返す。artifact class による補強は行わず、保持された stale metadata は現在の構文 token に影響しない。定義、参照、document symbol、resolved token の補強は保留し、task15 は未完了のままとする。

navigation は公開済み入力と既存の snapshot association を使い、保留中の OpenBuffer artifact provenance/storage、Session SourceId encoding、registry activation の判断を前提としない。ここではそれらを決定しない。

## Test design

この設計単位では code も test も変更しない。task15 が実装と機能検証の所有権を保ち、module completion と §23.6/SCA-003 の credit は変わらない。
