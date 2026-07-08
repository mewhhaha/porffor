# T04 — Shared ECMAScript operations and completion ABI

**Status:** Ready after initial T02 boundaries  
**Parallel group:** Foundation  
**Depends on:** T02  
**Blocks:** Most semantic feature tasks

## Objective

Create one spec-shaped implementation path for common ECMAScript abstract operations and one uniform ABI for normal/throw/return/break/continue completions. Remove feature-local copies whose subtle differences cause evaluation-order, proxy, realm and abrupt-completion failures.

## Required operation families

### Conversion and comparison

- `Type`, `IsCallable`, `IsConstructor` and `IsPropertyKey`.
- `ToPrimitive` with correct hint and `@@toPrimitive` ordering.
- `ToBoolean`, `ToNumeric`, `ToNumber`, `ToBigInt`, `ToString`, `ToObject` and `ToPropertyKey`.
- `ToIntegerOrInfinity`, `ToLength`, `ToIndex`, integer/uint conversions and clamping.
- `SameValue`, `SameValueZero`, strict equality, abstract equality and abstract relational comparison.

### Object and invocation operations

- `Get`, `GetV`, `Set`, `HasProperty`, `HasOwnProperty`, `DeletePropertyOrThrow`.
- `CreateDataProperty`, `CreateDataPropertyOrThrow`, `DefinePropertyOrThrow` and descriptor conversion.
- `GetMethod`, `Call`, `Construct`, `OrdinaryCreateFromConstructor`, `SpeciesConstructor` and `ArraySpeciesCreate`.
- Iterator acquisition/step/value/close operations, with sync/async variants exposed for T14/T15.

### Completion model

Define a Rust representation and Wasm calling convention for:

- normal value;
- throw with value and realm-correct error identity;
- return;
- break/continue with optional target;
- empty completion and completion-value updates.

The convention must work across user functions, builtins, proxy traps, host imports and nested `try/finally` without relying on unstructured scratch globals.

## Design constraints

- Operations must preserve observable order and stop immediately on abrupt completion.
- Object operations must dispatch through the internal-method protocol from T10; static-shape fast paths require guards proving no observable trap/accessor/prototype difference.
- Avoid a runtime interpreter. These are compiler-emitted helpers or specialized Wasm functions generated from typed operation IR.
- Design the Wasm-level completion convention from the experimental Wasmtime lower bound: `exnref` exception handling, typed function references and reference types are available and may carry throw/abrupt paths. Do not maintain a second completion mechanism for runtimes that lack them.
- Keep operation signatures stable enough for feature modules to depend on them. Version or feature-gate ABI changes rather than silently changing tuple layout.
- Emit structured diagnostics when an operation cannot yet lower; do not panic.

## Implementation sequence

1. Write a catalog mapping operation name to spec inputs, outputs and possible abrupt completions.
2. Introduce typed operation nodes/helpers in `porffor-ir`.
3. Introduce shared Wasm helper generation and a registry that emits each helper once per module.
4. Convert two representative families first: property access and builtin argument coercion.
5. Migrate remaining call sites incrementally, deleting old helpers as coverage moves.
6. Add operation-level differential tests against `spec-exec` using side-effecting coercion objects and proxies.

## Acceptance criteria

- There is one authoritative implementation for each listed operation or an explicit tracked gap.
- Side-effect/evaluation-order tests cover success and abrupt paths for every conversion family.
- Nested calls and builtins can propagate arbitrary thrown JavaScript values, not only error-name strings.
- `try/catch/finally`, proxy traps and cross-realm errors consume the same completion ABI.
- Representative Array, String, TypedArray, Date and Proxy tests use the shared operations rather than local coercion code.
- No operation silently maps unsupported object input to a primitive default.

## Required tests

```sh
cargo test -p porffor-ir operations_ --quiet
cargo test -p porffor-aot-wasm operations_ --quiet
cargo test -p porffor-engine --quiet
cargo test -p porffor-cli wasm_ --quiet
```

Run real Test262 coercion-order cases from several builtins plus `language/statements/try`, `built-ins/Proxy`, and `built-ins/Object` to verify cross-family behavior.