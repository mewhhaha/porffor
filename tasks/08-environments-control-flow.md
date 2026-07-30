# T08 — Environments, references, control flow and abrupt completion

**Status:** In progress — dedicated lowering/emission modules exist; conformance closure remains

**Parallel group:** Core foundations  
**Depends on:** T04, T07  
**Blocks:** T09, T12-T15, T24

## Current repository state

Environment, reference-adjacent lowering and structured control-flow emitters
now support substantial lexical scope, closure, loop, destructuring and
try/finally behavior. The parse-once prerequisite remains open, several
environment/control-flow files are still large shared hotspots, and the
language subtrees assigned to this task have not been proven zero-failure on a
current complete Wasm-AOT matrix.

## Objective

Implement spec-correct binding resolution and structured control flow so lexical scope, TDZ, assignment, loops and `try/finally` all share one model instead of feature-specific lowering shortcuts.

## Environment records

Provide explicit IR/runtime support for:

- declarative, function, module, global and object environment records;
- lexical/variable/private environment chains;
- mutable, immutable, deletable and indirect bindings;
- initialized vs uninitialized bindings and TDZ checks;
- global declaration instantiation and restricted global properties;
- `with` object environments and `Symbol.unscopables`;
- per-iteration environments for lexical loop bindings;
- catch environments and Annex B catch/`var` interactions.

Closures must capture cells/environment references, not copies, and capture analysis must remain correct across nested functions, classes, generators and async suspension.

## Reference model

Introduce a typed Reference representation covering:

- binding references;
- property references, including primitive bases and `super`;
- private references;
- unresolvable references;
- strictness, receiver and this-value information.

Implement `GetValue`, `PutValue`, `InitializeReferencedBinding`, `Delete`, `typeof` unresolvable behavior and assignment/update evaluation order through shared operations.

## Control-flow model

Lower statements to structured blocks with explicit completion edges:

- labels, `break` and `continue` target resolution;
- `return` and `throw`;
- `switch` fallthrough;
- `while`, `do`, classic `for`, `for-in`, `for-of` and per-iteration binding creation;
- `try/catch/finally`, including completion replacement and value preservation;
- destructuring in declarations, assignment, parameters, catch and loop heads;
- short-circuit expression control flow and optional chaining.

Wasm branch depth must be derived from a structured control stack, never patched with case-specific constants.

## Correctness focus

- TDZ begins at block entry, including loop heads and default parameter initializers.
- RHS/key/iterator expressions are evaluated in specification order.
- `finally` may override return/throw/break/continue exactly as specified.
- Iterator closing on abrupt loop completion is delegated to T15's shared iterator operations.
- Global `var`, lexical declarations and implicit-global writes obey property attributes and strict mode.

## Acceptance criteria

- Environment and Reference types are explicit in IR and do not depend on variable-name string conventions.
- Nested `try/finally` and labelled-loop tests pass without manual Wasm depth values.
- Closure mutation, per-iteration capture and TDZ tests pass.
- Destructuring abrupt-completion/evaluation-order cases pass across declarations, assignments and parameters.
- `with`/unscopables and global declaration instantiation are either implemented or left as explicit, owned failures—not approximated.
- Related `language/statements`, `language/expressions/assignment`, scope and global tests reach zero failures.

## Required tests

```sh
cargo test -p porffor-ir environment_ --quiet
cargo test -p porffor-aot-wasm control_flow_ --quiet
cargo test -p porffor-engine --quiet
cargo test -p porffor-cli wasm_ --quiet
```

Run focused real filters for lexical declarations, destructuring, `for-in`, `for-of`, `try`, labels, `with`, global code and closure capture.
