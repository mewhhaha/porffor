# TypedArrayCreateSameType ownership

Current source status, 2026-10-05: the atomic Wasm-GC rewrite is authored only. Compilation, emitted Wasm, focused controls, real agents and full pinned conformance remain unverified. No status counts changed.

The actual factory remains builtins/standard/typed_array_create_same_type.rs. It consumes the saved current-function Realm intrinsic constructor, builds one complete GC ValueArray containing Number length, performs real Construct and uses the shared concrete write-view/minimum-length validation. Result publication is a whole CompletionLocals, and the caller retains its rooted result until cleanup. Constructor/species properties on the source remain unobserved. The GC native method callers are documented separately in gc-typed-array-native-methods.md.

Four paired strict/sloppy finite Engine cohorts in `aot_gc_binary_data_entries.rs` cover native buffers, DataView, TypedArray construction/statics/species and Atomics/Realm lifecycle. Existing CLI semantic fixtures remain; obsolete raw-spelling guards are retired rather than replaced with mirrors. The historical implementation and receipts below do not certify this batch.

## Historical record before the atomic GC rewrite


Status: 2026-10-04 dry source. Rust compilation, emitted Wasm validation and
runtime controls remain pending. Published conformance counts are unchanged.

The actual toReversed, toSorted and with callers share one private factory in
`builtins/standard/typed_array_create_same_type.rs`. Its non-Copy completed
result type has private fields and no raw target projection. Callers cannot
mint an unchecked result: indexed writes, stable sorting and final consuming
publication accept the owner created by the factory.

The factory borrows the caller's validated element-kind and captured-length
locals. It reserves its result pair, consumes the existing current-function
Realm constructor capture, passes one Number length argument to real Construct
with the original constructor as newTarget, propagates abrupt completion, and
uses the shared constructed-target validator before returning the owner.
The immutable original concrete intrinsic supplies same element kind and
Number/BigInt content. Source constructor and @@species properties are never
read. The existing twelve-row closed element-kind selection and original
per-Realm identity slots are used directly. Each of the three method roots
closes all twelve constructor dependencies through the existing once-only
planner walk; it does not depend on which public constructors the source names.

Factory scratch is released in reverse order while the result pair remains
owned. Caller loop and value locals are reserved before acquisition, and the
borrowed kind/length remain live until publication. Publication consumes the
result, copies the result pair, sets Normal and retires the pair in reverse
order before caller scratch is released. This ordering prevents result locals
from being reused while indexed writes or sorting still refer to them.

toReversed validates entry, captures length, creates, then performs live reads
in reverse order and target writes. toSorted checks comparator admissibility
before receiver validation, captures length, creates and completes all source
reads into its private result before invoking the shared stable sorter. The
fresh result has no user-visible reference during that copy. with validates
entry and retains initial length; it coerces index, computes the relative index
against that length, coerces replacement, and checks the current integer index
before creating. Its remaining source reads stay live. It does not add a late
ValidateTypedArray that would replace the required current-index RangeError.

The shared sorter collects the complete captured target range before comparing.
Proxy-aware comparator Call is followed by ToNumber; either abrupt completion
propagates before writeback. A normal comparison must not create an extra
detached-buffer abort. Subsequent comparisons continue on the collected list,
and every final target write observes fresh validity after callback effects.
The existing stable Number/NaN/signed-zero and signed/unsigned BigInt ordering
is preserved. Comparator counts remain algorithm-dependent; the finite fixture
that throws on its second coercion asserts only that first normal detachment
does not suppress that required abrupt path.

Array.prototype.toSorted retains its separate closed Copy policy and now uses
the defining-function-Realm Array allocator. Its ascending Get collection reads
through holes and creates own undefined entries, without HasProperty or source
deletion. A separate dry source repair now preserves Array.sort Receiver's
real HasProperty result for TypedArrays; its finite BigInt control and runtime
acceptance remain unverified. That repair is outside this SameType batch.

Three paired strict/sloppy Engine sources use WasmAot with the Test262 host
surface and assert one scenario print plus Normal(262). They cover twelve
element kinds × three methods × both borrowing directions (72 results),
original result/backing-buffer prototypes after public-global and constructor
property poisoning, With order and live views, Number/BigInt earlier abrupt
completion, stable sorting, Proxy comparator/result coercion, foreign thrown
identity and finally/prior-assignment behavior, detachment, fresh resize
writeback, private copies and borrowed Array hole/Get ordering. Existing guards
are changed only for their affected source consumers; the unchanged With
entry/coercion/current-index guard retains its original bytes and policy.

These are source invariants and authored controls. No old test receipt proves
this batch. Full T16/T17, semantic GC, real agents and all-suite acceptance remain open.
The normative algorithms are
[TypedArrayCreateSameType](https://tc39.es/ecma262/2026/multipage/indexed-collections.html#sec-typedarraycreatesametype),
[with](https://tc39.es/ecma262/2026/multipage/indexed-collections.html#sec-%typedarray%.prototype.with),
[CompareTypedArrayElements](https://tc39.es/ecma262/2026/multipage/indexed-collections.html#sec-comparetypedarrayelements)
and [SortIndexedProperties](https://tc39.es/ecma262/2026/multipage/indexed-collections.html#sec-sortindexedproperties).
