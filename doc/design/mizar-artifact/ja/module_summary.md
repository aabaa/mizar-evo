# ModuleSummary

> 正本は英語です。英語版: [../en/module_summary.md](../en/module_summary.md)。

## 目的

`ModuleSummary` は 1 module の resolver-facing exported interface projection
である。下流の module/name resolution は、private declarations、bodies、proofs
を load せずに dependency を消費できる。

## Schema

Schema version は `mizar-artifact/module-summary/v1`。

Fields:

- `schema_version`。
- `package_id`。
- `module_path`。
- `source_hash`。
- `interface_hash`。
- `exports`。
- `exported_labels`。
- `imported_interfaces`。
- `local_metadata`。

`local_metadata` は optional user/development metadata であり、interface hash
から除外する。

## Export entries

Export entry は dependency-facing declarations だけを記録する:

- stable fully-qualified name。
- exported surface spelling。
- declaration kind。
- visibility。
- canonical signature text。
- re-export の場合は optional defining module。

Private declarations、declaration bodies、proof bodies、raw resolver/checker IR、
source-local helper identities は禁止する。

## Ordering and hashes

Entry は canonical key で sort する:

1. declaration kind。
2. fully-qualified name。
3. spelling。
4. signature text。

`interface_hash` は `schema_version`、package/module identity、exports、
exported labels、imported interface hashes を覆う。`source_hash` と
`local_metadata` は除外するため、source bytes だけを変える body-only edit は
exported interface が同じなら interface hash を保つ。

## Reader rules

Reader は次を検証する:

- supported `schema_version`。
- parse 後の deterministic entry ordering。
- recorded `interface_hash`。
- forbidden raw/private payload fields がないこと。

Validation failure は structured artifact error である。
