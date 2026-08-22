# Plain-async-function `await using` scopes

## Evidence and scope

At source commit `7a89e27ec79fe6210fff04a58b6bb3eace535e09` and pinned
Test262 revision `aa55200d1310384c5cf69ea95b2a2ecba457007b`, these exact
files report `0/4` under Wasm AOT:

```text
language/statements/await-using/initializer-Symbol.asyncDispose-called-at-end-of-asyncfunctionbody.js
language/statements/await-using/initializer-Symbol.dispose-called-at-end-of-asyncfunctionbody.js
```

All four sloppy/strict Script executions are `Runtime/NotImplemented` with the
diagnostic `await using declaration`. The parser and binding analysis already
accept the declarations; lowering rejects them before Wasm code generation.
There is no exact Test262 rewrite or mask for either file.

This contract owns ordinary statement-list `await using` declarations in a
plain async function. It covers the direct `@@asyncDispose` protocol and the
`@@dispose` fallback exercised by the two files above. The other 47 positive
plain-async statement-list files in the same directory remain regression
inventory rather than a claimed result until measured.

Classic-`for` and `for-of` resource heads, modules, dynamic source, `await
using` outside an async function or async generator, and explicit `await` or
`yield` inside a resource initializer are nonclaims. Async-generator ownership
is specified separately by `async-generator-await-using-scope.md`. The syntax
subtree and its negative tests remain parser/early-error work rather than
runtime evidence.

## Normative acquisition and disposal

When evaluation reaches a declaration, resources are processed in declaration
order. For each resource the implementation:

1. evaluates its initializer exactly once;
2. for a non-nullish value, performs `GetMethod(value, @@asyncDispose)` exactly
   once;
3. only when that result is `undefined`, performs `GetMethod(value, @@dispose)`
   exactly once and records the synchronous fallback through the spec-created
   async wrapper;
4. throws `TypeError` when neither method exists or a present method is not
   callable;
5. registers the value, selected method, and async-dispose hint only after all
   validation succeeds; and
6. initializes the immutable lexical binding only after registration.

Nullish values register an empty async-dispose resource. Consequently, an
evaluated `await using` scope still performs an Await at exit; a declaration
that evaluation never reaches registers nothing and does not create a
spurious suspension.

On normal, return, or throw completion, the scope detaches its capability once
and disposes registered resources in reverse order. Each selected method is
called with the resource value as `this`. A direct `@@asyncDispose` result is
awaited. The synchronous fallback wrapper calls `@@dispose`, discards a normal
return value (including a thenable), and fulfills with `undefined`; an abrupt
call rejects the wrapper. Disposal then awaits that wrapper result. An empty
resource likewise awaits `undefined` before the next resource or final
completion. Rejection becomes the current throw completion. Multiple failures
fold through `SuppressedError` in disposal order before the async function
settles.

## Closed IR ownership

The synchronous scope stays closed to synchronous resources. `await using`
uses a separate public IR node and resource domain:

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

pub struct AsyncDisposableResourceIr { /* private fields */ }
pub struct AsyncDisposableResourcesIr { /* private non-empty first/rest */ }
pub struct AsyncFunctionAsyncDisposableCapabilityIr { /* private fields */ }
pub struct AsyncDisposableFinalizerPlanIr { /* private fields */ }
```

`AsyncDisposableResourcesIr` is statically non-empty and can be constructed
only by lowering. Its iterator preserves declaration order and supports reverse
consumption. Each resource entry owns exactly one initializer and exactly one
immutable binding name; there is no disposal-kind boolean and no route into
`SyncDisposableResourceIr`.

`AsyncFunctionAsyncDisposableCapabilityIr` and
`AsyncDisposableFinalizerPlanIr` have private fields, are non-`Copy`, and are
`#[must_use]`. The capability is minted only after an exhaustive function-owner
match selects the plain `Async` execution kind. The required execution enum
keeps this capability distinct from the async-generator capability. It owns the
hidden binding allocated by `alloc_suspension_owned_binding`, so a backend
cannot substitute a temporary local or attach the plan to another owner.

The finalizer plan carries four ordered state roles:

- `entry_state`: evaluation before capability acquisition;
- `dispose_state`: the state that starts or continues the LIFO disposal walk;
- `resume_state`: the state entered after one disposal Await settles; and
- `exit_state`: the state reached only after the stack is empty and the saved
  completion has been restored.

Its sole constructor validates `entry_state < dispose_state < resume_state <
exit_state`. Lowering advances the enclosing async function's state cursor to
`exit_state`; the four roles therefore cannot overlap source Await states. The
backend must consume the named roles rather than deriving adjacent integers.
The disposal cursor, saved completion, and live resource stack belong to the
activation-backed capability across every suspension.

## Producer obligations

The lila-ir producer must:

- select the plain async owner before lowering any initializer;
- reject explicit initializer suspension before allocating the capability or
  finalizer states;
- allocate one distinct hidden capability binding and one distinct finalizer
  plan per reached source declaration;
- preserve the existing statement-list suffix nesting, so a resource exists
  only after its declaration is reached and nested scopes dispose inner first;
- lower initializers in source order without emitting generic lexical
  initialization statements; and
- make every new `StatementIr` consumer choose the async node explicitly.

The resource protocol belongs to the dedicated async resource type. A backend
consumer may share low-level `GetMethod`, resource-stack, Promise, or
`SuppressedError` machinery with `AsyncDisposableStack`, but it may not treat a
lexical capability as that user-visible branded object.

## Verification

Central integration owns compilation and runtime execution:

```sh
cargo fmt --all -- --check
cargo check -p lila-ir
cargo check -p lila-aot-wasm --lib
cargo test -p lila-ir plain_async_function_await_using --quiet
./target/debug/lila test262 run \
  language/statements/await-using/initializer-Symbol.asyncDispose-called-at-end-of-asyncfunctionbody.js \
  --suite-root test262/vendor/test262 --execution-backend wasm-aot \
  --timeout-ms 60000 --threads 1
./target/debug/lila test262 run \
  language/statements/await-using/initializer-Symbol.dispose-called-at-end-of-asyncfunctionbody.js \
  --suite-root test262/vendor/test262 --execution-backend wasm-aot \
  --timeout-ms 60000 --threads 1
```

Only the two Test262 commands can move the exact claim from `0/4` to `4/4`.
This result is not a claim that all 49 plain-async statement-list files, the
complete `await using` directory, or the pinned Test262 matrix are green.
