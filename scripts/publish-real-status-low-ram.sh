#!/usr/bin/env bash
set -euo pipefail

BACKEND="${1:-wasm-aot}"
SNAPSHOT_NAME="${2:-codex-published-real}"
LILA_BIN="${LILA_BIN:-./target/release/lila}"
SUITE_ROOT="${SUITE_ROOT:-test262/vendor/test262}"
SNAPSHOT_DIR="${SNAPSHOT_DIR:-test262/snapshots}"
THREADS="${THREADS:-1}"
JOBS="${JOBS:-1}"
ISOLATE_CASES="${ISOLATE_CASES:-1}"
MAX_MATRIX_NODES="${MAX_MATRIX_NODES:-1}"
README_PATH="${README_PATH:-}"
MATRIX_TOTAL=""
MATRIX_COMPLETED=0
REPORT_RAN=0

fail() {
  echo "publish-real-status-low-ram: $*" >&2
  exit 1
}

if [[ "$BACKEND" != "wasm" && "$BACKEND" != "wasm-aot" ]]; then
  echo "publish-real-status-low-ram only publishes the wasm-aot product backend" >&2
  echo "run oracle matrices with: lila test262 report-all --execution-backend spec-exec" >&2
  exit 2
fi
BACKEND=wasm-aot

# Bound decimal inputs before Bash arithmetic (no octal interpretation or overflow).
# Eighteen digits fit in Bash's signed 64-bit arithmetic without wrapping.
positive_integer() {
  [[ "$1" =~ ^[1-9][0-9]{0,17}$ ]]
}
for setting in THREADS JOBS MAX_MATRIX_NODES; do
  positive_integer "${!setting}" || fail "$setting must be a positive decimal integer (at most 18 digits)"
done
[[ "$ISOLATE_CASES" == 0 || "$ISOLATE_CASES" == 1 ]] || fail "ISOLATE_CASES must be 0 or 1"

unset LILA_TEST262_DISABLE_CASE_RUNNER
if [[ "$ISOLATE_CASES" == 1 ]]; then
  export LILA_TEST262_FORCE_CASE_RUNNER=1
else
  unset LILA_TEST262_FORCE_CASE_RUNNER
fi

if [[ ! -x "$LILA_BIN" ]]; then
  echo "missing executable: $LILA_BIN" >&2
  echo "build first: cargo build --release -p lila-cli" >&2
  exit 1
fi

REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
export REPO_ROOT LILA_BIN SUITE_ROOT SNAPSHOT_DIR SNAPSHOT_NAME THREADS JOBS ISOLATE_CASES MAX_MATRIX_NODES README_PATH
exec python3 "$REPO_ROOT/scripts/publication-session.py" run
