# GlobalDeclarationInstantiation as one binding plan

ECMA-262 16.1.7 does not build the global environment by concatenating a list
of properties. It resolves several different name domains in a fixed order:

1. the realm already owns standard and host-defined global properties;
2. global lexical declarations claim names in the declarative record, without
   replacing properties on the global object;
3. function declarations are considered from the end of the script, so the
   last declaration for a name supplies its function object;
4. `var` names are de-duplicated, and an existing own global property satisfies
   the declaration without being recreated;
5. Annex B.3.2 may add a `var`-like variable-environment binding, then copies a
   block function into that binding only when execution reaches the block.

Those facts are one compiler invariant. They must not be reconstructed from
the order in which lowering happened to append rows.

## IR authority

`GlobalBindingPlan` is the sole script-global name table. Its map is private,
so a `ScriptIr` cannot carry two rows with the same name. Construction records
two independent facts for every object-record name:

- its initial global-property value (`Infinity`, a standard builtin, a host
  function, fresh `undefined`, or an exact source `FunctionId`); and
- the declaration set which claimed it (`None`, `Var`, `Function`, or
  `FunctionAndVar`).

Keeping these axes separate is load-bearing. For `var Infinity`, the
initializer remains the realm's immutable infinity value while the declaration
set records `Var`; a declaration without an initializer must therefore read the
existing property rather than a fresh local containing `undefined`. For a
duplicate function group, the initializer carries the exact `FunctionId` of
the last declaration rather than asking codegen to find an arbitrary function
with the same display name. A `var` accompanying that function is absorbed by
the same entry. A function declaration replaces a configurable existing
global, but collision with a restricted non-configurable property is surfaced
as an explicit unsupported GlobalDeclarationInstantiation case until the
runtime entry-realm rejection path owns its TypeError.

Global lexical names live in the plan's separate lexical-name set. They never
become object-record bindings, but the same spelling may still name a
pre-existing property (`let Infinity` shadows rather than deletes the realm's
`Infinity`). The planner owns restricted-property and lexical/variable
collision decisions; the parser's early-error checks are not used as a reason
to make an invalid declaration combination representable.

Every initializer owns its descriptor policy exhaustively:

| Initializer | writable | enumerable | configurable |
| --- | --- | --- | --- |
| `globalThis` | yes | no | yes |
| `Infinity`, `NaN`, `undefined` | no | no | no |
| fresh `var` or source function | yes | yes | no |
| standard/host global | yes | no | yes |

Adding an initializer variant therefore requires code to decide both its value
and attributes.

## Emission and writes

AOT receives the already-unique plan. It may filter entries for tree-shaken
bootstrap, but it may not collect a sequence with last-write-wins semantics.
When a script `var` was satisfied by a pre-existing property, the main-frame
cache is seeded from that property. Every later mirrored write uses ordinary
`[[Set]]`, then refreshes the cache from the property; it never overwrites the
descriptor payload directly. This applies even to a property that was fresh at
instantiation because script code can subsequently make it non-writable. The
cache therefore cannot retain a value rejected by the global object.

`AnnexBFunctionCopyTargetIr` makes its variable-environment destination
explicit. A function-owned copy writes only an owner binding. A script-owned
copy writes the planned script-global binding and uses the same mirror policy
as an ordinary `var` write. Adding another destination becomes an exhaustive
IR and emitter decision instead of falling through an unconditional property
write.

This contract is intentionally bounded to the compiler's known entry realm.
Dynamic global-object mutation before a separately evaluated script will need
the same plan vocabulary plus runtime `CanDeclareGlobalFunction` /
`CanDeclareGlobalVar` checks; it must not weaken this unique-plan invariant.
