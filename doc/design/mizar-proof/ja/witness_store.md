# mizar-proof Witness Store

> 正本: [../en/witness_store.md](../en/witness_store.md)。

## 目的

`mizar-proof` は、選択された trusted proof outcome の proof witness draft staging と
publication reference を所有する。

witness store は proof authority ではない。proof を accept せず、kernel を再実行せず、
ATP backend を実行せず、SAT solving を行わず、cache を query せず、artifact manifest
を書かない。すでに `mizar-kernel` が accept した evidence の byte と metadata を記録し、
安定した content hash を計算し、artifact publication boundary が witness の manifest
到達可能性を証明した後にだけ publication reference を公開する。

## 入力

store は、同じ obligation の deterministic status projection とも一致する trusted
selection に対する `ProofWitnessDraft` だけを消費する:

- 対応する accepted proof-obligation kernel result を持つ `KernelVerified`
  formula/substitution kernel evidence;
- `DischargedBuiltin` は internal staged outcome とし、下記の明示 file route も unpublished reference のみを持つ。

draft は次を持つ:

- `obligation_id`、`ObligationAnchor`、`obligation_fingerprint`;
- status projection と kernel metadata token から copy された selected winner class と
  selected evidence hash;
- witness payload schema family と schema version;
- canonical witness payload bytes;
- `ProofWitnessRef` version `2.0` が要求する kernel-acceptance metadata;
- verifier policy fingerprint と checker/evidence schema version;
- target binding、formula evidence、substitution evidence、provenance、optional formula
  context、accepted result、dependency artifact、build snapshot fingerprint を含む
  provenance metadata;
- non-trusted attachment のための optional diagnostics-owned provenance reference。

externally attested evidence、policy assumption、open obligation、rejected obligation、
no-selectable evidence、backend log、backend proof method、resolution trace、SMT proof object、
cache record、backend-reported axiom list は trusted witness draft を作らない。

## State Machine

witness handling には 3 つの状態がある:

1. `ProofWitnessDraft`: producer-owned bytes、trusted metadata、対応 status projection。保存・公開されず、それ自体で cache validation しない。
   metadata field は opaque、generic binder は crate-internal とし、限定 [Status token production](./status.md#source-theorem-trusted-witness-metadata-with-wire) と [Draft composition](./status.md#source-theorem-witness-draft) が fresh な検査済み metadata を供給する。
2. `ProofWitnessStagedRef`: generic `stage` は draft の path/hash/schema、obligation、selected class、provenance と `KernelVerified` の unpublished reference candidate を記録する。
   下記の明示 file route は検証した file profile/hash と unpublished builtin reference を記録し、どちらも committed reachability を与えない。
3. `ProofWitnessPublishedRef`: artifact publication boundary が staged tuple と一致し、
   committed main artifact から到達可能にする committed witness publication proof を
   供給した後にだけ `publish_ref` が返す。

artifact layer が witness bytes を write/hash できるよう、`stage` は artifact commit の前に
行わなければならない。`publish_ref` は committed manifest entry が同じ witness path、
obligation id、obligation fingerprint、witness artifact hash を参照し、module manifest
entry が同じ main `VerifiedArtifact` proof-witness set を参照した後にだけ行う。単独で一致
する witness tuple だけでは十分ではない。artifact-owned な
`CommittedWitnessPublicationProof` input は、witness entry を committed module artifact
entry または同等の committed `VerifiedArtifact` reference set に bind し、manifest の
`proof_witnesses` array が artifact の `proof_witnesses` collection をちょうど cover することを
示さなければならない。token は opaque のままで、caller による一致 tuple の合成は禁止する。
下記の限定 source-theorem constructor は公開 tuple でなく既存の opaque Artifact commit を
origin evidence とする。committed publication evidence なしの `publish_ref` は error となる。

`CommittedWitnessPublicationProof::from_source_theorem_manifest_commit` は実際の
`ManifestCommit`、root、完全な module identity、coordinator が期待する snapshot を受け取る。
全 flag を有効にした fresh manifest validation で commit hash と manifest 全体を照合し、
normal1.1 main と exact witness coverage の検証後、具体的な source-theorem candidate FILE の
hash/reference を検証する。空でない KernelVerified 集合を要求する。
期待 snapshot は既存の `publish_ref` が staged provenance と比較する値であり、commit 時の
snapshot receipt、現在の root/source の認証、lease ではない。token は検証時の過去の到達性を
保持し、新規構築では file を再検証する。後の変更は既存 token を変えず、reuse を認証しない。
通常の main hash 除外と同一内容の複製 root は有効であり、排他的な filesystem origin は認証しない。
他の payload profile と DischargedBuiltin publication は引き続き未対応である。

`publish_ref` は commit 後に新しい artifact reference を発明しない。`stage` が出した
unpublished reference candidate が committed `VerifiedArtifact` と manifest に記録された
同じ reference であることを検証し、その同じ reference に対する published wrapper を返す。

## Stable Hashing

generic `stage` の proof witness hash は、正確な staged payload bytes と payload schema identity に対する
artifact-framed hash である:

- schema family;
- schema version;
- canonical payload bytes;
- witness payload hash domain;
- selected accepted evidence hash;
- verifier policy fingerprint;
- obligation fingerprint。

hash には temporary file path、staging directory name、arrival order、backend completion
time、process id、wall-clock time、random data、cache-hit metadata、manifest commit timing を
含めてはならない。

2 つの draft が同一の payload bytes と同一の hash input を持つ場合、同じ witness hash を
生む。accepted evidence hash、policy fingerprint、obligation fingerprint、payload schema
version、payload bytes など trusted input が変わる場合、witness hash は変わるか staging
が reject される。

### Payload hash value

`ProofWitnessDraft::payload_hash_value(&self) -> CanonicalJson` は既存 hash encoder を変更せず、7 key の generic OBJECT の owned copy を返す。key は hash_domain、obligation_fingerprint、payload_bytes_hex、payload_schema_family、payload_schema_version、selected_evidence_hash、verifier_policy_fingerprint とする。既存 encoding/domain を保持し、caller の編集は draft を変更しない。I/O、validation、新 producer/schema は追加しない。[published schema](../../mizar-artifact/ja/store.md#schema-versions) に必須の top-level schema_version を持たず、実 file は [unreferenced/unpublished](../../mizar-artifact/ja/store.md#store-layout) のままである。この旧 profile は header なしのままとし、[別の file schema](#source-theorem-witness-file)でも変更しない。ここへの header/exclusion の追加、raw STRING/composite profile の代用、staging・Builtin/publication guard・acceptance・reuse credit の変更は禁止する。

既存 source_theorem_reflexivity_witness_draft_matches_captured_inputs の元の pure check 後だけを拡張し、従来の全 line/helper/fixture を保持する。実 native Disk/OpenBuffer capture の draft/wire から独立に全7 key の expected OBJECT を組み、新しい値、既存 Artifact writer/readback の全値・canonical 全 bytes、writer/read/stage digest と family/version/domain、replay、draft purity を比較する。既存 temporary root の unpublished-witness-hash-values subroot で draft path は変更せず、従来の no-file assertion を維持する。Builtin staged reference は None のままとし、committed metadata の捏造、新 helper、lower I/O matrix は禁止する。publication、currentness、Hit、CompleteSlice、広い gate 達成とはしない。

## Source algorithm state witness file
Statusの[生成](./status.md#optional-bounded-source-algorithm-state-witness-file)がFILE family mizar-proof/source-algorithm-state-witness-file1.0を供給する。OBJECTのkeyは厳密にschema_version、schema_family、obligation_id、obligation_fingerprint、payload_schema_family、payload_schema_version、payload_bytes_hex、kernel_acceptanceで、最後以外はstring。Artifact1.0 domain、除外なしでcanonical OBJECT全体をhash化し、generic七key Draft hashを流用しない。crate-private SOURCE_ALGORITHM_STATE_WITNESS_FILE_SCHEMA_FAMILYをproducer identity/hash、reader互換性、reader hashで共有し、旧producerはlocal family bindingだけを定数へ替えて他の全byte/出力を保つ。
元idとInterface/mizar-proof/source-algorithm-state-original-obligation1.0 fingerprintを使う。唯一のpayload profileはmizar-proof/source-algorithm-state-transitivity-checked-wire2.0で、全payloadを非空・偶数長・小文字ASCII hexで表す。kernel_acceptanceは必須nullable formula_context_hashを含む全10fieldの[Artifact metadata](../../mizar-artifact/ja/proof_witness.md#conceptual-shape)で、evidence_schema_version2.0。参照は実path/identity/全metadata/FILE Artifact hashの明示Ref3.0/DischargedBuiltin/FormulaSubstitutionKernelEvidenceとする。
`read_source_algorithm_state_witness_file(&PublishedArtifactRead, &ProofWitnessRef) -> Result<Vec<u8>, String>`を完全なtheorem-reader doccomment前に一つinline追加する。OBJECTを要求し、最初にpath付きSchemaVersionSupport(family,1,0,UpToSupported)で、missing/non-string headerを旧theorem reader同様に扱う。その後family完全一致、厳密key/型、唯一payload組合せ、安全な全hex復元を検査する。byte-offset sliceより前に空/奇数長/大文字/非hex/非ASCIIを拒否し、migration/forward-minorを宣言しない。
state FILE全体のhashを除外なしに再計算し、callerがcopy可能なread.artifact_hashへSome(実再計算値)を要求する。固定3.0/discharged_builtin/formula_substitution_kernel_evidence、実FILE id/fingerprint/metadata/read.path/再計算typed hashから非信頼RefJSONを組む。実pathとexpected.witness_artifact_hashで不変read_builtin_proof_witness_refを呼び、返る参照全体=expected、元Interface/state family1.0、evidence2.0を要求する。全decoded bytesだけを返し下位Stringを保つ。expectedからclassを選ばずmetadataを手動parseしない。
固有errorはsource algorithm state witness file must be an object、invalid source algorithm state witness file fields、source algorithm state witness file {key} must be a string、source algorithm state witness file kernel_acceptance must be an object、invalid source algorithm state witness file payload profile、invalid source algorithm state witness file payload hex、source algorithm state witness file read hash mismatch、source algorithm state witness file reference mismatch。family不互換は旧theorem形式でfamily/実version/対応範囲/artifact pathを示す。この順序と下位schema/codec Stringを保つ。
非信頼データの構造的対応だけを検査し、Kernel evidence/source受理/snapshot・filesystem currentness/publicationを検査しない。profile field/metadata/hash/referenceの整合した編集・relabelは任意bytesを復元し得る。既存codec/profile guardを越えて任意metadata digest/checker version/contextを制限し認証を捏造しない。別の[state staging対応付け](#source-algorithm-state-file-staging)だけが供給実Draftを照合し、reader成功だけでは対応付けを与えない。enclosing Artifact/commit/token/publication/reuse/Hit/Driver/登録/SAME/一般finder/Steps6–8は保留し、theorem reader/stager、generic stage refNone、Builtin publish拒否、raw Open/Deferred/incomplete/MISSを保つ。
### Source algorithm state witness file reader tests
最終term_proof_tests brace前にsource_algorithm_state_witness_file_reader_matches_genuine_transportとsource_algorithm_state_witness_file_reader_rejects_profiles_and_associationsだけを追記し、不変native Disk helperを使う。reader Rust前に真正72全triple/file/ref、実canonical Artifact transportの全bytes/LF/domain/digest、FILE field由来の独立非信頼RefJSONによる旧Builtin readerを示す。期待する全wire bytesは真正Draftから直接得て、72の独立FILE hex符号化を継承しreaderのhex decoderをテストへ複製しない。最終必須の既存72回帰familyが実71全tuple対応/全FILE・metadata直列化/theorem拒否/generic stage refNone/raw/publicationを保持し、71意味論/Kernel matrixを継承する。新familyは小さな真正source/path/policy変化とreader・read/referenceの反復/入力不変を確認し、新module helperやcheck/snapshot/Accepted/token偽造を加えない。負例familyでは三つ以上の異なる実呼出し箇所の既存Artifact write/readだけを共有するlocal I/O専用closure一つを許す。decode/profile検査/hash再解釈/error oracleを含めない。
負例は実72生成物由来で実canonical transportを使い、深いguardへ到達するためだけにexpected read digest省略または通常FILE/ref hashの整合更新をする。実SchemaVersionSupportで不正内容より先の互換性、OBJECT/family/missing/unknown/型/profile/空・奇数・大文字・非hex・非ASCII hex、代表nested metadata下位拒否、read hash不在/別domain/読後value編集、別safe・非proof path、参照全体/id/policy/metadata/path/hash/class/version/kind不一致を扱う。整合した誤original family/version/evidence versionは先に下位成功を示してprofile拒否を確認する。Artifact全malformed metadata/codec/I/Oと72 optional source errorを継承しreader adapter複製をしない。
phase1は既存下位基準だけを実行し、実baseline成功後のphase2でこの文書のprofile/errorに基づく新reader byte/error確認を追加する。明示する正の構造上限: 実非信頼FILE/refをpayload hex00へ整合編集・再hashして実transportに通すとreaderは[0]を返すが、元実Draft/wireは不変かつ異なる。これは非信頼データの復元であって新Acceptedではない。最終focused検査では既存72二familyも実行し、共有family bindingがproducer全出力を保持することを示す。

## Source theorem witness file
Status の[実生成](./status.md#source-theorem-witness-file-candidate)が、schema_version、schema_family、obligation_id、obligation_fingerprint、payload_schema_family、payload_schema_version、payload_bytes_hex、kernel_acceptance の8 key の別 top-level OBJECT を作る。kernel_acceptance 以外は string。file family は `mizar-proof/source-theorem-witness-file`、version1.0、Artifact class、全 canonical OBJECT を除外なしに hash する。crate-private SOURCE_THEOREM_WITNESS_FILE_SCHEMA_FAMILY は生成 hash・読戻し互換性・読戻し hash で共有し、公開 schema 型/helper は作らない。
元の実 obligation id と Interface fingerprint family `mizar-proof/source-theorem-original-obligation`1.0 を使う。payload は同じ検査の非空・小文字・偶数長ASCII hexで、厳密に二つのprofileだけを許す。`mizar-proof/source-theorem-reflexivity-checked-wire`2.0 はBuiltin/ref3、`mizar-proof/source-theorem-candidate-checked-wire`2.0 はKernelVerified/通常ref2。[candidate生成](./status.md#source-theorem-candidate-witness-file)も既存8key/file1.0を使い、他のfamily/version組合せは拒否する。kernel_acceptance は既存 [Artifact metadata OBJECT](../../mizar-artifact/ja/proof_witness.md#conceptual-shape)そのもので、formula_context_hash 不在は明示 null、evidence_schema_version は2.0。旧7 key hash、raw STRING/composite identity/ref と混同しない。
`read_source_theorem_witness_file(&PublishedArtifactRead, &ProofWitnessRef) -> Result<Vec<u8>, String>` は構造データだけを検証する。OBJECT を要求し、path付き SchemaVersionSupport(family,1,0,UpToSupported) で schema_version を最初に確認し、次に schema_family の完全一致（不互換は family・実 version・対応範囲・path を示す）、正確な key/型、厳密に許可されたpayload family/version、安全なhex復元を検査する。migration/forward-minor は宣言しない。byte slice より先に非ASCII、空・奇数長・大文字・非hex を拒否する。
全 file hash を新 domain で再計算し、read.artifact_hash=Some(再計算値) を要求する。許可された実fileprofileだけからclass/versionを選ぶ。reflexivity2.0はBuiltin/ref3/read_builtin_proof_witness_ref、candidate2.0はKernelVerified/ref2/通常read_proof_witness_ref。同じfile元identity/metadata/read.path/再計算typedhashと選択class/versionの非信頼JSONを組み、実pathとexpected.witness_artifact_hashで対応する未変更Artifact readerを呼び、返る ref 全体と expected の一致、および元 fingerprint/evidence version の profile 一致を要求する。wire bytes だけを返し、下位 error は String で伝播する。kernel evidence の parse/replay、受理、確定公開 token は作らない。
canonical parse・duplicate key/UTF-8/path/I/O integrity・ref/metadata 検証は Artifact が所有する。Builtin stage/publication guard、status/cache gap、不完全 slice は不変で、別の committed publication owner ができるまで新 file/ref は unreferenced である。
### Source theorem witness file tests
既存 term_proof_tests に source_theorem_witness_file_matches_genuine_artifact_transport と source_theorem_witness_file_rejects_invalid_schema_and_associations だけを追記し、旧全 byte/import/helper/fixture を保持する。実 native Disk/OpenBuffer・Frontend・元 resolver 入力を使う。新しい各成功の前に未変更の元ソース draft 取得を独立に呼び、triple 全体と独立に組んだ8 key/file/metadata/hash/ref を比較する。実 Artifact write/read の canonical 全 byte・hash/domain・復元 wire・反復・入力不変性・実 source/policy 変化と、元 Open/incomplete/miss/publication gap/stageNone を確認する。
負例は実生成物から作った非信頼データを実 Artifact transport に通す。内容検証より先の互換性、family、unknown/missing/型、hex、metadata、別 identity/policy/path/ref/hash、read hash 不在/不一致を扱う。schema error に到達する generic read の expected hash 省略は authority ではない。実上流 error/優先順は旧取得と比較し、下位 parser/I/O/kernel 行列を複製せず、架空の正の Accepted/token/receipt/complete slice を作らない。構成不能な guard は静的に確認する。

## Source algorithm state file staging
`ProofWitnessDraft::stage_source_algorithm_state_file(self, read: &PublishedArtifactRead, expected: &ProofWitnessRef) -> Result<ProofWitnessStagedRef, String>`を完全な旧theorem-stager doccomment前に一つinline追加しselfを消費する。差替wire/hash/metadata/tokenを受け取らず、他の旧production/test/helper全byteを保つ。
最初に不変read_source_algorithm_state_witness_file(read, expected)を呼び、Draft不一致検査より先に実Stringを保つ。reader成功は下位検証と参照全体一致によりexpectedがproof_status DischargedBuiltinを含む固定Builtin Ref3であることを保証するので委譲する。selfの(selected_class,payload family)は厳密に(DischargedBuiltin,mizar-proof/source-algorithm-state-transitivity-checked-wire)だけを許しwildcard/theorem代替を作らない。不変の二profile theorem stagerは独自class/profile/status照合を保つ。
obligation id、typed fingerprint全体、witness path、全kernel_acceptance、payload version2.0/canonical_bytes_required=true、復元全bytes=self.payload_bytes、expected_selected_proof_witness_hash=Noneを要求する。固有不一致はsource algorithm state witness file staging mismatch。anchor field/hashの発明、evidence/metadataのparse/replayはせず、実anchorとchecked-wire authorityはproducer所有のままとする。
続けて不変stage(self)を一回呼びerrorをString化する。staged.payload_schemaを共有state FILE family1.0/canonical=true、staged.staged_payload_hashをexpected.witness_artifact_hash.clone()、staged.artifact_reference_candidateをSome(expected.clone())にだけ変更し、他のgeneric staged全field/全provenanceをmoveで保持する。generic七key hash/profileとcanonical FILE hash/profileを区別しsource projection/selected witness hashを変えない。
供給Draftとの未公開の構造的対応であり、任意public Draft payloadがKernel受理済みという認証ではない。Source/Kernel replay、I/O、source/snapshot/filesystem currentness、enclosing Artifact/manifest/commit/token/publication/cache Hit/Driver/登録/SAME/一般finder/Steps6–8を行わない。classはBuiltinを保ち、committed proof検証より先の無条件publish_ref拒否とprivate token構築を保持する。raw不完全/MISS/publication gapを継承する。
### Source algorithm state file staging tests
最終term_proof_tests brace前にsource_algorithm_state_witness_file_staging_matches_genuine_associationとsource_algorithm_state_witness_file_staging_preserves_reader_errors_and_rejects_mismatchだけを追記する。不変native Disk state helper、実72生成物、実canonical Artifact write/read後の実73 reader成功、実generic stage基準を新method Rust前に得る。保持する全staged getter/全provenanceと変更するschema/hash/全参照の三fieldだけを比較し、小さな真正source/policy/path変化、新method反復とread/reference/product不変を確認する。負例familyに三つ以上の実呼出し箇所のwrite/readだけを共有するlocal I/O専用closure一つを許し、decoder/profile/hash再解釈/error oracleを含めない。新module helper/type/fixture/偽Accepted/check/snapshot/tokenを作らない。
真正source/policy/path間の対応違いは、先に実reader成功と関連する実差異を示してから固有不一致を比較し、snapshot間hash安定を仮定しない。通常の非信頼wirehex00/id/fingerprint/policyまたはaccepted metadata編集は実transport/再hash/整合参照を使い、reader成功（全元bytesまたは[0]）の後に真正検査済Draftとの対応拒否を示す。代表的なread hash/schemaと不正内容/expected ref/pathは実foreign Draftと組み、reader-FIRSTと実下位Stringを比較する。73の全reader matrixを継承する。
不変with_source_theorem_file_staging_inputsから真正foreign theorem Builtin Draftを得て、state reader成功の後の固有拒否を検査し、旧theorem stagerによる実state FILE拒否も保つ。真正KernelVerified Draftは既存candidate factoryで構築可能だが、このsliceに新native candidate captureは含めず、既存実class/profile matrixを継承し厳密self pairと委譲reader事後条件を独立静的reviewする。foreign Builtin例は別selected classを検査しない。KernelVerifiedを構築不能と呼ばず、無関係token/snapshotで受理済state Draftを捏造しない。
許可した真正state factoryはversion2.0/canonical=true/selectedHashNoneのみを公開しsetterがない。他のgeneric値が常に構築不能とはせず、private field注入や受理済state payload再構築なしに厳密guard/constructorを静的検査する。private committed tokenは外部harnessで構築できず既存private unit公開guardを継承する。最終共通source_algorithm_state_witness_file_検査は既存72二件+73二件+新staging二件を必須とし、producer/Kernel/serializer/reader/raw/generic stage/theorem/publicationを継承する。

## Explicit source theorem file staging
`ProofWitnessDraft::stage_source_theorem_file(self, read: &PublishedArtifactRead, expected: &ProofWitnessRef) -> Result<ProofWitnessStagedRef, String>` は既存 draft を消費し、bytes/metadata と別 file profile を構造的に対応付ける。draft 最初の getter 直前に inline method 一つを加える。
最初にread_source_theorem_witness_file(read, expected)を呼びerror Stringを保つ。実draft class・実payload family・検証済expected.proof_statusの三者を、(DischargedBuiltin, reflexivity-checked-wire, DischargedBuiltin)または(KernelVerified, candidate-checked-wire, KernelVerified)に厳密限定する。familyは上記完全mizar-proof/source-theorem-名を使う。元obligation id/fingerprint/path/全metadata一致、payload version2.0/canonical=true、復元bytesと全draft payload一致、expected selectedhash不在も保持する。draft/fileのprofileを独立に認めるだけでは整合profile relabelを許すため、検証済ref statusも照合する。不一致は内部 source theorem witness file staging mismatch とする。
既存 stage(self) を呼び error String を保つ。payload_schema を SOURCE_THEOREM_WITNESS_FILE_SCHEMA_FAMILY1.0/canonical=true、staged_payload_hash を expected.witness_artifact_hash、artifact_reference_candidate を Some(expected.clone()) にだけ置き換え、他の staged field は全て保つ。旧 generic hash/wire profile と別 canonical-file hash/profile を同一視せず、元 draft の再 hash/再解釈/上書きをしない。
これは unpublished structural association であり任意の caller draft bytes が Kernel 検査済みという認証ではない。checked-wire authority は実 source producer に残す。evidence の parse/replay、Kernel、I/O、filesystem/snapshot currentness 認証は行わない。generic stage、builtin publish_ref 拒否、status/reuse gap、raw 不完全 slice/MISS を保ち、committed token/witness publication/受理/enclosing gate credit を与えない。
### Explicit source theorem file staging tests
既存 term_proof_tests 最後に source_theorem_file_staging_matches_genuine_file_and_legacy_boundaries、source_theorem_file_staging_rejects_genuine_cross_associations、source_theorem_file_staging_preserves_reader_refusals_and_inputs と、それら三つの実 family が使う private with_source_theorem_file_staging_inputs 一つだけを追記する。全 old bytes/helper/fixture を保つ。
新 API 前に実 native Disk/OpenBuffer capture、元 unaugmented Resolver inputs、既存 source file-candidate tuple 全体、generic stage、実 Artifact write/read/recovery を得る。保持する staged 全 field、別 file schema/hash/reference、反復/不変、元 status/slice 全体/publication/MISS を比較する。実 old 値を使い、新 hash grammar や編集間安定性を仮定しない。
実 source/policy/path の別 products では先に reader 成功を示してから対応違いを拒否する。実 file 由来の coherent rehash 付き untrusted wire/identity/metadata 編集と実 lower-reader failure も使い、事前の lower String と比較し実拒否条件を示す。架空 positive draft/check/Core/VC/Accepted/token/commit や下位 matrix 重複を作らず、構築不能 trusted-class/selected-hash guard は静的 review とする。

Candidate fileと両方向の構造reader成功・実Draft stage拒否のprofile relabelテストは[候補fileの所有者](./status.md#source-theorem-candidate-witness-file)に置く。旧Builtin正負例は全て保持し、reader/helper/type/default/別文法/受理/実commit/tokenを追加しない。

## Publication References

`KernelVerified` formula/substitution evidence では、`stage` が次を持つ unpublished
`ProofWitnessRef` version `2.0` candidate を準備する:

- `proof_status = "kernel_verified"`;
- `evidence_kind = "formula_substitution_kernel_evidence"`;
- staged witness path と witness artifact hash;
- accepted proof-obligation kernel evidence boundary から copy した kernel acceptance
  metadata。

`publish_ref` に成功すると、committed manifest reachability が証明された後にだけ、その同じ
reference を publication-reachable として返す。

store は non-trusted status の `ProofWitnessRef` を publish してはならない。unsupported
trusted status を `kernel_verified` に書き換えてもならない。

`DischargedBuiltin` publication は `external_dependency_gap` を保ち、実 Artifact 所有 committed reachability が供給されるまで `publish_ref` は拒否する。
Generic `stage` は internal hash/reference absent を保つ。明示 file route は別 file hash の unpublished reference3.0 のみを保持してよい。
どちらも selected_proof_witness_hash を供給せず、必須 deterministic-discharge reuse input を置き換えず、status/publication/cache authority を昇格しない。

## Provenance Metadata

staged record と published record は、diagnostics と reuse validation に必要な provenance を
保持する:

- build snapshot と producer identity;
- selected candidate id と selected winner class;
- kernel evidence origin;
- target VC fingerprint と obligation fingerprint;
- dependency slice と dependency artifact fingerprint;
- verifier policy fingerprint;
- checker と evidence schema version;
- 利用可能な場合の accepted result hash と trusted used-axiom reference hash;
- advisory backend data の diagnostics-owned reference。

trusted value が存在する場合、provenance metadata は selected status projection、kernel
acceptance metadata、trusted used-axiom token と一致しなければならない。provenance
metadata は trust boundary を広げない。dependency artifact fingerprint は task 11 では
producer-owned reuse metadata として保持される。committed dependency artifact への
authoritative binding は artifact/cache integration まで deferred である。backend log、
externally attested citation、cache record、diagnostic hint は diagnostic または
reuse-validation material のままである。

## Failure Semantics

store は次を reject するか gap として報告する:

- unsupported witness class または evidence kind;
- missing または mismatched accepted evidence hash;
- status projection と kernel acceptance metadata に一致しない draft selected class、
  selected evidence hash、obligation identity、policy fingerprint、selected candidate id、
  kernel evidence origin、schema version、trusted `used_axioms` reference;
- malformed payload schema identity;
- payload schema が canonical bytes を要求する場合の empty payload bytes。byte-level の
  canonicality validation は concrete producer-owned schema validator まで deferred;
- `proof-witnesses/` から escape する witness path;
- staged bytes と manifest reference の hash mismatch;
- 1 つの obligation に対する duplicate manifest reference;
- matching committed witness publication proof が存在する前の `publish_ref`;
- committed main artifact に bind されていない、または `VerifiedArtifact.proof_witnesses`
  collection をちょうど cover しない manifest witness entry;
- stale build snapshot または mismatched obligation fingerprint;
- externally attested、assumed、open、rejected、no-selectable evidence を trusted witness
  material として publish しようとすること。

failure は deterministic diagnostic または typed store error である。trusted proof status に
ならず、trusted `used_axioms` も合成しない。

## Cache And Reuse Boundary

publish 可能な `KernelVerified` witness について、selected witness hash と published
witness ref は proof-reuse validation に参加するが、proof authority ではない。cache record
は、witness hash、selected evidence hash、obligation fingerprint、policy fingerprint、
schema version、accepted proof-obligation kernel metadata が現在の validation predicate と
すべて一致する場合だけ proof を reuse してよい。dependency artifact fingerprint は、
downstream artifact/cache owner が authoritative binding を供給した後にだけその predicate
の一部になる。task 11 では producer-owned reuse metadata として保持する。selected
candidate は、現在の
artifact witness schema が参照できる witness payload artifact hash
（`witness_artifact_hash`）を持つ場合、`selected_proof_witness_hash` を保持してよい。
その hash は `ProofWitnessRef` metadata object の hash ではなく、committed publication proof
でもない。committed `ProofWitnessPublishedRef` は、artifact boundary が matching committed
witness publication proof を供給した後に `publish_ref` からだけ得られる。artifact support
が存在するまで、`DischargedBuiltin` reuse は `deterministic_discharge_hash` を使い続ける。
internal staged hash は selected proof witness hash ではない。cache hit は witness を
publish できず、non-trusted status を昇格させられず、trusted `used_axioms` も作れない。

## Deferred And External Dependency Gaps

| Gap | Classification | Handling |
|---|---|---|
| `WITNESS10-G001` | `external_dependency_gap` | `mizar-artifact` `ProofWitnessRef` version `2.0` は現在 `kernel_verified` formula/substitution evidence だけを受理する。`DischargedBuiltin` witness publication は unsupported のままであり、collapse してはならない。 |
| `WITNESS10-G002` | `deferred` | [State Machine](#state-machine) の限定 source-theorem candidate 経路が committed exact coverage を供給する。汎用 publication integration は deferred であり、この module は manifest を書かない。 |
| `WITNESS10-G003` | `deferred` | formula/substitution witness bytes 以外の concrete payload schema は producer-owned のままである。store は schema identity と bytes を hash するが backend proof payload は解釈しない。 |
| `WITNESS11-G001` | `external_dependency_gap` | opaque Artifact commit は [限定 source-theorem constructor](#state-machine) だけを可能にする。他の production token profile は未提供であり、一致する公開 tuple は commit origin を証明しない。 |
| `WITNESS11-G002` | `external_dependency_gap` | 汎用 caller-metadata 受入れは未提供。限定された検査済み token の経路は [State Machine](#state-machine) が所有する。 |
| `WITNESS11-G003` | `deferred` | task 11 は payload schema が canonical bytes を要求するかを記録し、その attestation 下では empty payload を reject するが、byte-level canonicality は concrete payload schema が validator を公開するまで producer-owned のままである。witness hash は引き続き exact bytes と schema identity を覆う。 |
| `WITNESS11-G004` | `deferred` | task 11 は `dependency_artifact_fingerprint` を producer-owned provenance として保持する。committed dependency artifact と cache validation への binding は artifact/cache integration まで deferred であり、それ単体で witness status を昇格してはならない。 |

## Public Enum Policy

task 14 は public-enum forward-compatibility procedure をこの module に適用する。
public witness-store error enum は downstream-facing API surface であり、
`#[non_exhaustive]` を維持しなければならない。downstream consumer は wildcard match
arm を保つ。新しい error variant には paired spec/test review が必要であり、opaque な
publication-token boundary や trusted-metadata boundary を弱めてはならない。

| Enum | Compatibility decision |
|---|---|
| `ProofWitnessStoreError` | forward-compatible |

No exhaustive public enum exceptions are owned by this module.

## Non-Goals

witness store は proof search、premise selection、substitution invention、ATP/SAT backend 呼び出し、
kernel 呼び出し、cache query、artifact manifest write、placeholder witness reference publication
を行わない。
