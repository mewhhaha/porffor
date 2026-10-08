# BigInt prototype result ownership

The actual native dispatcher exhaustively selects ExactValue, RadixString or
LocaleString. It validates a BigInt primitive or a PrimitiveBox containing a
BigInt before evaluating radix, locales or options hooks. The ordinary BigInt
prototype object has no BigInt instance data and fails that validation.

ValueOf retains the whole canonical GC primitive. RadixString coerces and
checks its radix once, then consumes the existing non-coercing GC formatter.
LocaleString retains the primitive across the intrinsic NumberFormat call;
it never rounds the BigInt through Number or consults mutable public Intl
properties. The NativeIntl producer must consume this whole-value seam in
the same atomic GC batch.

Output is a complete Normal or Throw result. Raw payload/tag pairs, separate
inline/heap result routes and policy tokens without a consumed invariant are
retired. The compiler-enforced owners and semantic CLI/Engine controls replace
obsolete source-spelling mirrors. Compilation and runtime checks are pending.
