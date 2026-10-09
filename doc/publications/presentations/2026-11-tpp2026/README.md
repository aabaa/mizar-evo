# TPP 2026 Presentation

Working materials for the Mizar Evo talk at TPP 2026, the 22nd Theorem Proving
and Provers meeting, RIKEN AIP Tokyo Office, November 16-17, 2026. The talk is
designed for a thirty-minute slot; confirm the slot length and language with
the organizers.

## Title

Deck title (English):

**Mizar Evo: Design Principles**
Reconnecting automated proof, readable mathematics, and verified computation

Japanese title for the program (first candidate in `draft.ja.md`):

**Mizar Evo の設計指針について**
— 自動証明・数学的記述・検証可能な計算を再接続する —

## Purpose

The presentation starts with six challenges in modernizing current Mizar
and maps each to a design principle in the new specification. It explains how
Mizar Evo preserves the logical foundation and readable mathematical language
while rebuilding generic mechanisms, verified computation, development tools,
and evidence checking.

Benchmarks and HOL/FOL connections are supplementary context. A first-order
performance advantage is not the motivation of this talk. The main checking
story is evidence, formula instantiation, and a trusted SAT check.

## Files

- `draft.ja.md` - Japanese narrative draft. **Canonical content document**:
  the story, the claim levels, the 45-minute additions, and the pre-talk
  checklist live here.
- `slides.md` - English deck source, designed from the draft as a story rather
  than slide-by-slide. Slide prose is plain English at CEFR B1 to easy B2.
- `slides.ja.md` - Japanese deck source using the English frame numbering,
  with frame 0.2 omitted. Figures are
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

Section 0 pairs the six challenges and design principles in one table.
Sections 1–6 each cover the matching row. Closing frames have no section number.

| Section | Minutes | Beat | Frames |
|---|---:|---|---|
| 0. Introduction | 0-3 | challenges and principles in one table | 0.1-0.3 |
| 1. Logical foundation | 3-6 | first-order logic, set theory, and MML | 1.1-1.2 |
| 2. Readable mathematics | 6-9 | abstraction, explicit operation views, registration traces | 2.1-2.2 |
| 3. Generic mathematics | 9-13 | functor templates and schemes | 3.1-3.2 |
| 4. Verified computation | 13-19 | algorithm contracts, proofs, and execution | 4.1-4.3 |
| 5. Development infrastructure | 19-23 | dependencies, tools, and the whole picture | 5.1-5.2 |
| 6. Checking and automation | 23-27 | ATP flow, evidence instantiation, SAT | 6.1-6.3 |
| Closing | 27-30 | implementation status, roadmap, and discussion | unnumbered |

Frames marked `[deep dive]` (0.2, 3.2, 4.3) can be skipped; 0.2 is omitted
from the Japanese deck. Backups 1-10 hold benchmark details, template
instantiation, and infrastructure material; 11-17 hold the benchmark and HOL/FOL
comparison sequence. Frame 6.2 keeps the ATP flow diagram in the main talk;
6.3 explains the core content of Backup 5. The closing line is fixed:
*Keep the foundation small. Keep the mathematics readable. Modernize everything
else.*

## Language Policy

- English slide prose: CEFR B1 to easy B2. Short sentences, common words, one
  idea per sentence. Technical terms stay and are explained in simple words on
  first use. Code, exact MML excerpts, and figure labels are unchanged.
- Japanese deck: English frame numbering, with frame 0.2 omitted. The spoken
  script `script.ja.md` works with either deck.
- Keep the two sources aligned when a frame changes: `slides.md` first, then
  `slides.ja.md`.

## Claim Levels

Untagged statements are facts about existing systems, published benchmarks,
the Mizar Evo specification, or the main branch. "Research hypothesis" marks
an evaluation question rather than a result. "Future direction" marks targets
with no committed date or design. The closing status frame separates the specification from
the implementation.

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
- [x] Exact MML excerpts: `funct_1.miz` lines 138-140, `funct_2.miz` lines
      87-90.
- [x] Current specification sends ATPs only cited premises and local
      hypotheses (spec 21.7.2); the native hammer is a 2027 research item.

Still to do before the talk:

- [ ] Re-read the HOL-to-FOL encoding frame (Backup 15) against Meng and Paulson 2008
      and Blanchette et al. 2016; it is schematic.
- [ ] Check the historical wording: FOL completeness and the ATP tradition,
      LCF tactics, and "Mizar never had a user-programmable tactic language".
- [ ] Re-check the template and algorithm examples against the current
      `doc/spec/en/` text.
- [ ] Update the closing implementation status frame from `doc/design/todo.md`,
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
- lead with current Mizar's challenges and the new specification's responses;
- explain evidence instantiation and SAT checking in the main talk;
- keep benchmarks and HOL/FOL performance questions in the backups;
- keep `script.ja.md` aligned with the frame numbers in `slides.md`.
