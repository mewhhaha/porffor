# Property reads and live intrinsic facts

The 2026-10-07 source batch gives primitive property reads and acquired method
callees the same `lower_primitive_property_key` owner. Boolean, Number, BigInt
and Symbol reads, all five primitive method-call receivers, and ordinary String
property reads consume it. String length retains its immutable own-property
rule; computed String keys retain the original exotic-key classifier.

The governing operations are [GetV](https://tc39.es/ecma262/2026/multipage/abstract-operations.html#sec-getv)
and the [Symbol prototype properties](https://tc39.es/ecma262/2026/multipage/fundamental-objects.html#sec-properties-of-the-symbol-prototype-object).
Boxing selects the lookup object while the original primitive remains the
receiver. An inherited accessor runs when the property is read, even when the
result is never called. A method's Get precedes its argument evaluation.

`IntrinsicMethod` still has one private factory. Named and Symbol-keyed methods
both require its current prototype proof; Number/Boolean deleted-toString
tracking participates in that factory. An unproven method keeps an open callee
set including the original builtin, but observes possible property hooks and
invalidates captured value/shape facts before subsequent expressions are
lowered. Uncatalogued BigInt methods now use ordinary lookup instead of rejecting
an otherwise valid call. The runtime Get, primitive receiver, arguments and
whole abrupt completion use the existing emitted call pipeline.

Symbol descriptions require the original getter in the live prototype shape.
The known well-known Symbol case retains its String fact; other Symbols admit
undefined as well. Replaced getters use ordinary result inference and effects.
Symbol `constructor`, Symbol `@@toPrimitive` and String `@@iterator` no longer
receive an unconditional builtin fact solely from the spelling of the key.
Out-of-range String indices can consult prototype accessors, so computed-index
lookup cannot claim that a numeric key alone excludes user code.

Object and Array shapes now carry `HeapShapeProvenance`. Intrinsic prototype
snapshots name their actual constructor, including constructors in the Intl and
Temporal namespaces; internal prototypes without a tracked live owner are
explicitly untracked. The current-property owner compares each prototype's own
descriptor with the live recorded descriptor and validates each inherited link
separately. It never recovers precision by comparing two stale inherited chains.
Unproven reads retain open possible callable targets for code generation.

Fresh RegExp objects have an instance shape with an own `lastIndex` and a
separate prototype. Typed arrays do not invent own `constructor` properties.
Fresh Date, collection, buffer/view, boxed primitive, Intl.Locale, Temporal and
iterator shapes no longer restore pristine prototype assumptions after a
mutation. Boxed String's iterator catalogue names the actual String iterator.
Named Array reads use the same owner; a hole, absent index or unknown index does
not prove an own data property. Optional reads, super reads, compound operations,
descriptor fields and constructor observations consume current property facts.

Property reads retain their actual receiver expression even when the callee is
proved native. Explicit `globalThis.name` reads remain object property reads;
they do not use the identifier-read operation that consults global lexical
bindings. Object prototype fallback keeps a possible native target without
treating missing shape evidence as proof of its identity. The related
[source-identity contract](compiler-source-identity.md) records removal of the
source-name and constant-call shortcuts.

Compiler-owned global References use `ExecutionGlobalObject`, which selects the
execution Realm's actual global object. Ordinary source `globalThis` first
resolves local/captured storage or its live global binding; replacement, getters
and deletion invalidate the initial object proof. Callable replacements retain
open candidates for finite-source admission. `typeof` resolves the binding and
observes HasBinding hooks, skipping GetValue only for an unresolvable Reference.
Property deletion retains its receiver/key even beside a same-named global
lexical; identifier deletion retains its selected Global Environment Record and
refreshes that record's lexical delegation before DeleteBinding. The extended
`property_reads_keep_the_receiver_and_actual_global_object` cohort and matching
IR controls cover these distinctions; verification of this repair is pending.

Meaningful IR controls cover getter-driven fact invalidation, description result
domains and inherited BigInt method admission. The paired native controls in
`aot_primitive_property_reads` cover all five primitive receivers, getter and
argument order, exact thrown values, mutable Symbol properties, custom/inherited
BigInt calls, String own versus inherited indices and a replaced String iterator
getter. They also cover newly constructed instances after prototype mutation,
array holes and inherited getters, effectful function receivers, global lexical
shadowing, and boxed String iteration.

`aot_live_prototype_reference_consumers` adds strict/sloppy controls for optional
named/Symbol calls, compound assignments, super reads and `Iterator.from`.
Get effects precede later arguments and right-hand sides, including when a
snapshot has no descriptor. [Iterator.from](https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-iterator.from)
also performs `OrdinaryHasInstance`, so own protocol data properties alone
cannot exclude a Proxy prototype trap. Its alternate callable/Array result
controls require the actual iterator returned by the protocol to remain valid.
The binding/reference repair above still requires the joined compile/regression
checkpoint. Formatting and source review do not establish semantic correctness;
executed regression results and full task acceptance are recorded separately.
