# Float16Array storage and realm ownership

`Float16Array` uses the existing TypedArray brand, buffer/view slots, and live
view witness. It adds no object representation or runtime evaluator. Its
constructor belongs to the standard builtin catalog, has two-byte elements,
and inherits the same `%TypedArray%` constructor/prototype graph as other
numeric views in each realm.

`TypedArrayElementKind` owns the twelve storage words, byte widths, Number or
BigInt content types, and Atomics admissibility. Indexed loads and stores match
this closed domain exhaustively. An invalid internal kind cannot silently fall
through to signed integer storage. Typed-array source construction, generic set and species methods compare
explicit content types rather than assuming an ordering among the ABI words.
The appended Float16 word is Number content even though its ordinal follows the
BigInt kinds. Atomics rejects floating and clamped kinds before argument
coercion through the existing validated receiver path.

Reads load a 16-bit word from the selected backing memory and reuse the DataView
binary16 decoder. Writes reuse DataView's direct binary64-to-binary16 conversion,
including ties to even, subnormals, signed zero, infinities and NaN. A binary32
intermediate is forbidden on writes because values adjacent to binary16
midpoints can double-round. Conversion preserves its input payload, including
across repeated `fill` stores.

Realm prototype pointers are appended to both authoritative heap schemas:
function-object offset312 (size320) and realm-intrinsics offset472 (size480).
Bootstrap, function creation, foreign realm construction, default-prototype
selection and global root registration include the new constructor/prototype.
The ordinary TypedArray view lifecycle still governs resize, detachment,
shared buffers, iteration, copy operations and species results.

Verification targets are `lila-ir --test float16_array`, the backend element-kind
and heap layout units, `lila-engine --test aot_float16_array`, and the existing
`aot_typed_array_fill` target. Native regressions exercise every binary16 bit
encoding, direct rounding at adjacent midpoints and overflow, shared/resizable
views, constructor forms, inherited methods, species, realm ownership and
Atomics coercion order. These are focused checks, not a complete Test262 claim.

The pinned `testTypedArray.js` harness conditionally excludes Float16Array when
absent. Adding the constructor exposes a new element kind in previously green
harness executions; those executions need a fresh run before their prior status
can count as Float16 coverage.
