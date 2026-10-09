# parseInt radix conversion

The emitted global parseInt and Number.parseInt share the canonical callable.
It performs ToString(input), then ToNumber(radix) once, propagating either
conversion's actual thrown value before digit scanning. An undefined radix
retains the existing direct zero path.

ToInt32's Number residue uses the backend's sole
`emit_to_uint32_i64_from_number_payload` authority. It truncates in binary64,
subtracts the floor-quotient multiple of 2^32 in binary64, and only then
narrows the bounded residue. The radix consumer interprets those low bits
as signed with i32.wrap_i64 followed by i64.extend_i32_s. It never saturates
the original Number to i64: doing so maps finite 1e308 to an invalid radix
instead of zero. NaN, infinities and signed zero reduce to zero through the
same authority; powers of two preserve exact division and multiplication.

The existing radix 2–36 validation, hexadecimal-prefix selection, whitespace,
sign, digit accumulation and final negative-zero result remain in their
existing order. This batch changes no parser, runtime ABI or dynamic-source
policy and introduces no second numeric conversion implementation.

The separate source packet contains six Engine tests for twelve fresh
sloppy/strict observations: four native controls and the complete pinned
`staging/sm/Number/parseInt-01.js` and `staging/sm/global/parseInt-01.js`, each
with the full sta.js and assert.js. Controls exercise extreme finite and
nonfinite radices, fractional modular residues, signed range rejection,
single ordered conversions, mutation, thrown object/undefined identity and
Symbol/BigInt TypeErrors. These observations are UNRUN until Root integrates
and binds an actual Source. Historical publication records four failures;
no replacement aggregate or pass claim is made here.

```sh
cargo test -p lila-engine --test aot_builtins -- aot_parse_int_radix:: --test-threads=1
./target/debug/lila test262 run staging/sm/Number/parseInt-01.js --execution-backend wasm-aot
./target/debug/lila test262 run staging/sm/global/parseInt-01.js --execution-backend wasm-aot
```
