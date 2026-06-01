# mizar-parser: Pratt Parsing

Status: planned.

## Purpose

This module defines the precedence parser for term and formula expressions.

## Responsibilities

- use active-lexicon operator metadata for term-level prefix, postfix, and infix forms;
- use the fixed formula connective table for formula-level precedence;
- parse syntactic shape without performing overload resolution;
- report non-associative chaining and precedence surprises with source-local diagnostics.

