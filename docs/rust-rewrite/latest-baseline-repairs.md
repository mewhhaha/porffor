# Latest observed baseline repairs — 2026-09-10

The comparison starts from a freshly fetched `origin/main` at
`9d435d996c8b9f72947007d81467794f8396b3ab`. The running baseline uses the older
compiler `c5115bf03`; its outcomes alone do not establish failures on main or
on this branch. A separately built main compiler and frozen branch compilers
provide that comparison.

## Frozen observation scope

The completed-node snapshot contains 39,299 of 102,043 exact executions,
across 302 completed matrix nodes: 35,604 Success, 1,836 Bug, 740 Crash and
1,119 NotImplemented. The Test262 tree pin is
`aa55200d1310384c5cf69ea95b2a2ecba457007b`. Active partial nodes are excluded.
The aggregate SHA-256 is
`7571e5c1073c778268b6f4b978fc9de63ef1f2c5f2e49c254ee9ce6fabcdc0a6`.

Of those 3,695 recorded failures, 1,012 belong to the
[earlier repair cohort](observed-later-failure-repairs.md). The additional
2,683 identities are retained in
[`observed-latest-20260910.executions`](../../test262/replays/observed-latest-20260910.executions),
with [snapshot provenance](../../test262/replays/observed-latest-20260910.provenance.json).
These are baseline failures to investigate, not an expected-failure list.
The full additional 2,683-case list was not replayed against main or the final
branch compiler. The family counts below describe the frozen baseline; current
compiler claims are limited to exact comparisons and representative probes.

| Family | Additional baseline failures |
| --- | ---: |
| RegExp and RegExpStringIteratorPrototype | 1,256 |
| Temporal | 1,028 |
| Object | 155 |
| ShadowRealm | 124 |
| Promise | 50 |
| String | 45 |
| Proxy | 13 |
| Number | 4 |
| SuppressedError | 4 |
| Reflect | 2 |
| SharedArrayBuffer | 2 |

## Repair scope

The branch includes the earlier independently compiled prepared-source and
runtime repair batch. This follow-up fixes confirmed semantic defects within
implemented operations:

- Runtime string-to-number conversion recognizes binary and octal prefixes in
  either case, validates digits for their radix and rejects signed radix forms.
  Binary, octal and hexadecimal integers retain an exact significand and sticky
  bit before one binary64 rounding step, fixing the older hexadecimal rounding
  defect as well. `Number()` also accepts heap-backed BigInts through the
  existing exact decimal formatter and correctly rounded conversion; ordinary
  ToNumber still rejects BigInt. Object construction accepts mixed inferred
  argument kinds using the existing runtime boxing path and conservative
  object-result information.
- AggregateError and SuppressedError calls without `new` select the active
  constructor, including foreign constructors saved before global replacement.
  SuppressedError prototype fallback resolves the canonical intrinsic of the
  new target's realm, including bound and proxy new targets.
- Temporal PlainDate and PlainDateTime field readers use the existing
  ToMonthCode operation, requiring a String and validating its syntax before
  reading later fields. Calendar suitability checks retain their specified
  ordering relative to options.

- RegExp construction performs IsRegExp and the active-constructor identity
  check, observes source/flags and new-target prototype access in order, and
  retains matcher slots selected before a getter can recompile the input.
  Constructor and compile share flags validation; malformed group prefixes are
  syntax errors while legal unimplemented assertions remain capability errors.
- RegExp string iterators have a distinct canonical prototype and next function
  in every realm. Array iterator next no longer accepts their internal brand;
  lowering reports the correct iterator shape.
- The Object.prototype.__proto__ getter rejects nullish receivers with a
  TypeError from its defining realm and boxes primitive receivers in that realm.
  Created realms publish their own getter and setter with canonical descriptors.
- Generic property Reference reads dispatch canonical numeric typed-array keys
  through integer-indexed element access. Reflect.get and compound assignments
  therefore read actual Number/BigInt elements, and invalid canonical indexes
  do not consult a prototype property.
- ArrayBuffer backing stores use the same selected memory for allocation and
  byte access, including transfer and ordinary/immutable slicing. Resizable
  transfers reserve their maximum byte length so later growth stays within the
  allocated store. This fixes the flatMap detachment regression found by CI.
- Numeric updates retain distinct old and new payload/tag pairs and use canonical
  BigInt arithmetic. Prefix/postfix results remain correct when increments or
  decrements cross inline/heap representation boundaries, including typed-array
  writes that subsequently wrap the stored integer.

The unchanged host assert.throws operation was moved to its own child module
to keep the existing host-module size limit; its implementation is byte-for-byte
identical. The new intrinsic slots remain checked by the heap layout tests;
unrelated structural tests no longer hard-code the entire realm record size.

## Verified latest scope

The pre-CI-repair runtime compiler SHA-256 is
`b2cc76fcd6b4a098074e37ab4f4296fca26fafdfadb975da076dc434b399f6d2`.
The 182-case focused replay was audited on 2026-09-10 against every native
snapshot, transcript, exact execution identity and frozen input/compiler hash.

| Selected additional baseline cases | Success | NotImplemented | Bug | Crash |
| --- | ---: | ---: | ---: | ---: |
| Fresh origin/main, 134 executions | 2 | 4 | 128 | 0 |
| Pre-CI-repair compiler, same 134 executions | 128 | 0 | 6 | 0 |

This verifies 126 newly repaired main failures: 122 Bug and 4 NotImplemented
outcomes become Success, and both existing main successes are preserved.
All 48 neighboring controls pass, for a total of 176 Success and 6 Bug in the
182-case focused replay. No failure is skipped or relabeled as success.

The three remaining files fail in both sloppy and strict modes:

| Duration.prototype.total probe | Observed result / expected result |
| --- | --- |
| `precision-exact-mathematical-values-5.js` | Microseconds: `8.69228866946552e+21` / `8.692288669465521e+21` |
| `precision-exact-mathematical-values-6.js` | `NaN` / `22` |
| `precision-exact-mathematical-values-7.js` | `2251799813685248.5` / `2251799813685248` |

## Verification after the CI buffer repair

The repaired runtime compiler SHA-256 is
`9a84e3ad40d06cb7cf15ced46b4882e2c883ac5f00bdf1fbe6d1a5c5d3e6eb45`.
Its independently audited 182-case replay reproduces **176 Success and 6 Bug**,
with no outcome changes from the earlier checkpoint: all 126 repaired main
failures, both existing main successes and all 48 neighboring controls pass.
There are no NotImplemented outcomes or crashes in this replay. It peaked at
13.735 GiB without memory throttling or OOM events.

The same production compiler passes 68 native Wasmtime regressions across 15
targets, including all 17 flatMap tests and three buffer allocation regressions,
plus the Temporal CLI regression. The IR's 1,122 unit tests and both Object
constructor integration tests pass; the full backend passes 427 unit tests.
The selected structural suite passes 48 tests, and four separately compiled
buffer structural targets pass another 16. Workspace checking includes every
target. Deep planner regressions retain their original depths and assertions
while running on the compiler's 64 MiB worker-stack convention.

The full fake suite passes **191/191 executions across 190 files**, with zero
NotImplemented, Bug or Crash outcomes. Formatting, module boundaries, identity,
task-plan checks, execution-replay tests and failure-collector tests pass. These
fake-suite results remain separate from the pinned real Test262 observations.
The [final verification record](../../test262/replays/latest-baseline-20260910.verification.json)
retains compiler/source hashes, raw failed attempts, corrected checks and exact
replay comparisons. Only the two reviewed integration-test files changed after
the final production compiler was frozen.

## Earlier cohort preservation

The same pre-CI-repair compiler replayed all 1,012 earlier identities. The raw
run completed with **984 Success, 18 NotImplemented, 0 Bug and 10 Crash**. Each
Crash was a timeout; a fresh two-worker replay of exactly those ten identities,
with the same compiler and timeout settings, completed **10 Success**. Those
separate observations preserve all 994 previously passing identities and all
111 successes from the freshly built main comparator. The original timeout
records remain unchanged; this is not a single-run 994-pass result.

Main records 111 Success, 830 NotImplemented and 71 Bug on that exact cohort.
Across the primary observations and separate rechecks, 812 NotImplemented and
71 Bug outcomes from main have a passing reproduction: **883 repaired main
failures**. The 126 additional focused repairs are disjoint, giving **1,009
unique repaired executions** in this checkpoint's evidence. The audit verifies
3,362 native snapshots across both comparators, the primary run, timeout
rechecks and focused cohorts. The [compact evidence record](../../test262/replays/latest-baseline-checkpoint4-20260910.evidence.json)
retains hashes, exact remaining identities and separate attempt counts.

The primary replay encountered substantial memory-limit throttling before its
soft limit was raised. The two-worker recheck peaked at 9.464 GiB with no memory
throttling or OOM events. Passing repeats support resource sensitivity without
proving the sole cause of the original timeouts.

These 1,012-case results predate the subsequent CI buffer allocation repair.
They do not claim a full replay on the later compiler. The original historical
65 passing identities remain unavailable; preservation is verified against the
fresh main comparator rather than inferred for that historical set.

## Remaining work

This patch does not claim a completed full-suite run or full conformance.
Representative replays distinguish remaining failures from baseline cases that
the earlier repair batch already fixed. In particular, the baseline contains
large missing Temporal Instant/Now operations, PlainDate.toZonedDateTime,
Duration calendar-relative operations, ShadowRealm, and SharedArrayBuffer
realm construction. RegExp still needs Unicode property data coverage,
lookbehind support/correctness and work on large generated programs/timeouts.
The buffer regression also exposed an absent `ArrayBuffer.prototype.immutable`
accessor; it remains a separate capability gap. Immutable backing-store behavior
is verified through rejected DataView writes, independently of that accessor.
Other Object descriptor/enumeration and Proxy/String failures remain to be
repaired in their owning operations.

The focused Duration `total/precision-exact-mathematical-values-{1,2}.js`
failures expose typed-array Reference reads and are repaired by that shared
operation. The `{5,6,7}` probes remain red in both execution modes: independent
calendar-relative calculations and exact arithmetic/rounding remain incorrect.
Fixing Number(heap BigInt) removes one earlier blocker in `{5}` without claiming
the whole test passes.

Some Promise combinator tests intentionally leave rejected promises unhandled
after finishing their assertions. The existing backend turns unhandled
rejections into a failed Script completion. That host-policy/failure-reporting
contract needs a separate resolution; this patch does not suppress those
failures. The earlier 18 prepared-source discovery limitations remain explicit
red outcomes owned by T13.

## Reproduce

Build the branch with `cargo build --release --locked -j2 -p lila-cli`.
Replay an exact list through the native runner with:

```sh
python3 scripts/replay-test262-executions.py \
  test262/replays/observed-latest-20260910.executions \
  --binary target/release/lila \
  --output-dir target/latest-baseline-replay --workers 4
```

The [focused replay list](../../test262/replays/latest-baseline-focused-20260910.executions)
contains 182 identities: 134 cases from the additional baseline failures and
48 neighboring controls. Substitute that path in the command for the bounded
verification scope. It includes the remaining Duration precision probes; it is
not a pass-only selection. The earlier 1,012-case cohort is the disjoint union of
`observed-20260908.executions` and `observed-later-20260908.executions` in the same
directory.

The full additional list includes known unresolved cases and can take hours.
The replay tool freezes the compiler and execution list, retains native runner
snapshots and transcripts, and rejects mismatched resume inputs. Use a fresh
output directory, or add `--resume` to continue the same frozen run.
The canonical README publisher block is refreshed only by the full verified
publisher; scoped replay results do not overwrite it.
