# Independent counted body transitions

The actual counted Begin can translate a mandatory count only after proving the
complete source body resets every capture it can write before entering any
alternative, and has fresh source-owned child counters and progress attempts.
The proof is bound to the admitted pair and matcher workspace inside an HRTB callback; no
caller can manufacture a different count or clear interval.

The first actual body instruction supplies the capture Clear interval. Every
CaptureStart, CaptureEnd and descendant Clear, including writes in nested
assertions, must lie in that interval. A body without an entry Clear cannot
write captures. Captures outside the interval remain unchanged. Input atoms and
backreferences use the original matcher. Outside assertions, every nonzero
successful transition advances by at least one UTF-16 unit in the unchanged
parent direction. Structurally discharged assertions restore all three cursor
words and their parent direction. The shared source scanner still
checks exact assertion direction/polarity/ancestry, ordinary edges, canonical
progress attempts, distinct nested repeat rows and inactive child lifecycles.

At a given cursor, every mandatory iteration consequently starts with unchanged
external captures and all of its own captures unmatched. Its ordered successful
transitions depend on that cursor and immutable input, not on a previous
iteration's capture output. The continuation can inspect the last iteration's
captures. Intermediate output disappears at the next actual entry Clear.

Let R(k,D,p) be the original ordered continuation result from k mandatory
iterations at cursor p with fixed optional gap D. Let N bound the UTF-16 input
remaining in the parent direction. For k=N+1, every positive-consuming branch
calls a smaller-input state with at least its own remaining-input bound plus
one mandatory iteration. Induction makes those consuming continuation results
constant for larger k. Each zero transition calls the same R(k-1,D,p), because
its capture output is erased before the next body. The remaining ordered
operator therefore has fixed consuming-result constants and identical zero
branches. It is idempotent: a successful constant before the first zero branch
dominates; otherwise that zero branch returns the previous successful result,
or the first later successful constant. Failure is retained when both fail.
Applying it twice changes neither selected priority nor final captures. Thus
R(k,D,p) stabilizes at N+1, including later failure/backtracking. N=0 supplies
the base: one real iteration retains every possible final capture result.

The implementation uses the full actual input UTF-16 length plus one as a
conservative bound for either direction. The matcher reads that length from
the immutable input String's CodeUnitArray before entering any candidate,
including patterns with no captures. Capture count cannot decide whether this
bound is initialized. If the source scanner proves every
path is zero-width outside restoring assertions, the bound is one. No cursor
equality or equality between observed templates substitutes for this complete
transition and continuation proof. Last-iteration choices, assertion atomicity,
reverse behavior and capture snapshots still use the original physical body.

For mandatory count M greater than the proven bound L, the proof translates it
to L. A finite maximum X becomes X-(M-L): bounded subtraction computes the exact
skipped count in all source-sized limbs, and the original maximum-minus-minimum
operation subtracts that same count. The optional gap X-M is unchanged; an
unbounded maximum remains unbounded. Counts at or below the bound retain
the original path. The original program descriptor, exact decimal bounds and
compiler producers are unchanged. Both literals and computed RegExp
constructors consume this native Begin proof.

This proof closes successful empty alternatives that would otherwise reach End
and recreate same-row runs, including independent consuming alternatives. It
does not weaken the Run's actual-End observation. A capture write outside the
entry Clear, active child, escaped owner or unproved fresh progress/counter
lifecycle prevents translation. Such unproved program shapes keep their
original matcher path rather than acquiring a certificate from observed state.

The authored Engine cohort covers huge successful empty choices followed by
failure, last-iteration capture selection through a failing continuation,
capture-writing positive/negative assertions, unchanged outside captures,
anchors, reverse matching, nested zero-width repetition and zero mandatory
counts. A second cohort compares N+1, N+2 and huge mandatory bounds with zero
alternatives before/between/after consuming alternatives, capture-dependent
continuation selection, unchanged outside references, finite optional gaps,
astral UTF-16 consumption, reverse indices and nested failed/successful cycles.
It exercises literal and computed producers in strict and sloppy code. These controls are unrun; no
compilation, formatting or runtime result is claimed.

The capture-free regression cohort retains the legacy `\u{3}` and `\p{2}`
identity-escape repetitions and checks exact, greedy/lazy bounded, reverse and
astral UTF-16 matches through literal and computed construction in strict and
sloppy code. It includes short-input rejection and exact match indices. These
new controls are pending execution; the original legacy grammar expectation
is unchanged.
