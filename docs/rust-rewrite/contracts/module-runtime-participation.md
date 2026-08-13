# Module runtime participation

This contract closes the boundary between a module being part of the loaded,
linked graph and a module contributing code or objects to the emitted artifact.
Those are deliberately different facts for source-phase requests.

## Defect closed

`ModuleEvaluationModeIr` already classifies a unit reached only through
`import source` as `NotEvaluated`. The body emitter honors that classification,
but several sibling collectors previously iterated every unit. A source-only
unit could therefore contribute namespace aliases, module-source aliases,
`import.meta` cells and dynamic-import dispatchers even though the unit was
never instantiated. Because the linker merges those declarations into one
Script scope, an inactive alias could become visible to active code or create a
false collision with an active declaration.

For example, an entry that imports `inactive.js` in the source phase and reads
`typeof ghost` must observe `"undefined"` when `ghost` exists only as a
namespace import inside `inactive.js`. Emitting that inactive import's alias
makes the entry observe an object instead.

## Closed domain

`ModuleMaterializationModeIr` is the private, two-case domain consumed by
runtime source generation:

- `Eager`: the unit contributes its body inline;
- `Deferred`: the unit contributes its body thunk and deferred namespace
  machinery.

There is one exhaustive conversion from `ModuleEvaluationModeIr`:

- `Eager` becomes `Some(Eager)`;
- `Deferred` becomes `Some(Deferred)`;
- `NotEvaluated` becomes `None`.

All runtime collectors consume `ModuleGraphIr::materialized_units()` or the
same per-unit conversion. Adding a fourth evaluation mode therefore requires a
decision at the conversion or fails to compile; it cannot silently inherit the
runtime behavior of an existing arm. `ModuleNamespaceIr` carries this typed
mode rather than a parallel `deferred: bool`, and namespace getter generation
matches it exhaustively.

## Invariants

1. A `NotEvaluated` unit remains in the loaded graph. It is parsed, checked for
   early errors, linked, and available as a module source object when an active
   referrer requests one.
2. A `NotEvaluated` unit contributes no body, import alias, namespace, nested
   module-source object, `import.meta` cell, dynamic-import dispatcher or
   runtime-only name-collision diagnostic of its own.
3. Participation is decided by the referrer. An eager or deferred referrer's
   `import source x from "m"` still materializes the module source object for
   `m`, even though `m` itself has no runtime materialization mode.
4. Dynamic components are all discovered before evaluation modes are fixed,
   because a component can make its target eager or deferred. Once that fixed
   point is complete, only components whose referrer materializes remain in the
   artifact registry.
5. A namespace can exist only for a unit with an eager or deferred
   materialization mode. An attempt to construct one for a source-only unit is
   refused rather than manufacturing an eager namespace over bindings whose
   body is absent.
6. Filtering runtime artifacts never filters parse, early-error, host
   resolution or link-error work. Source phase is not a silent-skip mechanism.

## Durable regression

One linker regression builds an active entry and a source-only unit whose body
contains a namespace import, a nested source import, `import.meta` and a static
dynamic import. The linked source must contain the active entry's module source
object and alias, but none of the inactive unit's namespace, alias, nested
source object, meta cell, dispatcher or dynamic component.

## Nonclaims

This contract does not make dynamic target evaluation lazy, implement exact
module-namespace exotic descriptors, support deferred cycles or top-level
await, or close T12/Test262. It only makes the already-computed runtime
participation decision authoritative at every current source-emission site.
