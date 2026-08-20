//! The one `cli` test that has to run by itself.
//!
//! This module holds exactly one test, moved verbatim out of [`crate::frontend`]
//! (`frontend.rs:1345-1367` before the move). It is a module rather than a
//! function in `frontend.rs` for a scheduling reason, not a taxonomy one.
//!
//! # Why a whole module for one test
//!
//! `scripts/rung1c-chunks.sh` runs rung 1c as per-module chunks because the
//! whole suite outlives a container window. Batch 5 sampled `ps -o rss` on this
//! test's child every 10 s on an otherwise **idle** 4-CPU / 15 GiB box and
//! measured a single process — `lila test262 run language/wasm/pass
//! --execution-backend wasm` — holding **8.4-8.7 GiB RSS**, flat, for minutes at
//! ~230 % CPU. That is ~57 % of the box in one test, with `MemAvailable` down to
//! 1.6 GiB. Under `--test-threads=3` the other two libtest workers run their own
//! cold Wasm-AOT compiles beside it, and the `frontend` chunk never produced a
//! verdict across three container windows — which made it the resume's
//! chokepoint, since it sat 13th of 17 and every retry paid ~1 h of
//! already-banked work to reach it again.
//!
//! ## The three deaths were three DIFFERENT deaths
//!
//! This header, `frontend.rs`'s tombstone and the runner script all used to say
//! "SIGKILLed by the OOM killer in three consecutive container windows". Only
//! one of the three was an OOM kill, and the most recent was not. The tree's own
//! results file (`target/lane-notes/rung1c-chunks.md`) records them distinctly:
//!
//! | start (UTC) | outcome | what it was |
//! |---|---|---|
//! | 11:20:51 | `EXIT=101` | the OOM SIGKILL |
//! | 13:00:41 | no result line at all | container restart mid-chunk |
//! | 13:53:31 | `EXIT=124` | the **stall guard** |
//!
//! That matters because the false cause picked the fix. A chunk here must
//! satisfy **two independent bounds**, and only one of them is about memory:
//!
//! 1. peak RSS must fit the box — addressed by `--threads 2` below, and by
//!    isolation, which stops two other libtest workers compiling beside it;
//! 2. the test's wall clock must fit the chunk's stall budget — addressed by
//!    `chunk_stall` in `scripts/rung1c-chunks.sh`, **not** by anything in this
//!    file. `run-watched.sh` judges liveness only by growth of the chunk log,
//!    this test's child output is swallowed by `.output()`, and libtest emits
//!    its "running for over 60 seconds" warning once — so the budget is
//!    `60 + stall`, and the 13:53 run shows the test still unfinished after
//!    >= 960 s against a 960 s ceiling. `--threads 2` moves that number the
//!    wrong way, which is exactly why the stall budget had to move with it.
//!
//! Giving it its own module gives it its own chunk, and a chunk of one test
//! schedules nothing beside it whatever `--test-threads` says. That is the whole
//! mechanism. Note the alternative that does *not* work: `-- --exact <name>` is
//! rejected by `known_failures::rung_1c_chunks_cover_every_cli_area_module`
//! three separate ways (the chunk's filter argument must equal
//! `format!("{name}::")`; only `--skip <module>::` pairs may follow it; and the
//! chunk-key set must equal the module-stem set in both directions), and
//! libtest applies `--exact` to `--skip` as well as to the filter, so
//! `--exact frontend:: --skip <one test>` cannot express "frontend minus one
//! test" either. Lowering `--test-threads` is banned outright: libtest names
//! each worker thread after the test it runs and
//! `known_failures::execution_path` routes on that name, so under
//! `--test-threads=1` every test runs on the thread named `main` and the whole
//! suite falls back to spawning a cold `lila` child per test.
//!
//! **Naming constraint for anyone adding a sibling module.** The overlap rule
//! (`known_failures.rs:1554-1559`) is
//! `format!("{other}::").ends_with(&format!("{chunk}::"))`. `frontend_test262_subset::`
//! does not end with `frontend::` — the character after `frontend` is `_`, not
//! `:` — and `frontend::` does not end with `frontend_test262_subset::`, so
//! neither chunk needs a `--skip` and `array`/`typed_array` remains the only
//! overlapping pair in the suite. A module named `subset_frontend` would *not*
//! be safe.
//!
//! # Why the child now gets `--threads 2`
//!
//! Isolation makes the chunk bankable; it does not make the 8.7 GiB go away, and
//! rung 1c still cannot run beside the Test262 sweep while this test is in
//! flight. The memory itself was traced to the child's own defaults, and the
//! trace contradicts the standing guess that this is a compile-cache sizing
//! problem:
//!
//! - `SuiteConfig::default().worker_count` is
//!   `available_parallelism().min(4)` (`lila-test262/src/lib.rs:233`) = **4**
//!   here, and the CLI passes no `--threads`.
//! - Those workers are **scoped threads in one process**, not child processes:
//!   `execute_cases` uses `thread::scope` + `spawn_scoped` per slot with 64 MiB
//!   stacks. The per-case child runner is off by default (`case_runner_bin` is
//!   `None`; only `LILA_TEST262_FORCE_CASE_RUNNER=1` turns it on).
//! - `--jobs` (Cranelift threads) defaults to half the logical CPUs = 2.
//!
//! So the 8.7 GiB is 187 cases run as **4 concurrent cold Wasm-AOT compiles in
//! one process** — which is exactly the measured shape: one process, flat RSS
//! (a working set, not a leak), ~230 % CPU. `--threads 2` halves the concurrent
//! compile count, and it is the only lever here that touches the measured cause.
//! `LILA_{FUNCTION,MODULE,PROGRAM}_CACHE_LIMIT_BYTES` would **not** have
//! prevented the OOM kill: all three tiers are on-disk caches
//! (`lila-engine/src/cache.rs` — `FunctionCache` implements wasmtime's
//! `CacheStore` over `fs::read`/`fs::write`, and the module and program tiers are
//! directories), so those limits bound bytes on disk, not process RSS.
//!
//! Expect the wall clock to rise roughly in proportion as the compiles
//! serialise; batch 5 measured the whole 46-test `frontend` chunk at ~330 s with
//! this test the only one over 60 s. If a future measurement shows the peak
//! still does not fit beside whatever else must run, the next step is
//! `--threads 1` (and then `--jobs 1`), not raising the limits above.
//!
//! # Known gap, deliberately not changed by the move
//!
//! This test calls `ProcessCommand` (`std::process::Command`, aliased in
//! `main.rs`) directly instead of the crate's `Command` wrapper, so it is on
//! **neither** of `main.rs`'s two bounded paths: `HANG_TIMEOUT` is applied by
//! `Command::output`, and this call site bypasses it. Nine call sites in this
//! target do that (`frontend.rs` 6, `language.rs` 2, and `main.rs`'s own guarded
//! path, which is the legitimate one), so eight tests in the suite are unbounded
//! by wall clock — and the most memory-expensive test in rung 1c is one of them.
//! Converting them is a separate change with its own risk (the wrapper's
//! in-process path would run the CLI inside the test process, which for *this*
//! test would put the 8.7 GiB inside the libtest binary rather than in a child
//! that exits), so it is recorded rather than done here.

use crate::*;

#[test]
fn test262_wasm_backend_runs_supported_fixture_subset() {
    let output = ProcessCommand::new(env!("CARGO_BIN_EXE_lila"))
        .arg("test262")
        .arg("run")
        .arg("language/wasm/pass")
        .arg("--suite-root")
        .arg(suite_root())
        .arg("--snapshot-dir")
        .arg(snapshot_dir())
        .arg("--snapshot-name")
        .arg("cli-wasm-fixture")
        .arg("--execution-backend")
        .arg("wasm")
        // Bounds the child's peak RSS; see this module's header. Without it the
        // child runs 4 concurrent cold Wasm-AOT compiles in one process and
        // holds 8.4-8.7 GiB, which is what the OOM killer SIGKILLed the
        // `frontend` chunk for at 11:20 (the other two deaths were a container
        // restart and the stall guard; only this one is a memory problem).
        // This changes only how many cases are compiled at once, never which
        // cases run or what is reported, so the three assertions below are
        // unaffected by it. It also serialises the compiles, so it makes the
        // wall clock worse — that bound is the chunk's stall budget, not this
        // argument.
        .arg("--threads")
        .arg("2")
        .output()
        .expect("test262 wasm run should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("execution_backend: wasm-aot"));
    assert!(stdout.contains("total: 187"));
    assert!(stdout.contains("passed: 187"));
}
