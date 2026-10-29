# Task TEST-PROPERTY-NATIVE-LAYOUT: native lexical layout evidence
Canonical language: English; [Japanese pointer](../ja/TEST-PROPERTY-NATIVE-LAYOUT.md).
Status: complete. Tier: full. Owner: [test](../../mizar-test/en/00.crate_plan.md).
Purpose: close the native-equivalence test_gap before Property source generalization.
Authority: lexical §§2.1/2.2/2.3/2.5.3/2.9.1/2.10, unchanged Task264 MIZ profiles
and their existing exact-source/Surface tests. Dependency: current native Frontend APIs.

Append one test to the existing dedicated Property runner test file. Preserve every
old test/helper, production byte, source fixture/hash, expectation, trace and six exceptions.
Use real FrontendOutput from the existing disk-backed helper and its retained source/
package/module/edition/preprocessing/token metadata; no fabricated SourceUnit/provider.
Both Means/Equals cover original, actual generated leading-line/intertoken edits,
missing final LF, appended trailing space and compact punctuation with an ordinary
internal Unicode block comment. Keep the same closed token/AST family in all variants.

Independently preprocess/scan, collect local declarations for the actual module,
build scope skeleton and disambiguate with locals; compare every native token kind,
text, mapped current-source range/raw slice and the actual local declarations/maps.
Require one source-map fragment per token, zero diagnostics/import stubs/recovery,
canonical AST storage, unchanged kind/text/ordered-child topology and correct roots.
Keep declaration Identifier2 versus active UserSymbol22 and Means Numeral39/48 explicit.
Preserve intentional root token aliases and full Surface parameter envelopes65/47;
this test constructs no typed arena and claims no source-bridge or semantic admission.

No new helper/API/type/field/parser, old oracle correction or coverage-audit credit.
Derived docs stay within the test-only cap. Exit requires independent SPEC/intent/trust,
TEST/CODE/VOLUME/consistency, scoped gates, actual native cases and affected Task264/
guard/docs/link checks, fmt/Clippy and inherited full cadence.
