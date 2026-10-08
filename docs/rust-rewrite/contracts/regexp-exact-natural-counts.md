# Exact natural RegExp repetition bounds

Static and emitted RegExp construction share one version-three immutable program
descriptor. A written finite decimal bound remains finite, regardless of its
value. `RegExpNatural` owns validated canonical ASCII decimal digits; leading
zeros are removed without rounding or saturation. `RegExpRepeatMaximum` separates
finite values from an unbounded maximum. The checked `RegExpRepeatBounds`
constructor accepts only ordered finite pairs. The original source decimal spans
remain the parser's sole authority for a reversed-bound SyntaxError.

The static source owner retains these exact bounds through parsing and lowering.
Only zero/one trivial forms use the existing split/progress paths. Every other
admitted count publishes one Begin/Guard/body/End/Exit region and one bound row.
An independently proved must-advance atom with minimum at least 2^32 remains
unsatisfiable on the backend's addressable input domain; nullable atoms publish
their actual exact bounds. Forward and reverse bodies retain the same existing
capture-clearing and sequence-direction owners.

The 80-byte program header appends `RepeatStateByteLength` at offset 72. Following
the actual instruction and range sections, each dense repeat slot owns a 48-byte
row. `RegExpRepeatBoundWord` defines its six allocation-relative words:

| Offset | Word |
| --- | --- |
| 0 | MinimumDigitsOffset |
| 8 | MinimumDigitsLength |
| 16 | MaximumKind: Finite = 0, Unbounded = 1 |
| 24 | MaximumDigitsOffset |
| 32 | MaximumDigitsLength |
| 40 | StateOffset within the repetition state slab |

Rows precede their contiguous payloads in slot order. Each row owns its minimum
digits followed by its finite maximum digits. Unbounded maxima own zero offset
and zero length. Digits are nonempty decimal strings with no redundant leading
zero. The payload ends with zero padding to an eight-byte boundary, followed by
the unchanged named-group table. `RepeatBegin` stores its dense slot in operand
zero and zero in operand one; the immediately paired Guard names the same slot.

Descriptor admission proves the actual slot/bound bijection, reciprocal PCs,
region nesting and cross-region control flow before treating End as finite
progress. It rejects overlapping digit ownership, malformed or reversed natural
bounds, unused rows, unexpected padding and any state-layout disagreement. The
canonical static reserialization boundary includes these rows and payloads in
the program's encoded-byte identity. The GC program still owns one immutable
byte array; this representation adds no alternate matcher or mutable descriptor.

Each state row has a 40-byte metadata prefix: active, minimum used limbs, maximum
used limbs, pre-iteration UTF-16 cursor and Required/Optional stage. It then owns
`ceil(minimum digit length / 9)` four-byte little-endian limbs and the equivalent
finite maximum capacity, aligned to eight bytes. Limbs use radix 1,000,000,000.
Row offsets and the total state length are derived from actual admitted digits.
Thus program and counter storage grow with source digit length rather than the
numeric count. Existing choice and assertion snapshots must retain the complete
state slab, including all counter limbs.

Required empty iterations decrement exact counts. Optional empty attempts still
backtrack before committing their counters. Exact counting does not prove prompt
completion of astronomically many mandatory empty iterations or their nested
choices. Acceleration requires a separate semantic proof; a cutoff, timeout or
approximate count is not acceptance evidence.

The source controls retain the existing counted semantic/region tests and add
bounded descriptor construction beyond u64, canonical leading-zero handling,
finite versus unbounded ownership, nested/reverse/assertion admission and actual
alias/order/padding/state corruption rejection. These controls construct small
programs and do not execute huge empty matches. Compilation, semantic execution
and complete joined emitted-compiler/matcher verification remain deferred to the
whole batch's capped checkpoint.
