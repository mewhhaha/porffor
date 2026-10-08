# BigInt fixed-width operations

The actual native entry chooses the closed Signed or Unsigned operation. It
completes ToIndex(bits), then ToBigInt(value), including when bits is zero.
An abrupt coercion keeps its original whole completion and suppresses later
work. See [BigInt.asIntN](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-bigint.asintn).

A positive magnitude already fitting an unsigned width is retained. Either
sign already fitting a strictly wider signed width is retained. These paths
avoid allocating storage proportional to a large requested width.
Truncation uses a private GC limb construction, forms the two's-complement
residue, masks the final partial limb and projects the requested signedness.
A negative signed residue is converted back to sign/magnitude before canonical
publication. Published input limbs are never writable through this owner.
Array counts are checked before narrowing to Wasm i32 indices.

The CLI arbitrary-width fixture remains a semantic control. New GC numeric
controls cover coercion order, zero width, widths above 64 bits and a huge
width with a small retained result. All are pending capped verification.
