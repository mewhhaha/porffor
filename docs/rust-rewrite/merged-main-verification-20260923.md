# Merged-main verification after PR #52 — 2026-09-23

PR #52 was merged as `63b33c705` while its `batch23` checkpoint had run only
33 of 221 focused groups and none of the 1,428 prepared pinned executions (see
[the completed-baseline follow-up](completed-baseline-follow-up.md)). Its
immutable run receipts were written in a worktree that no longer exists, so
this follow-up re-established verification on the merged tree instead of
resuming it.

## CI on the merge commit

- `CI` and `Arguments iteration` failed on `check-module-boundaries.sh`: the
  exhaustive builtin call-result table grew by one legitimate row
  (`Temporal.Instant.prototype.toLocaleString`) against a zero-margin budget.
  Every later step of those jobs (formatting, `cargo xc`, focused tests) never
  ran. The budget is re-measured at 2,273 lines.
- `Observed Test262 failure regressions` was cancelled at its 90-minute limit on
  every PR #52 run and on `main`; its last twelve steps had never executed. The
  same 24 steps (190 test selectors, unchanged) now run as seven parallel jobs.

## Full native release suite

Every workspace test executable was run in release mode, which CI does not do
(it runs curated subsets). On the merged tree:

| Area | Failing | Cause |
| --- | --- | --- |
| Source-structure tests (`lila-ir`, `lila-aot-wasm`, `lila-test262`) | 88 tests in 58 binaries | Census counts, markers and exact records not updated when their subjects moved |
| `lila-engine` library tests | 21 of 765 | 17 already failed at `2abe45211` (before PR #52) |
| `lila-cli` `cli` suite | 23 of 796 | 21 already failed at `2abe45211` |
| `lila-engine` integration binaries (156) | 0 | — |

Each structure failure was traced to the commit that moved its subject and
updated to the same strength, except three that were real invariant breaks and
were fixed in source: a `matches!` Boolean projection in
`Object.keys/values/entries`, an `unreachable!` behind a re-matched IR
discriminant in prepared destructuring writes, and a `_ => panic!` in module
instantiation.

Test expectations that contradicted the specification were corrected, each
citing its section: synchronous Module entries complete with `undefined`;
`matchAll` returns a RegExp String Iterator; `%GeneratorFunction%.[[Prototype]]`
is `%Function%`; cross-realm `eval` of a prepared source runs in the callee
Realm; revoked-Proxy errors come from the caller's Realm; Date `toLocale*String`
follow ECMA-402; and several "still unsupported" expectations now assert the
specified results of features that landed.

## Compiler fixes

| Behaviour | Before |
| --- | --- |
| `delete x` of an unresolved global identifier | Constant `true`; the property survived |
| Throws from `instanceof` (`@@hasInstance`, Proxy `prototype` trap, non-object `prototype`) and JSON reviver calls | Skipped the enclosing `catch`/`finally` |
| Generator and async-generator instance `[[Prototype]]` | Taken from a creation-time header copy and the entry Realm, not `Get(F, "prototype")` after parameter initialization |
| TypedArray-owned `ArrayBuffer` members after detach | `undefined` (installer not rooted) |
| `Promise.try`, directly called `then`/`finally` TypeErrors | Wasm trap |
| Property descriptor objects | Non-enumerable fields (`Object.keys` empty, `JSON.stringify` `{}`) |
| `Object.getOwnPropertyDescriptor(proxy, k)` | Returned the trap's object; most 10.5.5 checks missing |
| Built-in method resolution after user writes to built-in prototypes | Overwritten `Error.prototype.toString`, `Function.prototype.apply` and name-table methods ignored or mistyped |
| Iterator and async-generator result allocation in `main` | Read a lexical environment as a function Realm slot; out-of-bounds traps |
| Annex B block function copies | Overwrote a same-named catch parameter |
| Builtin constructors with self-allocating bodies | Extra observable `Get(newTarget, "prototype")` |
| `%Iterator.prototype%` `constructor`/`@@toStringTag` setters | SetterThatIgnoresPrototypeProperties missing |
| String concatenation across surrogate halves | Non-canonical payloads (`"\uD83D" + "\uDCA9" !== "💩"`) |
| `lila::main` size | Native error objects and the bootstrap-only `%Array.prototype%` append arm inlined at every site (one fixture exceeded Wasmtime's function size limit at 5.5 MB) |

Built-in method resolution now goes through one proof type
(`lowering/intrinsic_method.rs`). Methods whose prototype cannot be proven
untouched lower to an ordinary property read and call. Across the CLI fixture
corpus this reduced static `CallMethod` sites from 601 to 324 and grew total Wasm
by 0.009%.

## Pinned Test262 replay (partial, older compiler)

The 14,402 failing executions of the completed `c5115bf03` baseline were
replayed against `814e7db77`, an intermediate commit on this branch without the
later fixes. 12,018 executions completed before the machine ran out of memory:
6,013 pass (3,109 former Bug, 2,164 former NotImplemented, 740 former Crash),
5,642 are Bug, 359 NotImplemented and 4 Crash. No execution moved to Crash;
154 moved from NotImplemented to Bug (now compiled, still failing) and 4 from
Crash to Bug. These are partial, intermediate results, not a full-suite status;
published counts are unchanged.

The remaining failures cluster in Temporal (Duration, Instant, Now,
ZonedDateTime), RegExp property escapes and `v`-flag sets, and ShadowRealm.

## Resource limits

`language/identifiers/start-unicode-*.js` (75–125 KB of identifier
declarations) drive a single compile above 11 GB resident. Several running
together exhausted a 93 GB machine and the kernel killed the terminal scope. Long
test and Test262 runs now execute in a `systemd-run --user` slice capped at
40 GB, with each executable in its own 12 GB scope, so an over-limit case is
killed alone. Compile memory for these cases needs an owner.

## Known gaps found during this verification

- Proxy constructor has a `prototype` property and does not throw without `new`
  (`proxy-no-prototype.js`, `proxy-undefined-newtarget.js`).
- `Object.prototype` methods, Symbol-keyed methods and constructor statics
  (`Object.is`, `Math.pow`, …) are still resolved without a live-prototype
  proof; Map/Set/Date/Promise/RegExp-literal instances still take a fresh-realm
  prototype snapshot.
- The lowering's `array_prototype_mutated` flag is never cleared, so its
  Array-receiver specializations are unreachable.
- A repeated `str[Symbol.iterator]()` on a `String(...)` value after an
  intervening user call lowers as unsupported.
- AsyncGeneratorYield creates its iterator result in the previous context's
  Realm; the selector uses the running Realm.
- `Reflect.construct(Number, [obj], nt)` reads `prototype` before ToNumeric;
  Temporal constructors fall back to the current Realm, not the new-target
  function's Realm.
- The function-header `prototype` copy is not updated by ordinary `[[Set]]`.
- Non-Proxy `Object.getOwnPropertyDescriptor` results use the main Realm's
  `%Object.prototype%`.
