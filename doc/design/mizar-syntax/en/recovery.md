# mizar-syntax: Recovery Nodes

Status: planned.

## Purpose

This module defines the syntax representation of parser recovery.

## Responsibilities

- represent missing constructs, skipped tokens, unmatched delimiters, and malformed annotations;
- mark recovered nodes so resolver and checker phases can skip or reject them explicitly;
- preserve original source spans for diagnostics.

