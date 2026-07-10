#!/usr/bin/env python3
"""Generate a detailed Beamer deck from draft.md.

This is intentionally small and local to the presentation draft.  It handles the
Markdown patterns used in this file rather than trying to be a full Markdown
implementation.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
DEFAULT_INPUT = ROOT / "draft.md"
DEFAULT_OUTPUT = ROOT / "bialystok_detail.tex"

PART_REMAP = {
    "Part 0. Opening": "Part 0. Opening",
    "Part 1. Why Now": "Part 1. Why Now",
    "Part 2. Story 1: Dependencies You Can See": "Part 2. Story 1: Dependencies You Can See",
    "Part 3. Story 2: Structures Without Hidden Merges": "Part 3. Story 2: Structures Without Hidden Merges",
    "Part 4. Story 3: Automation You Can Audit": "Part 4. Story 3: Automation You Can Audit",
    "Part 5. Story 4: Powerful Search, Small Trust": "Part 5. Story 4: Powerful Search, Small Trust",
    "Part 6. Story 5: Verification That Scales": "Part 6. Story 5: Verification That Scales",
    "Part 7. Story 6: Templates For Generic Mathematics": "Part 7. Story 6: Templates For Generic Mathematics",
    "Part 8. Story 7: Verified Computation With Algorithms": "Part 8. Story 7: Verified Computation With Algorithms",
    "Part 9. Story 8: A Library You Can Cite": "Part 9. Story 8: A Library You Can Cite",
    "Part 10. Architecture In One Picture": "Part 10. Architecture In One Picture",
    "Part 11. Roadmap And Collaboration": "Part 11. Roadmap And Collaboration",
    "Part 12. Closing": "Part 12. Closing",
}

PART_APPEND_TO: dict[str, str] = {}

LABEL_REWRITES = {
    "Bullets": "Key Points",
    "Detailed claim": "Claim",
    "Slide text": "Key Phrase",
    "Why": "Reason",
    "Design reason": "Design Reason",
    "Review point": "Review Point",
    "Speaker note": "Presenter Note",
}

SECTION_LABEL_RE = re.compile(r"^[A-Za-z][A-Za-z0-9 `/'()&.,-]{0,80}:$")
IMAGE_RE = re.compile(r"^!\[([^\]]*)\]\(([^)]+)\)$")
DEEP_DIVE_TAG = " [deep dive]"
FRAME_WEIGHT_LIMIT = 18.5
MAX_CODE_LINES_PER_BLOCK = 15
MAX_LIST_ITEMS_PER_BLOCK = 7

CODE_STATUS_RE = re.compile(
    r"^(?P<pre>.*?)\s*\((?P<status>exact MML excerpt|specification example|sketch)"
    r"(?P<rest>[^)]*)\)$"
)

STATUS_BADGES = {
    "exact MML excerpt": r"\badgeMML",
    "specification example": r"\badgeSpec",
    "sketch": r"\badgeSketch",
}


SPECIALS = {
    "\\": r"\textbackslash{}",
    "&": r"\&",
    "%": r"\%",
    "$": r"\$",
    "#": r"\#",
    "_": r"\_",
    "{": r"\{",
    "}": r"\}",
    "~": r"\textasciitilde{}",
    "^": r"\textasciicircum{}",
}

UNICODE_REPLACEMENTS = {
    "ł": r"\l{}",
    "Ł": r"\L{}",
    "ś": r"\'s",
    "Ś": r"\'S",
    "ć": r"\'c",
    "Ć": r"\'C",
    "ń": r"\'n",
    "Ń": r"\'N",
    "ó": r"\'o",
    "Ó": r"\'O",
    "ą": r"\k{a}",
    "Ą": r"\k{A}",
    "ę": r"\k{e}",
    "Ę": r"\k{E}",
    "ż": r"\.{z}",
    "Ż": r"\.{Z}",
    "ź": r"\'z",
    "Ź": r"\'Z",
}


def escape_tex(text: str) -> str:
    out: list[str] = []
    for ch in text:
        if ch in UNICODE_REPLACEMENTS:
            out.append(UNICODE_REPLACEMENTS[ch])
        else:
            out.append(SPECIALS.get(ch, ch))
    return "".join(out)


def inline_tex(text: str) -> str:
    def link_repl(match: re.Match[str]) -> str:
        label = inline_tex(match.group(1))
        url = escape_tex(match.group(2))
        return rf"{label} (\url{{{url}}})"

    def code_repl(match: re.Match[str]) -> str:
        return rf"\texttt{{{escape_tex(match.group(1))}}}"

    text = re.sub(r"\[([^\]]+)\]\(([^)]+)\)", link_repl, text)
    parts: list[str] = []
    pos = 0
    for match in re.finditer(r"`([^`]+)`", text):
        parts.append(escape_tex(text[pos : match.start()]))
        parts.append(code_repl(match))
        pos = match.end()
    parts.append(escape_tex(text[pos:]))
    return "".join(parts)


def frame_title(markdown_title: str) -> str:
    title = re.sub(r"^Frame\s+", "", markdown_title.strip())
    return inline_tex(title)


def plain_text(text: str) -> str:
    text = re.sub(r"`([^`]+)`", r"\1", text)
    text = re.sub(r"\[([^\]]+)\]\(([^)]+)\)", r"\1", text)
    return text.strip()


def plain_frame_title(markdown_title: str) -> str:
    title = re.sub(r"^Frame\s+", "", markdown_title.strip())
    title = re.sub(r"^[0-9][0-9A-Za-z.]*\s*-\s*", "", title)
    return plain_text(title)


def section_part_number(section_title: str) -> str | None:
    match = re.match(r"^Part\s+(\d+)\.", section_title)
    return match.group(1) if match else None


def renumber_frame_heading(title_text: str, part_number: str, counter: int) -> str:
    title = re.sub(r"^Frame\s+", "", title_text.strip())
    title = re.sub(r"^[0-9][0-9A-Za-z.]*\s*-\s*", "", title)
    return f"Frame {part_number}.{counter} - {title}"


def is_table_start(lines: list[str], index: int) -> bool:
    if index + 1 >= len(lines):
        return False
    first = lines[index].lstrip()
    second = lines[index + 1].lstrip()
    return first.startswith("|") and second.startswith("|") and set(second.strip()) <= set("|:- ")


def split_table_row(line: str) -> list[str]:
    stripped = line.strip()
    if stripped.startswith("|"):
        stripped = stripped[1:]
    if stripped.endswith("|"):
        stripped = stripped[:-1]
    return [cell.strip() for cell in stripped.split("|")]


def table_to_tex(rows: list[str]) -> list[str]:
    data = [split_table_row(row) for row in rows]
    if len(data) < 2:
        return [inline_tex(row) for row in rows]
    header = data[0]
    body = data[2:]
    cols = max(len(header), *(len(row) for row in body)) if body else len(header)
    font_size = r"\scriptsize" if cols >= 4 or (cols == 3 and len(body) >= 5) else r"\footnotesize"
    font_env = font_size.lstrip("\\")
    if (len(body) > 7 and cols >= 3) or cols >= 5:
        out = [
            rf"\begin{{{font_env}}}",
            r"\begin{description}",
            r"\setlength{\itemsep}{0.5pt}",
            r"\setlength{\parsep}{0pt}",
            r"\setlength{\parskip}{0pt}",
            r"\setlength{\topsep}{1pt}",
        ]
        for row in body:
            padded = row + [""] * (cols - len(row))
            label = inline_tex(padded[0])
            parts: list[str] = []
            for column_name, cell in zip(header[1:], padded[1:]):
                if cell:
                    parts.append(rf"\textbf{{{inline_tex(column_name)}:}} {inline_tex(cell)}")
            out.append(rf"\item[{label}] " + r" \par ".join(parts))
        out.extend([r"\end{description}", rf"\end{{{font_env}}}"])
        return out
    spec = "".join([r">{\raggedright\arraybackslash}X"] * cols)
    out = [
        rf"\begin{{{font_env}}}",
        r"\setlength{\tabcolsep}{3pt}",
        r"\renewcommand{\arraystretch}{1.15}",
        rf"\begin{{tabularx}}{{\textwidth}}{{{spec}}}",
        r"\toprule",
    ]
    out.append(
        " & ".join(rf"\textbf{{{inline_tex(cell)}}}" for cell in header) + r" \\"
    )
    out.append(r"\midrule")
    for row in body:
        padded = row + [""] * (cols - len(row))
        out.append(" & ".join(inline_tex(cell) for cell in padded[:cols]) + r" \\")
    out.extend([r"\bottomrule", r"\end{tabularx}", rf"\end{{{font_env}}}"])
    return out


def flush_paragraph(paragraph: list[str], out: list[str]) -> None:
    if not paragraph:
        return
    text = " ".join(line.strip() for line in paragraph).strip()
    paragraph.clear()
    if not text:
        return
    if text.endswith(":") and len(text) <= 80:
        label = LABEL_REWRITES.get(text[:-1], text[:-1])
        status = CODE_STATUS_RE.match(label)
        out.append(r"\smallskip")
        if status:
            pre = status.group("pre").strip()
            badge = STATUS_BADGES[status.group("status")]
            rest = status.group("rest").strip().lstrip(",").strip()
            line = rf"\textbf{{{inline_tex(pre)}}}~{badge}"
            if rest:
                line += rf"~{{\footnotesize({inline_tex(rest)})}}"
            out.append(line)
        else:
            out.append(rf"\textbf{{{inline_tex(label)}:}}")
    else:
        out.append(inline_tex(text))
        out.append("")


def collect_list(lines: list[str], index: int, ordered: bool) -> tuple[list[str], int]:
    items: list[str] = []
    current: list[str] = []
    marker = re.compile(r"^\s*(\d+)\.\s+(.*)$") if ordered else re.compile(r"^\s*-\s+(.*)$")
    while index < len(lines):
        line = lines[index]
        match = marker.match(line)
        if match:
            if current:
                items.append(" ".join(part.strip() for part in current).strip())
            current = [match.group(2 if ordered else 1)]
            index += 1
            continue
        if current and (line.startswith("  ") or line.startswith("    ")) and line.strip():
            current.append(line.strip())
            index += 1
            continue
        break
    if current:
        items.append(" ".join(part.strip() for part in current).strip())
    return items, index


def verbatim_font_size(code_lines: list[str]) -> str:
    longest = max((len(line) for line in code_lines), default=0)
    if len(code_lines) > 7 or longest > 74:
        return r"\footnotesize"
    return r"\small"


def is_code_fence(line: str) -> bool:
    return line.strip().startswith("```")


def is_list_marker(line: str) -> bool:
    return bool(re.match(r"^\s*(?:-\s+|\d+\.\s+)", line))


def is_block_boundary(lines: list[str], index: int) -> bool:
    if index >= len(lines):
        return True
    stripped = lines[index].strip()
    return (
        not stripped
        or is_code_fence(lines[index])
        or is_table_start(lines, index)
        or is_list_marker(lines[index])
        or stripped.startswith(">")
        or stripped.startswith("### ")
        or SECTION_LABEL_RE.match(stripped) is not None
    )


def collect_markdown_blocks(lines: list[str]) -> list[list[str]]:
    blocks: list[list[str]] = []
    pending_prefix: list[str] = []
    index = 0

    def push(block: list[str]) -> None:
        nonlocal pending_prefix
        cleaned = list(block)
        while cleaned and not cleaned[0].strip():
            cleaned.pop(0)
        while cleaned and not cleaned[-1].strip():
            cleaned.pop()
        if not cleaned:
            return
        if pending_prefix:
            cleaned = pending_prefix + [""] + cleaned
            pending_prefix = []
        blocks.append(cleaned)

    while index < len(lines):
        line = lines[index].rstrip("\n")
        stripped = line.strip()

        if not stripped:
            index += 1
            continue

        if stripped.startswith("### ") or SECTION_LABEL_RE.match(stripped):
            if pending_prefix:
                blocks.append(pending_prefix)
            pending_prefix = [line]
            index += 1
            continue

        if is_code_fence(line):
            block = [line]
            index += 1
            while index < len(lines):
                block.append(lines[index].rstrip("\n"))
                if is_code_fence(lines[index]):
                    index += 1
                    break
                index += 1
            push(block)
            continue

        if is_table_start(lines, index):
            block = []
            while index < len(lines) and lines[index].lstrip().startswith("|"):
                block.append(lines[index].rstrip("\n"))
                index += 1
            push(block)
            continue

        if is_list_marker(line):
            block = []
            while index < len(lines):
                current = lines[index].rstrip("\n")
                if is_list_marker(current) or (
                    block and (current.startswith("  ") or current.startswith("    ")) and current.strip()
                ):
                    block.append(current)
                    index += 1
                    continue
                break
            push(block)
            continue

        if stripped.startswith(">"):
            block = []
            while index < len(lines) and lines[index].strip().startswith(">"):
                block.append(lines[index].rstrip("\n"))
                index += 1
            push(block)
            continue

        block = []
        while index < len(lines) and not is_block_boundary(lines, index):
            block.append(lines[index].rstrip("\n"))
            index += 1
        push(block)

    if pending_prefix:
        blocks.append(pending_prefix)
    return blocks


def split_code_block(block: list[str]) -> list[list[str]]:
    fence_index = next((index for index, line in enumerate(block) if is_code_fence(line)), -1)
    if fence_index < 0:
        return [block]
    prefix = block[:fence_index]
    code_block = block[fence_index:]
    if len(code_block) < 2:
        return [block]
    opener = code_block[0]
    closer = code_block[-1] if is_code_fence(code_block[-1]) else "```"
    code_lines = code_block[1:-1] if is_code_fence(code_block[-1]) else code_block[1:]
    if len(code_lines) <= MAX_CODE_LINES_PER_BLOCK:
        return [block]
    chunks: list[list[str]] = []
    for start in range(0, len(code_lines), MAX_CODE_LINES_PER_BLOCK):
        part = [opener] + code_lines[start : start + MAX_CODE_LINES_PER_BLOCK] + [closer]
        chunks.append(prefix + [""] + part if prefix else part)
    return chunks


def split_table_block(block: list[str]) -> list[list[str]]:
    if any(is_code_fence(line) for line in block):
        return [block]
    table_index = next((index for index, line in enumerate(block) if line.lstrip().startswith("|")), -1)
    if table_index < 0:
        return [block]
    prefix = block[:table_index]
    rows = block[table_index:]
    columns = len(split_table_row(rows[0])) if rows else 0
    row_limit = 3 if columns >= 4 else 5 if columns == 3 else 7
    body_rows = rows[2:]
    if body_rows:
        max_row_len = max(
            sum(len(plain_text(cell)) for cell in split_table_row(row))
            for row in body_rows
        )
        if max_row_len > 110:
            row_limit = min(row_limit, 3)
    if len(rows) <= row_limit + 2:
        return [block]
    header = rows[:2]
    body = rows[2:]
    chunks: list[list[str]] = []
    for start in range(0, len(body), row_limit):
        part = header + body[start : start + row_limit]
        chunks.append(prefix + [""] + part if prefix else part)
    return chunks


def split_list_block(block: list[str]) -> list[list[str]]:
    if any(is_code_fence(line) for line in block):
        return [block]
    first_item_index = next((index for index, line in enumerate(block) if is_list_marker(line)), -1)
    if first_item_index < 0:
        return [block]
    prefix = block[:first_item_index]
    lines = block[first_item_index:]
    items: list[list[str]] = []
    current: list[str] = []
    for line in lines:
        if is_list_marker(line):
            if current:
                items.append(current)
            current = [line]
        elif current:
            current.append(line)
    if current:
        items.append(current)
    if len(items) <= MAX_LIST_ITEMS_PER_BLOCK:
        return [block]
    chunks: list[list[str]] = []
    for start in range(0, len(items), MAX_LIST_ITEMS_PER_BLOCK):
        part: list[str] = []
        for item in items[start : start + MAX_LIST_ITEMS_PER_BLOCK]:
            part.extend(item)
        chunks.append(prefix + [""] + part if prefix else part)
    return chunks


def split_oversized_block(block: list[str]) -> list[list[str]]:
    blocks = [block]
    for splitter in (split_code_block, split_table_block, split_list_block):
        next_blocks: list[list[str]] = []
        for current in blocks:
            next_blocks.extend(splitter(current))
        blocks = next_blocks
    return blocks


def count_code_lines(block: list[str]) -> int:
    code_lines = 0
    in_code = False
    for line in block:
        if is_code_fence(line):
            in_code = not in_code
            continue
        if in_code:
            code_lines += 1
    return code_lines


def has_table_rows(block: list[str]) -> bool:
    in_code = False
    for line in block:
        if is_code_fence(line):
            in_code = not in_code
            continue
        if not in_code and line.lstrip().startswith("|"):
            return True
    return False


def count_list_items(block: list[str]) -> int:
    if any(is_code_fence(line) for line in block):
        return 0
    return sum(1 for line in block if is_list_marker(line))


def should_start_new_chunk(current: list[str], block: list[str]) -> bool:
    if not current:
        return False
    if has_table_rows(current) and count_code_lines(block) >= 6:
        return True
    if count_code_lines(current) >= 8 and count_list_items(block) >= 4:
        return True
    return False


def block_weight(block: list[str]) -> float:
    code_lines = count_code_lines(block)
    if code_lines:
        prefix_lines = sum(1 for line in block if line.strip() and not is_code_fence(line)) - code_lines
        return 2.5 + prefix_lines * 0.9 + code_lines * 0.78

    table_rows = sum(1 for line in block if line.lstrip().startswith("|"))
    if table_rows:
        rows = [line for line in block if line.lstrip().startswith("|")]
        body = rows[2:] if len(rows) > 2 else rows
        weight = 2.2
        for row in body:
            cells = split_table_row(row)
            text_len = sum(len(plain_text(cell)) for cell in cells)
            weight += 1.2 + text_len / 82.0
        return weight

    list_items = count_list_items(block)
    if list_items:
        continuation = sum(
            1
            for line in block
            if line.strip() and not is_list_marker(line) and not SECTION_LABEL_RE.match(line.strip())
        )
        return 1.0 + list_items * 1.35 + continuation * 0.8

    weight = 0.0
    for line in block:
        stripped = line.strip()
        if not stripped:
            continue
        if IMAGE_RE.match(stripped):
            weight += 9.0
        elif stripped.startswith("### ") or SECTION_LABEL_RE.match(stripped):
            weight += 1.0
        else:
            weight += max(1.0, len(plain_text(stripped)) / 74.0)
    return weight


def split_frame_lines(lines: list[str]) -> list[list[str]]:
    blocks: list[list[str]] = []
    for block in collect_markdown_blocks(lines):
        blocks.extend(split_oversized_block(block))

    chunks: list[list[str]] = []
    current: list[str] = []
    current_weight = 0.0
    for block in blocks:
        weight = block_weight(block)
        if current and (
            should_start_new_chunk(current, block) or current_weight + weight > FRAME_WEIGHT_LIMIT
        ):
            chunks.append(current)
            current = []
            current_weight = 0.0
        if current:
            current.append("")
        current.extend(block)
        current_weight += weight
    if current:
        chunks.append(current)
    return chunks or [[]]


def is_key_phrase_block(code_lines: list[str], lang: str) -> bool:
    if lang != "text" or not code_lines or len(code_lines) > 4:
        return False
    for line in code_lines:
        if "->" in line or line.startswith("  ") or len(line) > 72:
            return False
    return True


def append_key_phrase(out: list[str], code_lines: list[str]) -> None:
    body = r"\\ ".join(inline_tex(line.strip()) for line in code_lines if line.strip())
    out.append(r"\begin{center}")
    out.append(r"\begin{beamercolorbox}[rounded=true,sep=2.5mm,wd=0.9\textwidth]{keyphrase}")
    out.append(rf"\centering\itshape {body}")
    out.append(r"\end{beamercolorbox}")
    out.append(r"\end{center}")


def append_verbatim_block(out: list[str], code_lines: list[str], lang: str = "") -> None:
    if is_key_phrase_block(code_lines, lang):
        append_key_phrase(out, code_lines)
        return
    font_size = verbatim_font_size(code_lines)
    language = {"mizar": "mizar", "toml": "toml"}.get(lang, "")
    out.append(
        rf"\begin{{lstlisting}}[language={{{language}}},"
        rf"basicstyle=\ttfamily{font_size}]"
    )
    out.extend(code_lines)
    out.append(r"\end{lstlisting}")


def append_image(out: list[str], path: str) -> None:
    out.append(r"\begin{center}")
    out.append(
        rf"\includegraphics[width=\textwidth,height=0.62\textheight,"
        rf"keepaspectratio]{{{path}}}"
    )
    out.append(r"\end{center}")


def split_speaker_notes(lines: list[str]) -> tuple[list[str], list[str]]:
    visible: list[str] = []
    notes: list[str] = []
    index = 0
    while index < len(lines):
        stripped = lines[index].strip()
        if stripped.lower() in {"speaker note:", "speaker notes:"}:
            index += 1
            while index < len(lines):
                next_stripped = lines[index].strip()
                if next_stripped and SECTION_LABEL_RE.match(next_stripped):
                    break
                notes.append(lines[index])
                index += 1
            continue
        visible.append(lines[index])
        index += 1
    return visible, notes


def extract_key_points(lines: list[str], max_points: int = 5) -> tuple[list[str], bool, bool]:
    points: list[str] = []
    in_code = False
    has_code = False
    has_table = False
    current: list[str] = []

    def flush_current() -> None:
        nonlocal current
        if current and len(points) < max_points:
            text = " ".join(part.strip() for part in current).strip()
            if text:
                points.append(plain_text(text))
        current = []

    for raw_line in lines:
        line = raw_line.rstrip("\n")
        stripped = line.strip()
        if stripped.startswith("```"):
            flush_current()
            in_code = not in_code
            has_code = True
            continue
        if in_code:
            continue
        if not stripped:
            flush_current()
            continue
        if IMAGE_RE.match(stripped):
            flush_current()
            continue
        if stripped.startswith("|"):
            flush_current()
            has_table = True
            continue
        if SECTION_LABEL_RE.match(stripped):
            flush_current()
            continue
        bullet = re.match(r"^\s*-\s+(.*)$", line)
        ordered = re.match(r"^\s*\d+\.\s+(.*)$", line)
        if bullet or ordered:
            flush_current()
            current = [(bullet or ordered).group(1)]
            continue
        if current and (line.startswith("  ") or line.startswith("    ")):
            current.append(stripped)
            continue
        if len(stripped) <= 160 and len(points) < max_points:
            flush_current()
            current = [stripped]
    flush_current()
    return points[:max_points], has_code, has_table


def make_talk_track(title: str, visible_lines: list[str], explicit_note_lines: list[str]) -> list[str]:
    clean_title = plain_frame_title(title)
    points, has_code, has_table = extract_key_points(visible_lines, max_points=5)
    if explicit_note_lines or has_code or has_table:
        points = points[:3]
    note_lines: list[str] = ["Presenter script:"]

    if explicit_note_lines:
        note_lines.extend(["", "Say:"])
        note_lines.extend(explicit_note_lines)

    if points:
        note_lines.extend(["", "Read aloud:"])
        for point in points:
            note_lines.append(f"- {point}")

    if has_code and not explicit_note_lines:
        note_lines.extend(
            [
                "",
                "After the example, say which design obligation the syntax makes visible.",
            ]
        )
    if has_table:
        note_lines.extend(
            [
                "",
                "For the table, read the row labels first, then the design reason.",
            ]
        )

    if "Syntax" in clean_title or "Grammar" in clean_title:
        note_lines.extend(
            [
                "",
                "Slow down: this is language-specification review, not only implementation detail.",
            ]
        )
    if not points and not explicit_note_lines:
        note_lines.extend(
            [
                "",
                f"Use this as the transition: {clean_title}.",
                "Point to the visible example, question, or diagram before moving on.",
            ]
        )
    return note_lines


def render_frame_body(lines: list[str]) -> list[str]:
    out: list[str] = []
    paragraph: list[str] = []
    index = 0
    in_code = False
    code_lang = ""
    code_lines: list[str] = []

    while index < len(lines):
        line = lines[index].rstrip("\n")
        stripped = line.strip()

        if stripped.startswith("```"):
            flush_paragraph(paragraph, out)
            if not in_code:
                in_code = True
                code_lang = stripped[3:].strip()
                code_lines = []
            else:
                append_verbatim_block(out, code_lines, code_lang)
                in_code = False
            index += 1
            continue

        if in_code:
            code_lines.append(line)
            index += 1
            continue

        image = IMAGE_RE.match(stripped)
        if image:
            flush_paragraph(paragraph, out)
            append_image(out, image.group(2))
            index += 1
            continue

        if stripped.startswith("### "):
            flush_paragraph(paragraph, out)
            out.append(r"\smallskip")
            out.append(rf"\textbf{{{inline_tex(stripped[4:])}}}")
            index += 1
            continue

        if is_table_start(lines, index):
            flush_paragraph(paragraph, out)
            table_rows: list[str] = []
            while index < len(lines) and lines[index].lstrip().startswith("|"):
                table_rows.append(lines[index].rstrip("\n"))
                index += 1
            out.extend(table_to_tex(table_rows))
            out.append("")
            continue

        if re.match(r"^\s*-\s+", line):
            flush_paragraph(paragraph, out)
            items, index = collect_list(lines, index, ordered=False)
            out.append(r"\begin{itemize}")
            out.append(r"\setlength{\itemsep}{1pt}")
            out.append(r"\setlength{\parsep}{0pt}")
            out.append(r"\setlength{\parskip}{0pt}")
            for item in items:
                out.append(rf"\item {inline_tex(item)}")
            out.append(r"\end{itemize}")
            continue

        if re.match(r"^\s*\d+\.\s+", line):
            flush_paragraph(paragraph, out)
            items, index = collect_list(lines, index, ordered=True)
            out.append(r"\begin{enumerate}")
            out.append(r"\setlength{\itemsep}{1pt}")
            out.append(r"\setlength{\parsep}{0pt}")
            out.append(r"\setlength{\parskip}{0pt}")
            for item in items:
                out.append(rf"\item {inline_tex(item)}")
            out.append(r"\end{enumerate}")
            continue

        if stripped.startswith(">"):
            flush_paragraph(paragraph, out)
            quote_lines: list[str] = []
            while index < len(lines) and lines[index].strip().startswith(">"):
                quote_lines.append(lines[index].strip().lstrip("> ").strip())
                index += 1
            quote = " ".join(line for line in quote_lines if line)
            out.append(r"\begin{quote}")
            out.append(inline_tex(quote))
            out.append(r"\end{quote}")
            continue

        if not stripped:
            flush_paragraph(paragraph, out)
            index += 1
            continue

        paragraph.append(line)
        index += 1

    flush_paragraph(paragraph, out)
    if in_code:
        append_verbatim_block(out, code_lines, code_lang)
    return out


def parse_markdown(path: Path) -> tuple[str, list[tuple[str, str | None, list[str]]]]:
    lines = path.read_text(encoding="utf-8").splitlines()
    title = "Mizar Evo"
    units: list[tuple[str, str | None, list[str]]] = []
    current_title: str | None = None
    current_lines: list[str] = []
    appendix_started = False
    include_current_section = False
    current_part_number: str | None = None
    frame_counters: dict[str, int] = {}

    def close_frame() -> None:
        nonlocal current_title, current_lines
        if current_title is not None and include_current_section:
            heading = current_title
            if plain_frame_title(heading) == "Title":
                units.append(("title", heading, current_lines))
                current_title = None
                current_lines = []
                return
            if current_part_number is not None and heading.startswith("Frame "):
                frame_counters[current_part_number] = frame_counters.get(current_part_number, 0) + 1
                heading = renumber_frame_heading(
                    heading,
                    current_part_number,
                    frame_counters[current_part_number],
                )
            units.append(("frame", heading, current_lines))
        current_title = None
        current_lines = []

    for line in lines:
        if line.startswith("# "):
            title = line[2:].strip()
            continue
        if line.startswith("## Part "):
            close_frame()
            title_text = line[3:].strip()
            include_current_section = title_text in PART_REMAP
            if include_current_section:
                mapped_title = PART_REMAP[title_text]
                current_part_number = section_part_number(mapped_title)
                units.append(("section", mapped_title, []))
            elif title_text in PART_APPEND_TO:
                include_current_section = True
                current_part_number = PART_APPEND_TO[title_text]
            else:
                current_part_number = None
            continue
        if line.startswith("## Backup "):
            close_frame()
            include_current_section = True
            current_part_number = None
            if not appendix_started:
                units.append(("appendix", None, []))
                appendix_started = True
            title_text = line[3:].strip()
            units.append(("section", title_text, []))
            current_title = title_text
            current_lines = []
            continue
        if line.startswith("## "):
            close_frame()
            current_part_number = None
            if include_current_section:
                current_title = line[3:].strip()
                current_lines = []
            continue
        if line.startswith("### Frame "):
            close_frame()
            if include_current_section:
                current_title = line[4:].strip()
                current_lines = []
            continue
        if line.startswith("### "):
            if not include_current_section:
                continue
            if current_title is None:
                current_title = line[4:].strip()
                current_lines = []
            else:
                current_lines.append(line)
            continue
        if current_title is not None:
            current_lines.append(line)

    close_frame()
    return title, units


def extract_code_blocks(lines: list[str]) -> list[list[str]]:
    blocks: list[list[str]] = []
    current: list[str] | None = None
    for line in lines:
        if is_code_fence(line):
            if current is None:
                current = []
            else:
                blocks.append(current)
                current = None
            continue
        if current is not None:
            current.append(line.rstrip("\n"))
    return blocks


def title_page_fields(units: list[tuple[str, str | None, list[str]]]) -> tuple[str, str, list[str]]:
    """Extract \\title, \\subtitle, and title-page notes from the Title frame."""
    for kind, _, body in units:
        if kind != "title":
            continue
        visible, notes = split_speaker_notes(body)
        blocks = extract_code_blocks(visible)
        title_lines = [line.strip() for line in (blocks[0] if blocks else []) if line.strip()]
        subtitle_lines = [
            line.strip() for line in (blocks[1] if len(blocks) > 1 else []) if line.strip()
        ]
        main_title = inline_tex(title_lines[0]) if title_lines else "Mizar Evo"
        subtitle_parts = title_lines[1:] + subtitle_lines
        subtitle = r"\\ ".join(inline_tex(part) for part in subtitle_parts)
        return main_title, subtitle, notes
    return "Mizar Evo", "", []


def emit_beamer(
    title: str,
    units: list[tuple[str, str | None, list[str]]],
    *,
    show_notes: bool = False,
) -> str:
    main_title, subtitle, title_notes = title_page_fields(units)
    if show_notes:
        subtitle = subtitle + r"\\ \textit{presenter notes edition}" if subtitle else r"\textit{presenter notes edition}"
    out: list[str] = [
        r"\documentclass[aspectratio=169,11pt]{beamer}",
        r"\usetheme{Madrid}",
        r"\usecolortheme{seahorse}",
        r"\setbeamertemplate{navigation symbols}{}",
        r"\setbeamertemplate{headline}{}",
        r"\setbeamertemplate{footline}[frame number]",
        r"\setbeamertemplate{frametitle continuation}{}",
        r"\setbeamertemplate{frametitle}{\nointerlineskip\begin{beamercolorbox}[wd=\paperwidth,sep=0.45ex,leftskip=7.5mm,rightskip=7.5mm]{frametitle}\usebeamerfont{frametitle}\insertframetitle\end{beamercolorbox}}",
        r"\setbeamerfont{frametitle}{size=\large}",
        r"\setbeamerfont{normal text}{size=\normalsize}",
        r"\setbeamersize{text margin left=8mm,text margin right=8mm}",
        r"\setbeamercolor{keyphrase}{bg=blue!8,fg=black!85}",
        r"\setbeameroption{show notes}" if show_notes else r"\setbeameroption{hide notes}",
        r"\usepackage[T1]{fontenc}",
        r"\usepackage{tabularx}",
        r"\usepackage{array}",
        r"\usepackage{booktabs}",
        r"\usepackage{listings}",
        r"\usepackage{hyperref}",
        r"\usepackage{fancyvrb}",
        r"\lstdefinelanguage{mizar}{morekeywords={definition,end,struct,field,property,inherit,extends,where,from,let,be,being,mode,theorem,proof,thus,hence,by,registration,cluster,coherence,reduce,reducibility,import,for,holds,st,is,func,pred,attribute,algorithm,terminating,requires,ensures,do,while,invariant,decreasing,return,var,const,if,scheme,provided,environ,vocabularies,notations,constructors,registrations,theorems,begin,qua,reconsider,consider,such,that,not,or,and,implies,per,cases,set,thesis},sensitive=true,morecomment=[l]{::}}",
        "\\lstdefinelanguage{toml}{morecomment=[l]{\\#},morestring=[b]\"}",
        r"\lstset{basicstyle=\ttfamily\small,keywordstyle=\color{blue!50!black}\bfseries,commentstyle=\color{green!35!black}\itshape,stringstyle=\color{violet!70!black},showstringspaces=false,columns=fullflexible,keepspaces=true,backgroundcolor=\color{black!4},frame=single,rulecolor=\color{black!20},framesep=0.7mm,framerule=0.3pt,xleftmargin=1mm,xrightmargin=1mm,aboveskip=0.6mm,belowskip=0.8mm}",
        r"\newcommand{\statusbadge}[2]{\colorbox{#1}{\textcolor{white}{\strut\scriptsize\sffamily\bfseries #2}}}",
        r"\newcommand{\badgeMML}{\statusbadge{blue!45!black}{exact MML excerpt}}",
        r"\newcommand{\badgeSpec}{\statusbadge{green!35!black}{specification example}}",
        r"\newcommand{\badgeSketch}{\statusbadge{orange!75!black}{sketch}}",
        r"\newcommand{\deepdivetag}{\hfill\colorbox{black!15}{\textcolor{black!65}{\strut\scriptsize\sffamily deep dive}}}",
        r"\pdfstringdefDisableCommands{\def\translate#1{#1}}",
        rf"\title{{{main_title}}}",
        rf"\subtitle{{{subtitle}}}",
        r"\author{Mizar Evo project}",
        r"\date{September 2026}",
        "",
        r"\begin{document}",
        r"\begin{frame}",
        r"\titlepage",
    ]
    if title_notes:
        out.append(r"\note{")
        out.extend(render_frame_body(["Presenter script:", ""] + title_notes))
        out.append(r"}")
    out.extend([r"\end{frame}", ""])

    for kind, heading, body in units:
        if kind == "appendix":
            out.append(r"\appendix")
            continue
        if kind == "section":
            out.append(rf"\section{{{inline_tex(heading or '')}}}")
            continue
        if kind == "frame":
            is_deep = DEEP_DIVE_TAG in (heading or "")
            heading = (heading or "").replace(DEEP_DIVE_TAG, "")
            visible_body, explicit_notes = split_speaker_notes(body)
            chunks = split_frame_lines(visible_body)
            for chunk_index, chunk in enumerate(chunks):
                chunk_heading = heading
                if len(chunks) > 1:
                    chunk_heading = f"{chunk_heading} ({chunk_index + 1}/{len(chunks)})"
                chunk_notes = explicit_notes if chunk_index == 0 else []
                body_tex = render_frame_body(chunk)
                note_tex = render_frame_body(make_talk_track(chunk_heading, chunk, chunk_notes))
                title_tex = frame_title(chunk_heading)
                if is_deep:
                    title_tex += r"\deepdivetag"
                out.append(rf"\begin{{frame}}[fragile,t]{{{title_tex}}}")
                out.extend(body_tex or [r"\vfill"])
                out.append(r"\note{")
                out.extend(note_tex or [r"\vfill"])
                out.append(r"}")
                out.append(r"\end{frame}")
                out.append("")

    out.append(r"\end{document}")
    out.append("")
    return "\n".join(out)


def main() -> int:
    input_path = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_INPUT
    output_path = Path(sys.argv[2]) if len(sys.argv) > 2 else DEFAULT_OUTPUT
    notes_output_path = (
        Path(sys.argv[3])
        if len(sys.argv) > 3
        else output_path.with_name(f"{output_path.stem}_notes{output_path.suffix}")
    )
    title, units = parse_markdown(input_path)
    detail_tex = emit_beamer(title, units, show_notes=False)
    notes_tex = emit_beamer(title, units, show_notes=True)
    output_path.write_text(detail_tex, encoding="utf-8")
    notes_output_path.write_text(notes_tex, encoding="utf-8")
    source_frame_count = sum(1 for kind, _, _ in units if kind == "frame") + 1
    generated_frame_count = detail_tex.count(r"\begin{frame}")
    print(f"wrote {output_path}")
    print(f"wrote {notes_output_path}")
    print(f"source frames including title: {source_frame_count}")
    print(f"generated frames including title: {generated_frame_count}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
