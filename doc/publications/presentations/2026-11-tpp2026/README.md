# TPP 2026 Presentation

Working materials for the Mizar Evo talk at TPP 2026, the 22nd Theorem Proving
and Provers meeting, RIKEN AIP Tokyo Office, November 16-17, 2026. The talk is
designed for a thirty-minute slot; confirm the slot length and language with
the organizers.

## Title

Deck title (English):

**Mizar Evolution: Why First-Order Logic, Now**
Reconnecting automated proof, readable mathematics, and verified computation

Japanese title for the program (first candidate in `draft.ja.md`):

**Mizar Evolution: なぜ今、一階述語論理なのか**
— 自動証明・数学的記述・検証可能な計算を再接続する —

## Purpose

This directory is intentionally a working area rather than part of the language
specification. The presentation explains the architectural motivation of
Mizar Evolution, especially the role of first-order automated theorem provers
(ATPs) in an AI-assisted mathematics workflow.

The central question is:

> What becomes possible if a large-scale interactive theorem prover is designed
> from the outset around a first-order logical substrate and a native hammer,
> rather than connecting first-order ATPs through a higher-order translation
> layer?

## Files

- `draft.ja.md` - Japanese narrative draft. **Canonical content document**:
  the story, the claim levels, the 45-minute additions, and the pre-talk
  checklist live here.
- `slides.md` - English deck source, designed from the draft as a story rather
  than slide-by-slide. Slide prose is plain English at CEFR B1 to easy B2.
- `slides.ja.md` - Japanese deck source with the same frames and numbering,
  for delivering the talk in Japanese with Japanese slides. Figures are
  shared with the English deck and keep their English labels.
- `script.ja.md` - Japanese spoken script, frame by frame, with timing and
  claim-level tags, for delivering the talk in Japanese over English slides.
- `build_beamer.py` - loads the Bialystok generator
  (`../2026-09-bialystok/build_beamer.py`) as a library and overrides the
  deck-specific settings (parts, title-page metadata, figure heights,
  preamble). `python3 build_beamer.py [en|ja|all]`.
- `tpp2026.tex` / `tpp2026.pdf` - generated English deck (A4 landscape pages,
  16:9 content, as in the Bialystok deck).
- `tpp2026_notes.tex` / `tpp2026_notes.pdf` - English deck with presenter
  notes ("Read aloud" is the dark-blue bold spine; sources follow).
- `tpp2026_ja.tex` / `tpp2026_ja.pdf`, `tpp2026_ja_notes.tex` /
  `tpp2026_ja_notes.pdf` - generated Japanese decks.
- `figures/` - six TikZ figures new to this talk: `evaluation_units`,
  `two_paths`, `where_you_pay`, `layer_stack`, `llm_atp_loop`, `roadmap_tpp`.
- `references.bib` - seed bibliography. Verify metadata before the final talk.

## Storyline (Thirty Minutes)

The deck does not follow the chapter order of `draft.ja.md` mechanically. It
keeps the draft's facts, hypotheses, and future directions, and arranges them
so that the audience feels the question before hearing the answer.

| Part | Minutes | Beat | Frames |
|---|---:|---|---|
| 1. Two hammers, two numbers | 0-6 | the puzzle: 58.4% vs 60.7%, and why they must not be compared; two paths; the research question | 1.1-1.4 |
| 2. Where complexity lives | 6-14 | functions in HOL pay at the ATP boundary; functions in set theory pay at the keyboard; Mizar's language absorbs that cost | 2.1-2.6 |
| 3. Modernizing Mizar's answer | 14-24 | keep the substrate; templates; algorithms as a second pillar; search outside, trust inside; fifty years of infrastructure in one table | 3.1-3.8 |
| 4. The AI era | 24-28 | the whole picture; LLM thinks, ATP proves, Mizar Evo remembers and verifies; honest implementation status | 4.1-4.3 |
| 5. Roadmap and closing | 28-30 | 2026, 2027, 2028+; back to the two paths; closing line | 5.1-5.3 |

Frames marked `[deep dive]` (0.2, 2.6, 3.3, 3.6) can be skipped. Backups 1-10
hold detailed benchmark conditions, encoding details, template instantiation,
and the Bialystok infrastructure material. The closing line is fixed:
*Keep the foundation small. Keep the mathematics readable. Modernize everything
else.*

## Language Policy

- English slide prose: CEFR B1 to easy B2. Short sentences, common words, one
  idea per sentence. Technical terms stay and are explained in simple words on
  first use. Code, exact MML excerpts, and figure labels are unchanged.
- Japanese deck: same frames and numbering as the English deck. The spoken
  script `script.ja.md` works with either deck.
- Keep the two sources aligned when a frame changes: `slides.md` first, then
  `slides.ja.md`.

## Claim Levels

Untagged statements are facts about existing systems, published benchmarks,
the Mizar Evo specification, or the main branch. "Research hypothesis" marks
what Mizar Evo is built to test (how much of the MizAR / hammer difference is
architectural; how far a native first-order hammer can go; the LLM plus cheap
ATP cascade). "Future direction" marks targets with no committed date or
design (cryptographic protocols, quantum algorithms, autonomous theory
generation). Frame 4.3 separates the specification from the implementation.

## How The Bialystok Deck Is Used

- Figures reused by relative path: `reasoning_boundary`, `certificate_replay`,
  `pipeline`, `fingerprint_graph`.
- The Bialystok deck's eight problem-driven stories are the reference material
  for every feature this talk only names (structures, registrations, packages,
  incremental verification, publication). Main frames point to them; they are
  not repeated.
- The Markdown-to-Beamer generator is shared. Its author and date are
  parametrized so this deck can override them; the Bialystok output is
  unchanged.

## Regenerating The Deck

```bash
# figures (only when a figures/*.tex source changed)
cd figures && for f in *.tex; do pdflatex -interaction=nonstopmode "$f"; done && cd ..

# both decks (tex)
python3 build_beamer.py

# English deck
pdflatex tpp2026.tex
pdflatex tpp2026_notes.tex

# Japanese deck (upLaTeX + dvipdfmx; luatexja is not installed here)
uplatex tpp2026_ja.tex && dvipdfmx tpp2026_ja.dvi
uplatex tpp2026_ja_notes.tex && dvipdfmx tpp2026_ja_notes.dvi
```

Run `pdflatex` / `uplatex` twice when frame numbers in the footer look stale.
The reused Bialystok figure PDFs must exist; they are tracked in the
repository. The Japanese deck uses the Harano Aji fonts through the default
`ptex-fontmaps` setup and `bxdpx-beamer` + `pxjahyper` for Beamer under
dvipdfmx.

Both main decks build without vertical overflow. In the notes editions a few
note pages run long because the "Read aloud" text and sources are reproduced
in full; trim speaker notes before printing if that matters.

## Pre-Talk Checklist

Verified on October 7, 2026 while designing the deck:

- [x] MizAR 60 conditions: MML 1147, 57,897 theorems including unnamed
      top-level lemmas; 58.4% in hammering mode with a 420 CPU s portfolio;
      over 75% with library premises chosen by a human or a machine; strongest
      single method 40% in 30 s (arXiv 2303.06686).
- [x] AFP study conditions: 6,934 goals from 128 theories, 30 s per prover,
      about 50% one-line replay per prover, 60.7% union as oracle; Judgement
      Day 46% (2010) and 75% (2015 preliminary) (CICM 2015 paper).
- [x] Exact MML excerpts: `funct_1.miz` lines 138-140, `funct_2.miz` lines
      87-90.
- [x] Current specification sends ATPs only cited premises and local
      hypotheses (spec 21.7.2); the native hammer is a 2027 research item.

Still to do before the talk:

- [ ] Re-read the HOL-to-FOL encoding frame (2.2) against Meng and Paulson 2008
      and Blanchette et al. 2016; it is schematic.
- [ ] Check the historical wording: FOL completeness and the ATP tradition,
      LCF tactics, and "Mizar never had a user-programmable tactic language".
- [ ] Re-check the template and algorithm examples against the current
      `doc/spec/en/` text.
- [ ] Update frame 4.3 (implementation status) from `doc/design/todo.md`,
      Crate Status, on the talk date; do not claim end-to-end external-prover
      results unless they exist.
- [ ] Verify `references.bib` metadata against publishers.
- [ ] Confirm the slot length (30 or 45 minutes) and the question time.
- [ ] Rebuild both decks and skim every page for overflow.
- [ ] Rehearse `script.ja.md` against the clock.

## Instructions for Codex

When revising the slides:

- read `draft.ja.md` first; it is the content authority, and `slides.md` is a
  design derived from it;
- also inspect `doc/design/architecture/en/00.pipeline_overview.md`,
  `08.reasoning_boundary.md`, `09.atp_interface_protocol.md`, and
  `10.atp_backend_integration.md`;
- do **not** modify the language specification merely to fit the presentation;
- distinguish established facts, interpretation, research hypotheses, and
  future directions, and keep the tags in the text;
- do not compare success percentages across MizAR and Sledgehammer as if the
  benchmarks were identical;
- preserve the distinction between proof search and trusted verification;
- keep the presentation focused on architecture and research questions, not
  on attacking other theorem provers;
- keep `script.ja.md` aligned with the frame numbers in `slides.md`.
