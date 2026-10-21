#!/usr/bin/env bash
set -euo pipefail

source "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/publication-settings.sh"

REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
: "${SCRIPTW_PUBLICATION_MANIFEST:?publication session manifest is required}"
SOURCE_COMMIT="$(git -C "$REPO_ROOT" rev-parse --verify HEAD)"
compiler_digest() {
  local digest
  digest="$(sha256sum < "$SCRIPTW_BIN")" || fail "cannot hash executable: $SCRIPTW_BIN"
  printf '%s\n' "${digest%% *}"
}
COMPILER_SHA256="$(compiler_digest)"

# The supervisor binds this family to observed inputs across invocations.
# This remains observed provenance, not an attestation of how the binary was built.
printf 'source_commit: %s\ncompiler_sha256: %s\n' "$SOURCE_COMMIT" "$COMPILER_SHA256"
printf 'execution_backend: %s\nsnapshot_name: %s\n' "$BACKEND" "$SNAPSHOT_NAME"
printf 'suite_root: %s\nsnapshot_dir: %s\n' "$SUITE_ROOT" "$SNAPSHOT_DIR"

verify_compiler() {
  [[ -x "$SCRIPTW_BIN" ]] || fail "executable disappeared or is no longer executable: $SCRIPTW_BIN"
  [[ "$(compiler_digest)" == "$COMPILER_SHA256" ]] || fail "compiler changed during publication; retain the log and checkpoints for triage"
  [[ "$(git -C "$REPO_ROOT" rev-parse --verify HEAD)" == "$SOURCE_COMMIT" ]] || fail "source commit changed during publication; retain the log and checkpoints for triage"
  python3 "$REPO_ROOT/scripts/publication-session.py" verify "$SCRIPTW_PUBLICATION_MANIFEST"
}

matrix_progress() {
  local progress fields status
  local -a progress_command
  verify_compiler
  if progress="$(
    "$SCRIPTW_BIN" test262 progress-status \
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
    python3 "$REPO_ROOT/scripts/publication-session.py" require-fresh "$SCRIPTW_PUBLICATION_MANIFEST"
    echo "progress-status unavailable before first report-all (exit $status); attempting initial checkpoint" >&2
    MATRIX_COMPLETED=0
    MATRIX_TOTAL=""
    return
  fi

  progress_command=(python3 "$REPO_ROOT/scripts/publication-session.py"
    record-progress "$SCRIPTW_PUBLICATION_MANIFEST")
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
      "$SCRIPTW_BIN" test262 publish-status
      --jobs "$JOBS"
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
  report_command=(
    "$SCRIPTW_BIN" test262 report-all
    --jobs "$JOBS"
    --execution-backend "$BACKEND"
    --suite-root "$SUITE_ROOT"
    --snapshot-dir "$SNAPSHOT_DIR"
    --snapshot-name "$SNAPSHOT_NAME"
    --resume
    --threads "$THREADS"
    --max-matrix-nodes "$MAX_MATRIX_NODES"
  )
  if [[ "$REUSE_CASE_WORKERS" == 1 ]]; then
    report_command+=(--reuse-case-workers)
  fi
  "${report_command[@]}"
  REPORT_RAN=1
done
