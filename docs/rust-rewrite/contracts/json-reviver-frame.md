# JSON reviver frame protocol

## Scope

This contract owns the canonical iterative Wasm-AOT implementation of
[InternalizeJSONProperty](https://tc39.es/ecma262/2026/multipage/structured-data.html#sec-internalizejsonproperty).
The source-only T20 batch dated 2026-10-03 retires static JSON materialization
and its reviver specialization. All ordinary source `JSON.parse` calls retain
callee/input/reviver evaluation through ordinary Call IR and enter the existing
emitted runtime parser and this walk. Compilation, Wasm validation, semantic
execution and conformance acceptance of that batch remain pending.

The private static parser, static value IR, two special producer branches,
backend materializer/walker and every exhaustive special visitor/planning/data/
expression arm are removed together. String and callback target inference still
serve ordinary evaluation; neither inference replaces runtime JSON text or
materializes the parsed value. The existing canonical parser owns grammar and
primitive decoding; `JSON.stringify` remains a separate owner.

The reviver walk is depth-first postorder. For each property it observes the
current value, recursively internalizes that value's children, calls the
reviver, and only then applies the reviver result to the holder. A reviver can
replace an unvisited child, install accessors or proxies, mutate later
properties, or throw an arbitrary JavaScript value. The walk must observe all
of those effects through the ordinary object operations at their specified
positions.

## Frame state

The sole walk stores one private frame per active property. Its state is the
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

The classification guard uses the existing heap-object tag authority, covering
Object, Array, Function and Arguments representations. Inserted Functions and
Arguments therefore supply real descendant holders. The consumed callback
observation uses the existing heap-coercible kind domain plus Function and
keeps Function target knowledge open. A callback's `this()` cannot be inferred
as a proven empty target set, undefined result or effect-free call.

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

The role is explicit at the sole frame caller. It is never derived from the
key spelling: an ordinary nested property named the empty string is still
`Nested`. Frames persist the role through its stable wire word, and frame
creation accepts the typed role rather than a Boolean local. Only the shared
post-call emitter in `builtins/json.rs` consumes the distinction. Nested
CreateDataProperty and deletion ignore a false Boolean result while retaining
an abrupt completion, including Array descriptors and Proxy definitions.

## Source context and abrupt completion

Parse metadata may provide the third reviver argument's `source` property only
for a primitive whose current value remains `SameValue` to the value produced
from that source slice. Mutation clears that eligibility. Arrays and Objects
receive an empty context object.

Private Object metadata-child maps have an explicit null prototype. A newly
inserted current key has no parse record and cannot trigger an inherited user
getter while looking up hidden metadata. Private Array metadata reads remain
own-element reads. Public parsed objects, root wrappers and context objects
retain their existing prototype owners. The actual current value's Get occurs
through ordinary operations, with source context absent for inserted values
and present for eligible original primitives.

Every observable `Get`, `IsArray`, key enumeration, length conversion, reviver
call, deletion and property creation retains its existing abrupt-completion
edge. A throw stops the walk immediately and is propagated unchanged. State or
role validation is an internal boundary check; it must not turn an ordinary
JavaScript abrupt completion into a parser error or a default result.

## Durable evidence owner

`crates/lila-aot-wasm/tests/json_reviver_frame_structure.rs` is the bounded
source owner for this protocol. Its six maintained tests pin:

- the sole runtime parser/iterative owner and removal of static IR/modules;
- ordinary acquired-callee, receiver and complete argument evaluation order;
- the exact four-state and two-role wire domains, including ordered words and
  generated `ALL` sets;
- typed state/role persistence, exhaustive dispatch, actual heap-object
  admission and explicit traps for invalid persisted words;
- one post-call result owner with exactly one iterative-frame caller; and
- the active exact CLI registration plus non-vacuous dynamic-fixture assertions
  for postorder traversal, collection snapshots, forward mutation, source
  eligibility, nested empty-string versus root roles, deletion and abrupt
  propagation.

Existing JSON IR assertions now require ordinary calls for literal/static
input, spread arguments, repeated mutable input and scoped input. They retain
acquired non-property callees, one materialized property receiver, argument
positions, TDZ input effects and callback-holder observations.

Seven new `aot_json_canonical_reviver` semantic sources cover nonempty same-tag
sibling replacement and new descendants, Function/Arguments holders and a
Function-holder call's effect/result, live snapshots and inherited indices,
SameValue source eligibility and duplicate JSON names, `__proto__` and nested
empty-string keys, ordinary call operands/closure callbacks, arbitrary abrupt
identity and metadata maps insulated from inherited hooks. Each expected trace
is authored for sloppy and strict mode. These sources have not been executed.
Existing `aot_json_reviver_definitions` continues to own descriptor, ignored
false result, Proxy throw, public intrinsic replacement and foreign-Realm
controls; their earlier evidence is not new acceptance of this batch.

The CLI and fixture guards mask line and block comments before resolving their
owners and executable markers. The CLI guard also rejects attached `ignore`,
`cfg` and `cfg_attr` attributes. The fixture guard requires the throwing
failure boundary, unique load-bearing assertions, scenario-local mutation and
abrupt-completion order, and one final success value at end of file. It
therefore cannot pass when an expected owner or marker survives only inside a
line or block comment, or when an attached attribute disables the CLI test.

## Pre-retirement recorded verification

The following records concern the earlier static/dynamic composition and
source-equivalent capability changes. They are retained as historical evidence
and do not verify the 2026-10-03 static retirement or canonical corrections.

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
all JSON grammar, prove deep-input resource bounds, or cover stringify,
replacer, cycle, BigInt or Proxy semantics outside the reviver walk. Static
specialization removal and the recorded canonical corrections are source
implementation claims pending the combined checkpoint. Complete
current-pin Wasm-AOT evidence remains a separate verification requirement.
