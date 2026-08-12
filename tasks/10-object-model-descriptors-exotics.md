# T10 — Object model, descriptors and exotic-object protocol

**Status:** In progress — canonical descriptor lattice is consumed; exotic closure remains

**Parallel group:** Core foundations  
**Depends on:** T04, T05, T06  
**Blocks:** T11, T16-T24

## Current repository state

`lila-ir` now owns one closed ECMA-262 6.2.6 descriptor lattice: six typed
fields, three presence states, validation before classification, the
data/accessor/generic partition and two complete stored kinds. The ordinary
object `ValidateAndApplyPropertyDescriptor` emitter consumes that lattice, and
the array named-property validator now does too. Its data/accessor entry points
construct `ValidatedDescriptor<WasmLocals>` values, while the validator derives
kind-change checks from `classify`/`KindTerms`; the former parallel
`requested_data_descriptor: bool`, six positional field fragments and
hand-written four-field kind-presence fold are gone. Heap descriptor values and
masks are also distinct typed domains, so an accessor word cannot acquire a
`[[Writable]]` bit through their constructors.

Arguments-object `length` writes now take the same closed
`PropertyDescriptorKind` domain rather than an `accessor: bool`. The Generic
arm preserves the existing data/accessor kind (and data-only writability) while
applying only the requested attributes, so a generic update can no longer
silently turn an accessor `length` back into a data property. Its backing value
is a tagged ECMAScript value rather than a coerced integer, its getter and
setter are stored independently, and the read, write and
`GetOwnPropertyDescriptor` paths exhaustively follow the stored kind. Updating
one accessor field preserves the omitted peer, while a real kind conversion
initializes omitted fields to `undefined` instead of reviving stale storage.
The remaining arguments `callee` and `length` attribute tables also carry
`DescriptorMask` values rather than raw `u64` words. They can test or apply only
the three named attribute masks, and cannot accidentally receive a complete
stored descriptor word at that boundary.

This is still a foundation, not task closure. Array application and index
paths, arguments descriptors, several builtin/exotic emitters and lowering
shape facts still consume derived raw words or parallel positional forms. The
`Presence::Present` step-4 exemption remains the explicit LN10 obligation, the
ordinary `Object.defineProperty` adapter still relies on its emitted run-time
6.2.6.5 step-9 check, and the shortcut audit still finds path/source-dependent
materializations. `cargo check -p lila-ir -p lila-aot-wasm` and the focused
array descriptor CLI fixture are green; a complete current-pin Wasm-AOT
Object/descriptor subtree run has not been performed.

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
cargo test -p lila-ir object_ --quiet
cargo test -p lila-aot-wasm object_ --quiet
cargo test -p lila-cli wasm_object --quiet
./target/debug/lila test262 run built-ins/Object --execution-backend wasm
./target/debug/lila test262 run built-ins/Reflect --execution-backend wasm
```

Include tests with accessors, symbols, proxies, inherited properties, non-extensible targets and cross-realm descriptor functions.
