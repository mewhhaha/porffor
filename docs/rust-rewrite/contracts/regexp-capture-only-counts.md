# Required capture-only repetition batches

The matcher may batch a mandatory repetition only from the actual admitted
instruction body, after one successful iteration. CaptureStart, CaptureEnd and
ClearCaptureRange retain the capture-only proof below; an empty interval also
qualifies. The consumed owner now also has the separate
[required-empty replay proof](regexp-required-empty-replay.md) for fixed-position
assertions, untouched empty references and positive exact nested repeats. Choices,
progress, jumps, lookarounds and consuming instructions remain excluded. No producer
annotation or numeric cutoff authorizes a batch, and the version-three immutable
descriptor is unchanged.

The opcode allowlist alone does not prove that a byte-admitted body can replay.
Forward CaptureEnd reads its start word. For each such instruction the scanner
also requires that capture's actual post-body start is initialized. Any earlier
clear of that start without an intervening start would already have failed the first
successful iteration; otherwise replay reads that initialized post-body value.
Reverse endpoints only write, so this additional check is conservative and
admits the balanced reverse bodies emitted by both real source producers. A
body that ends by clearing an inherited capture start stays on the ordinary
path and preserves any later CorruptProgram outcome.

After the read precondition is proved, these accepted instructions overwrite
capture words with the same UTF-16 cursor or the undefined sentinel, preserving
untouched words. Their ordered composition is therefore idempotent at that cursor
in either matching direction. The first
iteration still executes every real instruction and preserves its normal or
abrupt outcome. At its End, the proof also requires the real Required stage,
unchanged cursor, active paired slot and an exact minimum greater than one.

The current minimum M includes the completed iteration. A finite maximum is
replaced by maximum minus M before the minimum is cleared. An unbounded maximum
remains unbounded. Capture words remain those of the successful first iteration.
Returning to the ordinary Guard reproduces the state after all M mandatory
iterations; optional greedy/lazy choice and empty-progress rejection continue
through their existing owners. Minimum zero or one keeps its ordinary path.

`NaturalCounterLocals` uses sealed MinimumCounter and MaximumCounter roles.
Only MaximumCounter has subtract_minimum, and its operand must be MinimumCounter.
The operation checks canonical used prefixes, limb ranges, zero unused capacity
and exact maximum/minimum order before mutation. It subtracts little-endian
base-1,000,000,000 limbs with exact borrow, requires zero final borrow and trims
all high zero limbs. Clearing the minimum zeros its complete source-sized
capacity and used length. The selected descriptor's disjoint regions remain the
arithmetic ownership authority.

The required-empty emitter opens and closes its own runtime scopes. A private
proof retains the actual paired loop, its exact workspace and a higher-ranked
callback lifetime, so it cannot escape the If that admitted the body or accept a
foreign workspace. Its sole consuming method
performs maximum subtraction followed by minimum clearing. The parent callback
sets an acceleration flag; the parent End dispatch owns normal-versus-batched
decrement and its existing instruction-loop branch. Choice and assertion copies
continue to retain the full existing state slab, including every limb.

The authored strict/sloppy Engine cohort compares literal and genuinely runtime
constructed programs against independent UTF-16/capture expectations. Tiny
patterns exercise beyond-u64 and thirty-one-digit minima, exact/finite/unbounded
maxima, optional-empty capture distinction, adjacent counter slots, reverse
matching, positive/negative assertions and outer backtracking. Small nullable
choice, assertion, backreference and nested-repeat cases retain the normal path.
The controls are unrun source obligations. Compilation, runtime and whole-batch
verification remain pending; no timeout is treated as a passing result.

The native program-boundary control also replaces the exact embedded allocation
with a structurally admitted graph whose CaptureStart lies outside the repeated
body, followed inside by CaptureEnd and ClearCaptureRange. One iteration must
succeed through the real native byte gate with an undefined capture. Two must
reach the next ordinary CaptureEnd and preserve its CorruptProgram exception and
transactional lastIndex. Both use the same paired graph and allocation footprint,
so an earlier admission rejection cannot satisfy the successful control.

General huge nullable bodies with escaping choices, capture-dependent lookarounds, nonempty capture-dependent
references or nested optional repeats still require
separate acceleration proofs. This source batch does not claim their prompt
completion or general RegExp conformance.
