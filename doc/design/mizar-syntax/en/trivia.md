# mizar-syntax: Trivia

Status: planned.

## Purpose

This module defines syntax-adjacent trivia retained for diagnostics, formatting, documentation, and LSP features.

## Responsibilities

- store comments, doc comments, whitespace-sensitive attachment hints, and skipped token ranges;
- preserve enough source fidelity for formatting and code actions;
- keep doc-comment attachment syntactic and reject semantic meaning to later phases.

