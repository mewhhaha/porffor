# Whole plain async ForIn

Plain Async ForIn in the current FunctionBody graph owns its complete head,
advance, body and exit. Nested With and ForIn retain that graph; foreign classic
and ForOf loop regions preserve their original routes. AsyncGenerator and
suspended per-key heads remain explicit separate domains. The Annex B
WebCompatCall head retains its original eager throwing path.

Both Generator and Async source facades consume one actual eager head classifier
and `GeneratorForInHeadProof::from_source`. `ForInSourceIdentity` can be minted
only from the original ForIn AST. The checked mode, BoundNames, scoped lexical
storage mapping and real ignored immutable Identifier write evidence remain
bound to that identity. No repeated name-resolution or copied immutable policy
stands in for the original per-key Reference.

The async source uses the existing expression planner and the actual shared
With/ForIn statement tape. It reserves complete If, Try, Switch, labelled,
Array-pattern and nested enumeration ranges. Await identities must match the
lowered carrier. Eager enclosing branches still own zero-Await ForIn phases;
foreign eager loops cannot replay unowned child phases.

The head runs once under its original lexical TDZ record and publishes its raw
value into an actual invocation-owned cell. The native owner leaves that TDZ
record before creating the original enumerator. Each selected String key runs
the same original eager initialization block, with the original per-iteration
lexical cells. Four distinct actual invocation cells retain the raw head,
native cursor edge, selected key and persistent StatementList value. Normal
Await keeps the real cursor and lexical records. Committed outward completions
retire the cursor through the existing whole-completion cleanup path after
nested finalizers. ForIn introduces no IteratorClose protocol.

The physical native enumeration authority continues to own lazy accepted own
keys, descriptors, visited names, prototypes, primitives and nullish heads.
Source controls assert original records, exact ranges, nested owners and foreign
scope refusals. Real JS-to-Wasm strict/sloppy fixtures cover deletion and
nonenumerable shadowing, captured per-key cells across GC, patterns and property
targets, Reference selection before Await, nested With/Array owners, labelled
break/continue and whole abrupt values through awaited finalizers. These controls
are authored and unrun; compilation and runtime verification remain required.

This batch adds no GC schema, JavaScript runtime representation, interpreter or
fallback backend. It uses the existing experimental Wasmtime GC/reference path.
