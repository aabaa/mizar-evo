# Documentation Language Policy

This repository uses **English as the canonical documentation language** and provides **Japanese companion documents** where helpful.

## Policy

- English documents are canonical for implementation decisions, terminology, APIs, and architectural boundaries.
- Japanese documents are companion translations or explanations intended to make review easier for Japanese readers.
- When documents disagree, the English document wins until the Japanese companion is synchronized.
- File names should match across languages whenever possible.
- New architecture documents should be written in English first, then mirrored or summarized in Japanese.

## Directory Convention

```text
doc/
  design/
    architecture/
      README.md      # language entrypoint
      en/            # canonical English architecture docs
      ja/            # Japanese companion architecture docs
```

Other documentation areas may adopt the same `en/` / `ja/` split as they are reorganized.
