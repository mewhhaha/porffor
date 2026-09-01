# Async execution Realm authority

An async function's returned Promise and every compiler-owned async
continuation reaction belong to the invoked function's defining Realm. The
dynamic Realm installed while a Promise job happens to run is not allocation
or capture authority.

## Typed boundary

`AsyncExecutionRealmContext` is opaque, non-`Copy` and must-use. Its factories
derive a Realm from exactly three durable sources:

- an invoked async function object;
- the Realm word retained by an ordinary async-function activation; or
- an async-generator activation's retained function object.

The ordinary activation adds one traced `realm` slot. Async generators do not
duplicate it: their existing traced `function` slot leads to the function
header's defining Realm. A missing function or Realm is an internal invariant
failure.

Promise allocation borrows the context and derives `%Promise.prototype%` from
the same Realm intrinsic table. The borrowed copy is consumed by the existing
opaque Promise allocation context; the execution context remains live until
the caller explicitly releases it. Temporary locals are reserved and released
in strict reverse order.

## Covered allocation sites

The boundary owns the four direct allocations that previously read the dynamic
current-Realm global:

1. the Promise returned by an async-function invocation;
2. the rejected Promise wrapping a synchronous fallback-disposer throw;
3. the rejected Promise wrapping a `for-await-of` iterator-close failure; and
4. the rejected Promise wrapping a `for-await-of` iterator-next failure.

The control-flow sites exhaustively select the ordinary async-function or
async-generator activation layout before obtaining a Realm context.

## Captured reactions

Default Promise reactions and compiler-owned async reactions have separate
construction APIs. A default reaction stores the null Realm sentinel and the
job later applies `GetFunctionRealm(handler)`. An async-function or
async-generator continuation stores the Realm derived from its activation.
The shared reaction initializer accepts only the closed typed choice, so it
cannot consult dynamic Realm state or combine a default callback word with a
captured Realm accidentally.

## Explicit deferrals

PromiseResolve catalog selection now borrows this context through the separate
`promise-resolve-realm-context.md` boundary. This contract still does not
change Promise allocation in other async builtins; those producers need their
own durable owner records before their dynamic authority can be removed.
Async-generator request Promise ownership is closed independently by
`async-generator-request-promise-realm.md`.
It also does not change callback-created AggregateError and `allSettled`
result-object ownership, which is covered by the adjacent Promise internal-
function contract.

## Focused verification

The bounded structure target pins the activation layout, opaque lifecycle,
three Realm factories, four direct allocation sites, split reaction APIs and
reverse-order releases. The finite CLI fixture invokes entry-Realm async and
async-generator functions from a created-Realm Promise handler, observes
entry-Realm resolving functions and post-resume TypeErrors, and drains both
continuation paths. It does not poll, block, use Atomics or create an unbounded
job chain. Its request-Promise assertion is also consumed by the adjacent
async-generator request Realm contract, which supplies the distinct catalog and
method-ownership structure guard.

```sh
cargo test -p lila-aot-wasm --test async_execution_realm_structure --quiet
cargo test -p lila-cli --test cli run_wasm_backend_uses_async_function_realms_for_promises_and_reactions --quiet
./scripts/check-module-boundaries.sh
```

The consolidated semantic golden also passes `2/2` in 677.52 seconds. Its 663
dumps add only the three focused callback/async Realm witnesses, remove none
and preserve all 660 retained structural summaries after expected code-size
and local-accounting fields are normalized. This evidence does not complete
T06, T14, the current-pin Promise/async aggregate or the deferred allocation
families above.
