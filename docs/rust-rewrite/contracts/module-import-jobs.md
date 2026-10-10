# Contract: canonical Module-entry dynamic imports

Current source implementation, 2026-10-03: Module and Script graphs use the
canonical activation graph, including top-level await, deferred requests and
cycles. Script roots retain ordinary Script completion; import jobs start their
module targets. Source-only JavaScript targets load and parse, then static source
bindings/forwarded exports fail linking and dynamic `import.source` rejects with
native SyntaxError before target dependencies or evaluation. No fabricated source
cell or retained product driver remains. See the
[source-phase rejection contract](source-text-module-source-phase-rejection.md).
Executable acceptance of this retirement is pending; earlier focused evidence
below does not verify the new source implementation.

All graph activations and namespace identities are created before evaluation.
For a Module root, only the entry starts initial evaluation; it walks its
static evaluation requests in source order using runtime DFS, cycle-root
capabilities and cached completion. Script roots start targets through import jobs.
A dynamic-only target has no observable body effects until an import job reaches
its evaluator. Repeated imports create distinct promises but share the module's
namespace and evaluation completion, including exact thrown-value identity.

Compiler-owned async dispatchers preserve operand evaluation at the call site.
Their intrinsic promise exists before specifier conversion and options reads,
which execute synchronously. Reflection and error operations carry closed
catalog identities through trusted source-position metadata. Generated intrinsic
calls are values, not environment references, so an eval-visible environment
cannot reinterpret their private names.

The [ContinueDynamicImport algorithm](https://tc39.es/proposal-defer-import-eval/#sec-continuedynamicimport)
first attaches a reaction to LoadRequestedModules. For ordinary evaluation it
then attaches a second reaction to Evaluate's promise, including when a
synchronous body throws. A deferred import with no asynchronous dependencies
settles in the first continuation. The generated dispatcher uses ordinary
Await of undefined for the load boundary.
Its evaluation wait accepts an owned intrinsic Promise record directly, without
PromiseResolve, constructor, species or then observations. Deferred imports gather
async dependencies, start every selected Evaluate operation before attaching any
join reactions, and wait for all successful occurrences; the join retains the
first exact rejection. An empty gather adds no evaluation wait. Ordinary source
Await continues to perform its specified PromiseResolve operation. Cached
evaluation throws survive the second continuation.
Direct host-load syntax rejection can settle immediately; transitive loading or
link rejection belongs to the first continuation. These are distinct paths,
not an unconditional delay applied to every outcome.

Loaded sources retain their original requests and participate in artifact cache
identity. A Source occurrence loads/parses its target without opening that
target's dependencies; Evaluation/Defer can later open the cached target. Graphs
without source jobs or language rejection take the existing single linking pass. When an admitted canonical graph has a language rejection,
`modules/admission` validates its static entry closure first, then each reachable
dynamic root's static closure. Failed dynamic-only roots become explicit
request-owned rejection records; their invalid bodies never become activations.
Successful shared dependencies are retained when another valid root reaches
them. Projection preserves agreeing duplicate host keys and rejects host
contradictions. Unresolved direct dynamic requests keep the existing TypeError
fallback for requests outside the compiled registry.

Ordinary dependency parser rejections carry `E_MODULE_SYNTAX`, SyntaxError and
resolution phase. Recognized early-error codes retain their existing identity.
Caught parser aborts, host contradictions, source limits and internal compiler
failures retain their compiler-failure route. A static path to an invalid target
remains a compile failure even if a dynamic path names the same target; no test body or global
Script prelude runs to manufacture the expected negative result.

`module_import_jobs` checks closure ownership and canonical capability boundaries;
the maintained source-phase controls now require rejection for JavaScript targets.
`aot_module_import_jobs` checks body timing, two versus one reaction ordering,
repeated imports, eager/deferred namespace identity, cached abrupt identity,
missing/malformed/missing-export dependencies, intrinsic independence, coercion
order and compile-only static syntax rejection. These focused targets do not
establish full-suite conformance.

## Computed imports in a loaded closure

A computed import may select an exact request key already discovered from the
same referrer's source, including a static deferred edge. Its actual call site
still owns the Evaluation/Defer phase and runtime attribute validation. Source
requests and host resolution keys remain separate typed domains. Additional
host rows without a discovered source request remain unavailable in a loaded
closure; complete catalog admission keeps its separately declared scope.

The cloud checkpoint exposed this gap in the original ordinary and mixed
async generator import controls: raw operands resumed correctly, but the
request dispatcher omitted their already loaded `./value.js` target. Admission
and component discovery now consume the same source-discovered key projection.
The graph regression checks phase, referrer and catalog boundaries, while the
original native controls retain operand order, promise and namespace identity,
one module evaluation and abrupt-value checks. See the
[cloud continuation receipt](../cloud-continuation-20261009.md).
