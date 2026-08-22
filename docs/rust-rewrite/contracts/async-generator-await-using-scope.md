# Async-generator `await using` scopes

## Evidence and scope

At source commit `5ad393f3d0` and pinned Test262 revision
`aa55200d1310384c5cf69ea95b2a2ecba457007b`, these exact files report `0/4`
under Wasm AOT:

```text
language/statements/await-using/initializer-Symbol.asyncDispose-called-at-end-of-asyncgeneratorbody.js
language/statements/await-using/initializer-Symbol.dispose-called-at-end-of-asyncgeneratorbody.js
```

All four sloppy/strict Script executions are `Runtime/NotImplemented` with the
diagnostic `await using declaration in an async generator`. The parser and
binding analysis accept both declarations, and neither path has a Wasm-AOT
source rewrite, mask, or known-failure entry. This contract owns only this
two-file, four-execution delta.

The supported source domain is an ordinary statement-list `await using`
declaration owned by an async generator whose existing suspension plan can
represent the body. It covers the direct `@@asyncDispose` protocol and the
`@@dispose` fallback, and permits `yield` and `await` after acquisition while
the scope remains live.

`await using` in classic-`for` and `for-of` resource heads, modules, dynamic
source, binding patterns, explicit `await` or `yield` inside a resource
initializer, and nonlinear async-generator forms rejected by the existing
suspension plan remain nonclaims. Plain async-function behavior remains owned
by the retained plain-async contract.

## Normative lifetime

Calling the async generator creates its activation without evaluating any
resource initializer. The first request that reaches a declaration:

1. publishes one activation-backed async DisposeCapability;
2. evaluates resource initializers in declaration order;
3. for each non-nullish value, performs `GetMethod(value, @@asyncDispose)` and,
   only when absent, `GetMethod(value, @@dispose)`;
4. validates and registers the selected method before initializing its
   immutable lexical binding; and
5. registers an empty async-dispose resource for a nullish initializer.

Neither `yield` nor `await` disposes the retained resources. Normal, return, or
throw completion leaving the scope detaches the capability once and disposes
the registered entries in reverse order before completing the current async-
generator request or draining its queue. A direct `@@asyncDispose` result is
awaited. The `@@dispose` fallback is called by the spec-created async wrapper;
its normal result, including a thenable, is discarded before the wrapper
fulfills with `undefined`, while an abrupt call rejects the wrapper. Disposal
then awaits the wrapper. Rejections become throw completions, and multiple
failures fold through `SuppressedError` in disposal order.

An initializer or method-acquisition failure takes the same exit after every
previously registered resource. Nested scopes own distinct capabilities and
dispose inner before outer.

## Closed IR ownership

Every async-dispose statement-list scope carries one required execution proof:

```rust
StatementIr::AsyncDisposableScope {
    execution: AsyncDisposableScopeExecutionIr,
    resources: AsyncDisposableResourcesIr,
    body: BlockIr,
}

pub enum AsyncDisposableScopeExecutionIr {
    AsyncFunction(AsyncFunctionAsyncDisposableCapabilityIr),
    AsyncGenerator(AsyncGeneratorAsyncDisposableCapabilityIr),
}
```

Both capability types have private fields, are non-`Copy`, and are
`#[must_use]`. They derive `Clone` because `StatementIr` is cloneable, but only
lowering may construct them. `AsyncGeneratorAsyncDisposableCapabilityIr` is
minted only after the exhaustive owner match selects `AsyncGenerator`, from a
hidden binding returned by `alloc_suspension_owned_binding` with the fixed
`async.generator.await.dispose.capability.` prefix. Backend crates may read the
binding identity and finalizer roles but cannot manufacture either owner proof
from an arbitrary `String`.

Both capabilities own an `AsyncDisposableFinalizerPlanIr` whose sole
constructor enforces the ordered roles `entry_state < dispose_state <
resume_state < exit_state`. The roles are shared because both owners perform
the same asynchronous resource protocol; their distinct capability types make
activation layout and completion routing an exhaustive backend decision.

The async-generator resumable-plan collector reserves exactly three implicit
states for every admitted statement-list `await using` declaration, at the end
of that declaration's containing statement list. A nested block therefore
reserves its finalizer before collection continues with a following outer
`yield` or `await`; the containing scope reserves its finalizer only after its
remaining suffix. Multiple declarations reserve one three-state group each,
matching the producer's reverse suffix nesting. Resource-loop heads and
declarations rejected for initializer suspension do not reserve a group.

Lowering consumes those positions monotonically: `dispose_state`,
`resume_state`, and `exit_state` are the three states immediately after the
current unified async-generator state. It advances both the async and generator
state cursors to `exit_state`. The next preplanned source suspension may use
that exit state as its suspend state, but no finalizer state may equal a later
source resume state. `ResumablePlanIr::state_count` includes every implicit
group.

Analysis projects every function kind through
`AsyncDisposableScopeOwnerPlan::{Ordinary, Generator, AsyncFunction,
AsyncGenerator}`. Ordinary and plain-generator owners remain explicit
diagnostics. The two admitted owners must mint different capabilities; there
is no optional capability, boolean resumability flag, or default match arm.

## Producer obligations

The lila-ir producer must:

- select the exhaustive owner before lowering any initializer;
- reject initializer suspension before allocating a capability or finalizer
  state;
- allocate exactly one hidden activation binding per reached scope;
- allocate finalizer states only after the remaining suffix has consumed all
  source suspension states;
- reserve each async-generator finalizer's three implicit states at its exact
  statement-list suffix boundary before lowering consumes the preplanned source
  suspension chain;
- preserve declaration order, registration-before-initialization, non-empty
  resources, and suffix nesting; and
- keep the plain-async and async-generator capability domains distinct at every
  public IR construction site.

The Wasm consumer must match `AsyncDisposableScopeExecutionIr` exhaustively,
select the correct activation and resume-state layout, retain the capability
through both `GeneratorYield` and `AsyncAwait`, and route normal, external
return, external throw, source throw, and awaited rejection through disposal
before request completion.

## Verification

Central integration owns compilation and runtime execution:

```sh
cargo fmt --all -- --check
cargo check -p lila-ir
cargo check -p lila-aot-wasm --lib
cargo test -p lila-ir async_generator_await_using --quiet
./target/debug/lila test262 run \
  language/statements/await-using/initializer-Symbol.asyncDispose-called-at-end-of-asyncgeneratorbody.js \
  --suite-root test262/vendor/test262 --execution-backend wasm-aot \
  --timeout-ms 60000 --threads 1
./target/debug/lila test262 run \
  language/statements/await-using/initializer-Symbol.dispose-called-at-end-of-asyncgeneratorbody.js \
  --suite-root test262/vendor/test262 --execution-backend wasm-aot \
  --timeout-ms 60000 --threads 1
```

Only the two Test262 commands can move the exact result from `0/4` to `4/4`.
This is not a claim about the complete `await using` directory or the pinned
Test262 aggregate.
