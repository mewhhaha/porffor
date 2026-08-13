# Module root `this` binding

This contract preserves the source parse goal's root `this` binding when a
linked module graph is lowered through the merged Script pipeline.

## Defect closed

The linker concatenates module source and reparses it with the Script goal so
the graph can share one activation environment and one function-id/slot
numbering domain. That implementation parse goal is not the semantic source
goal. A Script Global Environment Record returns the global object from
`GetThisBinding`, while a Module Environment Record returns `undefined`.

The previous lowerer always represented root `this` as `ExprIr::This`. A
post-lowering diagnostic rejected direct module-root uses, but a root arrow was
already considered a function body and bypassed that diagnostic. With no
lexical `$this` capture, the Wasm backend then used its Script-global fallback,
so `() => this` in a module observed `globalThis`.

## Closed domains

`RootThisBinding` is derived once from the original source goal:

- `GlobalObject` for Script code;
- `Undefined` for Module code.

Every `ScriptLowerer` constructor requires that value, including prepasses and
nested/generated lowerers. A new construction path therefore cannot silently
default to Script semantics.

`CurrentThisBinding` separates a root binding from a function activation:

- `Root(RootThisBinding)` remains lexical through every root arrow, including
  nested arrow chains;
- `Activation(ValueInfo)` belongs to an ordinary function activation or a real
  lexical capture of one.

The distinction is exhaustive. In a flat eager synchronous Module-entry graph,
module-root `this` lowers to `ExprIr::Undefined`; Script-root `this` lowers to
`ExprIr::This`; activation `this` keeps the existing runtime operation.
Ordinary functions and derived constructor activations are therefore unaffected
by the merged source's goal.

Three pre-existing graph shapes deliberately put source-level module code in a
strict activation: a Script-entry graph wraps its module closure in an ordinary
IIFE, a deferred unit uses an ordinary thunk, and a top-level-await Module-entry
graph uses an async IIFE. In those shapes `this` remains `ExprIr::This`; the
wrapper's bare strict call supplies the required `undefined`. This seam adds no
new wrapper and changes only the flat eager synchronous Module-entry path that
previously had neither an activation nor the correct root binding.

`ScriptIr::top_level_this_uses` counts only root reads that resolve to the
Script global object. The AOT planner may use that count to request global
bootstrap; a statically undefined module-root read cannot request it.

## Invariants

1. The original parse goal, not the merged Script reparse, chooses root `this`.
2. A root arrow cannot turn a Module root binding into a function activation or
   a Script-global fallback.
3. An arrow nested in an ordinary function still reads that function's lexical
   `this`.
4. Ordinary and derived-constructor activations retain their existing dynamic
   `this` behavior.
5. Flat eager synchronous Module-entry `this` is represented directly in IR and
   needs no backend special case or new ordinary-function wrapper; existing
   wrapper paths retain their activation representation.
6. Only Script-global root reads contribute to global-object bootstrap.

## Durable regressions

IR regressions cover direct module-root `this`, a root arrow, a nested root
arrow chain, and guards for Script root and ordinary-function activation
behavior. An engine regression executes the direct and lexical module cases on
Wasm AOT and calls `this.propertyIsEnumerable("Infinity")` to ensure the
Script-global constant-fold path cannot bypass the typed root binding, while
retaining the existing Script `this === globalThis` coverage. The exact pinned
witness is `language/module-code/eval-this.js`.

## Nonclaims

This seam does not create a distinct Module Environment Record per linked
unit, stop module `var` declarations from reflecting on the merged Script
global object, rename colliding unit bindings, make dynamic imports lazy,
implement Module Namespace Exotic Object internal methods, coordinate
top-level-await jobs or close T12/Test262. It adds no new wrapper. Existing
Script-entry module closures, deferred units and top-level-await graphs retain
their strict function/thunk wrappers; only the flat eager synchronous
Module-entry path remains wrapper-free, avoiding an invented top-level
`arguments` binding there.
