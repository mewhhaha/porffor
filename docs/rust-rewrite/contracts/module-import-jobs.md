# Contract: canonical Module-entry dynamic imports

Canonical Module-entry graphs, including top-level await, use these continuations.
Graphs with source-phase requests or a Script entry keep their existing driver
and capability boundaries.

All graph activations and namespace identities are created before evaluation.
Only the entry starts initial evaluation; it walks its static evaluation
requests in source order using runtime DFS, cycle-root capabilities and cached completion.
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
settles in the first continuation. The generated driver uses ordinary Await of undefined for the load boundary.
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

The loaded source closure still contains every discoverable request and
participates in artifact cache identity. A successful graph takes the existing
single linking pass. When an admitted canonical graph has a language rejection,
`modules/admission` validates its static entry closure first, then each reachable
dynamic root's static closure. Failed dynamic-only roots become explicit
request-owned rejection records; their invalid bodies never become activations.
Successful shared dependencies are retained when another valid root reaches
them. Projection preserves agreeing duplicate host keys and rejects host
contradictions. Unresolved direct dynamic requests keep the existing TypeError
fallback for requests outside the compiled registry.

Ordinary dependency parser rejections carry `E_MODULE_SYNTAX`, SyntaxError and
resolution phase. Recognized early-error codes retain their existing identity.
Caught parser aborts, host contradictions, source limits and unsupported drivers
remain compiler failures. A static path to an invalid target remains a compile
failure even if a dynamic path names the same target; no test body or global
Script prelude runs to manufacture the expected negative result.

`module_import_jobs` checks closure ownership and retained capability boundaries.
`aot_module_import_jobs` checks body timing, two versus one reaction ordering,
repeated imports, eager/deferred namespace identity, cached abrupt identity,
missing/malformed/missing-export dependencies, intrinsic independence, coercion
order and compile-only static syntax rejection. These focused targets do not
establish full-suite conformance.
