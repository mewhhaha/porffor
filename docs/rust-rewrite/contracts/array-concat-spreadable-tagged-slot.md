# Array `@@isConcatSpreadable` ordinary-property ownership

## Semantic boundary

An Array's `Symbol.isConcatSpreadable` property is an ordinary JavaScript
property. Get, Set and DefineProperty must therefore share the same descriptor
authority as every other Array Symbol key. Concat obtains the exact property
value through ordinary Get and applies `ToBoolean` only afterward; only
Undefined falls back to `IsArray`.

## Single owner

The ordinary Array named-property owner is the only Array storage for
`@@isConcatSpreadable`. It owns complete data descriptors and complete accessor
descriptors, including getter, setter, writable, enumerable and configurable
state. Ordinary Set consequently decides getter-only rejection, setter calls,
non-writable failure, extensibility, inherited setters and Proxy order without a
second Array-specific interpretation.

The former dedicated representation could encode a getter but not its setter.
Its receiver-side write branch also treated every occupied descriptor as
writable data. Those states are no longer representable: the capability enum,
read/write emitters, four heap fields, four layout rows and both initializer
sites are deleted. Recursive source evidence pins zero occurrences of every
removed owner name.

The removed offsets remain padding. `HEAP_ARRAY_RECORD_SIZE` and all unrelated
Array offsets are unchanged; this closure does not move dense elements or named
property storage.

## Routing

Static and computed Array assignment both call the ordinary object Set owner.
Static Array reads and concat call ordinary object Get. Object.defineProperty
routes both accessor and data descriptors through the existing Array named
descriptor compilers. Arguments objects retain their distinct exotic
`@@isConcatSpreadable` implementation.

This routing leaves one representation and one observable order. Adding a new
dedicated producer or consumer makes the recursive owner test fail, while new
ordinary descriptor fields are inherited automatically by this Symbol key.

## Durable evidence

The descriptor-assignment fixture covers three observable descriptor shapes:

- a getter-only accessor ignores sloppy assignment and throws in strict code;
- a getter/setter accessor calls its setter with the Array receiver and exact
  assigned value;
- a non-writable data property ignores sloppy assignment and throws in strict
  code.

Each false-valued property also suppresses spreading and leaves the Array itself
as the concat result element. The existing aggregate, direct accessor, receiver
and order/error fixtures retain exact tagged values, accessor receivers, Proxy
calls, prototype traversal and abrupt completion coverage.

The recursive structure target passes `5/5`. The focused
`array_concat_spreadable` CLI filter passes `5/5`: aggregate core, direct
accessor, descriptor assignment, receiver and order/error. The descriptor case
also passes alone as an exact `1/1` witness. The neighboring Array-at receiver
policy target passes `3/3`.

The pinned `is-concat-spreadable-val-truthy.js`,
`is-concat-spreadable-get-order.js` and `is-concat-spreadable-get-err.js`
controls pass all `6/6`
sloppy/strict Wasm-AOT executions with every failure bucket at zero. The shared
`cargo xc` checkpoint is green.

The frozen raw-body SHA-256 values are:

- Array named data descriptor owner: `b970db24ecd2f945b25e564610b598b5c4163a4661bb61a507f38a81cb760bde`;
- Array named accessor descriptor owner: `88549739cc949da6a6e5834ef75e52593b5cdd85ba87899f416ddca4bb3771de`;
- canonical concat compiler: `e3bcf4992367960b8a205469f5ec94e1d56ade0a4039ff7f64ebf2995a7fd3e4`;
- Array own named-property reader: `febe236df75d13bf589053d980867bb63f44e595c9e1a4a1613bb111b164098f`;
- OrdinarySet receiver-fallback owner: `d58fee7ab153c8d398a112fb38ac086c3661c3f198b3503add6ff960d70f454c`.

Recursive product-source census finds zero removed owner names. Object
definition has eight calls into the two ordinary Array named-descriptor
compilers, in addition to the data-property compiler's internal data-descriptor
call. The semantic golden remains deferred.

## Nonclaims

This seam does not shrink the Array record, move later fields into the padding,
change Arguments-object storage, or establish complete Array conformance.
