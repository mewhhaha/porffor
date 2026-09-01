# Bound functions capture `this` exactly

## Semantic boundary

`Function.prototype.bind` stores its `thisArg` without conversion. Calling the
resulting bound function forwards that stored ECMAScript value to the target's
`[[Call]]`; only the target call protocol may then apply strict or sloppy
`this` binding.

That separation is observable:

- a strict target receives a primitive unchanged;
- a sloppy target converts a primitive to an Object when the target is called,
  producing a fresh wrapper for each call;
- an ignored bound `this` value, including a Symbol or BigInt, cannot throw
  merely because the bound function is created; and
- nullish substitution and primitive boxing belong to the target's
  `[[ThisMode]]` and realm, not to the realm that evaluated `bind`.

This is the split between `Function.prototype.bind`, `BoundFunctionCreate`, a
bound function exotic object's `[[Call]]`, and `OrdinaryCallBindThis`.

## Defect closed

The former bind emitter loaded argument zero and immediately passed it through
`emit_adapt_call_this_arg`. That helper boxed Number, String, and Boolean
values during bound-function creation and rejected every other non-object
primitive. The converted pair was then stored as `[[BoundThis]]`.

The common function-call emitter already inspects the target function's strict
flag. It preserves an exact `thisArgument` for a strict target and performs
nullish substitution or `ToObject` for a sloppy target. Eager bind-time
adaptation therefore duplicated the later authority while changing its input:
strict functions observed wrapper objects, sloppy functions reused one wrapper
across calls, and Symbol or BigInt values could fail before the target ran.

## Closed producer domain

The backend has exactly two current producers of its bound-function record:

1. `Function.prototype.bind` captures builtin argument zero as an exact tagged
   value, including the synthesized undefined value when the argument is
   absent.
2. `Proxy.revocable` uses the same record representation for its hidden revoke
   closure and captures the already-allocated Proxy as a known Object.

A private, exhaustive `ExactBoundThisSource` domain owns those two cases. The
raw `(payload, tag)` allocator is private to
`functions/bound_function_allocation.rs`; sibling modules can call only the two
semantic entry points. Adding another bound-function producer therefore
requires choosing how its exact `[[BoundThis]]` value is obtained rather than
passing an arbitrary, possibly adapted pair.

The private dispatcher reserves the payload/tag locals, materializes the
selected source without conversion, calls the raw allocator, and releases the
locals as one lifecycle. No public constructor exposes either raw local.

## Call-time authority

The bound-function invoker continues to load `[[BoundThis]]` and pass it to the
shared target-call emitter. The target-call emitter remains the sole adaptation
authority:

- strict target: forward the exact payload and tag;
- sloppy nullish target: substitute the supported global object;
- other sloppy primitive target: convert to an Object at call time.

This slice deliberately does not add a second adaptation path to the bound
invoker.

## Durable evidence

The existing `wasm_bind_builtins.js` fixture additionally fixes three
properties:

- a Number bound to a strict function remains a Number;
- Symbol and BigInt values bound to a strict function retain their exact
  primitive identities and do not throw during binding; and
- two calls to one sloppy function bound to a Number receive distinct wrapper
  objects.

A backend structural test fixes the private raw allocator, the exhaustive
two-source domain, the two semantic entry points, deletion of the eager adapter,
and continued strict/sloppy handling in the common target-call path.

The pinned Test262 witnesses are
`staging/sm/strict/15.3.4.5.js` and
`staging/sm/Function/function-bind.js`.

## Deferred verification

Cargo and Test262 remain deferred to the centralized batch while another
low-memory conformance process owns the build/runtime lease. After release,
run:

```sh
cargo fmt --all -- --check
cargo check -p lila-aot-wasm --lib
cargo test -p lila-aot-wasm bound_this_capture --quiet
cargo test -p lila-aot-wasm proxy_revocation_function_metadata_module_validates --quiet
cargo test -p lila-cli --test cli -- --exact functions::run_wasm_backend_succeeds_for_supported_bind_builtin_fixture
./target/debug/lila test262 run staging/sm/strict/15.3.4.5.js --execution-backend wasm --timeout-ms 180000 --threads 1
./target/debug/lila test262 run staging/sm/Function/function-bind.js --execution-backend wasm --timeout-ms 180000 --threads 1
```

## Nonclaims

This seam does not implement bound-function `name` or `length`, callable Proxy
targets for `bind`, bound-function prototype-realm correction, or dynamic
Function construction. The shared call path still uses entry-realm primitive
wrapper prototypes for cross-realm sloppy calls; exact capture is necessary for
the eventual callee-realm fix but does not claim it. This is not complete T09
or current-SHA Test262 closure.
