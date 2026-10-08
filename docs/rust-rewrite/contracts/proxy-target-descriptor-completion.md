# Proxy target descriptor completion

Status: authored dry source. Compilation, Wasm validation and execution are
pending; this is a shared invariant correction within T11.

The common direct descriptor authority retains its existing ordinary, namespace,
Array, arguments, TypedArray, boxed primitive and Function projections. Its
Proxy branch now invokes the actual existing Object.getOwnPropertyDescriptor
owner, including recursive target hooks and descriptor validation. The private
completed descriptor object is never exposed to user code. Its own data fields
supply the existing Fact, Get or Set projection without invoking property values
or adding another descriptor algorithm. The same target-descriptor factory
continues to serve the public Proxy descriptor algorithm's compatibility check.

Each of the four outer invariant consumers acquires a completed projection;
a reserved raw projection cannot enter that consumer boundary. A normal Get
trap result reaches target GetOwnProperty before SameValue/missing-getter checks.
Set false returns before lookup; Set true reaches the target descriptor before
frozen-data/missing-setter checks. Has true and Delete false return before lookup.
Has false and Delete true retain presence independently from descriptor bits;
an absent descriptor skips outer IsExtensible, while a present nonconfigurable
property fails before that query. Original target throws leave the outer
constraints unobserved and preserve the original completion.

These ordering requirements follow
[ECMAScript2026 Proxy clauses10.5.7–10](https://tc39.es/ecma262/2026/multipage/ordinary-and-exotic-objects-behaviours.html#sec-proxy-object-internal-methods-and-internal-slots-hasproperty-p).
Literal global declaration descriptor acquisition retains its existing direct
Fact path and declaration policy.

Actual callers use the existing trusted Proxy execution or set-path Realm
projection. The planned ordinary builtin entry and private input encoder carry
that projection into the real descriptor call. A lexical environment is never
interpreted as a native Realm record. Planning retains the descriptor builtin
for every emitted caller, including source operators and outlined helpers.
Nested descriptor errors and outer Get/Delete invariant errors use the executing
operation's intrinsic TypeError. Inline source Delete propagates generated
errors into the active catch/finally handler; builtin/helper completions retain
their existing return behavior. Existing original user throws remain unchanged.

The strict/sloppy Engine controls require actual WasmAot, normal Number262 and
one final print. They cover all four operation orders, String/Symbol key identity,
original target and outer-trap throws, early Boolean outcomes and missing
properties, frozen/accessor and SameValue NaN/signed-zero constraints. Both
borrowed Reflect Realm directions and source operators retain native error
prototypes after public foreign intrinsic bindings are replaced. These controls
are authored and unexecuted. Current scalar layouts, semantic GC migration,
full descriptor-lattice obligations and complete T11/Test262 acceptance remain
separate open work.
