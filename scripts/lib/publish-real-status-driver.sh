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

REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
: "${LILA_PUBLICATION_MANIFEST:?publication session manifest is required}"
SOURCE_COMMIT="$(git -C "$REPO_ROOT" rev-parse --verify HEAD)"
compiler_digest() {
  local digest
  digest="$(sha256sum < "$LILA_BIN")" || fail "cannot hash executable: $LILA_BIN"
  printf '%s\n' "${digest%% *}"
}
COMPILER_SHA256="$(compiler_digest)"

# The supervisor binds this family to observed inputs across invocations.
# This remains observed provenance, not an attestation of how the binary was built.
printf 'source_commit: %s\ncompiler_sha256: %s\n' "$SOURCE_COMMIT" "$COMPILER_SHA256"
printf 'execution_backend: %s\nsnapshot_name: %s\n' "$BACKEND" "$SNAPSHOT_NAME"
printf 'suite_root: %s\nsnapshot_dir: %s\n' "$SUITE_ROOT" "$SNAPSHOT_DIR"

verify_compiler() {
  [[ -x "$LILA_BIN" ]] || fail "executable disappeared or is no longer executable: $LILA_BIN"
  [[ "$(compiler_digest)" == "$COMPILER_SHA256" ]] || fail "compiler changed during publication; retain the log and checkpoints for triage"
  [[ "$(git -C "$REPO_ROOT" rev-parse --verify HEAD)" == "$SOURCE_COMMIT" ]] || fail "source commit changed during publication; retain the log and checkpoints for triage"
  python3 "$REPO_ROOT/scripts/publication-session.py" verify "$LILA_PUBLICATION_MANIFEST"
}

matrix_progress() {
  local progress fields status
  local -a progress_command
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
    python3 "$REPO_ROOT/scripts/publication-session.py" require-fresh "$LILA_PUBLICATION_MANIFEST"
    echo "progress-status unavailable before first report-all (exit $status); attempting initial checkpoint" >&2
    MATRIX_COMPLETED=0
    MATRIX_TOTAL=""
    return
  fi

  progress_command=(python3 "$REPO_ROOT/scripts/publication-session.py"
    record-progress "$LILA_PUBLICATION_MANIFEST")
  if [[ "$REPORT_RAN" == 1 ]]; then
    progress_command+=(--after-report)
  fi
  # Persist the observation BEFORE deciding to resume or publish. This also
  # checks observations made by earlier invocations of this snapshot family.
  if ! fields="$("${progress_command[@]}" <<<"$progress")"; then
    fail "invalid or inconsistent matrix progress"
  fi
  MATRIX_COMPLETED="${fields%%:*}"
  MATRIX_TOTAL="${fields#*:}"
}

while true; do
  matrix_progress
  completed="$MATRIX_COMPLETED"
  total="$MATRIX_TOTAL"
  echo "matrix_progress: ${completed}/${total:-unknown}"

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
