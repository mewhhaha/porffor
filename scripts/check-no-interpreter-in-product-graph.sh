#!/usr/bin/env bash
# T27: product/release builds of the library and CLI must link no JavaScript
# interpreter/VM engine (AGENTS.md hard ban). lila-spec-exec wraps the Boa
# interpreter and exists only as a hidden, developer-only differential
# oracle gated behind the `spec-exec-oracle` cargo feature. This script
# proves that feature is off by default and that boa_engine does not appear
# in the default dependency graph of lila-engine or lila-cli.
set -euo pipefail

failures=0

fail() {
  printf 'check-no-interpreter-in-product-graph: %s\n' "$*" >&2
  failures=$((failures + 1))
}

# Capture Cargo's exit status separately from inspecting its output. A failed
# graph query is not evidence that the product excludes the interpreter.
check_graph() {
  local package="$1" expectation="$2" graph count
  shift 2
  if ! graph="$(cargo tree --locked -p "$package" --prefix none --format '{p}' "$@")"; then
    fail "could not inspect dependency graph of $package ($expectation)"
    return
  fi
  if [[ -z "$graph" ]]; then
    fail "empty dependency graph for $package ($expectation)"
    return
  fi
  # Match package names, not paths or similarly named crates. --prefix none
  # makes the first field independent of Cargo's tree-drawing characters.
  count="$(awk '$1 == "boa_engine" { count++ } END { print count + 0 }' <<<"$graph")"
  case "$expectation" in
    absent)
      if [[ "$count" -ne 0 ]]; then
        fail "expected 0 boa_engine crates in the default dependency graph of $package, found $count"
      fi
      ;;
    present)
      if [[ "$count" -eq 0 ]]; then
        fail "expected boa_engine to be reachable from $package with --features spec-exec-oracle (developer oracle build)"
      fi
      ;;
  esac
}

check_graph lila-engine absent
check_graph lila-cli absent
check_graph lila-engine present --features spec-exec-oracle
check_graph lila-cli present --features spec-exec-oracle

artifact_test="crates/lila-aot-wasm/tests/product_artifact.rs"
if [ ! -f "$artifact_test" ]; then
  fail "missing emitted product artifact audit: $artifact_test"
elif ! grep -q 'product_wasm_contains_compiled_semantics_without_a_source_evaluator' "$artifact_test"; then
  fail "product artifact audit no longer proves compiled semantics and source-evaluator absence"
fi

if ! grep -q 'cargo test -p lila-aot-wasm --test product_artifact' .github/workflows/ci.yaml; then
  fail "CI does not execute the emitted product artifact audit"
fi

if [ "$failures" -ne 0 ]; then
  exit 1
fi

printf 'check-no-interpreter-in-product-graph: ok\n'
