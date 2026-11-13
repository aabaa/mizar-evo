# Backend-Free Instance Finder

Canonical language: English; [Japanese companion](../ja/instance_finder.md).
Authority: specification §21.7.1–§21.7.5 and [architecture 10](../../architecture/en/10.atp_backend_integration.md#evidence-extraction-by-instance-finding).

## API and boundary

`instance_finder::find_instance_evidence(&AtpProblem, &EncodedBackendProblem, &VcKernelEvidenceHandoff, Option<[u64; 4]>, Option<&[String]>) -> Result<Option<CandidateKernelEvidence>, String>` searches without a backend. `CandidateKernelEvidence` reexports the existing `BackendCandidateEvidence` transport; it grants no accepted proof status.
The structured problem must reproduce the entire encoded input and agree with the original local handoff's target, source formulas and provenance. Supported inputs use local formula evidence, prefix universally quantified premises and a ground goal; type guards must duplicate an authenticated premise exactly; encoded properties, existing substitution payloads and unsupported formula/term shapes fail closed. No premise outside the encoded problem may enter the evidence.
Advisory used-premise labels only restrict search to actual premise labels; unknown labels and conjecture labels are invalid. Backend traces, logs and reports supply no formulas or trusted facts.

## Search and limits

The caller passes [verifier-policy limits](../../mizar-proof/en/policy.md#instance-search-limits) unchanged. None returns no evidence; Some preserves literal counts including zero. Coordinates bound distinct observed/generated ground terms, premise-instantiation candidates (including empty substitutions of ground premises), named equality-instance candidates and kernel/SAT checks. Exhaustion yields no evidence.
Substitution domains contain the existing observed closed subterms; named equality search also uses the finitely generated instance subterms without extending that domain. Premise substitutions and reflexivity, symmetry, transitivity, function congruence and predicate congruence names use deterministic order. Only named equality instances are supplied; the kernel constructs their formulas.
Original source/provenance and goal bindings remain in the local wire. Every returned candidate has passed the ordinary kernel evidence checker against the original handoff context. Consumers must retain their own normal kernel acceptance and policy checks.
Backend extraction, portfolio execution, source-language extraction completeness, proof witnesses, publication and cache reuse remain separate. The rejecting task-29 joint corpus is unchanged.

## Tests

The translator tests build real structured/encoded problems and local wire: first-order instantiation; a congruence/transitivity equality chain; backend-free deterministic runs through both encoders; literal limits and exhaustion; independent kernel acceptance of every emitted candidate; stale inputs and forged reports. Six families bound this core's test matrix; no backend success report substitutes for kernel acceptance.
