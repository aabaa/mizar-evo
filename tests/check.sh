#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
    printf 'Usage: bash tests/check.sh <scope> [quick|standard|heavy]\n' >&2
    exit 2
fi
scope=$1
cost=${2-standard}
# Keep the original long-matrix commands unchanged for existing callers.
if [[ $# -eq 1 && $scope =~ ^(resolver|evidence)$ ]]; then
    cost=heavy
fi
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."

run_exact() {
    local name=$1
    shift
    if ! cargo test "$@" "$name" -- --exact --list | grep -Fx -- "$name: test" >/dev/null; then
        printf 'Expected test missing or listing failed: %s\n' "$name" >&2
        return 1
    fi
    cargo test "$@" "$name" -- --exact
}

case $scope:$cost in
    docs:standard)
        cargo test -p mizar-test --test lint_policy
        ;;
    parser:quick)
        cargo test -p mizar-syntax -p mizar-parser -p mizar-frontend --lib
        ;;
    parser:standard)
        cargo test -p mizar-syntax -p mizar-parser -p mizar-frontend
        cargo run -p mizar-test -- parse-only
        cargo run -p mizar-test -- syntax-smoke
        run_exact scheduled_theorem_elaboration_matches_genuine_publication -p mizar-driver --test source_load
        ;;
    verifier:quick)
        cargo test -p mizar-checker -p mizar-kernel -p mizar-proof -p mizar-vc --lib
        ;;
    verifier:standard)
        cargo test -p mizar-checker -p mizar-kernel -p mizar-proof -p mizar-vc
        cargo run -p mizar-test -- proof-verification
        run_exact native_theorem_metadata_caller_preserves_genuine_outputs -p mizar-driver --test source_load
        run_exact native_theorem_metadata_caller_preserves_refusals -p mizar-driver --test source_load
        ;;
    resolver:quick)
        cargo test -p mizar-resolve --lib
        ;;
    resolver:standard)
        cargo test -p mizar-resolve
        cargo run -p mizar-test -- declaration-symbol
        run_exact native_frontend_resolver_binding_preserves_scheduled_captured_pairs -p mizar-driver --test source_load
        run_exact native_frontend_resolver_binding_rejects_bad_associations_and_preserves_lower_results -p mizar-driver --test source_load
        ;;
    resolver:heavy)
        run_exact scheduled_theorem_resolver_edits_match_independent_clean_modules -p mizar-driver --test source_load
        ;;
    evidence:standard)
        run_exact runner::tests::source_registration_candidate_metadata_single_matches_independent_wire_and_kernel -p mizar-test --lib
        run_exact runner::tests::source_registration_candidate_metadata_rejects_mixed_unsupported_and_denied_inputs -p mizar-test --lib
        ;;
    evidence:heavy)
        run_exact runner::tests::source_registration_candidate_metadata_aggregate_preserves_rejection_snapshot_and_policy_identity -p mizar-test --lib
        run_exact runner::tests::source_registration_candidate_witness_binds_all_seventeen_participants_and_full_name_bytes -p mizar-test --lib
        ;;
    declaration:standard)
        cargo run -p mizar-test -- declaration-symbol
        ;;
    type:standard)
        cargo run -p mizar-test -- type-elaboration
        ;;
    formula:standard)
        cargo run -p mizar-test -- formula-statement
        ;;
    cache:standard)
        cargo test -p mizar-cache
        run_exact native_disk_cache_lookup_rebinds_resident_and_blob_for_clean_frontend -p mizar-driver --test source_load
        run_exact native_disk_cache_lookup_misses_changed_source_maps_context_and_policy -p mizar-driver --test source_load
        run_exact native_disk_cache_lookup_rejects_binding_currentness_and_foreign_producer -p mizar-driver --test source_load
        ;;
    full:standard)
        cargo test
        ;;
    *)
        printf 'Unsupported scope/cost: %s/%s\n' "$scope" "$cost" >&2
        exit 2
        ;;
esac
