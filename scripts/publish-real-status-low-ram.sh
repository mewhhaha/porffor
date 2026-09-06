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

if [[ "$ISOLATE_CASES" == 1 ]]; then
  export LILA_TEST262_FORCE_CASE_RUNNER=1
fi

if [[ ! -x "$LILA_BIN" ]]; then
  echo "missing executable: $LILA_BIN" >&2
  echo "build first: cargo build --release -p lila-cli" >&2
  exit 1
fi

REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE_COMMIT="$(git -C "$REPO_ROOT" rev-parse --verify HEAD)"
compiler_digest() {
  local digest
  digest="$(sha256sum < "$LILA_BIN")" || fail "cannot hash executable: $LILA_BIN"
  printf '%s\n' "${digest%% *}"
}
COMPILER_SHA256="$(compiler_digest)"

# This is invocation evidence, not an attestation that this checkout built the
# executable or that older checkpoints were produced by the same compiler.
printf 'source_commit: %s\ncompiler_sha256: %s\n' "$SOURCE_COMMIT" "$COMPILER_SHA256"
printf 'execution_backend: %s\nsnapshot_name: %s\n' "$BACKEND" "$SNAPSHOT_NAME"
printf 'suite_root: %s\nsnapshot_dir: %s\n' "$SUITE_ROOT" "$SNAPSHOT_DIR"

verify_compiler() {
  [[ -x "$LILA_BIN" ]] || fail "executable disappeared or is no longer executable: $LILA_BIN"
  [[ "$(compiler_digest)" == "$COMPILER_SHA256" ]] || fail "compiler changed during publication; retain the log and checkpoints for triage"
  [[ "$(git -C "$REPO_ROOT" rev-parse --verify HEAD)" == "$SOURCE_COMMIT" ]] || fail "source commit changed during publication; retain the log and checkpoints for triage"
}

matrix_progress() {
  local progress fields status
  verify_compiler
  if progress="$(
    "$LILA_BIN" test262 progress-status \
      --execution-backend "$BACKEND" \
      --suite-root "$SUITE_ROOT" \
      --snapshot-dir "$SNAPSHOT_DIR" \
      --snapshot-name "$SNAPSHOT_NAME"
  )"; then
    :
  else
    status=$?
    # A fresh run may not have a checkpoint yet. Bootstrap only once; after a
    # successful report-all, losing progress is an error, never an empty matrix.
    [[ "$REPORT_RAN" == 0 ]] || fail "progress-status failed after report-all (exit $status)"
    echo "progress-status unavailable before first report-all (exit $status); attempting initial checkpoint" >&2
    MATRIX_COMPLETED=0
    MATRIX_TOTAL=""
    return
  fi

  if ! fields="$(awk -F ': ' '
    $1 == "matrix_nodes_completed" { completed = $2; completed_fields++; if (NF != 2) invalid = 1 }
    $1 == "matrix_nodes_total" { total = $2; total_fields++; if (NF != 2) invalid = 1 }
    END {
      if (invalid || completed_fields != 1 || total_fields != 1) exit 1
      printf "%s\n%s\n", completed, total
    }
  ' <<<"$progress")"; then
    fail "invalid matrix progress: expected exactly one completed and total field"
  fi
  MATRIX_COMPLETED="${fields%%$'\n'*}"
  MATRIX_TOTAL="${fields#*$'\n'}"
  [[ "$MATRIX_COMPLETED" =~ ^(0|[1-9][0-9]{0,17})$ ]] || fail "invalid matrix progress: completed count"
  positive_integer "$MATRIX_TOTAL" || fail "invalid matrix progress: total must be positive"
  (( MATRIX_COMPLETED <= MATRIX_TOTAL )) || fail "invalid matrix progress: completed exceeds total"
}

while true; do
  previous_completed="$MATRIX_COMPLETED"
  previous_total="$MATRIX_TOTAL"
  matrix_progress
  completed="$MATRIX_COMPLETED"
  total="$MATRIX_TOTAL"
  echo "matrix_progress: ${completed}/${total:-unknown}"

  if [[ "$REPORT_RAN" == 1 ]]; then
    [[ -z "$previous_total" || "$total" == "$previous_total" ]] || fail "matrix total changed after report-all"
    (( completed > previous_completed )) || fail "report-all did not advance completed matrix nodes"
  fi

  if [[ -n "$total" && "$completed" == "$total" ]]; then
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
    verify_compiler
    exec "${cmd[@]}"
  fi

  verify_compiler
  "$LILA_BIN" test262 report-all \
    --jobs "$JOBS" \
    --execution-backend "$BACKEND" \
    --suite-root "$SUITE_ROOT" \
    --snapshot-dir "$SNAPSHOT_DIR" \
    --snapshot-name "$SNAPSHOT_NAME" \
    --resume \
    --threads "$THREADS" \
    --max-matrix-nodes "$MAX_MATRIX_NODES"
  REPORT_RAN=1
done
