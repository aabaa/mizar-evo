# Task DOC-LEGACY-COMPACTION-LIFECYCLE-REPAIR: registered batch lifecycle repair

> canonical English:
> [../en/DOC-LEGACY-COMPACTION-LIFECYCLE-REPAIR.md](../en/DOC-LEGACY-COMPACTION-LIFECYCLE-REPAIR.md)。
> 本文書は同一logical taskの日本語companionである。

これはderived documentation-maintenance contractであり、language behavior、test
intent、diagnostic、public API、active behavior、semantic/coverage creditを追加・
上書きしない。

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-LEGACY-COMPACTION-LIFECYCLE-REPAIR` |
| Status | implementation、independent reviews、required verification完了。exact task-only commitとclean postcommit proofが残る。 |
| Purpose | 全schema-2 registered compaction batchのlive status fieldを、既にcommittedなmigrationと整合させ、全historical checkpointを保存する。 |
| Primary owners | repository documentation policyとchecker/test temporary consolidation gate |
| Consumers | paired batch contracts、checker/test crate plans、schema-2 ledger consumer、successor inventory agents |
| Dependencies | current clean HEAD `9e40f3cfa2d0a0bbd50784efffb71e61aeee4293`、32 registered migration histories、schema-2 ledger/lint support、C4C4 closeout `7b53784a6f2525ebb35ce8d59230f07d1c9041bf` |
| Readiness | unique。全registered EN/JA batch pairにstale top-level live `Status`が1件あり、committed migration、redirect、ledger row、owner link、historical evidenceはその他すべてconsistent。 |

[checker plan](../../mizar-checker/ja/00.crate_plan.md#task-index)と
[runner plan](../../mizar-test/ja/00.crate_plan.md#task-index)が本contractをindexする。

## Authority And Classification

ordering authorityは [`AGENTS.md`](../../../../AGENTS.md)、
[autonomous crate protocol](../../autonomous_crate_development.md#migration-policy)配下の
[temporary consolidation gate](../../todo.md)
である。language authorityを再解釈しない。

| Class | Decision |
|---|---|
| `design_drift` | 32 registered EN/JA batch pairsすべてのlive statusが、migrationはcommit/registration済みにもかかわらずstaging、commit、clean replayが残ると記す。 |
| `repo_metadata_conflict` | current checkoutにはない。HEAD、remote `origin/main`、local tracking refは同一commit。historical remote movementはtime-local evidenceとして保持しrepairしない。 |
| `spec_gap`, `test_gap`, `source_drift`, `source_undocumented_behavior`, `test_expectation_drift`, `boundary_violation` | lifecycle-only taskが導入・repairするものはない。 |

## Read-Only Reconciliation Baseline

selection時worktreeは`9e40f3cfa2d0a0bbd50784efffb71e61aeee4293`でclean、remote
`origin/main`も同一commit、`origin/main...HEAD`は`0/0`。protected stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`は存在し、不変とする。

schema-2 ledgerは1,024 physical lines、SHA-256
`2a66d200a1976861600bcf7686388faa3efb19b2b42a43c756c9e689d7f27359`、exact
batch/canonical-task/task-reference/redirect/index cardinality
`32/44/4/638/304`。recursive paired-contract/ledger/link/fragment/anchor/count/
expanded-hash lintはpassする。下表の全migration/registration commitはselection HEADの
ancestorである。

| Batch | committed migrationまたはregistration evidence |
|---|---|
| `DOC-247-COMPLETION-COMPACT` | `75d8af2d5e071f415d1cada9e1a8981aaef2d3b2` |
| `DOC-248P-DOC-REVIEW-COMPACT` | `bee5a905c3e0b291018a33165b382d14bb5eb9fd` |
| `DOC-249M-ACTIVE-EVIDENCE-COMPACT` | `331fdc055d9416225ccc6e2acb22d199c17cb8ee` |
| `DOC-249PI-DOC-REVIEW-COMPACT` | `6b139bf1ab37cdc6c0d7239d202802db1efe113f` |
| `DOC-249S-ACTIVE-EVIDENCE-COMPACT` | `cbacea8efa0c7ac60f16636c2932c49b877e3eae` |
| `DOC-258AB-COMPACT` | `5a83db6f82aa789e31b00601e66d57fe4cda2601` |
| `DOC-258B3M1-M2A-IMPLEMENTATION-LEDGER-COMPACT` | `a9435046608eeb69c8ac284c65b069729d62cab2` |
| `DOC-258B3M2B1-B2A-IMPLEMENTATION-LEDGER-COMPACT` | `e9465ba0ffabf78544cc9ad5663c2d999b6898bf` |
| `DOC-258B3M2B2B1B1P-B1B1-IMPLEMENTATION-LEDGER-COMPACT` | `7467fdc1601479d62002a4e16ee7a07a368519ad` |
| `DOC-258B3M2B2B1P-B1A-IMPLEMENTATION-LEDGER-COMPACT` | `4c030c9d66245439c28ec7659d624aefe414494f` |
| `DOC-258B3M2B2B2C-FINAL-REVIEW-COMPACT` | `9b356722d29c26ffc1ba5e927112555ead51babb` |
| `DOC-258B3M2B2B2C-IMPLEMENTATION-LEDGER-COMPACT` | `b91ca9cfe9eb4789045eda271db8160c226e3133` |
| `DOC-258B3M2B2B2C-RUNNER-IMPLEMENTATION-COMPACT` | `7f771af69cb2ffed9d9c7f784c5b723c7f22b977` |
| `DOC-258B3M2B2B2CP-IMPLEMENTATION-COMPACT` | `0343f8e7ef47d6b24a64e8b14b3a85f600a95380` |
| `DOC-258B3M2B2B3-ACDE-COMPACT` | `9c31231eae4a0bb1cff9d6bb037ab030eb2d5fef` |
| `DOC-258B3M2B2B3P-REVIEW-COMPACT` | `787c16fb682db58f2a9fddc0d3f9aee1f9fd22bf` |
| `DOC-258B3N-IMPLEMENTATION-LEDGER-COMPACT` | `b4f97b2ea5f9bba17bf084929214b749389b08b9` |
| `DOC-258B4A-COMPACT` | `fee14f18c2301b1523250f25843d96b91f759b8e` |
| `DOC-258B4B-COMPACT` | `1d32ed06cc110ed98e9116dd59af82e9ef724b15` |
| `DOC-258B4C-DOC-REVIEW-COMPACT` | `d94dfd6330c1dd067be8b26c814ac95e077b2639` |
| `DOC-258B4C-IMPLEMENTATION-LEDGER-COMPACT` | `71edf3400bd8da556322c0510d6824bb62302c60` |
| `DOC-258B5A-COMPACT` | `95b4ce9801bc0b5ec85dbdba30d40ec26d44d3d7` |
| `DOC-258B5A-IMPLEMENTATION-LEDGER-COMPACT` | `ada9f5a3c773dc59687462dbd2a0be72bee03157` |
| `DOC-258B5B-UPPER-IMPLEMENTATION-LEDGER-COMPACT` | `440d27ae6e42f0aef6a58578a643ec5461763af3` |
| `DOC-260-DOC-REVIEW-COMPACT` | `9451e57df52dc105a3faa2348432e3d81642519a` |
| `DOC-269A-DOC-REVIEW-COMPACT` | `a9d5f40650d2ed694ba9304e2448fbd95e272406` |
| `DOC-269B-DOC-REVIEW-COMPACT` | `1ad52ed39cfa98d9a9b08f639e2d75f123de80cf` |
| `DOC-269CTGP-COMPACT` | `f77f68f9b0bd48c681396afb4125cba343a294a8` |
| `DOC-269G-COMPACT` | `34b42908fcc3a7734200e962878dca02b6dafe8f` |
| `DOC-269G-INTERMEDIATE-COMPACT` | `f3dd80bc396d17a76d8bf127f34b2e9f519999c7` |
| `DOC-269GT-COMPACT` | `a1bf34e86b42b19a81cf7ca07bb1e420a266637f` |
| `DOC-269SD-COMPACT` | migration `5080d3fddaad6e9683e5eecc5e497b4b16908e8a`、later ledger registration `0ec5fce293a6105e04761c5298b605d3f4ff60ca` |

`DOC-269SD-COMPACT`はdata-driven ledgerより前に完了し、later backfillはreview済み
generic-ledger implementationであるため、distinct migration/registration commitは
metadata conflictでなくconsistent。他31 batch rowはlisted task-local migration
commitで追加された。

## Frozen Repair Surface

implementationが変更するcurrent-state ownerはexactに次だけである。

1. 32 registered English batch contractsと32 Japanese companionのtop-level `Status`
   field各1件。
2. 本paired contractとchecker/test EN/JA Task Index各1 compact row。
3. `doc/design/todo.md` temporary-gate checklist items 1/2だけ。

exact English replacementは次である。

```text
| Status | Complete. The migration is registered in the schema-2 ledger; task-local completion evidence below preserves the committed migration and clean replay. |
```

synchronized Japanese replacement valueは次である。

```text
完了。migrationはschema-2 ledgerに登録済みであり、task-local completion evidenceがcommitted migrationとclean replayを保存する。
```

existing field labelは保存する。したがってJapanese 31 pathsは
`| Status | <replacement value> |`を使い、
`doc/design/task_contracts/ja/DOC-258B4B-COMPACT.md`だけはhistorical lowercase
labelを保存して`| status | <replacement value> |`とする。これらrowの他byteには
frozen replacementからの例外を認めない。

taskは71 pathsを変更する。64 registered batch contracts、本EN/JA pair、4 crate
plans、top-level roadmap。task-contract countsは`95/95`、`doc/design` Markdown filesは
824となり、64 stale live status valueはfield-label spellingを保存したまますべて
completeになる。ledger rowは追加せずbyte identical。
`doc/design/spec_coverage_audit.md`はmapping/owner/traceability/deferral/
credit impactがなく、変更しない。

## Protected And Forbidden Changes

historical precommit/implementation/review/migration/postcommit/handoff proseはimmutable。
existing owner links、redirects、fragments、headings、anchors、task rows、hashes、countsも
immutable。`doc/spec`、`.miz`、expectation、trace、production/test Rust、Cargo、
diagnostic、active route/result、test intent、semantic/coverage creditを変更しない。
本repair taskでnew familyをselect/compactしない。

## Reviews, Verification, And Exit

implementation前のindependent specification/equivalence reviewを **NO FINDINGS** にする。
implementation後のindependent test-sufficiency、implementation/equivalence、source/
documentation、bilingual/boundary、final-quality reviewsも **NO FINDINGS** にする。
全9 protocol hard gatesをscore capなしでpassし、final quality `90/100`以上を必要とする。

verificationはexisting field labelを保存するcase-aware exact 64-value replacement、
registered pairs内stale live wording消滅、
byte-identical ledger/hash/count replay、recursive contract/link/fragment lint、Markdown/
count inventory、`git diff --check`、protected-path review、format、warnings-denied Clippy、
full workspace testsを含む。exitはexact task-only staging/commit、clean postcommit
worktree/origin/stash proof、fresh schema-2 family inventoryを必要とする。successor
inventoryはunregistered familyの安全性を仮定しない。

## Handoff

clean repair commit後にtemporary gateのschema-2-safe family inventoryを再実行する。
dependency-ready familyがなければregistered totals/residual shape classesだけを記録する
bounded closeoutをfreezeし、repository-wide consolidationをclaimしない。parent authority
reasoningは`xhigh`、bounded independent reviewは`high`を推奨する。
