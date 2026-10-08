# Required empty RegExp replay

The actual counted End dispatch consumes one private proof for batching required
empty iterations. Its higher-ranked callback token retains both the admitted pair
and its actual MatcherWorkspace. The token has one consuming operation: subtract
the remaining minimum from a finite maximum, then clear the minimum. A caller
cannot substitute another workspace. The matcher returns to its original Guard;
no skipped iteration is materialized or re-entered.

Admission starts from a real successful body completion at the paired End, with
Active set, Required stage, unchanged UTF-16 cursor and a canonical exact minimum
greater than one. The minimum still includes the completed iteration. The first
body always runs through the actual native matcher and retains its outcome.
[ECMA-262 RepeatMatcher](https://tc39.es/ecma262/multipage/text-processing.html#sec-repeatmatcher)
permits mandatory empty repetitions, rejects further optional empty repetitions,
and preserves ordered body and continuation alternatives. The existing Guard,
progress and choice owners continue to implement those rules.

The proof scans the whole physical Guard-to-End region with an exhaustive opcode
classification. Unknown words refuse it. It admits capture writes, AssertStart,
AssertEnd and WordBoundary, replay-safe empty numbered/named references, and checked
positive finite exact nested regions. Consuming instructions and general Split,
Jump and ProgressSplit/Check remain refused outside a closed input-only assertion
or checked completed greedy attempt. Consequently a proved body has no surviving internal choices or direction
changes. Assertions see the
same immutable input, cursor, flags and per-instruction modifier on every replay.

A lookaround island must have the original Start, sentinel Split, reciprocal
End/Failure and exact shared continuation, polarity and parent direction. Every
internal branch/progress target stays in its actual assertion owner. Input atoms,
assertions, internal alternatives and nested assertions are allowed; capture
writes and references are refused there. Counted children use the original pair
admission and must have inactive canonical post-body rows. The real first assertion
outcome depends only on immutable input and its fixed entry cursor. Its original
matcher completion removes its private choices and restores the cursor and
whole repeat slab, so replay preserves the same result. This proof skips only
that validated physical island, leaving the ordinary assertion algorithm intact.

A completed greedy optional/star attempt has its exact forward primary entry and
local fallback. An ordinary star also has the reciprocal terminal Jump; a nullable
attempt has the actual greedy ProgressSplit/ProgressCheck pair and only its original
repeat or exit continuation. Ordinary atoms are straight-line and input-only;
nullable progress atoms also admit closed pure alternatives, progress loops and
inactive canonical child counters through the shared source-owner proof. Capture
reads and writes remain refused inside either atom. Closed input-only assertions
must end before its check or fallback. An ordinary attempt must have a mandatory
consuming instruction on its only success path. At the unchanged
whole-body cursor that attempt could only have failed and popped its saved choice.
A nullable greedy attempt instead uses the real no-progress Check to pop its frame.
Both leave the same input/capture/counter state and no continuation of their own.
Earlier choices remain intact; later failure still restores their original snapshots.
The greedy no-progress owner discharges the nullable atom's internal choices before
its own fallback. Lazy outer attempts and capture-dependent atoms require the Run
owner or normal matching and cannot use this predicate.

Capture writes retain their original ordered idempotence at the fixed cursor.
For every CaptureEnd, the actual post-body Start must remain initialized. A first
successful End followed by a later Clear does not establish replay, so such a body
keeps normal execution and any later CorruptProgram outcome. This conservative
Start check also applies in reverse matching.

A reference qualifies only when its actual post-body capture pair is canonical
undefined or empty within the input. From that retained fixed point, the proof
replays its ordered physical prefix in private emission locals. Actual Start,
End and overlapping Clear writes follow the current forward/reverse matcher
rules at the unchanged cursor. It checks End's read precondition and requires
an empty or nonparticipating pair at the reference. An open capture can be
temporarily nonparticipating; its post-body pair must still be canonical.
Positive exact nested bodies contain the same idempotent constant writes, so
their prefix has the same capture result after each required repetition.

A named reference checks every original candidate and counts participation
again at its replay prefix. More than one participating alias refuses batching,
including a first absent read followed by writes to two candidates. No workspace
capture or choice snapshot is changed by the predicate. Untouched captures and
balanced body-written empty captures share this one proof; no source width hint
alone establishes an empty reference.

A nested Begin uses the same checked version-three pair reader as actual counted
dispatch. Its dense row, reciprocal End/Exit and enclosing region are preserved.
The proof compares canonical decimal minimum and finite maximum bytes and requires
an equal positive value. At outer completion the child's actual slot must be
inactive, both canonical counters zero, Required stage, and PreUtf16 equal to the
current cursor. Its complete body is inspected by the same physical scan, including
all deeper regions. Positive exact bounds execute each admitted leaf at least
once; their Guards create no optional choices. Re-entry reinitializes the same
child and ends with the same captures and complete live child state. Zero,
unequal and unbounded child bounds do not receive this proof.

The original immutable descriptor, producers, byte admission, counter roles,
workspace extents and 512 MiB ceiling are unchanged. Existing whole-slab choice
save/restore and assertion repeat restoration are unchanged. Predicate work scales
with the source region, capture candidates and decimal archive, rather than the
numeric iteration count. Simple predicates allocate only native emission locals.
Nested assertion ancestry additionally uses one temporary GC byte array sized by
the admitted source instruction count, with no quantifier-sized allocation.

Three meaningful strict/sloppy Engine cohorts compare genuine literal and
runtime character-loop constructor producers with independent capture and UTF-16
results. They cover boundary/modifier assertions, astral cursor halves, absent and
aliased names, outer fallback, large nested exact bounds, reverse direction and
normal-path exclusions. Existing capture-only cohorts and the End/Clear corruption
control are retained. An additional actual byte-admitted control has flat and
nested bodies which first read absent aliases, then write both captures. One
iteration succeeds after a post-loop Clear. Two must reach the second ambiguous
named reference, throw CorruptProgram and retain lastIndex; empty post-body pairs
cannot authorize skipping that read.

Literal/computed strict/sloppy controls also cover completed greedy a*/a? attempts,
nullable assertion attempts, earlier greedy backtracking, finite optional maximum,
reverse direction, astral UTF-16 indices and retained normal-path exclusions.
These added controls have not run. A real byte-admitted greedy-shaped End/Clear
atom also proves that capture writes and a surviving optional fallback cannot be
treated as a pure completed attempt: one iteration succeeds; two must report the
existing corruption result and preserve lastIndex.

This is source work. No compilation, runtime result
or task completion is claimed. General escaping choices, capture-writing lookarounds without a surviving template,
nonempty capture-dependent references and nested optional repetitions still require their own
continuation or replay proof. None is approximated as no-match.
