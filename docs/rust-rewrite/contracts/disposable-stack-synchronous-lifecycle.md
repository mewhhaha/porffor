# DisposableStack synchronous lifecycle

## Scope

This contract extends
[`disposable-stack-construction-brand.md`](disposable-stack-construction-brand.md)
from the constructor shell to the complete synchronous `%DisposableStack%`
surface:

- `DisposableStack.prototype.use`;
- `DisposableStack.prototype.adopt`;
- `DisposableStack.prototype.defer`;
- `DisposableStack.prototype.move`;
- `DisposableStack.prototype.dispose` and its identical `Symbol.dispose`
  alias; and
- the `DisposableStack.prototype.disposed` getter.

It does not change `%AsyncDisposableStack%`, lower `using` declarations, or
add an interpreter/runtime fallback. The emitted Wasm owns the resource stack
and executes the acquired disposal functions directly.

## Closed runtime domains

`[[DisposableState]]` is the closed domain
`DisposableStackState::{Pending, Disposed}`. A synchronous resource entry is
the closed domain `DisposableStackEntryKind::{Use, Adopt, Defer}`. Each kind
has exactly one exhaustive disposal call shape:

| Entry | Stored resource | Stored method | Disposal call |
| --- | --- | --- | --- |
| `Use` | `value` | acquired `value[Symbol.dispose]` | `Call(method, value, « »)` |
| `Adopt` | `value` | `onDispose` | `Call(method, undefined, « value »)` |
| `Defer` | `undefined` | `onDispose` | `Call(method, undefined, « »)` |

There is no synchronous `Empty` entry. `use(null)` and `use(undefined)` return
their argument unchanged without appending anything. That differs deliberately
from async disposal, where an empty resource still contributes an `Await`.

The heap words are emitted only through these Rust enums. The disposal walker
derives its comparison chain from `DisposableStackEntryKind::ALL` and obtains
the call shape through an exhaustive `dispose_call` match. Adding an entry kind
without defining its call shape is therefore a Rust compile error.

## Receiver and registration order

Every prototype operation first requires an Object carrying the distinct
`OBJECT_INTERNAL_BRAND_DISPOSABLE_STACK` brand and obtains its attached stack
record. An ordinary object with a lookalike property, the intrinsic prototype,
the constructor, and an `%AsyncDisposableStack%` instance all fail this check.

`use`, `adopt`, `defer`, and `move` then require the state to be `Pending` and
throw `ReferenceError` when it is `Disposed`. This check precedes argument
validation and every observable property access:

1. require the synchronous internal slot;
2. require `Pending`;
3. evaluate the already-supplied arguments into locals;
4. validate or acquire the disposal method; and
5. append one fully initialized entry.

For `use`, nullish values take the no-entry return path. Every other value must
be object-like. `GetMethod(value, Symbol.dispose)` is performed once while the
resource is registered; a missing, nullish, or non-callable result throws
`TypeError`, and a later property mutation cannot replace the acquired method.

Returning a value has one closed backend disposition. The early nullish
`use()` path must return the current function immediately, while the completed
`use()` and `adopt()` paths install the normal result and fall through. Callers
select these two states through a private Rust enum consumed by an exhaustive
match; every caller must name its route and cannot silently transpose an
unlabeled Boolean or omit the choice. This is a local completion-routing
invariant, not the T04
tuple-completion-to-`exnref` redesign.

`adopt` accepts every ECMAScript value but requires a callable `onDispose`.
`defer` requires a callable `onDispose`. Validation completes before the entry
length changes, so an abrupt validation or getter path cannot publish a partial
entry.

## Move ownership

`move` transfers the receiver's whole DisposeCapability to a fresh base
`%DisposableStack.prototype%` instance. It never consults the receiver's
prototype and never invokes a disposer, including when the receiver is a
subclass instance.

The backend expresses the transfer as a private, non-`Copy`, `#[must_use]`
capability witness. Minting the witness snapshots the pointer, length, and
capacity, replaces the source with the canonical empty capability, and sets
the source state to `Disposed`. A single consuming finalizer installs the
snapshot into a freshly allocated pending record and publishes the branded
base instance. The capability cannot be installed twice or silently discarded
without a compiler diagnostic.

The returned stack is `Pending`; the source is permanently `Disposed`. A later
`source.dispose()` is an idempotent no-op, while registration and another
`move()` throw `ReferenceError`.

## Synchronous DisposeResources

`dispose` requires the synchronous brand but does not require `Pending`.
Calling it on an already-disposed stack returns `undefined` without walking the
entries. Otherwise it performs the state transition to `Disposed` before the
first callback. This makes re-entry and repeated disposal no-ops and prevents a
callback from registering or moving resources on the active stack.

The backend snapshots the resource-stack pointer and descending length in a
private, non-`Copy`, `#[must_use]` disposal witness. A single consuming walker
visits entries in strict reverse registration order. Each entry's exhaustive
call shape is used, and the acquired function and resource payload/tag pairs
remain intact until their call.

A throwing callback does not stop the walk. The walker resets the ambient
completion after capturing the thrown value, folds it into a private threaded
completion, and continues with the next entry. The fold is:

- no previous disposal error: retain the thrown value unchanged;
- previous disposal error `P`: allocate
  `SuppressedError(new_error, P)` with no message.

Because the walk is LIFO, three callbacks registered as `E1`, `E2`, `E3` and
all throwing produce
`SuppressedError(E1, SuppressedError(E2, E3))`. After the final entry, the
walker either returns `undefined` or restores the accumulated value as the
function's throw completion. The state remains `Disposed` in both cases.

## Intrinsic surface

All six function objects are non-constructable and have the following native
names and lengths:

| Function | `name` | `length` |
| --- | --- | --- |
| `use` | `"use"` | 1 |
| `adopt` | `"adopt"` | 2 |
| `defer` | `"defer"` | 1 |
| `move` | `"move"` | 0 |
| `dispose` | `"dispose"` | 0 |
| `disposed getter` | `"get disposed"` | 0 |

The string-named methods are writable, non-enumerable, and configurable.
`disposed` is a configurable, non-enumerable accessor with no setter.
`Symbol.dispose` is a writable, non-enumerable, configurable alias containing
the exact same function object as `dispose`, not a separately allocated
wrapper.

## Pinned witness inventory

The complete synchronous lifecycle unlocks exactly 76 deferred
`built-ins/DisposableStack` files at the current Test262 pin:

| Family | Files |
| --- | ---: |
| `prototype/use` | 19 |
| `prototype/adopt` | 12 |
| `prototype/defer` | 11 |
| `prototype/move` | 13 |
| `prototype/dispose` | 13 |
| `prototype/disposed` | 7 |
| `prototype/Symbol.dispose.js` | 1 |
| **Total** | **76** |

Together with the constructor shell's 16 non-dynamic witnesses, this is 92 of
the 93 files under `built-ins/DisposableStack`. The remaining
`proto-from-ctor-realm.js` constructs source through another realm's dynamic
`Function` constructor and remains the explicit Wasm-AOT dynamic-code policy
case described by the constructor contract; it is not a semantic skip hidden
by this batch.

The seven `staging/explicit-resource-management/disposable-stack-*.js` files
are overlapping integration witnesses. In particular,
`disposable-stack-re-entry.js` directly pins the pre-callback disposed-state
transition.

## Wiring and verification boundary

The body and heap representation are only reachable after the standard builtin
catalog, name constants, return-shape inference, arity planner, dependency
closure, dispatcher, intrinsic installer, and string pool are wired as one
batch. The installer must define `dispose` and `Symbol.dispose` through one
function-value allocation and must root `%SuppressedError%` whenever synchronous
`dispose` is reachable.

Central verification owns compilation and runtime execution. The focused
checkpoint is:

```sh
cargo fmt --all -- --check
cargo check -p lila-aot-wasm --lib
cargo test -p lila-ir disposable_stack_ --quiet
cargo test -p lila-aot-wasm disposable_stack_ --quiet
cargo test -p lila-cli --test cli wasm_disposable_stack --quiet
./target/debug/lila test262 run built-ins/DisposableStack --execution-backend wasm --timeout-ms 180000 --threads 1
./target/debug/lila test262 run staging/explicit-resource-management/disposable-stack --execution-backend wasm --timeout-ms 180000 --threads 1
```

The narrower 2026-08-23 value-return routing change passed the capped
workspace `cargo xc` gate, the exact structural lifecycle witness (`1/1`) and
the existing exact CLI lifecycle fixture (`1/1`). That checkpoint proves the
typed caller map and preserves the focused runtime behavior; it did not rerun
the two broad Test262 directories or refresh the 76-file inventory above.
