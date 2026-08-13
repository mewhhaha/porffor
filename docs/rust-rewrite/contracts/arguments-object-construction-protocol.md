# Arguments-object construction: one semantic choice and one validated map

## Decision

Each Wasm function builder carries one closed arguments-object construction
protocol:

```text
Absent
Present(Unmapped)
Present(Mapped(plan))
```

The protocol is selected once when the builder is created. Arrow functions use
`Absent` because their `arguments` reference is lexical. Every ordinary
function uses `Present`: a strict function or a function with a non-simple
parameter list uses `Unmapped`, while a sloppy function with a simple parameter
list uses `Mapped(plan)`.

This is the split made by FunctionDeclarationInstantiation. Its choice between
CreateUnmappedArgumentsObject and CreateMappedArgumentsObject depends on
`[[ThisMode]]`, strictness and whether the parameter list is simple. It does not
depend on which backend storage happened to be allocated for a parameter.

## The invalid fallback

The former emitter recomputed the semantic choice from four independent
predicates at the point of object construction:

```text
ordinary function
and not strict
and no default or rest parameter
and every retained parameter currently has an owned environment slot
```

The last predicate was not a semantic condition. If lowering omitted a mapped
parameter's environment slot, the predicate silently changed a required mapped
arguments object into an unmapped one. That changed indexed aliasing and the
`callee` property instead of identifying malformed compiler IR.

The same emitter later recomputed duplicate-name handling and used the raw
parameter index as the environment slot when the whole list had no duplicate.
That shortcut made the output depend on an unstated coincidence between two
integer namespaces.

## Construction

The backend derives the semantic variant only from the function protocol,
strictness and parameter-list shape:

- `Arrow` and `AsyncArrow` become `Absent`;
- strict ordinary functions become `Present(Unmapped)`;
- ordinary functions with a default, rest or destructured parameter become
  `Present(Unmapped)`;
- every other ordinary function becomes `Present(Mapped(plan))`, including an
  empty simple parameter list, whose plan is intentionally empty.

Destructuring is represented by the lowered function's
`ParameterInitialization` statement. The construction protocol treats that
marker as non-simple even though the storage parameter itself has a generated
name.

Only after choosing `Mapped` does construction validate backend storage. The
plan retains the last occurrence of each duplicate parameter name, exactly as
CreateMappedArgumentsObject builds its ParameterMap in reverse list order. For
each retained parameter it records a typed pair:

```text
ArgumentIndex -> ParameterEnvironmentSlot
```

Those are distinct private newtypes. A producer cannot transpose them, pass a
raw parameter index as a storage slot, or construct an entry outside the
planner. Every retained name must resolve to exactly one owned environment
binding, and that slot must have exactly one owner. A missing, duplicate or
aliased slot is malformed lowered IR and makes Wasm emission fail. Storage
validation never reclassifies the function as unmapped.

## Consumption

Parameter binding, local-count planning and root-var reuse consume `Absent`
versus `Present` directly. Arguments-object emission accepts only the narrower
present protocol and exhaustively selects:

- the current environment plus the mapped `callee` data property for
  `Mapped(plan)`;
- no parameter environment plus the unmapped poison-pill `callee` accessor for
  `Unmapped`.

Indexed descriptor emission iterates only the prevalidated mapped entries. It
does not inspect parameter syntax, search environment bindings, recalculate
duplicates or infer an environment slot from an argument index.

Internal Wasm callables retain their existing strict/unmapped object, while the
script main builder uses `Absent`. User-function builder creation is fallible,
and the public emitter propagates protocol-construction errors.

## Enforced invariants

1. A function has either no own arguments object or exactly one present object
   kind; impossible cross-products cannot be assembled.
2. Function shape and strictness are the only inputs to mapped versus unmapped
   semantics.
3. `Mapped(empty)` remains distinct from `Unmapped`.
4. Duplicate names map only their last parameter occurrence.
5. Argument indexes and environment slots cannot be interchanged at a typed
   producer/consumer boundary.
6. Every mapped entry names validated, unique environment storage; malformed IR
   is an emission error, not a semantic fallback.
7. Adding another present object kind requires updating every exhaustive
   construction consumer or the backend fails to compile.

## Verification boundary

Backend module tests cover absence for arrows, unmapped strict/default/rest/
destructured functions, the empty mapped plan, valid unique and duplicate-name
maps, and emission rejection for a lowered function whose required parameter
slot is removed. Existing engine and CLI regressions already cover both
directions of mapped aliasing, last-duplicate behavior, mapped descriptors and
mapped versus unmapped `callee`; this seam adds no duplicate JavaScript fixture.

Cargo and Test262 verification remain deferred to the centralized batch while
their build/runtime leases are active. The change is intended to preserve all
valid emitted behavior; it closes a compiler-internal failure mode and does not
claim a newly passing runtime case.

## Nonclaims

This protocol does not complete parameter/body environment separation, add a
new arguments-object representation, change descriptor layout, or broaden the
supported function surface. It makes the existing construction decision and
its storage dependency explicit. T09 remains in progress.
