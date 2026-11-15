# 遅延 code-action validator 契約
Canonical language: English; [英語正本](../en/code_action_validators.md)。

## Status and authority

**所有者の承認待ちの設計案です。** 以下の選択肢は、承認済みのハッシュの意味、由来情報の API、コマンドの符号化ではありません。必要な選択肢の承認と再 dispatch まで Task38 は停止を維持し、承認済み validator と実行所有者が利用可能になるまで既存の artifact/hash/command フィルタを継続します。Task39 には独自の native payload producer も必要です。
既存要件は spec [§23.6.1](../../../spec/en/23.package_management_and_build_system.md#2361-hover-code-actions-and-diagnostics)、architecture12 [action classes](../../architecture/en/12.diagnostics_and_lsp.md#code-actions)、internal03 [action flow](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#code-actions)、[code-action safety](code_action.md#applicability-and-freshness) に従います。以下は不足する契約を補う提案であり、言語・証明の受理や coverage credit を変更しません。

## Hash domains and inputs

汎用 `Hash` 値だけではドメインを特定できません。validator には生成所有者、ドメインと版、正確な入力との関連が必要であり、digest の一致だけでは不十分です。未知のドメイン、比較可能な入力の欠落、非互換な版では対象 fix を利用不可のままとします。既存の [fix fields](../../mizar-diagnostics/en/fix.md#data-model) を維持し、この案だけでは payload field や production API を追加しません。

| 入力 | 既存の権威または提案する検証契約 |
|---|---|
| Build source hash | 元の **loaded** source text に Session の `hash_text` を再利用します。[disk/open-buffer text は BOM/CRLF 正規化後、generated text は byte-for-byte の保持](../../mizar-session/en/source.md#loaded-source) に従います。BLAKE3 の入力は `b"mizar-session/source-text/v1" || u64_le(UTF8_length) || UTF8_bytes`、manifest 表記は `mizar-session/hash-text/v1:<digest>` です。正確な source entry と module の manifest source hash を照合し、Driver の source-input fingerprint、build ID、生の editor bytes で代用しません。 |
| Artifact hash | manifest/store 所有者の [hash domains](../../mizar-artifact/en/manifest.md#hash-string-domains) と [reader validation](../../mizar-artifact/en/manifest.md#reader-requirements) に従い、canonical JSON、schema が指定する除外、artifact framing/class/family/version を使用します。manifest 到達可能性と参照 module/content を検証し、生ファイルの checksum や interface hash で代用しません。 |
| `required_text_hash` | **Q1:** A 案では単一の編集対象 source の元の loaded text 全体に Session の `hash_text` を再利用します。range、replacement、URI、version、publication identity は含めず別途検証します。A では複数 source の hashed fix は利用不可を維持します。producer のドメイン・入力宣言なしに既存の opaque 値を再解釈しません。 |
| Explanation hashes | `required_artifact_hash` は所有 artifact、`Artifact.content_hash` は backing content、`summary_hash` は canonical な bounded summary/descriptor に結び付き、ローカライズ済み preview には結び付きません。**Q2** で content/summary のドメインと比較入力を定義する必要があります。汎用 hash の一致や preview/store lookup の成功だけでは編集を認可できません。 |
| Missing facts | 認証済み enclosing artifact または manifest から到達可能な development artifact、その所有者定義の content hash/schema、この fix が利用する正確な obligation/fact entry が必要です。**Q2** で selector と source/VC の関連を定義する必要があります。`missing_facts.json` という名前、path、title、candidate text だけでは証明になりません。 |

[Snapshot association](snapshot.md#source-association) に従い、生の editor URI/version/incarnation、保持した元の text/maps、current publication、各 expected-text/range の検証は引き続き必須です。正規化後の source hash や build ID の一致で stale action は復活しません。各編集対象 source に元と現在の対応および必要な artifact provenance が必要であり、primary source 一つの hash で別ファイルを検証しません。
**Q1 — text hash の範囲:** 既存ドメインを使う限定的な最初の consumer として上記 A を推奨します。B は、決定的な順序、source の対応、source ごとの bytes、framing を含む版付き aggregate edit-source digest を定義し、複数 source の hashed fix を扱います。正確な符号化は利用前に別途承認された所有者定義が必要です。A はフィルタが多く残り、B は新しい producer/validator 契約を要しますが、保留中の SourceId serialization を決めてはなりません。所有者が意味と producer migration/registration の境界を承認します。
**Q2 — backing のドメインと selector:** A を推奨します。artifact-backed `content_hash` は既存の正確な store-framed artifact hash を再利用し、非 artifact の bounded content/summary には BLAKE3 入力 `b"mizar-lsp/action-backing/v1" || frame(role) || frame(encoding) || frame(bytes)` を提案します。`frame(x) = u64_le(byte_length(x)) || x`、role は UTF-8 の `content` または `summary`、encoding は producer 所有の版付き encoding key、bytes はその元の bounded payload または canonical structured summary/descriptor そのもの（ローカライズ済み preview を除く）です。producer が bytes と encoding を供給し、未知の encoding は利用不可です。missing-facts selector は認証済み backing artifact 内の producer 認証済み obligation key と正確な entry selector とし、source/VC の対応を producer が検証し、LSP が schema を推測しません。B は backing と summary の全てを既存 schema で検証した artifact として公開し、store hash を使い、未提供ならフィルタを維持します。A は保持した query data に対応しますが新たな producer/validator registration が必要で、B は範囲が狭く upstream publication 作業が必要になり得ます。ドメイン/role、対応する所有者 encoding、missing-facts selector の意味を承認するか、対象 consumer のフィルタ継続を明示します。missing-facts schema や保留中の producer 判断は定義せず、backend candidate と kernel-verified `used_axioms` を区別します。

## Fix and handle provenance

**提案する受理経路:** 再利用しない process-local identity で単一の immutable current publication を選び、その正確な build index 内で diagnostic handle を解決し、その record 内の fix id と canonical [fix payload](../../mizar-diagnostics/en/fix.md#attachment-to-records) を照合し、その特定 fix に関連する所有者認証済み backing を取得し、各 target source、manifest/module/artifact、必要な handle を検証します。ID、subject string の一致、同一 record 上の併存だけでは fix と handle の関連を証明しません。
**Q3 — 関連の保持場所:** A として、選択した publication とともに genuine producer から登録する host 所有の immutable association を推奨します。publication/diagnostic/canonical fix を key とし、既存の正確な explanation handle または missing-facts selector に結び付けます。B は別途承認する schema 変更で producer payload に明示的な typed reference を追加します。A は `FixSuggestion` field が不要で、B は永続化できますが upstream scope が広がります。どちらも diagnostic title、path、配列位置、唯一の attached handle、client data から推測して登録しません。candidate が一つでも producer binding がなければ拒否します。
関連は cross-module input を含め fix が実際に使う backing dependency 全てを対象とし、最初の一致だけでなく全てを検証します。explanation は元の handle の kind/subject/source、snapshot、artifact/content/summary precondition を所有 store/validator で検証します。無関係な登録 payload や cache hit は権威にならず、backing が欠落・曖昧・未対応・不一致なら fix は利用不可です。保持した stale explanation は [explanation rules](explain.md#freshness-and-stale-handles) に従って読めますが、編集の権威にはなりません。
これは memory 内の所有関係の提案であり、OpenBuffer artifact provenance/storage、Session SourceId encoding、native producer の完全性を決めません。それらの所有者が必要な認証済み関連を供給できなければ、対象 action のフィルタを維持します。

## Command resolution and execution

[Fix owner](../../mizar-diagnostics/en/fix.md#data-model) に従い `FixCommandRef` は opaque な producer reference を維持します。LSP は shell command、CLI string、URI、任意の method name、信頼済み client argument として扱いません。提案では host の明示的 allowlist により、正確な producer reference と canonical fix binding を、対応する単一の操作とその保持済み所有者入力に結び付け、未知の ref は利用不可とします。
**Q4 — 経路と wire 契約:** A として、実行 host があるときだけ advertise する単一の server command `mizar.executeDiagnosticCommand` を推奨します。初期 code action は title/kind/data locator を維持し、command 対応 client には `command` resolve の対応を要求します。resolve は固定名と単一の opaque locator argument を持つ command を再構築し、publication/diagnostic/fix/元の document association に結び付けます。client が供給する操作、file path、command argument は権威になりません。B は既存の bounded query/scheduling outcome を返す専用 custom request とし、capability と request schema を別途合意するため client integration が必要です。経路、名前/argument shape、capability/error 契約の承認前には公開せず、既存の edit-only client の動作を維持します。
操作所有者の提案: explain は bounded [explanation host](explain.md#implementation-boundary) に委譲し、show-VC は検証済みの保持した obligation data を読んで client 用表示データを返し、reverify は既存の Driver/build host の currentness/admission 検証後にのみ submit します。操作所有者がなければ offer/execute しません。query semantics、VC schema、scheduler policy、proof acceptance、shell 起動、直接の workspace mutation は追加しません。
lifecycle の提案: offer は対応と由来を検証し、resolve は current publication を選び直して server の権威から再構築し、execute は client の edits/command contents を無視して locator を再解決し、委譲の直前に canonical fix、snapshot/diagnostic/handle、backing hash、全ての live source association を再検証します。所有者は replacement/edit/close/reopen に対し効果の受付を atomic に保護する必要があります。非同期 read は元の入力を保持し、current result を返す前に currentness を再検証します。受付前の cancellation は作業を防ぎ、受付後は既存の操作所有者の契約に従い rollback を約束しません。scheduling は受付を検証するのであり、自らの build が snapshot を置換した後の同一性を要求しません。後で source を編集するには別途明示的な editor application と完全な編集検証が必要です。
**Q5 — 結果と受付:** A を推奨します。不正な request/argument shape → InvalidParams、未対応 route → MethodNotFound、形は正しいが未知・stale・未関連の locator、canonical fix の変更、resolve/execute 時の検証失敗 → ContentModified と効果なし、受付前の cancellation → RequestCancelled、host 自体の失敗 → InternalError、成功 → bounded な操作所有者のデータまたは scheduling acknowledgement とし、暗黙の編集は行いません。offer は利用不可の suggestion をフィルタします。B は explanation query 同様、command 検証失敗に structured unavailable/stale outcome を返します。所有者は result shape を承認し、atomic admission seam を確認する必要があります（なければ効果を伴う command のフィルタを継続）。新しい error code、budget、rollback policy はここでは選択しません。

## Conditional validation matrix

以下は承認済み設計と将来実装の review 義務であり、新たな実行 test や現在の positive-path credit ではありません。既存のフィルタ・拒否 test を変更しません。

| Case | 入力形と必要な結果 |
|---|---|
| 1 | BOM/CRLF/Unicode または version-only edit で build ID が同じ場合: loaded-source hash は一致し得ますが、raw/currentness/range 検証で stale edit を拒否し、元の座標を保持します。 |
| 2 | 複数 source/edit、expected text または hash scope の欠落: 全 source と明示的な text precondition を検証します。Q1-A では複数 source の hashed fix を拒否し、既存の hash なし複数 source の動作を維持します。 |
| 3 | manifest/module/artifact の一致と、source/hash/class/schema の不一致または未到達 member: 正確な所有者検証済み経路のみが action を認可し、不一致は全て拒否します。 |
| 4 | handle を共有する二つの fix/diagnostic、関連の欠落・曖昧さ、summary/content/VC/fact selector の不一致: 正確な認証済み fix 関係と全 backing dependency を要求し、推測しません。 |
| 5 | 既知・未知の ref、host/capability の欠落、偽造 locator、供給された command contents: 承認済み登録 route/operation と server 再構築入力のみを許し、それ以外はフィルタ・拒否します。 |
| 6 | offer/resolve/execute 間の edit/reopen/equal-id または別 Server の replacement/cancellation: 古い locator は復活せず、stale な効果を受け付けません。保持 read は元の source を保ち、cancellation は所有者の境界に従います。 |

所有者は Q1–Q5 の選択と、意図的にフィルタを維持する consumer を記録して承認する必要があります。その後も Task38 には新たな実装 dispatch と所有者に裏付けられた validation/admission seam が必要であり、この設計案だけでは提供しません。
