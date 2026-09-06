#!/bin/sh
set -eu

# Build configuration lives in .cargo/config.toml so that this wrapper and a
# bare `cargo` invocation agree. Exporting RUSTFLAGS from here made the two
# entry points disagree on Cargo's unit fingerprint, so each invalidated the
# other's artifacts and forced a full workspace rebuild. Do not reintroduce it.
#
# The default job count also comes from .cargo/config.toml. LILA_JOBS remains
# available to request a lower cap for a single invocation; unlike RUSTFLAGS,
# job count does not affect codegen and so does not fork the fingerprint.
jobs_flag=
if [ -n "${LILA_JOBS:-}" ]; then
  case "$LILA_JOBS" in
    ''|*[!0-9]*|0) echo "LILA_JOBS must be a positive integer" >&2; exit 2 ;;
  esac
  jobs_flag="--jobs $LILA_JOBS"
fi

usage() {
  cat <<'EOF'
usage: ./scripts/dev.sh <command> [cargo/lila arguments]

commands:
  build [args]       cargo build (defaults to -p lila-cli)
  check [args]       cargo check (defaults to --workspace)
  exact-test <args>  cargo test with caller-supplied package/test filters
  test262 <args>     build lila, then run `lila test262 ...`
  timings [args]     Cargo HTML timings for lila-ir and lila-aot-wasm

Build flags and the default job count come from .cargo/config.toml, so this
wrapper and a bare `cargo` invocation share one artifact fingerprint. Set
LILA_JOBS to request a lower cap for a single invocation. This wrapper uses
the existing target/ directory and never deletes developer artifacts.
EOF
}

command=${1:-}
[ -n "$command" ] || { usage; exit 2; }
shift

case "$command" in
  build)
    if [ "$#" -eq 0 ]; then set -- -p lila-cli; fi
    exec cargo build $jobs_flag "$@"
    ;;
  check)
    if [ "$#" -eq 0 ]; then set -- --workspace; fi
    exec cargo check $jobs_flag "$@"
    ;;
  exact-test)
    [ "$#" -gt 0 ] || { echo "exact-test needs cargo test arguments" >&2; exit 2; }
    exec cargo test $jobs_flag "$@"
    ;;
  test262)
    # Cargo owns target-dir, target triples and runners. A hard-coded path
    # can execute a stale binary when any of those settings changes.
    exec cargo run $jobs_flag -p lila-cli -- test262 "$@"
    ;;
  timings)
    if [ "$#" -eq 0 ]; then set -- -p lila-ir -p lila-aot-wasm; fi
    exec cargo build $jobs_flag --timings "$@"
    ;;
  -h|--help|help) usage ;;
  *) echo "unknown developer command: $command" >&2; usage >&2; exit 2 ;;
esac
