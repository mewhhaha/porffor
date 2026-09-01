# Nullable RegExp quantifier progress

Status: normative implementation contract for the Rust IR matcher-program
producer and Wasm-AOT matcher.

## Conformance boundary

The pinned raw Test262 witness is:

- `built-ins/RegExp/nullable-quantifier.js`

Its two executions are currently rejected before matching with
`unbounded quantifier over a nullable atom is unsupported by this
matcher-program grammar`. This batch removes that rejection and implements the
`RepeatMatcher` progress rule in the real compiled matcher program. The exact
witness `/(a?b??)*/` must match all of `"ab"`: the first optional iteration may
consume `"a"`, the next may consume `"b"`, and a later empty optional iteration
must be discarded rather than accepted or repeated forever.

## Normative lifecycle

A quantifier has three closed producer facts:

1. its required iteration count;
2. either a finite optional iteration count or an unbounded optional tail; and
3. greedy or lazy branch preference.

Required iterations and optional iterations are not interchangeable. A
nullable atom may match the empty string while the remaining minimum is
non-zero. Once the minimum has been discharged, an optional iteration whose
end index equals its start index fails that iteration. Failure restores the
captures that existed before the attempt and continues through the ordered
fallback. A successful iteration which changes the end index preserves its
captures and continues to the next finite optional iteration or the unbounded
tail.

The matcher program represents every nullable optional attempt with one paired
control edge:

- `REGEXP_OPCODE_PROGRESS_SPLIT` owns the attempt target, the fallback target,
  and greedy/lazy preference. Its runtime choice frame owns the input position
  and capture snapshot from before that attempt.
- `REGEXP_OPCODE_PROGRESS_CHECK` names that exact split and the continuation
  taken only after a changed end index. If the end index did not change, normal
  ordered backtracking is entered, thereby trying still-pending alternatives
  inside the atom before eventually restoring the split's pre-attempt captures
  and taking its fallback.

The producer's private, non-`Copy`, `#[must_use]`
`PendingNullableQuantifierProgress` is minted only when the progress split is
emitted and consumed only when the matching progress check is emitted. A check
therefore cannot accidentally name an unrelated split, and a nullable optional
attempt cannot be completed without its progress check. Completing the check
mints a private, non-`Copy`, `#[must_use]`
`PendingNullableQuantifierFallback`; consuming that second carrier is the only
producer path which patches the split's fallback. A successful finite optional
iteration continues to the next attempt, while an empty attempt skips every
remaining optional iteration and enters the continuation after the complete
quantifier. A successful unbounded iteration continues to its own progress
split; its empty fallback exits the quantifier.

Each syntactic attempt has its own split instruction identity. Nested nullable
quantifiers consequently select the newest active frame bearing the named split
identity; they do not share one mutable progress slot. Choice frames retain the
progress identity and start position through backtracking, together with the
capture snapshot they already own. Lazy execution may reach the attempt only
after restoring its ordered choice, but restores the same progress authority
before entering the atom.

Forward and reverse matching use the same paired representation. Progress is
equality of ECMAScript UTF-16 input indices, so it is direction-independent:
advancing forward and retreating in lookbehind both count as progress, while an
unchanged index does not.

The public bytecode remains fixed-width. For `REGEXP_OPCODE_PROGRESS_SPLIT`,
`operand0` is the attempt instruction index and `operand1` packs the fallback
instruction index in bits 1 and above and the lazy bit in bit 0. For
`REGEXP_OPCODE_PROGRESS_CHECK`, `operand0` is the paired progress-split
instruction index and `operand1` is the successful-progress continuation.
Targets are absolute instruction indices and receive the same bounds validation
as `Split` and `Jump`.

## Rust invariants

- `OptionalAtomProgress::{MustAdvance, MayRemainAtSameIndex}` is private and
  derives no cloning, copying or comparison capability. Forward and reverse
  lowering each classify the atom once, after emitting every required
  iteration, and consume that classification in the selected finite or
  unbounded optional branch. A second consuming, by-value observation of the
  same classification is therefore a Rust move error.
- The type system cannot prevent borrowing or explicitly recomputing the
  classification. The focused structure guard consequently forbids those
  observations by owning the exact two-constructor, four-`MustAdvance`-arm and
  four-`MayRemainAtSameIndex`-arm census and the complete ordered forward and
  reverse quantifier bodies.
- `QuantifierOptionalIterations::{Finite, Unbounded}` is exhaustive; infinity
  is not a sentinel integer.
- `QuantifierPreference::{Greedy, Lazy}` is exhaustive; preference is not a
  Boolean whose meaning can be inverted at one call site.
- Required atoms are emitted without optional-progress rejection. Every
  nullable optional atom is emitted through the paired progress builder.
- Finite and unbounded optional lowering, forward and reverse lowering, and
  greedy and lazy lowering use exhaustive matches without catch-all arms.
- The parser's nullable-unbounded rejection is deleted. Nullability selects a
  bytecode representation; it is no longer a capability error.
- Program validation and resource accounting recognize both progress opcodes.
  A progress split is an ordered choice and contributes a choice frame. A
  progress check is a control edge, not a consuming instruction.

## Explicit nonclaims

This batch does not replace the ordered-backtracking matcher, change RegExp
source parsing, add dynamic-source evaluation, or widen the existing supported
atom/class grammar. It does not claim all RegExp Test262 coverage. Lookaround,
backreference, Unicode-set, or legacy grammar cases which are independently
unsupported remain independently unsupported; the progress representation only
removes nullable-quantifier rejection when the contained atom is otherwise a
valid matcher program.

## Verification

Cheap producer checks:

```sh
cargo fmt --all -- --check
cargo test -p lila-ir nullable_quantifier_progress
git diff --check
```

Integrated focused verification:

```sh
cargo test -p lila-cli --test cli regexp -- --nocapture
./target/debug/lila test262 run built-ins/RegExp/nullable-quantifier.js \
  --suite-root test262/vendor/test262 --execution-backend wasm-aot \
  --snapshot-name regexp-nullable-quantifier --timeout-ms 180000 --threads 1
```

The Test262 result is exactly `2/2`; broader RegExp and pinned-matrix
publication remain later verification steps.

The one-shot `OptionalAtomProgress` ownership hardening is source-equivalent:
it changes no matcher-program instruction, branch or evaluation order. Its
dedicated structure executable remains `5/5`, the exact IR unit is `1/1`, and
the nullable-progress and retained quantifier CLI witnesses are each `1/1`.
Formatting and scoped diff checks are green. Test262 was not rerun for this
capability-only follow-up; the earlier exact `2/2` result above remains the
behavioral record. Independent review confirmed the exact route census and
both complete quantifier bodies. The coordinated workspace checkpoint passes
`cargo fmt --all -- --check`, `cargo xc`, `git diff --check`, the
module-boundary check and the task-plan check; the compile retains the
repository's existing warnings.
