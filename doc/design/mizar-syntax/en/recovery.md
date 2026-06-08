# mizar-syntax: Recovery Nodes

Status: minimal task-12 recovery nodes implemented; full recovery vocabulary planned.

## Purpose

This module defines the syntax representation of parser recovery.

## Responsibilities

- represent missing constructs, skipped tokens, unmatched delimiters, and malformed annotations;
- mark recovered nodes so resolver and checker phases can skip or reject them explicitly;
- preserve original source spans for diagnostics.

Current minimal vocabulary includes recovered token nodes for lexer error tokens
and explicit recovered nodes for missing `end` and missing string literals.
Broader skipped-token, unmatched delimiter, and malformed-annotation recovery
remains planned.
