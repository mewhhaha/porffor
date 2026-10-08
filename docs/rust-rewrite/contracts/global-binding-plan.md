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
set records `Var`; a declaration without an initializer leaves the existing
property unchanged and does not invoke its getter. Subsequent source reads
resolve the actual property. For a
duplicate function group, the initializer carries the exact `FunctionId` of
the last declaration rather than asking codegen to find an arbitrary function
with the same display name. A `var` accompanying that function is absorbed by
the same entry. A function declaration replaces a configurable existing
global, but collision with a restricted non-configurable property is surfaced
as an explicit unsupported GlobalDeclarationInstantiation case until the
runtime entry-realm rejection path owns its TypeError.

Global lexical bindings live in the plan's separate map from name to the
closed `GlobalLexicalBindingModeIr::{Mutable, Immutable}` domain. Lowering
obtains the mode from the analyzed Script activation's binding modes; codegen
does not scan declaration syntax again. They never become object-record
bindings. A configurable pre-existing global may share the spelling, but a
non-configurable property such as `Infinity` rejects the lexical declaration
during instantiation. The planner owns restricted-property and
lexical/variable collision decisions; the parser's early-error checks are not
used as a reason to make an invalid declaration combination representable.

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
`main_frame_write_bindings` allocates publication temporaries for declaration
and Annex B writes. These slots do not establish a declarative source Reference
or cache source-visible global values. Each publisher writes its value before
reading that storage for ordinary `[[Set]]`. Instantiation does not seed the
slots by reading global properties, and a publication mirror performs no second
`[[Get]]`.

An ordinary source assignment to an unshadowed script-global `var` uses the
runtime Global Environment Reference path even when that declaration also owns
a publication temporary. ResolveBinding precedes RHS evaluation; PutValue keeps
that selected Record and rechecks its live delegate after RHS effects. Deleting
a configurable global on the RHS therefore throws ReferenceError in strict
assignment instead of recreating the property through a local write mirror.
Same-spelled parameter, local, lexical and captured bindings retain their actual
binding storage. The exact native RHS-deletion failure and focused IR controls
cover this distinction; verification of this producer repair is pending.

Logical assignments carry an `EnvironmentIdentifierIr` whose constructor fixes
its resolution start to the Global Environment. The emitter resolves once,
reads through that Reference, and evaluates the RHS and PutValue only on the
taken branch. A getter deleting the global property therefore cannot trigger a
second ResolveBinding between GetValue and the RHS. Each operation still checks
the selected Global Record's current declarative/object delegate. Local and
captured bindings keep their own storage; an enclosing `with` first selects its
object Reference and reaches the Global start only on fallback. Has/Get effects
invalidate lowering facts before RHS lowering, and possible callable results
remain open candidates. The existing both-mode global-assignment native cohort
now includes getter-deletion, inherited `has` ordering and abrupt/short-circuit
controls; execution of these added controls remains pending.

Publication preserves the current StatementList value on normal completion,
including empty loop bodies, and propagates an abrupt setter completion before
restoring that value.
Thus an existing accessor is invoked only by the JavaScript read or write that
requires it. A rejected sloppy write leaves the global property unchanged;
later source reads observe that property rather than the write temporary.

Global root functions, including names also declared with `var`, use the same
property resolution. Only function declarations that require body-local
initialization receive local source bindings. This preserves installed function
identity and observes later global replacement without a second cache authority.
Host-name reachability does not override the current property's value facts;
source declarations and deletion also control names such as `parseInt`.
Script-global `var` metadata does not make a property undeletable. Deletion
checks the runtime descriptor, and subsequent reads distinguish an absent
binding from a present property whose value is `undefined`. Only local and
lexical ownership establishes a non-deletable declarative binding.

Every script-global read-modify-write retains the actual Global Environment
Record. Eager arithmetic/bitwise operations and numeric updates use the shared
`EnvironmentIdentifierIr::global` lifecycle in both root and nested owners.
Resolution and GetValue precede the RHS/coercion; PutValue retains that same
Record and checks its current lexical delegate. A lexical installed by Has,
Get, the RHS or coercion therefore replaces only the delegate, not the held
Reference. An object delegate observes its independent HasProperty/Get/Set
steps; initial absence cannot become an update by reading a prototype Proxy's
get trap. Declaration-publication temporaries never substitute for this
lifecycle. The original ToLength nested-callback fixture remains a regression
control, alongside new native late-lexical/Has-order and parsed IR controls;
verification of the joined repair is pending.

`AnnexBFunctionCopyTargetIr` makes its variable-environment destination
explicit. A function-owned copy writes only an owner binding. A script-owned
copy writes the planned script-global binding and uses the same publication
mirror policy as a `var` declaration initializer. Adding another destination
becomes an exhaustive IR and emitter decision instead of falling through an
unconditional property write.

Fresh entry scripts use this unique plan to bootstrap their global object.
Prepared Script units retain the same binding vocabulary and a separate
source-ordered declaration plan. Their runtime instantiation checks current
lexical bindings, own descriptors and extensibility before installing globals;
see [precompiled realm Scripts](precompiled-realm-scripts.md).

Each realm owns a Global Environment root. Its declarative table maps names
to the same cells used by ordinary lexical captures, including uninitialized
and immutable bindings. Ordinary lexical environments terminate at this root;
source functions resolve globals and intrinsic prototypes through their
captured root's realm. Dynamic Function allocation preserves its immutable
function execution context and installs the constructor realm's root as that
context's lexical environment. Overwriting the function's environment handle
would discard the active-function identity needed by its prologue.

Main initialization constructs that root from the still-rooted actual Realm
before allocating the script activation. The existing lexical and captured
cells remain in their activation layer, whose parent is the Global Environment;
the Realm retains the same root through `GLOBAL_ENVIRONMENT`. Main entry uses
the allocator's nonnull typed result rather than reading an uninitialized
nullable Realm field. Intrinsic bootstrap, global lexical publication and global
object publication keep their original order. Source literal prototypes and
global Reference resolution still require the defining Realm from the terminal
environment; they do not substitute the active Realm when that edge is absent.

The original mixed Array lifecycle runtime fixture exposed this missing edge
after successful native compilation and instantiation: its first object literal
trapped when requiring the root's defining Realm. The finite Engine control
`aot_execution_global_environment::entry_global_root_preserves_literal_prototypes_and_captured_activation_cells`
checks entry-Realm Object/Array prototypes, escaping captures sharing the original
activation cell, and global function publication. This native control passes in
`tasks-main-root1` on 2026-10-07, together with the workspace type check. The
original lifecycle fixture now executes without the null trap, but fails its
output assertions because ordinary Await prematurely completes its activation.
That separate repair belongs to the async suspension return boundary.

Global var/function state is the global object's own property state.
[ECMA-262 PR 3226](https://github.com/tc39/ecma262/pull/3226) removed the separate
`VarNames` list. Consequently a configurable binding introduced by sloppy eval
may be shadowed by a later Script lexical declaration, as the pinned
`script-decl-lex-var-declared-via-eval.js` test requires.
