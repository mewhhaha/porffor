# RegExp counted matcher workspace

The emitted matcher consumes the sole checked immutable GC RegExpProgram and
String roots. Temporary bytes contain program/text copies and numeric matching
state; they do not represent JavaScript objects or serve as an interpreter.
The counted opcode contract is defined in `gc-regexp-counted-compiler.md`.

`MatcherWorkspace::allocate` runs after immutable program/text materialization.
It owns the private allocation tail until match completion rewinds the saved
checkpoint. An ordered choice can only be published through the workspace:
`ensure_choice_bytes` checks actual tail ownership, then doubles byte capacity on
demand, clamped to the existing 512 MiB matcher scratch ceiling. Memory growth
and address admission precede allocation. Another transient allocation cannot
silently become part of a choice. No numeric repetition bound sizes an arena.

The caller allocates only its capture output prefix (16 bytes per capture).
The workspace contains a live capture prefix, dense counted repeat slots, and
an initially single-snapshot choice arena. Each repeat slot has five u64 header
words and exact source-sized base-10^9 minimum/maximum limbs. The header contains
active state, used minimum/maximum lengths, pre-attempt UTF-16 cursor and the
closed required/optional iteration stage. A Snapshot choice contains byte
length, previous arena-relative link, closed kind, fallback and originating PC,
all three cursor words, and a complete live-state snapshot. Every walk and
restore stays bound to the same private workspace; no caller supplies a foreign
slab or indexes a frame by physical depth.
The scratch ceiling includes caller output, live state, and every choice.

RepeatBegin initializes its admitted slot. Guard requires another body
execution while the minimum is positive; a required empty success decrements
the minimum and finite maximum. Once the minimum is zero, a body success at the
same UTF-16 cursor fails before decrementing a counter. The existing ordered
backtracker tries newer atom alternatives first. Greedy Guard saves Exit and
runs the body; lazy Guard saves the body and selects Exit. Both snapshots
retain the active optional attempt, so selecting a later fallback restores the
exact counter and capture state before that attempt. Exit deactivates only its
own slot.

Every ordinary/progress choice snapshots the full live state. Backtracking
restores counters together with captures. Positive assertion completion keeps
successful captures while restoring enclosing repeat state from its entry
frame; negative success restores both. Each new unanchored candidate resets
repeat slots. Immutable pair/slot admission checks precede state addressing.

Successful matching copies captures into the caller prefix before scrubbing
and rewinding matcher-owned bytes. Failed matching and resource/corruption
returns rewind the same checkpoint. Caller completion subsequently scrubs its
output prefix and rewinds its own mark before an observable exit. Allocation
failure preserves the existing matcher status or realm-owned RangeError flow.

The matcher remains a self-contained raw Wasm control body. Counted dispatch
adds two If frames around End; its backtracking branch uses caller depth two,
and successful dispatch branches outside both If frames to the instruction
loop. Structural witnesses check this consumed boundary; the semantic Engine
controls exercise literal and computed programs, required empty iterations,
same-cursor retained choices, greedy/lazy fallback, captures, reverse matching,
and positive/negative assertion completion.

A RequiredRun is published only by the checked two-template required-iteration
proof. Its source-sized counters and immutable full template represent every
ordered virtual fallback; they never encode logical depth in a machine word.
Pop restores the whole snapshot and substitutes only the authorized outer
minimum/maximum counters. Assertion and progress searches traverse the same
linked arena and retain their original matching predicates. Run payloads and
templates are aligned to eight bytes, matching the private allocator and its
exclusive-tail accounting. The existing scratch ceiling remains unchanged.

This is source-authored work. Formatting and independent source review precede
the joined compile and semantic checkpoint; this contract records no runtime
or conformance pass.
