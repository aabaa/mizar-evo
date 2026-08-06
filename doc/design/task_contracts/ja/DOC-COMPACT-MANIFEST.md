# Task DOC-COMPACT-MANIFEST: data-driven legacy-compaction ledger

> canonical English:
> [../en/DOC-COMPACT-MANIFEST.md](../en/DOC-COMPACT-MANIFEST.md)。
> 本文書は同一logical taskの日本語companionである。

これはderived documentation/test-policy prerequisiteであり、language behavior、
test intent、diagnostic、public API、coverage creditを追加・上書きしない。今後の
legacy-evidence batchより先に、Rust-coded historical ledgerを一つのboundedかつ
reviewableなdata sourceへ置換する。

> live schema note: schema 2とcross-batch historical-task referenceは
> [DOC-COMPACT-MANIFEST-TASK-REF](./DOC-COMPACT-MANIFEST-TASK-REF.md)が所有する。
> 以下のschema-1 contractはfrozen historical implementation baselineとして残り、
> そのlater deltaは記述しない。

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-COMPACT-MANIFEST` |
| Status | implementation、independent reviews、required verification完了。exact task-only stagingとcommitが残る。 |
| Purpose | exact legacy-compaction inventoryをlint implementation code外へ移し、既存Task-269SDP/269SDC保証をbyte-for-byte保持し、後続bounded batchがtaskごとのRust test/ledger branchを追加せずdataだけを拡張できるようにする。 |
| Primary owners | repository documentation policyと`mizar-test` lint policy |
| Consumers | `task_contracts_are_recursively_paired_and_supported_links_resolve`、paired task contracts、owning crate-plan Task Index、全後続exact-section compaction batch |
| Dependencies | `DOC-269SD-COMPACT` commit `5080d3fddaad6e9683e5eecc5e497b4b16908e8a`とexact 82 redirects/12 index rows |
| Readiness | documentation prerequisiteはcommit `a16fee8fa7d059dd2d1930cca1ce067434b3ebe2`。そのfresh inventoryはclean、`origin/main...HEAD`は`0/1`、protected `stash@{0}`は`f65cf4a13752ec380710814a9ac6392ccb9d75d4`のまま。 |

[checker plan](../../mizar-checker/ja/00.crate_plan.md#task-index)と
[runner plan](../../mizar-test/ja/00.crate_plan.md#task-index)が本contractをindexする。

## Authority And Classification

authorityはrepository-wide documentation consolidationを優先するuserの明示決定、
[`AGENTS.md`](../../../../AGENTS.md)、および
[legacy migration policy](../../autonomous_crate_development.md#migration-policy)である。
language specification authorityは消費・変更しない。

| Class | Decision |
|---|---|
| `design_drift` | existing lintがTask-269SDP/269SDC ID、file groups、forbidden headings、redirects、Task Index rowsをRustへ埋め込み、policy codeが第二のhistorical ledgerになっている。 |
| `test_gap` | current exact checksはpassするが、later batchがtest name/countとpolicy implementationを安定させたままledgerを拡張するversioned data boundaryがない。 |
| `spec_gap` | 本derived policy taskにはない。historical semantic gapはscope外で不変。 |
| `source_drift` | production sourceにはない。later Rust deltaはaccepted repository stateが同一のtest-policy refactorだけ。 |
| `source_undocumented_behavior` | 導入・推測しない。 |
| `test_expectation_drift` | なし。semantic expectationを保護する。 |
| `boundary_violation` | manifestをtask-contract owner配下に置き、`mizar-test` lint policyだけがconsumeすることで回避する。 |
| `repo_metadata_conflict` | prior post-commit remote-tracking refは外部`update by push`で移動し、agentはpushしていない。current HEADと`origin/main`は一致するためreport-onlyでsafe commit targetを妨げない。 |

## Documentation-Prerequisite Scope

本prerequisiteはexact 6 Markdown files、すなわち本EN/JA pairとchecker/test EN/JA
crate plan各1 compact Task Index rowだけを変更する。policy text、Rust、manifest data、
production source、specification、test、fixture、sidecar、expectation、trace、Cargo、
count/status/hashを変更しない。dedicated commit後、fresh inventoryから下記implementationへ戻る。

## Frozen Manifest Contract

implementationはlanguage-neutral/versioned UTF-8 TSV file
`doc/design/task_contracts/legacy_compactions.tsv`をexact 1件追加する。lintにCargo
dependencyまたはhand-written partial TOML parserを追加しないためTOMLではなくTSVを
選択する。blank lineと`#`開始lineはignoreし、最初のnon-comment/nonblank rowとして
exact 1件の`schema<TAB>1`を必要とする。後続data rowはfield間にliteral tabをexact
1個使う。comment/blank除去後、`schema`後の全data rowsはrecord kindと全separator tabを
含むcomplete physical rowのunsigned UTF-8 byte昇順でなければならない。field内
tab/CR/LF、unknown record kind/version/column、duplicate identity、unsorted data row、
absolute path、`..` traversal、workspace外pathはfail closedする。

schemaは次のexact recordsからなる。

| Kind | kind後のfields |
|---|---|
| `schema` | version (`1`)、first data positionにexact 1 record |
| `batch` | batch ID、EN batch-contract path、JA batch-contract path、canonical expanded-inventory SHA-256、task count、redirect count、distinct source-file count、index-row count |
| `task` | batch ID、task ID、EN historical-contract path、JA historical-contract path |
| `redirect` | batch ID、task ID、language (`en`/`ja`)、source path、legacy heading level (`2`〜`6`)、exact forbidden legacy heading、exact replacement line、exact preceding same-or-higher-level headingまたは`BOF`、exact following same-or-higher-level headingまたは`EOF` |
| `index` | batch ID、indexed task/batch ID、language (`en`/`ja`)、owning `00.crate_plan.md` path、exact Task Index row |

全pathはslash-separated workspace-relative path、task/batch IDは
`[A-Za-z0-9][A-Za-z0-9._-]*`とする。`redirect`は一つのexact ATX heading sectionから
次の同level以上headingまでのwhole replacementを表す。mixed owner-local sectionと
paragraph-only migrationはschema version 1で表現せず、推測削除ではなく別review済み
schema extensionを必要とする。

legacy/neighbor heading fieldsは`#` levelとtextを含むexact raw ATX heading lineである。
fenced code外だけでheadingを認識する。declared legacy levelに対し、unique redirectから
外向きにscanしてnearest preceding/following raw ATX headingsのうちsame-or-higher levelを
anchorとする。intervening lower-level headingはlegacy sectionをterminateしない。同じraw
anchor textの反復は許容し、このnearest-heading ruleだけで解決する。その側にqualifying
headingが存在しない場合だけ`BOF`/`EOF`を使用できる。

batch IDはuniqueとする。task IDはmanifest batches全体でglobally uniqueで、各`task`は
exact 1 declared batchに属する。declared EN/JA batch/task contractsはpaired filesとして
存在し、declared IDをfile stem/title IDとし、対応する`task_contracts/en`/`ja` tree内に
なければならない。全`redirect` taskはそのbatchに属する。replacementはstandard central
completion-evidence sentenceであり、単なるexisting documentではなくtask recordが宣言した
exact language-local contractと`#completion-evidence`へresolveしなければならない。全
`index` IDはそのbatch内taskまたはbatch IDであり、exact EN/JA rowは対応するdeclared
language-local task/batch contractへresolveする。plan-path languageとrecord languageも
一致し、plan filenameはexact `00.crate_plan.md`でなければならない。

各batchのcanonical expanded inventoryは`schema`、`batch`、comment、blank lineを除外する。
batch-id fieldが当該batchと一致する全`task`/`redirect`/`index` rowについて、record kindと
literal separator tabsを含むcomplete physical UTF-8 bytesをwhole-row unsigned byte昇順で
sortし、各row（last rowを含む）の後へLFをexact 1個付けてconcatenateする。declared
SHA-256と全declared countはこのexact byte stringに一致しなければならない。これにより
self-referential hashを作らず、Rust sourceをledgerにせずreview surfaceをauthenticateする。
SHA-256はCargo dependencyおよびexternal executableなしでlint-policy test support内に
実装し、manifest inventory hashより先にstandard empty-input known-answer
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
へpassしなければならない。

## Frozen Lint Consumer

existing single test
`task_contracts_are_recursively_paired_and_supported_links_resolve`だけをconsumerとし、
test nameを保持する。implementationは以下を満たす。

1. strict schema parse、physical ordering、identity、relational membership/linkage、batch
   total、exact inventory hash validation。
2. referenced Markdown document/heading slug setのcache。
3. duplicate expanded key、`doc/design`内の全exact forbidden heading、missing/multiple
   expected redirect、unexpected central historical-contract redirectをreject。
4. 各redirectのexact source path、nearest preceding/following same-or-higher-level
   heading anchors、language-local file target、`#completion-evidence` fragmentを検証。
5. 各exact `index` rowを指定planの`## Task Index`内でexact 1回検証。
6. existing recursive EN/JA contract pair、title/marker、reciprocal contract link、
   owning-plan backlink、supported inline-link/fence/ATX-fragment検証を保持。

manifestがownするのは次のreserved grammarへ完全一致するfull lineだけである：
`Completion evidence: [central Task-<id> historical contract](<relative-md-path>#completion-evidence).`
または末尾Japanese `。`。`<id>`はtask-ID grammar、pathはsupported relative `.md`
targetとする。ここでunexpectedとは、このcomplete grammarへmatchするがmanifest
`redirect` recordがないlineだけであり、ordinary dependency/reference linkとpartial
proseはrejectしない。

implementationはcompleted `DOC-269SD-COMPACT`を最初にexact encodeする。historical
tasks 2、42 distinct source filesの82 redirects、12 Task Index rowsである。current
accepted repositoryは引き続きacceptedとし、schema/count/hash/path/heading/redirect/
anchor/index/link/fragment mutationは同じtest内のfocused parser/checker vectorsで
failしなければならない。targetは15 tests、raw list hash
`b044e771a655e72131d0371636bbac5684ef93a3ea503984537a4bb9dd13a7cf`を保持する。

same-test section-boundary vectorsは全H2〜H6 legacy level、`BOF`/`EOF`、sectionを
terminateしないlower-level heading、equal/higher-level terminator、repeated identical
raw anchor text、backtick/tilde fence内のheading-shaped lineをcoverする。digest vectorは
manifest mutation caseより先にempty-input known answerを含む。

implementation scopeはTSV manifest、existing lint-policy test file、`AGENTS.md`、
`doc/design/README.md`、autonomous protocol、およびcompletion status/evidence用の本
EN/JA pairだけ。new Rust testおよびCargo changeはない。

## Protected Boundaries And Deferrals

`doc/spec/**`、`.miz`、fixture、sidecar、expectation、
`tests/coverage/spec_trace.toml`、trace row/status/backlink、production Rust、public API、
diagnostic、parser/resolver output、active route、CLI result、executable coverage creditを
変更しない。別legacy sectionをcompactせず、historical task contractを追加せず、future
language semanticsを選択しない。Chapter-4/15 `set` disagreementとproof/goal/guard/fact/
obligation/capture questionsはscope外のまま。

`doc/design/spec_coverage_audit.md`はcoverage-status/ownership impactなしで変更しない。
next exact-section batchはmanifest implementation commit後のfresh inventoryだけで選ぶ。
current review evidenceはexact shared Task-269GUP/269GCT/269GCU familyを支持するが、
本taskはそのmigrationをpre-authorizeしない。consolidation programはstructureが安定する
まで`mizar-checker`-owned task familyを優先し、`mizar-test` docsは選択checker familyの
runner/harness/traceability/audit/plan-index/redirect consumerをownする場合だけ参加する。

## Documentation-Prerequisite Evidence

independent specification/policy/EN-JAおよびtest-contract reviewは、schemaのrelational
linkage、canonical bytes、H2〜H6 boundary、digest implementation、reserved grammarを
exact化した後、**NO FINDINGS**で終了した。diffはfrozen exact 6 Markdown pathsだけである。
full 15-test `mizar-test` lint-policy target、`cargo fmt --all --check`、Cargo metadata、
local link/fragment checks、protected-scope inspection、`git diff --check`はpassする。
specification、semantic test、production source、coverage audit、trace、Cargo file、
executable statusは変更していない。

## Implementation Evidence

- 7-path implementation surfaceは本EN/JA pair、TSV ledger、existing `mizar-test`
  lint-policy file、`AGENTS.md`、design index、autonomous protocolだけである。
  production、specification、fixture、sidecar、expectation、trace、Cargo、public API、
  coverage-audit pathは変更していない。
- 99-line TSVのphysical SHA-256は
  `c537eda8401c1cdc0a3386ca648d112075b0728b702b56d03f89e353d4a4347f`。
  one batch、two tasks、42 distinct source filesの82 redirects、12 index rowsを宣言し、
  independent replayはexpanded-inventory SHA-256
  `66087afe7a11a73aafeda4853dba2b684ef9edccae5fc014cc8fa01bb8265f8b`を再現する。
- Rust consumerにTask-269SDP/269SDCまたはbatch-specific ledgerは残らない。strict
  mutation vectors、document-evidence boundary vectors、fence-aware headings、canonical
  containment、non-symlink traversal、cached document/fragment validationはexisting single
  testを使う。lint targetは15 tests、raw list hash
  `b044e771a655e72131d0371636bbac5684ef93a3ea503984537a4bb9dd13a7cf`を保持する。
- independent test-sufficiency、implementation/security、source/document/EN-JA reviewsは
  全findings修正後 **NO FINDINGS** で終了した。
- focused/full lint policy、checker 530-test/runner 600-test libraries、137 metadata tests、
  checker lint、formatting、warnings-denied workspace Clippy、full `cargo test`がpassした。
  全5 CLI hashesは順に
  `700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`、
  `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`、
  `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`、
  `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`、
  `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`
  を保持し、protected trace hashは
  `55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`のまま。

## Reviews, Verification, And Exit

documentation prerequisiteはindependent specification/policy/EN-JA review **NO
FINDINGS**、全9 documentation gates PASS、score capなし`>=90/100`、exact six-file
staging、docs-only commit、clean post-commit HEAD/origin/stash inventoryを必要とする。

implementationはindependent test-sufficiency、implementation/security、source/
document/EN-JA consistency reviewをすべて **NO FINDINGS** にする。verificationは
focused malformed-manifest vectors、full lint-policy target、checker/runner libraries、
metadata tests、`cargo fmt --all --check`、warnings denied workspace Clippy、full
`cargo test`、全5 CLI、protected count/hash replay、exact current 82/42/12 acceptance、
`git diff --check`、cached/unstaged auditを含む。final read-only qualityは全9 gates PASS、
score capなし`90/100`以上を必要とし、implementationはtask-only commit、clean
inventory、unchanged stash、no agent-issued pushでexitする。
