# Array `@@isConcatSpreadable` as an exact tagged slot

## Semantic boundary

An Array's own `Symbol.isConcatSpreadable` property is an ordinary JavaScript
property value. `[[Get]]` must return that value unchanged. The later
`IsConcatSpreadable` abstract operation decides whether to apply `ToBoolean`,
and treats only `undefined` as the signal to fall back to `IsArray`.

The old Array-specific slot crossed that boundary early. A data-property write
stored only the value's truthiness, using `u64::MAX` as an `undefined`
sentinel, and a read reconstructed either Boolean or Undefined. Concat itself
usually remained correct because it needs only the eventual truthiness, while
ordinary reads silently lost object and Symbol identity, string and numeric
types, signed zero, and NaN.

## Closed stored-value shape

The occupied special slot has exactly two shapes:

```text
ArrayConcatSpreadableSlotValue
  Data(TaggedLocals)
  Getter(TaggedLocals)
```

Both variants carry one complete tagged JavaScript value. Exhaustive
projections select the tagged locals and the corresponding data/accessor
descriptor shape. The sole occupied-slot writer accepts this enum, so a
producer cannot write a payload without its tag or select a getter descriptor
for data. Adding another stored shape fails compilation until both projections
are defined.

Descriptor word zero remains the absent state. It is not an enum variant
because there is no occupied value to carry; a read returns Undefined without
consulting stale slot contents.

## Physical storage

Data and accessor descriptors are mutually exclusive, so they share the
existing tag/payload pair historically named for the getter. The descriptor
shape decides whether the pair is the exact data value or the getter value.
That payload word is already declared as a strong heap edge, unlike the old
truthiness word, so object, string, BigInt, Symbol and callable values remain
reachable without growing the Array record.

The old pointer-free truthiness word is no longer read or written by property
semantics. Its allocator initialization and physical record slot remain until
the shared Array allocator can be compacted without colliding with active
compiler work.

## Read and use order

An Array `[[Get]]` of `@@isConcatSpreadable` now follows one closed dispatch:

1. an absent descriptor returns Undefined;
2. a data descriptor copies the stored tagged value exactly;
3. an accessor descriptor calls its callable getter with the Array receiver,
   or returns Undefined for an absent getter.

Callable Proxy getters use the same callability gate and call path as other
property accessors. The reader does not coerce the result. Array concat obtains
that exact result and performs `ToBoolean` only inside `IsConcatSpreadable`.

## Durable evidence

The focused concat-spreadable fixture retains its existing concat cases and
also reads Array-owned data values back directly. It covers object and Symbol
identity, string and numeric tags, signed zero, NaN, Undefined fallback, and
the distinction between truthy spreading and falsey non-Undefined suppression.
An Array-owned accessor also returns its exact object result with the Array as
receiver. A callable Proxy getter proves the same receiver, exact-result and
concat path through the callability abstraction, while a second Array-owned
getter throws a unique object sentinel that concat must propagate unchanged
through the nested accessor and `IsConcatSpreadable` abrupt routes.

The Rust projection test fixes the two occupied shapes, their complete tagged
carriers and their data/accessor descriptor roles. Static source checks keep
the old truthiness word out of behavioral reads and writes.

## Nonclaims

This seam does not complete the special property's descriptor-attribute,
deletion, redefinition, inherited-setter or Proxy-trap behavior. It does not
change generic object or Arguments-object storage, compact the Array record,
remove Array Test262 materializers, or establish full Array conformance.
