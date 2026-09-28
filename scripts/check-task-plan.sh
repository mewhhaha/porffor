#!/usr/bin/env bash
set -euo pipefail

# Retain the CI entry point while validating the current measured failure backlog.
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
exec python3 "$script_dir/check-failure-backlog.py" --tasks-dir "${TASKS_DIR:-tasks}" "$@"
