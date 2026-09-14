# Function values in generic string and number conversion

Generic `ToString` and `ToNumber` treat Function values as objects. They execute
`@@toPrimitive`, or the ordinary `toString`/`valueOf` fallback in the order selected
by the hint. Only `Function.prototype.toString` reads a function's stored source.

The string conversion and all three value-to-number variants use the existing
`emit_tagged_to_primitive_locals_pending` producer. Its private pending token must
be consumed by the corresponding primitive conversion or abrupt continuation.
The Number constructor still accepts a BigInt primitive result; ordinary ToNumber
still rejects it. ArraySetLength still converts the original value twice, and
Iterator `take`/`drop` still own IteratorClose after a limit-conversion throw.

The `CurrentExecutionContext` conversion-error policy serializes to the existing
current-function Realm ABI word. ToString and ToPrimitive helper parameter 6 is
an existing trusted Realm projection, never a raw lexical environment. The
ToString, ToNumber, ToNumeric, and three ToPrimitive bodies declare this trusted
context for hook property reads and calls. Generated conversion errors use the
same context; values thrown by getters and coercion hooks retain their identity.
The existing fixed-current-function phase token remains non-copyable and couples
its two fixed Realm selections.

This change removes the Function source-text and NaN bypasses. It preserves the
existing Array and Arguments branches of the tagged ToPrimitive implementation;
it does not establish their complete coercion conformance. No helper IDs,
operation-catalog rows, source omissions, or Test262 fingerprints change.

The `aot_function_coercion` native target has ten tests, including both execution
modes of the unchanged pinned slice fixture, receiver/index coercion ordering,
BigInt policy, ArraySetLength rechecks, IteratorClose, abrupt identity, generated
error realms, array-element conversion, and direct intrinsic source behavior. The backend's exact Realm
source census and helper membership assertions are updated with the implementation.
Native execution and workspace validation are required before reporting these
regressions as repaired.
