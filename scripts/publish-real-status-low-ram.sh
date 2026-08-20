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

if [[ "$BACKEND" != "wasm" && "$BACKEND" != "wasm-aot" ]]; then
  echo "publish-real-status-low-ram only publishes the wasm-aot product backend" >&2
  echo "run oracle matrices with: lila test262 report-all --execution-backend spec-exec" >&2
  exit 2
fi

if [[ "$ISOLATE_CASES" == "1" ]]; then
  export LILA_TEST262_FORCE_CASE_RUNNER=1
fi

if [[ ! -x "$LILA_BIN" ]]; then
  echo "missing executable: $LILA_BIN" >&2
  echo "build first: cargo build --release -p lila-cli" >&2
  exit 1
fi

matrix_progress() {
  local progress
  if ! progress="$(
    "$LILA_BIN" test262 progress-status \
      --execution-backend "$BACKEND" \
      --suite-root "$SUITE_ROOT" \
      --snapshot-dir "$SNAPSHOT_DIR" \
      --snapshot-name "$SNAPSHOT_NAME" 2>/dev/null
  )"; then
    MATRIX_COMPLETED=0
    MATRIX_TOTAL=""
    return 0
  fi

  MATRIX_COMPLETED="$(awk -F ': ' '$1 == "matrix_nodes_completed" { print $2; exit }' <<<"$progress")"
  MATRIX_TOTAL="$(awk -F ': ' '$1 == "matrix_nodes_total" { print $2; exit }' <<<"$progress")"
  if [[ ! "$MATRIX_COMPLETED" =~ ^[0-9]+$ || ! "$MATRIX_TOTAL" =~ ^[0-9]+$ ]]; then
    echo "invalid matrix progress returned by Lila" >&2
    exit 1
  fi
}

while true; do
  matrix_progress
  completed="$MATRIX_COMPLETED"
  total="$MATRIX_TOTAL"
  echo "matrix_progress: ${completed}/${total:-unknown}"

  if [[ -n "$total" && "$total" -gt 0 && "$completed" -ge "$total" ]]; then
    cmd=(
      "$LILA_BIN" test262 publish-status
      --execution-backend "$BACKEND"
      --suite-root "$SUITE_ROOT"
      --snapshot-dir "$SNAPSHOT_DIR"
      --snapshot-name "$SNAPSHOT_NAME"
    )
    if [[ -n "$README_PATH" ]]; then
      cmd+=(--readme-path "$README_PATH")
    fi
    exec "${cmd[@]}"
  fi

  "$LILA_BIN" test262 report-all \
    --jobs "$JOBS" \
    --execution-backend "$BACKEND" \
    --suite-root "$SUITE_ROOT" \
    --snapshot-dir "$SNAPSHOT_DIR" \
    --snapshot-name "$SNAPSHOT_NAME" \
    --resume \
    --threads "$THREADS" \
    --max-matrix-nodes "$MAX_MATRIX_NODES"
done
