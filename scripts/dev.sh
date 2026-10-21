#!/bin/sh
set -eu

# Build configuration lives in .cargo/config.toml so that this wrapper and a
# bare `cargo` invocation agree. Exporting RUSTFLAGS from here made the two
# entry points disagree on Cargo's unit fingerprint, so each invalidated the
# other's artifacts and forced a full workspace rebuild. Do not reintroduce it.
#
# The default job count also comes from .cargo/config.toml. SCRIPTW_JOBS remains
# available to request a lower cap for a single invocation; unlike RUSTFLAGS,
# job count does not affect codegen and so does not fork the fingerprint.
jobs=
if [ -n "${SCRIPTW_JOBS:-}" ]; then
  case "$SCRIPTW_JOBS" in
    ''|*[!0-9]*|0) echo "SCRIPTW_JOBS must be a positive integer" >&2; exit 2 ;;
  esac
  jobs=$SCRIPTW_JOBS
fi

# Build the optional Cargo flag as separate arguments, never by word splitting.
exec_cargo() {
  cargo_command=$1
  shift
  if [ -n "$jobs" ]; then set -- --jobs "$jobs" "$@"; fi
  exec cargo "$cargo_command" "$@"
}

usage() {
  cat <<'EOF'
usage: ./scripts/dev.sh <command> [cargo/scriptw arguments]

commands:
  build [args]       cargo build (defaults to -p scriptw-cli)
  check [args]       cargo check (defaults to --workspace)
  exact-test <args>  cargo test with caller-supplied package/test filters
  test262 <args>     build scriptw, then run `scriptw test262 ...`
  timings [args]     Cargo HTML timings for scriptw-ir and scriptw-aot-wasm

Build flags and the default job count come from .cargo/config.toml, so this
wrapper and a bare `cargo` invocation share one artifact fingerprint. Set
SCRIPTW_JOBS to request a lower cap for a single invocation. This wrapper uses
the existing target/ directory and never deletes developer artifacts.
EOF
}

command=${1:-}
[ -n "$command" ] || { usage; exit 2; }
shift

case "$command" in
  build)
    if [ "$#" -eq 0 ]; then set -- -p scriptw-cli; fi
    exec_cargo build "$@"
    ;;
  check)
    if [ "$#" -eq 0 ]; then set -- --workspace; fi
    exec_cargo check "$@"
    ;;
  exact-test)
    [ "$#" -gt 0 ] || { echo "exact-test needs cargo test arguments" >&2; exit 2; }
    exec_cargo test "$@"
    ;;
  test262)
    # Cargo owns target-dir, target triples and runners. A hard-coded path
    # can execute a stale binary when any of those settings changes.
    exec_cargo run -p scriptw-cli -- test262 "$@"
    ;;
  timings)
    if [ "$#" -eq 0 ]; then set -- -p scriptw-ir -p scriptw-aot-wasm; fi
    exec_cargo build --timings "$@"
    ;;
  -h|--help|help) usage ;;
  *) echo "unknown developer command: $command" >&2; usage >&2; exit 2 ;;
esac
