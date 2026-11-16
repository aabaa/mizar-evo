# Ranged diagnostic host input
正本: 英語。[English companion](../en/ranged_diagnostic_host.md)。

## Admission と所有権

本書は [native producers](native_payload_producers.md) の E0041 profile と組になる承認済み Q3-B を定義します。メモリ内の診断 publication であり、成功した manifest-backed [full snapshot](snapshot.md#freshness-and-replacement) や version-keyed syntax overlay とは別です。Semantic metadata、artifact、proof 権威を与えません。
明示的な originating submission は保護された native capture を消費して Driver を一度呼び、既存 native currentness guard に従う terminal Succeeded/Failed/Blocked session の元の不変 BuildDiagnosticIndex だけを受け入れます。全 record/anchor は元 captured source に対して ranged かつ変換可能である必要があります。下記 captured-failure 経路以外の Non-range SourceLoad と partial/nonterminal/obsolete/foreign 入力は unavailable とし、拒否時は以前の選択を保持します。
元 capture Arc、source text/maps、URI/version/incarnation、正確な集約済み record、producer-backed context/store を一緒に保持します。最後の publication/reader まで既存 OpenBufferOverlay retention を保ち、IR output や completed manifest を合成しません。再利用しない process-local identity で、snapshot id が同じ場合も activation を区別します。

`NativeOpenBufferCapture::capture_source_load_failures` は元 Driver session に実 Disk request の失敗を[捕捉契約](../../mizar-driver/ja/request.md#native-source-capture)に従って保持する。同じ originating host entry は元 capture/editor association と不変の native guard を要求し、実 `SourceLoadFailed` native index を wire conversion なしで受け入れ package/path を保持する。この経路は実 producer の secondary anchor なし SourceLoad record だけを受け入れ、ranged record は既存 conversion preflight を使う。失敗 source の text を保持せず full snapshot を作らない。配送は[承認済み non-range 表示](diagnostics.md#range-conversion)を使い、Task41 証跡は別途扱う。

## Host selection と binding

Adapter は action/explanation/command read 用に一つの診断 publication を原子的に選択します。後の受入済み診断または full publication が置換し、metadata/navigation は full snapshot のみを選び続けます。既存 public full-snapshot accessor は成功時だけという意味を保持し、admission 失敗時は以前の host selection を保ちます。
Driver は決定的な producer occurrence key を持つ実 canonical diagnostic draft を供給します。Host は正確な canonical attached producer command/handle と保持した owner-authenticated context だけを結び、ordinal/code/span/title や単なる共在から推測しません。Command 公開前に正確な store と承認済み deferred registration に相当する binding を設置します。未対応 dependency/encoding/association では該当 payload を個別に省きます。
Source-error 入力には manifest 権威がなく、artifact-assisted/artifact-dependent/missing-facts action は unavailable です。Command は承認済み read-only explain reference のみです。Local edit と bounded retained CacheRecord read は既存検証と wire conversion を再利用し、別 command carrier や client 権威を導入しません。
明示的な host explanation entry は選択された retained producer store を読みます。既存 caller-store entry は互換性と handle 検証を保ちます。Store content から currentness や fix 権威は作れません。

## Currentness と保持

Current action delivery は同じ選択 publication identity、live native Driver guard、正確な元 editor tuple/incarnation を必要とします。Offer/resolve/execute と async read 後に再検証し、既存 cancellation/error を保ちます。Version-only change、normalized-equal edit、同一 tuple の reopen も元 association の action を恒久的に無効化します。
編集後も説明用に診断 publication の選択を保持できます。元 content、URI/version、raw map を保持して stale_artifact とし、旧 offset を live text に再写像しません。古い requested snapshot や read 中の置換は stale_handle です。同じ id の新 record に旧 backing を貸しません。Terminal connection では current action delivery を許可しません。
選択破棄は既存 retained-read lifetime に従い、実行中 reader は元 association/store 全体を保持します。永続 cache format、eviction policy、OpenBuffer artifact provenance、SourceId serialization は決定しません。
