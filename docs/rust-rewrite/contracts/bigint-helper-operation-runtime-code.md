# Typed BigInt helper operation domain

BigIntHelperOp owns the fourteen arithmetic/comparison/bitwise/shift operations
and their stable internal codes. The product dispatcher matches that enum
exhaustively; an added operation requires a compiler-visible implementation.
Typed helper argument and parameter owners derive from the actual registered
BigIntArithmetic row, with two whole values and an i32 selector. No generic
JavaScript-call signature or raw semantic address is accepted.

The helper decodes published immutable magnitudes into private GC construction
digits. Exact add/subtract, multiplication, division/remainder, infinite
two's-complement operations and shifts share those owners. Exponentiation
scans arbitrary-width exponent bits, retaining exact zero/one/minus-one cases
without an artificial u64 exponent rejection. Negative exponents and division
by zero publish whole RangeError results. Resource exits use the helper's real
control target before any index narrowing.

Publication returns a canonical GC BigInt or Number comparison in a complete
five-result completion. Existing semantic arithmetic controls remain; raw-ABI
source-spelling guards are retired with their representation. This draft has
not been compiled or executed.
