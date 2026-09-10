# Later September baseline failure cohort

This batch starts from `origin/main` at `9d435d996`. The input is 420 failing
executions discovered by the continuing `c5115bf03` Wasm-AOT baseline after the
original 592-execution repair cohort. Strict and sloppy executions are distinct.
The exact input is
[`observed-later-20260908.executions`](../../test262/replays/observed-later-20260908.executions).
Its original classifications were 87 Bug and 333 NotImplemented. This is a
partial failure cohort, not a conformance percentage or expected-failure list.

## Repairs

- Bound functions define their own configurable `length` and `name` properties.
  Length uses numeric truncation with positive zero, large values and Infinity
  preserved. Metadata getters run in specification order and propagate throws;
  the target's actual internal prototype is retained.
- Function source extraction converts parser UTF-16 positions to UTF-8 string
  boundaries. The lexer preserves its line-terminator buffering invariant across
  comments, including CR, LF, CRLF and Unicode separators.
- Ordinary sloppy functions uniformly expose an immutable own `caller` whose
  value is `null`. Strict and non-ordinary functions retain their forbidden
  caller behavior; no strict caller is exposed.
- Iterator consumers call predicates with an explicit undefined receiver before
  the callback's own strict/sloppy this-binding rules apply.
- Created realms publish synchronous DisposableStack intrinsics. Constructor
  fallback uses the new target's canonical realm prototype, while `move` uses
  its defining method's realm.
- CreateListFromArrayLike reads TypedArray indexed storage through the same
  dispatch used by ordinary indexed access, including resized and out-of-bounds
  views. `apply`, `Reflect.apply` and `Reflect.construct` share this path.
- Created-realm arrays publish their `values` method under `Symbol.iterator`
  with the same function identity and specified descriptor attributes, so
  prepared Scripts and callers in other realms can iterate their literals.

Six Iterator test-source replacements were deleted. Their original pinned bodies
now reach the compiler. The path-based resizable-arraybuffer gate was also
deleted: unsupported compiler operations must report their actual diagnostics.
The remaining source substitutions and gates are still recorded by the shortcut
audit; this batch does not establish a shortcut-free harness.

The batch also implements independently compiled finite Function and Script
sources. Function parameter and body grammars are parsed separately, nested
functions have unique identities, and all four Function-family constructors
retain live argument conversion, prototype lookup and realm selection. Finite
literal records and arrays can supply source candidates without replacing the
original callback, property access or constructor call. Candidates authorize
compiled bodies only after their exact runtime source tuple matches.

Direct eval captures the original Reference and callable before evaluating its
arguments and checks the current realm's immutable eval intrinsic. Its named
environment records preserve lexical and variable scope, TDZ and const rules,
with/unscopables lookup, delete/recreate behavior, parameter/body separation,
escaped arrows, this/new.target, private names and super context. Replaced,
foreign, bound and proxied eval calls retain ordinary call semantics. Prepared
Scripts perform declaration validation before mutation and retain the specified
statement completion value.

Global function declarations now share their realm-global property identity;
closures resolve these mutable global bindings through that same property.
Generator and async resumption retain distinct parameter and body records.
AggregateError prototype fallback resolves the new target's canonical realm
intrinsic, including bound and nested Proxy targets. Boxed primitive source
arguments honor their observable conversion hooks.

HasProperty dispatch is emitted once as a shared Wasm helper. Callers retain the
selected Reference and propagate abrupt completion, while the helper receives
the caller's realm explicitly for Proxy errors. Empty function-specialization
passes return before cloning maps when there is no work, reducing repeated
lowering cost for prepared sources.

Named assignments use the existing shared OrdinarySet helper too, avoiding a
full copy of descriptor and exotic-object dispatch at every write. Resolved
object-environment assignments recheck HasProperty on their retained binding
object after right-hand-side effects. This preserves Proxy trap order and thrown
values, and rejects strict writes when the binding has disappeared. Initially
unresolvable assignments retain their original resolution and strictness.

Promise combinators obtain AggregateError prototypes from their canonical realm
intrinsics instead of the uninitialized snapshots captured by early bootstrap
functions. Suspended property assignments consume the ordinary Reference plan
directly, so nullish bases fail at runtime after RHS resumption without coercing
the key. The error-message pool remains sorted and preserves fixed literal
offsets when new messages and function-name prefixes are added.

The complete cohort replay then exposed declaration-completion corruption:
initializing a captured lexical cell overwrote the result tag with its TDZ
marker, and borrowed eval variable initializers were lowered as value-producing
statements. Cell initialization now stores directly into the selected binding,
and declaration evaluation has its own IR statement so initializer effects and
throws are retained without replacing the preceding StatementList value.
Nested functions also retain optional finite constructor and source hints for
Script-global declarations, whose runtime reads go through global properties.
Local shadowing, replacement callees and the actual source tuple still control
execution. This restores prepared parameter SyntaxErrors inside ordinary and
async-generator callbacks.
The native completion controls also exposed eager global-cache reads. Global
function names now resolve through their installed properties, including names
also declared with `var`. The compiler retains only write temporaries for
publishers and removes both prologue reads and post-write refreshes. Empty
Scripts no longer inherit a cached value's tag, and declarations no longer
invoke accessors before a source-level read or perform an extra Get after Set.
The same publisher preserves loop completion values while propagating setter
throws; empty `for-in` and `for-of` bodies therefore retain `undefined`.
Prepared global `var` deletion checks the runtime descriptor rather than
treating declaration metadata as a local binding. Reads after successful deletion throw
ReferenceError, while `typeof` retains its unresolvable-binding behavior.
Global eager compound assignments use the runtime Object Environment Reference
path, preserving Number, String and BigInt result tags and coercion order.
Shared dynamic addition uses the canonical BigInt arithmetic helper for inline
and heap operands, including overflow promotion. Its runtime result tag remains
authoritative; Number/BigInt mixtures throw before publishing an assignment.
Both operands are evaluated before their ordered primitive conversions, and
string concatenation retains its precedence over numeric addition.
Coercive arithmetic uses the same tagged numeric conversion path when its
payload is discarded or consumed. An inferred Number result does not justify
converting a BigInt operand with ToNumber before the other operand's observable
conversion; both ToNumeric operations precede the mixed-type check.
Borrowed eval loop heads publish each iteration value to the caller's variable
environment before evaluating the body.

The obsolete `fnGlobalObject.js` source replacement was removed too: its original
Function construction now reaches the compiler. No parser or interpreter is
bundled into the emitted Wasm. Arbitrary runtime-generated source remains an
explicit unsupported dynamic-code-generation outcome; malformed prepared source
throws the appropriate runtime SyntaxError instead.

## Reproduce

Build the product CLI, then replay every exact execution:

```sh
cargo build --release --locked -j 2 -p lila-cli
cat test262/replays/observed-later-20260908.executions \
  test262/replays/observed-20260908.executions > target/observed-combined.executions
LILA_MODULE_MEMORY_CACHE_ENTRIES=1 python3 scripts/replay-test262-executions.py \
  target/observed-combined.executions \
  --output-dir target/observed-combined-replay --workers 2
```

Use a fresh output directory; the replay tool retains a frozen executable,
execution identities, hashes, native transcripts and reports. Every non-passing
execution remains red. After interruption, resume with the retained inputs:

```sh
LILA_MODULE_MEMORY_CACHE_ENTRIES=1 python3 scripts/replay-test262-executions.py \
  target/observed-combined-replay/executions \
  --binary target/observed-combined-replay/compiler \
  --output-dir target/observed-combined-replay --workers 2 --resume
```

The POSIX replay driver locks its evidence directory, verifies the compiler and
execution-list hashes and suite location, and checks each saved outcome against
its native transcript before reusing it. Completed failures remain red. Reports
are published atomically; unfinished transcripts are retained when a case is
retried. On Linux, long replays can run as a `systemd-run --user` service so an
interactive tool restart does not terminate the worker pool.

To collect additional observations from a later native
matrix checkpoint, use `scripts/collect-new-test262-failures.py` with the
aggregate path, `--exclude` execution list and a fresh `--output-dir`.

## Verification

The final repair checkpoint passed on `2026-09-09` with release compiler SHA-256
`37cb9c33f04c7ab186f93476d6ffef0bca1eccc4f901f57658df493034d6b034`:

| Check | Result |
| --- | ---: |
| Backend planner, runtime tags and temporary storage | 55 tests passed |
| Arithmetic, global module state and realm Array structure | 27 tests passed |
| Declaration, loop and arithmetic behavior through Wasmtime | 27 tests passed |
| Neighboring numeric behavior through Wasmtime | 12 tests passed |
| Full fake Test262 suite | 191 executions passed |

The checkpoint passed `cargo check --release --locked -j2 --workspace
--all-targets`, formatting, module-boundary, task-plan and diff checks.
Commands, exit statuses, durations, compiler identity and the exact fake
execution comparison are retained in
`target/failure-review/coercion-completion-checkpoint/`.

The fake run covers 190 physical files and includes all 187 wasm-safe
executions, with no failures or timeouts. Its fixture pin is
`11d2e5ef5ec4250e88cc9827ef46a3b97fa986a3`, separate from the real Test262 pin.
All 191 execution identities match the earlier full fake checkpoint. These
results do not establish full real-suite or ECMAScript conformance.

The complete combined real replay finished on `2026-09-09` with
**994 Success, 18 NotImplemented, zero Bug and zero Crash**:

| Exact execution cohort | Success | NotImplemented | Bug | Crash |
| --- | ---: | ---: | ---: | ---: |
| Original 592 | 588 | 4 | 0 | 0 |
| Later 420 | 406 | 14 | 0 | 0 |
| **Combined 1,012** | **994** | **18** | **0** | **0** |

The real pin is `aa55200d1310384c5cf69ea95b2a2ecba457007b`. All 1,012 native
snapshots, transcripts, execution identities and frozen hashes reconcile in
`target/failure-review/completion-cohort-evidence.json`. The replay list has
SHA-256 `580a7cae8fd23ca6d71add23a19d6d18d0fe040adedcf3c2a3c678d652e4cf1a`;
its membership is exactly the two tracked cohorts, with the 28 follow-up
failures scheduled first. All 24 previous Bug cases and all four literal
AsyncGeneratorFunction source cases now pass. Every one of the earlier 966
passing execution identities remains a pass, with zero regressions, as recorded
in `target/failure-review/numeric-preflight/completion-transition-report.json`.

The replay used 12 single-job workers, completed in 43m15.121s and peaked at
51,542,921,216 bytes of cgroup memory with zero swap or OOM events. Exit status
1 reflects the 18 retained unsupported outcomes; there were no infrastructure
errors. This is a repair-cohort result, not a full-suite conformance percentage.
Canonical full-suite counts remain owned by the Rust publisher.

The preceding complete combined replay used compiler
`33e618db3c9ec75e83f78aff4ada8b13f577bab3e36ef9f95f71e9013213aafe`
and reported 966 Success, 22 NotImplemented, 24 Bug and no Crash outcomes.
The original cohort contributed 560 Success, eight NotImplemented and 24 Bug;
the later cohort contributed 406 Success and 14 NotImplemented. Its 1,012
native reports and execution identities reconcile in
`target/failure-review/final-cohort-evidence.json`. The 24 declaration bugs
and four literal AsyncGeneratorFunction source cases are the 28 transitions to
Success verified in the final replay above.

Earlier scoped verification remains separately attributed to its compiler.
The `7be962bd6b2c4237a8cc4f2b14ca9d41491ee610589ad2230b693e711f2773c1`
checkpoint passed 196 affected IR tests and 116 neighboring native tests outside
the declaration target; its one declaration BigInt failure was repaired and
rerun above. The final arithmetic-only changes did not change IR lowering.
The earlier `33e618...` checkpoint passed all 184 frontend tests, 86 generator
and Reference IR tests, 74 native tests, ten backend invariant tests, 21 Promise
structure tests and one Promise CLI test. Those are prior scoped results, not
additional tests rerun on the final compiler.

The earlier broad sweep passed all 1,310 IR tests with one preexisting ignored
documentation example. Its frontend, backend and neighboring native failures
were subsequently repaired and rerun; that full broad sweep was not repeated.
The replay and failure-collection tools passed all 26 Python tests. The final
capability-documentation correction also passed 16 affected standalone IR
structure tests and changed no product code. Those 16 checks were rerun after
the final T13 status update; evidence is retained under
`target/failure-review/final-documentation-checkpoint/`.

The original 65-pass checkpoint's exact passing identities were not recoverable
from its removed worktree. Its historical counts remain recorded in the
[original repair notes](observed-failure-repairs.md); the final replay checks all
592 original execution identities again, without claiming a comparison to that
missing per-case evidence.

The intermediate shared-helper compiler has SHA-256
`94974580be361814d10a3c9cf508e231ac8e98bebfe2c7429bbfd3e2c60b48b7`.
The exact oversized direct-eval scope fixture now executes successfully through
Wasmtime. Its main function shrank from 4,110,057 to 2,594,374 bytes and its module
from 17,390,396 to 11,918,194 bytes. The previous artifact was reproduced
byte-for-byte before comparison. Evidence is retained under
`target/failure-review/direct-eval/combined-scope-helper-verification.json`.

An isolated, uncached replay of
`sloppy-script:built-ins/AsyncFunction/is-a-constructor.js` now passes. Identical
test and include bytes produce 87 lowering invocations in both measurements.
Elapsed time fell from 202.07 to 92.03 seconds; IR lowering fell from 194.55 to
83.68 seconds. Native compilation remained approximately 6.4–6.7 seconds. Peak
RSS increased from 2,766,216 to 3,498,756 KiB, so this establishes a time
improvement, not a memory improvement. Commands, hashes, phase timings, sampled
resources and function sizes are retained under
`target/failure-review/async-function-constructor-profile-after/`.

## Remaining source-discovery cases

All 18 remaining executions are typed Unsupported outcomes, owned by
[T13 dynamic source evaluation](../../tasks/13-dynamic-source-evaluation.md).
Each of these nine physical files fails in both sloppy and strict Script mode:

- `annexB/built-ins/RegExp/RegExp-leading-escape-BMP.js`
- `annexB/built-ins/RegExp/RegExp-trailing-escape-BMP.js`
- `built-ins/Function/S15.3.2.1_A2_T4.js`
- `built-ins/Function/S15.3.2.1_A2_T5.js`
- `built-ins/Function/S15.3.2.1_A2_T6.js`
- `built-ins/Function/prototype/apply/S15.3.4.3_A7_T3.js`
- `built-ins/Function/prototype/apply/S15.3.4.3_A7_T4.js`
- `built-ins/Function/prototype/call/S15.3.4.4_A6_T3.js`
- `built-ins/Function/prototype/call/S15.3.4.4_A6_T4.js`

The two RegExp files generate 65,521 eval source strings per file through a
bounded BMP loop. Current direct-eval discovery does not expand the numeric
range and String.fromCharCode computations into prepared source units.

The seven Function files build parameter names through stateful ToString hooks
using `"arg" + (++i)`. Current finite discovery does not derive those source
tuples. The apply/call cases fail during that Function construction, before
apply/call behavior is reached. These are current source-analysis limitations;
the programs are bounded, and this result does not prove they inherently
require a runtime parser or are fundamentally incompatible with AOT.

Every case ran and remains red. No skip, expected-pass substitution or runtime
JavaScript parser/interpreter was added. Exact diagnostics, execution modes
and owner-family evidence are retained in the final cohort and transition
audits linked above.

## Adjacent issue at the earlier checkpoint

At the earlier `37cb9c33...` checkpoint, foreign `AggregateError` calls without
`new` used the entry realm's prototype. This reproducer returned `false`;
`.call` and `.apply` had the same result:

```js
var other = __lilaCreateRealm().global;
Object.getPrototypeOf(other.AggregateError([])) === other.AggregateError.prototype;
```

The owner is `crates/lila-aot-wasm/src/builtins/errors.rs`, specifically
`emit_aggregate_error_new_target_prototype_to_local` and the undefined-NewTarget
branch of `emit_new_target_prototype_to_locals`. That branch must use the active
callee before looking up its prototype. The behavior predates this batch and
was outside its repaired scope. The original 592-execution cohort contains only the two
`AggregateError/newtarget-proto-fallback.js` variants; the later 420-execution
cohort contains no AggregateError cases. The independent reproducer was
rechecked on the final `37cb9c33...` compiler: direct, call and apply all retain
the wrong entry-realm prototype. Its transcript is
`target/failure-review/foreign-aggregate-error-call-final.log`.

The [latest baseline follow-up](latest-baseline-repairs.md) fixes active
constructor identity centrally and adds direct, call and apply regressions for
AggregateError and SuppressedError. See that checkpoint for final validation.
