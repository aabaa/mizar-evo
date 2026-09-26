# Task ATPK-KERNEL-EQUALITY-INSTANCES: kernel-constructed equality instances
Canonical language: English; [Japanese pointer](../ja/ATPK-KERNEL-EQUALITY-INSTANCES.md).
Status: planned. Tier: full. Owner: [kernel plan](../../mizar-kernel/en/00.crate_plan.md); consumers: [atp plan](../../mizar-atp/en/00.crate_plan.md), [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: the current formula/substitution evidence parser and SAT encoding, including the equality reflexivity rule.
Authority: specification §21.7.1 and §21.7.5 (equality axiom instances, E0352); [architecture 15](../../architecture/en/15.kernel_certificate_format.md#equality-axiom-instances-are-kernel-constructed).
Gap: `source_drift`; the kernel accepts no equality instance names. Approved kernel-scope expansion.

## Scope

Extend `formula_evidence` with equality-instance names (reflexivity, symmetry,
transitivity, function congruence, predicate congruence) carrying the schema,
the manifest symbol and arity for congruence, and the actual terms. The kernel
constructs each instance, adds it to the derived formulas before encoding, and
rejects an unknown schema, a symbol outside the manifest, an arity mismatch, or
an ill-formed term as invalid substitution evidence. Update the owner
documents [formula evidence](../../mizar-kernel/en/formula_evidence.md),
[SAT encoding](../../mizar-kernel/en/sat_encoding.md), and the
[soundness argument](../../mizar-kernel/en/soundness_argument.md) in the same task.

## Forbidden

Accepting a supplied instance formula; searching for or generating unnamed
instances; congruence closure or any change to the SAT checker; linear
arithmetic; weakening the 23-case rejecting certificate corpus.

## Tests and exit

Acceptance of `a = b, f(a) = c ⊢ f(b) = c` with named congruence, symmetry, and
transitivity instances, and its rejection when any needed name is missing;
predicate congruence; rejection of every malformed-name class; determinism of
the derived encoding. Require independent specification, test-sufficiency,
implementation, volume/scope, and consistency reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: named equality instances are the only new derived formulas, every one is
kernel-constructed, and every malformed name is rejected.
