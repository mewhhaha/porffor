#!/bin/sh
# Run a command pinned to a share of its inherited CPU affinity (default half).
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
#   ./scripts/capped.sh cargo test -p lila-engine --lib
#   ./scripts/capped.sh ./target/release/lila run --execution-backend wasm x.js
#
# LILA_CPU_PERCENT overrides the share (default 50).

set -eu

logical_cpus() {
  if command -v getconf >/dev/null 2>&1; then
    getconf _NPROCESSORS_ONLN 2>/dev/null || echo 4
  else
    echo 4
  fi
}

percent=${LILA_CPU_PERCENT:-50}
case "$percent" in
  [1-9]|[1-9][0-9]|100) ;;
  *) echo "LILA_CPU_PERCENT must be a decimal integer from 1 to 100" >&2; exit 2 ;;
esac

[ "$#" -gt 0 ] || { echo "usage: ./scripts/capped.sh <command...>" >&2; exit 2; }

if command -v taskset >/dev/null 2>&1; then
  # Online CPU count is not the usable CPU set: containers, taskset and batch
  # schedulers can assign a sparse/non-zero-based subset. Never widen that set.
  affinity=$(LC_ALL=C taskset -pc "$$") || {
    echo "capped: cannot read inherited CPU affinity" >&2; exit 1;
  }
  affinity=${affinity##*: }
  selection=$(printf '%s\n' "$affinity" | awk -v percent="$percent" '
    /^[0-9]+(-[0-9]+)?(,[0-9]+(-[0-9]+)?)*$/ {
      n = split($0, ranges, ",")
      for (i = 1; i <= n; i++) {
        parts = split(ranges[i], ends, "-")
        first[i] = ends[1] + 0
        last[i] = (parts == 2 ? ends[2] + 0 : first[i])
        if (last[i] < first[i] || (i > 1 && first[i] <= last[i - 1])) exit 1
        cpus += last[i] - first[i] + 1
      }
      allowed = int(cpus * percent / 100)
      if (allowed < 1) allowed = 1
      remaining = allowed
      for (i = 1; i <= n && remaining > 0; i++) {
        take = last[i] - first[i] + 1
        if (take > remaining) take = remaining
        list = list (list == "" ? "" : ",") first[i]
        if (take > 1) list = list "-" (first[i] + take - 1)
        remaining -= take
      }
      printf "%d %d %s\n", cpus, allowed, list
      next
    }
    { exit 1 }
  ') || { echo "capped: invalid inherited CPU affinity: $affinity" >&2; exit 1; }
  [ -n "$selection" ] || { echo "capped: empty inherited CPU affinity" >&2; exit 1; }
  read -r cpus allowed cpu_list <<EOF_SELECTION
$selection
EOF_SELECTION

  # Keep compilation-unit concurrency within the same hard affinity ceiling.
  CARGO_BUILD_JOBS=$allowed
  export CARGO_BUILD_JOBS
  echo "capped: CPUs $cpu_list of $cpus available (${percent}%), CARGO_BUILD_JOBS=$allowed" >&2
  exec taskset -c "$cpu_list" "$@"
fi

# No taskset (non-Linux): fall back to the job-count knobs alone and say so,
# rather than silently running uncapped.
cpus=$(logical_cpus)
case "$cpus" in ''|*[!0-9]*|0) cpus=4 ;; esac
allowed=$(( cpus * percent / 100 ))
[ "$allowed" -ge 1 ] || allowed=1
CARGO_BUILD_JOBS=$allowed
export CARGO_BUILD_JOBS
echo "capped: taskset unavailable, limiting job counts only (CARGO_BUILD_JOBS=$allowed)" >&2
exec "$@"
