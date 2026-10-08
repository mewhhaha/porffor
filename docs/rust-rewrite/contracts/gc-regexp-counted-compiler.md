# Counted RegExp compiler contract

The static Rust compiler and emitted RegExp constructor compiler retain one atom
body for counted repetitions. Numeric bounds do not expand parser nodes,
instructions, captures or repeat slots. The actual source-body instruction ceiling
remains 32,768; finite Unicode string-set lowering retains its existing limits.

The semantic reference is [ECMAScript RepeatMatcher](https://tc39.es/ecma262/multipage/text-processing.html#sec-runtime-semantics-repeatmatcher-abstract-operation).
Required empty successes reduce the minimum. After the minimum is satisfied, an
attempt at the same input index fails; newer alternatives remain eligible for
backtracking. Each attempt clears the atom's capture range. Greedy/lazy choices,
reverse matching and atomic assertion completion keep their established order.

## Immutable descriptor

The branded immutable `RegExpProgram` remains the sole descriptor. RGPB version 3
has ten little-endian u64 header words (80 bytes). Existing offsets are retained:
`RepeatSlotCount` is at 64 and `RepeatStateByteLength` is at 72. Instructions retain
three u64 words (24 bytes); the named-group table version remains unchanged.

| Opcode | Word | Operand 0 | Operand 1 |
| --- | ---: | --- | --- |
| `RepeatBegin` | 26 | dense bound slot | zero |
| `RepeatGuard` | 27 | paired End PC | dense slot shifted left one, lazy bit zero |
| `RepeatEnd` | 28 | paired Begin PC | zero |
| `RepeatExit` | 29 | paired Begin PC | zero |

The physical order is Begin, Guard, one body, End, Exit. Guard immediately follows
Begin, End precedes Exit, and End/Exit backlink their Begin. Guard selects the body
or Exit; End continues at Guard; Exit deactivates the slot. Optional/star and
consuming-plus layouts retain their ordinary/progress opcodes. Nullable plus and
larger finite/required layouts use the counted lifecycle.

Each slot owns one 48-byte bound row after the range pool: canonical minimum digit
offset/length, closed maximum kind, finite maximum digit offset/length, and the
exact live-state offset. Canonical ASCII digits are contiguous, source-sized and
allocation-relative. Unbounded has its own kind and zero maximum extent. Zero
alignment padding precedes the unchanged named-group section. State extents are
checked sums of a 40-byte metadata header and source-sized base-10^9 u32 limbs.

Static lowering consumes a pending pair to write End/Exit and seal Guard. Emitted
lowering has the same owner and queues the atom once. Its width is atom width plus
four. Source-sized decimal spans and rows are retained before publishing the blob;
output writes cannot overwrite unconsumed pattern text.

## Admission and termination

`RegExpNatural` retains canonical mathematical decimal values without a machine
integer cutoff. `RegExpRepeatBounds` validates the original ordered minimum/maximum
pair. Finite values at and beyond u64::MAX remain finite; Unbounded is separate.
Leading zeroes normalize once. Both actual producers reject reversed source spans.
An oversized consuming minimum may retain proven never-match lowering because it
cannot fit addressable input; nullable minima retain exact bounds. Finite maxima
are never approximated as Unbounded. Resource rejection concerns genuine source
and descriptor capacities.

Both validators check canonical digits, section extents, finite ordering, padding
and exact state size before admitting a layout. They certify reciprocal PCs,
nesting, unique dense slots and active-region ownership. Ordinary flow cannot enter
Guard/Exit, leave a body or enter another active region. Accept cannot occur inside
an active region. The emitted region proof uses bounded scratch on the existing GC
byte-array representation and clears its scratch reference on admission/rejection.

The non-consuming proof treats a certified End as eventually reaching its Exit.
It still rejects surrounding empty cycles that re-enter Begin and reset a counter.
Runtime flow and repeatable-choice accounting retain the real End-to-Guard edge;
the static termination proof does not replace that execution edge.

## Matching and source controls

The matcher retains admitted slot/state extents before consuming the checked view.
Live counters use exact base-10^9 limbs; minimum and maximum have different sealed
types. Decrement borrows and normalizes the used prefix. Ordered choices and
assertions snapshot the complete metadata/limb slab with cursor/captures. Scratch
grows on demand under the unchanged 512 MiB ceiling, without allocation from a
numeric bound.

The [capture-only acceleration](regexp-capture-only-counts.md) has an actual scoped
proof for mandatory empty capture writes. It consumes an exact remaining count
after one successful body execution. General empty choices, assertions, references
and nested repeats remain outside that proof.

Retained controls cover greedy/lazy fallback, capture clearing, nested choices,
assertions, reverse matching, malformed paired ownership and counter-reset cycles.
Exact-bound controls add canonical descriptor round trips and damage rejection,
finite radix boundaries and counts beyond u64. Native Engine cohorts pair literal
and runtime character-loop producers against independent expected results. All
eight existing AOT control names remain; three obsolete numeric-domain source
mirrors now validate real descriptors and real native Wasm emission.

The transactional compile-rejection witness now exceeds the genuine source-body
instruction ceiling. It verifies that a failed recompile retains the receiver's
source, flags, lastIndex and old executable program. Large numeric bounds are no
longer treated as resource rejection by that control.

This batch is source-authored and isolated-formatted. Compilation, descriptor
controls and native execution remain unrun until the combined capped checkpoint.
It is not a conformance or verified acceptance result.
