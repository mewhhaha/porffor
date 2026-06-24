# T10 — Object model, descriptors and exotic-object protocol

**Status:** Blocked on T04-T06  
**Parallel group:** Core foundations  
**Depends on:** T04, T05, T06  
**Blocks:** T11, T16-T24

## Objective

Implement the ECMAScript object internal-method model and exact property descriptor semantics as a reusable runtime/compiler layer. Arrays, typed arrays, strings, module namespaces and proxies should extend this protocol rather than bypass it with unrelated representations.

## Internal methods

Define an explicit dispatch contract for:

- `[[GetPrototypeOf]]`, `[[SetPrototypeOf]]`;
- `[[IsExtensible]]`, `[[PreventExtensions]]`;
- `[[GetOwnProperty]]`, `[[DefineOwnProperty]]`;
- `[[HasProperty]]`, `[[Get]]`, `[[Set]]`, `[[Delete]]`;
- `[[OwnPropertyKeys]]`;
- optional `[[Call]]` and `[[Construct]]` integration for callable objects.

Ordinary objects should use optimized implementations. Exotic objects register overrides while retaining shared invariant checks.

## Property descriptors

- Represent absent descriptor fields distinctly from fields containing `undefined`/`false`.
- Implement data/accessor/generic descriptor classification, `CompletePropertyDescriptor`, `IsCompatiblePropertyDescriptor` and `ValidateAndApplyPropertyDescriptor`.
- Preserve getter/setter identity and callable validation.
- Enforce non-configurable/non-writable transitions exactly.
- Implement `FromPropertyDescriptor` and `ToPropertyDescriptor` with observable property access order.

## Ordinary object behavior

- Prototype traversal, receiver-aware accessors and assignment.
- Prototype-cycle detection.
- Integer-index/string/symbol own-key ordering.
- Extensibility, seal/freeze/integrity-level operations.
- Object literal definitions, computed keys, methods/accessors/spread and `__proto__` semantics.
- `Object` constructor/static/prototype methods and exact descriptors.

## Exotic protocol targets

Create extension points for:

- arrays (T16);
- string wrapper objects (T18);
- arguments objects (T09);
- integer-indexed typed arrays (T17);
- module namespace objects (T12);
- immutable-prototype and host-defined objects;
- proxies (T11), which must wrap and validate any target implementation.

## Optimization constraints

Static shapes and direct offsets are allowed only when guards prove that prototypes, descriptors, accessors, proxies and symbols cannot make the shortcut observable. A deoptimization/fallback path must execute the same internal operation.

## Acceptance criteria

- All property operations route through the explicit internal-method API or a proven guarded fast path.
- Descriptor conversion and redefinition order tests pass with side-effecting/proxy descriptors.
- Own-key ordering is correct for numeric strings, ordinary strings and symbols.
- Object integrity methods handle primitives, proxies and exotics correctly.
- Prototype mutation/cycle and receiver-aware setter cases pass.
- Feature modules can add an exotic implementation without editing a giant central match.
- Object and descriptor Test262 subtrees reach zero failures before this task is closed.

## Required tests

```sh
cargo test -p porffor-ir object_ --quiet
cargo test -p porffor-aot-wasm object_ --quiet
cargo test -p porffor-cli wasm_object --quiet
./target/debug/porf test262 run built-ins/Object --execution-backend wasm
./target/debug/porf test262 run built-ins/Reflect --execution-backend wasm
```

Include tests with accessors, symbols, proxies, inherited properties, non-extensible targets and cross-realm descriptor functions.