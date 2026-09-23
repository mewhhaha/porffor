# Exact NumberToBigInt conversion

The BigInt function accepts every finite integral Number. NumberToBigInt
rejects NaN, infinities and fractional values with RangeError before BigInt allocation,
then reuses the exact binary64 decoder already used for BigInt–Number
comparisons. The decoder separates sign and significand, shifts into base-2^32
digits, and normalizes zero. Both signs of Number zero produce `0n`.

The shared BigInt result packer selects the existing signed inline value or
arbitrary-precision heap record. Its destination locals are explicit so a
conversion cannot overwrite a caller's unrelated completion payload. Number
magnitudes outside signed 64-bit range never pass through Wasm's trapping
float-to-i64 conversion. A Number's existing binary64 value is converted
exactly; the decimal source spelling is not reinterpreted.

Object inputs still undergo one number-hinted ToPrimitive before the closed
Number policy is selected. The abstract ToBigInt operation continues to reject
Number inputs with TypeError; only the BigInt function admits NumberToBigInt.
The created-realm BigInt function uses the existing self-backed builtin ENV
so NumberToBigInt RangeErrors select that function's defining Realm. Abrupt
ToPrimitive results propagate unchanged; other conversion-policy errors are
outside this repair.

Current-main terminal results for both execution modes of
`staging/sm/BigInt/Number-conversion-rounding.js` trap with integer overflow in
`builtin::BigInt`. Frozen checkpoint6 probes independently confirm traps for
dynamic `BigInt(2**63)`, `BigInt(2**64)` and `BigInt(Number.MAX_VALUE)`; the
negative signed boundary remains a passing control. Frozen checkpoint6 also
confirms that a foreign BigInt function incorrectly creates main-Realm
RangeErrors for fractional, NaN and infinite arguments. The Number validation
path now uses the existing current-function Realm error helper, and the
created-realm constructor retains the function identity that helper requires.

Verification targets are `aot_number_to_bigint`, the existing
`aot_bigint_number_conversion` arithmetic/rounding controls, and
`bigint_number_policy_structure`. Native validation and replay of both pinned
failures remain necessary before reporting the family fixed.
