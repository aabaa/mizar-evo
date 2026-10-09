#!/usr/bin/env python3
"""Generate the TPP 2026 Beamer decks from slides.md (English) and slides.ja.md (Japanese).

The Markdown-to-Beamer generator lives in the Bialystok deck directory
(../2026-09-bialystok/build_beamer.py).  This script loads it as a library,
overrides the deck-specific settings (parts, title-page metadata, figure
heights, preamble), and writes:

  tpp2026.tex, tpp2026_notes.tex        English deck, compile with pdflatex
  tpp2026_ja.tex, tpp2026_ja_notes.tex  Japanese deck, compile with
                                        uplatex + dvipdfmx (luatexja is not
                                        installed on the build machine)

Figures from the Bialystok deck are referenced by relative path from the
Markdown sources, so both decks share one source for those diagrams.

Usage: python3 build_beamer.py [en|ja|all]   (default: all)
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
GENERATOR = ROOT.parent / "2026-09-bialystok" / "build_beamer.py"

PART_REMAP = {
    "Part 0. Opening": "Part 0. Opening",
    "Part 1. Two Hammers, Two Numbers": "Part 1. Two Hammers, Two Numbers",
    "Part 2. Where Complexity Lives": "Part 2. Where Complexity Lives",
    "Part 3. Modernizing Mizar's Answer": "Part 3. Modernizing Mizar's Answer",
    "Part 4. The AI Era": "Part 4. The AI Era",
    "Part 5. Roadmap And Closing": "Part 5. Roadmap And Closing",
}

FIGURE_HEIGHT_LIMITS_EN = {
    "two_paths": 0.58,
    "evaluation_units": 0.50,
    "where_you_pay": 0.34,
    "layer_stack": 0.60,
    "llm_atp_loop": 0.38,
    "roadmap_tpp": 0.46,
    "reasoning_boundary": 0.52,
}

# Japanese prose wraps into more lines than English, so figures get less room.
FIGURE_HEIGHT_LIMITS_JA = {
    "two_paths": 0.56,
    "evaluation_units": 0.48,
    "where_you_pay": 0.34,
    "layer_stack": 0.57,
    "llm_atp_loop": 0.36,
    "roadmap_tpp": 0.44,
    "reasoning_boundary": 0.50,
}

DECKS = {
    "en": {
        "input": ROOT / "slides.md",
        "output": ROOT / "tpp2026.tex",
        "figure_heights": FIGURE_HEIGHT_LIMITS_EN,
        "documentclass_options": "aspectratio=169,11pt",
        "font_setup": [r"\usepackage[T1]{fontenc}"],
        "preamble_extra": [],
        "author": "Mizar Evo project",
        "date": "TPP 2026, RIKEN AIP Tokyo Office, November 16--17, 2026",
    },
    "ja": {
        "input": ROOT / "slides.ja.md",
        "output": ROOT / "tpp2026_ja.tex",
        "figure_heights": FIGURE_HEIGHT_LIMITS_JA,
        # dvipdfmx driver for upLaTeX; bxdpx-beamer fixes Beamer under dvipdfmx,
        # pxjahyper fixes Japanese PDF bookmarks and must follow hyperref.
        "documentclass_options": "dvipdfmx,aspectratio=169,11pt",
        "font_setup": [r"\usepackage[T1]{fontenc}"],
        "preamble_extra": [
            r"\usepackage{bxdpx-beamer}",
            r"\usepackage{pxjahyper}",
            r"\renewcommand{\kanjifamilydefault}{\gtdefault}",
            # Beamer's shaded "ball" markers do not render under dvipdfmx; use flat ones.
            r"\setbeamertemplate{itemize items}[circle]",
            r"\setbeamertemplate{enumerate items}[default]",
            r"\institute{岩手県立大学}",
        ],
        "author": "中正 和久",
        "date": r"TPP 2026、理化学研究所 AIP 東京オフィス、2026年11月16--17日",
    },
}


def load_generator():
    spec = importlib.util.spec_from_file_location("bialystok_build_beamer", GENERATOR)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    module.ROOT = ROOT
    module.PART_REMAP = PART_REMAP
    return module


def build(gen, deck: dict) -> None:
    gen.FIGURE_HEIGHT_LIMITS = deck["figure_heights"]
    gen.DOCUMENTCLASS_OPTIONS = deck["documentclass_options"]
    gen.FONT_SETUP = deck["font_setup"]
    gen.PREAMBLE_EXTRA = deck["preamble_extra"]
    gen.AUTHOR = deck["author"]
    gen.DATE = deck["date"]
    output_path: Path = deck["output"]
    notes_output_path = output_path.with_name(f"{output_path.stem}_notes{output_path.suffix}")
    title, units = gen.parse_markdown(deck["input"])
    detail_tex = gen.emit_beamer(title, units, show_notes=False)
    notes_tex = gen.emit_beamer(title, units, show_notes=True)
    output_path.write_text(detail_tex, encoding="utf-8")
    notes_output_path.write_text(notes_tex, encoding="utf-8")
    source_frame_count = sum(1 for kind, _, _ in units if kind == "frame") + 1
    generated_frame_count = detail_tex.count(r"\begin{frame}")
    print(f"wrote {output_path}")
    print(f"wrote {notes_output_path}")
    print(f"source frames including title: {source_frame_count}")
    print(f"generated frames including title: {generated_frame_count}")


def main() -> int:
    selection = sys.argv[1] if len(sys.argv) > 1 else "all"
    langs = list(DECKS) if selection == "all" else [selection]
    gen = load_generator()
    for lang in langs:
        build(gen, DECKS[lang])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
