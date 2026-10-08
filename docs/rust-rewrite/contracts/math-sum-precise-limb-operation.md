# Math.sumPrecise exact limb operations

The accumulator exclusively owns a private non-null `MathSumPreciseLimbArray`.
Its factory allocates exactly 34 zeroed 64-bit limbs. This compiler-private
array has no JavaScript tag or public object identity. No mutable capability is
added to published BigInt limbs.

Closed Add/Subtract operations fold finite binary64 significands into a signed
exact superaccumulator. Scalar operands are `I64Local`; only proven bounded
limb indices narrow to the GC array's i32 indexing API. Two's-complement
magnitude extraction and guard/sticky/ties-to-even rounding retain the single
exact final binary64 rounding step. The fixed capacity covers at most
2^53-1 finite inputs, including subnormals and maximum finite values.

The old byte heap and its mirror guard are retired. This draft is source only:
no type, numeric oracle, runtime or conformance proof has run. Existing CLI and
new Engine cancellation/tie/subnormal/overflow controls are retained for the
later checkpoint under the confirmed 4096 MiB aggregate memory limit.
