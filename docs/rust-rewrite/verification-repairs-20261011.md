# Verification repairs — 2026-10-11

The watcher now wakes when the supervised process exits, rather than waiting
for its next log poll. Periodic log-growth checks, stall deadlines, signal exit
statuses and whole-group cleanup remain unchanged. All eight focused controls
pass; the new long-poll control times out against the original wrapper.
Three isolated short-command samples measure a median 0.041 seconds versus
1.033 seconds with the original one-second poll. These are wrapper measurements,
not compiler or execution benchmarks.

Two IR controls retain their original guarantees against the current owners.
The parser-range module rewrite still maps all three failures exhaustively and
preserves source-edit argument order. The `with` variable-initializer control
checks its typed global-fallback Reference, HasBinding selection, ordered single
RHS and empty declaration completion. Existing native controls retain observable
property deletion and unscopables changes during that initializer.

All 1,082 NumberFormat locales and 78 numbering systems reproduce the committed
6,091,829-byte payload exactly. DateTime, active timezone names, DisplayNames,
RelativeTime and Duration reproduction also pass. Collator's genuine sources,
ICU archive and ten exported data files match; its two compiler-emitter owner
hashes are refreshed through the normal generator, followed by the dependent
locale-collations identity. CI now checks Collator and the consumed timezone
native image. Historical timezone Rust rows and their report remain unchanged.
Actual Collator reexport remains a separate Cargo acceptance check.

The interrupted workspace probe retains 3,359 completed passes, two IR failures
and two existing documentation ignores across 168 completed scopes. Frontend
passes all 217 checks and AOT passes all 969. Its active engine scope is incomplete:
317 passing and 25 failing markers are diagnostic only. The test process saw
three CPUs; the engine library derives its first default pool from CPU affinity
and selected two workers before later controls explicitly required one.
The same executable reproduces that conflict, then passes both lifecycle
controls when the normal cloud launcher pins one test CPU. Compiler builds can
retain the inherited auto affinity while each test runs under the serial cloud
policy. Every completed transcript is archived before successor runs.

All sixteen current Intl source/image checks pass. The repaired-source joined
all-feature/all-target type check passes in 221 watched seconds. The complete
tooling suite passes 438 tests with zero failures, errors or skips in 135.730
test seconds / 137 watched seconds. Both focused IR targets pass all six tests
on one test CPU. A preceding focused run exposed two further stale formatting
expectations; their correction retains the complete error routes and argument
order. Exact commands and log hashes are in the [receipt](verification-repairs-20261011.json).

All eleven formatting/repository guards also pass, including the shortcut
scanner controls, task ownership, host ABI and product dependency graph.

Complete workspace/default/ignored, differential and current pinned Test262
acceptance remain required. Task states and canonical publisher counts are unchanged.
