# Math.sumPrecise runtime reduction

Status: normative for the Wasm-AOT `Math.sumPrecise` iterator walk, exact
finite accumulator and final binary64 rounding.

## Specification boundary

[`Math.sumPrecise`](https://tc39.es/ecma262/#sec-math.sumprecise) consumes a
sync iterable at run time. It obtains the iterator before allocating reduction
state, then repeatedly performs `IteratorStepValue`. Before accepting each
value it rejects a count of `2^53 - 1` with a `RangeError`, requires the value's
ECMAScript type to be Number without coercion, updates the reduction, and then
increments the count.
Those two algorithm-created abrupt completions perform `IteratorClose`; abrupt
completions from acquiring or stepping the iterator propagate directly.

The reduction starts in the specification's minus-zero state. Negative zero
does not change it, while positive zero enters the finite state. Finite values
are summed exactly. NaN and either infinity are observations rather than loop
exit conditions: later values must still be fetched and type-checked. Opposite
infinities produce NaN. A zero exact finite sum is positive zero, so only an
empty iterable or one containing exclusively negative zeros returns negative
zero.

## Closed iterator-error policy

The shared sync-iterator helpers accept a closed `SyncIteratorErrorPolicy`
instead of loose message and realm parameters. Existing array/destructuring
consumers select `LegacyMainRealm`; `Math.sumPrecise` selects
`MathSumPrecise`. An exhaustive mapping from a private protocol-error enum
chooses the message and whether the TypeError is allocated in the main realm
or the current builtin function's realm. The Math policy covers a non-iterable
input, a non-object iterator-method result, a non-callable cached `next`, and a
non-object `next` result. It also boxes primitive iterator inputs with the
current function realm's intrinsic prototypes, so a created-realm Math method
observes that realm's `String.prototype[Symbol.iterator]`.

`GetIterator` failures and failures from `next`, `done`, or `value` access are
propagated unchanged and do not close the iterator. Once a value has been
obtained, only the count overflow and exact-Number rejection paths create an
error, close while preserving that throw, and return the completion. A throw
from `return` therefore does not replace either algorithm-created error.

The helper also runtime-discriminates a dynamically tagged Arguments object
into the backend's existing exotic default-iterator lookup. That makes an
entry-realm `arguments` value consumable through the builtin's dynamic
parameter. The current Arguments representation still virtualizes this lookup
as entry-realm `Array.prototype.values`; it cannot yet observe an own
`arguments[Symbol.iterator]` override or preserve a created-realm Arguments
iterator identity. Those are explicit pre-existing Arguments-object gaps, not
claims of this Math seam.

## Fixed exact accumulator

Every finite binary64 is an integer coefficient times `2^-1074`. A subnormal's
fraction is already that coefficient. A normal with exponent field `e` has a
53-bit significand shifted left by `e - 1`, so the largest coefficient has
2098 bits. Fewer than `2^53` accepted terms have an absolute sum below
`2^2151`.

The runtime therefore allocates one fixed signed two's-complement buffer of 34
little-endian `u64` limbs (2176 bits). A positive finite term is added and a
negative finite term is subtracted modulo `2^2176`; the proven bound means the
signed interpretation cannot overflow. This is constant space and linear time
with a fixed 34-limb factor. It performs no binary64 addition during the finite
fold.

Finalization converts a negative buffer to magnitude, finds its highest set
bit and extracts a 53-bit significand. Discarded bits supply guard and sticky
information. The finisher rounds to nearest, ties to even, renormalizes a
carried significand, emits infinity only when rounding crosses the binary64
limit, and reapplies the exact sign. Magnitudes below bit 52 are exact
subnormals because every coefficient is integral in `2^-1074` units.

The constants encode the proof: 2151 required signed magnitude bits, 64 bits
per limb and exactly 34 limbs. `MathSumPreciseState` is a closed domain for the
five specification states. The sole reducer produces a private, non-`Copy`,
`#[must_use]` `CompletedMathSumPreciseReduction`; only the finisher consumes
it. Invalid internal state words trap instead of silently selecting a result.

## Runtime-only route and durable witnesses

Lowering no longer recognizes literal arrays, generators, or overridden array
iterators for compile-time `Math.sumPrecise` evaluation. Every call reaches the
same runtime iterator and exact accumulator path. The generic static generator
and iterator materializers remain outside this seam.

`wasm_math_sum_precise_runtime.js` exercises runtime arrays, generators,
custom and overridden iterators, `arguments`, signed-zero and exact-cancellation
states, adversarial rounding, overflow, NaN/infinity precedence, rejection
without coercion, continued iteration after exceptional values, close-on-type
failure, direct propagation of iterator abrupts, and created-realm TypeError
selection and primitive iterator-prototype lookup. A bounded source-structure
test pins the closed policies, ordering, proof constants, exact finite fold,
final rounding, planning roots and absence of the former static route.

## Owned files and verification

- `crates/lila-aot-wasm/src/builtins/math.rs`
- `crates/lila-aot-wasm/src/builtins/host.rs`
- `crates/lila-aot-wasm/src/control_flow.rs`
- `crates/lila-aot-wasm/src/builtins/array.rs`
- `crates/lila-aot-wasm/src/planning.rs`
- `crates/lila-aot-wasm/src/data.rs`
- `crates/lila-ir/src/lowering.rs`
- `crates/lila-ir/src/lib.rs`
- `crates/lila-aot-wasm/tests/math_sum_precise_runtime_structure.rs`
- `crates/lila-cli/tests/fixtures/wasm_math_sum_precise_runtime.js`
- `crates/lila-cli/tests/cli/language_numerics.rs`
- this contract
- `tasks/20-number-bigint-math-json.md`

The lane's freeze gates are scoped `rustfmt --check`, `node --check`, source
inventory and `git diff --check`. Cargo, the CLI fixture, pinned Test262, the
complete Math tree and the broad batch ladder remain owned by the central
verifier.

## Nonclaims

This seam does not claim a current-HEAD ten-of-ten `Math.sumPrecise` Test262
result before those tests run. It does not close the rest of Math or T20, make
the generic iterator helpers realm-complete for every consumer, repair own or
created-realm Arguments iterator lookup, or add generic generator-close
behavior. The `2^53 - 1` count failure is structurally pinned but is not
practical to exercise through the runtime fixture. The fixed accumulator is a
correctness design, not a claim of optimal throughput.
