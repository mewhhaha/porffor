# Native Function P factory and pure owner bootstrap (ABI6 component)

Status: **UNAPPLIED, NON-MERGEABLE, NATIVE ACCEPTANCE PENDING**. This is a later
selected-file component, separate from the sealed ABI5 repair checkpoint. It
requires the sealed IR, ProgramInstance, literal, typed-hook and entry components.
No source root, HEAD or real index was changed. No Cargo, rustc, Wasm/native
execution or Test262 command ran for this component.

## Actual request and runtime identity

`emit_runtime_function(&RuntimeFunctionProgram, &Arc<RuntimeArtifact>,
&IntlCompilationProfile)` emits a fresh linked P only. The opaque IR witness owns
this request's independently parsed/lowered FunctionId and kind; its first actual
PreparedDynamicFunction row is checked again against the completed AOT registry.
The factory export aliases that P's actual registered PreparedDynamicFunction
helper. It cannot name an arbitrary root function, unrelated Script or R body.

The adapter directly passes ModuleKind::Program(the supplied runtime) to physical
module assembly. It never asks ordinary emit() to select/rebuild R and never
creates an Engine, Store or Realm. RuntimeLayout supplies actual R function/global
imports and verifies the pool boundary. Physical Intl identity, service-selection
and component frames are compared with the actual immutable R bytes, including
same-name custom-profile differences, missing/duplicate frames and parser errors.
The returned artifact retains the exact same immutable Runtime Arc, key and raw
bytes; its P byte slice and auxiliary export names are admitted together. Native
engine/configuration admission remains owned by the existing host integration.

## Dedicated P bootstrap

Every linked heap P receives a separately typed `lila.program_init` export:
`[] -> (ref ProgramInstance)` with a non-null result. Runtime R and heap-free
Standalone keep their previous function layouts. FunctionIndexLayout alone mints
ProgramInitializeIndex, places it immediately after main, shifts P source/helper
indices and supplies the prefix count. No external source does index arithmetic.

The package compiles the initializer before main against its own sealed GC
schema, stores its completed body/index together, and emits runtime bodies,
main, initializer and P callables/helpers in their planned order. The private
CompletedProgramInitializeBody token can only be created by the dedicated
bootstrap compiler. EmittedFunction consumes that token; ModuleCode asserts its
actual emitted position and declares StaticSignature::ProgramInitialize. The
bootstrap is not an ordinary main/Completion or JavaScript callable entry.

A real closed ProgramInitialize builder role starts a fresh zero-argument owned
body without an entry owner. It initializes genuine R literal/Symbol roots under
the R-only guard, then unconditionally produces fresh P literals, immutable P
metadata and typed hook references, publishes one ProgramInstance and returns it.
The result remains rooted on the Wasm operand stack while the temporary local is
cleared. It performs no main/source/Realm initialization, JS parsing/evaluation,
shared P publication, P MemoryInit or transient-cursor reset.

Ordinary P main calls its own registered initializer once, immediately roots the
nonnull result through RuntimeSchema::call_program_initialize, and binds it before
Realm/global environment, source function or capture initialization. P literal
publication is allowed only from the dedicated bootstrap role; an R guard cannot
suppress a later P owner. Generic JS compilation rejects the bootstrap role.

## Native miss and actual constructor effects

Each constructor argument is converted to String exactly once, left to right,
through the existing JavaScript conversion path. Abrupt conversion exits before
prepared/native compilation. A matching prepared hook still owns its completion,
including a prototype-getter abrupt result. Every prepared miss, including zero
arguments, invokes CompileDynamicFunction once with the original kind, defaulted
new.target, converted GC String array, active constructor FunctionContext and
caller environment. There is no empty-body or rejection fallback on this path.

The host import takes seven scalar words and returns the original five Completion
words plus a nullable GC String syntax message. Its typed bridge roots that top
message first, then stores the complete Completion. Message null copies the
native factory result unchanged. Message non-null is reserved for genuine source
syntax/early-error rejection; the Wasm consumer allocates SyntaxError with the
active constructor Realm's SyntaxErrorPrototype and that GC String directly.
Capability/resource/internal errors are explicit host/compiler failures and must
never acquire a fake SyntaxError classification.

The host instantiates these P bytes in its existing Store against actual current
R, calls program_init, then calls the exported PreparedDynamicFunction helper with
that owner followed by the original seven operands. It never calls P2 main.
Existing generic function allocation owns constructor Realm/global environment,
new.target prototype getters, source identity and abrupt completions. The factory
matched/status result remains separate from Completion, and actual-request
matching must return 1. Deadline/epoch and native-cache policies are unchanged.

## Required joins and remaining limits

The derived registry closes the 20 additional actual-R bytegraph rows, preserving
the sealed 36-row packet. Its 56 R rows use explicit final caller_program:Program;
P-owned CompactArrayLiteral and all eight ordinary module helpers have the same
required operand. Module record helpers pass their retained graph P. Hook semantic argument
constructors remain unchanged because their first owner comes from InstalledHook.
Root owns transitive caller edits; module/job owners must pass retained P where no
calling source frame remains. No ambient-P or optional-owner fallback exists.

The separately sealed metadata reader migration is a required dependency of the
writer-removal derivative. Numeric/module/RegExp/recipe readers use checked
P-relative ImmutableByteArray spans and explicit R/P literal domains. JS main no
longer installs P bytes into shared linear memory or resets R's transient scratch
cursor; the legacy_static_data child is removed. program_init publishes complete
P roots and immutable metadata, and main binds that owner before Realm/source
entry. R's active data prefix, fixed runtime tables and aligned transient scratch
allocator remain. P-owned passive segments still supply GC arrays.

The native host, 56-row caller closure, retained jobs/modules/activations, complete
metadata reader and writer-removal derivatives, ABI/cache/schema admission must
join before a native compile and acceptance. The written controls cover layout
ordering, wrong bootstrap index, serialized P bootstrap/factory signatures,
physical profile rejection and the closed private builder state. They are unrun
Rust tests. Structural controls, formatting and patch replay are weaker evidence.

Preserve the exact passing same-Store P2-to-P1 forced-GC function42 transport and
rerun the exact failing old-P hook conflict: P2's Function('return 23') must leave
P1's old Function('return 17') returning 17. Then cover ordered coercion/reentry,
prototype-getter throws, actual foreign constructor Realm/global environment,
syntax versus capability failures, zero-argument and four constructor kinds,
colliding UTF-16 literals/metadata, reverse entry, retained jobs and forced GC.
No acceptance or dynamic-code-generation conformance claim is made here.
