# Complete resumable ForOf and ForAwait

Generator, Async and AsyncGenerator consume one complete source/lowering/native iterator pipeline. The analyzed token retains the actual ForOf AST identity and closed execution protocol. Unsupported foreign iterator/resource ancestry keeps its original source owner.

The head runs under the original lexical head TDZ. Acquisition stays outside the iterator close scope. One original InvocationFrame iterator-table row retains the IteratorRecord and cached next method; a separate owned incoming cell survives complete per-key target evaluation and defaults. The original per-key environment remains attached through initialization, body suspension and disposal. Normal continuation never re-advances a partially initialized key.

Shared regions preserve their original protocol validators. The typed source and IR suspension census keeps implicit ForAwaitNext and ForAwaitClose points distinct from explicit Await, including nested complete loops and clauses. Generator Yield values retain the original unadopted ordinary-generator semantics. AsyncFromSync uses the existing native adapter.

A resource head owns one original capability for each iteration, spanning registration and body. Disposal finishes before advancing or closing the outer iterator. Awaited resource disposal uses the original Async or AsyncGenerator continuation driver according to the actual source protocol. Bare registration operations remain refused outside the checked resource initializer.

Retargeted IR controls retain original source strings and assert physical initializer targets, original capture/TDZ environments and genuine owned cells. Three strict/sloppy Engine cohorts cover raw head/target ordering, cached methods, nested close/rejection identity, captured environments, GC and per-key disposal ordering. `tasks-iterator-intrinsics3` passes all three native controls in both modes after the input-alias and initialization-scope repair. Workspace compilation passes; broader and pinned conformance remain open.
