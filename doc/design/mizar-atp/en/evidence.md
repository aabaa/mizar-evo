# Vampire Evidence Extraction

Canonical language: English; [Japanese companion](../ja/evidence.md).
Status: guarded Vampire 4.9 extraction implemented in `backend::run_vampire_backend`; candidates come only from the existing kernel-checked finder. The frozen contract remains the design record.
Authority: [specification §21.7.1–§21.7.5](../../../spec/en/21.source_code_annotation_and_atp.md#217-atp-integration), [architecture 10](../../architecture/en/10.atp_backend_integration.md#evidence-extraction-by-instance-finding), and [backend contract](backend.md).

## Guarded route and input

Vampire is the only backend in this route. Use the existing [generic runner](backend.md#process-model), [profile/command](backend.md#backend-profile-and-command), version probes, private stdin/file input, captured streams, policy timeout/resource limits, exit/stream fingerprints and private-temp cleanup. No daemon, library linking, shell interpretation or fallback backend is introduced.
Send only the existing deterministic [TPTP FOF encoding](tptp_encoder.md) for the declared profile, produced by the translator from the encoded problem. No source text, extra premises, widened premise set or backend-specific input additions are allowed. Bind every produced record to the encoded-problem hash; retain the original target, formula-label, symbol-binding and provenance metadata.

The guarded API takes the existing run input, original structured problem and VC/kernel handoff, and unchanged optional finder limits. It requires `--mode vampire --input_syntax tptp --proof tptp --output_axiom_names on`, an empty environment allowlist, and absolute command/probe paths to the same executable with a `--version` probe.

## Extraction and advisory mapping

Extraction maps one completed run's captured output to the existing [result-classification](backend.md#result-classification) input and an optional advisory used-premise report. It accepts nothing and produces no trusted evidence. From stdout take only the SZS status, reported encoded-problem formula names, and backend identity/version or reproducibility metadata; stderr and remaining output are diagnostic only.
An advisory report is the set of input formula names reported as used: read `file(input,name)` annotations only on axiom-role input records inside the SZS Proof block; conjecture-role records are excluded. Formulas and inference annotations stay opaque. Resolve the entire set through the encoded problem's formula labels and symbol bindings to original premises, in deterministic label order with duplicates removed. Pass only actual premise labels to the finder; conjecture labels are not premises. The report may narrow search but cannot supply formulas, substitutions, additional premises or trusted `used_axioms`.
Proof objects, TSTP traces, inference steps and substitutions inside proofs never enter a candidate payload. Reading input-name references for the advisory set does not translate a trace into evidence; all proof content remains diagnostic.

## Finder handoff and trust

Use exclusively the existing [instance finder API and supported-input boundary](instance_finder.md#api-and-boundary), with its original structured problem, encoded problem and VC/kernel handoff. No new candidate schema is defined. `CandidateKernelEvidence` / `BackendCandidateEvidence` must carry explicit kernel-schema formula/substitution payload bytes or an explicit payload reference, target binding, encoded-problem hash and provenance, including matching `formula_label_refs` / `symbol_binding_refs`; metadata alone is not evidence ([candidate boundary](backend.md#candidate-evidence-boundary)).
Advisory labels only restrict premise search. The finder produces substitutions and named equality-instance candidates; the kernel constructs equality formulas and independently checks candidates. Preserve [verifier-policy search limits](instance_finder.md#search-and-limits), authenticated inputs and deterministic ordering. Backend output supplies no goal polarity, context-identity payload or imported-statement projection: task-29's upstream boundary remains mandatory.
Backend success without finder candidates yields no kernel evidence and cannot be `Proved` under the existing classifier. Any externally attested/open outcome belongs to downstream policy; extraction grants no acceptance, proof status, witness publication or cache promotion.

## Failure and determinism

Apply [existing failure semantics](backend.md#failure-semantics) before candidate handoff: timeout, cancellation, crash, nonzero exit, incomplete/corrupt capture, missing/malformed SZS status or syntactically unparseable advisory output produces no candidates from the run. Unsupported or mismatched observed results never become `Proved`.
A syntactically parsed advisory report that cannot be mapped completely, including an unknown or non-premise label, is discarded whole with a diagnostic. Never partially narrow: run the finder as if no report existed, over all encoded premises. This mapping fallback does not waive process, status, version, original-handoff or policy checks. An absent report likewise uses all encoded premises.
Extraction is a pure deterministic function of the encoded problem, captured stdout bytes and pinned version record. The same inputs yield the same advisory set and handoff; with the same original handoff and policy limits, finder candidate ids and hashes are identical.

## Executable provisioning

The outer orchestrator provisions Vampire 4.9 from the [official release distribution](https://github.com/vprover/vampire/releases) outside the sandbox, after this freeze and before implementation dispatch. Install the executable at `.tools/provers/vampire-4.9/vampire` inside this checkout; `.tools/` is ignored tooling. No purchase, licence acquisition, credential or private binary is part of this open-source route. This freeze installs nothing.
At provisioning, compute the binary's SHA-256 and record it in the provisioning/implementation run record, never in live design documents. Before any solve, the existing runner's version probe must identify Vampire and self-reported version 4.9. Missing executable, failed probe or version mismatch means backend unavailable: refuse the route without solving, substitution or fallback. Preserve backend identity/version and command fingerprint in run metadata.

## Deferred ownership

Portfolio execution, CVC5/Z3/E adapters, SMT-LIB extraction, TFF/typed paths (ATP-G-010/ATP-G-012), proof policy, witness/cache/artifact integration and externally attested handling remain deferred to their owners. CVC5 remains primary for theory-heavy goals and Z3 optional/diagnostic; neither joins this route. Step 7 item 1 stays open. Implementation must exercise the frozen contract's real encoded-problem matrix and guarded executable before recording ATP-G-015/ATP-G-016 resolved; provisioning alone is insufficient.
