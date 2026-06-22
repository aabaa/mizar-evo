# ModuleSummary

> Canonical language: English. Japanese companion: [../ja/module_summary.md](../ja/module_summary.md).

## Purpose

`ModuleSummary` is the resolver-facing exported interface projection of one
module. It lets downstream module and name resolution consume a dependency
without loading private declarations, bodies, or proofs.

## Schema

The schema version is `mizar-artifact/module-summary/v1`.

Fields:

- `schema_version`;
- `package_id`;
- `module_path`;
- `source_hash`;
- `interface_hash`;
- `exports`;
- `exported_labels`;
- `imported_interfaces`;
- `local_metadata`.

`local_metadata` is optional user/development metadata and is excluded from the
interface hash.

## Export Entries

Export entries record dependency-facing declarations only:

- stable fully-qualified name;
- exported surface spelling;
- declaration kind;
- visibility;
- canonical signature text;
- optional defining module when the item is a re-export.

Private declarations, declaration bodies, proof bodies, raw resolver/checker IR,
and source-local helper identities are forbidden.

## Ordering And Hashes

Entries are sorted by canonical key:

1. declaration kind;
2. fully-qualified name;
3. spelling;
4. signature text.

`interface_hash` covers `schema_version`, package/module identity, exports,
exported labels, and imported interface hashes. It excludes `source_hash` and
`local_metadata`, so a body-only edit that changes source bytes but leaves the
exported interface unchanged preserves the interface hash.

## Reader Rules

Readers must validate:

- supported `schema_version`;
- deterministic entry ordering after parsing;
- recorded `interface_hash`;
- absence of forbidden raw/private payload fields.

Validation failure is a structured artifact error.
