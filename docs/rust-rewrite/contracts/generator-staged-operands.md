# Ordinary generator staged selectors and Identifier assignments

An ordinary logical selector retains its complete checked source continuation.
The source allocator consumes that prefix before reserving selected RHS resumes
and a join. Lowering consumes the same original selector through existing staged
expression owners, then retains its actual GetValue in a GC activation cell. A
selected yielded RHS uses the existing scalar branch owner; an eager RHS uses
ordinary short-circuit IR after the completed prefix. Neither path re-evaluates
the selector on resume. Conditional selectors and optional-chain base rules keep
their existing separate admission.

A suspended compound Identifier operation admits the twelve closed arithmetic
and bitwise assignment operators and all three logical assignment operators.
The private Reference owner selects the actual declarative cell, runtime named
Environment Record, global Object Record or ordered with binding object once.
Capture performs GetValue before the RHS prefix. The whole old operand stays in
its ordinary activation value cell without mutable shape facts. The separate
`CapturedIdentifierReferenceIr` slot retains the actual selected Reference in
`BindingCell.CAPTURED_IDENTIFIER_REFERENCE`, an immutable native GC record with
key, closed kind, record, named entry, exact declarative cell and binding object.
It never becomes a JavaScript value or another object representation.

After eager RHS completion, the existing binary owner evaluates both operands
before ordered coercion. `PutCapturedReference` restores the saved Reference and
retires its private slot; it never restarts ResolveBinding or HasBinding. The
original Object Record still performs its own SetMutableBinding property check,
with the original strictness. The original declarative cell keeps actual TDZ,
mutable/immutable and named-function-self strictness metadata.

A logical assignment owns a complete selected RHS source plan through the
existing checked ordinary-generator branch regions. Several yielded arguments,
nested admitted selectors and their joins remain inside that selected region.
The skipped branch keeps the original whole GetValue result, retires the private
Reference with `ReleaseCapturedReference`, and does not perform PutValue. The
selected branch writes only after its complete RHS. Both branches consume the
same allocated Reference slot. Async and async-generator continuation admission
is unchanged.

The source controls retain original selector/operand/loop checks and now verify
actual native captures for global, with and runtime-visible names. Complete
logical RHS regions are checked against their suspension points and skipped
Reference release. The Engine cohorts cover strict/sloppy global property
changes, coercion order, captured cells, all logical skip predicates, multiple
RHS yields, const writes, genuine unresolvable/TDZ throws, original with selection
across unscopables/deletion changes, GC and whole injected Return/Throw identity.

The native Reference field is retired on actual Throw transfer and committed
Return before an abandoned RHS reaches a handler or finalizer. The completed
invocation can retain its activation for closures, so dropping only Wasm locals
would leave this edge alive. The cleanup visits the actual owned activation cell
table and clears only `CAPTURED_IDENTIFIER_REFERENCE`; existing JS values and
other private fields retain their own lifecycle. Normal Yield and delegated
`return()` results with `done: false` keep live References. Normal PutValue and
logical skips retire their own exact slot. Extended cohort controls cover a
caught RHS throw followed by a yielding catch and fresh assignment, pending
Return/Throw across yielding finally, and delegated incomplete Return followed
by normal completion.

Source authoring and isolated formatting are not execution evidence. Compilation,
focused regressions and broad verification remain mandatory and pending.
