# Boolean primitive and box ownership

The native constructor applies non-coercing truthiness to its argument. A call
returns the Boolean primitive; construction resolves NewTarget's prototype and
publishes a complete PrimitiveBox with that header and immutable stored Boolean.
Prototype lookup errors retain their complete original Throw result.

The private ToString/ValueOf operation is exhaustively consumed by the actual
shared prototype body. The body accepts only a Boolean primitive or a box whose
stored primitive is Boolean. A Number, BigInt, String or Symbol box does not
acquire Boolean instance data merely by sharing the box representation.
ValueOf retains the Boolean value; ToString publishes the canonical GC string
for true or false. Neither result is a semantic integer address.

The existing boxed-builtin CLI fixture remains a semantic control. The Engine
GC box control covers wrong-brand rejection, custom NewTarget prototypes,
prototype identity, BigInt truthiness and absence of coercion hooks. The old
source-spelling mirror is retired with the byte-heap emitter. Historical
2026-08-27 verification belongs to that predecessor; this GC source draft has
not been compiled or executed. Full atomic integration and capped verification
remain required.
