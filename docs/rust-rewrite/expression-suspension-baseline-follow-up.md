# Expression failures after merged PR #48

This batch addresses the 22 execution failures reproduced on freshly fetched
main `260a173abe26181a8eb3557cd8b96c9ccb53bf21`: suspended computed class
names, generator calls with yielding arguments, finite eval sources passed
through spreads, and a dynamic `eval` binding that prevented proper tail calls.
All execution uses the Rust JavaScript-to-Wasm compiler and Wasmtime.

## Baseline and exact scope

The frozen September 12 observation of the continuing `c5115bf03` baseline
contains 63,569 of 102,043 executions across 475 of 744 nodes. It records
52,812 Success, 8,126 Bug, 746 Crash, and 1,885 NotImplemented. This is an
old-compiler observation, separate from current main and published full-suite
status.

Since the earlier detailed observation at 60,183 executions, the baseline
added 3,386 executions: 3,335 Success and 51 failures. A freshly built main
compiler replayed those exact 51 failures and produced 29 Success,
21 NotImplemented, and one Crash, with no timeouts. The
[51-execution replay list](../../test262/replays/expression-suspension-20260912.executions)
keeps the 29 current-main passes as controls; only the
[22 remaining failures](../../test262/replays/expression-suspension-20260912.observed.executions)
are repair candidates.

| Reproduced behavior | Executions | Main outcome |
| --- | ---: | --- |
| Await in computed class names | 4 | NotImplemented |
| Yield in computed class and accessor names | 12 | NotImplemented |
| Composite async-generator return with a yielding argument | 1 | NotImplemented |
| Eval spread source preparation | 4 | NotImplemented |
| Proper tail call through a dynamically introduced eval binding | 1 | Crash |

The [adjacent cohort](../../test262/replays/expression-suspension-20260912.adjacent.executions)
contains another 59 executions, disjoint from the first 51. Main produces
43 Success, 13 NotImplemented, and three Crash, with no timeouts. Its 16
failures are the corresponding class declarations, named async-generator return,
and ordinary-function/global/with eval-binding tail calls. Together the two
cohorts contain 38 reproduced failures and 72 passing controls. Mode selection
comes from each pinned fixture's metadata.

The main compiler SHA-256 is
`91d221c8be3cb2ef6a4678a35b0e96061b66132c8f53241da60f4720f2eb14c6`.
Its 2,888 declared build inputs were hashed before and after the release build.
The Test262 tree pin is `aa55200d1310384c5cf69ea95b2a2ecba457007b`.
Raw main evidence is retained under
`target/failure-review/after-pr48-20260912/main-replay/`.

## Compiler behavior

ClassDefinitionEvaluation now has a statement-level continuation plan. Its
heritage and computed keys retain ordered prefixes, while the constructor,
class-name environment, and surrounding completion live in the activation.
Immediate and resumable class definitions share the same backend algorithm.
Heritage validation and the superclass's prototype Get precede computed keys;
each key conversion precedes the next key; class-name initialization and static
initializers run after key evaluation. Resumption restores the partially
constructed class and its private and field-key contexts. Nested and interleaved
activations retain distinct state, and abrupt resumption leaves pending static
initialization unexecuted.

All live superclass candidates reach runtime IsConstructor validation.
Superclass prototypes must be objects or null: primitives throw before key
effects, and null remains the new prototype's parent. Flow facts are invalidated
across yields and potentially observable superclass prototype Gets because
caller code and getters can mutate captured values before the next computed
key. Existing loop and branch admission limits for class-key suspension remain
explicit; this batch covers the admitted straight-line, nested, and
try-statement paths. Proven source/class constructors and the exact Array
intrinsic retain their existing flow facts: their own nonconfigurable
data `prototype` cannot invoke a getter. Bound functions, Proxy values, and
unknown constructors retain conservative invalidation. The actual prototype
value still undergoes runtime validation.

Suspended calls retain each already evaluated operand in its activation.
Property calls preserve the receiver, evaluate and convert computed keys in
the ordinary property Reference path, and save the callee's GetValue before
evaluating arguments. Replacing a method while an argument yields therefore
does not replace the already selected callee. Abrupt resumption skips the
pending call and later arguments. Async-generator return still awaits the
eventual call result, including thenables.

Plain property assignments inside suspended expressions retain the evaluated
receiver and raw computed key across RHS suspension, then use the existing
ordinary property assignment operation. This preserves the original receiver,
strictness, assignment result, and the current specification's
[key conversion after RHS evaluation](https://tc39.es/ecma262/2026/multipage/ecmascript-language-expressions.html#sec-evaluate-property-access-with-expression-key). Abrupt resumption skips the pending write.

The new straight-line call staging supports ordinary value calls and simple
property references without spread arguments. Calls requiring a suspended
dynamic environment Reference, direct-eval identity dispatch, optional chain,
private reference, or super reference remain explicit unsupported shapes in
this staging path. Existing ordinary non-suspending call paths remain available.

Finite eval source discovery follows optional source-function return candidates
through `@@iterator`, `next`, and result `value`, including source-defined
getters and indexed reads from finite source arrays. A leading spread can be
empty, so discovery continues through the first ordinary argument. The existing
256-candidate bound applies. These hints only prepare compiler specializations:
the emitted program still resolves the live callee, performs every iterator
operation and argument effect, and dispatches using the actual collected source.
Source text outside the prepared set retains the explicit Wasm-AOT dynamic-code
generation rejection; no runtime parser or interpreter is introduced.

Tail-position calls carry their continuation through environment-identifier
resolution and eval identity dispatch. Genuine intrinsic eval keeps its caller
environment; the ordinary-call branch uses Wasm tail calls. Outlined Proxy call
forwarding also releases its dispatcher frame. Active catch, finally, and
disposal work continue to prevent eliminating a required caller continuation.

## Verification

The two independently audited candidate replays pass **110/110** executions:
51/51 in the observed cohort and 59/59 adjacent executions. They repair all
38 main failures (34 NotImplemented and four Crash), preserve all 72 main
successes, and have zero timeouts. The four native regression targets pass
37 tests: 12 class, 12 generator-expression, six eval-spread, and seven tail-call
controls. The 12 class tests ran on the final source; the other 25 passes retain
their earlier source manifests and the subsequent source differences in the
verification record. The complete IR, backend, and frontend library suites pass
1,125, 428, and 164 tests respectively; all 80 structural checks pass across
16 targets, and 56 neighboring Wasmtime tests pass across eight targets.
Another 15 selected engine-library tests pass. The fake fixture suite passes
191/191 executions from 190 source files, with zero timeouts; this is separate
from pinned real Test262. Workspace checks across all targets, 67 replay-tooling
tests, formatting, module boundaries, task-plan validation, and diff checks
also pass.
The original long-running baseline is not restarted or relabeled as a candidate
run.

The [verification record](../../test262/replays/expression-suspension-20260912.verification.json)
contains exact execution transitions, test inventories and commands, source
and transcript hashes, resource limits, and the separately retained failed
attempts. Only completed passing targets are credited to their stated source
revisions.

The frozen candidate compiler SHA-256 is
`600b2f7a34d7d3b1dca7d0ae931af6443b89a98359c52918aa8b9f08a71a3442`.
All 2,899 declared build inputs match source commit
`6b8f489c56eedae17f6add09196d67a2ae4dd7b0`. Both cohorts were replayed after
the final heritage-effect correction; earlier compiler checkpoints remain
separate retained evidence.

The later validation commits, ending at
`2957f2007c2d6b7630044fe4bc49feb5fe93fddf`, change only an embedded test
module and two standalone structural tests. Their source references now include
the extracted class emitter. The accessor census
also refreshes totals already stale on main; main and candidate have identical
19 getter-role constructions, nine setter-role constructions, 25 descriptor-role
uses, and 13 host accessor definitions. The three class accessor definitions
remain exact. Production bytes before and after the embedded test module are
unchanged, and the completed 1,125-test IR checkpoint retains its original
compiler-source provenance.
The descriptor boundary assertion now names the current validated entrypoint;
the generator contract assertion reflects its already documented completed
verification. Both stale expectations predate this branch. The completed
428-test backend and 164-test frontend checkpoints retain their intermediate
validation commit, `d453d9ffe2cdca650edf19eb8b55e0877fb0d657`.

Refresh the focused cohort with:

```sh
cargo build --release --locked -j2 -p lila-cli
python3 scripts/replay-test262-executions.py \
  test262/replays/expression-suspension-20260912.executions \
  --binary target/release/lila \
  --output-dir target/test262-scratch/expression-suspension-20260912 \
  --workers 4
python3 scripts/audit-test262-replay.py \
  target/test262-scratch/expression-suspension-20260912 \
  --compiler target/test262-scratch/expression-suspension-20260912/run.json \
  --output target/test262-scratch/expression-suspension-20260912.audit.json
```

Repeat these commands with the `.adjacent.executions` list and a distinct
output directory to verify the 59 adjacent executions.

The replay auditor checks execution identities, modes, source hashes, native
transcripts, snapshots, and the frozen compiler hash. An origin audit supplied
with `--origin` additionally verifies exact outcome transitions.

## Separate existing async-assignment issue

A review reproducer outside these 110 executions still selects an assignment's
receiver too late when its computed key awaits. Both frozen main and the
candidate print `0:1` below; the assignment must retain `first` before evaluating
the key, so the expected result is `1:0`:

```js
async function check() {
  let first = { value: 0 }, second = { value: 0 }, receiver = first;
  receiver[await (receiver = second, "value")] = 1;
  print(first.value + ":" + second.value);
}
check();
```

The existing async property-assignment path needs to retain its Reference
operands. This is a separate pre-existing issue, not a repaired case in this
cohort. A property assignment with an awaited RHS also remains explicitly
unsupported (`async await assignment target`). Local reproducer and paired
transcript hashes are retained in
`target/failure-review/after-pr48-20260912/async-assignment-unrelated-review.json`.
