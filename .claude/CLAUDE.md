# Project Instructions

## Documentation Maintenance

- English is the canonical documentation language.
- Bilingual EN/JA maintenance is mandatory only for `doc/spec/{en,ja}` and `doc/design/architecture/{en,ja}` (language-scope decision, user-approved 2026-09-01).
- When updating documentation in those bilingual areas, maintain both the English canonical document and the Japanese companion document in the same change.
- For language specifications, update matching files under `doc/spec/en/` and `doc/spec/ja/`.
- For architecture specifications, update matching files under `doc/design/architecture/en/` and `doc/design/architecture/ja/`.
- Status, audit, process, roadmap, and archive documents elsewhere under `doc/design/` are English-only; where a Japanese companion path exists or a bilingual crate-tree layout expects one, add or keep a pointer stub that links the canonical English file (precedent: `doc/design/mizar-test/ja/semantic_bridge_corpus_map.md`). Module design documents in existing bilingual crate trees keep their current pairing; see `doc/design/documentation_compaction_rules.md`.
- Keep file names aligned across language directories whenever possible.
- If an English document in a bilingual area changes but the Japanese companion cannot be updated in the same change, explicitly note the reason and mark the Japanese document as needing synchronization.
- When adding a new English documentation file in a bilingual area, add the corresponding Japanese companion or a clearly marked Japanese placeholder that links to the canonical English file.

## Documentation Volume

- Design documents record decisions and boundaries only. Never write scores, gate tallies, digests, review outcomes, model names, or reasoning settings under `doc/`; `cargo test` enforces this via `tests/coverage/doc_volume_baseline.tsv`, whose rows may only be lowered or removed.
- Task contracts are at most 60 lines per language and are linked only from `doc/design/todo.md`, crate plans, crate todos, or `doc/design/spec_coverage_audit.md`.

## Commit Message Suggestions

- After editing source code or documentation, include a suggested commit message in the final response.
- Prefer a concise Conventional Commits-style subject such as `docs: ...`, `fix: ...`, or `feat: ...`.
- When the change is broad enough to need context, include an optional short body explaining the main changes.
