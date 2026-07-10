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
  publication), project status, architecture synthesis, roadmap, and backup
  material including anticipated objections (Backup F).
- `draft.ja.md` - Japanese companion draft for faster review.
- `build_beamer.py` - generates the Beamer decks from `draft.md`: title
  page, status-label badges, key-phrase blocks, Mizar syntax highlighting,
  booktabs tables, embedded figures, deep-dive tags.
- `bialystok_detail.tex` / `bialystok_detail.pdf` - generated discussion deck.
- `bialystok_detail_notes.tex` / `bialystok_detail_notes.pdf` - generated
  deck with presenter notes.
- `references.bib` - seed bibliography for the talk and the paper outline
  (Backup D); entries must be verified against publishers before final use.
- `figures/` - TikZ standalone sources and compiled PDFs. Produced so far:
  `reasoning_boundary`, `pipeline`, `fm_links`; the remaining diagrams are
  listed in Backup C of `draft.md`.

## Two-Tier Pacing

Frames whose headings carry `[deep dive]` in `draft.md` can be skipped
without breaking a story arc; the generated deck marks them with a small
"deep dive" tag. The unmarked core path is roughly 48 frames (about 60-75
minutes plus discussion).

## Regenerating The Deck

```bash
# figures (only when a figures/*.tex source changed)
cd figures && for f in *.tex; do pdflatex -interaction=nonstopmode "$f"; done && cd ..

# decks
python3 build_beamer.py
pdflatex bialystok_detail.tex
pdflatex bialystok_detail_notes.tex
```

Run `pdflatex` twice when frame numbers in the footer look stale.

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

## Pre-Talk Checklist (August 2026)

- [ ] Re-check the Mizar home page: current system version, MML version, and
      article count (frames 1.1 and 1.2 cite MML 5.94.1493 / 1493 articles,
      checked June 18, 2026).
- [ ] Re-verify every exact MML excerpt against the current MML: article,
      line numbers, and text (Backup A lists all excerpts).
- [ ] Re-check every specification example against `doc/spec/en/` (last
      re-check: July 10, 2026; see Source Status in `draft.md`).
- [ ] Update the project-status frame (11.x - Where The Project Stands
      Today): crate count, line counts, completed audits.
- [ ] Verify `references.bib` entries against publisher metadata.
- [ ] Rebuild figures and both decks; skim every page for overflow.
- [ ] Confirm `draft.ja.md` is synchronized with `draft.md`.
