# mizar-frontend: Source Loading

Status: planned.

## Purpose

This module defines source loading for `.miz` files before preprocessing and lexing.

## Responsibilities

- read source bytes from disk or editor-provided buffers;
- validate UTF-8 and strip one leading UTF-8 BOM when allowed;
- derive package-relative module paths;
- compute source hashes;
- build line maps and loading maps using `mizar-session` source identity types;
- produce file-level diagnostics without parsing syntax.

