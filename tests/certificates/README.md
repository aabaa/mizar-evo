# Certificate Tests

Certificate payload tests use `.cert.json` with adjacent `.expect.toml`
sidecars. Many certificate failures are independent of `.miz` parsing.

## Reject-First Corpus

The `fail/` tree is a reject-first corpus for the trusted `mizar-kernel`
acceptance boundary (pipeline phase 14). Every case is an invalid certificate
or kernel evidence object that must be rejected with the stable failure
identity recorded in its sidecar. Design references:

- `doc/design/mizar-kernel/en/soundness_argument.md` — invariant ids (E/B/F/P/
  S/I/C/R/L/D) cited by the sidecar `notes`;
- `doc/design/architecture/en/15.kernel_certificate_format.md` — evidence
  format and rejection semantics;
- `doc/design/architecture/en/16.substitution_and_binding.md` — substitution,
  alpha-conversion, and freshness rules;
- `doc/design/architecture/en/20.test_strategy.md` — reject-first priority for
  kernel-adjacent tests.

Directory split:

| Directory | Rejection focus |
|---|---|
| `fail/malformed/` | envelope/schema/profile failures and legacy certificates under normal policy |
| `fail/substitution/` | capture, freshness, free-variable, and payload-provenance failures |
| `fail/sat/` | SAT refutation failures, goal-polarity misuse, legacy resolution replay |
| `fail/symbols/` | imported-fact identity/fingerprint/status and target/context binding failures |
| `fail/resources/` | deterministic resource and step-budget non-acceptance outcomes |

## Payload Encoding

The kernel v1 evidence envelope is a binary format
(`doc/design/mizar-kernel/en/formula_evidence.md`). Until a corpus serializer
lowers seeds to canonical envelope bytes, `.cert.json` payloads here are JSON
seed projections tagged `mizar-kernel-evidence-json-seed/1` (or
`mizar-kernel-legacy-certificate-json-seed/1` for legacy certificates). Each
payload states its intended defect precisely enough for the serializer task to
produce the equivalent malformed bytes; the sidecar remains the authoritative
expected-failure contract. Sidecar metadata is validated by `mizar-test plan`
today; executable certificate checking is gated on a future
`certificate_check`/`kernel_check` runner and must not weaken any expectation
recorded here.
