#!/usr/bin/env bash
set -euo pipefail

source "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/lib/publication-settings.sh"

REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
export REPO_ROOT SCRIPTW_BIN SUITE_ROOT SNAPSHOT_DIR SNAPSHOT_NAME THREADS JOBS ISOLATE_CASES REUSE_CASE_WORKERS MAX_MATRIX_NODES README_PATH
exec python3 "$REPO_ROOT/scripts/publication-session.py" run
