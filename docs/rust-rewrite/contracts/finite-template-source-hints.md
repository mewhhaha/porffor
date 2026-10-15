# Finite template, filtered-array and prepared-callable source hints

Optional AOT source discovery does not change emitted call, property lookup,
string conversion, filter, callback or realm behavior.

An interpolated template first uses the bounded Cartesian product of its existing
finite substitution text views. Every nonempty product remains unchanged,
including mixed choices that can arise after reassignment. Only when that
expansion is empty, including an oversized product, an additional bounded search
may reuse one text choice for repeated simple identifier holes resolving to the
same analyzed environment/storage cell. Distinct or shadowed cells and other
holes keep independent choices. Intercepted identifier references do not use this
fallback. It reuses the already discovered hole candidates, including captured
and supplemental text views, without rediscovering nested expressions.

These are supplemental source proposals, not proofs that live reads correlate.
Assignments, calls, getters and every substitution's Get and ToString still run
in their original order. A source produced by mutation or coercion must match
the complete live string, source kind and permitted active context before a
prepared unit executes. An unlisted source keeps the existing typed Wasm-AOT
capability rejection; the search neither folds runtime operations nor asserts
that its catalog is exhaustive.

A possible intrinsic String call can contribute all finite text views of its
first argument. A property call named `filter` can retain every possible
input-array element as a possible result element, matching the existing
conservative array callback hint protocol. These views are supplemental and
cannot displace an existing primary candidate.

The compiler does not run the filter predicate, select its true branch, prove
the returned object's identity, or replace the returned array. A modified method,
getter, proxy, species constructor or callback can produce values outside these
possibilities. Such a live value still has to match a compiled source of the
right kind and active runtime context; otherwise it receives the existing typed
Wasm-AOT capability rejection.

The 256-candidate/tuple limits apply to both searches. If the fallback's independent
choices also exceed the limit, it contributes no sources. Existing primary
sources and every fitting Cartesian product remain available. Unknown live
strings remain unsupported unless they match an admitted prepared source; no
parser or interpreter is added to emitted Wasm. Supplemental views also retain
the restriction against authorizing eager realm-module requests.

Controls cover primitive and nullish String views, chained filter callbacks,
the complete 256-template product and overflow, retained primary sources,
unknown values, live method Get/argument/conversion/throw order, a replaced
filter with observable callbacks, and typed rejection of an unmatched result.
Repeated-cell controls additionally cover captured eval and Function sources,
independent and shadowed cells, the unchanged 256-choice boundary, reassignment,
live getter/conversion effects and typed rejection of an unlisted mixed source.
Fixture coverage includes the unmodified pinned WrappedFunction/name.js and
evaluate/globalthis-available-properties.js files in both script modes.

A returned ShadowRealm callable can contribute another optional hint when the
prepared Script ends in an ordinary function expression or arrow literal. The
compilation-local prepared-source cache retains the exact parsed syntax owner
and that callable's existing AST-site key. It does not assign the callable a
FunctionId or publish a runtime identity fact. Possible calls to that source
contribute only finite argument text views to the retained owner and formal
position. Calls, getters, argument evaluation, string conversion, exceptions,
wrapper identity, Realm selection and closure environments still execute live.

The caller finishes collecting these requests before the existing recursive
prepared-unit lowering enters the child. The child resolves the AST-site key in
its own completed analysis, then seeds its own parameter hints before the
ordinary prepass and function lowering. No parent FunctionId, binding cell or
runtime object crosses this boundary. Independently parsed owners and different
source grammars cannot receive each other's requests. Equal prepared text may
share a parse and conservative text hints across several runtime Realms; this
does not share their live callable, lexical state or global environment.

Each formal retains at most the first 256 distinct requested texts. Requests
are monotone and supplemental; they cannot evict primary source candidates or
increase the existing candidate/tuple limits. The registry has one entry per
retained terminal callable site and adds no retry loop or dynamic-source work
queue. The existing prepared-source cache owns parsing, including deferred
errors. Supplemental argument text also cannot authorize eager importValue
requests. The executable dispatcher continues to require the exact live source,
source kind and permitted active context.

This bounded route covers direct calls and fixed identifier formals. Rest or
destructured parameters, unknown arguments, identifier or conditional return
values, factory calls, and async or generator literals do not acquire a guessed
cross-unit identity. Other forwarding forms may retain their existing support,
but this route makes no new call/apply claim. Unmatched dynamic source keeps the
existing typed runtime capability rejection.

Additional controls cover retained parse identity and grammar isolation, equal
AST spans in different owners, the literal whitelist, monotone 256-domain
capacity and rest exclusion, independent child source kinds, unknown return
values, primary-source retention, and import-request restrictions. Native
controls exercise equal source in distinct Realms with captured lexical state,
reassigned callees, live method/Get/argument/throw behavior, and unknown nested
source rejection. The two unchanged pinned nested-realms and nested
wrapped-function fixtures cover both modes. Acceptance requires actual native
execution of these controls. Historical eight-execution NI evidence remains
separate from current-source replay results.
