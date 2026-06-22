# RegistrationSummary

> 正本は英語です。英語版: [../en/registration_summary.md](../en/registration_summary.md)。

## 目的

`RegistrationSummary` は activated registrations と replay trace references の
checker-facing projection である。checker-internal registration database を load
せずに cross-module reuse できるようにする。

## Schema

Schema version は `mizar-artifact/registration-summary/v1`。

Fields:

- `schema_version`。
- `package_id`。
- `module_path`。
- `registration_hash`。
- `registrations`。
- `cluster_trace_refs`。
- `local_metadata`。

Configured verification pipeline により coherence、existence、reducibility
obligations が accepted になった registrations だけが active registrations として
現れてよい。Pending、rejected、open registrations は別の diagnostic metadata に
現れてよいが importer facts ではない。

## Registration entries

各 registration entry は次を記録する:

- stable registration id と label。
- registration kind。
- checker lookup が使う trigger key。
- canonical source pattern。
- generated fact または reduction result summary。
- accepted proof status summary。
- optional witness reference id。

Entry は projection である。raw `TypedAst`、`ResolutionTrace`、checker indexes、
implicit trace steps を含めてはならない。

## Trace references

Cluster/reduction traces は別 artifact に留まる。Trace reference は次を記録する:

- portable path。
- content hash。
- trace kind。
- trace schema version。

Trace bytes があるとき、reader は reference を使う前に content hash を検証する。

## Ordering and hashes

Registration entries は kind、trigger key、registration id、label で sort する。
Trace references は path と hash で sort する。

`registration_hash` は active registration entries と trace reference identity を
覆う。`local_metadata` は除外する。

## Reader rules

Reader は unsupported versions、hash mismatches、nondeterministic order、
missing trace hashes、forbidden raw checker payloads を reject する。
