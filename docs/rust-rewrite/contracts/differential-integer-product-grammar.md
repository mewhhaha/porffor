# Bounded integer products with signed zero

`IntegerProductV3`, selected explicitly as `integer-product-v3`, extends the
existing arithmetic campaign with Add, Subtract, Multiply and arithmetic
Negate. It uses the same typed seed, 1–32 check count, depths 1–4, 1–512 replay
budget, schema-v1 self-checking Script protocol, reducer and two oracle gates.
The CLI default remains `IntegerBitwiseV2`. V1 and V2 retain their existing
SplitMix64 draw order, literal ranges, operation choices and emitted source.

V3 literals range from -8 through 8. If an expression at depth d has magnitude
at most B, its arithmetic children can produce at most max(2B, B², B).
Starting at B = 8, products dominate that bound. Thus at depth four every
intermediate result is integral and has magnitude at most 8^16 = 2^48, below
the binary64 safe-integer limit. Generation requires no retry, fallback
operation or sampling failure policy. V3 admits no bitwise operations,
fractions, infinity, NaN or BigInt.

The evaluator's exact result domain has separate `PositiveZero` and
`NegativeZero` variants and a `NonZero` variant carrying a validated safe
nonzero integer. Numeric admission still goes through the safe-integer
constructor; the nonzero payload cannot hold zero or an out-of-range value.
Operators accept this result type rather than unchecked integers or floats.
Rendering exhaustively consumes the result and cannot use the old integer
getter to flatten negative zero. Mathematical integer extraction is confined
to numeric arithmetic and ToInt32/ToUint32, where the zero sign is restored by
the selected operation or intentionally erased by bitwise conversion.

Negation exchanges the two zero variants. Addition produces negative zero
only when both zero-producing addends have negative signs; subtraction
produces negative zero only for negative zero minus positive zero. Exact
nonzero cancellation produces positive zero. A zero product takes the XOR of
the operand signs, including signed zero. Bitwise results retain positive
zero. These rules preserve V1/V2 results while letting V3 emit `Object.is`
checks with an explicit `-0` expected literal. Arithmetic unary operands are
parenthesized, so a negative literal renders as `(-(-5))` instead of a
decrement token; V2's complement rendering is unchanged.

The program constructor checks grammar membership, including V3's literal
bound. The held grammar owns the case id and filename, and materialization
rejects a plan naming another grammar. Reduction retains that grammar,
rebuilds expected signed results from each candidate expression and admits
only nonempty programs with strictly decreasing existing complexity. The
ordinary replay still requires dependency-sealed Scripts, product host
surfaces, rejecting module loading, Wasm-AOT first and an explicitly enabled
spec-exec oracle. Shared failures and observation violations remain red and
cannot be persisted as generated corpus entries.

Authored controls cover all zero-sign combinations, safe-range and overflow
rejection, the depth-four product bound, grammar rejection, unary token
boundaries, signed expected rendering, schema round-trip, reducer preservation
and explicit CLI selection. The finite hand-authored
`t25-integer-product-v3-signed-zero.js` probe is consumed by the existing
feature-gated two-backend replay test. It is a regression source, not a
generated corpus capture.

This 2026-10-03 change is source-only and uncompiled. No generator, oracle,
runtime, compilation or test command ran. V1/V2 byte preservation, V3 runtime
behavior, a deterministic generated capture and broader T25 verification
remain pending. Matching schema-v1 disposition and empty output does not
establish full semantic equivalence or conformance.
