# T09 — Functions, constructors, classes and private elements

**Status:** In progress — broad function/class support exists; full call/construct semantics remain

**Parallel group:** Core foundations  
**Depends on:** T04, T06, T08  
**Blocks:** T12-T15, T24

## Current repository state

The IR and Wasm backend contain explicit function metadata, call/construct
lowering, closures, bound functions, classes and private-element support, with
many focused fixtures. Class element definitions now carry the closed
`ClassMethodKindIr::{Method, Getter, Setter}` domain: a constructor or a
no-class-role function cannot enter a public/private method row, and the Wasm
definition emitter consumes the three cases exhaustively instead of rejecting
an impossible kind at runtime. The function lifecycle is now also a closed
`FunctionProtocolIr`: analysis, lowering signatures, `FunctionIr` and Wasm
metadata carry one of the reachable ordinary/arrow/resumable/class roles rather
than independently combining flavor, execution kind, constructability and
class role. Generated accessors cannot become resumable or constructable,
class constructors cannot lose `[[Construct]]`, and the backend derives its
runtime flags exhaustively from the same protocol. Backend prototype
materialization is a separate policy, so realm bootstrap no longer lies about
the constructability of GeneratorFunction, AsyncFunction or
AsyncGeneratorFunction while suppressing their automatically generated
`prototype` object. The exact matrix and boundary choices are recorded in
`docs/rust-rewrite/contracts/function-protocol.md`.

Private-element heap storage now has the closed five-row
`PrivateElementHeapKind` protocol. Receiver rows are either a brand or a field;
shared definition rows are a setter, method or getter. The entry writer accepts
only legal row variants instead of independently combining an optional
receiver, a raw integer kind and an optional value, and definition lookup has
the narrower three-kind domain. Private read and write trap compiler-owned
corrupt rows rather than treating an unknown kind as a brand. The stable wire
words and backend/spec boundary are recorded in
`docs/rust-rewrite/contracts/private-element-entry-protocol.md`.

Arguments-object construction now has the closed backend protocol
`Absent | Present(Unmapped | Mapped(plan))`. Arrow functions have no own
binding; strict or non-simple ordinary functions are unmapped; sloppy simple
ordinary functions carry a prevalidated argument-index-to-environment-slot
plan. Missing mapped storage is rejected as malformed lowered IR instead of
silently changing the function to unmapped, duplicate names retain only their
last occurrence, and an empty simple list remains `Mapped(empty)`. The semantic
and storage boundaries are recorded in
`docs/rust-rewrite/contracts/arguments-object-construction-protocol.md`.

Bound-function creation now preserves `[[BoundThis]]` as the exact tagged
ECMAScript value supplied to `bind`. A private two-source domain admits only
builtin argument zero and the compiler-owned Proxy revocation Object; sibling
modules cannot call the raw payload/tag allocator. Strict preservation and
sloppy substitution/boxing remain centralized in the target-call path, so a
strict primitive is not boxed during binding and a sloppy primitive receives a
fresh wrapper on each invocation. The boundary and its cross-realm nonclaim are
recorded in
`docs/rust-rewrite/contracts/bound-function-this-capture.md`.

Non-generic Boolean prototype method calls now retain the acquired function
object and the reference base as separate `CallIndirect` operands. Shape
analysis may identify `%Boolean.prototype.toString%` or `valueOf`, but that
knowledge no longer authorizes a key-only `CallMethod` whose backend fast path
can replace the transferred function according to the receiver and property
name. The receiver-materialization boundary now has a durable binding-identity
witness proving the callee read and `this_arg` share the same single evaluation.
Both methods, valid boxed-Boolean calls, standard and unrelated destination
names, and the four wrong-brand object families in the ten bounded Test262
witnesses are recorded in
`docs/rust-rewrite/contracts/non-generic-builtin-method-callee-identity.md`.

Cross-realm Function construction remains an explicit dynamic-source
exclusion, and complete Function/class/private-element subtrees have not been
verified against the current pin without materializations. This remains an
active foundation task.

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
cargo test -p lila-ir function_ --quiet
cargo test -p lila-aot-wasm function_ --quiet
cargo test -p lila-cli wasm_function --quiet
cargo test -p lila-cli wasm_class --quiet
```

Run real filters under `language/expressions/function`, `arrow-function`, `class`, `language/statements/function`, `built-ins/Function`, `Function/prototype`, and private-element feature groups.
