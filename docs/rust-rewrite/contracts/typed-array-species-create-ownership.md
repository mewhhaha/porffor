# TypedArraySpeciesCreate and validated result ownership

Status: 2026-10-03 dry source implementation. Rust compilation, emitted Wasm
validation and runtime controls remain pending. Published counts are unchanged.

The complete current species consumer batch is map, filter, slice and subarray.
A closed length-source acquisition accepts only map/filter/slice and captures
one validated method-entry view. A distinct subarray source type captures its
non-throwing length snapshot; it cannot enter the validated length-source route.
Both source owners retain immutable view/kind fields through construction and
release their scratch only after result publication. Their entry policy and
source views remain separate from result construction. A private factory owns
constructor/@@species selection,
Proxy-aware Construct, genuine TypedArray result brand and a fresh seq-cst result
view. Number/BigInt content must agree with the exemplar. A single numeric length
argument also requires a result at least that long; subarray's buffer argument
vector has no such minimum-length requirement.

The result is private, non-Copy and cannot be minted by a caller. Actual target
access and final publication accept that owner. Slice's target copy-state access
and map/filter writes therefore follow real validation rather than a comment or
an unchecked local. The factory's checked view is an observation, not permission
to reuse stale backing data after later callback/coercion effects. Existing live
integer-indexed writes and source freshness rules still apply.

Default constructors come from the executing builtin's defining Realm. The
Realm intrinsic record appends twelve concrete constructor identity slots without
moving existing fields. Both entry and created installers store actual functions
after their constructor/prototype setup. A private one-shot current-function
constructor capture selects an immutable slot from the closed element-kind
domain. The zero-environment entry convention uses the canonical callable
Function prototype only to locate its Realm. Missing internal state traps;
mutable globals and public `prototype.constructor` properties are not defaults.
Every species method roots all twelve constructor installers. The same
original-constructor capture is now consumed by toReversed/toSorted/with; their
planner roots also close all twelve installers, without observing species.
See [same-type ownership](typed-array-create-same-type-ownership.md). This is required
when an entry method receives another Realm's kind that its own source never
names. Existing once-only dependency walking closes and terminates this cycle.

Constructor observation remains an ordinary Get on the exemplar. Undefined
constructor selects the default; a non-object constructor rejects. Nullish
@@species selects the default, while a non-constructor species rejects. Selected
user constructors retain their ordinary Construct argument vector and newTarget.
The shared Proxy-aware path preserves traps, revocation, thrown arbitrary values
and non-object trap-result errors. A returned Proxy is not a genuine TypedArray.
Selected constructor behavior and method-owned validation errors retain their
existing distinct Realm authorities.

Map creates its result using the captured entry length before invoking any
callback. Filter finishes its captured-length callback walk and selected list
before creating the result. Each callback still performs a live source indexed
read. Target writes retain their original ordering and element conversions.

Slice coerces bounds against entry length, constructs for the original count,
then freshly validates the source only when that count is positive. Shrinkage
may reduce the copied prefix without changing the constructor argument. A zero
count still constructs and validates the result but performs no later source
check. Same-kind overlapping byte copies remain ascending and preserve stored
bits; different compatible kinds use live indexed reads and converted writes.

Subarray requires TypedArray slots but retains its non-throwing initial length
snapshot: detached/out-of-bounds source contributes zero before bound coercions.
A tracking source with omitted or undefined end receives exactly two arguments,
buffer and byte offset. Fixed sources or a supplied non-undefined end receive
three, including the captured normalized length. Both argv header and callee
count describe the same complete vector. The selected constructor may ignore
the source buffer and return another compatible valid TypedArray.

TypedArray.from and TypedArray.of keep their separate validated-construction
policy. This batch does not create a second object model, GC identity bridge,
backend representation or unsupported-runtime fallback.

Paired Engine fixtures run through actual Wasm-AOT/Test262-host sources when
verification resumes. They observe constructor Proxies/newTarget/argument arity,
callback/species and coercion order, thrown foreign markers, wrong result brands,
content mismatches, short/detached/out-of-bounds targets, overlap/zero-count copy,
tracking/explicit end, global clobber and borrowed entry/created-Realm defaults.
A sparse-source control uses only an entry Uint8Array reference while receiving
a created-Realm Float16Array. It observes borrowed default identity and values;
active Realm creation already requests full entry globals, so that fixture alone
cannot prove the narrow installer dependency closure. The actual planner owns
that dependency. Existing structure guards are narrowly
maintained around the actual shared operation, without new mirrored groups.

These are authored controls and source invariants. Full TypedArray/T17, semantic
GC, real-agent behavior and all-suite acceptance remain open. Historical witness
checkpoints retain their original evidence and do not certify this new source.
