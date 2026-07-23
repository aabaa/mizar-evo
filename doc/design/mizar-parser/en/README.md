# mizar-parser

`mizar-parser` implements the syntax grammar for Mizar Evo.

Current Task-46 status: concrete infix, prefix, and postfix operator
declarations are parsed as dedicated `OperatorDeclaration` nodes at annotated,
visible top-level and definition-local notation positions. Completed frontend
Task 20 already supplied the named position-sensitive string context and local
operator metadata handoff. Task 46 is syntax-only: it does not activate an
operator or mutate `ParseRequest::operator_fixity`. The post-Task-46 parser
milestone meets all nine closeout hard gates; its fresh independent read-only
score is 99/100.

It should keep a narrow dependency on parser-facing token transfer objects and
syntax structures: frontend-adapted tokens in, `SurfaceAst` plus syntax
diagnostics out. Parser-assisted lexing is allowed only through explicit context
objects such as string-required positions and symbol-kind filters.

Status: the crate exposes a parser entry point that consumes the
frontend-adapted token transfer object with session `SourceRange`s and returns
`mizar_syntax::SurfaceAst` plus syntax diagnostics. Task 2 parser
infrastructure is in place as private cursor, syntax-event, expected-token
diagnostic, synchronization, and recovery-emission helpers. The parser
workstream has grown through parser task 42, including module/import/export,
type/term/formula, statement/proof, definition, structure, registration,
template, algorithm, verification-clause, annotation surfaces, and the
predicate redefinition label repair, plus recovery consolidation and fail-corpus
expansion, parse-only `SurfaceAst` snapshot baselines, parser determinism
coverage, the parser-owned valid-UTF-8 fuzz target, and frontend passthrough
follow-through audit, plus a private annotation/test module-boundary split.
Tasks 43-45 and 47-48 are complete, and Task 46 now closes the concrete
operator-declaration syntax gap. All parser Tasks 1-48 are implemented.
P-265-47D remains a nonblocking human-owned wording gap. The independently
classified overbroad frontend string-position heuristic remains external to
parser scope. Global Step 5 is not closed, and Task 49 and Steps 6/7 are not
authorized.

Module specs and audits:

- `00.crate_plan.md`
- `grammar.md`
- `pratt.md`
- `recovery.md`
- `source_spec_audit.md`
- `bilingual_documentation_synchronization.md`
- `crate_exit_report.md`

Implementation roadmap: [todo.md](./todo.md).
