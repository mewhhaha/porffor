# Synchronous module instantiation and global Script preludes

The compiler allocates each eligible module's environment before installing any
import or evaluating any module body. Imports refer to canonical exporter cells;
namespace readers resolve those same cells, preserving live values, TDZ errors,
and immutable import writes.

## Admission boundary

`SynchronousInstantiationGraph` is the sole source-construction witness. It
accepts Module-entry graphs without top-level await or source-phase requests.
Ordinary imports and deferred imports use the same canonical owners; static
evaluation cycles and deferred-readiness cycles are admitted. Script entries,
asynchronous graphs, and source-phase requests remain on the existing driver.
Their remaining scope and lifecycle cases are not complete.

Retained Module-entry drivers run their merged body in a private lexical arrow
owner, using an async arrow for TLA. This keeps their declarations separate from
the global Script environment and introduces no implicit `arguments` binding.
Those drivers still share one environment between module units: a dependency's
free name can incorrectly resolve to an entry Module declaration. Per-module
scope isolation in retained TLA/source-phase drivers remains unresolved. Their
renamed import aliases also retain global accessors that can conflict with Script
bindings. The separate global Script boundary does not repair those cases.

The original source is parsed and linked as Module code before the compiler's
private driver is parsed. Generated arrow syntax cannot authorize source-level
`return`, `new.target`, `yield`, or sloppy `with` statements in a Module.

## Allocation, instantiation, and evaluation

`FunctionProtocolIr::ModuleActivation` has lexical arrow semantics and the
existing generator call ABI. It has no `[[Construct]]` and no public generator
source syntax. The generator's INITIALIZING call allocates all environment
cells, without executing the body. After every owner exists, one private resume
hoists declarations, installs imports, publishes namespaces, and suspends at a
compiler-owned IR boundary. Evaluation resumes past that boundary.

The original module body has Ordinary source execution even though its private
owner uses the generator ABI. Only trusted boundary metadata emits the fixed
0-to-1 suspension; source `try`/`catch`/`finally`, loops, and synchronous resource
scopes cannot allocate additional resume states. Consequently, repeated catches
and finally blocks run on every iteration and do not skip following statements.
Resources acquired during evaluation never cross the instantiation suspension.
The source execution plan supports that immediate lifetime, but checkpoint
twelve still reaches the older blanket module-resource admission guard; its
resource regressions remain failing until that guard uses canonical ownership.

A runtime module record retains its activation, evaluator, eager and deferred
namespace cells, lifecycle state, and cached thrown value. These records are
registered heap roots. Export cell addresses point into the canonical activation
environment; an importing environment cell stores one resolved indirect target.
Writes check the importing binding's initialization and immutability before
consulting its target, so assignment to an initialized import throws TypeError
even while the export remains in TDZ.

Each immutable evaluation plan carries the module, its evaluation-phase
dependencies in request order, and its complete static evaluation component.
The evaluator marks its own record Evaluating and visits dependencies before
running its body. Reentering an Evaluating record does not execute it again.

The first active evaluator in a component owns completion. Its flag remains in
a Wasm local while nested synchronous evaluators run; no second module heap
layout or runtime DFS index is needed. Completed members keep Evaluating state
until that owner finishes. On success, all entered members become Evaluated. On
failure, every entered member caches the same thrown value, including members
whose bodies already completed. Members not yet visited remain Linked: a later
evaluation must still visit their earlier dependencies before encountering the
cached failure. Other components that already completed stay Evaluated.

Initial static traversal starts with the entry evaluator. Pre-evaluating its
dependencies would change the DFS root for an entry cycle and could move an
external dependency ahead of a cycle member whose body should run first.

Deferred access first performs a pure readiness traversal with a seen set over
all requested dependencies. Evaluated and errored records stop traversal;
encountering an Evaluating record throws TypeError. Only successful readiness
can invoke the evaluator. Deferred-edge cycles terminate without eager body
execution. Namespace publication and dispatcher reads use typed private cells,
without creating source-visible namespace aliases.

Existing dynamic import dispatchers retain their Promise construction,
specifier/option evaluation, attribute matching, and rejection behavior. Their
generated namespace reads lower to private cell operations. Ordinary dynamic
import scheduling retains the existing eager behavior; this is not a replacement
for the asynchronous module job scheduler.

## Independent global Script before a Module

`CompileOptions::module_prelude` supplies an independent global Script in the
entry Module's realm. The engine parses both source goals separately and rejects
the option for a Script entry. The prelude source participates in the framed
program-Wasm cache key. Its functions and environments share the compiler's ID
allocator with the graph.

The prelude lowers through the existing prepared RealmScript and runtime
GlobalDeclarationInstantiation machinery. AOT main calls its private Script thunk
before module allocation and evaluation. The thunk is outside dynamic eval source
candidate tables. A throwing prelude prevents Module evaluation. Module lowering
starts without fresh-realm assumptions, because the prelude may replace globals
or intrinsic properties. The spec-exec oracle evaluates the Script separately in
the same context.

Test262 materialization places canonical helpers and includes in this Script,
while retaining the original Module source bytes for compilation, execution, and
negative preflight. Module declarations keep their lexical scope, and dependency
modules see the global harness through ordinary global binding resolution.
Raw Modules retain exactly their original bytes and have no prelude. There is no
compiler exception for a harness identifier or Test262 path.

## Verification boundary

Focused coverage lives in `lila-ir/tests/module_instantiation.rs`,
`lila-engine/tests/aot_module_instantiation.rs`, and
`lila-engine/tests/aot_module_scope_isolation.rs`, with engine parse/cache and
Test262 materialization/runner controls. The cases cover deferred dependency
order, cyclic readiness, evaluation reentry, TDZ, live immutable imports, cached
throw identity, repeated synchronous catches and finally completions, per-iteration
catch closures and resource disposal, nested import closures and live reassignment,
namespace identity,
lexical module this, default function names,
all ECMAScript line terminators, independent Script strictness/global scope,
prelude throws, cache identity, and raw/negative source-goal separation.

The scope regressions cover dependency free-name resolution, absent global alias
properties, independent Script lexical and var bindings, same-spelled module
bindings, entry-rooted cycle order, hoisting and import TDZ/immutability, deferred
access to a completed-but-still-evaluating member, shared late-error identity,
and unvisited members after an earlier failure. Namespace initialization and
retained TLA/Script/source-phase controls remain in the focused verification set.
No full-suite status count follows from this bounded stage.
