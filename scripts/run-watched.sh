#!/bin/sh
# Run a long command under a stall guard.
#
# Two failure modes have cost real hours in this repository, and neither one
# announces itself:
#
#   1. A test or Test262 case hangs. Wasm-AOT compilation has no wall-clock
#      bound, and `Atomics.wait` blocks outright, so `cargo test -p lila-cli`
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
# killed. `lila test262 report` streams "test262 checkpoint: N/M cases" lines
# precisely so this works - filtering them out defeats it. Let the full output
# reach the log and grep the log afterwards.
#
# usage:
#   ./scripts/run-watched.sh [--label NAME] [--stall SECONDS] [--poll SECONDS] -- <command...>
#
# examples:
#   ./scripts/run-watched.sh --label cli -- cargo test -p lila-cli --test cli -- --skip atomics_wait_core
#   ./scripts/run-watched.sh --label sweep --stall 900 -- ./target/release/lila test262 report-all --resume
#
# Exit status is the wrapped command's, or 124 if it was killed for stalling.

# Python 3 owns the POSIX process group, so descendants cannot outlive a stall
# or cancellation. The command-line interface and log location stay unchanged.
set -eu
exec python3 "$(dirname "$0")/run_watched.py" "$@"
