# Exact NumberToBigInt conversion

The conversion validates finite integral binary64 inputs before decoding sign,
significand and exponent. Both Number zeros publish the same canonical BigInt
zero. Large magnitudes never pass through a trapping float-to-i64 conversion.
The decoder fills private base-2^32 GC construction digits; publication joins
pairs into canonical unsigned u64 limbs with a Boolean negative flag.

`emit_number_to_bigint_locals` consumes typed Number bits and a complete result
owner. Success and RangeError retain tag, scalar, reference, kind and target.
There is one published GC BigInt representation and no inline/heap choice.
`ToBigInt` still rejects Number; the closed `NumberToBigInt` policy is admitted
only at the BigInt function boundary after one number-hinted ToPrimitive.

See [NumberToBigInt](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-numbertobigint).
The experimental Wasmtime GC backend is required. New GC numeric controls and
existing Number-to-BigInt controls remain unrun in the source-only batch; older
byte-heap failure receipts are historical evidence, not proof of this draft.
