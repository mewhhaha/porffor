# AggregateError iterator-list ownership

The complete 2026-10-04 source batch implements the current
[AggregateError constructor](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-aggregate-error-constructor)
through the existing preparation, iterable-list and finalization owners.
NewTarget prototype acquisition precedes message conversion, cause installation
and iterator acquisition. The prepared non-Copy Error instance remains unpublished
until list completion and the non-enumerable, writable, configurable errors
property is defined. Promise.any retains its separate existing owner.

Every input now enters the shared GetIterator path. Arrays and Arguments observe
the actual Symbol.iterator key instead of taking a snapshot shortcut. Primitive
sources are boxed in the called function Realm for lookup while the original
value remains the Get and Call receiver. Shared IsCallable and Proxy-aware Call
accept callable Proxy iterator methods and next methods, cache next once, and
propagate original getter, trap and call completions.

The private, non-Copy SyncIteratorLocals record is reserved by the shared owner,
borrowed for acquisition and IteratorStepValue, and consumed on release. The
sixth SyncIteratorConsumer::AggregateError authority selects wording only.
Four exhaustive mappings select existing catalog messages; the separate
body-Realm projection chooses the called builtin's intrinsic TypeError.
No lexical environment, source object, method or NewTarget selects that error Realm.

[IteratorToList](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-iteratortolist)
repeatedly steps, reads done before value and omits terminal value. Acquisition,
next, done and value abrupts propagate directly without a return Get or
IteratorClose. The unpublished Array storage uses the shared current-function
Realm allocator. Its eventual errors Array therefore belongs to the called
constructor Realm even when a different-Realm NewTarget supplies the Error
prototype. Public Array/AggregateError replacement and source constructors do
not select that intrinsic allocation.

Two strict/sloppy WasmAot Engine cohorts cover 20 normal call/Construct cases,
two cross-NewTarget cases, actual Array/Arguments iterator overrides, String
code points, callable Proxy receiver/zero-argument checks, cached next mutation,
done-before-value and errors attributes. Prefix controls retain exact
prototype/message/cause/iterator order and three abrupt cutoffs. Six original
foreign-marker protocol stages retain identity, prior assignment and finally;
finite native failures cover all four diagnostics and called-Realm ownership.
Every close getter remains unobserved.

Existing consumer/error/Math architecture guards retain the closed domain,
24 exhaustive rows, 17 producers, 43 error identifiers and the actual borrowed
AggregateError acquisition/step/release join. The module inventory records one
new current-function Realm Array allocator consumer. Code, controls, types and
documentation are authored. The ref97 combined all-target Rust type check passed;
emitted-Wasm validation and runtime execution remain pending. This batch makes no
full iterator, T24, pinned-conformance or published-count claim.
