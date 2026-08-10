#!/bin/sh
# Rung 1c -- the whole `porffor-cli --test cli` suite -- as RESUMABLE per-module
# chunks.
#
# usage:
#   setsid ./scripts/rung1c-chunks.sh >/dev/null 2>&1 </dev/null &
#   disown
#
# Re-run it verbatim after a container restart. It skips every chunk already
# banked in the done-file and continues with the next one.
#
# # Why this exists as a tracked script
#
# The suite is ~2.5 h on a 4-CPU box, libtest has NO resume, and this container
# restarts about hourly. Four consecutive single-invocation attempts died
# mid-run (batch 3 twice at 15 tests, batch 4 at 44 tests); the first chunked
# attempt banked 262 of 593 and then died too -- but it lost only the chunk in
# flight. Chunking is what makes rung 1c reachable at all here.
#
# That machinery previously lived in `target/watched/b4c-rung1c.sh`, which is
# gitignored, so every batch re-derived it from a lane note and rung 1c has
# never survived a restart as a repeatable artefact. The SCRIPT is what needs
# tracking. Its run state (the done-file, the logs, the results file) correctly
# stays under `target/`.
#
# `docs/rust-rewrite/batch-workflow.md` section "Rung 1c terminates, and checks
# its own expectations" is the reference for what a green run means.
#
# # Three load-bearing properties. Do not "simplify" any of them.
#
# 1. `--test-threads=3`, NEVER 1. libtest names each worker thread after the
#    test it runs, and `known_failures::execution_path` routes on that name.
#    Under `--test-threads=1` every test runs on the thread named `main`, the
#    name is unavailable, and all ~600 tests fall back to spawning a cold `porf`
#    child instead of the warm in-process call the runtime estimate is built on.
#    It is still correct and terminating, just far slower. For one test use
#    `-- --exact <name>`, not a lower thread count.
#
# 2. libtest filters are SUBSTRINGS, not exact names. `array::` therefore also
#    selects `typed_array::`, so the array chunk carries `--skip typed_array::`
#    and nothing is counted twice. THE GENERAL RULE, because this will recur:
#    if a new area module's stem ends with an existing one (`sub_array` ends
#    with `array`), the shorter chunk must skip the longer one. This is checked
#    by `known_failures::rung_1c_chunks_cover_every_cli_area_module`, which
#    reads the `run_chunk` lines below back out of this file -- so a chunk set
#    that no longer partitions the suite is a red test, not a silent subset.
#
# 3. THE DONE-FILE *IS* THE RESUME STATE. Do not "clean it up" between runs.
#    Deleting it re-runs hours of already-banked work.
#
# 4. A CHUNK IS BANKED ONLY WHEN IT PRODUCED A VERDICT. Exit 124 (the stall
#    guard SIGKILLed it), a missing log, a missing `test result:` line, or
#    `running 0 tests` all leave the chunk out of the done-file and make the
#    script exit 1 with RUN INCOMPLETE. It used to bank on every exit status,
#    which meant a killed chunk was skipped forever and the run still printed
#    ALL CHUNKS DONE -- a chunk that measured nothing, recorded as measured.
#    A libtest FAILURE is a verdict and IS banked; that is the point of the run.
#
# # The partition is the whole claim
#
# Every test runs exactly once and the union of the chunks is the suite, so a
# chunked run is a complete rung 1c rather than a subset. The arithmetic that
# proves it, per chunk: `N passed + N filtered out` must equal the compiled test
# count reported by `cargo test -p porffor-cli --test cli -- --list | tail -1`.
# Recount the source side with the EXACT-line form (a substring
# `grep '#\[test\]'` over-counts, because prose in `known_failures.rs` names the
# attribute):
#
#   awk '/^[[:space:]]*#\[test\][[:space:]]*$/{n++} END{print n}' \
#     crates/porffor-cli/tests/cli/*.rs
#
# minus the `#[cfg(feature = "spec-exec-oracle")]`-gated tests in `frontend.rs`.
#
# What chunking gives up, stated plainly: whole-suite ordering/interference
# effects, and a single `test result:` line. Neither is what the known-failures
# ledger needs -- it needs the failing set, with messages.
#
# # Budget, measured on this 4-CPU box rather than estimated
#
# Per-module cost spans 6.8 s/test (`heap`) to 38 s/test (`date`). Add ~900 s of
# dead wall-clock in `binary_data::` for the declared T17 `Atomics.wait` hang,
# which `tests/cli/main.rs` converts into a bounded failure. `PORFFOR_CPU_PERCENT=100`
# below is load-bearing: `scripts/run-watched.sh` routes through
# `scripts/capped.sh`, which silently pins to half the CPUs and made an earlier
# attempt look 2x slower than it was.

set -u

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/.." && pwd)
cd "$repo_root" || exit 1

RESULTS=${RUNG1C_RESULTS:-target/lane-notes/rung1c-chunks.md}
DONE=${RUNG1C_DONE:-target/watched/rung1c-done}

# The batch-4 done-file, which this script's untracked predecessor wrote.
#
# NOT read by default, and that is a correctness rule rather than caution. Its
# verdicts were measured against a compiler that batch 5 then rewrote
# (`porffor-aot-wasm/{functions,objects,heap}.rs` and six `porffor-ir` files),
# so seeding from it makes a resumed run execute 6 of 17 chunks and still print
# ALL CHUNKS DONE -- and the ledger is then filled from a run that never
# measured most of the suite. `known_failures` is in that legacy list, so even
# the ledger gate itself would be skipped.
#
# Opt in only when the tree has NOT moved since those chunks were measured:
#
#   RUNG1C_SEED_FROM_B4=1 ./scripts/rung1c-chunks.sh
#
# `known_failures` is dropped from any seed unconditionally: it is cheap, it is
# the gate that validates the ledger the whole run exists to fill, and a stale
# verdict for it is worthless.
LEGACY_DONE=target/watched/b4c-done

mkdir -p target/watched target/lane-notes

if [ ! -f "$DONE" ] && [ -f "$LEGACY_DONE" ]; then
  if [ "${RUNG1C_SEED_FROM_B4:-0}" = "1" ]; then
    grep -vx 'known_failures' "$LEGACY_DONE" > "$DONE"
    echo "rung1c: seeded $DONE from $LEGACY_DONE ($(wc -l < "$DONE" | tr -d ' ') chunk(s) banked, known_failures excluded)"
    echo "rung1c: WARNING -- those verdicts were measured at an older tree. Re-run any chunk whose subsystem has changed."
  else
    echo "rung1c: NOT seeding from $LEGACY_DONE (set RUNG1C_SEED_FROM_B4=1 to inherit its chunk verdicts)."
    echo "rung1c: its verdicts predate the current compiler; a blind seed reports ALL CHUNKS DONE after running a minority of the suite."
  fi
fi
touch "$DONE"

# Chunks that ran but produced no bankable verdict this pass. A run with any of
# these is NOT a complete rung 1c, so it must not print ALL CHUNKS DONE.
UNBANKED=""

# run_chunk <module-stem> <module-stem>:: [--skip <other-stem>::]...
#
# The first argument is the done-file key and the log name; the rest are passed
# to libtest verbatim. Keep the second argument spelled `<stem>::` -- the
# hygiene test asserts it, by parsing these very lines back out of this file.
# Keep the invocations below spelled `run_chunk <stem> <stem>::` for the same
# reason: a wrapper name would make every chunk invisible to that check.
run_chunk() {
  name=$1
  shift
  if grep -qx "$name" "$DONE"; then
    echo "rung1c: skip $name (already banked in $DONE)"
    return 0
  fi
  log=target/watched/rung1c-$name.log
  echo "=== $name START $(date -u +%Y-%m-%dT%H:%M:%SZ) ===" >> "$RESULTS"
  PORFFOR_CPU_PERCENT=100 ./scripts/run-watched.sh --label "rung1c-$name" --stall 900 -- \
    cargo test -p porffor-cli --test cli -- --test-threads=3 "$@"
  rc=$?
  line=$(grep -E '^test result:' "$log" 2>/dev/null | tail -1)
  # The partition arithmetic, banked mechanically instead of left for a human
  # to re-derive from the logs. `running N tests` is what libtest selected and
  # `filtered out` is what it did not; per chunk the two must sum to the
  # compiled test count (`cargo test -p porffor-cli --test cli -- --list | tail -1`).
  ran=$(grep -Eo '^running ([0-9]+) tests?' "$log" 2>/dev/null | tail -1 | awk '{print $2}')
  filtered=$(printf '%s' "$line" | grep -Eo '[0-9]+ filtered out' | awk '{print $1}')
  ran=${ran:-0}
  filtered=${filtered:-0}
  echo "$name EXIT=$rc  ran=$ran filtered_out=$filtered  sum=$((ran + filtered))  $line" >> "$RESULTS"

  # BANK ONLY ON A VERDICT. `run-watched.sh` returns 124 after it SIGKILLs a
  # stalled run, and the done-file IS the resume state -- so banking on every
  # exit status records a chunk that produced no verdict as complete and skips
  # it forever, which is the dynamic form of the "subset masquerading as rung
  # 1c" this whole script exists to prevent. `binary_data::` sits directly on
  # that boundary (HANG_TIMEOUT 900 s vs --stall 900). A libtest FAILURE (a
  # `test result:` line with rc != 0) is a verdict and is banked.
  if [ "$rc" -eq 124 ] || [ ! -f "$log" ] || [ -z "$line" ]; then
    echo "$name NO-VERDICT (EXIT=$rc): killed by the stall guard, or no 'test result:' line. NOT banked; re-run this chunk." >> "$RESULTS"
    echo "rung1c: $name produced no verdict (EXIT=$rc); not banked" >&2
    echo "=== $name END $(date -u +%Y-%m-%dT%H:%M:%SZ) ===" >> "$RESULTS"
    UNBANKED="$UNBANKED $name"
    return 1
  fi
  # A filter that matches nothing exits 0 on `0 passed`. Banking that records a
  # chunk which measured nothing -- exactly what a missing `mod <stem>;` line in
  # tests/cli/main.rs produces.
  if [ "$ran" -eq 0 ]; then
    echo "$name RAN ZERO TESTS: the filter selected nothing (missing 'mod $name;' in crates/porffor-cli/tests/cli/main.rs?). NOT banked." >> "$RESULTS"
    echo "rung1c: $name selected zero tests; not banked" >&2
    echo "=== $name END $(date -u +%Y-%m-%dT%H:%M:%SZ) ===" >> "$RESULTS"
    UNBANKED="$UNBANKED $name"
    return 1
  fi
  if [ "$rc" -ne 0 ]; then
    # Names AND per-test stdout. The batch-4 predecessor banked only the
    # indented lines, which is the failure names plus their backtraces and NOT
    # the panic message -- so when its `frontend` log was truncated by a
    # container restart the message was unrecoverable and no ledger row could be
    # written for a measured failure. `should_panic(expected = "...")` needs the
    # message, so bank the message.
    echo "--- failures in $name (names, then per-test stdout) ---" >> "$RESULTS"
    sed -n '/^failures:$/,/^test result:/p' "$log" | head -400 >> "$RESULTS"
  fi
  echo "=== $name END $(date -u +%Y-%m-%dT%H:%M:%SZ) ===" >> "$RESULTS"
  echo "$name" >> "$DONE"
  return 0
}


# Order is cheapest-and-most-diagnostic first, so a short container window still
# banks something useful. `known_failures` leads deliberately: it holds
# `ledger_is_well_formed`, and a malformed ledger makes every other chunk take
# the guarded-subprocess path and look merely slow.
run_chunk known_failures    known_failures::
run_chunk throw_propagation throw_propagation::
run_chunk dynamic           dynamic::
run_chunk heap              heap::
run_chunk date              date::
run_chunk iterator          iterator::
run_chunk iterator_helpers  iterator_helpers::
run_chunk regexp            regexp::
run_chunk object            object::
run_chunk string            string::
run_chunk data_view         data_view::
run_chunk functions         functions::
run_chunk frontend          frontend::
run_chunk typed_array       typed_array::
run_chunk array             array:: --skip typed_array::
run_chunk language          language::
run_chunk binary_data       binary_data::

if [ -n "$UNBANKED" ]; then
  echo "RUN INCOMPLETE $(date -u +%Y-%m-%dT%H:%M:%SZ) -- no verdict for:$UNBANKED" >> "$RESULTS"
  echo "rung1c: INCOMPLETE. No verdict for:$UNBANKED" >&2
  echo "rung1c: re-run this script to retry them; they were deliberately not banked." >&2
  exit 1
fi

echo "ALL CHUNKS DONE $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$RESULTS"
echo "rung1c: all chunks done; verdicts in $RESULTS"
