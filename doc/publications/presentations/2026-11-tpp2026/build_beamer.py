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
    "Part 0. Introduction": "Part 0. Introduction",
    "Part 1. Logical Foundation": "Part 1. Logical Foundation",
    "Part 2. Readable Mathematics": "Part 2. Readable Mathematics",
    "Part 3. Generic Mathematics": "Part 3. Generic Mathematics",
    "Part 4. Verified Computation": "Part 4. Verified Computation",
    "Part 5. Development Infrastructure": "Part 5. Development Infrastructure",
    "Part 6. Checking And Automation": "Part 6. Checking And Automation",
    "Part Closing. Status And Roadmap": "Status And Roadmap",
    "Part 7. Status And Roadmap": "Part 7. Status And Roadmap",
}

FIGURE_HEIGHT_LIMITS_EN = {
    "two_paths": 0.58,
    "evaluation_units": 0.50,
    "where_you_pay": 0.54,
    "layer_stack": 0.64,
    "llm_atp_loop": 0.54,
    "roadmap_tpp": 0.46,
    "reasoning_boundary": 0.52,
}

# Japanese prose wraps into more lines than English, so figures get less room.
FIGURE_HEIGHT_LIMITS_JA = {
    "two_paths": 0.56,
    "evaluation_units": 0.48,
    "where_you_pay": 0.54,
    "layer_stack": 0.64,
    "llm_atp_loop": 0.54,
    "roadmap_tpp": 0.44,
    "resolution_evidence": 0.69,
    "sat_evidence_check": 0.68,
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
        "date": (
            r"{\small\bfseries TPP 2026}\\[1.5mm]"
            r"{\footnotesize 理化学研究所 AIP 東京オフィス}\\[1mm]"
            r"{\footnotesize 2026年11月16日～17日}"
        ),
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
    split_frame_lines = gen.split_frame_lines
    make_talk_track = gen.make_talk_track
    if deck["input"].name == "slides.ja.md":
        # Subheadings organize the slide; omit them from the spoken-note list.
        gen.make_talk_track = lambda visible, notes: make_talk_track(
            [line for line in visible if not line.startswith("### ")], notes
        )
        # Keep composed slides and diagrams intact.
        gen.split_frame_lines = lambda lines: (
            [lines] if any(marker in lines for marker in (
                "| 時期 | 実装史の主な節目 |",
                "| 時期 | 開発内容 |",
                "### 言語設計の深化",
                "thus euclid_gcd(12,8) = 4 by computation;",
                "| 箇所 | 証明義務 | 数学的な根拠 |",
                "| equalities / expansions | 等式 / 定義展開の情報 |",
                "| 領域 | 現行 Mizar から取り組む課題 | 新仕様の設計指針 |",
                "![Resolution tree with composed substitutions and extracted evidence](figures/resolution_evidence.pdf)",
                "![Evidence checks, ground instances and a five-variable ten-clause SAT refutation](figures/sat_evidence_check.pdf)",
                "| 領域 | 実装済み | 完成時見積 | 限定実装と残る範囲 |",
            )) else split_frame_lines(lines)
        )
    try:
        detail_tex = gen.emit_beamer(title, units, show_notes=False)
        notes_tex = gen.emit_beamer(title, units, show_notes=True)
    finally:
        gen.split_frame_lines = split_frame_lines
        gen.make_talk_track = make_talk_track
    if deck["input"].name == "slides.ja.md":
        # Keep labels and numbers compact; give prose the remaining width.
        generated = [detail_tex, notes_tex]
        for _, _, body in units:
            for line in body:
                if line.startswith("### 表"):
                    caption = gen.inline_tex(line[4:])
                    heading = r"\smallskip" + "\n" + rf"\textbf{{{caption}}}"
                    label = r"\par\noindent{\footnotesize\bfseries " + caption + r"\par}"
                    generated = [tex.replace(heading, label) for tex in generated]
        for annotation in (
            "Source: Matuszewski & Rudnicki (2005); Carneiro (ITP 2023), Table 1。",
            "科研費 基盤研究(C)・2024年度採択: 24K14897「集合論ベースの定理証明支援系の言語仕様の刷新」",
        ):
            source = gen.inline_tex(annotation)
            generated = [tex.replace(source, r"{\footnotesize " + source + "}") for tex in generated]
        for headers, columns in (
            (("時期", "実装史の主な節目"), "lX"),
            (("処理系", "概要"), "lX"),
            (("時期", "開発内容"), "lX"),
            (("出発点", "自動的に得られる事実", "記録する規則"), "lll"),
            (("値の型", "結果型の再定義", "属性・全域性の登録"), "lll"),
            (("呼び出し（実数 R・複素数 C）[^1]", "結果型"), "lX"),
            (("構文", "検査する条件（部分正当性）[^1]"), "lX"),
            (("algorithm の形", "requires の下での意味"), "lX"),
            (("箇所", "証明義務", "数学的な根拠"), "llX"),
            (("一覧", "取り込むもの"), "lX"),
            (("段階", "著者が確認できるもの"), "lX"),
            (("領域", "実装済み", "完成時見積", "限定実装と残る範囲"), "lrrX"),
            (("評価", "対象・方式", "成功率"), "lXr"),
            (("目的", "出典", "参照箇所", "使用箇所"), "llll"),
            (("", "MizAR 60 (ITP 2023)", "Sledgehammer / AFP (ITP 2022)"), "lXX"),
            (("", "HOL ITP + ATP", "FOL ITP + ATP"), "lXX"),
        ):
            flex = r">{\raggedright\arraybackslash}X"
            header = " & ".join(rf"\textbf{{{gen.inline_tex(cell)}}}" for cell in headers)
            original = (
                rf"\begin{{tabularx}}{{\textwidth}}{{{flex * len(headers)}}}"
                "\n" r"\toprule" "\n" + header
            )
            spec = columns.replace("X", flex)
            begin = (
                rf"\begin{{tabularx}}{{\textwidth}}{{{spec}}}"
                if "X" in columns else rf"\begin{{tabular}}{{{spec}}}"
            )
            if headers[1] in ("実装史の主な節目", "概要", "取り込むもの"):
                begin = r"\renewcommand{\arraystretch}{0.93}" + "\n" + begin
            for index, tex in enumerate(generated):
                start = tex.index(original)
                end = tex.index(r"\end{tabularx}", start) + len(r"\end{tabularx}")
                table = tex[start:end].replace(original, begin + "\n" r"\toprule" "\n" + header, 1)
                if "X" not in columns:
                    table = table.replace(r"\end{tabularx}", r"\end{tabular}")
                generated[index] = tex[:start] + table + tex[end:]
        # Give the six-principle overview enough type size and space to scan.
        for index, tex in enumerate(generated):
            header = gen.inline_tex("現行 Mizar から取り組む課題")
            start = tex.rindex(r"\begin{scriptsize}", 0, tex.index(header))
            end = tex.index(r"\end{scriptsize}", start) + len(r"\end{scriptsize}")
            table = tex[start:end].replace("scriptsize", "small")
            table = table.replace(r"\arraystretch}{1.15}", r"\arraystretch}{1.5}")
            flex = r">{\raggedright\arraybackslash}X"
            table = table.replace("{" + flex * 3 + "}", "{l" + flex * 2 + "}")
            generated[index] = tex[:start] + table + tex[end:]
        detail_tex, notes_tex = generated
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
