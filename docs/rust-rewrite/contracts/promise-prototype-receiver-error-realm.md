# Promise prototype receiver TypeError Realm

Status: implemented with focused source and finite runtime verification on
2026-08-26.

## Ownership

`Promise.prototype.then` requires a Promise receiver. Its incompatible-receiver
TypeError is created in the executing method's Realm. `Promise.prototype.finally`
first requires an Object receiver, and that direct validation TypeError has the
same owner. Borrowing either self-backed method from a created Realm must not
make the entry Realm, receiver, Promise constructor or active job the error
Realm authority.

The receiver checks remain before argument loading and SpeciesConstructor.
`then` performs its Promise brand check before acquiring error authority;
`finally` performs its Object classification before acquiring it.

## Typed boundary

The private `builtins/promise/promise_prototype_receiver_type_error.rs` owner
contains the non-`Copy`, must-use
`PromisePrototypeReceiverTypeErrorPrototypeLocal`, its only factory and its
consuming throw emitter. The two methods are visible only to the parent Promise
family. The parent may pass their inferred proof for the `then` and `finally`
receiver failures, but cannot name, import, re-export, construct or project its
raw prototype local. The factory explicitly selects the entry TypeError
prototype for a zero standard-builtin environment. A nonzero environment loads
the self-backed TypeError-prototype snapshot published on the executing method
and traps if the slot is absent. It does not consult `CURRENT_REALM`, a Promise
constructor or a defining-Realm fallback.

The child-private `PromisePrototypeReceiverError` is the closed two-variant
selector for the direct `then` and `finally` receiver failures. Its raw
consumer is private even to the parent Promise module. The parent can invoke
only the two named `then`-incompatible and `finally`-non-object wrappers, while
the child alone selects the diagnostic, creates the TypeError from the proved
prototype and releases the local before returning either success or
`EmitError`. The domain does not implement equality, so neither the parent nor
a later receiver-error family can construct a raw selector or bypass its
exhaustive diagnostic projection.

## Delegated `then` boundary

`Promise.prototype.catch` now performs ToObject with the executing method's
Realm. Nullish receivers therefore use its TypeError prototype, and primitive
wrappers use its intrinsic prototype catalog. The subsequent `then` property
read has an immediate abrupt-completion checkpoint before the value can enter
callability validation.

Both `catch` and `finally` pass the retrieved method and original receiver into
the private `builtins/promise/promise_prototype_then_invocation.rs` owner. Its
non-`Copy`, must-use `ValidatedPromisePrototypeThenInvocationLocals`, one
two-caller validator and one two-caller consumer move as a unit. The parent can
pass the inferred carrier between child methods, but cannot name, import,
re-export, construct or project its private method/receiver pair. The validator
uses the shared Proxy-aware IsCallable operation and creates a non-callable
TypeError in the executing method's Realm. Its sole consumer performs the
two-argument delegated Call with the original receiver through the shared
Function-or-Proxy path. Thus a non-callable value cannot enter Call and an
already validated method cannot be paired with a different receiver.

## Focused evidence

The recursive source test pins the private child, sole proof, construction,
projection and method ownership, explicit zero-environment entry selection,
strict nonentry snapshot trap, closed receiver-error domain, both exact
two-caller censuses, lookup checkpoint and one-shot delegated Call. The finite
CLI witness covers the direct receiver errors plus borrowed `catch` and
`finally` non-callable `then` errors, created-Realm primitive wrapping, an
abrupt `then` getter and a callable Proxy with its exact receiver and arguments.
The source guard also pins the shared Call path's four existing error branches
and their order, without assigning their later Proxy-generated errors to this
Promise boundary.

The source-equivalent owner move selected the exact two-line proof block at
SHA-256
`f674791ba2a55602068a89bda2c001417a2bf38c369f69bafb3ba5391d4a8ee9`
and the exact 47-line method block at SHA-256
`da7c455d62b10e71264034444dd59ff767c7b079384fffa0454e8f39212de998`.
Their combined 49 selected lines retain SHA-256
`9156f4cd0689dd3c78bdedfbf1fb356c32e0b2f179f5103a4ace7e3d2fa2f457`.
The resulting 54-line child has SHA-256
`8e6935089851bc5087be8dbfc019b88b0c644d03a6c9183462aec2b0dbc4ad80`,
and the parent decreases from 9,558 to 9,508 lines. Method and caller bodies are
unchanged; only child ownership and parent-facing `pub(super)` spelling were
added. The recursive structure target passes `8/8`; module boundaries,
task-plan policy, focused formatting and diff hygiene are green. The exact
created-Realm Promise internal-callback CLI witness passes `1/1`, and the
shared `cargo xc` checkpoint is green with only the workspace's existing
warnings. Semantic goldens were not rerun for this source-equivalent owner
move.

The diagnostic-policy closure selects the exact 14-line enum and exhaustive
message projection at SHA-256
`475de0cb7a31556b182f6e705f8c5b64cbf33f1bd4d267bc883c3b028b7ca1f8`
and retains that hash after moving the block into the private child. The two
named semantic wrappers add 31 lines at SHA-256
`0a470e691356d67011c025864045b7f6d5767ed814dfc7e39742ba85ef4c1a5a`.
The resulting 101-line child has SHA-256
`f8d25e1f5a4950fb1e01abde37a0bce0048af500fc1c65af3bd4054110605719`,
and the concurrent parent decreases from 7,608 to 7,591 lines after removing
the domain and its two raw selector arguments. The raw throw body and both
diagnostics remain unchanged. The recursive guard and module-boundary audit
now pin sole private domain ownership, exhaustive projection, the private raw
consumer, the two semantic wrapper-to-variant mappings and their one parent
caller each. Batch P ran only non-compiling source checks; the focused
structure target now passes `8/8`, the created-Realm CLI witness passes `1/1`,
and the shared `cargo xc`, formatting, diff, module-boundary and task-plan
checks are green. Semantic goldens were not rerun for this source-equivalent
move.

The delegated-then owner move selected the exact five-line carrier block at
SHA-256
`5d76c430bb3d979c257021485616e144a97ed64a56543d25fb2e305b92ae3e0e`
and the exact 45-line method block at SHA-256
`42107b7cb4e722fb213841ee3c72cf350a839562e979da591f95affee35eca02`.
Their combined 50 selected lines retain SHA-256
`294e1d92a3579b3cab9abd0625d501044d1c03fb13978a5193b1d0719a4fe89d`.
The resulting 55-line child has SHA-256
`a630f2c5aeb0af045a090078e891a552226701ea7dbbf88c1e074bdb842d192a`,
and the parent decreases from 9,508 to 9,457 lines. Method and caller bodies are
unchanged; only child ownership and parent-facing `pub(super)` spelling were
added. The recursive structure target passes `8/8`; module boundaries,
task-plan policy, focused formatting and diff hygiene are green. The exact
created-Realm Promise internal-callback CLI witness passes `1/1`, and the
shared `cargo xc` checkpoint is green with only the workspace's existing
warnings. Semantic goldens were not rerun for this source-equivalent owner
move.

```sh
cargo test -p lila-aot-wasm --test promise_prototype_receiver_error_realm_structure --quiet
cargo test -p lila-aot-wasm --test created_realm_promise_publication_structure --quiet
cargo test -p lila-aot-wasm --test promise_species_realm_context_structure --quiet
cargo test -p lila-cli --test cli run_wasm_backend_preserves_created_realm_promise_internal_callbacks --quiet
./target/debug/lila --jobs 1 test262 run built-ins/Promise/prototype/catch/this-value-non-object.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila --jobs 1 test262 run built-ins/Promise/prototype/catch/this-value-then-not-callable.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila --jobs 1 test262 run built-ins/Promise/prototype/catch/this-value-then-poisoned.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila --jobs 1 test262 run built-ins/Promise/prototype/finally/this-value-then-not-callable.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
```

All three structure executables pass `19/19`, the focused CLI fixture passes
`1/1`, and the four exact leaves pass all `8/8` sloppy/strict executions. The
shared workspace golden passes `2/2` in 800.46 seconds with 679 dumps. It adds
only the independent Array.fromAsync result-definition Realm witness and
removes none; this expanded Promise fixture is the sole structural change
among 678 retained dumps, while the other 677 are equal after accounting
normalization.

## Deferrals

The shared Call operation remains authoritative after validation. In
particular, errors produced while invoking a callable Proxy, including revoked
Proxy and `apply`-trap failures, remain T11 work rather than being recast as
Promise method errors. PromiseResolve and its NewPromiseCapability errors are
closed by the adjacent `promise-resolve-realm-context.md` boundary.
AggregateError remains independent work.
