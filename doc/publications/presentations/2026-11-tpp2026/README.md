# TPP 2026 Presentation

Working materials for the Mizar Evo talk at TPP 2026, the 22nd Theorem Proving
and Provers meeting, RIKEN AIP Tokyo Office, November 16-17, 2026. The talk is
being revised in Japanese for a provisional 45-minute slot: 38 minutes of
presentation and seven minutes of questions. Confirm the official slot and
question time with the organizers. Aim for 35–38 minutes, never over 40.

## Title

Deck title (English):

**Mizar Evo: Design Principles**
Reconnecting automated proof, readable mathematics, and verified computation

Japanese title for the program (first candidate in `draft.ja.md`):

**Mizar Evo の設計指針について**
— 自動証明・数学的記述・検証可能な計算を再接続する —

## Purpose

The Japanese presentation opens with Mizar history and the project’s design
work in 2023–2026, including the sustained work in 2024–2025. It then maps six
challenges in modernizing current Mizar to six design principles. It explains how
Mizar Evo preserves the logical foundation and readable mathematical language
while rebuilding generic mechanisms, verified computation, development tools,
and evidence checking.

Benchmarks and HOL/FOL connections are supplementary context. A first-order
performance advantage is not the motivation of this talk. The main checking
story is evidence, formula instantiation, and a trusted SAT check.

## Files

- `draft.ja.md` - Japanese narrative draft. **Canonical content document**:
  the story, the historical sources, claim levels, timing, and the pre-talk
  checklist live here.
- `slides.md` - previous English deck source, unchanged during this Japanese
  revision. Translation follows Japanese content approval; CEFR B1 to easy B2.
- `slides.ja.md` - Japanese deck source. Principle section numbers stay;
  Section 0 covers the introduction, sections 1–6 the design principles, and section 7 status and plans.
  Figures retain their English labels.
- `script.ja.md` - Japanese spoken script, frame by frame, with timing and
  claim-level tags, for the revised Japanese deck. It is not yet aligned with the English deck.
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
- `figures/` - TikZ figures new to this talk: `evaluation_units`,
  `two_paths`, `where_you_pay`, `layer_stack`, `llm_atp_loop`, `roadmap_tpp`,
  plus `resolution_evidence` and `sat_evidence_check` for the Japanese checking example.
- `references.bib` - seed bibliography. Verify metadata before the final talk.

## Storyline (Japanese Revision, 38 Minutes)

History traces early experimental processors, the PC port, PC-Mizar and MML,
then Mizar-rs and Evo before the six-principle table in frame 0.5. Sections 1–6
retain their technical examples. Closing frames distinguish specified behavior,
limited implementation, and plans; the full source-to-artifact path is unfinished.
The five-slide introduction combines history with processor comparisons, the
project origin with its design timeline, and design expansion with AI implementation.

| Section | Clock | Minutes | Frames |
|---|---|---:|---|
| History and development | 0–9 | 9 | 0.1–0.5 |
| 1. Logical foundation | 9–11 | 2 | 1.1–1.2 |
| 2. Readable mathematics | 11–16 | 5 | 2.1–2.6 |
| 3. Generic mathematics | 16–19 | 3 | 3.1–3.4 |
| 4. Verified computation | 19–24 | 5 | 4.1–4.5 |
| 5. Development infrastructure | 24–27 | 3 | 5.1–5.3 |
| 6. Checking and automation | 27–32 | 5 | 6.1–6.4 |
| Status, roadmap, closing | 32–38 | 6 | 7.1, 7.2, 7.3, 7.4 |
| Questions (outside main talk) | 38–45 | 7 | selected backups |

For 35 minutes, skip 3.3 (40 seconds), shorten 3.4 to 15 seconds (saving 20),
skip 5.3 (one minute), and shorten the design-expansion part of 0.4 to
30 seconds and 7.3 to 90 seconds. Keep the design timeline and evidence/SAT
examples. These budgets include pointing at code and diagrams; rehearsal is
still required. Backups are for questions, not extra material to fill 45 minutes.

Japanese frame 1.2 briefly shows two current-Mizar theorem sketches: Cantor
and the compact-to-Hausdorff continuous-bijection theorem. Proofs are omitted;
the examples share one minute to show mathematical objects, structures and attributes.
Kernel details stay in part 6.

Frame 6.2 keeps the ATP flow diagram in the main talk;
6.3-6.4 show a resolution tree, composed substitutions, kernel preprocessing,
and all clauses of a Tseitin SAT encoding;
external resolution steps are not replayed as kernel acceptance evidence.
Japanese frames 2.1-2.6 follow type, registration, then overload: structure
and property declarations (2.2), post-declaration inheritance (2.3), then the
Unital mode and its unit implementation, reused as zero through the selected
LoopStr view (2.4). Labeled registration chains follow in 2.5, and definition
selection with explicit inheritance views in 2.6. The latter distinguishes
the additive AddMagma and multiplicative MulMagma views by obtaining zero
or one as the neutral element of the same ring.
Frames 3.1-3.2 contrast per-type result redefinitions, registrations and proofs
for complex, real, rational, integer and natural values with one generic sum.
Frame 3.3 shows a scheme as a theorem; 3.4 gives brief struct VectorSpace[K]
and attr dense[M] examples of shared type/value parameterization, including
`Vector of K` / `M-dense` as syntax sugar for `Vector[K]` / `dense[M]`.
Part 3 keeps its three-minute budget (45/60/40/35 seconds);
4.1 summarizes the basic assignment, branch, loop and return Hoare rules;
4.3 repeats the Euclid body with numbered annotations matching the table of
establishment, preservation, termination and exit obligations;
4.4 explains promotion through checked termination, contract citation, and
`by computation` using a concrete-input example (MVM execution remains planned);
4.5 connects lambda function notation, reflected syntax and verified symbolic
differentiation. Part 4 keeps five minutes (45/75/75/60/45 seconds);
5.1 lists all ten environment directives and emphasizes notation/definition
precedence and order dependence as the central authoring difficulty;
5.2 then introduces imports. These are explanatory
sketches and specification examples, not execution or external-prover results.
The closing line is fixed:
*Keep the foundation small. Keep the mathematics readable. Modernize everything
else.*

## Language Policy

- English slide prose: CEFR B1 to easy B2. Short sentences, common words, one
  idea per sentence. Technical terms stay and are explained in simple words on
  first use. Code, exact MML excerpts, and figure labels are unchanged.
- Production order: `draft.ja.md` → `slides.ja.md` and `script.ja.md` → English.
- This task updates Japanese content and this README only. Keep `slides.md`,
  figures, language specification, internal designs, and Rust sources unchanged.
  The generator uses compact first columns for the two Japanese history tables. Existing tracked PDFs/TeX still represent the earlier deck
  until explicitly regenerated; use the revised Markdown for content review.
- Work in a dedicated branch/worktree. Do not merge into main, push, or create
  a PR until instructed.

## Claim Levels

Untagged statements are facts about existing systems, published benchmarks,
the Mizar Evo specification, or the fixed implementation snapshot in
`draft.ja.md` §7. Historical claims distinguish publications, Git diffs, and
the author’s account. The snapshot must be refreshed before the talk. "Research hypothesis" marks
an evaluation question rather than a result. "Future direction" marks targets
with no committed date or design. The closing status frame separates the specification from
the implementation.

## How The Bialystok Deck Is Used

- Figures reused by relative path: `reasoning_boundary`, `certificate_replay`,
  `pipeline`, `fingerprint_graph`.
- The Bialystok deck's problem-driven stories supply the structure and
  registration examples. It remains the detailed reference for packages,
  incremental verification, and publication.
- The Markdown-to-Beamer generator is shared. Its author and date are
  parametrized so this deck can override them; the Bialystok output is
  unchanged.

## Regenerating The Deck

```bash
# figures (only when a figures/*.tex source changed)
cd figures && for f in *.tex; do pdflatex -interaction=nonstopmode "$f"; done && cd ..

# Japanese only (inside the dedicated worktree)
python3 build_beamer.py ja

# Japanese deck (upLaTeX + dvipdfmx)
uplatex tpp2026_ja.tex && dvipdfmx tpp2026_ja.dvi
uplatex tpp2026_ja_notes.tex && dvipdfmx tpp2026_ja_notes.dvi
```

Run `pdflatex` / `uplatex` twice when frame numbers in the footer look stale.
The reused Bialystok figure PDFs must exist; they are tracked in the
repository. The Japanese deck uses the Harano Aji fonts through the default
`ptex-fontmaps` setup and `bxdpx-beamer` + `pxjahyper` for Beamer under
dvipdfmx.

The generator may split a source frame into continuation pages. Check the
rendered Japanese deck and notes after revision, especially tables and source
notes. Timing follows source frame IDs, not generated page numbers.

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
- [ ] Confirm the assumed 45-minute slot and seven-minute question time.
- [ ] Confirm any precise ordering of the 2023 consultations, repository rename
      date, or specific Białystok feedback before adding those details.
- [ ] Keep the proposed 2027 work and unapproved grant extension as plans.
- [ ] Build and inspect the Japanese deck; translate and rebuild English later.
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
- lead with history and development, then connect to the existing six principles;
- explain evidence instantiation and SAT checking in the main talk;
- keep benchmarks and HOL/FOL performance questions in the backups;
- keep `script.ja.md` aligned with the frame order and IDs in `slides.ja.md`.
