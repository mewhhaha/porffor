# Runtime-error active-handler routing

## Status and evidence boundary

This contract owns fresh runtime errors created inside ordinary non-resumable
functions while an in-function `try` or `finally` target is active. At clean
commit `22ab459107`, the current-pin Test262 witness

`built-ins/Object/preventExtensions/15.2.3.10-3-4.js`

reports `1/2` under Wasm AOT: the sloppy execution passes, while the strict
execution leaks the expected array-index assignment `TypeError` as an uncaught
completion instead of entering `propertyHelper.js`'s `catch`.

`Object.preventExtensions` and the array's non-extensible state are not the
failure. The complete `built-ins/Object/preventExtensions` leaf was `77/78`
after the typed Proxy `[[PreventExtensions]]` batch, with this exact strict
execution as its sole remaining failure. The source witness calls
`verifyNotWritable`, whose user function performs the strict failed assignment
inside `try` and catches `TypeError`.

The failure is current-source-proven independently of that measurement.
`emit_throw_runtime_error_to_active_handler` constructs a Throw completion and
then returns immediately whenever the current Wasm body is not the main export.
It consults `active_throw_target` only for the main export, so a user function's
in-function handler is unreachable from every runtime error created through
that wrapper.

## Normative lifecycle

For a fresh runtime error whose policy is "to active handler":

1. Allocate the current-Realm native error object and publish its payload and
   tag as the current result.
2. Set the current completion to Throw, preserving the error identity and
   auxiliary error-name payload.
3. If an in-function catch or finally target is active, branch to the innermost
   applicable typed `ControlTarget` regardless of whether the current body is
   the main export or an ordinary user function.
4. Only when no handler or finalizer is active may the Throw completion return
   from the current function through its selected return ABI.

No statement or expression after creation may observe the fresh runtime error
as a normal value. A nested user function is not a reason to bypass a handler
owned by that same function.

## Closed Rust seam

`emit_propagate_current_throw` is the canonical routing operation for an
already-published Throw completion. It consumes the only relevant closed
choice: `active_throw_target() -> Option<ControlTarget>`. `Some(ControlTarget)`
branches through the target's compiler-recorded Wasm label; `None` returns the
current completion according to the function's ABI.

`emit_throw_runtime_error_to_active_handler` must perform exactly two actions:

1. call `emit_throw_runtime_error` to create and publish the fresh error;
2. delegate routing to `emit_propagate_current_throw`.

It must not duplicate a second main-versus-user or branch-versus-return
decision. The existing `ControlTarget` identity is the compile-enforced seam:
callers cannot supply a raw branch depth, and changes to target representation
must update the canonical routing operation. A new request type around this
single fixed-policy wrapper would not reject an additional plausible mistake,
so this batch does not add a decorative carrier.

Emitters that intentionally leave a Throw in the completion tuple or return it
from an outlined runtime helper retain their separate, explicitly named
routing APIs. They do not call this active-handler wrapper.

## Scope and nonclaims

The product change is limited to
`crates/lila-aot-wasm/src/builtins/errors.rs`. The array assignment path in
`builtins/array.rs` and the strict ordinary-object write guards in `objects.rs`
remain consumers of the corrected wrapper; they do not need a second repair.

This batch does not alter `Object.preventExtensions`, array extensibility,
ordinary `[[Set]]`, strictness selection, Proxy operations, typed-array writes,
or object-literal method `[[HomeObject]]`.
It does not claim every throw/catch site or the complete Test262 matrix.
Resumable generator and async completion transport remains outside this bounded
non-resumable route.

## Verification ladder

Cheap implementation checks:

```sh
cargo fmt --all -- --check
git diff --check
./scripts/check-module-boundaries.sh
```

Central focused verification after the product and evidence lanes are
assembled:

```sh
cargo check --workspace --all-targets
cargo test -p lila-aot-wasm --test runtime_error_active_handler_structure
cargo test -p lila-cli --test cli \
  run_wasm_backend_succeeds_for_object_prevent_extensions_missing_writes_fixture

./target/debug/lila test262 run \
  built-ins/Object/preventExtensions/15.2.3.10-3-4.js \
  --execution-backend wasm --timeout-ms 120000 --threads 1

./target/debug/lila test262 run built-ins/Object/preventExtensions \
  --execution-backend wasm --timeout-ms 120000 --threads 4
```

The exact witness must report `2/2`, and the complete current leaf must report
`78/78`, with zero unsupported, crash, timeout, or runtime-failure outcomes.
Broader workspace and current-pin verification remains the centralized final
gate.
