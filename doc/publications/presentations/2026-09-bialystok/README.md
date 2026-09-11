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
  Frame identifiers follow the draft headings. Backups C and E are production
  notes retained in the drafts and excluded from both generated decks.
- `bialystok_detail.tex` / `bialystok_detail.pdf` - generated discussion deck.
  The PDF uses A4 landscape physical pages for reliable printer-side handout
  layouts; the 16:9 slide content is scaled proportionally and centered.
- `bialystok_detail_notes.tex` / `bialystok_detail_notes.pdf` - generated
  deck with presenter notes, also on A4 landscape physical pages.
- `references.bib` - seed bibliography for the talk and the paper outline
  (Backup D); entries must be verified against publishers before final use.
- `figures/` - TikZ standalone sources and compiled PDFs. All nine diagrams
  from Backup C of `draft.md` are produced: `three_pressures`,
  `environ_migration`, `diamond_inheritance`, `reasoning_boundary`,
  `certificate_replay`, `fingerprint_graph`, `fm_links`, `pipeline`,
  `roadmap_timeline`.

## Two-Tier Pacing

Frames whose headings carry `[deep dive]` in `draft.md` can be skipped
without breaking a story arc; the generated deck marks them with a small
"deep dive" tag. Keep the full deck as a handout. For the 45-minute talk,
read the dark-blue bold sentences from top to bottom. Start with the title-page
notes; after that, slides without highlighted prose can be skipped. Other prose,
code, tables, sources, and sections marked "for later review" are optional.
Code syntax colors do not mark talk priorities. The notes edition keeps the same
emphasis under "Read aloud", followed by optional notes and sources.

Use this provisional budget, including time to point to code and diagrams.
The highlighted text alone is not a timed 45-minute script. Whether the slot
includes questions is still to be confirmed; allow five minutes for questions
or extra explanation and check the timing in rehearsal.

| Parts | Topic | Minutes |
|---|---|---:|
| 0–1 | Opening and motivation | 6 |
| 2–4 | Dependencies, structures, registrations; focus on 3.3 | 10 |
| 5 | Search and checking; focus on 5.2 | 8 |
| 6–7 | Scale and templates | 5 |
| 8–10 | Algorithms, publication, pipeline; focus on 8.2 | 7 |
| 11–12 | Status, roadmap, closing | 4 |
| — | Questions or extra explanation | 5 |

In the drafts, `**...**` marks these priorities in prose and short `text`
key-phrase blocks. Keep the EN/JA selections aligned. Mizar and TOML listings
retain their original text and syntax highlighting.

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

- The audience includes the Mizar group and other faculty members.
  The opening briefly introduces Mizar before discussing the proposed changes.
- The talk should be in English.
- Main-slide prose uses short, complete sentences at CEFR B2 middle or below.
  Keep technical terms, code, and exact MML excerpts unchanged when simplifying it.
- Every code example carries a status label: exact MML excerpt (with article
  and line numbers), specification example, or sketch.
- Exact MML excerpts keep attribution, URLs, and line numbers in speaker
  notes (GPL-3.0-or-later / CC-BY-SA-3.0-or-later distribution terms).
- Each story ends with questions for later review; the oral overview uses
  selected examples and leaves the details in the handout.

## Pre-Talk Checklist (August 2026)

- [ ] Re-check the Mizar home page: current system version, MML version, and
      article count (frames 1.1 and 1.2 cite MML 5.94.1493 / 1493 articles,
      checked June 18, 2026).
- [ ] Re-verify every exact MML excerpt against the current MML: article,
      line numbers, and text (Backup A lists all excerpts).
- [ ] Re-check every specification example against `doc/spec/en/`; see
      Source Status in `draft.md` for the authority and example conventions.
- [ ] Update the project-status frame (11.0 - Where The Project Stands
      Today): supported parts, incomplete integration, deferred execution.
- [ ] Verify `references.bib` entries against publisher metadata.
- [ ] Rebuild figures and both decks; skim every page for overflow.
- [ ] Confirm `draft.ja.md` is synchronized with `draft.md`.
