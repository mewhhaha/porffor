#!/bin/sh
# Run a command pinned to at most half the machine's CPUs.
#
# Job-count flags are not a cap. `cargo --jobs N` limits compilation units but
# not rustc's own threads; `--test-threads N` limits concurrent tests but each
# Test262/engine test drives a Cranelift rayon pool that itself defaults to half
# the logical CPUs. Multiplied together those overshoot badly and make the
# machine unusable.
#
# CPU affinity is a hard ceiling instead: the process and every child it spawns
# are confined to the listed CPUs regardless of how many threads they create.
#
# usage:
#   ./scripts/capped.sh cargo test -p porffor-engine --lib
#   ./scripts/capped.sh ./target/release/porf run --execution-backend wasm x.js
#
# PORFFOR_CPU_PERCENT overrides the share (default 50).

set -eu

logical_cpus() {
  if command -v getconf >/dev/null 2>&1; then
    getconf _NPROCESSORS_ONLN 2>/dev/null || echo 4
  else
    echo 4
  fi
}

cpus=$(logical_cpus)
case "$cpus" in ''|*[!0-9]*|0) cpus=4 ;; esac

percent=${PORFFOR_CPU_PERCENT:-50}
case "$percent" in ''|*[!0-9]*|0) echo "PORFFOR_CPU_PERCENT must be a positive integer" >&2; exit 2 ;; esac

allowed=$(( cpus * percent / 100 ))
[ "$allowed" -ge 1 ] || allowed=1
last=$(( allowed - 1 ))

[ "$#" -gt 0 ] || { echo "usage: ./scripts/capped.sh <command...>" >&2; exit 2; }

# Keep cargo's own job count at or below the affinity width, so it does not
# queue more units than there are usable CPUs.
CARGO_BUILD_JOBS=$allowed
export CARGO_BUILD_JOBS

if command -v taskset >/dev/null 2>&1; then
  echo "capped: CPUs 0-$last of $cpus (${percent}%), CARGO_BUILD_JOBS=$allowed" >&2
  exec taskset -c "0-$last" "$@"
fi

# No taskset (non-Linux): fall back to the job-count knobs alone and say so,
# rather than silently running uncapped.
echo "capped: taskset unavailable, limiting job counts only (CARGO_BUILD_JOBS=$allowed)" >&2
exec "$@"
