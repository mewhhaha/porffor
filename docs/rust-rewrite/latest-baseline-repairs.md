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
- Numeric updates retain distinct old and new payload/tag pairs and use canonical
  BigInt arithmetic. Prefix/postfix results remain correct when increments or
  decrements cross inline/heap representation boundaries, including typed-array
  writes that subsequently wrap the stored integer.

The unchanged host assert.throws operation was moved to its own child module
to keep the existing host-module size limit; its implementation is byte-for-byte
identical. The new intrinsic slots remain checked by the heap layout tests;
unrelated structural tests no longer hard-code the entire realm record size.

## Verified latest scope

The final runtime compiler SHA-256 is
`b2cc76fcd6b4a098074e37ab4f4296fca26fafdfadb975da076dc434b399f6d2`.
The 182-case focused replay was audited on 2026-09-10 against every native
snapshot, transcript, exact execution identity and frozen input/compiler hash.

| Selected additional baseline cases | Success | NotImplemented | Bug | Crash |
| --- | ---: | ---: | ---: | ---: |
| Fresh origin/main, 134 executions | 2 | 4 | 128 | 0 |
| Final compiler, same 134 executions | 128 | 0 | 6 | 0 |

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

## Remaining work

This patch does not claim a completed full-suite run or full conformance.
Representative replays distinguish remaining failures from baseline cases that
the earlier repair batch already fixed. In particular, the baseline contains
large missing Temporal Instant/Now operations, PlainDate.toZonedDateTime,
Duration calendar-relative operations, ShadowRealm, and SharedArrayBuffer
realm construction. RegExp still needs Unicode property data coverage,
lookbehind support/correctness and work on large generated programs/timeouts.
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
