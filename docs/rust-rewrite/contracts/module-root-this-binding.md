# Module root `this` binding

This contract preserves each source owner's root `this` binding when a linked
module graph is lowered through the shared Script pipeline.

## Defect closed

The linker assembles module source and reparses it with the Script goal for
one function-id/slot numbering domain. Each canonical Module still has its own
activation and binding environment. That implementation parse goal is not the
semantic source goal. A Script Global Environment Record returns the global object from
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

The distinction is exhaustive. In a canonical Module owner, module-root `this`
lowers to `ExprIr::Undefined`; Script-root `this` lowers to
`ExprIr::This`; activation `this` keeps the existing runtime operation.
Ordinary functions and derived constructor activations are therefore unaffected
by the assembled source's goal.

Canonical source owners carry `ModuleActivation` or `AsyncModuleActivation`
protocol metadata. The function lowerer derives their root binding as
`Undefined` before lowering their bodies; lexical arrows retain that root
binding even when the source owner is represented by an async arrow for the
private instantiation suspension boundary. A Script entry stays outside those
owners: its original body is appended as the Script root source after module
instantiation. Ordinary exported functions are strict ordinary functions;
their calls retain their receiver, and arrows nested inside them capture it.

`ScriptIr::top_level_this_uses` counts only root reads that resolve to the
Script global object. The AOT planner may use that count to request global
bootstrap; a statically undefined module-root read cannot request it.

## Invariants

1. The original parse goal and trusted Module owner protocol, not the shared
   Script reparse, choose root `this`.
2. A root arrow cannot turn a Module root binding into a function activation or
   a Script-global fallback.
3. An arrow nested in an ordinary function still reads that function's lexical
   `this`.
4. Ordinary and derived-constructor activations retain their existing dynamic
   `this` behavior.
5. Both synchronous and async Module owners use the explicit undefined root
   binding rather than depending on an ordinary-function wrapper's receiver.
6. Only Script-global root reads contribute to global-object bootstrap.

## Durable regressions

IR regressions cover direct module-root `this`, a root arrow, a nested root
arrow chain, and guards for Script root and ordinary-function activation
behavior. An engine regression executes the direct and lexical module cases on
Wasm AOT and calls `this.propertyIsEnumerable("Infinity")` to ensure the
Script-global constant-fold path cannot bypass the typed root binding, while
retaining the existing Script `this === globalThis` coverage. The exact pinned
witness is `language/module-code/eval-this.js`.

The `aot_script_import_jobs` target additionally checks Module-root this and
nested arrows from sloppy and strict Scripts, ordinary exported calls with and
without a receiver, same-file Script/Module source ownership, and root global
reflection before and after import jobs. Compiler-private module owners must
not become visible as global own keys or named direct-eval bindings. A separate
sloppy Script proves a Proxy `with` environment observes the source specifier
without observing private dispatcher lookups.

## Nonclaims

The root-binding domain alone does not implement Module Namespace Exotic
Object internal methods or close the complete cyclic/deferred/async surface or
T12/Test262. Separate canonical graph ownership provides per-module
environments, lazy evaluation and lifecycle jobs; the Script import batch and
its pending verification are recorded in
[`script-dynamic-import-jobs.md`](script-dynamic-import-jobs.md). Source-phase
Script imports remain an explicit unsupported compilation boundary.
