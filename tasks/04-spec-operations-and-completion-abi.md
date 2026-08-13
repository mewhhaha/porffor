# T04 — Shared ECMAScript operations and completion ABI

**Status:** In progress — shared catalogs exist; migration is incomplete

**Parallel group:** Foundation  
**Depends on:** T02  
**Blocks:** Most semantic feature tasks

## Current repository state

`lila-ir/src/operations.rs` and
`lila-aot-wasm/src/operations.rs` provide shared operation catalogs and
emitters, while the backend has explicit ABI and control-flow modules. The 29
expression-shaped `SpecOperationIr` rows now come from one typed descriptor
declaration containing the name, family, operand domain, normal result and
abrupt capability. The backend validates that closed operand domain before
dispatch, and the former parallel family/result/abrupt matches are gone.

Typed abrupt routing now covers `GetV` inside `GetMethod`, the `ToNumber` of
`Number.prototype.toFixed` argument zero, and every caller of the shared tagged
`ToPrimitive` emitter. The sole tagged emitter requires a closed
`ToPrimitiveAbruptRoute`: route to the active handler, return the current
function, or close a named iterator and return. Adding a route requires an
exhaustive match update, and a new caller cannot omit the decision. The
duplicate tagged `_without_throw_propagation` entry point is gone.

The same route is also mandatory at the lower object/function-specialized
ToPrimitive seam. Its byte-identical `_without_throw_propagation` twin is gone,
and the former generic raw-completion route is gone. Private raw emitters now
return a `#[must_use]` `PendingToPrimitiveCompletion` with private fields. Every
internal numeric/string composite consumes that token in its exact guarded
continuation; the runtime-helper generator reaches only a dedicated wrapper
that emits all four ABI result slots. `unused_must_use` is denied in the module,
so a new internal raw call that omits its continuation fails to build. Array
element stringification selects active-handler routing before coercion.

Primitive ToString now has the same closed ownership rule. Its sole emitter
requires a `PrimitiveToStringAbruptRoute`: active handler, current-function
return, or iterator-close-and-return with a complete local witness. The former
raw `_to_local_without_throw_return` copy is gone. Every consumer names its
policy, and adding a policy requires an exhaustive match update. This fixes the
shared `SpecOperationIr::ToString`, `String(object)` and array-element paths:
when an object's coercion hook returns a Symbol, the resulting TypeError now
reaches an enclosing catch just like a value thrown by the hook, instead of
unconditionally returning the whole function. Object.fromEntries and
Object.groupBy retain their iterator-close-before-return discipline.

This migration also fixes the Temporal month-code coercion path: a user value
thrown by `toString` now escapes unchanged instead of being overwritten by the
later non-String TypeError check. Existing coercion and iterator-close order is
otherwise unchanged. These wrappers do not make the remaining property and
builtin-coercion sites authoritative: feature
emitters still contain substantial local coercion, property and completion
logic, and the large Test262 materialization layer shows that shared operations
are not yet authoritative across every family. The Wasm completion convention
also remains the existing tuple/current-completion mechanism rather than the
target `exnref` design.

The descriptor and migration boundary are specified in
[`docs/rust-rewrite/operation-descriptors.md`](../docs/rust-rewrite/operation-descriptors.md).
Keep new cross-family semantics in the shared operation layer and delete local
copies only as callers migrate.

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
2. Introduce typed operation nodes/helpers in `lila-ir`.
3. Introduce shared Wasm helper generation and a registry that emits each helper once per module.
4. Convert representative property access, builtin argument coercion and tagged `ToPrimitive` paths.
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
cargo test -p lila-ir operations_ --quiet
cargo test -p lila-aot-wasm operations_ --quiet
cargo test -p lila-engine --quiet
cargo test -p lila-cli wasm_ --quiet
```

Run real Test262 coercion-order cases from several builtins plus `language/statements/try`, `built-ins/Proxy`, and `built-ins/Object` to verify cross-family behavior.
