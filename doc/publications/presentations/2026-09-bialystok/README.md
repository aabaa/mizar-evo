# Bialystok Presentation, September 2026

This directory is for a planned presentation to the Mizar team in Bialystok.

The third draft restructures the talk around eight problem-driven stories:
each starts from a real cost in current Mizar practice (with exact MML
excerpts), shows the Mizar Evo answer in code, and states what is preserved.
EBNF is intentionally absent; `doc/spec/en/` remains the grammar authority.

## Artifacts

- `draft.md` - third English content draft (canonical): opening hook,
  three-pressures motivation, eight stories (dependencies, structures,
  auditable automation, search vs trust, scale, templates, algorithms,
  publication), architecture synthesis, roadmap, and backup material.
- `draft.ja.md` - Japanese companion draft for faster review.
- `build_beamer.py` - generates the Beamer decks from `draft.md`.
- `bialystok_detail.tex` / `bialystok_detail.pdf` - generated discussion deck.
- `bialystok_detail_notes.tex` / `bialystok_detail_notes.pdf` - generated
  deck with presenter notes.
- `references.bib` - future bibliography.
- `figures/` - diagrams listed in Backup C of `draft.md` (to be produced).

## Regenerating The Deck

```bash
python3 build_beamer.py
pdflatex bialystok_detail.tex
pdflatex bialystok_detail_notes.tex
```

## Working Assumptions

- The target audience knows current Mizar well; the deck acknowledges shared
  pains instead of explaining Mizar to its authors.
- The talk should be in English.
- Every code example carries a status label: exact MML excerpt (with article
  and line numbers), specification example, or sketch.
- Exact MML excerpts keep attribution, URLs, and line numbers in speaker
  notes (GPL-3.0-or-later / CC-BY-SA-3.0-or-later distribution terms).
- The seminar is informal: depth is preferred over strict time discipline,
  and every story ends with questions for the Bialystok team.
