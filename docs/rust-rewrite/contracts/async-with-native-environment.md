# Plain async With native environment lifetime

The checked `AsyncFunctionWithIr` and `OrdinaryGeneratorWithIr` carriers enter
one native With pipeline through a private closed `ResumableWith` owner. Each
entry requires the matching actual function execution kind and its exact
allocated head binding. Async generators cannot enter this scope. The original
generator path retains its operand, entry, body and cleanup instruction order.

The complete head compiles in the outer environment, including every Await and
the sole ToObject publication into the original invocation cell. A shared
`compile_resumable_operand_region(block, entry)` suppresses surrounding generator
StatementList checkpoints during only this operand evaluation and restores the
previous compiler context even if child emission fails. The existing generator
operand facade consumes this same physical helper. A rejected head Await or
ToObject failure never enters the With environment or its own cleanup scope.

Fresh body entry allocates the original analyzed Object Environment. A resumed
body reattaches the same physical child from the saved lexical chain. Its hidden
BindingCell receives the retained head object only on fresh entry; it does not
replay boxing, environment allocation or object publication after Await. With's
UpdateEmpty supplies Undefined on that fresh entry. Closures retain the same
real lexical and object records rather than a JavaScript environment encoding.

The native body reconstructs its cleanup block before resumed Await can inject
a rejected whole completion. Nested finally, disposal and Array IteratorClose
scopes execute before this outer cleanup. Pending Normal Await exits through
the existing suspension machinery with the With lexical record still saved;
it does not leave the environment. Every committed outward completion reaches
the common cleanup, which advances the exhausted state, leaves the original
child environment, saves the actual outer environment and then dispatches the
whole completion unchanged. Catch, finally and labelled destinations therefore
observe the restored parent. The existing captured Identifier Reference cells
remain selected across Normal Await and retire through their established Put or
committed abrupt lifecycle.

The implementation adds async entry/exit discovery, direct lexical traversal
and statement dispatch to the existing native parent. It adds no GC fields,
environment model, JS tag, backend fallback or independent interpreter. The
existing Wasmtime experimental GC, typed references and exception capabilities
remain required.

Two authored actual compiler-artifact controls consume the complete async With
environment and Reference semantic cohorts. They validate emitted Wasm and
follow Environment, ObjectEnvironment, BindingCell and InvocationFrame edges,
including saved lexical owner reads/writes and retained Reference publication.
The source and controls are uncompiled and unrun in this implementation batch;
joined compilation, focused regressions and broad verification remain required.
