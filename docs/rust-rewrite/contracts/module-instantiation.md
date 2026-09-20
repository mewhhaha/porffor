# Canonical module instantiation and global Script preludes

The compiler allocates each eligible module's environment before installing any
import or evaluating any module body. Imports refer to canonical exporter cells;
namespace readers resolve those same cells, preserving live values, TDZ errors,
and immutable import writes.

## Admission boundary

`ModuleInstantiationGraph` is the sole source-construction witness. It accepts
Module-entry graphs without source-phase requests, including local top-level
await and transitive asynchronous dependencies. Ordinary and deferred imports
use the same canonical owners. Original request phases and order survive linking;
runtime traversal owns cycle and async dependency state.

Script entries and source-phase graphs retain their separate driver and explicit
capability boundaries. Retained Module-entry drivers use a private lexical arrow
owner, including an async arrow when required. Their declarations stay outside
the independent global Script, but those drivers still share declarations between
module units and retain their global import-alias limitations.

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

Statement-list, classic-for and for-of synchronous resource declarations require
an enclosing canonical ModuleActivation or AsyncModuleActivation owner. Nested functions retain their own
source execution lifetime under that module owner. Classic-for and for-of heads
still require immediate execution; statement-list scopes retain the existing
supported resumable function lifetimes. Retained source-phase drivers have
no canonical module owner and keep their explicit resource admission gap.

`FunctionProtocolIr::AsyncModuleActivation` uses ordinary async function
resumption with closed Allocate, Instantiate and Execute entry modes. Allocation
initializes the canonical invocation environment but executes no source body.
Instantiation hoists declarations, installs import cells and publishes namespaces,
then stops at a private 0-to-1 boundary without settling its body Promise or
creating an await job. Evaluation begins at state 1. Subsequent source Await and
resource scopes use the existing async execution protocols.

The record owns the canonical environment separately from the current suspended
lexical chain. Async invocation environment offset 144 remains canonical; offset
80 is saved at every suspension, including private import waits. Resuming a nested
block never changes the environment used by live imported cells.

Every record retains its activation, source function, namespaces, request vector,
Realm, runtime DFS state, cycle root, async parent occurrences, owned Evaluate
Promise and exact completion discriminator/payload. Module records and their
pointer-bearing backing records are included in the passive root inventory.
Imports resolve to the same canonical cells as namespace readers, retaining TDZ
and immutable binding checks.

The runtime evaluation and deferred readiness rules are specified in
[async module lifecycle](module-async-lifecycle.md). Static SCC lists do not
schedule execution. Evaluation starts from the entry, and dynamic-only targets
start from their import continuation. Errors are cached only on entered members;
independently completed components retain their outcomes.

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
ordinary TLA and retained Script/source-phase controls remain in the focused verification set.
No full-suite status count follows from this bounded stage.

Async activation, raw import waits, readiness, cycle occurrence counts, exact undefined rejection and graphless artifact boundaries are covered by `aot_module_async_lifecycle` and `module_async_runtime_tests`.
