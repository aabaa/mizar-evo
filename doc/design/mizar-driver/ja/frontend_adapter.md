# Source Services

> 正本は英語です。[英語版](../en/frontend_adapter.md)。

## Disk SourceLoad service

`PhaseRegistryBuilder::register_source_load()` は `mizar-frontend` 所有の実 disk-only service `SourceLoad` を `PipelinePhase::SourceLoad` に登録する。引数なしの登録メソッドは単一 phase の descriptor/catalog identity を固定し、別の public service type を公開しない。Frontend の完全 payload と共有診断 mapping は引き続き外部依存 gap とする。
`SourceLoadInputs<'a>` は capture 済み `BuildSnapshot`、`BuildPlan`、`ModuleIndex`、呼出元 `SessionIdAllocator` を借用する。`PhaseExecutionResources<'a>` と `PhaseExecutionContext<'a>` が optional source inputs を運び、実 submission が scheduler dispatcher に渡す。後続 service がない default graph ではこの経路は未実行となるため、A5 は新しい graph/profile を作らず registry 直接実行を検証する。登録 service は allocator や payload を保持しない。

## Binding and execution

module work unit に対応する disk SourceVersion、workspace package plan、package index entry、workspace module entry がそれぞれ一意であることを要求する。package/module/path/edition、package root/source root/manifest、snapshot/workspace root を照合し、重複や矛盾を拒否する。module の source-relative path は package-relative path と source root に整合させる。安全でない workspace-relative plan root は拒否する。
canonical package root は canonical captured workspace 内に限定し、確定した canonical root から load する。root が利用できない場合は loader の実診断を通すが、publication には root 包含の確認を要求する。
capture 済み typed normalized path と metadata から Disk SourceInput を作り、workspace と package plan から package root を得る。filesystem 正規化と source 受理は session loader が所有する。
dispatch input hash は capture した version の SourceUnitCacheKey と一致させる。SourceLoad は output parent と dependency hash を消費しない。不足した dispatch identity は blocking のままとし planner output handle を捏造しない。
current publisher と空かつ未 seal で同一 snapshot/SourceLoad scope の diagnostic sink を要求する。不足・不整合 resource、binding/identity 不正、publication 失敗、capture 後の source 変更は output なしの Blocking。成功した load の hash 不一致は architecture 22 の obsolete dispatch result として拒否し、loader failure や新しい E0600 原因とは扱わない。一致する cancellation token は Cancelled、異なる snapshot の token は Blocking とする。
呼出元 allocator で `FrontendSourceLoader<DiskSourceLoader>::load_source_unit` を呼ぶ。loaded metadata、Disk origin、normalized text hash を照合し、disk codec で実 payload/map を capture 済み SourceId に結び直して publisher に渡す。snapshot の登録・復活や work-unit 許可は service が行わない。
publication 表現は [frontend owner](../../mizar-frontend/ja/source.md#sourceunit-publication) に従う。decoder は current request metadata、SourceId、expected source hash だけを捕捉し、bytes を復号する。完成 payload の保持や再読込・再parse はしない。
publication 成功時だけ実 sealed output を持つ Complete を返す。後続 service がない全体 graph は従来通り submit を拒否する。SourceLoad の直接実行を full build 完了とは扱わない。

## Diagnostics and cache boundary

実 loader error は request の package/path、SourceLoad phase/category、current snapshot を持つ共有 draft を emit し、output なしの Fatal を返す。偽 range は使わない。InvalidUtf8 は E0601、UnreadableSourceFile は E0602、SourcePathOutsidePackageRoot は E0603。E0601–E0603 の detail key は割当済み semantic name とする。その他は E0600 とし、detail key は `source.` と variant 名の snake_case、未知の将来 variant は未対応の integration gap として診断を捏造せず Blocking とする。message は表示専用。draft 作成・emit 失敗は Blocking。
`cache_key` は NoKey。normalized hash だけでは current file の raw loading map と cache の一致を証明できないため、毎回実 source を load する。codec は storage 用であり cache-hit scheduling や snapshot 間再利用を保証しない。

## Tests and remaining ownership

実 temporary file と planner/index/snapshot の出力を使い、resident/blob publication、current SourceId/map、同じ normalized text の異なる raw map、capture 後の text 変更、binding/resource 不正、cancel、stale publisher、work-unit 不許可を検証する。
実 invalid UTF-8、削除・読取不可 file、対応環境での symlink escape、allocator failure を shared sink 経由で検証する。後続 service 不足の blocking と既存 registry test を維持する。
preprocess/lex/parse/recovery、Frontend 全体の serialization と診断変換は frontend integration に残す。cache compatibility、LSP、artifact と後続 semantic/proof phase は既存 owner に残す。
