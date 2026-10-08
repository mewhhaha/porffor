# Math.min and Math.max argument reduction

A closed `MathExtremum` enum owns the empty identity and binary64 combination.
The native loop reads whole arguments from the private GC vector and consumes
each `ToNumber` completion before folding its typed scalar bits. NaN does not
skip later coercions. A later thrown object retains its identity and exits
through the native completion cleanup.

The Wasm min/max scalar operations retain the specified signed-zero choice.
No array backing address, payload/tag argument pair or raw result slot remains
in this native reduction. The old spelling guard is retired; existing CLI
semantic controls and new Engine order/zero controls remain.

This atomic T05 source has not been compiled or executed. Validation belongs to
the later capped whole-batch checkpoint.
