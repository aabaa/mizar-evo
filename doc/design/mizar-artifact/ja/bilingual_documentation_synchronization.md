# Bilingual Documentation Synchronization: mizar-artifact

> 正本は英語です。英語版: [../en/bilingual_documentation_synchronization.md](../en/bilingual_documentation_synchronization.md)。

## 範囲

この監査は `doc/design/mizar-artifact/en/` 配下の英語正本と
`doc/design/mizar-artifact/ja/` 配下の日本語 companion を対象にする。

## Pairing matrix

| English canonical | Japanese companion | Status |
|---|---|---|
| `00.crate_plan.md` | `00.crate_plan.md` | synchronized |
| `todo.md` | `todo.md` | synchronized |
| `store.md` | `store.md` | synchronized |
| `module_summary.md` | `module_summary.md` | synchronized |
| `registration_summary.md` | `registration_summary.md` | synchronized |
| `proof_witness.md` | `proof_witness.md` | synchronized |
| `verified_artifact.md` | `verified_artifact.md` | synchronized |
| `manifest.md` | `manifest.md` | synchronized |
| `source_spec_correspondence.md` | `source_spec_correspondence.md` | synchronized |
| `bilingual_documentation_synchronization.md` | `bilingual_documentation_synchronization.md` | synchronized |
| `crate_exit_report.md` | `crate_exit_report.md` | synchronized |

## Notes

- English が canonical である。
- Japanese companions は同じ task state、task 17 deferred reason、module
  ownership、schema field sets、verification expectations を保持する。
- この変更では Japanese placeholder は不要である。

## Result

既知の bilingual documentation synchronization gap はない。
