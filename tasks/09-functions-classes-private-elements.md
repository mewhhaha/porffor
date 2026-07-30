# T09 — Functions, constructors, classes and private elements

**Status:** In progress — broad function/class support exists; full call/construct semantics remain

**Parallel group:** Core foundations  
**Depends on:** T04, T06, T08  
**Blocks:** T12-T15, T24

## Current repository state

The IR and Wasm backend contain explicit function metadata, call/construct
lowering, closures, bound functions, classes and private-element support, with
many focused fixtures. Cross-realm Function construction remains an explicit
dynamic-source exclusion, and complete Function/class/private-element
subtrees have not been verified against the current pin without
materializations. This remains an active foundation task.

## Objective

Complete the ECMAScript call/construct model and class semantics, including metadata, parameter environments, `this` modes, inheritance and private elements. Async/generator execution engines are owned by T14/T15, but their function objects must use the interfaces defined here.

## Function object model

Represent all required internal slots and behavior for:

- ordinary functions, arrows, methods, getters/setters and concise methods;
- base and derived constructors;
- builtin functions and host functions;
- bound functions;
- generator/async/async-generator function objects;
- class constructors and field initializer functions.

Each function must retain realm, environment, private environment, source-text representation, strictness, `this` mode, constructor kind, home object and code identity.

## Call and construct

Implement shared `[[Call]]`/`[[Construct]]` paths with:

- ordinary call binding and lexical `this` for arrows;
- sloppy `this` substitution/boxing and strict preservation;
- `new.target`, constructor return-value rules and derived-constructor `this` initialization;
- `super()` and `super` property access through the home object;
- bound arguments/this, bound constructor forwarding and bound metadata;
- custom new target and realm-correct prototype fallback;
- callable/constructable proxy integration through T11.

## Parameters and `arguments`

- Function declaration instantiation.
- Simple/non-simple parameter lists, defaults, rest and destructuring.
- Correct parameter/body environment separation.
- Mapped and unmapped `arguments`, aliasing, iterator and property descriptors.
- Duplicate parameter and strict-mode interactions from T07.
- Function `name`, `length`, inferred names and `toString` source representation.

## Classes and private elements

- Heritage evaluation, `extends null`, constructor synthesis and prototype creation.
- Instance/static public fields, methods, accessors and static blocks.
- Private fields, methods and accessors; brand creation/checking and lexical private-name resolution.
- Correct ordering of computed names, decorators if standardized in the pin, field initializers and static initialization.
- Class name TDZ, immutable inner binding and strict semantics.
- `super` in fields/static blocks and cross-realm inheritance.

## Acceptance criteria

- All function kinds share one coherent metadata/call protocol.
- Arbitrary thrown values propagate through calls/constructors.
- Bound, proxy-wrapped and cross-realm constructors preserve new-target behavior.
- Parameter/default/rest/arguments aliasing and evaluation-order tests pass.
- Class fields/private elements/static blocks pass brand, ordering, inheritance and abrupt-completion tests.
- Function `name`, `length`, prototype-property presence and descriptors match Test262.
- No function family is implemented by source-text pattern matching.

## Required tests

```sh
cargo test -p porffor-ir function_ --quiet
cargo test -p porffor-aot-wasm function_ --quiet
cargo test -p porffor-cli wasm_function --quiet
cargo test -p porffor-cli wasm_class --quiet
```

Run real filters under `language/expressions/function`, `arrow-function`, `class`, `language/statements/function`, `built-ins/Function`, `Function/prototype`, and private-element feature groups.
