#!/bin/sh
# Run a long command under a stall guard.
#
# Two failure modes have cost real hours in this repository, and neither one
# announces itself:
#
#   1. A test or Test262 case hangs. Wasm-AOT compilation has no wall-clock
#      bound, and `Atomics.wait` blocks outright, so `cargo test -p porffor-cli`
#      runs to 580/581 and then sits forever burning cores.
#   2. The output is piped somewhere buffering, so "no output" is
#      indistinguishable from "still working" and nobody notices for hours.
#
# This wrapper fixes both: output always lands in a log file that is polled for
# growth, and a run whose log stops growing for --stall seconds is killed and
# reported rather than left to spin.
#
# DO NOT pipe the wrapped command through grep, head, tail or any other filter.
# The guard judges liveness by whether the log is growing, so a filter that
# discards progress output makes a perfectly healthy run look stalled and get
# killed. `porf test262 report` streams "test262 checkpoint: N/M cases" lines
# precisely so this works - filtering them out defeats it. Let the full output
# reach the log and grep the log afterwards.
#
# usage:
#   ./scripts/run-watched.sh [--label NAME] [--stall SECONDS] [--poll SECONDS] -- <command...>
#
# examples:
#   ./scripts/run-watched.sh --label cli -- cargo test -p porffor-cli --test cli -- --skip atomics_wait_core
#   ./scripts/run-watched.sh --label sweep --stall 900 -- ./target/release/porf test262 report-all --resume
#
# Exit status is the wrapped command's, or 124 if it was killed for stalling.

set -eu

label=run
stall=300
poll=15

while [ "$#" -gt 0 ]; do
  case "$1" in
    --label) label=${2:?--label needs a value}; shift 2 ;;
    --stall) stall=${2:?--stall needs a value}; shift 2 ;;
    --poll)  poll=${2:?--poll needs a value};   shift 2 ;;
    --) shift; break ;;
    -h|--help) sed -n '2,26p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "unknown option: $1 (did you forget the -- separator?)" >&2; exit 2 ;;
  esac
done

[ "$#" -gt 0 ] || { echo "run-watched: no command given after --" >&2; exit 2; }

case "$stall$poll" in *[!0-9]*) echo "--stall and --poll must be integers" >&2; exit 2 ;; esac
[ "$poll" -gt 0 ] || { echo "--poll must be positive" >&2; exit 2; }

log_dir=target/watched
mkdir -p "$log_dir"
log=$log_dir/$label.log
: > "$log"

echo "run-watched: $* "
echo "run-watched: log $log, stall limit ${stall}s"

# The command owns the log; the watcher only reads it. Everything long-running
# goes through the CPU cap: these runs are the ones that make the machine
# unusable, and job-count flags alone do not bound them (see scripts/capped.sh).
capped=./scripts/capped.sh
[ -x "$capped" ] || capped=""
# shellcheck disable=SC2086
$capped "$@" > "$log" 2>&1 &
command_pid=$!

previous_size=-1
quiet_seconds=0
elapsed=0

while kill -0 "$command_pid" 2>/dev/null; do
  sleep "$poll"
  elapsed=$((elapsed + poll))
  size=$(wc -c < "$log" 2>/dev/null || echo 0)

  if [ "$size" -eq "$previous_size" ]; then
    quiet_seconds=$((quiet_seconds + poll))
  else
    quiet_seconds=0
    # Heartbeat so a healthy long run is visibly healthy.
    printf 'run-watched: %ss elapsed, %s bytes, last: %s\n' \
      "$elapsed" "$size" "$(tail -n 1 "$log" 2>/dev/null | cut -c1-100)"
  fi
  previous_size=$size

  if [ "$quiet_seconds" -ge "$stall" ]; then
    echo "run-watched: STALLED — no output for ${quiet_seconds}s after ${elapsed}s. Killing." >&2
    echo "run-watched: last 5 lines of $log:" >&2
    tail -n 5 "$log" >&2 || true
    kill -TERM "$command_pid" 2>/dev/null || true
    sleep 5
    kill -KILL "$command_pid" 2>/dev/null || true
    wait "$command_pid" 2>/dev/null || true
    exit 124
  fi
done

wait "$command_pid"
status=$?
echo "run-watched: finished in ${elapsed}s with status $status ($log)"
exit "$status"
