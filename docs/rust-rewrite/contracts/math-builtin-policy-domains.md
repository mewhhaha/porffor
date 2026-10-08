# Native Math policy domains

The T05 atomic source draft uses the consumed closed `MathBuiltin` and
`MathUnaryBuiltin` enums and exhaustive native dispatcher. Every numeric
argument passes through one whole `ToNumber` completion before its scalar bits
are read. A thrown reference is retained intact and prevents later argument
coercion. One native cleanup edge publishes the complete result.

Scalar kernels take `I64Local` operands. Host math imports keep their existing
binary64 ABI. Shared binary16 kernels live in `binary_data/float16.rs`; Math and
typed element writers call the same typed producer. No semantic value is an
integer address, and no payload/tag return pair remains in Math.

This is uncompiled source. Existing CLI semantic controls remain, with new
Engine GC controls for coercion order, signed zero, binary16 rounding and exact
summation. Spelling-only predecessor guards are retired with the raw ABI. Their
historical results do not establish this draft's type or runtime correctness.
The later checkpoint must use the confirmed 4096 MiB aggregate kernel cap.
