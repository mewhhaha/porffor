#!/bin/sh
set -eu

logical_cpus() {
  if command -v getconf >/dev/null 2>&1; then
    getconf _NPROCESSORS_ONLN 2>/dev/null || true
  elif command -v sysctl >/dev/null 2>&1; then
    sysctl -n hw.logicalcpu 2>/dev/null || true
  fi
}

cpus=$(logical_cpus)
case "$cpus" in
  ''|*[!0-9]*|0) cpus=2 ;;
esac
jobs=$((cpus / 2))
[ "$jobs" -ge 1 ] || jobs=1

# A caller can deliberately choose a lower cap, but the developer wrapper
# never exceeds half the logical CPUs (eight on the primary development box).
if [ -n "${PORFFOR_JOBS:-}" ]; then
  case "$PORFFOR_JOBS" in
    ''|*[!0-9]*|0) echo "PORFFOR_JOBS must be a positive integer" >&2; exit 2 ;;
  esac
  if [ "$PORFFOR_JOBS" -lt "$jobs" ]; then
    jobs=$PORFFOR_JOBS
  fi
fi

# Rust invokes the C compiler as its linker driver. lld is optional: retain a
# portable system-linker fallback on machines where it is not installed.
if command -v ld.lld >/dev/null 2>&1 || command -v lld >/dev/null 2>&1; then
  if [ -n "${RUSTFLAGS:-}" ]; then
    RUSTFLAGS="$RUSTFLAGS -C link-arg=-fuse-ld=lld"
  else
    RUSTFLAGS="-C link-arg=-fuse-ld=lld"
  fi
  export RUSTFLAGS
fi

export CARGO_BUILD_JOBS=$jobs

usage() {
  cat <<'EOF'
usage: ./scripts/dev.sh <command> [cargo/porf arguments]

commands:
  build [args]       cargo build (defaults to -p porffor-cli)
  check [args]       cargo check (defaults to --workspace)
  exact-test <args>  cargo test with caller-supplied package/test filters
  test262 <args>     build porf, then run `porf test262 ...`
  timings [args]     Cargo HTML timings for porffor-ir and porffor-aot-wasm

Set PORFFOR_JOBS to request a lower cap. This wrapper uses the existing target/
directory and never deletes developer artifacts.
EOF
}

command=${1:-}
[ -n "$command" ] || { usage; exit 2; }
shift

case "$command" in
  build)
    if [ "$#" -eq 0 ]; then set -- -p porffor-cli; fi
    exec cargo build --jobs "$jobs" "$@"
    ;;
  check)
    if [ "$#" -eq 0 ]; then set -- --workspace; fi
    exec cargo check --jobs "$jobs" "$@"
    ;;
  exact-test)
    [ "$#" -gt 0 ] || { echo "exact-test needs cargo test arguments" >&2; exit 2; }
    exec cargo test --jobs "$jobs" "$@"
    ;;
  test262)
    cargo build --jobs "$jobs" -p porffor-cli
    exec ./target/debug/porf test262 "$@"
    ;;
  timings)
    if [ "$#" -eq 0 ]; then set -- -p porffor-ir -p porffor-aot-wasm; fi
    exec cargo build --jobs "$jobs" --timings "$@"
    ;;
  -h|--help|help) usage ;;
  *) echo "unknown developer command: $command" >&2; usage >&2; exit 2 ;;
esac
