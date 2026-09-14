# Deferred module evaluation completions

The active source linker compiles each deferred module body into its own private function. `deferred_body_source` now emits a separate evaluator with four closed states: unstarted, evaluating, evaluated, and errored. The first request changes the state to evaluating before invoking the body. Recursive requests reuse the already-published export readers. Successful completion is retained, and an abrupt completion stores its original JavaScript value before marking the module errored. Every later evaluation request rethrows that value, including `undefined`, without running the body again.

The exception boundary surrounds the body invocation. The module body and its export-reader closures retain one function declaration scope, so the repair does not move lexical declarations under a generated try block or change function declaration instantiation. Anonymous default definition tracking follows the private execution function and keeps the original user-visible name and source text.

Namespace internal methods continue to decide whether an operation requests evaluation. Symbol reads and the deferred `then` exclusion retain their existing behavior. An error thrown by a user getter after a successful namespace binding read is outside module evaluation and does not poison the cached module completion.

`aot_module_namespace` covers repeated primitive, symbol, and object failures; reflection after failure; reentrant reads and caught TDZ errors; hoisted functions; anonymous defaults; and errors from later user getters. Its cross-realm fixture explicitly uses the Test262 host policy because creating a realm is a harness capability. Product defaults are unchanged.

This repair does not complete module allocation and instantiation before evaluation, per-module indirect import cells, or dependency and asynchronous cycle scheduling. The larger staged activation rewrite remains unfinished and is not required or integrated by this patch. No full-suite conformance increase is claimed before replay.
