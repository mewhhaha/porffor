# JSON reviver frame protocol

## Scope

This contract owns the iterative Wasm-AOT implementation of
`InternalizeJSONProperty`, including the static-JSON specialization's final
reviver application. It does not own JSON tokenization, primitive decoding,
`JSON.stringify`, or the validity of the static source proof.

The reviver walk is depth-first postorder. For each property it observes the
current value, recursively internalizes that value's children, calls the
reviver, and only then applies the reviver result to the holder. A reviver can
replace an unvisited child, install accessors or proxies, mutate later
properties, or throw an arbitrary JavaScript value. The walk must observe all
of those effects through the ordinary object operations at their specified
positions.

## Frame state

The dynamic walk stores one private frame per active property. Its state is the
closed domain:

- `Enter`: read the current property value and classify it;
- `ArrayChildren`: visit the snapshotted array-index range in ascending order;
- `ObjectChildren`: visit the snapshotted enumerable own string keys in order;
- `Apply`: call the reviver and consume its result.

`Enter` performs `Get` before classification. For an Array it observes and
converts `length` once, then stores that limit on the frame. For another Object
it obtains the enumerable own string-key list once, then stores that list and
its length. Child frames are pushed in ascending cursor order onto a LIFO
stack, which makes their `Apply` steps run before the parent's `Apply` step.

Every persisted state word comes from `JsonReviverFrameState`. Runtime dispatch
is generated from its complete ordered set and reaches an exhaustive Rust
match. An invalid word traps as an internal invariant failure instead of
silently inheriting one state's behavior. Adding a state therefore requires an
explicit emitter decision before the backend builds.

## Root versus nested properties

The synthetic wrapper property used by `JSON.parse` is semantically different
from an ordinary child property. That distinction is the closed
`JsonReviverPropertyRole` domain:

- `Root`: the reviver result is the result of `JSON.parse` and does not mutate
  the wrapper;
- `Nested`: `undefined` requests deletion from the holder, while any other
  result requests creation or replacement of the holder property.

The role is explicit at both static and dynamic reviver call sites. It is never
derived from the key spelling: an ordinary nested property named the empty
string is still `Nested`. Dynamic frames persist the role through its stable
wire word, and frame creation accepts the typed role rather than a Boolean
local. Only the shared post-call emitter in `builtins/json.rs` consumes the
distinction. The static caller in `builtins/json/static_reviver.rs` and the
dynamic caller in the parent therefore cannot drift into different root or
child mutation rules.

## Source context and abrupt completion

Parse metadata may provide the third reviver argument's `source` property only
for a primitive whose current value remains `SameValue` to the value produced
from that source slice. Mutation clears that eligibility. Arrays and Objects
receive an empty context object.

Every observable `Get`, `IsArray`, key enumeration, length conversion, reviver
call, deletion and property creation retains its existing abrupt-completion
edge. A throw stops the walk immediately and is propagated unchanged. State or
role validation is an internal boundary check; it must not turn an ordinary
JavaScript abrupt completion into a parser error or a default result.

## Durable evidence owner

`crates/lila-aot-wasm/tests/json_reviver_frame_structure.rs` is the bounded
source owner for this protocol. Its five tests pin:

- the private static-reviver child boundary, exact type/function inventory,
  sole compiler entry, two expression callers and retained shared parent
  operations;
- the exact four-state and two-role wire domains, including their ordered
  words and generated `ALL` sets;
- typed state/role persistence, exhaustive state and role dispatch, and an
  explicit trap for each invalid persisted word;
- one shared post-call result owner with exactly one static-specialized caller
  and one dynamic-frame caller; and
- the one active exact CLI registration plus non-vacuous dynamic-fixture
  assertions for postorder traversal, collection snapshots, forward mutation,
  source eligibility, nested empty-string versus root roles, deletion and
  abrupt propagation.

The CLI and fixture guards mask line and block comments before resolving their
owners and executable markers. The CLI guard also rejects attached `ignore`,
`cfg` and `cfg_attr` attributes. The fixture guard requires the throwing
failure boundary, unique load-bearing assertions, scenario-local mutation and
abrupt-completion order, and one final success value at end of file. It
therefore cannot pass when an expected owner or marker survives only inside a
line or block comment, or when an attached attribute disables the CLI test.

## Recorded verification

The coordinated checkpoint ran this focused ladder on 2026-08-25:

```sh
cargo check -p lila-aot-wasm
cargo xc
cargo test -p lila-aot-wasm --test json_reviver_frame_structure
cargo test -p lila-cli --test cli language_numerics::run_wasm_backend_succeeds_for_json_parse_dynamic_reviver_frame_fixture -- --exact --test-threads=1
cargo test -p lila-cli --test cli language_numerics::run_wasm_backend_succeeds_for_json_parse_reviver_array_getter_throw_fixture -- --exact --test-threads=1
cargo test -p lila-cli --test cli language_numerics::run_wasm_backend_succeeds_for_json_parse_reviver_forward_modification_fixture -- --exact --test-threads=1
cargo test -p lila-cli --test cli language_numerics::run_wasm_backend_succeeds_for_json_parse_reviver_nonconfigurable_fixture -- --exact --test-threads=1
./target/debug/lila --jobs 1 test262 run built-ins/JSON/parse/reviver-call-order.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila --jobs 1 test262 run built-ins/JSON/parse/reviver-call-args-after-forward-modification.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila --jobs 1 test262 run built-ins/JSON/parse/reviver-array-length-get-err.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila --jobs 1 test262 run built-ins/JSON/parse/reviver-forward-modifies-object.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila --jobs 1 test262 run built-ins/JSON/parse/reviver-context-source-primitive-literal.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila --jobs 1 test262 run built-ins/JSON/parse/reviver-wrapper.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
```

`cargo check -p lila-aot-wasm` and `cargo xc` are green. The structure target
passes `4/4`, and the four exact CLI fixtures pass `4/4`. None of the six
direct Test262 leaves declares `onlyStrict`, `noStrict` or `raw`, so they
discover twelve ordinary sloppy-and-strict executions. All `12/12` pass under
Wasm-AOT at vendored suite content tree
`aa55200d1310384c5cf69ea95b2a2ecba457007b`, with every failure and
non-success bucket at zero.

Batch AM makes the three macro-generated domains capability-free JSON wire
domains. `JsonReviverFrameState`, `JsonReviverPropertyRole` and
`JsonParseFrameState` no longer derive clone, copy, debug, equality or any
other incidental identity capability. Their stable wire projection borrows
the selected identity, and all three complete-set traversals borrow the
macro-owned identities instead of copying them. The wire words, complete sets,
exhaustive dispatch and emitted instructions remain unchanged. The
source-present, source-ineligible and source-absent static branches now borrow
one role through their shared helper boundary. At the Batch AM checkpoint,
`cargo xc` is green, the reviver and parse-frame structure targets pass `5/5`
and `4/4`, and the exact dynamic-reviver CLI witness passes `1/1`. No focused
Test262 leaf or semantic golden was required or run for this source-equivalent
capability hardening.

Batch AN makes the private static-reviver key a capability-free
`JsonStaticPropertyKey`. Its exact string-or-array-index roles are now borrowed
through key materialization, holder lookup and the final reviver-result stage;
clone, copy, formatting, default, comparison, ordering and hashing cannot
create a second key identity route. All three producers borrow their temporary
closed role immediately. The key payloads, Array index words, lookup order,
reviver calls and emitted instructions remain unchanged. At the Batch AN
checkpoint, `cargo xc` is green, the five-test reviver structure owner passes
`5/5`, and the exact forward-modification CLI witness passes `1/1`. No focused
Test262 leaf or semantic golden was required or run for this source-equivalent
capability hardening.

## Non-claims

This protocol does not close T20 or the pinned JSON tree. It does not validate
JSON grammar, remove the static specialization, prove deep-input resource
bounds, change non-configurable-property behavior, or cover stringify,
replacer, cycle, BigInt or proxy semantics outside the reviver walk. Complete
current-pin Wasm-AOT evidence remains a separate verification requirement.
