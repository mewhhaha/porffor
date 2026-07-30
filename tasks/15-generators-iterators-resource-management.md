# T15 — Generators, iterators, iterator helpers and resource management

**Status:** In progress — iterator helpers and generator records exist; general suspension remains

**Parallel group:** Feature lane  
**Depends on:** T04, T05, T08, T09; async portions also depend on T14  
**Blocks:** Iterator-dependent closure across arrays, collections, promises and modules

## Current repository state

Generator activation/delegation records, iterator operations, several iterator
helpers, async-iterator support and resource-management builtins now exist.
Some generator and helper behavior is still handled through focused
materialization, and the README records unsupported suspended/control-flow
families. General sync/async generator state machines, iterator-close coverage
across all consumers and complete resource-management filters remain open.

## Objective

Implement resumable generator execution, the complete iterator protocols, iterator helpers and explicit resource management through reusable state-machine and iterator-operation layers.

## Generator state machines

Lower generator bodies into explicit states with stored environments, operand values, completion records and finally/handler state. Cover:

- generator function/method object creation and prototypes;
- `next`, `return`, `throw`, suspended-start/yield/completed/executing states;
- `yield` and `yield*` delegation, including missing methods and iterator closing;
- re-entrancy errors;
- `try/catch/finally`, return and throw across suspension;
- captured variables, `this`, `arguments`, `super` and private environment;
- async generators by composing the state machine with T14 jobs/promises.

Do not implement generators by interpreting a stored AST at runtime.

## Iterator operations

Complete and centralize:

- `GetIterator`/`GetIteratorFromMethod` for sync and async hints;
- iterator records, `IteratorNext`, `IteratorComplete`, `IteratorValue`, `IteratorStep`;
- `IteratorClose` and `AsyncIteratorClose` with correct completion precedence;
- `%IteratorPrototype%`, `%AsyncIteratorPrototype%` and identity methods;
- array, string, typed-array, Map/Set and custom iterator interoperability.

All consumers (`for-of`, spread, destructuring, Promise combinators, constructors and builtins) must use these operations.

## Iterator helpers

Implement the pinned Iterator/AsyncIterator helper APIs, including lazy helper objects, `map`, `filter`, `take`, `drop`, `flatMap`, `reduce`, `toArray`, `forEach`, `some`, `every`, `find`, helper `return`, close behavior, limits/coercions and branding.

## Explicit resource management

Implement current standardized syntax/builtins present in the pin:

- `using` and `await using` declaration lowering;
- `Symbol.dispose`/`Symbol.asyncDispose`;
- `DisposableStack`, `AsyncDisposableStack` and `SuppressedError` integration;
- LIFO disposal, move/adopt/defer/use, abrupt-completion chaining and async disposal jobs.

Coordinate syntax/early errors with T07 and error objects with T24.

## Acceptance criteria

- Generator protocol/state/re-entrancy tests pass.
- `yield*` handles sync and async delegates, return/throw absence and close precedence.
- All iterator consumers close iterators on the exact required abrupt paths.
- Iterator helpers are lazy and pass mutation/close/branding tests.
- Explicit resource management preserves suppression order and disposal timing.
- Generator/iterator objects remain valid across GC cycles.
- Pinned generator, iterator-helper and resource-management filters reach zero failures.

## Required tests

```sh
cargo test -p porffor-ir generator_ --quiet
cargo test -p porffor-aot-wasm iterator_ --quiet
cargo test -p porffor-cli wasm_iterator --quiet
./target/debug/porf test262 run built-ins/Iterator --execution-backend wasm --timeout-ms 120000 --threads 4
```

Also run language generator/`yield`, `for-of`, spread/destructuring, AsyncIterator, DisposableStack, AsyncDisposableStack and explicit-resource-management filters.
