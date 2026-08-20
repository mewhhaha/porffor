// PER-ITERATION BINDINGS ACROSS AN AWAIT — the oracle for a defect that is NOT
// fixed at this head. This fixture is deliberately NOT wired into
// `crates/lila-cli/tests/cli/functions.rs`: its async half does not compile
// today, and a lane lands green or does not land. Read
// `target/lane-notes/ir-shapes-b7-integration.md` §2 before wiring it.
//
// # What it is for
//
// `for (const v of [..]) { …; await …; }` inside a plain async function is
// specialized by `lower_async_for_of_array_with_body_await` into a
// `StatementIr::GeneratorLoop` index walk. That specialization refuses any head
// whose loop binding is captured by a closure, because ECMA-262 14.7.5.7 then
// requires a fresh environment record per iteration.
//
// Batch 7 made the refusal say so (it used to claim "requires an array iterable
// and a plain binding", which is false for exactly this shape) and filed the fix.
//
// # Why the two test262 cases are NOT a sufficient gate
//
// `built-ins/Array/fromAsync/asyncitems-asynciterator-not-callable.js` and its
// `@@iterator` sibling are the only two failures in that 95-case node, they
// share one `detail_hash`, and they are the shape above. But their body only
// ever reads `typeof v` inside a closure that is CALLED IN THE SAME ITERATION.
// A "fix" that hoists the loop binding into a single activation-record slot —
// which is the one thing `lila-ir` can do on its own — passes both of them
// and is still wrong: every closure would share one cell.
//
// The async half below is the assertion those two cannot make. It collects
// `() => v` per iteration, calls all six AFTER the loop, and requires all six
// answers to be DISTINCT. A single hoisted slot answers `6,6,6,6,6,6`.
//
// # The sync half is the control, and it passes today
//
// The same shape without the `await` lowers to `StatementIr::ForOfArray`, whose
// emitter calls `emit_enter_lexical_environment` inside the loop and so really
// does allocate one environment record per iteration. It is here so that a
// failure of the async half cannot be blamed on per-iteration semantics being
// unimplemented generally: if the sync half ever goes red, the defect is
// somewhere else entirely and this fixture is not the right report.

let log = "";

// ------------------------------------------------------------- sync control

const syncClosures = [];
for (const v of [1, 2, 3, 4, 5, 6]) {
  syncClosures.push(() => v);
}
const syncValues = syncClosures.map((f) => f());
for (let i = 0; i < syncValues.length; i++) {
  for (let j = i + 1; j < syncValues.length; j++) {
    if (syncValues[i] === syncValues[j]) {
      throw "sync for-of shared one binding across iterations";
    }
  }
}
if (syncValues.join(",") !== "1,2,3,4,5,6") throw "sync for-of captured wrong values";
log += "sync=" + syncValues.join(",");

// ------------------------------------------------ async half (does not compile yet)

let asyncValues = null;

async function collect() {
  const closures = [];
  for (const v of [1, 2, 3, 4, 5, 6]) {
    closures.push(() => v);
    // One direct await in the body: this is what routes the loop through
    // `lower_async_for_of_array_with_body_await` rather than `ForOfArray`.
    await 0;
  }
  // Every closure is called AFTER the loop has finished, so a shared cell can
  // no longer hide behind same-iteration evaluation.
  asyncValues = closures.map((f) => f());
}

async function main() {
  await collect();

  for (let i = 0; i < asyncValues.length; i++) {
    for (let j = i + 1; j < asyncValues.length; j++) {
      if (asyncValues[i] === asyncValues[j]) {
        throw "async for-of shared one binding across iterations";
      }
    }
  }
  if (asyncValues.join(",") !== "1,2,3,4,5,6") throw "async for-of captured wrong values";
  if (asyncValues.length !== 6) throw "async for-of captured the wrong number of bindings";
}

main();

// The completion value is reached before the job queue drains: the async
// assertions report by throwing, so whoever wires this test asserts on
// `number(262` AND on the absence of `uncaught throw` in the output. `log` is
// kept so the sync half's values are visible in a failure dump.
if (log.length === 0) throw "sync control did not run";

262;
