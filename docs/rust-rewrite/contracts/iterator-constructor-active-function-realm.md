# Iterator constructor active-function identity

## Scope

This contract spans the realm-local active identity and Iterator body in
`crates/lila-aot-wasm/src/builtins/standard.rs`, the direct-returning construct
routing in `crates/lila-aot-wasm/src/functions.rs`, the structural guards in
`crates/lila-aot-wasm/src/lib.rs`, and the focused CLI fixture/test. Entry and
created-realm identity publication are existing bootstrap/host inputs that the
guard verifies but this seam does not modify.

`Iterator` is unusual among subclassable constructors: it must throw when
`NewTarget` is either `undefined` or the active function object. The second
case is an object-identity test, not a test for the shared builtin body or for
the entry realm's `%Iterator%` object.

The normative algorithm is
[`Iterator ( )`](https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-iterator).
Its first step precedes
`OrdinaryCreateFromConstructor(NewTarget, "%Iterator.prototype%")`, so an exact
active-function match throws before any observable `NewTarget.prototype` Get.

The pinned
`built-ins/Iterator/newtarget-or-active-function-object.js` case covers
`Iterator()` and `new Iterator()` in the entry realm. It does not distinguish
two realm-local `%Iterator%` objects backed by the same emitted builtin body.
This seam is therefore a direct specification and source invariant, not a
claim that a pinned Test262 failure measured it.

## Runtime identity sources

Entry bootstrap preallocates `%Iterator%` in
`ITERATOR_CONSTRUCTOR_GLOBAL_INDEX`. Its standard-builtin environment is zero,
so that global is the active function object for an entry-realm invocation.

`$262.createRealm()` materializes a distinct Iterator constructor. The created
realm builder stores that function object in its own
`HEAP_FUNCTION_ENV_HANDLE_OFFSET`; the shared builtin body receives the value as
`current_env_local`. The same self-backed object carries the created realm's
TypeError prototype.

The active object is consequently:

1. `current_env_local` when it is nonzero; otherwise
2. the closed active-builtin identity's entry global.

The former implementation compared `NewTarget` directly with
`ITERATOR_CONSTRUCTOR_GLOBAL_INDEX`. It therefore inverted both cross-realm
cases:

- constructing the created-realm Iterator with itself as `NewTarget` missed
  the required throw; and
- invoking that created-realm Iterator with the entry Iterator as `NewTarget`
  threw even though the two function objects are distinct.

## Closed active-builtin domain

`ActiveStandardBuiltinFunction` is the private closed domain of standard
builtins whose algorithms need this active-object identity. Its initial and
only member is `IteratorConstructor`; an exhaustive map ties that member to
`ITERATOR_CONSTRUCTOR_GLOBAL_INDEX`.

The active-function emitter consumes that domain and emits the environment-or-
entry-global choice. The Iterator constructor arm may not read the entry global
directly. Adding a second active-builtin identity without defining its entry
global is therefore an exhaustive-match compile error, while the structural
guard rejects bypassing the typed operation in the Iterator arm.

The existing Function-tag check remains part of `SameValue`: a Proxy or bound
function around `%Iterator%` is a distinct object and must not be mistaken for
the active function merely because it eventually reaches the same builtin
body.

## Construct-dispatch ownership

`Iterator` is a direct-returning constructor in the shared `[[Construct]]`
dispatcher. This is a semantic ownership boundary, not only a return-value
optimization: the Iterator body owns active-function rejection followed by the
sole `GetPrototypeFromConstructor` and result allocation.

The direct-returning dispatch must therefore invoke the Iterator body and leave
the generic construct block before the generic path reads
`NewTarget.prototype` or preallocates a receiver. Otherwise distinct
`NewTarget` proxies observe two prototype Gets, and a self-active Iterator does
observable construction work before its required throw. A structural guard
pins exact Iterator membership plus the dispatch/Get/allocation source order.

## Observable regression

The durable CLI fixture covers the full two-realm identity matrix:

1. entry target plus the same entry `NewTarget` throws an entry-realm
   TypeError;
2. created target plus the same created `NewTarget` throws a created-realm
   TypeError;
3. created target plus the distinct entry `NewTarget` constructs with the entry
   Iterator prototype; and
4. entry target plus the distinct created `NewTarget` constructs with the
   created-realm Iterator prototype.

The two distinct directions are also repeated with observing Proxy new targets;
each must see exactly one `prototype` Get. The active constructor's own
`prototype` is non-configurable, so its zero-Get requirement is pinned by the
algorithm ordering and structural dispatch guard rather than replacing that
property with an accessor.

The entry self-case also exercises the zero-environment entry-global branch.
The created self-case and created-target/entry-NewTarget case independently
fail under the former entry-global comparison. The final cross-realm direction
prevents a replacement from treating every realm-local Iterator constructor as
the same active object.

## Deferred gates

This implementation batch performs static source and diff checks only while
the central verifier owns Cargo and Test262 resources. Later verification must
include:

```sh
cargo test -p lila-aot-wasm iterator_constructor_active_function_ --quiet
cargo test -p lila-cli run_wasm_backend_distinguishes_iterator_active_function_across_realms --quiet
./target/debug/lila test262 run built-ins/Iterator/newtarget-or-active-function-object --execution-backend wasm --timeout-ms 180000 --threads 1
./target/debug/lila test262 run built-ins/Iterator/proto-from-ctor-realm --execution-backend wasm --timeout-ms 180000 --threads 1
```

The complete T15 ladder and current-SHA low-RAM publication path remain the
final closure gates.

## Non-claims

This seam does not generalize active-function identity to every builtin, change
the created-realm ABI, or alter the shared `GetPrototypeFromConstructor`
implementation. It does classify Iterator as direct-returning so that its body
is the sole owner of that operation and allocation. It does not
address generator suspension, IteratorClose, helper closing, explicit resource
management, GC, the separate flatMap harness rewrite, or
`AsyncDisposableStack` realm publication. It makes no Test262 status claim.
