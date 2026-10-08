# Computed UnicodeSets finite atom lowering

This is the narrow source contract for the emitted compiler's completed finite
atom consumer. Source preparation is uncompiled and unexecuted. Whole-batch
compilation, finite runtime controls and broad verification remain required.

The runtime class-set producer retains exact canonical string keys through
union, intersection, subtraction and nesting. Singleton keys join the ordinary
code-point bitmap. A completed finite atom contains only surviving strings of
at least two code points, one combined singleton matcher, and the empty-member
bit. Static MayContainStrings remains a grammar fact and does not follow whether
algebra eliminates exact string members. The lowerer cannot admit unvalidated
grammar or a syntax-only string marker.

The shared `FiniteClassSetAtomWord` owns the 48-byte workspace header: the
string-row pointer and count, the singleton opcode and two operands, and the
empty-member bit. `FiniteClassSetStringWord` owns each 16-byte row: a prepared
instruction pointer and code-point length. Each position is one existing
24-byte matcher instruction. Sensitive positions use LiteralCodePoint; folded
positions use UnicodeProperty ranges containing the complete simple-fold
equivalence class. Canonical keys own algebra; prepared position matchers own
input matching. Neither operation expands Unicode simple folding into multiple
code points. Rows retain descending code-point length and the producer's stable
lexicographic tie order.

`CheckedFiniteClassAtom` and `CheckedFiniteClassString` have private fields and
private constructors consumed by the actual width and atom routes. Loading an
atom requires its actual FiniteClassSet node, an aligned complete header span,
a Boolean empty-member value, and a complete row-table span. A row requires an
in-range index, length at least two and its complete prepared instruction span.
Spans are checked within the private compiler heap checkpoint and current
workspace end. Division occurs before multiplication/address addition, so an
unrepresentable count cannot wrap into a valid memory load. Empty row tables
may have an absent pointer; loaded headers and positions are nonempty. The
consumer's closed character-instruction domain permits only LiteralCodePoint
and UnicodeProperty. The existing final graph validator validates their actual
operands and sorted disjoint range slices.

Width is emitted instruction count, rather than consumed input length. For row
lengths `L`, row count `N`, and Boolean empty membership `E`, the complete atom
width is `sum(L) + 1 + 2 * (N + E)`. The mandatory singleton instruction
contributes one even when its range set is empty. Each choice except the final
alternative contributes a Split/Jump pair; the final empty alternative emits
no instruction. Width sums saturate at the existing instruction limit before
quantifier composition. The existing root width gate runs after complete
ordinary grammar and capture/name validation, before instruction emission or
descriptor publication. Erased quantifiers retain the existing zero-width
lifecycle, and oversized expansion remains a resource failure with checkpoint
rollback.

Width analysis validates descending row lengths and the nullable bit published
from the same completed empty-member fact. The existing group and quantified
owners then receive that fact. A required empty iteration remains an iteration;
an enclosing optional repetition owns its ProgressSplit/ProgressCheck. Internal
finite choices use ordinary Split/Jump and never mint their own progress owner.

Actual emission preserves the native finite atom's priority: all multi-code-point
strings first, then the singleton matcher, then empty. A failed later pattern
term can backtrack through those choices. Lookbehind preserves this priority and
reverses only the prepared instruction positions inside each string. The node's
validated direction and existing quantified task intervals compose with capture
and assertion owners. Every emitted instruction passes through the existing
checked writer, and the completed finite interval must end exactly where width
analysis assigned it.

The string payload does not expand into parser Sequence nodes. Those nodes have
a source-sized capacity, while a short immutable property escape can supply
thousands of finite strings. The durable payload belongs to the compiler's
private workspace and is consumed before compact descriptor publication. No
matcher opcode, program descriptor, public RegExp object representation or
alternate matching engine is added.

The 2026-10-05 follow-up repairs a Wasm predicate mismatch discovered by the
MAIN137 dense-array validation control. The checked finite atom now keeps
ContainsEmpty in an I32Local after validating the full I64 header word as 0 or 1.
Its conditional therefore loads the required I32 predicate. Nullable agreement
compares two I32 predicates; numeric width sums require the explicit
LowerWord::Predicate zero-extension. Passing this field to LowerWord::Local is
a Rust type error. The temporary header word and predicate use their matching
local release pools. Existing finite-string controls cover empty membership,
priority and nullable progress; no new mirrored control is added. This source
successor awaits compilation and runtime verification.
