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
#    Deleting it re-runs hours of already-banked work. A TARGETED invalidation
#    (removing the line for a chunk whose subsystem a lane has since rewritten)
#    is legitimate and is how a stale verdict is retired; deleting the whole
#    file is not. See "the counts sidecar" below for the half of that judgement
#    the script can now make on its own.
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

# THE COUNTS SIDECAR -- `<chunk> <#[test] count of its module when it banked>`.
#
# The done-file records THAT a chunk was measured; it cannot record WHAT was
# measured, and that gap has a live cost. A chunk banked at 16 tests stays
# banked after a lane adds a 17th to the same module: the resume skips it, the
# run prints ALL CHUNKS DONE, and the new test has never executed. That is the
# same class of defect as banking a chunk that selected zero tests (property 4),
# arriving from the other direction, and it is not hypothetical -- batch 5
# banked `date` at 16 tests and batch 6 added a 17th in the same file.
#
# So the skip predicate is "banked AND its module still declares the same number
# of tests", and a chunk whose module has grown or shrunk re-runs. The count is
# the same exact-line `awk` form the hygiene scanner uses, over the one file the
# chunk selects. It is a FINGERPRINT, not an assertion about the suite: it
# deliberately counts `#[cfg]`-gated tests too (only `frontend.rs` has any), so
# it moves whenever the module's source does.
#
# What it does NOT see, and nobody should think it does: a chunk goes stale when
# the COMPILER changes as well as when its tests do, and no count can detect
# that. Retiring those verdicts is still a human judgement -- delete the chunk's
# line from the done-file. This mechanism removes one whole class of that
# judgement, not the class.
#
# The two unknown cases are deliberately resolved in OPPOSITE directions, and
# the difference is which way the unknown can lie:
#
#   * AN UNREADABLE MODULE (count 0) keeps its banked verdict and prints a
#     warning. A read error is a bug in this script, and a bug here must not
#     cost hours of re-run.
#   * A CHUNK WITH NO RECORDED COUNT RE-RUNS. It used to be grandfathered --
#     recorded at whatever the tree declares today and skipped -- which blesses
#     unmeasured work as measured, the exact vacuous pass this sidecar exists to
#     close. `touch "$COUNTS"` below recreates the file empty, so losing only
#     the sidecar while the done-file survived took EVERY banked chunk down that
#     path. When we do not know what a verdict measured, we measure it again.
COUNTS=${RUNG1C_COUNTS:-target/watched/rung1c-done-counts}

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
touch "$COUNTS"

# `#[test]` attributes declared by one area module, counted with the EXACT-line
# form (a substring `grep '#\[test\]'` over-counts; see the header). Prints 0
# rather than failing when the file cannot be read, and callers treat 0 as
# "unknown" so a read error never invalidates a banked chunk.
module_test_count() {
  mtc_count=$(awk '/^[[:space:]]*#\[test\][[:space:]]*$/{n++} END{print n+0}' \
    "crates/porffor-cli/tests/cli/$1.rs" 2>/dev/null)
  echo "${mtc_count:-0}"
}

# The count banked alongside a chunk, or empty when it predates the sidecar.
banked_test_count() {
  awk -v key="$1" '$1 == key { value = $2 } END { print value }' "$COUNTS" 2>/dev/null
}

# Replace (never duplicate) a chunk's row in the sidecar.
record_test_count() {
  grep -v "^$1 " "$COUNTS" > "$COUNTS.tmp" 2>/dev/null
  mv "$COUNTS.tmp" "$COUNTS"
  echo "$1 $2" >> "$COUNTS"
}

# Chunks that ran but produced no bankable verdict this pass. A run with any of
# these is NOT a complete rung 1c, so it must not print ALL CHUNKS DONE.
UNBANKED=""

# THE STALL BUDGET IS PER CHUNK, because one chunk does not fit the default.
#
# `run-watched.sh` judges liveness ONLY by growth of the chunk's log file. Its
# own heartbeat `printf` goes to run-watched's stdout and never into that log
# (`run-watched.sh:86`), and `frontend_test262_subset`'s child has its stdout
# swallowed by `.output()`, so between libtest's single "has been running for
# over 60 seconds" warning and the final `test result:` line NOTHING reaches the
# log. The budget for a one-test chunk is therefore exactly `60 + stall`.
#
# MEASURED, and this is why 900 is wrong for that chunk. b5's last `frontend`
# attempt did not die of memory at all -- it died on this guard:
# `target/watched/b5r4-rung1c-driver.log` shows the test still unfinished when
# run-watched killed the run ("no output for 900s after 1335s"), having started
# at ~375 s. That is >= 960 s of runtime with the DEFAULT 4 workers on an
# otherwise idle box, against a 60 + 900 = 960 s ceiling. `--threads 2` in
# `tests/cli/frontend_test262_subset.rs` serialises the compiles further, so it
# can only move that number the wrong way.
#
# Isolation makes the chunk schedulable; it does not make it fit. A one-test
# chunk is also the case where the guard buys least: there is nothing else the
# silence could be hiding, and the chunk's failure mode on a real hang is an
# hour of wall clock rather than a corrupted verdict.
#
# FIRST ACTION FOR WHOEVER RUNS THIS: time that chunk alone, with the sweep
# down, and set the number from the measurement rather than from this comment.
chunk_stall() {
  case "$1" in
    frontend_test262_subset) echo "${RUNG1C_STALL_FRONTEND_SUBSET:-3600}" ;;
    *) echo "${RUNG1C_STALL:-900}" ;;
  esac
}

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
  tests_now=$(module_test_count "$name")
  # Set only on the stale path, so the done-file is rewritten in place of being
  # appended to exactly when a duplicate line would otherwise appear.
  rebank=0
  if grep -qx "$name" "$DONE"; then
    tests_banked=$(banked_test_count "$name")
    if [ "$tests_now" = "0" ]; then
      echo "rung1c: skip $name (already banked in $DONE; WARNING -- could not count tests in crates/porffor-cli/tests/cli/$name.rs, so its verdict could not be re-validated)" >&2
      return 0
    fi
    if [ -z "$tests_banked" ]; then
      # RE-RUN, do not grandfather. This branch used to print UNVERIFIED,
      # record the CURRENT count and return 0 -- i.e. bless whatever the tree
      # declares today as though it had been measured, which is precisely the
      # vacuous pass the sidecar exists to close, arriving from the one
      # direction the sidecar cannot see. It is not a hypothetical path either:
      # `touch "$COUNTS"` above recreates the file empty, so losing ONLY the
      # sidecar while the done-file survives took every banked chunk through
      # here and re-blessed all of them at their current counts.
      #
      # The counts file's own header states the doctrine this now follows:
      # seed from the head a chunk was measured at, NEVER from the current
      # tree. When we do not know what a banked verdict measured, the honest
      # answer is to measure it again.
      echo "rung1c: RE-RUN $name -- banked in $DONE with no recorded test count, so what its verdict measured is unknown. Re-measuring rather than blessing the current $tests_now." >&2
      echo "$name UNVERIFIED-REBANK: banked with no recorded test count; re-running instead of grandfathering." >> "$RESULTS"
      rebank=1
    elif [ "$tests_banked" = "$tests_now" ]; then
      echo "rung1c: skip $name (already banked in $DONE, still $tests_now test(s))"
      return 0
    else
      echo "rung1c: RE-RUN $name -- banked at $tests_banked test(s), tests/cli/$name.rs now declares $tests_now. The banked verdict never measured the difference." >&2
      echo "$name STALE: banked at $tests_banked test(s), module now declares $tests_now; re-running." >> "$RESULTS"
      rebank=1
    fi
  fi
  log=target/watched/rung1c-$name.log
  echo "=== $name START $(date -u +%Y-%m-%dT%H:%M:%SZ) ===" >> "$RESULTS"
  # The three cache limits are CACHE HYGIENE and nothing more. They bound bytes
  # ON DISK -- `porffor-engine/src/cache.rs` implements all three tiers over
  # `fs::read`/`fs::write` -- so they do not bound any process's RSS and they did
  # not fix, and could not have fixed, the `frontend` chunk's OOM kill (that was
  # one child running 4 concurrent cold Wasm-AOT compiles in one process; the
  # fix is `--threads 2` on that child, in
  # tests/cli/frontend_test262_subset.rs).
  #
  # ONE OOM kill, not three -- see the module header of
  # `tests/cli/frontend_test262_subset.rs` for the corrected record of the three
  # deaths, and `chunk_stall` above for the bound the third one actually hit.
  #
  # What they DO buy: the CLI suite compiles hundreds of distinct sources, and
  # the module and program tiers are keyed by source text, so they are pure
  # write-and-prune churn here while the function-stencil tier is keyed per
  # function and converges.
  #
  # THE VALUES ARE THIS BOX'S, NOT THE DEV MACHINE'S, and that is deliberate.
  # Batch 6 first wrote 8 GiB / 512 MiB / 512 MiB here, copying the shape of the
  # sweep invocation in `docs/rust-rewrite/batch-workflow.md` (32 GiB function
  # tier). Two measurements say that is wrong on this container:
  #
  #   * THE CACHE IS ONE SHARED DIRECTORY. `porffor_cache_root()` is
  #     `$PORFFOR_CACHE_DIR` or `~/.cache/porffor`, with no per-process keying,
  #     so the Test262 sweep and every CLI-test child manage the SAME bytes. The
  #     supervisor actually running on this box
  #     (`target/test262-scratch/sweep-supervisor.sh`, and confirmed in
  #     `/proc/<pid>/environ` of the live process) uses
  #     1 GiB / 64 MiB / 64 MiB. A chunk that let the shared directory grow to
  #     8 GiB would hand the next sweep process 7+ GiB to prune back to 70 % of
  #     ITS 1 GiB budget: two processes with different budgets over one
  #     directory undo each other's sizing.
  #   * THE DISK IS A FIXED PER-SESSION ALLOWANCE. Measured 2026-08-10: 19 GiB
  #     available, `target/debug` alone 7.3 GiB, `~/.cache/porffor` 948 MiB (at
  #     the sweep's cap). An 8 GiB tier is a third of the remaining allowance
  #     spent on a cache, and "no space left on device" mid-chunk looks like
  #     anything but a disk problem.
  #
  # So: match the sweep exactly. The function tier equals the `cache.rs` default
  # (1 GiB), and the only real movement is the module and program tiers DOWN to
  # 64 MiB from the 512 MiB default -- which is the actual hygiene win here,
  # because those two are keyed by source text and never serve a hit across
  # distinct fixtures. Raise the function tier only together with the sweep
  # supervisor's, and only after checking `df` on this session.
  #
  # This is the ONLY correct place for them. Every limit is memoised in a
  # `OnceLock` on first use, so `std::env::set_var` inside a test is both useless
  # (an earlier test in the same process may already have resolved it) and
  # unsound under libtest's threads. Set here, it is inherited by `cargo test`,
  # by the test binary, and by every `porf` child it spawns -- which covers both
  # the in-process and the guarded execution paths at once.
  #
  # `PORFFOR_CPU_PERCENT=100` is separately load-bearing: `run-watched.sh` routes
  # through `capped.sh`, which silently pins to half the CPUs.
  stall=$(chunk_stall "$name")
  PORFFOR_CPU_PERCENT=100 \
  PORFFOR_FUNCTION_CACHE_LIMIT_BYTES=1073741824 \
  PORFFOR_MODULE_CACHE_LIMIT_BYTES=67108864 \
  PORFFOR_PROGRAM_CACHE_LIMIT_BYTES=67108864 \
    ./scripts/run-watched.sh --label "rung1c-$name" --stall "$stall" -- \
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
  if [ "$rebank" = "1" ]; then
    grep -vx "$name" "$DONE" > "$DONE.tmp" 2>/dev/null
    mv "$DONE.tmp" "$DONE"
  fi
  echo "$name" >> "$DONE"
  # Bank the fingerprint with the verdict, never separately: a recorded count
  # for a chunk that did not produce a verdict would silence the staleness
  # check for a chunk that measured nothing.
  record_test_count "$name" "$tests_now"
  return 0
}


# Order is cheapest-and-most-diagnostic first, so a short container window still
# banks something useful. `known_failures` leads deliberately: it holds
# `ledger_is_well_formed`, and a malformed ledger makes every other chunk take
# the guarded-subprocess path and look merely slow.
run_chunk known_failures    known_failures::
# SECOND, and the position is the whole point of isolating it.
#
# It is one test, and the single most memory-expensive one in the suite
# (8.4-8.7 GiB in one child, measured with `ps -o rss` on an idle box). Alone in
# its own chunk it schedules nothing beside it whatever `--test-threads` says.
#
# It sat 13th of 18 when the chunk was first added, behind ~26 min of work this
# batch's own done-file edits reinstate (`date` re-runs on the counts guard,
# `iterator`/`iterator_helpers`/`known_failures` were invalidated), plus a full
# rebuild of the test binary. Against an hourly container restart that is most
# of the window spent before the chokepoint even starts -- which is the failure
# the isolation was supposed to end, not reproduce. Chunks partition the suite,
# so this has no ordering dependency on anything, and the hygiene test does not
# constrain order.
#
# `known_failures` stays ahead of it: it is 0.02 s and it holds
# `ledger_is_well_formed`, without which every other chunk takes the guarded
# subprocess path and merely looks slow.
#
# The overlap rule is satisfied without a `--skip`: `frontend_test262_subset::`
# does not end with `frontend::` (the next character after `frontend` is `_`,
# not `:`) and libtest's substring filter `frontend::` therefore does not select
# it either. Do not rename either module so that one stem becomes a `::`-suffix
# of the other. See `chunk_stall` above for its separate stall budget.
run_chunk frontend_test262_subset frontend_test262_subset::
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
