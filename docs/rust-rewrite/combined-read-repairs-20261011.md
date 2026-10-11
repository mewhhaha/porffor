# Complete read dispatch and bounded libtest scratch — 2026-10-11

The source-bound complete-inventory attempt on `c51cd5cd5` admits 809 native
controls: 801 pass, eight fail and none are ignored. The AOT library has 330
passes and one body-size failure; the original-budget worker target has sixteen
passes; the generated campaign has eight passes and seven five-second AOT
timeouts; the Test262 library has 447 passes. The ordinary-generator campaign
now compiles and matches its actual paired observation. The separate new IR and
engine regressions have not yet been admitted. The corpus transcript reports
four native passes, but the paused driver did not collect its watcher status,
so its target remains interrupted. Another 243 targets are unstarted. The
[compact receipt](combined-read-repairs-20261011.json) preserves this boundary.

Nine additional With reads add 22,401 bytes (5,825 to 28,226), exceeding the
9,000-byte control. Each read still marshals a twelve-word ResolveBinding result
back into P, roots its held Reference, and calls GetValue separately. Four
closed complete-read helpers now keep that Reference inside R. They compose
the existing resolve and get owners, preserve caller execution Realm and lexical
environment, skip Get after abrupt search and retire the Reference through one
exit. `typeof` only suppresses an unresolvable name; TDZ and other abrupt
completions remain abrupt. P supplies operands once and receives one complete
Completion. Assignments, calls, updates and compound operations retain their
separate held-reference route across intervening source effects. The body
control retains its 9,000-byte limit and verifies one actual shared read call
per observable read.

The first completed-unit cache monitor never retired any cache: rustc's
incremental argument names a shared root, rather than the opaque child assumed
by the monitor. A guessed prefix deletion during Cargo would not prove safety.
The next driver removes that monitor. An opt-in compiler wrapper omits
incremental storage only for `--test` units; normal library builds retain their
reusable caches. Compiler options, libtest inventories, optimization profiles,
actual compiler exit status and semantic execution remain intact. Four tooling
controls cover exact normal arguments, both incremental option forms, literal
argument transport and a real failing compiler subprocess. Inactive caches can
still be retired between Cargo producers while holding Cargo's lock.

The foundational backend and full all-target/all-feature workspace type checks,
all 469 tooling controls, eleven repository guards and eight regenerated Intl
reproduction checks pass. Subsequent native acceptance remains pending.
Default/release suites, full pinned acceptance and ownership-reference migration
remain open; the handoff and task directory remain until those gates close.
