# `AggregateError` construction phases

Status: theory, integrated implementation, independent review and capped
focused verification complete for the bounded T24 Wasm-AOT construction seam,
2026-08-23. The constructor and Promise.any origins use closed prepared-object
paths.

## Specification boundary

The living and pinned-2026
[`AggregateError ( errors, message [ , options ] )`](https://tc39.es/ecma262/2026/multipage/fundamental-objects.html#sec-aggregate-error-constructor)
algorithm has one observable sequence:

1. resolve the effective `NewTarget` and create the branded object with
   `OrdinaryCreateFromConstructor`;
2. when `message` is not `undefined`, perform `ToString(message)` and create the
   non-enumerable `message` property;
3. perform `InstallErrorCause(obj, options)`;
4. acquire and consume the `errors` iterator with `GetIterator` and
   `IteratorToList`; and
5. create the non-enumerable `errors` property and return the object.

Every conversion or object operation in that sequence can complete abruptly.
In particular, an abrupt message conversion must prevent both cause inspection
and iterator acquisition, while an abrupt `HasProperty(options, "cause")` or
`Get(options, "cause")` must prevent iterator acquisition. When all three own
properties are present, their creation order is `message`, `cause`, `errors`.

The pre-lane emitter did not satisfy this boundary. It converted the optional
message, consumed the complete `errors` iterator, allocated an object containing
`message` and `errors`, and only then inspected `options.cause`. That made the
iterator observable before cause and gave the own properties the wrong order.
This contract selects that real semantic defect; it is not merely a local
refactor of argument indices.

`Error` and the six NativeError message constructors use the distinct
`(message, options)` signature, so their options value is argument one.
`AggregateError(errors, message, options)` uses argument two. The pre-lane
shared `InstallErrorCause` emitter accepted an arbitrary `usize`, allowing a
caller to transpose those roles without a compiler error.

The same backend module also owns the two internal AggregateError origins in
[`PerformPromiseAny`](https://tc39.es/ecma262/2026/multipage/control-abstraction-objects.html#sec-performpromiseany).
That algorithm does not call the observable constructor: it creates a fresh
AggregateError object, defines only its `errors` property and rejects the
result promise with it. Those paths must therefore share final `errors`
installation without reading a builtin message or options argument.

## Closed options role

The only product constructors that call the shared cause installer are carried
by a private closed domain:

```rust
enum ErrorCauseOptionsArgument {
    MessageError,
    AggregateError,
}
```

An exhaustive projection maps `MessageError` to argument index one and
`AggregateError` to argument index two. The shared emitter accepts this role,
not a raw `usize`; neither caller may recover the index and pass it through a
second untyped entry. `SuppressedError` has no `options` or `cause` step and is
not a member of this domain.

The two message-constructor branches must both pass `MessageError`. The one
AggregateError preparation phase must pass `AggregateError`. A new
cause-installing signature therefore requires an explicit variant and an
exhaustive projection update.

## Prepared AggregateError lifecycle

The AggregateError arm must not expose an allocator that can install `errors`
before cause. One private, non-`Copy`, `#[must_use]`
`PreparedAggregateErrorLocal` means that the specification-specific prefix for
one AggregateError origin has completed and the object is ready for its
`errors` property.

The constructor producer returns the token only after exactly this prefix has
been emitted:

1. allocate the object with the already-resolved prototype and install its Error
   brand;
2. conditionally convert and define `message`;
3. install `cause` through `ErrorCauseOptionsArgument::AggregateError`.

The constructor's errors iterator is consumed only after that producer returns.

A second private producer exists only for `PerformPromiseAny`. It allocates the
object with the intrinsic AggregateError prototype and installs the Error
brand, but deliberately performs no message or cause operation because the
Promise algorithm specifies neither. A narrowly named crate-visible wrapper
keeps that token private, immediately sends it to the same consuming finalizer,
and has exactly the two Promise.any callers required by the standard: the
empty/exhausted input path and the last reject-element path.

The sole finalizer takes ownership of either prepared token, defines `errors`,
publishes the exact object/tag result pair and releases its temporary local.
The old combined allocator that accepted an optional message, arbitrary
precomputed errors list and prototype remains deleted; Promise code cannot call
the constructor producer or fabricate the token.

This split makes the important property boundary type-visible: code that wants
to define `errors` must hold an object whose complete origin-specific prefix has
already been emitted. A structural ordering guard remains necessary because
Rust types cannot prevent an unrelated iterator helper from being called before
the constructor token producer.

## Preserved semantics

The change retains the existing effective-NewTarget and prototype-resolution
path, abrupt-completion transport including arbitrary user-thrown values,
iterator protocol implementation, Error brand, descriptors, and result tuple.
The message-undefined branch creates no `message` property. Primitive or absent
options create no `cause` property. The errors list is still materialized as an
Array and installed as a writable, non-enumerable, configurable data property.

Allocation before message coercion follows the specification even though the
fresh object is not ordinarily observable until construction succeeds. No
operation may be duplicated between the prepared prefix and finalizer.

## Durable structural regression

A bounded source-structure test must isolate the AggregateError match arm, the
cause-options domain, the prepared-prefix producer and the consuming finalizer.
It must pin:

- exactly the two options-role variants and an exhaustive one/two projection;
- a typed cause-installer signature with no raw options index;
- exactly two `MessageError` callers and one `AggregateError` caller;
- a private, must-use, non-`Copy` prepared token with one object-local field;
- exactly two private token producers and one consuming finalizer;
- one allocation and Error-brand write in each producer;
- optional message conversion/definition before cause installation;
- prefix preparation before iterator consumption, and iterator consumption
  before the consuming finalizer;
- one narrowly named Promise.any wrapper with no message/cause operation and
  exactly the exhausted-input and reject-element callers;
- `errors` definition only in the finalizer, followed by result publication and
  local release; and
- source-wide absence of the old combined AggregateError allocator.

The guard checks compiler structure, not runtime behavior. It should normalize
whitespace only around exact wiring sentinels and avoid a broad source snapshot.

## Focused runtime evidence

The durable CLI fixture
`crates/lila-cli/tests/fixtures/wasm_aggregateerror_constructor_properties.js`
must additionally observe:

- `message` conversion before cause access before `errors[Symbol.iterator]`;
- an omitted message producing no own `message` property while cause still
  precedes iterator acquisition;
- an abrupt message conversion preventing both cause access and iterator
  acquisition;
- the own-key order `message`, `cause`, `errors`;
- an abrupt cause getter preventing iterator acquisition; and
- the existing cause values and descriptors.

The existing iterable fixture remains the focused iterator-protocol control.
After the full write batch and independent review, the capped serial ladder is:

```sh
cargo test -p lila-aot-wasm \
  --test aggregate_error_construction_structure -- --test-threads=1
cargo test -p lila-aot-wasm --lib \
  tests::error_prototype_to_string_has_typed_ordered_observable_phases -- \
  --exact --test-threads=1
cargo test -p lila-cli --test cli -- --exact language_errors::run_wasm_backend_succeeds_for_aggregateerror_constructor_properties_fixture
cargo test -p lila-cli --test cli -- --exact language_errors::run_wasm_backend_succeeds_for_aggregateerror_iterable_to_list_fixture
./target/debug/lila --jobs 1 test262 run built-ins/AggregateError/cause-property.js --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 180000 --threads 1
./target/debug/lila --jobs 1 test262 run built-ins/AggregateError/message-undefined-no-prop.js --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 180000 --threads 1
./target/debug/lila --jobs 1 test262 run built-ins/AggregateError/message-tostring-abrupt.js --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 180000 --threads 1
./target/debug/lila --jobs 1 test262 run built-ins/AggregateError/order-of-args-evaluation.js --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 180000 --threads 1
./target/debug/lila --jobs 1 test262 run built-ins/Promise/any/iter-arg-is-empty-iterable-reject.js --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 180000 --threads 1
./target/debug/lila --jobs 1 test262 run built-ins/Promise/any/reject-deferred.js --suite-root test262/vendor/test262 --execution-backend wasm --timeout-ms 180000 --threads 1
```

The checkpoint is green as of 2026-08-23 under the repository's shared
eight-core and 22 GB resource cap. `cargo fmt --all -- --check`, `cargo xc` and
`git diff --check` pass. The AggregateError construction structure suite passes
`3/3`, and the existing exact
`error_prototype_to_string_has_typed_ordered_observable_phases` library witness
passes `1/1`. The constructor-properties and iterable-to-list CLI fixtures each
pass `1/1`. Each of the four AggregateError and two Promise.any current-pin
Test262 leaves above passes its ordinary sloppy and strict variants, for `12/12`
Wasm-AOT executions with every failure bucket at zero under
`--jobs 1 --threads 1`.

## Nonclaims

This seam does not complete AggregateError prototype fallback or created-Realm
construction. In particular, it does not change the existing Realm selection
inside generic object-to-string conversion or iterator-created native errors.
It does not replace the iterator implementation, change iterator-close
semantics, change Promise.any settlement behavior, migrate SuppressedError,
redesign the completion ABI, add `exnref`, or close the full Promise,
AggregateError or NativeErrors trees. It does not change a published
conformance count, refresh a snapshot or make T24 complete.
