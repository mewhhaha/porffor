# Promise internal-function Realm context

Promise resolving functions, capability executors, `finally` continuations and
combinator element functions are observable built-in function objects. Their
`[[Prototype]]`, `[[Realm]]`, error construction and Promise-reaction job Realm
must come from one defining Realm even when a created-Realm Promise method is
borrowed by entry-Realm code.

## Representation boundary

The function-object header has a GC-visible `builtin_closure_context` pointer.
Compiler-owned builtin closures keep their algorithm capture there and use a
self environment handle. Realm, error and Proxy operations may therefore read
the function header without interpreting an unrelated Promise record as a
function object.

Both function allocation paths initialize the slot to zero. Promise is the
only writer in this batch, through one materializer that installs the defining
Realm, its `%Function.prototype%`, its TypeError and RangeError prototypes, the
algorithm capture and the self environment before exposing the destination.
Standard combinator element functions pass through one typed wrapper that also
propagates the active Promise combinator's AggregateError prototype snapshot.

## Typed lifecycle

The private
`builtins/promise/promise_internal_function_materialization.rs` owner contains
the non-`Copy`, must-use `PromiseInternalFunctionMaterializationContext`, its
three factories, borrowing materializer, closure-context loader and consuming
release. Rust requires the carrier and methods to be `pub(super)` because the
retained parent callers and sibling PromiseResolve owner use their signatures,
but all four fields remain child-private. The parent has one private import for
those existing inferred callers; no parent or sibling can construct or project
the raw context. One factory derives it from a Promise record's stored Realm for
the resolve/reject pair. Another derives it from the active Promise function,
using the canonical entry Promise constructor only when the standard-builtin
environment is zero. Neither factory reads the dynamic current-Realm global.

Sibling closures borrow the same context. A consuming release returns its
prototype and Realm locals in reverse reservation order. Callback bodies load
their algorithm record only through `builtin_closure_context`. The child also
owns the narrow Realm-intrinsics load used by PromiseResolve, so that sibling
never projects `realm_local` directly.

The private
`builtins/promise/promise_combinator_element_materialization.rs` owner contains
the non-`Copy`, must-use
`PromiseCombinatorElementFunctionMaterializationContext`, its sole active-
function factory, sole borrowing materializer and consuming release. The
carrier couples the shared internal-function context to the same active
Promise method's AggregateError-prototype snapshot. The Promise parent can
pass the inferred carrier between those child-owned operations. Rust requires
the carrier name to be `pub(super)` because those sibling-visible method
signatures expose it, but both fields remain child-private, so the parent
cannot construct or project the raw pair. The recursive source policy forbids
the parent from explicitly naming, importing or re-exporting the carrier.

## Covered functions

The boundary covers all fourteen escaping Promise algorithm closures:

- resolve and reject functions;
- the Promise capability executor;
- ThenFinally, CatchFinally, ValueThunk and Thrower;
- the three keyed-combinator element functions; and
- the four standard-combinator element functions.

The formerly raw `Promise.resolve` call surrogates are now closed independently
by the private `promise_resolve_realm_context.rs` owner documented in
`promise-resolve-realm-context.md`. Its two local resolve-function
materializations consume the same self-backed materializer without widening
this internal-function boundary. The recursive census is five occurrences in
the parent, two in that child, one in the standard-combinator materialization
child, one in the finally-completion child and two in the keyed-combinator
child, preserving eleven total.

Promise reaction jobs already use `GetFunctionRealm(handler)`. Correct closure
headers therefore make the existing job-Realm selection observe the defining
Realm without adding another queue policy.

## Error ownership

Calling a capability executor more than once constructs TypeError through its
self-backed function header. Promise self-resolution constructs TypeError from
the Promise record's Realm directly, so async and ordinary callers cannot
select an ambient Realm accidentally.

The callback-created allocation follow-on consumes this function ownership
without adding Realm fields to algorithm records. Standard and keyed
`allSettled` callbacks derive a private non-copyable Object prototype context
strictly from their self function's defining Realm. `Promise.any` consumes a
separate opaque AggregateError allocation context from either its self-backed
reject-element function or the canonical Promise-constructor fallback for the
empty-input path. Neither context admits a raw prototype or dynamic current
Realm.

Outer standard-combinator result/error Array ownership is closed separately by
`promise-combinator-outer-array-realm.md`. General AggregateError construction
remains separate. Direct async-function/control-flow Promise allocation and
captured-reaction Realm ownership are covered by `async-execution-realm.md`;
PromiseResolve constructor catalogs are covered by
`promise-resolve-realm-context.md`; other async builtins remain deferred.

## Focused verification

The bounded structure target pins the heap slot, both zero-initializers, typed
factory authority, reverse-order local lifecycle, all materialization and load
sites, error routes and the existing `GetFunctionRealm` path. The finite CLI
fixture synchronously captures all fourteen function objects, checks the two
TypeError routes, and drains one self-resolution rejection reaction. It does
not wait, poll Atomics or create an unbounded Promise chain.

The combinator-element owner move selects the original exact five-line private
carrier at SHA-256
`c7430a277ef2c67049ff8e71f75889af4a80580649a3b9d0e57c63e3197f2e3c`
and exact 35-line factory, 23-line borrowing materializer and seven-line
release blocks at SHA-256
`cb48555544e8ef28bfb0d7663e43f8fdb81303c13fa5c1cd11812ac03e660d07`,
`b51507411423930036d4aa3a74f88e670414c47dbf9f198d8100e4a09b340f2c`
and
`63d45e822fadada44a6b05de7791dcf0be7937d87194872628715637f10bd35f`.
The relocated carrier's required `pub(super)` spelling changes its raw hash to
`c0270ce24ae08522ec895d3781adc132e4b0ef2b1e109122c067d30c99ab47f6`;
normalizing only that visibility restores the original selected hash. Each
method likewise retains its original hash after normalizing only the required
`pub(super)` sibling visibility. The resulting 77-line child has SHA-256
`ca669fc19647144028e80761f7d62015f1ec3f1c71d6a2d960178e3a7aa91cf1`
and reduces the concurrent Promise parent from 7,561 to 7,488 lines. The
recursive `promise_callback_created_allocation_realm_structure` guard pins
five child-only carrier mentions, exact sibling-only type visibility, private
fields, sole construction, exact two raw-field projections each, the `1/2/1`
parent caller census and zero parent name, import or re-export paths. Batch R
used only non-compiling source checks during implementation. At the coordinated
checkpoint, `promise_callback_created_allocation_realm_structure` passes `7/7`,
`promise_internal_function_realm_context_structure` passes `6/6` and
`promise_resolve_realm_context_structure` passes `4/4`, for `17/17` focused
structure checks. The exact
`functions::run_wasm_backend_uses_callback_realms_for_promise_created_allocations`
CLI witness passes `1/1`, and `cargo xc` is green after the required
`pub(super)` carrier-privacy correction. Semantic goldens were not rerun for
this source-equivalent owner move.

The later PromiseResolve Realm-context owner checkpoint preserves that
historical evidence and retargets this materializer census across its new
private child. `promise_resolve_realm_context_structure` and
`promise_resolve_realm_authority_ownership_structure` each pass `4/4`, while
this contract's `promise_internal_function_realm_context_structure` target
passes `6/6`, for `14/14` focused structure checks. The exact
`functions::run_wasm_backend_uses_callback_realms_for_promise_created_allocations`
CLI witness passes `1/1`, and `cargo xc` is green. Semantic goldens were not
rerun because the owner move is source-equivalent.

The Batch AG owner move selects the original exact 14-line carrier and 179-line
six-method lifecycle at SHA-256
`f4e235ec31396e2a6d937505c6ccaa59fe7450498b262f72060889a017ca057d`
and
`39e984084505b37e9c3c95b73a0a4f05bce82ed8e020cee738b42138b5cbe2ce`;
their combined 193-line selection has SHA-256
`642f27686f144b24412a13e6235b174059a7b8751d3b827715b70fe2b82773f5`
and visibility-normalized SHA-256
`45b615e0aad9e0deb0c63a620408304a0c2729a8c5471985dc39471798453ef3`
after relocation. The former exact six-line PromiseResolve raw Realm projection
has SHA-256
`77ad4ff7cf2a1b826b9a8093df2deb6aab2ff8919860651d2dab6d944e59320d`;
its replacement five-line call to the child-owned capability has SHA-256
`a9b67364fab3ee72ee79d4f3421c4975e96a7135b6a0a88e98ad6f67996e3812`
and emits the same Realm-intrinsics load. The resulting 212-line child has
SHA-256
`18506619c5365cbb4354ead3759f98cc235a6e8581432f8f4c7235fc45039556`,
while the concurrent 7,111-line parent has SHA-256
`eea99601f506ffa59e1870607a04e24481b5ea83349aedb0ee439ff273d617f6`.
The recursive policy pins eleven carrier identifiers and the exact
`4/7/2/11/9/9/2` factory/materializer/load/release/capability censuses. At the
coordinated Batch AG checkpoint, the internal-function, PromiseResolve
Realm-context and callback-created-allocation structure targets pass `6/6`,
`4/4` and `7/7`, for `17/17` focused structure checks. The exact
`functions::run_wasm_backend_preserves_created_realm_promise_internal_callbacks`
and
`functions::run_wasm_backend_uses_callback_realms_for_promise_created_allocations`
CLI witnesses each pass `1/1`, and shared `cargo xc` is green. No Test262
cohort or semantic golden was run because this is a source-equivalent owner
move.

The intended focused commands are:

```sh
cargo test -p lila-aot-wasm --test promise_internal_function_realm_context_structure --quiet
cargo test -p lila-cli --test cli run_wasm_backend_preserves_created_realm_promise_internal_callbacks --quiet
cargo test -p lila-aot-wasm --test promise_callback_created_allocation_realm_structure --quiet
cargo test -p lila-cli --test cli run_wasm_backend_uses_callback_realms_for_promise_created_allocations --quiet
./scripts/check-module-boundaries.sh
```

This evidence does not complete T06, T14, the full Promise subtree, the
current-pin aggregate or the Wasm golden corpus.
