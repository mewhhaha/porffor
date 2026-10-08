# Native Array method ownership in the Wasm GC backend

Status: source authored on 2026-10-05. Compilation, runtime regressions and
conformance verification have not run.

The Array parent and its callback, copy direction, predicate and flattening
children emit whole rooted values and complete completions. The standalone
Array constructor, of, from and isArray entries use the actual callable,
constructor, iterator and object-operation owners. Old manual allocation,
payload/tag pair, sparse bitmap and direct canonical method shortcuts are
retired. Source calls retain observable property Get followed by actual Call.

Generic methods perform ToObject and one observable length/ToLength snapshot.
Later indexed operations retain their required live HasProperty/Get behavior,
original receiver and evaluation order. Callback methods preserve whole object,
String, Symbol and BigInt identity. Map preserves holes; filter retains selected
original values until output publication. Find visits holes, while quantifiers
and reduce skip absent generic properties. Flat owns strong GC traversal frames,
advances its parent before observable descent, and applies flatMap's mapper only
to outer source elements.

Sort validates its comparator before length observation. Generic sort collects
present values, uses stable comparisons, writes the receiver and deletes trailing
properties. ToSorted creates a new Array and reads through holes. Undefined
values sort after other values without invoking the comparator. Default
comparisons perform ordered ToString and full UTF-16 comparison. TypedArray's
six numeric/default sort and by-copy methods remain in the separately owned
standard/typed_array_methods.rs child.

Concat observes spreadability before a spread operand's length and live
HasProperty/Get traversal, checks the safe-index bound before publication, and
performs the final length Set. Splice preserves holes in the deleted result and
moves receiver properties in the direction required by overlap. ToSpliced,
ToReversed and With publish dense new Arrays and preserve their distinct Get
order; With does not Get its replacement slot. CopyWithin uses strict Set/Delete
and the shared closed direction projection.

Join converts its separator before element traversal; locale formatting retains
the actual element receiver, locale/options arguments and Get/Call/ToString
sequence. Array and TypedArray toString use the same canonical callable. That
generic entry performs ToObject, observable Get(join), a callable test and
actual Call, with Object.prototype.toString as its fallback.

TypedArray methods retain the sole BufferOwner and element-access authorities.
Fill rejects an immutable target before observable conversions, even when empty.
Set validates the target and offset/source protocol in order. Same-data-block
overlap captures an exact private byte snapshot before writes. Different element
kinds decode that snapshot through the sole element-word conversion owner;
same-kind transfers retain ascending byte-copy order. Shared backing identity
comes from the actual native resource, not wrapper reference identity.
Map/filter/slice species results receive write admission, including empty output.

The eight finite Engine cohorts in aot_gc_native_array_entries.rs cover live
callback operations and identity, stable sort and abrupt completion, concat
ordering, sparse splice and by-copy behavior, join/locale/shared toString,
standalone entries and iterator closure, TypedArray overlap/conversion/immutable
admission, and flattening/search behavior. Each cohort includes strict and
nonstrict source. All controls remain unrun. Verification is deferred until the
complete atomic GC source batch is authored, under the confirmed aggregate
4096 MiB limit with swap disabled and one worker.
