# Default-export names and async assignments after PR #49

## Baseline and scope

The September 12 observation of the continuing `c5115bf03` baseline contains
66,569 of 102,043 executions (65.2%) across 487 of 744 nodes: 55,732 Success,
8,127 Bug, 746 Crash, and 1,964 NotImplemented. All 63,569 executions and
475 leaf hashes from the preceding observation are preserved. The additional
3,000 executions contain 2,920 successes and 80 failures.

After fetching origin and rebasing onto merged main
`d9d130b7aba39e726d7b1f5c2b98f5d977ca7360`, a freshly built compiler replayed
the [80 exact failures](../../test262/replays/module-names-async-assignment-20260912.executions).
All 79 class-field eval cases already pass. One failure remains: an anonymous
default-exported class exposes the internal name `$d0$` instead of `default`
during static initialization.

The [22 adjacent executions](../../test262/replays/module-names-async-assignment-20260912.adjacent.executions)
contain 14 module-name assertions, six static-initializer name controls, and
two awaited computed-key accessor modules. On main, four anonymous definition
naming cases and two hoisted default function/generator binding cases fail;
16 cases pass. The combined 102-execution cohort therefore contains five naming
Bugs, two declaration-hoisting Bugs, and 95 passing controls, with zero main
timeouts. Execution modes come from fixture metadata.

The main compiler SHA-256 is
`7e9be88ca3ca091b1a1f23514838cd42f147613037b98bd720248ab622c61b44`.
Its 2,899 declared source inputs match merged main. The pinned Test262 tree is
`aa55200d1310384c5cf69ea95b2a2ecba457007b`. The original long-running baseline
continues with its original compiler; it is not a candidate run.

## Default-export names and initialization

Anonymous default exports need the observable name `default`, while the
linker's internal binding must remain distinct from source bindings. Naming
provenance identifies the exported initializer itself and its resulting
function or class, including expressions inside linked module wrappers.
The display name is established before class static initialization. Explicit
source names and class members that define their own `name` retain their
ordinary behavior. Nameless class expressions expose an empty name for both
explicit and implicit constructors; the compiler's `<class>` diagnostic label
is no longer used as their runtime name. A separate frozen-main probe
reproduced that leak while the new native controls exercised non-definition
default exports and array-contained classes.

Nameless ordinary function expressions also expose an empty name, matching
generator, async, and async-generator expressions; explicit and inferred names
and exact callable source remain intact.

Anonymous default function declarations also register their existing function
identity with the owning declaration-instantiation plan. Imports can call them
before the source declaration, including through cyclic dependencies and
linked module wrappers. Their synthetic evaluation-time initializer is removed
so it cannot create a second function object. The callable source remains
anonymous. Default-exported expressions retain their evaluation-time
initialization and temporal dead zone. When module linking rewrites an anonymous
default declaration as an initializer, it adds a terminator at the parsed
declaration boundary so a following statement on the same line stays valid.
The shared UTF-16 span conversion preserves Unicode offsets and callable source;
dynamic imports are rescanned after this rewrite.

## Async property assignment

The previous follow-up documented two additional defects outside the pinned
cohort. Fresh main reproduces both:

```js
async function check() {
  let first = { value: 0 }, second = { value: 0 }, receiver = first;
  receiver[await (receiver = second, "value")] = 1;
  print(first.value + ":" + second.value);
}
check();
```

Main prints `0:1`; the required result is `1:0`. A plain property assignment
with an awaited RHS also fails compilation with `async await assignment target`.

Plain property assignments retain the evaluated receiver and raw computed key
in the existing activation storage before subsequent awaits. The ordinary
Reference consumer performs the resumed write and preserves the assignment
result, strictness, Symbol keys, setter receivers, and abrupt completions.
Raw-key conversion remains after RHS evaluation. This also applies to async
arrows and async generators using the same await machinery.

Resumable functions also need a stable activation-environment decision before
capture hops are calculated. Previously, adding temporary operand storage to
an otherwise empty async arrow could introduce an environment after analysis,
causing captured receiver reads to follow the wrong path. Analysis and Wasm
emission now agree that resumable activations have their own environment even
when their initially planned binding list is empty.

Conditionally reached awaits in this assignment path remain an explicit
unsupported shape when the existing suspension validator cannot preserve their
branching. The change does not add compound assignment suspension or a new
runtime/backend representation.

## Verification

The verified compiler source is `34f999537f4b27762202cb1abcdd9722bbf6c74c`.
All 2,904 declared compiler inputs match that commit. Its frozen CLI SHA-256 is
`85c1a46a58ff39f04f40bb45ff1e15d5d29828e96ba7c85a3a06e7da2fd7dab2`.

The fresh candidate replay passes 102/102 pinned executions with zero timeouts: all seven
main Bugs become Success, and all 95 main successes remain Success. The workspace
all-targets check, 13 focused IR tests, 19 focused Wasmtime regression groups,
and both paired CLI async-assignment probes also pass. The broader checks pass
428 backend library tests, 164 frontend library tests, 15 module-structure tests,
45 neighboring Wasmtime regressions, and five targeted engine tests. The corrected
IR assertion passes separately, with an exact source comparison preserving the
1,125 unchanged passing tests. The fake suite passes 191/191 executions across
190 physical fixtures. These fake-suite results remain separate from pinned
real Test262 conformance.

The [verification record](../../test262/replays/module-names-async-assignment-20260912.verification.json)
contains compiler/source hashes, exact execution transitions, resource settings,
paired command transcripts, and the retained historical failures.

An earlier candidate passed 100/102 but retained the two declaration-hoisting
Bugs. A later full IR run on `1d2de64a4` passed 1,125 tests and failed one stale
capture-hop expectation: an empty async disposer now owns an activation frame,
so the captured parameter is two hops away. The test-only follow-up corrects
that expectation and adds a Wasmtime disposal regression checking TDZ access
and captured reads across `await`. Historical failures remain in the evidence;
the old full IR run is not reported as green. A byte-identity check also refused
replay reuse after the test-only change, requiring a fresh pinned replay.

The live original-compiler baseline subsequently reached 68,569 executions and
495 nodes. Its additional 2,000 executions contain 1,913 successes and 87
NotImplemented results (44 direct-eval, 39 indirect-eval, four arrow-heritage).
Those later observations have not been replayed on this candidate and are outside
the frozen comparison above. The full baseline remains incomplete.

Refresh both pinned cohorts with distinct output directories:

```sh
cargo build --release --locked -j2 -p lila-cli
python3 scripts/replay-test262-executions.py \
  test262/replays/module-names-async-assignment-20260912.executions \
  --binary target/release/lila \
  --output-dir target/test262-scratch/module-names-async-assignment-20260912 \
  --workers 4
python3 scripts/audit-test262-replay.py \
  target/test262-scratch/module-names-async-assignment-20260912 \
  --compiler target/test262-scratch/module-names-async-assignment-20260912/run.json \
  --output target/test262-scratch/module-names-async-assignment-20260912.audit.json
```

Repeat with `.adjacent.executions` and a separate output directory. Supply a
frozen main audit through `--origin` to verify exact outcome transitions.
These focused checks remain separate from the published full-suite status.
