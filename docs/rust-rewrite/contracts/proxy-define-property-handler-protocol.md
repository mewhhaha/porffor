# Proxy `[[DefineOwnProperty]]` handler protocol

Status: implemented and verified bounded T11 handler-acquisition contract.
Pre-fix evidence was captured at commit
`d412ca624be8fa3eba974b05274775d8165522eb`.

## Scope

This contract owns only the handler-acquisition half of Proxy
`[[DefineOwnProperty]]`: validating a live Proxy, retaining both tagged internal
slots, performing `GetMethod(handler, "defineProperty")`, and either calling
that method or forwarding the completed descriptor to the target. The existing
`ToPropertyDescriptor`/`FromPropertyDescriptor` adapters, Boolean result
handling and post-trap target invariants remain separate consumers.

The two public owners are:

- `Object.defineProperty`, emitted by
  `compile_object_define_property_builtin`; and
- `Reflect.defineProperty`, emitted by
  `compile_reflect_define_property_builtin`.

Receiver-side assignment to a Proxy also reaches the Reflect owner through the
ordinary receiver-set path. The boundary is therefore an internal-method
product path, not only an explicit builtin call.

## Pre-fix defect

Both public owners previously read `[[ProxyHandler]]` as a payload, fabricated
`ValueKind::Object` as its tag and performed an ordinary-only read of
`"defineProperty"`. That loses the representation of Function, Array and
arguments handlers before either `GetMethod` or Call.

Focused Wasm-AOT probes at the pre-fix commit observed all three failures:

- a Function handler used through `Object.defineProperty` failed its exact
  accessor and trap `this` checks;
- an Array handler used through `Reflect.defineProperty` failed the same
  checks; and
- an arguments handler used through `Object.defineProperty` failed the same
  checks.

A load-bearing Function control additionally observed
`typeof getterThis === "object" && getterThis !== functionHandler`, proving
that the retained heap payload was paired with the wrong tag. A simple Proxy
handler lookup and an abrupt Function-handler getter already preserved their
observations in the probes; those controls are not claimed as pre-fix
failures.

## Typed acquisition boundary

One shared `emit_proxy_define_property_trap_result` emitter owns acquisition
for both public callers. Its boundary carries complete roles rather than raw
positional payload/tag integers:

- `TaggedLocals` for the current traversal object;
- `ProxySlotLocals`, whose private fields are the distinct
  `ProxyTargetLocals` and `ProxyHandlerLocals` roles;
- `PropertyKeyLocals` for the canonical String-or-Symbol property key;
- `TaggedLocals` for the completed descriptor object;
- `TaggedLocals` for the prospective trap; and
- `TaggedLocals` for the trap result.

The target and handler cannot be transposed at a call site, and neither tag can
be omitted. The helper converts the internal property-key payload to the exact
JavaScript key value before Call, so a Symbol trap argument never exposes the
internal property-key marker.

The handler-payload word may be read directly only to classify the current
Object as a Proxy. Once that branch is entered,
`emit_load_live_proxy_slots` is the sole authority that reads
`[[ProxyTarget]]` and `[[ProxyHandler]]`. It runs once per traversal iteration
with `ProxyRevocationRoute::CurrentFunctionRealm`.

After acquisition, each caller retains its own specified result behavior:

- `Object.defineProperty` throws when the trap result is false and otherwise
  returns its original target; and
- `Reflect.defineProperty` publishes the Boolean trap result.

Both callers retain their existing call to
`emit_proxy_define_property_trap_invariants` after a truthy result. Neither may
keep a second inline `"defineProperty"` lookup, raw slot reconstruction or
ordinary-only handler read.

## Observable order

For every Proxy reached while following a nullish trap fallback, emission
preserves this order:

1. reject a revoked Proxy in the called builtin's Function Realm;
2. retain the exact tagged target and handler;
3. perform Proxy-aware `[[Get]]` of `"defineProperty"` on the handler with the
   exact tagged handler as receiver;
4. route an abrupt lookup completion before callable or nullish
   classification;
5. if the result is `undefined` or `null`, continue with the exact tagged
   target;
6. if the result is not callable, throw a `TypeError` in the called builtin's
   Function Realm; and
7. otherwise call it through the Proxy-aware Call operation with the exact
   handler as `this` and the exact target, property key and completed
   descriptor as its three arguments, in that order.

Retaining the handler tag is observable even when the payload address is
unchanged. Function, Array and arguments handlers select different storage and
identity behavior. A Proxy handler must observe its own `[[Get]]`, and a
callable Proxy trap must enter Proxy `[[Call]]` rather than a Function-only
path.

The property read leaves abrupt completion in its output locals. The shared
emitter routes that completion before `IsCallable`, so a getter's thrown value
cannot be replaced by a generated non-callable `TypeError`. Trap-call
completion is likewise routed before `ToBoolean` or any descriptor invariant.

## Focused runtime boundary

The source-free fixture and its live exact CLI owner cover both Object and
Reflect entry points. Their load-bearing scenarios are:

- Function, Array and arguments handlers with exact getter and trap receivers;
- a Proxy handler whose own `get` trap observes the key and receiver;
- a callable Proxy `defineProperty` trap;
- exact target, key and completed-descriptor arguments;
- an abrupt lookup sentinel;
- nullish fallback to a nested Proxy target; and
- created-realm revoked and non-callable errors.

The structural owner pins one typed acquisition, the sole live-slot read,
Proxy-aware lookup and Call, completion-before-classification order, exact
handler-as-receiver/`this`, both public consumers, absence of a fabricated
Object handler tag, and an active unignored Wasm CLI registration.

The 2026-08-25 coordinated verification passed every focused gate described
above.

## Raw Test262 cohort

At vendored suite content tree
`aa55200d1310384c5cf69ea95b2a2ecba457007b`, the smallest direct cohort is five
physical files:

- `built-ins/Proxy/defineProperty/call-parameters.js`;
- `built-ins/Proxy/defineProperty/return-is-abrupt.js`;
- `built-ins/Proxy/defineProperty/trap-is-not-callable.js`;
- `built-ins/Proxy/defineProperty/trap-is-not-callable-realm.js`; and
- `built-ins/Proxy/defineProperty/trap-is-undefined-target-is-proxy.js`.

None has a strictness-limiting flag, so each expands to ordinary sloppy and
strict Script execution: exactly ten variants. None is one of the paths
recognized by `rewrite_proxy_define_property_case`. Each exact path must run
separately through Wasm-AOT with `--jobs 1`, `--threads 1` and the repository
timeout, and verification must inspect the discovery count plus every failure
and non-success bucket.

All five files and ten variants pass under Wasm-AOT at the 2026-08-25
checkpoint, with every failure and non-success bucket at zero.
The complete current leaf has 24 physical files / 48 ordinary executions, and
three different paths still have exact materializer rewrites:

- `trap-is-undefined.js`;
- `trap-is-null-target-is-proxy.js`; and
- `return-boolean-and-define-target.js`.

Consequently even a path-counted full-leaf result would not establish raw
source closure until those rewrites are removed and the original sources are
verified.

## Verification ladder

The verification commands are:

```sh
cargo fmt --all -- --check
git diff --check
cargo check -p lila-aot-wasm

cargo test -p lila-aot-wasm --test proxy_define_property_handler_protocol_structure -- --test-threads=1
cargo test -p lila-cli --test cli object::run_wasm_backend_succeeds_for_proxy_define_property_handler_protocol -- --exact --test-threads=1

./target/debug/lila --jobs 1 test262 run <exact-path> \
  --suite-root test262/vendor/test262 \
  --execution-backend wasm-aot \
  --timeout-ms 180000 \
  --threads 1
```

The final command ran once for each of the five exact paths above. The
2026-08-25 checkpoint passed the structure target at `4/4`, the exact CLI test
at `1/1`, and the raw Test262 cohort at `10/10`, all with zero non-success
buckets. `cargo check -p lila-aot-wasm`, formatting and diff hygiene were green
at the same focused checkpoint.

## Explicit nonclaims

This checkpoint does not implement the recursive Proxy descriptor-record
protocol. Nested Proxy targets still require complete recursive
`[[GetOwnProperty]]`, `GetMethod`, Call and
`IsCompatiblePropertyDescriptor` behavior, including module-namespace exotic
descriptors.

It does not remove `rewrite_proxy_define_property_case`, verify the complete
24-file/48-variant leaf from raw source, or update snapshots and published
conformance counts. It does not close the T10 descriptor-lattice obligations,
including the retained `Presence::Present` compatibility debt, or change the
existing descriptor conversion and post-trap invariant algorithms.

The unrelated Proxy `[[Get]]` and `[[Set]]` acquisition paths remain separate
T11 work. The cross-realm descriptor-object allocation exercised by
`built-ins/Proxy/defineProperty/desc-realm.js` is also outside this acquisition
checkpoint. This bounded seam does not complete T10, T11, Proxy, Reflect or the
full pinned Test262 matrix.
