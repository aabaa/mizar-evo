# Artifact Manifest

> 正本は英語です。英語版: [../en/manifest.md](../en/manifest.md)。

## 目的

Package artifact manifest は 1 package artifact directory 配下で emit された
artifacts の publication index である。Reader はまず manifest を読み、任意
file scan で published artifacts を発見しない。

## Schema

Schema version は `mizar-artifact/manifest/v1`。

Manifest fields:

- `schema_version`。
- `package_id`。
- `package_version`。
- `lockfile_hash`。
- `toolchain`。
- `verifier_config_hash`。
- `modules`。

Module entry fields:

- `module_path`。
- `source_file`。
- `source_hash`。
- `artifact_file`。
- `artifact_hash`。
- `interface_hash`。
- `implementation_hash`。
- `proof_witnesses`。
- `diagnostics_hash`。

Entries は `module_path`、次に `artifact_file` で sort する。

## Transactions

Manifest transaction は次の通り:

1. 現在 visible な manifest があればそれを base にする。
2. referenced artifact / witness files がすでに存在し recorded hashes と一致する
   module updates を stage する。
3. referenced `VerifiedArtifact` を検証し、manifest の `module_path`、
   `source_file`、`source_hash`、`interface_hash`、`implementation_hash`、
   `diagnostics_hash`、proof witness entries が artifact と一致することを確認する。
4. fully serialized temporary manifest を write する。
5. flush する。
6. `artifact-manifest.json` へ rename する。
7. supported platform では containing directory を flush する。

したがって reader は old complete manifest または new complete manifest のどちらか
だけを見る。Temporary files や unreferenced artifacts から entries を merge しては
ならない。

## Recovery

Interrupted commits は temporary files や unreferenced artifacts を残すことがある。
Reader はそれらを無視する。同じ transaction を byte-identical staged entries で
replay することは idempotent である。

Referenced file が missing、malformed、hash-mismatched の場合、transaction は
fail し、previous manifest が authoritative のまま残る。

## Reader validation

Reader は schema version、deterministic entry order、artifact hashes、利用可能な
witness hashes、portable paths、manifest hash fields と referenced
`VerifiedArtifact` の一致を検証する。Duplicate module entries または unsupported
schema version を持つ manifest は reject される。
