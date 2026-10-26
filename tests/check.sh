#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 || ! ${1-} =~ ^(docs|parser|verifier|resolver|evidence|full)$ ]]; then
    printf 'Usage: bash tests/check.sh {docs|parser|verifier|resolver|evidence|full}\n' >&2
    exit 2
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

case $1 in
    docs)
        cargo test -p mizar-test --test lint_policy
        ;;
    parser)
        cargo test -p mizar-syntax -p mizar-parser -p mizar-frontend
        cargo run -p mizar-test -- parse-only
        cargo run -p mizar-test -- syntax-smoke
        run_exact scheduled_theorem_elaboration_matches_genuine_publication -p mizar-driver --test source_load
        ;;
    verifier)
        cargo test -p mizar-checker -p mizar-kernel -p mizar-proof -p mizar-vc
        cargo run -p mizar-test -- proof-verification
        run_exact native_theorem_metadata_caller_preserves_genuine_outputs -p mizar-driver --test source_load
        run_exact native_theorem_metadata_caller_preserves_refusals -p mizar-driver --test source_load
        ;;
    resolver)
        run_exact scheduled_theorem_resolver_edits_match_independent_clean_modules -p mizar-driver --test source_load
        ;;
    evidence)
        run_exact runner::tests::source_registration_candidate_metadata_aggregate_preserves_rejection_snapshot_and_policy_identity -p mizar-test --lib
        run_exact runner::tests::source_registration_candidate_witness_binds_all_seventeen_participants_and_full_name_bytes -p mizar-test --lib
        ;;
    full)
        cargo test
        ;;
esac
