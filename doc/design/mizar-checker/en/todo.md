# mizar-checker TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).
> Compacted 2026-09-02 (batch CPT-02, rules in
> [../../documentation_compaction_rules.md](../../documentation_compaction_rules.md)):
> completed task bodies and closed addenda-section bodies moved verbatim to
> [../../archive/checker_todo_sections.md](../../archive/checker_todo_sections.md).
> Every heading, every registered ledger redirect line, and every section
> with open work remains below.

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

Module specs do not exist yet; each is written by its own spec task (English
and Japanese in the same change) before the implementation tasks that cite it.
Module names follow the minimum split of
[internal 07](../../internal/en/07.crate_module_layout.md); the crate refines
architecture 04, 05, 16, 17, 18, and 19.

| Module | Spec | Source | Status |
|---|---|---|---|
| typed_ast | `typed_ast.md` (task 2) | `src/typed_ast.rs` | [x] |
| binding_env | `binding_env.md` (task 4) | `src/binding_env.rs` | [x] |
| type_checker | `type_checker.md` (task 6) | `src/type_checker.rs` | [~] |
| registration_resolution | `registration_resolution.md` (task 13) | `src/registration_resolution.rs` | [~] |
| cluster_trace | `cluster_trace.md` (task 15) | `src/cluster_trace.rs` | [~] |
| overload_resolution | `overload_resolution.md` (task 21) | `src/overload_resolution.rs` | [~] |
| resolved_typed_ast | `resolved_typed_ast.md` (task 27) | `src/resolved_typed_ast.rs` | [~] |

`mizar-checker` implements pipeline phases 6-8: `ResolvedAst` plus `SymbolEnv`
in, `TypedAst`, `ResolutionTrace`, and `ResolvedTypedAst` out. It is built in
three waves matching the phases: type checking (phase 6), cluster/registration
resolution with replayable traces (phase 7), and overload resolution (phase 8).
Soft types are semantic metadata: every fact must remain explainable as a
logical predicate or a registration-derived fact, and no wave performs proof
search.

Dependency order: `typed_ast` data → `binding_env` / `type_checker` (wave 1)
→ `registration_resolution` / `cluster_trace` (wave 2) →
`overload_resolution` / `resolved_typed_ast` (wave 3).

Each task below is deliberately small — one module spec, or one behavior slice
of one module — so that a single task can be implemented, tested, and
committed autonomously without holding the rest of the crate in flight.

## Crate Prerequisites

The production crate depends on `mizar-session` and `mizar-resolve` and reaches
`mizar-syntax` only transitively. Task 258B1 adds a direct test-only
`mizar-syntax` dev-dependency for parser-shaped corruption fixtures; production
source remains syntax-free. Wave 1 needs `mizar-resolve` tasks 14 and 20 (name
resolution, `SymbolEnv` skeleton); waves grow with `mizar-resolve` task 21
signature increments and the corresponding `mizar-parser` definition grammar
tasks (23-31). Architecture:
[04.type_and_registration_resolution.md](../../architecture/en/04.type_and_registration_resolution.md),
[05.overload_resolution.md](../../architecture/en/05.overload_resolution.md),
[16.substitution_and_binding.md](../../architecture/en/16.substitution_and_binding.md),
[17.cluster_trace_format.md](../../architecture/en/17.cluster_trace_format.md);
crate ownership: [internal 07](../../internal/en/07.crate_module_layout.md).

## Resolved And Open Decisions

- **TypedAst arena representation: resolved by task 3.** `TypedAst` uses a
  homogeneous `TypedNodeKind` arena with dense local ids, mirroring the current
  `mizar-syntax` compatibility view and `mizar-resolve` arena style. Task 3
  does not add a direct `mizar-syntax` dependency for node-kind storage; it uses
  a checker-local source-shape projection. `ResolvedTypedAst` revisits the same
  decision in task 28.
- **Registration activation gating: resolved by task 19.** Local
  registrations must not affect automatic inference until their proof
  obligations are accepted by the configured verifier policy (architecture 04
  constraints). Task 19 implements the interim policy because phases 11-14 do
  not exist yet: generated obligations are recorded with pending/unverified
  status, and registrations do not enter the active database until an explicit
  accepted verifier/artifact status input is available. Registered at the top
  level; revisited when `mizar-vc`/`mizar-proof` land.
- **Trace schema conformance: resolved.**
  [17.cluster_trace_format.md](../../architecture/en/17.cluster_trace_format.md)
  is the canonical `ResolutionTrace` schema; `cluster_trace.md` refines it,
  it does not fork it.
- **Diagnostics record: follows the `mizar-resolve` decision** on
  `mizar-diagnostics` adoption timing; the checker uses whatever record the
  resolver adopted. Registered at the top level.
- **Constructor property value source: resolved by task 35.** Default
  structure constructors accept fields only; `property` values come only from
  Chapter 7 property implementations. Task 35 updates spec 05/07 in English
  and Japanese, adds a reject-first inactive `advanced_semantics` seed, and
  records traceability without changing checker/core source semantics.

## Ordered Task List

Keep `cargo test -p mizar-checker` green after each task (see
[Recommended Verification](#recommended-verification)).

### Wave 1: type checking (phase 6)

1. **Crate scaffold and lint-policy guard.** [x]
2. **Spec: `typed_ast.md`.** [x]
3. **Implement `typed_ast` data shapes.** [x]
4. **Spec: `binding_env.md`.** [x]
5. **Binding environment and context build.** [x]
6. **Spec: `type_checker.md`.** [x]
7. **Type-expression normalization.** [x]
8. **Declaration and local-binding checking.** [x]
9. **Term and formula type inference.** [x]
10. **Coercion candidates, sethood, non-emptiness, and narrowing obligations.** [x]
11. **Type-fact recording and queries.** [x]
12. **Corpus runner at stage `type_elaboration`.** [x]

### Wave 2: cluster and registration resolution (phase 7)

13. **Spec: `registration_resolution.md`.** [x]
14. **Registration index.** [x]
15. **Spec: `cluster_trace.md`.** [x]
16. **Cluster resolution closure with trace recording.** [x]
17. **Cluster loop detection and bounded saturation.** [x]
18. **Reduction applications.** [x]
19. **Pending-registration validation and activation gating.** [x]
20. **Existential gating of attributed type use.** [x]

### Wave 3: overload resolution (phase 8)

21. **Spec: `overload_resolution.md`.** [x]
22. **Candidate site collection.** [x]
23. **Template expansion.** [x]
24. **Viability filtering.** [x]
25. **Specificity graph construction.** [x]
26. **Root selection, refinement joins, and view insertion.** [x]
27. **Spec: `resolved_typed_ast.md`.** [x]
28. **`ResolvedTypedAst` assembly.** [x]

### Hardening and cross-cutting follow-ups

29. **Deferred corpus obligations at stages `formula_statement` and `advanced_semantics`.** [x]
30. **Determinism suite.** [x]
31. **Public-enum forward-compatibility policy.** [x]
32. **Source/spec correspondence audit.** [x]
33. **Bilingual documentation sync audit.** [x]
34. **Module-boundary refactor gate.** [x]

### Wave 4: semantic-audit follow-ups (2026-07-03)

[semantic_spec_audit.md](./semantic_spec_audit.md) audited the checker-scoped
specification chapters (03, 05-08, 13, 14, 17-19) and recorded findings
SSA-001 through SSA-020 plus a 16-fixture adversarial rejection corpus. The
tasks below convert every finding into either an owned task or an explicit
disposition. Spec-decision tasks (35-44) come first because AGENTS.md places
`doc/spec/en/` above design docs and code: the checker must not implement
behavior the spec has not decided. Each spec task chooses among the audit's
proposed resolutions (or records a superior one), updates `doc/spec/en/` and
`doc/spec/ja/` in the same change, adds or activates reject-first corpus
seeds where the decision creates new rejections, and updates
`tests/coverage/spec_trace.toml`.

Finding dispositions (every SSA id maps to a task or a recorded reason):

| Finding | Disposition |
|---|---|
| SSA-001 | task 35 |
| SSA-002, SSA-011, SSA-012 | task 36 |
| SSA-003, SSA-010, SSA-016, SSA-019 | task 37 |
| SSA-004 | task 38 |
| SSA-005 | task 39 |
| SSA-006 | task 40 |
| SSA-007, SSA-008, SSA-020 | task 41 |
| SSA-009 | task 42 |
| SSA-013, SSA-014 | task 43 |
| SSA-015, SSA-017 | task 44 |
| SSA-018 | no task: the greedy `of`/`over` parse is deterministic and documented (spec 19.6.4); a scope-sensitivity lint belongs to the future diagnostics-adoption wave and is recorded in that wave, not here |
| corpus seeds | task 49 activates the 16 audit fixtures plus the task-35 constructor-property seed, task-36 duplicate-coverage seed, task-37 ordinary/template-derived equivalent-root seed, task-38 functorial-`for` guard seed, task-39 property-overlap coherence seed, and task-44 omitted-`reconsider`/ambiguous-redefinition-target seeds when the required runners, parser support, declaration-symbol support, and source-to-checker payload extraction land; Resolver Task 31 solely activates the task-37 same-return signature-conflict seed through `declaration_symbol`, and task 49 only reconciles/deduplicates that member with the exact 24-fixture set |

35. **Spec decision: constructor property arguments vs extensionality (SSA-001).** [x]
36. **Spec decision: structure member identity, upcast paths, acyclicity (SSA-002, SSA-011, SSA-012).** [x]
37. **Spec decision: overload tie-break and tie ambiguity (SSA-003, SSA-010, SSA-016, SSA-019).** [x]
38. **Spec decision: functorial cluster `for T` semantics (SSA-004).** [x]
39. **Spec decision: property-implementation coherence (SSA-005).** [x]
40. **Spec contract: registration activation timing (SSA-006).** [x]
41. **Spec clarifications: closure termination, contradiction site, `attr(args)` (SSA-007, SSA-008, SSA-020).** [x]
42. **Spec clarification: reduction determinism signature (SSA-009).** [x]
43. **Spec clarification: sethood for dependent modes and built-in inhabitation (SSA-013, SSA-014).** [x]
44. **Spec clarification: `reconsider` discharge and ambiguous redefinition target (SSA-015, SSA-017).** [x]
45. **Checker alignment: overload tie-break implementation.** [x]
46. **Checker alignment: closure contradiction and termination rules.** [x]
47. **Checker alignment: existential gate and activation contract.** [x]
48. **Reserve source declaration producer seam.** [x]
49. **Audit-corpus activation and task-29 record revision.** [ ]
    - When the `advanced_semantics`/`formula_statement` runners,
      property-implementation parser support, and source-to-checker payload
      extraction land (mizar-test runner growth +
      MC-G020/MC-G021/MC-G023/MC-G027 plus MC-G030/property-implementation
      payload extraction for the task-39 seed), activate the 16 semantic-audit
      fixtures plus the task-35 constructor-property seed, task-36
      duplicate-coverage seed, task-37 ordinary/template-derived
      equivalent-root ambiguity seeds, task-38 functorial-`for` guard seed,
      task-39 property-overlap coherence seed, and task-44 omitted-`reconsider`
      / ambiguous-redefinition-target seeds. The exact scope is the
      24-fixture reconciliation set in
      [payload_family_decomposition.md](./payload_family_decomposition.md):
      resolver Task 31 solely activates its same-return member through
      `declaration_symbol`; Task 49 activates the remaining 23 and then
      reconciles/deduplicates all 24. The same-signature/different-return
      fixture is already active outside the set and must remain an unmodified
      control rather than being reactivated. Revise the
      task-29 deferred corpus records to point at (or be superseded by) the
      audit requirement ids.
    - Acceptance: `mizar-test` plan shows the fixtures active with zero plan
      errors; deferred records no longer double-count them.
    - Verify: `cargo test -p mizar-test`.
    - Deps: completed tasks 35-44; parser Tasks 47-48; resolver Task 31;
      completed checker Task 247; checker Tasks 248-264 and 269-279, including
      the external accepted-status gate of blocked-reserved Task 274 and
      external scheme/theorem-role Gate S1; mizar-test Task-10 increments
      `MT10-FS` and `MT10-AS`. Tasks 266-268 alone are insufficient. Refs:
      [payload_family_decomposition.md](./payload_family_decomposition.md),
      semantic_spec_audit.md
      "Adversarial Corpus".

50. **Source-derived attributed reserve evidence-gap bridge.** [x]
51. **Source-derived local mode reserve expansion-gap bridge.** [x]
52. **Source-derived local structure reserve evidence-gap bridge.** [x]
53. **Source-derived attributed local structure reserve evidence-gap bridge.** [x]
54. **Source-derived attributed local mode reserve expansion-gap bridge.** [x]
55. **Source-derived bare local mode expansion bridge.** [x]
56. **Source-derived local mode expansion chain bridge.** [x]
57. **Source-derived local mode structure-RHS evidence-gap bridge.** [x]
58. **Source-derived local mode attributed-builtin RHS evidence-gap bridge.** [x]
59. **Source-derived attributed local mode reserve evidence-gap bridge.** [x]
60. **Source-derived attributed local mode structure-RHS evidence-gap bridge.** [x]
61. **Source-derived attributed local mode attributed-builtin-RHS evidence-gap bridge.** [x]
62. **Add source-derived local mode structure-RHS chain evidence-gap bridge.** [x]
63. **Add source-derived local mode attributed-builtin-RHS chain evidence-gap bridge.** [x]
64. **Add source-derived attributed local mode bare-builtin chain evidence-gap bridge.** [x]
65. **Add source-derived attributed local mode structure-RHS chain evidence-gap bridge.** [x]
66. **Add source-derived attributed local mode attributed-builtin-RHS chain evidence-gap bridge.** [x]
67. **Add source-derived structure-qualified attribute gap boundary.** [x]
68. **Add source-derived argument-bearing mode reserve gap boundary.** [x]
69. **Add source-derived argument-bearing structure reserve gap boundary.** [x]
70. **Add source-derived bracket-form local mode reserve gap boundary.** [x]
71. **Add source-derived bracket-form local structure reserve gap boundary.** [x]
72. **Add source-derived two-edge bare local mode chain bridge.** [x]
73. **Add source-derived three-edge bare local mode chain bridge.** [x]
74. **Add source-derived structural bare local mode chain bridge.** [x]
75. **Add source-derived local mode forward-reference active-range boundary.** [x]
76. **Add source-derived local structure forward-reference active-range boundary.** [x]
77. **Add source-derived local attribute forward-reference active-range boundary.** [x]
78. **Add source-derived imported structure reserve extraction-gap boundary.** [x]
79. **Add source-derived imported mode reserve extraction-gap boundary.** [x]
80. **Add source-derived imported attribute reserve extraction-gap boundary.** [x]
81. **Add source-derived argument-bearing local attribute reserve extraction-gap boundary.** [x]
82. **Add source-derived imported mode reserve provenance bridge.** [x]
83. **Add source-derived imported structure reserve provenance bridge.** [x]
84. **Add source-derived imported attribute reserve provenance bridge.** [x]
85. **Add source-derived imported non-empty attribute reserve provenance bridge.** [x]
86. **Add source-derived theorem formula extraction-gap boundary.** [x]
115. **Add exact source-derived formula statement recovery checker bridge.** [x]
116. **Add source-derived imported positive empty attribute reserve provenance bridge.** [x]
171. **Add source-derived imported negative empty object reserve provenance bridge.** [x]
117. **Add source-derived formula constant kind checker bridge.** [x]
118. **Tighten builtin binary theorem exact-token guard.** [x]
119. **Add exact source-derived reserved-variable equality checker bridge.** [x]
120. **Add exact source-derived reserved-variable membership checker bridge.** [x]
121. **Add exact source-derived reserved-variable inequality checker bridge.** [x]
122. **Add checker reflexive type-assertion admissibility and its exact reserved-variable source bridge.** [x]
123. **Add exact source-derived distinct reserved-variable equality checker bridge.** [x]
124. **Add exact source-derived multiple-reserve-declaration equality checker bridge.** [x]
125. **Add exact source-derived heterogeneous-reserve membership checker bridge.** [x]
126. **Add exact direct-local-mode reserved-variable equality checker bridge.** [x]
127. **Add exact one-edge local-mode-chain reserved-variable equality checker bridge.** [x]
128. **Add exact direct local-object-mode reserved-variable equality checker bridge.** [x]
129. **Add exact one-edge local-object-mode-chain reserved-variable equality checker bridge.** [x]
130. **Add exact direct-local-mode reserved-variable inequality checker bridge.** [x]
131. **Add exact direct-local-object-mode reserved-variable inequality checker bridge.** [x]
132. **Add exact one-edge local-mode-chain reserved-variable inequality checker bridge.** [x]
133. **Add exact one-edge local-object-mode-chain reserved-variable inequality checker bridge.** [x]
134. **Add exact two-edge local-mode-chain reserved-variable equality checker bridge.** [x]
135. **Add exact two-edge local-object-mode-chain reserved-variable equality checker bridge.** [x]
136. **Add exact two-edge local-mode-chain reserved-variable inequality checker bridge.** [x]
137. **Add exact two-edge local-object-mode-chain reserved-variable inequality checker bridge.** [x]
138. **Add exact direct-local-mode reserved-variable normalized-reflexive type assertion checker bridge.** [x]
139. **Add exact direct-local-mode left reserved-variable membership checker bridge.** [x]
140. **Add exact direct-local-object-mode left reserved-variable membership checker bridge.** [x]
141. **Add exact one-edge local-mode-chain left reserved-variable membership checker bridge.** [x]
142. **Add exact one-edge local-object-mode-chain left reserved-variable membership checker bridge.** [x]
143. **Add exact two-edge local-mode-chain left reserved-variable membership checker bridge.** [x]
144. **Add exact two-edge local-object-mode-chain left reserved-variable membership checker bridge.** [x]
145. **Add exact direct local-object-mode reserved-variable normalized-reflexive type assertion checker bridge.** [x]
146. **Add exact one-edge local-mode-chain reserved-variable normalized-reflexive type assertion checker bridge.** [x]
147. **Add exact one-edge local-object-mode-chain reserved-variable normalized-reflexive type assertion checker bridge.** [x]
148. **Add exact two-edge local-mode-chain reserved-variable normalized-reflexive type assertion checker bridge.** [x]
149. **Add exact two-edge local-object-mode-chain reserved-variable normalized-reflexive type assertion checker bridge.** [x]
150. **Add exact three-edge local-mode-chain reserved-variable normalized-reflexive type assertion checker bridge.** [x]
151. **Add exact three-edge local-object-mode-chain reserved-variable normalized-reflexive type assertion checker bridge.** [x]
152. **Add exact four-edge local-mode-chain reserved-variable normalized-reflexive type assertion checker bridge.** [x]
153. **Add exact four-edge local-object-mode-chain reserved-variable normalized-reflexive type assertion checker bridge.** [x]
154. **Add exact three-edge local-mode-chain reserved-variable equality checker bridge.** [x]
155. **Add exact three-edge local-object-mode-chain reserved-variable equality checker bridge.** [x]
156. **Add exact three-edge local-mode-chain reserved-variable inequality checker bridge.** [x]
157. **Add exact three-edge local-object-mode-chain reserved-variable inequality checker bridge.** [x]
158. **Add exact three-edge local-mode-chain left reserved-variable membership checker bridge.** [x]
159. **Add exact distinct-binding shared-reserve membership checker bridge.** [x]
160. **Add exact distinct-binding shared-reserve inequality checker bridge.** [x]
161. **Add exact multiple-reserve-declaration inequality checker bridge.** [x]
162. **Add exact multiple-reserve-declaration membership checker bridge.** [x]
87. **Add source-derived term formula extraction-gap boundary.** [x]
88. **Add source-derived proof skeleton extraction-gap boundary.** [x]
89. **Add source-derived statement proof extraction-gap boundary.** [x]
90. **Add source-derived predicate/functor definition extraction-gap boundary.** [x]
91. **Add source-derived attribute definition extraction-gap boundary.** [x]
92. **Add source-derived mode/structure definition extraction-gap boundary.** [x]
93. **Add source-derived proof-local declaration extraction-gap boundary.** [x]
94. **Add source-derived proof-local inline definition extraction-gap boundary.** [x]
95. **Add source-derived registration block extraction-gap boundary.** [x]
96. **Add source-derived redefinition/notation extraction-gap boundary.** [x]
97. **Add source-derived imported TypeCaseStruct reserve provenance bridge.** [x]
98. **Add source-derived imported predicate/functor term-formula extraction-gap boundary.** [x]
99. **Add source-derived formula connective/quantifier extraction-gap boundary.** [x]
112. **Add exact source-derived formula connective/quantifier shell checker bridge.** [x]
100. **Add source-derived builtin membership formula extraction-gap boundary.** [x]
101. **Add source-derived builtin inequality formula extraction-gap boundary.** [x]
102. **Add source-derived builtin type assertion formula extraction-gap boundary.** [x]
103. **Add source-derived imported attribute assertion formula extraction-gap boundary.** [x]
104. **Add source-derived attribute-level non-empty imported attribute assertion formula extraction-gap boundary.** [x]
114. **Add exact source-derived attribute-level non-empty imported attribute assertion theorem checker bridge.** [x]
105. **Add source-derived set-enumeration formula extraction-gap boundary.** [x]
111. **Add exact source-derived set-enumeration theorem checker bridge.** [x]
106. **Add source-derived builtin equality theorem term/formula checker bridge.** [x]
108. **Add source-derived builtin membership theorem term/formula checker bridge.** [x]
110. **Add source-derived imported predicate/functor theorem checker bridge.** [x]
163. **Add exact three-edge local-object-mode membership checker bridge.** [x]
164. **Add exact four-edge local-mode membership checker bridge.** [x]
165. **Add exact four-edge local-object-mode membership checker bridge.** [x]
166. **Add exact four-edge local-mode equality checker bridge.** [x]
167. **Add exact four-edge local-object-mode equality checker bridge.** [x]
168. **Add exact four-edge local-mode inequality checker bridge.** [x]
169. **Add exact four-edge local-object-mode inequality checker bridge.** [x]
172. **Add exact local-mode long-chain equality checker bridge.** [x]
173. **Add exact local-mode long-chain inequality checker bridge.** [x]
174. **Add exact local-mode long-chain membership checker bridge.** [x]
175. **Add exact local-mode long-chain type assertion checker bridge.** [x]
176. **Add exact local-object-mode long-chain equality checker bridge.** [x]
177. **Add exact local-object-mode long-chain inequality checker bridge.** [x]
178. **Add exact local-object-mode long-chain membership checker bridge.** [x]
179. **Add exact local-object-mode long-chain type assertion checker bridge.** [x]
180. **Add exact source-derived contradiction formula-constant checker bridge.** [x]
181. **Repair exact imported attributed-reserve routing.** [x]
182. **Add exact formula-side local-mode asserted-head checker bridge.** [x]
183. **Add exact object-terminal formula-side local-mode asserted-head checker bridge.** [x]
184. **Add exact one-edge formula-side local-mode asserted-head checker bridge.** [x]
185. **Add exact one-edge object-terminal formula-side local-mode asserted-head checker bridge.** [x]
186. **Add exact two-edge formula-side local-mode asserted-head checker bridge.** [x]
187. **Add exact two-edge object-terminal formula-side local-mode asserted-head checker bridge.** [x]
188. **Add exact builtin-object reserved-variable equality checker bridge.** [x]
189. **Add exact builtin-object reserved-variable type-assertion checker bridge.** [x]
190. **Add exact builtin-object reserved-variable inequality checker bridge.** [x]
191. **Add exact distinct-binding shared-builtin-object equality checker bridge.** [x]
192. **Add exact distinct-binding shared-builtin-object inequality checker bridge.** [x]
193. **Add exact multiple-reserve-declaration builtin-object equality checker bridge.** [x]
194. **Add exact multiple-reserve-declaration builtin-object inequality checker bridge.** [x]
195. **Add exact three-edge formula-side local-mode asserted-head checker bridge.** [x]
196. **Add exact three-edge object-terminal formula-side local-mode asserted-head checker bridge.** [x]
197. **Add exact four-edge formula-side local-mode asserted-head checker bridge.** [x]
198. **Add exact four-edge object-terminal formula-side local-mode asserted-head checker bridge.** [x]
199. **Add exact seven-expansion set-terminal formula-side local-mode asserted-head checker bridge.** [x]
200. **Add exact seven-expansion object-terminal formula-side local-mode asserted-head checker bridge.** [x]
201. **Add exact one-edge formula-side immediate-radix local-mode asserted-head checker bridge.** [x]
202. **Add exact one-edge object-terminal formula-side immediate-radix local-mode asserted-head checker bridge.** [x]
203. **Add exact two-edge set-terminal formula-side immediate-radix local-mode asserted-head checker bridge.** [x]
204. **Add exact two-edge object-terminal formula-side immediate-radix local-mode asserted-head checker bridge.** [x]
205. **Add exact three-edge set-terminal formula-side immediate-radix local-mode asserted-head checker bridge.** [x]
206. **Add exact three-edge object-terminal formula-side immediate-radix local-mode asserted-head checker bridge.** [x]
207. **Add exact four-edge set-terminal formula-side immediate-radix local-mode asserted-head checker bridge.** [x]
208. **Add exact four-edge object-terminal formula-side immediate-radix local-mode asserted-head checker bridge.** [x]
209. **Add exact seven-expansion set-terminal formula-side immediate-radix local-mode asserted-head checker bridge.** [x]
210. **Add exact seven-expansion object-terminal formula-side immediate-radix local-mode asserted-head checker bridge.** [x]
211. **Add exact two-edge set-terminal formula-side two-hop local-mode asserted-head checker bridge.** [x]
212. **Add exact two-edge object-terminal formula-side two-hop local-mode asserted-head checker bridge.** [x]
213. **Add exact three-edge set-terminal formula-side two-hop local-mode asserted-head checker bridge.** [x]
214. **Add exact three-edge object-terminal formula-side two-hop local-mode asserted-head checker bridge.** [x]
215. **Add exact four-edge set-terminal formula-side two-hop local-mode asserted-head checker bridge.** [x]
216. **Add exact four-edge object-terminal formula-side two-hop local-mode asserted-head checker bridge.** [x]
217. **Add exact three-edge set-terminal formula-side three-hop local-mode asserted-head checker bridge.** [x]
218. **Add exact three-edge object-terminal formula-side three-hop local-mode asserted-head checker bridge.** [x]
219. [x] **Bridge the exact four-edge set-terminal three-hop asserted head.**
220. [x] **Bridge the exact four-edge object-terminal three-hop asserted head.**
221. [x] **Bridge the exact four-edge set-terminal full-distance four-hop asserted head.**
222. [x] **Bridge the exact four-edge object-terminal full-distance four-hop asserted head.**
223. [x] **Bridge the exact transparent single-parenthesized reserved-variable equality.**
224. [x] **Bridge the exact seven-expansion set-terminal two-hop asserted head.**
225. [x] **Bridge the exact seven-expansion object-terminal two-hop asserted head.**
226. [x] **Bridge the exact seven-expansion set-terminal three-hop asserted head.**
227. [x] **Bridge the exact seven-expansion object-terminal three-hop asserted head.**
228. [x] **Bridge the exact seven-expansion set-terminal four-hop asserted head.**
229. [x] **Bridge the exact seven-expansion object-terminal four-hop asserted head.**
230. [x] **Bridge the exact seven-expansion set-terminal five-hop asserted head.**
231. [x] **Bridge the exact seven-expansion object-terminal five-hop asserted head.**
233. [x] **Bridge the exact parenthesized builtin-object reserved-variable equality.**
234. [x] **Bridge the exact seven-expansion set-terminal full-distance six-hop asserted head.**
236. [x] **Bridge the exact seven-expansion object-terminal full-distance six-hop asserted head.**

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-checker
cargo clippy -p mizar-checker --all-targets -- -D warnings
```

For tasks that touch the resolver boundary or the corpus, also run:

```text
cargo test -p mizar-resolve
cargo test -p mizar-test
```

Check the task off here once tests pass.

## Notes

- The checker owns soft-type facts, replayable registration effects, and
  overload finalization only: no proof search, no ATP premise selection, no
  arbitrary first-order reasoning.
- `VcId`s are never assigned here; phases 6-8 emit `InitialObligationId`s
  that `mizar-vc` later converts exactly once.
- Wave breadth is paced by `mizar-resolve` signature increments and the
  parser definition grammar tasks; do not check declaration kinds the
  resolver cannot yet collect.
- Dependency-slice and fingerprint integration (architecture 18) arrives
  with `mizar-cache`; the checker only has to keep per-source contribution
  tracking accurate so slices stay computable.

## Task 241 Active Addendum

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 242 Active Addendum

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 243 Active Addendum

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 244 Active Addendum

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 245 Active Addendum

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 246 Active Addendum

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Tasks 266-268 Final Checker Handoff Queue

Completion evidence: [central Task-247 historical contract](../../task_contracts/en/247.md#completion-evidence).
Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Tasks 248-264 And 269-279 STEP 5 Source-Payload Producer Queue

The full authority slice, dependencies, consumers, gates, negative scope, and
exit criteria are canonical in
[payload_family_decomposition.md](./payload_family_decomposition.md). Each
unchecked row below is one future nonempty logical task and one commit.

- [x] **Task 248:** source item/declaration site and binding-context producer.
  The syntax-free producer, immutable `TypedAst`/`ResolvedTypedAst` handoff,
  exact reserve/definition-parameter shadowing consumer, recovery boundary,
  corruption matrix, and bounded trace row are complete. No term-use
  selection, type result, RHS/formula/proof semantics, or accepted fact is
  credited.
- [x] **Task 249:** type head/application/argument producer. The public
  syntax-free flat producer, input-only legacy binding-environment seam,
  immutable `TypedAst`/clone-only `ResolvedTypedAst` handoff, exact broad
  10/13/6 consumer, Task-248 2/2/0 co-consumer, corruption/determinism matrix,
  runner-only readiness detail, bounded trace row, paired docs, and count/hash
  verification are complete. It publishes no normalization, evidence,
  term/`qua` selection, accepted fact/declaration/proof, or downstream IR.
- [x] **Task 250:** attribute-chain/qualification/provenance producer. The
  public syntax-free five-table producer, immutable `TypedAst`/clone-only
  `ResolvedTypedAst` handoff, exact four real consumers and 4/4/0 plus
  4/4/1/1/1 aggregate oracles, written polarity/qualifier/group/actual
  preservation, synthetic prefix extractor, corruption/determinism matrix,
  bounded trace/expectation progression, and paired documentation are
  complete. It publishes no semantic attribute instance, arity/admissibility,
  term result, evidence request/result, accepted fact/declaration/proof, or
  downstream IR.
- [x] **Task 251:** evidence-query request and dependency-fact-reference
  producer. The paired crate-plan contract is implemented: exactly the
  Task-249 broad route plus Task-84/85 emit 10 missing requests
  (5 mode-expansion / 3 structure-inhabitation / 2 attributed) through dense
  syntax-free request/response-reference
  tables owned by `TypedAst` and clone-preserved by `ResolvedTypedAst`.
  Production-path tests distinguish requested/missing/rejected/supplied
  without treating supplied input as accepted evidence; checker tests cover
  exact association, corruption, cardinality, and atomicity. No evidence
  result is fabricated.
- [x] **Task 252:** primary term producer. The paired crate-plan contract is
  implemented through the
  public three-table syntax-free contract, exact three-route 7/4/2 real
  consumer oracle, transparent-parenthesis rule, numeric-request-only
  boundary, synthetic constant/`it` dependency coverage, final ownership,
  tests, and bounded covered trace row. Tasks 260/264/269 retain real `it`
  owners and local-constant binding production. The post-freeze contract
  correction derives each
  reference ordinal from preceding completed binding rows, preserves
  duplicate-priority binding groups for reachable `Ambiguous` rejection, and
  records `Resolver` as structurally unreachable for this producer.
- [x] **Task 253:** functor and inline-functor application producer. The
  public five-table `source_application` contract is implemented for the exact
  imported and same-definition-block later-definiens consumers, aggregate
  applications/wrappers/candidates/arguments/requests 2/1/2/3/4 oracle,
  Task-252 terms/references/numeric-requests 3/1/2 slice, Task-248
  `DefinitionParameter` shadow-handoff reuse, transparent cross-family origin
  ownership, individual candidate-reference boundary, synthetic schema,
  corruption matrix, and final ownership. Task 270 retains inline identity/
  formals/capture/substitution, Task 277 direct template transport, and Task
  278 ordinary/template candidate collection/viability/winner; Task 253 does
  not duplicate Task-252 primary terms.
- [x] **Task 254:** structure constructor/selector/update term producer.
  The public seven-table syntax-free handoff, Task-248 context reuse,
  Task-252/253/254 ownership and fingerprint matrix, exact
  5/0/3/9/2/10/26 plus 8/0/8 consumer, five arena-key classes, bounded
  fixture/trace row and reciprocal backlinks, corruption/determinism/final
  ownership coverage, and measured 413/377 and 243/231 oracles are complete.
  Structure/member/view semantics remain Task 263 ownership.
- [x] **Task 255:** set/comprehension/choice/`qua` term producer. The public
  six-table syntax-free `source_set_term` transaction, exact local-definition
  consumer, 4/0/1/3/4/7 plus Task-252 4/0/4 oracle, one-way
  Task-252/253/254/255 child ownership, conditional fingerprints, one-shot
  final handoff, bounded fixture/trace row, and frozen
  producer/extractor/corruption/install-order matrix are complete.
  Comprehension binder identity/capture remains Task 257,
  conditioned-comprehension formula ownership remains Tasks 256-257, and
  semantic result/sethood/nonemptiness/widening decisions remain deferred.
- [x] **Task 256:** atomic formula producer. The public eight-table
  syntax-free transaction, private exact eight-route consumer, exact
  `8/0/1/1/1/2/13/11` aggregate, Task-252 `16/0/16`, Task-253
  `1/1/1/2/2`, Task-255 `2/0/0/0/4/2`, conditional
  Task-253/254/255 fingerprints, eleven unresolved input requests, final
  immutable handoff, reciprocal trace increment, and reviewed real/synthetic/
  exclusion/corruption/install matrix are complete. Existing semantic routes
  and all outcome/detail fields remain unchanged.
- [ ] **Task 257:** composite/quantified formula, binder, predicate-chain, and
  conditioned-comprehension umbrella.
  - [x] **Task 257A:** exact implication/universal/negation/contradiction tree
    and one explicit unused binder. The public seven-table transaction,
    `2/1/4` binding extension, private exact consumer, final ownership,
    reciprocal trace row, and reviewed real/synthetic/corruption/isolation
    matrix are complete. The preflight-corrected real ranges are retained
    without changing the canonical `.miz` or its semantic detail vector.
  - [ ] **Task 257B:** broader connective/quantifier shapes, implicit binders,
    bound use, and capture.
    - [x] **Task 257B1:** explicit universal-to-atomic composition and two
      binder-selected bound uses. The exact 79-byte pass consumer, second
      exact Task-257 composite profile, Task-252/256 dependencies, `1/2`
      formula-composition transaction, final ownership, and bounded trace row
      are complete without semantic truth or theorem acceptance.
    - [x] **Task 257B2:** exact conjunction/disjunction/`iff`/repetition and
      executable formula grouping transport after Task 257B1, with no
      connective truth or theorem acceptance.
    - [x] **Task 257B3:** exact existential, restricted/nested quantification,
      implicit reserved-binder shadowing, and six scoped uses after Tasks
      257B1/B2. The frozen source-to-final-handoff transport is implemented
      without semantic truth, closure, capture-result, or theorem credit.
  - [ ] **Task 257C:** predicate-chain and conditioned-comprehension
    composition after separately frozen Task-256/255 extensions.
    - [x] **Task 257C1:** extend Task 256 with predicate-chain segment,
      polarity-token, and shared-boundary transport.
      - [x] Freeze the syntax-free nine-table contract, exact consumer, tests,
        trace projection, ownership, and semantic deferrals.
      - [x] Implement the frozen contract after fresh preflight in a separate
        logical task and commit.
    - [x] Extend Task 255 with condition-bearing comprehension transport in a
      separately frozen documentation/implementation pair.
    - [ ] Add predicate-chain and conditioned-comprehension formula
      composition only in later separately frozen Task-257C slices.
      - [x] **Task 257C2 prerequisite:** freeze the exact independent
        condition-to-atomic-formula association without semantics.
      - [x] **Task 257C2 implementation:** implement only the frozen
        condition-formula association after separate Task-256C1 and fresh
        preflight.
      - [x] **Task 257C3 prerequisite:** freeze predicate-chain
        conjunction/segment-negation composition separately after Task 257C2.
      - [x] **Task 257C3 implementation:** implement only the frozen
        predicate-chain composition after fresh post-documentation preflight.
- [ ] **Task 258:** general theorem-owner, statement-semantic, assumption, and
  visibility-scoped input-fact producer; never publish accepted theorem facts.
  - [x] **Task 258A:** exact reserved-variable equality theorem owner,
    statement shell, implicit reserved-type-guard input, and unverified
    proposition candidate.
    - [x] Freeze the exact 81-byte future `MT10-FS` source, resolver owner/
      label provenance, Task-252/256 lower profiles, syntax-free
      `1/1/1/1/1` transaction, typed/resolved ownership, empty-semantic
      boundary, owned BindingEnv/fingerprint, Task-248 exclusion through the
      production and named test-only seams, tests, trace non-activation, and
      exit criteria.
    - [x] Implement only the frozen Task-258A transport after the dedicated
      documentation commit and fresh parser/resolver/lower-API/count/hash
      preflight.
  - [ ] **Task 258B:** explicit assumptions, conclusions, witnesses, local
    label/citation inputs, composite theorem roots, nested statement contexts,
    and broader visibility. Tasks 269-272 retain proof-local bindings,
    closures, reconsider intent, proof skeletons, and justification
    semantics.
    - [x] **Task 258B1 prerequisite:** freeze the exact 139-byte nested
      equality-statement source, one theorem owner, four statement/context/
      guard/candidate rows, three proof binding contexts, one local
      proof-step label, one resolved citation, replayable resolver projection/
      reference/result, the two-pass 77-node/root-76 resolver AST with sole
      keyed node 68, Task-252/256 dependencies, typed/resolved ownership,
      test-only syntax dev-dependency, empty-semantic boundary, tests, and
      non-activation.
    - [x] **Task 258B1 implementation:** implemented only the frozen nested
      conclusion/local-label transport after the dedicated documentation
      commit and fresh parser/resolver/lower-API/count/hash preflight. Four
      checker and five runner tests close the bounded `source_drift` and
      `test_gap`; all semantic and corpus activation gates remain deferred.
    - [x] **Task 258B2 prerequisite:** freeze the exact 113-byte single
      unlabeled-assumption source, 55-node/root-54 parser shape, theorem/
      assumption/conclusion `1/3/3/3/3` profile, Task-48 `2/1/0`,
      Task-252 `6/6/0`, Task-256 `3/0/0/0/0/0/0/6/6`, base-only
      typed/final ownership, empty-semantic boundary, tests, and
      non-activation.
    - [x] **Task 258B2 implementation:** implemented only the frozen
      single-assumption transport after its dedicated documentation commit
      and fresh parser/resolver/lower-API/count/hash preflight. Four checker
      and five runner tests close the bounded `source_drift` and `test_gap`;
      no semantic or corpus activation was added.
    - [x] **Task 258B3 prerequisite:** freeze the exact 104-byte unnamed
      witness source, 49-node/root-48 parser identity, theorem-only resolver
      provenance, Task-48 `2/1/0`, Task-252 `5/5/0`, Task-256
      `2/0/0/0/0/0/0/4/4`, formula-only base `1/2/2/2/2`, one-row witness
      companion, paired typed/final ownership, tests, and non-activation.
    - [x] **Task 258B3 implementation:** implemented only the frozen paired
      witness transport after its documentation commit and fresh preflight.
      Four checker and five runner tests close bounded `source_drift` and
      `test_gap`; no semantics or corpus activation was added.
    - [ ] **Tasks 258B3N/M:** after B3, separately freeze named-witness
      transport and multiple/other witness-term transport before B4. Do not
      infer abbreviation, substitution, type-obligation, or goal semantics.
    - [ ] **Tasks 258B4-B5:** separately freeze composite theorem roots and
      broader imported/outer/inner visibility profiles. Do not absorb Tasks
      269-272 semantics.
- [x] **Task 259:** predicate-definition and initial-obligation intake producer.
- [x] **Task 260:** functor-definition and initial-obligation intake producer.
- [x] **Task 261:** attribute-definition producer.
- [x] **Task 262:** mode-definition producer.
- [x] **Task 263:** structure/inheritance/constructor-definition producer.
- [x] **Task 264:** property-implementation producer; depends on parser Task 48.
- [ ] **Task 269:** proof-local declaration/binding producer.
- [ ] **Task 270:** inline-definition closure/capture/substitution-request producer.
- [ ] **Task 271:** `reconsider` intent/coercion/evidence-request producer;
  depends on parser Task 47.
- [ ] **Task 272:** non-Task-180 proof-skeleton/justification producer.
- [ ] **Task 273:** registration-item/correctness/initial-obligation intake producer.
- [ ] **Task 274 (blocked-reserved):** accepted verifier/artifact-status import
  and activation adapter. Not executable until canonical authority names the
  upstream owner, schema, authentication rules, and tests.
- [ ] **Task 275:** source-derived cluster-closure trace producer.
- [ ] **Task 276:** source-derived reduction/normalization trace producer.
- [ ] **Task 277:** direct template role/actual/guard producer. Missing
  scheme/theorem roles remain outside this executable task under external Gate
  S1, and Task 49 remains gated on S1.
- [ ] **Task 278:** ordinary/template overload input-to-selection producer.
- [ ] **Task 279:** redefinition/notation target/coherence/refinement producer;
  consumes Task 278 ordinary-root results without a dependency cycle.

Every task projects its family transactionally through applicable `TypedAst`
and `ResolvedTypedAst` tables and is consumed by a real `mizar-test` Task-10
case. An unconsumed DTO, placeholder runner, or documentation-only
implementation commit does not satisfy a producer task.

## Task 257B2 Frozen-Contract Addendum

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 257B3 Frozen-Contract Addendum

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 257C1 Frozen-Contract Addendum

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 255C1 Frozen-Contract Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 257C2 Frozen-Contract Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 256C1 Frozen-Contract Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 256C1 Implementation Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 257C2 Implementation Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 257C3 Frozen-Contract Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 257C3 Implementation Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3 Frozen-Contract Ledger

- [x] Freeze the final-LF 104-byte source/hash, 49-node/root-48 parser
  ranges, public/exported theorem provenance `[2,1]`, and no resolver
  companion.
- [x] Freeze Task-48 `2/1/0`, Task-252 `5/5/0`, Task-256
  `2/0/0/0/0/0/0/4/4`, formula-only base `1/2/2/2/2`, and exact term-2
  exclusion from atomic edges.
- [x] Freeze the public witness ID/input/row/table/handoff/producer/error,
  both fingerprints, deterministic debug, direct binding context, and one
  unnamed primary target.
- [x] Freeze dense base IDs 0/1 with global ordinals 0/2, witness
  source/within-take ordinals 1/0, and paired unique `[0,1,2]` validation.
- [x] Freeze typed/final pair-only ownership, all profile/source-family
  exclusions, rollback/replay, checker tests 4, runner tests 5, and empty
  semantics.
- [x] Keep every fixture, sidecar, expectation, trace row/status/count,
  active route, source, test list, and hash unchanged at libraries
  `346/379` and runner 30 paths / 36,479 lines.
- [x] Classify the closed contract as `design_drift`, future code as bounded
  `source_drift`, tests as `test_gap`, and find no blocking protocol
  disagreement.
- [x] Implement only Task 258B3 after this dedicated documentation commit
  and fresh parser/resolver/lower-API/count/hash preflight. Libraries are
  `350/384`; changed hashes and lines are remeasured in the implementation
  result.
- [x] Fresh-inventory and freeze Task 258B3N named-primary witness transport
  only: exact 107-byte/51-node source, `1 witness / 1 name` table, B3
  compatibility, no binding/semantics, four/five tests, and unchanged
  baselines.
- [x] Implement only Task 258B3N after its dedicated documentation commit
  and fresh parser/resolver/lower/count/hash preflight. The exact dense
  witness-name transport, four checker tests, and five runner tests pass;
  libraries are `354/389`, with no semantic or corpus activation.
- [x] Decompose broad Task 258B3M into exact reserved-variable B3M1 and
  non-reserved-variable/other-term B3M2 before selecting Task 258B4.
- [x] Freeze Task 258B3M1 only: exact 113-byte/56-node mixed two-witness
  source, Task-252 `6/6/0`, base/witness/name `1/2/2/2/2` + `2/1`,
  shared source ordinal 1, dense ordinals 0/1, no new API or semantics,
  four/five tests, and unchanged baselines.
- [x] Implement only frozen Task 258B3M1 after its documentation commit and
  fresh parser/resolver/lower/count/hash preflight.
- [x] Decompose Task 258B3M2 into exact unnamed-numeral B3M2A and remaining
  other-term B3M2B before selecting Task 258B4.
- [x] Freeze Task 258B3M2A only: final-LF 107-byte/49-node source,
  Task-252 `5/4/1`, base/witness/name `1/2/2/2/2` + `1/0`, numeric request
  ownership, no new API or semantics, four/five tests, and unchanged
  baselines.
- [x] Implement only frozen Task 258B3M2A after its documentation commit
  and fresh parser/resolver/lower/count/hash preflight.
- [x] Decompose Task 258B3M2B into exact single-level parenthesized
  reserved-variable B3M2B1 and remaining other-term B3M2B2.
- [x] Freeze Task 258B3M2B1 only; keep implementation separate.
- [x] Implement frozen Task 258B3M2B1 after its documentation commit and
  fresh preflight.
- [x] Decompose Task 258B3M2B2 into exact nested-parenthesized B3M2B2A
  and remaining authority-valid B3M2B2B.
- [x] Freeze only Task 258B3M2B2A; keep implementation separate.
- [x] Implement frozen Task 258B3M2B2A after its documentation commit and
  fresh parser/resolver/lower/count/hash preflight.
- [ ] Freeze and implement Task 258B3M2B2B before selecting Task 258B4.

Completion evidence: [central Task-258B3N historical contract](../../task_contracts/en/258B3N.md#completion-evidence).

## Checker Task 258B3M1 Documentation Ledger

Completion evidence: [central Task-258B3M1 historical contract](../../task_contracts/en/258B3M1.md#completion-evidence).
Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2A Documentation Ledger

Completion evidence: [central Task-258B3M2A historical contract](../../task_contracts/en/258B3M2A.md#completion-evidence).
Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B1 Frozen-Contract Ledger

Completion evidence: [central Task-258B3M2B1 historical contract](../../task_contracts/en/258B3M2B1.md#completion-evidence).
Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2A Frozen-Contract Ledger

Completion evidence: [central Task-258B3M2B2A historical contract](../../task_contracts/en/258B3M2B2A.md#completion-evidence).
Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B1P Frozen Lower-Prerequisite Ledger

Completion evidence: [central Task-258B3M2B2B1P historical contract](../../task_contracts/en/258B3M2B2B1P.md#completion-evidence).
Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B1A Frozen-Contract Ledger

Completion evidence: [central Task-258B3M2B2B1A historical contract](../../task_contracts/en/258B3M2B2B1A.md#completion-evidence).
Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B1B1P Frozen-Prerequisite Ledger

Completion evidence: [central Task-258B3M2B2B1B1P historical contract](../../task_contracts/en/258B3M2B2B1B1P.md#completion-evidence).
Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B1B1 Frozen-Contract Ledger

Completion evidence: [central Task-258B3M2B2B1B1 historical contract](../../task_contracts/en/258B3M2B2B1B1.md#completion-evidence).
Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B2P Frozen-Prerequisite Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B2P Implementation Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B2A Frozen-Contract Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B2A Implementation Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B2BP Frozen-Contract Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B2BP Implementation Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B2B Frozen-Contract Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B2B Implementation Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B2CP Frozen-Prerequisite Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B2C Frozen-Contract Ledger

Completion evidence: [central Task-258B3M2B2B2C historical contract](../../task_contracts/en/258B3M2B2B2C.md#completion-evidence).
Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B3P Frozen-Contract Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B3P Implementation-Closure Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B3A Frozen-Contract Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B3A Implementation Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B3B Frozen-Contract Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B3B Implementation Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B3C Frozen-Contract Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B3C Implementation Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B3D Frozen-Contract Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B3D Implementation Ledger

- [x] Close prerequisite
  `43af562c2cb84e72658cee059abbe7543ee73fe7` at clean
  ahead-2/behind-0 with stash fingerprint `f65cf4a13752ec...` unchanged.
- [x] Confirm no lower-stage prerequisite and implement only the frozen
  checker three plus runner four source consumers.
- [x] Preserve both `source_set_term.rs` owners, authority/corpus artifacts,
  public APIs/errors/debug, dependencies, active behavior, and semantics.
- [x] Implement the exact checker four plus runner five tests and
  `32/70/44/72/62/21` field matrices.
- [x] Complete independent test-sufficiency review with **NO FINDINGS**.
- [x] Pass focused checker `4/4`, runner `5/5`, checker package `406+15`,
  runner package `466+3/14/137/2/21`, formatting, and full Clippy.
- [x] Record exact checker/runner module sizes, production/test-list hashes,
  unchanged five CLI hashes/counts, and deliberate authority/trace no-op.
- [x] Complete repeated independent implementation review with
  **NO FINDINGS**.
- [x] Complete repeated source/documentation consistency, bilingual, and
  boundary review with **NO FINDINGS** after fixing one Medium stale-review
  `design_drift` and two Low 24-order/qua-edge `design_drift` findings.
- [x] Pass checker package `406+15`, runner package
  `466+3/14/137/2/21`, `cargo fmt --check`, all-target/all-feature
  Clippy with warnings denied, full workspace tests, five CLIs, and final
  count/hash reruns.
- [x] Complete independent final read-only quality review with
  **NO FINDINGS**, all nine hard gates PASS, no score cap, and valid
  `100/100` (`20/20/15/15/10/10/5/5`).
- [ ] Stage only the exact synchronized implementation scope, inspect cached
  diff, and create one implementation commit.
- [ ] Verify clean post-commit/stash invariants and fresh-inventory the next
  dependency-minimal task.

## Checker Task 258B3M2B2B3E Frozen-Contract Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B3M2B2B3E Implementation Ledger

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B4A Documentation Prerequisite

Completion evidence: [central Task-258B4A historical contract](../../task_contracts/en/258B4A.md#completion-evidence).
Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B4B Documentation Prerequisite

Completion evidence: [central Task-258B4B historical contract](../../task_contracts/en/258B4B.md#completion-evidence).
Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B4C Documentation Prerequisite

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B4C Lower-Stage Prerequisite Ledger

Completion evidence: [central Task-258B4C historical contract](../../task_contracts/en/258B4C.md#completion-evidence).
Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B5A Frozen-Contract Documentation Prerequisite

Completion evidence: [central Task-258B5A historical contract](../../task_contracts/en/258B5A.md#completion-evidence).
Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B5B Frozen-Contract Documentation Prerequisite

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B5B Lower-Stage Prerequisite

Completion evidence: [central Task-258B5B historical contract](../../task_contracts/en/258B5B.md#completion-evidence).
Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B5C Frozen-Contract Documentation Prerequisite

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## B5C R-032A Preflight Overlay

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## B5C R-032B Lint-Policy Preflight Overlay

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 258B5C Active Implementation

- [x] Add exactly two spec-derived fail cases and two covered trace rows
  through the private declaration-symbol consumer; keep public codes empty.
- [x] Correct the omitted metadata count consumer in four `5 -> 7`
  assertions, classified as `test_expectation_drift` and scope
  `design_drift`.
- [x] Preserve every checker source/API/semantic result as a no-op and close
  only the two confinement requirements within R-G007.
- [x] Complete findings-free test, implementation, and source/documentation
  reviews plus focused/crate/workspace/count/hash verification gates.
- [x] Complete final quality with **NO FINDINGS**, all nine hard gates PASS,
  no score cap, and valid `100/100` (`20/20/15/15/10/10/5/5`).
- [ ] Complete the task-only commit, post-commit invariants, and next-task
  fresh inventory.

## Checker Task 259 Frozen-Contract Documentation Prerequisite

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 248 Two-Parameter Profile-Extension Documentation Prerequisite

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 259 Frozen-Contract Correction Prerequisite

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 259 Active Implementation

Completion evidence: [central Task-260 historical contract](../../task_contracts/en/260.md#completion-evidence).
Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 249R Definition-Return Documentation Prerequisite

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 249R Active Implementation

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 260 Active Implementation

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 261 Frozen-Contract Documentation Prerequisite

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 261 Implementation

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 262 Frozen-Contract Documentation Prerequisite

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 263 Preflight Lower Prerequisite

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 249S Standalone Structure-Member Type Prerequisite

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 263 Structure-Definition Intake

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 264 Lower-Prerequisite Sequence

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 249PI Property-Type Composition Prerequisite

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 269A Named-Witness Binding Slice

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 269B Mixed-Witness Binding Increment

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 269CP Isolated Proof-`let` Lower Prerequisite

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 269C Binding-Only Proof-`let` Transaction

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 269CT Proof-`let` Source-Type Prerequisite

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 269GP Proof-`given` Lower Prerequisite

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 269GS Canonical `given` Scope Reconciliation

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 269G Proof-`given` Binding Consumer

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 269GT Proof-`given` Source-Type Consumer

Completion evidence: [central Task-269GT historical contract](../../task_contracts/en/269GT.md#completion-evidence).
Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 269GUP Proof-`given` Use-profile Binding Prerequisite

Completion evidence: [central Task-269GUP historical contract](../../task_contracts/en/269GUP.md#completion-evidence).
Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 269GUPT Source-Type Prerequisite

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 269GU Later-use Term/Reference Prerequisite

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 269GCP Given-condition Lower Prerequisite

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 269GC Given-condition Binding Consumer

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 269GCT Given-condition Source-Type Consumer

Completion evidence: [central Task-269GCT historical contract](../../task_contracts/en/269GCT.md#completion-evidence).
Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 269GCU Given-condition Term/reference Consumer

Completion evidence: [central Task-269GCU historical contract](../../task_contracts/en/269GCU.md#completion-evidence).
Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Checker Task 269SDP Descendant/Set Lower Prerequisite

- [x] Confirm clean committed GCU, `0/19` report-only
  `repo_metadata_conflict`, and unchanged protected stash.
- [x] Classify stale GCU completion text as repaired `design_drift`; classify
  the missing exact descendant/set lower contract, selector, and tests as
  `design_drift`, `source_drift`, and `test_gap`; record the nonblocking-for-SDP
  but capture-blocking Chapter-4/15 `set` `spec_gap`.
- [x] Freeze the exact 180-byte source, 68-node/root-67 Surface identity,
  theorem shell/resolver provenance, Given/now/two-Set ranges, fingerprints,
  private lower/debug ABI, four runner files/tests, baselines, exclusions,
  semantic deferrals, and exit criteria.
- [x] Repeat specification and bilingual review to **NO FINDINGS**, pass all
  docs-only gates at uncapped `100/100`, stage exactly 42 Markdown files,
  and commit documentation prerequisite `f468b0163bb00726dca9b356f48790c73bb1fe98`.
- [x] Fresh-preflight and implement only the lower profile; do not publish
  binding contexts, occurrences, captures, facts, proof results, or Task 270.
- [x] Complete separate test and implementation reviews at **NO FINDINGS**.
- [x] Complete source/docs re-review and final-quality review at uncapped
  `100/100`, and pass full verification.
- [ ] Complete exact staging, implementation commit, and fresh selection of
  the descendant context/binding consumer.

Completion evidence: [central Task-269SDP historical contract](../../task_contracts/en/269SDP.md#completion-evidence).

## Checker Task 269SDC Descendant Context/Binding Prerequisite

Completion evidence: [central Task-269SDC historical contract](../../task_contracts/en/269SDC.md#completion-evidence).
Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 269SDT

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 269SDU Descendant Given Occurrence/Reference

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 277A Direct Parser-Origin Template Transport

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 277B-L Template Type-Parameter Association

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 277C Fraenkel Structural Composition

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 257C4A Fraenkel generator binding context

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 257C4B Fraenkel generator bound-use transport

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 257C4C0 nested Fraenkel capture test intent

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 257C4C1 explicit-import lexical admission

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 257C4C3 nested binder/mapper-use transport

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 257C4C4 nested mapper primary transport

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 257C4C5 nested capture-identity receipt

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 257C4C6 nested capture-identity installation

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 257C4C7 two-capture prerequisite

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 257C4C8 normalized nested capture graph

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 33C opaque capture-graph owner receipt

- [x] Freeze the user-selected checker-owned scalar receipt, by-value C4C8 and
  Task33R dependencies, exact public API, and graph-dependency then owner-
  dependency then association precedence in the paired
  [Task33C contract](../../task_contracts/en/CHECKER-FRAENKEL-CAPTURE-GRAPH-OWNER-33C.md).
- [x] Complete independent specification/equivalence and bilingual/boundary
  review with **NO FINDINGS** before Rust edits.
- [x] Implement only the frozen handoff/error/producer, four checker tests,
  public-enum policy guard, and one private real-fixture probe.
- [x] Preserve C4C4/C4C5/C4C6, Typed/Resolved/Core ownership, parameter/order/
  GeneratedOrigin deferrals, active routes, diagnostics, protected artifacts,
  semantic coverage, and Task277B zero credit.
- [x] Complete all post-source reviews, verification, and final-quality hard
  gates with **NO FINDINGS**, `9/9` PASS, and valid uncapped `100/100`.
- [ ] Complete exact task-only commit, clean postcommit proof, and fresh
  inventory.

## Task 264C property carrier identity transport

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

## Task 264D equals selector identity association

Details archived: [checker_todo_sections.md](../../archive/checker_todo_sections.md).

