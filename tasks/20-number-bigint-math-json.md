# T20 — Number, BigInt, Math and JSON

**Status:** In progress — exact core BigInt operators and broad numeric/JSON support exist; full closure remains

**Parallel group:** Feature lane; split internally by Number, BigInt, Math and JSON  
**Depends on:** T04, T05, T10, T18  
**Blocks:** Numeric and JSON portions of T22-T23/T26

## Current repository state

Number operators/conversions, Math builtins, JSON parsing/stringification and
inline/heap BigInt representations have extensive Wasm implementations and
focused real-suite coverage. Core multi-limb BigInt arithmetic, comparison and
binary bitwise operators now share the exact arbitrary-precision runtime
representation. The binary bitwise path evaluates both operands before its
ordered ToNumeric conversions, rejects mixed numeric types and BigInt `>>>`,
implements negative shift-count reversal and reports unaddressable left-shift
results as an explicit resource RangeError. Unary complement now has its own
closed IR operation and an exhaustive Number-versus-BigInt backend dispatch:
the operand is evaluated once and crosses one ToNumeric boundary, the Number
arm consumes the shared exact modulo-2^32 emitter, and the BigInt arm applies
the existing arbitrary-precision XOR representation to `-1n`. Literal folding
uses the same mathematical identity without narrowing. Remaining
conversion/builtin integrations are still open. The private backend
`UnaryNumericKind::{Number, BigInt}` authority now derives no incidental
capabilities. Its exactly two producers remain ordered after the single
evaluation and `ToNumeric` conversion, while its sole exhaustive consumer
retains the exact BigInt call and Number instruction sequence. The recursive
source target passes `7/7`, the exact combined Number/BigInt CLI witness passes
`1/1`, and the pinned Number and BigInt bitwise-not Test262 leaves pass all
`4/4` ordinary executions with every failure bucket at zero. This derive-only
hardening is source-equivalent and expected to leave emitted Wasm
byte-identical; no broad suite, golden or conformance-count change is claimed.
Independent dry review is clean, and the shared format, `cargo xc`, diff,
module-boundary and task-plan checkpoint is green with the workspace's existing
warnings.
The arithmetic Number-pair conversion boundary now accepts the existing closed
`ArithmeticBinaryOp` directly. Its retired `NumericBinaryOperator` wrapper
admitted a never-constructed bitwise state and five incidental capabilities,
although all three real callers immediately wrapped an arithmetic operator.
One private exhaustive selector keeps `Add`'s ToPrimitive-both-before-ToNumeric
order distinct from the left-then-right Number conversion used by `Sub`,
`Mul`, `Div`, `Mod` and `Exp`. The helper and caller bodies otherwise retain
their execution order. The new recursive structure target passes `4/4`, and
the neighboring unary-numeric target remains green at `7/7`. The shared
`cargo xc` checkpoint is green. The pinned addition and multiplication order
controls pass all `4/4` sloppy/strict Wasm-AOT executions with every failure
bucket at zero. The bounded contract is
[`arithmetic-number-conversion-order.md`](../docs/rust-rewrite/contracts/arithmetic-number-conversion-order.md).
The BigInt prototype result
boundary now carries a closed exact-value, radix-string or locale-fallback
policy from each existing builtin producer. After shared receiver extraction,
marker-typed helpers make locale calls ineligible for the radix reader:
`toLocaleString` uses the permitted decimal core fallback and leaves its two
reserved arguments unused, while `toString` retains radix coercion/error order
and `valueOf` retains the exact representation. One prepared-radix witness
owns coercion and range validation before immediate-versus-heap formatting,
and a closed builder body-domain lets only standard builtins and the
`ValueToNumber`/`ValueToNumeric` helpers interpret their environment as
Realm-or-zero state. Main, user, host and every other helper body use the
global error fallback, while created-Realm BigInt prototype methods carry both
their TypeError and RangeError prototype slots. BigInt/Symbol `TypeError`s and
radix `RangeError`s therefore use the same defining-realm policy for immediate
and heap representations without exposing lexical environments. The focused
[result-policy contract](../docs/rust-rewrite/contracts/bigint-prototype-result-policy.md)
and registered CLI fixture cover immediate, heap and boxed values. This batch
has run only static gates for the seam; it does not implement or claim
`Intl.NumberFormat` or locale-aware ECMA-402 output. The Number side now has one
backend authority for exact modulo-2^32 conversion across unary and binary
bitwise operators, Array length conversion, String split limits,
`String.fromCharCode`'s `ToUint16` projection, `Math.imul`, `Math.clz32`, every
integer typed-array and Atomics store, and every integer DataView setter. The
emitter keeps the modulo in binary64 before its non-trapping unsigned
conversion, so NaN and infinities become zero while finite magnitudes at and
above 2^63 retain their low bits; signed consumers interpret those same low 32
bits only after conversion. A registered dynamic CLI fixture covers the large
positive and negative residues plus observable evaluation/coercion order. Its
focused Wasm product-path test has not yet been executed in this batch while
the repository-wide conformance matrix owns the verifier. The Math.random
product path uses a realm-owned
`HostRandom` capability: the only provider result type validates the exact
finite `[0, 1)` domain, the production provider maps 53 operating-system
entropy bits to binary64, and the builtin catalog alone decides whether the
optional host import exists. A deterministic injected provider covers the same
engine path without making production output constant. The central
feature-enabled CLI compile is green, as are focused checks for the typed host
import and injected provider. The full Number,
BigInt, Math and JSON current-pin trees have not met this task's shortcut-free
acceptance gate. The Math extremum backend now consumes the complete runtime
argument vector for `Math.min` and `Math.max` instead of a three-argument
prefix. One closed operation domain owns each identity and Wasm reduction,
while the loop applies `ToNumber` left-to-right to every argument even after
`NaN`; abrupt conversion stops the walk unchanged. The focused
[extremum-reduction contract](../docs/rust-rewrite/contracts/math-extremum-argument-reduction.md)
and CLI fixture cover arguments beyond the old cap, signed zero, later
coercion and later abrupt completion. This batch has run only static gates for
that seam while the repository-wide conformance matrix owns the verifier. The
private `MathExtremum::{Minimum, Maximum}` authority initially derived only
`Clone` and `Copy`; its unused equality, debug and default-style capabilities
were absent. Two exhaustive projections retain the exact identity and Wasm
instruction pairings, and the `Math.min` / `Math.max` builtin arms remain the
only producers. The recursive structure target passes `3/3`, the neighboring
`Math.hypot` structure target passes `3/3`, and the focused package format check
is green. The exact CLI witness passes `1/1`, and six focused Test262 leaves
pass all `12/12` sloppy/strict Wasm-AOT executions with every failure bucket at
zero. Independent dry review is clean and `cargo xc` passes. This capability
closure is source-equivalent; its semantic-golden checkpoint remains deferred.

Batch AH removes those remaining `Clone` and `Copy` capabilities. The extremum
emitter owns one selection and borrows it for the identity and combine
projections, so the two coupled decisions cannot be forked through an implicit
copy. The two producer arms, both exhaustive tables and every emitted
instruction remain unchanged. Shared `cargo xc` passes, the dedicated and
neighboring structure targets pass `3/3` each, the exact CLI witness passes
`1/1`, and the same six pinned leaves pass all `12/12` sloppy/strict Wasm-AOT
executions with every failure bucket at zero. This source-equivalent capability
closure needs no new semantic golden and claims no new Math behavior.

The `Math.hypot` backend now likewise consumes the complete runtime argument
vector. A private non-copy completed-reduction witness makes the full
left-to-right `ToNumber` pass a prerequisite for result selection, while
Infinity and NaN observations never terminate the argument loop. Finite
nonzero values use a scaled sum of squares so representable large and tiny
inputs do not overflow or underflow merely because an intermediate was
squared. The focused
[hypot reduction contract](../docs/rust-rewrite/contracts/math-hypot-argument-reduction.md)
and registered CLI fixture cover contributions and observable coercions after
the old seven-argument cap, Infinity-over-NaN precedence, abrupt completion,
positive-zero output and large/tiny finite vectors. This batch has run only
static gates for that seam; the focused Wasm fixture, pinned `Math/hypot` tree
and broader Math gates remain deferred, and this is not a correctly-rounded
last-bit, current-pin baseline-delta, complete Math or T20 closure claim. The
`Math.sumPrecise` backend now consumes every input through the runtime sync
iterator protocol; lowering no longer materializes literal arrays, generators
or overridden array iterators into a compile-time answer. The closed
`SyncIteratorConsumer::MathSumPrecise` selection gives the Math algorithm its
distinct diagnostics. Shared primitive acquisition and protocol TypeErrors use
the current function Realm, with the zero-environment main Realm fallback. An
explicit count guard plus exact Number-tag check are the only algorithm-created
errors that close the iterator while preserving the original throw. Five
closed reduction states retain the specification's minus-zero, finite,
infinity and NaN behavior while still visiting later values. Finite terms are
added to one fixed signed 34-limb two's-complement accumulator: binary64 values
are integer multiples of `2^-1074`, their largest coefficient has 2098 bits,
and fewer than `2^53` terms need at most 2151 signed magnitude bits, below the
2176-bit buffer. The sole finisher converts to magnitude and rounds once to
nearest, ties to even. The focused
[runtime reduction contract](../docs/rust-rewrite/contracts/math-sum-precise-runtime.md),
bounded structure test and registered CLI fixture cover the runtime routes,
signed zero, cancellation, adversarial rounding, exceptional-state
continuation, close behavior and created-realm TypeErrors. This batch has run
only static gates for the seam; the fixture also witnesses created-realm
primitive iterator-prototype selection. The focused Wasm fixture, practical runtime
coverage of the `2^53 - 1` RangeError guard, pinned `Math/sumPrecise` tree and
broader Math gates remain deferred. It is not a generic iterator-realm or
generator-close closure, an own/created-realm Arguments iterator repair, a
current-HEAD ten-of-ten Test262 result, a throughput claim, complete Math or
T20 closure. The private `MathSumPreciseLimbOperation::{Add, Subtract}` domain
now carries the finite accumulator's arithmetic and carry polarity without
deriving any capabilities. Four borrowed exhaustive projections preserve the
two arithmetic stages and their corresponding unsigned carry/borrow tests;
the finite-term sign branch remains the exact two-producer authority. The
focused [limb-operation contract](../docs/rust-rewrite/contracts/math-sum-precise-limb-operation.md)
and recursive, bounded structure guard record this source-equivalent seam. The
new structure target passes `3/3`, the neighboring runtime structure target
passes `6/6`, and the existing Wasm-AOT runtime fixture passes `1/1`. The
independently reviewed guard pins the attribute-free declaration and global
arithmetic/carry projection order. Coordinated `cargo xc`, formatter, diff and
repository policy checks are green. The Test262 Math tree and broad verification
remain deferred. The
JSON reviver frame protocol now has a theory source of
truth at `docs/rust-rewrite/contracts/json-reviver-frame.md`. Its dynamic frame
stores closed typed states and an explicit nested-versus-root property role;
exhaustive emission gives every valid wire word a semantic arm and traps an
invalid word as an internal invariant violation. The static-specialized and
dynamic parser paths share the post-call deletion/replacement emitter, so an
ordinary empty-string property cannot be mistaken for the synthetic root. A
registered dynamic-input CLI fixture pins postorder traversal, array-length
and object-key snapshots, forward mutation, `context.source` SameValue
eligibility, nested empty-string replacement, root replacement/`undefined`,
and abrupt completion ordering. A four-test bounded structure owner now pins
the exact state/role domains and wire words, typed persistence and invalid-word
traps, the unique shared static/dynamic post-call result owner, and the active
CLI registration with non-vacuous fixture assertions. At the 2026-08-25
coordinated checkpoint, `cargo check -p lila-aot-wasm` and `cargo xc` are
green, the structure target passes `4/4`, and the four exact CLI fixtures pass
`4/4`. The six direct Test262 leaves pass all twelve ordinary sloppy/strict
Wasm-AOT executions at vendored suite content tree
`aa55200d1310384c5cf69ea95b2a2ecba457007b`, with every failure and
non-success bucket at zero. This is not a JSON closure or full-tree conformance
claim.

The task-owned Test262 source-rewrite backlog is now empty. Twenty-nine Number
and DataView `ToNumber` observations were removed after the original sources
passed directly: the complete selected Number inventory passes `94/94`
sloppy/strict executions across 47 files, and the selected DataView abrupt
conversion inventory passes `124/124` across 62 files, with every non-success
bucket at zero. Number intrinsic installation now derives all relevant method
names from the existing builtin catalogs instead of duplicating string names;
the Wasm golden capture is unchanged. The repository-wide shortcut inventory is
now 404 exact observations, 256 semantic observations, and zero T20 rows. The
larger token-aware census is independent of the completed T20 removal. Full
Number, BigInt, Math and JSON tree closure remains outstanding.

The 13-file `built-ins/BigInt/prototype/toString` leaf now passes all `26/26`
sloppy/strict Wasm-AOT executions, up from `24/26`. Its only red source never
reached BigInt receiver validation: an earlier helper call invalidated the
configurable-global presence proof and common identifier lowering incorrectly
constant-folded `typeof BigInt` to `"undefined"`. T08 now retains a run-time
global-property read whenever presence is unknown; the independent
`built-ins/BigInt/asIntN/bigint-tobigint-errors.js` control passes `2/2`. This
is shared reference-resolution progress, not complete BigInt or T20 closure.

Strict equality, SameValue and SameValueZero now share one mixed-representation
BigInt comparison boundary. Inline and heap-backed values first prove that both
tags denote the ECMAScript BigInt type, then compare their mathematical value;
different non-BigInt tags remain unequal. A bounded structure target pins all
three equality algorithms to that boundary and passes `2/2`. A registered Wasm
fixture constructs heap-backed `-1` through a multi-limb bitwise operation and
passes strict equality, `Object.is` and `Array.prototype.includes` against
inline `-1n`; its focused CLI target passes `1/1`.
`cargo xc` and every repository policy gate pass. The semantic Wasm golden has
648 artifacts: the new fixture is the sole addition; all 645 prior fixture
dumps change only emitted-size summaries, with no import, export, runtime-root,
helper-count, memory or data-segment contract change. All prior modules gain
the shared equality path; 553 record the larger `Object.defineProperty` body
and 446 also record a larger main function.

Unary plus and unary minus now have distinct IR states. Unary plus remains a
Number-only `ToNumber` operation, while unary minus records the closed normal
Number-or-BigInt result domain and reaches one backend `ToNumeric` dispatch.
That dispatch preserves Number `f64.neg` semantics and routes BigInts through
the existing exact `Negate` operation, including captured bindings whose
runtime representation is not statically fixed. The bounded structure target
passes `3/3`, and the complete BigInt bitwise CLI fixture now passes `1/1` after
also replacing six Test262-host-only assertion calls with a local product-path
exception check. `cargo xc` and every repository policy gate pass. The semantic
Wasm golden remains at 648 artifacts with no additions or removals: 126 dump
summaries change, 125 only in emitted function/total-size attribution and the
BigInt bitwise fixture in its expected main-local, internal-function,
name-section and size summaries. Imports, exports, runtime roots, helper counts,
memory and data segments remain unchanged.

The isolated runtime owner now emits the complete Ryū binary64-to-shortest-
decimal algorithm with explicit unsigned 64x64-to-128 arithmetic, compact
power-of-five tables, interval shortening and even-bound handling. Its closed
power-table domain traps if the mathematical index proof is violated, and a
full-domain digest test pins all 618 reachable generated entries to ryu-js
1.0.2. ECMAScript's exhaustive fixed/scientific spelling projection follows
the shortest decimal; the old six-digit fractional path, unsafe-integral
special search and `1e19` placeholder are gone. Static Number folding and
numeric property-key materialization use the same pinned `ryu-js = "=1.0.2"`
semantic authority instead of Rust Display. The focused 4-case structure
target, full-domain table test and dynamic CLI fixture pass; the fixture covers
`0.30000000000000004`, `193744829919998.375` (`...998.38`), the `1e19`/`1e20`/
`1e21` notation boundary, minimum subnormal and normal values, adjacent powers,
and `-0`. The exact current-pin `Number.prototype.toString` and `toFixed`
leaves pass all `180/180` and `32/32` sloppy/strict Wasm-AOT executions,
respectively. The shared semantic golden contains 656 fixture dumps: four new
Realm fixtures and no removals. All 652 retained dumps preserve their imports,
exports, runtime roots, helper counts, memory, data segments and name counts;
547 carry only the Ryū code-size delta, while the remaining size deltas are the
expected Atomics/DataView Realm bodies or their combinations. The deliberately
expanded Number fixture additionally changes its main local count and
largest-function attribution.

The exponentiation `order-of-evaluation.js` and `bigint-toprimitive.js` cases
now execute their unchanged pinned sources with the full Test262 assertion
harness for 4/4 sloppy/strict variants. Their previous shared harness reduction
is gone; the ordinary assertion prelude now preserves the coercion-order,
abrupt-completion, BigInt `ToPrimitive`, and SameValue checks directly.

The Number constructor, four static predicates and six prototype methods now
enter an exact `NumberBuiltin` domain that derives no equality capability. Its
six branded prototype operations project into the narrower
`NumberPrototypeOperation` domain, which likewise initially derived only
`Clone` and `Copy`. The existing eleven-arm and six-arm matches remain
exhaustive, so a future operation must define both its top-level route and, when
applicable, its complete prototype result algorithm instead of inheriting
behavior through an equality/default projection. The bounded contract is
recorded in `docs/rust-rewrite/contracts/number-builtin-policy-domains.md`; its
structure target passes `4/4`. The coordinated 679-dump semantic golden passes
`2/2` in 800.46 seconds with no retained structural change attributable to that
policy closure.

Batch AI removes those remaining `Clone` and `Copy` capabilities from both
dispatch domains. Every top-level selection and every restricted prototype
selection is now owned by its sole exhaustive consumer, so neither policy can
be silently retained or forked across the dispatch boundary. The eleven and six
producers, owned parameters, exhaustive matches and emitted instructions remain
unchanged. Shared `cargo xc` passes, the structure target passes `4/4`, and the
aggregate Number runtime witness passes `1/1`. This source-equivalent closure
needs no Test262 cohort or semantic golden and claims no new Number behavior.
Final formatter, diff, module-boundary, task-plan and 240-entry
shortcut-inventory gates are green.

Batch AT makes the outer family a private `NumberBuiltin` and exposes only
eleven fixed Number entries to standard dispatch. The frozen 160-line
domain/emitter selection has SHA-256
`7465f52181186c7cd1dd4bb2be3fa2a124ac6794fe509c4d7a0e003984091e9a`;
restoring the former enum/emitter visibility and constructor worker name
reproduces that source exactly. At the 2026-08-28 Batch AT checkpoint,
`cargo xc` is green, the strengthened structure target passes `4/4`, and the aggregate
Number runtime witness passes `1/1`. No Test262 leaf or semantic golden was
required for this dispatcher-only closure. This source-equivalent boundary
claims no new Number behavior, broader conformance or published
conformance-count change.

Batch AJ makes the four-member JSON namespace dispatch a capability-free
`JsonBuiltin`. Each `parse`, `stringify`, `rawJSON` and `isRawJSON` selection is
constructed once by the standard dispatcher and moved into the sole exhaustive
JSON emitter match. It cannot be cloned, copied, formatted, defaulted, compared,
ordered or hashed, so one namespace selection cannot silently fork into a
second policy decision. The bounded contract is recorded in
`docs/rust-rewrite/contracts/json-builtin-policy-domain.md`. This
source-equivalent ownership closure changes no emitted instruction and claims
no new JSON behavior. Shared `cargo xc` passes, the structure target passes
`4/4`, and exact JSON parse and stringify CLI witnesses pass `2/2`. No Test262
cohort or semantic golden was needed or run.

Batch AO confines that raw `JsonBuiltin` policy and its exhaustive compiler to
`builtins/json.rs`. The standard dispatcher no longer imports, constructs or
passes the domain and cannot call the raw compiler; it sees only four private
fixed semantic wrappers for parse, stringify, rawJSON and isRawJSON. The former
ten-line raw selection has SHA-256
`c3276e86866cc00345ee4ad017e710465e8b9a4d9973ba045a7f576ffe7beee0`,
while the four-line wrapper-only selection has SHA-256
`72b2e14d442b15efe1a15d2d6fc3755e96b49bf7c9d0c75bb8da719efb0dcf8d`.
The strengthened four-test structure owner and module boundary pin the private
ten-mention census, four one-call producers, four standard routes and sole
owned exhaustive consumer. This source-equivalent boundary changes no emitted
instruction and claims no new JSON behavior. Shared Cargo and exact parse and
stringify controls pass with `cargo xc` green, the strengthened structure
target `4/4`, and the exact CLI witnesses `2/2`. No Test262 leaf or semantic
golden was required or run.

Batch AK replaces the flat 37-member Math policy with a capability-free
`MathBuiltin` whose `Unary(MathUnaryBuiltin)` variant carries the exact 29
one-argument operations while eight non-unary algorithms remain top-level.
The capability-free `MathUnaryBuiltin` is consumed only after the shared
argument-zero coercion. The old inner branches for eight impossible non-unary
operations are deleted, so an invalid unary route is unrepresentable instead
of guarded by a runtime `unreachable!`. All 37 standard producers retain their
exact named mapping, and neither domain can be cloned, copied, formatted,
defaulted, compared, ordered or hashed. The bounded contract is recorded in
`docs/rust-rewrite/contracts/math-builtin-policy-domains.md`. This
source-equivalent type split changes no emitted instruction and claims no new
Math behavior. Batch AK shared `cargo xc` is green, the structure target passes
`4/4`, and the extremum, `hypot` and `sumPrecise` CLI controls pass `3/3`. The
exact `Math.abs` and `Math.round` Test262 leaves pass all `4/4` Wasm-AOT
variants with every failure bucket at zero. No semantic golden was required or
run. Final formatter, diff, module-boundary, task-plan and 240-entry
shortcut-inventory gates are green.

Batch AW makes both capability-free Math domains and their raw exhaustive
emitter private to `builtins/math.rs`. Standard dispatch can reach the family
only through 37 fixed Math entries, one per namespace operation. The frozen
825-line domain/emitter selection has SHA-256
`25cedc56bf9f821608dad8f2c4b3d6b079a09279bbc5ca6e0703679d16e98049`;
restoring only the former enum and emitter visibility reproduces that source
exactly. `cargo xc` passes. The policy, extremum, `hypot`, `sumPrecise` limb and
`sumPrecise` runtime structure targets pass `4/4`, `3/3`, `3/3`, `3/3` and
`6/6`; the three established Math Wasm-AOT CLI controls pass `3/3`. No Test262
leaf or Wasm golden was required for this source-equivalent dispatcher boundary,
which claims no new Math behavior, conformance result or published-count change.

Batch AL makes the private, capability-free `MathSumPreciseState` the sole
five-state identity. Its exact `MinusZero`, `Finite`, `PlusInfinity`,
`MinusInfinity` and `NotANumber` domain can now only be consumed by the
exhaustive `0..=4` ABI-word
projection; clone, copy, formatting, default, comparison, ordering and hashing
cannot create a second identity route. The reducer, wire values and all emitted
instructions remain unchanged, so this source-equivalent hardening claims no
new Math behavior. The strengthened six-test runtime structure owner and
[Math.sumPrecise runtime contract](../docs/rust-rewrite/contracts/math-sum-precise-runtime.md)
pin the exact domain, capability absence and exhaustive projection. Shared
`cargo xc` is green, the structure target passes `6/6`, the exact runtime CLI
fixture passes `1/1`, and the two focused Test262 leaves pass all `4/4`
Wasm-AOT variants with every failure bucket at zero. No semantic golden was
required or run.
Final formatter, diff, module-boundary, task-plan and 240-entry
shortcut-inventory gates are green.

Batch AM makes `JsonReviverFrameState`, `JsonReviverPropertyRole` and
`JsonParseFrameState` capability-free JSON wire domains. Their shared macro no
longer derives clone, copy, debug or equality, its stable wire projection
borrows the identity, and each complete-set traversal borrows rather than
copies the macro-owned values. The exact four-state, two-role and eight-state
domains, consecutive word proof, persisted values, exhaustive dispatch and
emitted instructions remain unchanged. The strengthened reviver and
parse-frame structure owners, both bounded contracts and the exact dynamic
reviver CLI witness own the focused regression. The compiler-enforced borrow
also makes the source-present, source-ineligible and source-absent static
branches share one role authority. At the Batch AM checkpoint, `cargo xc` is
green, the reviver and parse-frame structure targets pass `5/5` and `4/4`, and
the exact dynamic-reviver CLI witness passes `1/1`. No focused Test262 leaf or
semantic golden was required or run for this source-equivalent capability
hardening.
Final formatter, diff, module-boundary, task-plan and 240-entry
shortcut-inventory gates are green.

Batch AN makes the private static-reviver string-or-array-index role a
capability-free `JsonStaticPropertyKey`. Its three producers immediately lend
the temporary identity, and the recursive internalizer borrows the same key
through materialization, holder lookup and final reviver-result application.
Clone, copy, formatting, default, comparison, ordering and hashing can no
longer create a second identity route. Key payloads, Array index words, lookup
order, reviver calls and emitted instructions remain unchanged. The
strengthened five-test reviver structure owner and the exact
forward-modification CLI witness own the focused regression. At the Batch AN
checkpoint, `cargo xc` is green, the structure owner passes `5/5`, and the
exact CLI witness passes `1/1`. No focused Test262 leaf or semantic golden was
required or run for this source-equivalent capability hardening. Final
formatter, diff, module-boundary, task-plan and 240-entry shortcut-inventory
gates are green.

Dynamic `toFixed`, `toExponential` and `toPrecision` formatting now enters one
private `NumberDecimalFormat` domain and one exhaustive shared decimal core.
The three wrappers retain their distinct coercion, range and omitted-argument
rules, while the data-bearing variants make each rounding/placement choice
explicit. The old empty-string sentinel, precision value table and magic
integer answer are gone. The dedicated parameterized fixture prevents literal
folding from masking the runtime path and covers rounding, carry, placement,
sign, zero, notation thresholds and representative binary64 edges across all
three methods. Long-digit cases reject shortest-decimal reuse for supplied
precision, and a lower-threshold carry pins notation selection after rounding.
The older aggregate Number-family fixture remains a regression.
Non-finite spellings remain covered independently of the finite rounding core.
The focused ownership and witness are recorded in
`docs/rust-rewrite/contracts/number-decimal-formatting.md`. The three related
structure targets pass `12/12`, the dynamic and existing CLI regressions pass
`3/3`, and the exact fixed/exponential/precision leaves pass all `6/6`
sloppy/strict Wasm-AOT executions. The 680-dump semantic golden passes `2/2` in
672.44 seconds, adds only the dynamic decimal fixture, removes none and leaves
all 679 retained dumps structurally equal after accounting normalization; the
shared formatter accounts for the measured code-size delta without changing
function or local topology. This repair does not claim the ECMA-402 locale
surface or the full pinned Number tree.

`BigInt.asIntN` and `BigInt.asUintN` now enter their shared truncation body
through the closed non-copyable
`BigIntFixedWidthOperation::{Signed, Unsigned}` domain. The two standard
builtin rows construct the named operation explicitly, and four borrowed
exhaustive matches own sub-64 signed interpretation, the 64-bit unsigned heap
boundary, wide unsigned passthrough eligibility and wide signed high-bit
interpretation. The former broad-builtin equality policy is absent. The
focused contract and existing arbitrary-width fixture are recorded in
[`bigint-fixed-width-operation.md`](../docs/rust-rewrite/contracts/bigint-fixed-width-operation.md).
The bounded structure target passes `5/5`, the existing arbitrary-width CLI
regression passes `1/1`, and the exact `arithmetic.js` and `order-of-steps.js`
leaves under both operations pass all `8/8` sloppy/strict Wasm-AOT executions
with every failure bucket at zero. The coordinated `cargo xc`, rustfmt and diff
checks are green. This is a source-equivalent invariant checkpoint; no full
BigInt or T20 closure is claimed.

Batch AX makes the builtin, fixed-width, prototype-result and three
result-authority domains, their associated prototype producers and the raw
exhaustive emitter private to `builtins/bigint.rs`. Standard dispatch reaches
them only through six fixed BigInt entries. The frozen 736-line domain/emitter
selection has SHA-256
`5b61c6cfedaf3b988517eab492bb6c3dedb85a5eb9ac98992120ff39e7f30f18`;
restoring only the former visibilities reproduces that source exactly. Batch AX
`cargo xc` passes. The fixed-width, heap-slot, helper-operation, number-policy
and prototype-result structure targets pass `5/5`, `4/4`, `4/4`, `2/2` and
`4/4`. The constructor, arbitrary-width and wrapper-coercion Wasm-AOT CLI
controls pass `3/3`; a direct stdin control exercises `toString`,
`toLocaleString` and `valueOf` together and returns `number(3)`. The broader
prototype fixture remains red at its unrelated captured-main-lexical Symbol
Realm assertion. No Test262 leaf or Wasm golden was required for this
source-equivalent boundary, which claims no new BigInt behavior, conformance
result or published-count change.

The shared arbitrary-precision helper now serializes its fourteen legal
`BigIntHelperOp` rows only through one borrowed exhaustive `runtime_code`
authority. The operation derives no incidental capability, the former 21 raw
casts are absent, and all nine semantic producer routes retain their existing
mapping and instruction order. The focused
[runtime-code contract](../docs/rust-rewrite/contracts/bigint-helper-operation-runtime-code.md)
records the lexical recursive census, exact three tables, nine complete
producer calls, all nineteen direct helper-body decision sites, two dynamic
helper-call sequences and the unchanged numeric words 0 through 13. The generated
helper still consumes a runtime `i64` through its existing XOR/remainder
fallthrough tree, so this is a serialization invariant rather than an
arbitrary-word decoder or full T20 closure. The structure target passes `4/4`,
the focused BigInt bitwise, exponentiation and mixed Number relational CLI
witnesses pass `3/3`, and six selected arithmetic, unary-minus and BigInt
relational Test262 leaves pass all `12/12` sloppy/strict Wasm-AOT executions
with every failure bucket at zero. Scoped rustfmt and diff checks are green; no
broad suite or golden is claimed. Independent dry re-review is clean after the
producer and serializer guards were hardened. The following shared workspace
compile, formatter, module-boundary, task-plan and diff gates all pass.

The three BigInt prototype outcomes now carry a move-only result authority
from their exact builtin producer through the sole exhaustive emitter match.
The policy and marker types no longer derive duplication capabilities, so the
radix marker can enter its preparation stage only once and radix coercion
cannot be duplicated by copying the authority. The Rust-lexical recursive
`bigint_prototype_result_ownership_structure` guard pins the attribute-free
four-type domain, exact producers, global mention census and one-way typed
handoffs. This source-equivalent ownership closure does not change the current
locale fallback or claim wider BigInt/T20 conformance. Its structure guard
passes `4/4`, the neighboring fixed-width guard passes `5/5`, and the exact
producer unit witness passes `1/1`. The existing product fixture currently
fails `0/1` on the shared conversion/Realm path with `main lexical Symbol
ToNumeric realm fallback`; this derive-only seam does not claim that unrelated
failure as green.

The complete prepared-radix carrier lifecycle now has one private
`builtins/bigint/radix_formatting.rs` owner. The raw carrier, its constructor
and projections, sole preparation stage, two representation formatter reads
and consuming release moved together; the parent retains only its unchanged
semantic radix-result call. The 94-line child has SHA-256
`3bf6fb2fa973a4c21c00c3023dff857b2e29e8e425ffcbb7910d87145ee8abe9`
and reduces the concurrent BigInt parent from 896 to 807 lines. The exact
13/28/47-line selections retain visibility-normalized SHA-256
`6a204fe4279fe2887901a4eeac6179c52a13d3fe91e269c66c158c4e33cb1855`,
`0b8c98f44841961d75dc81143ca96ebd46c285b635a2fad8cc3f59d6f211b330`
and
`b8c715a6186a586a9ff0ecf0e967aeb9bfd5a96e626d9f566cd6c8763eb8f4f1`.
The unchanged radix policy arm retains SHA-256
`e42b93bc786cda7b3733d24f6d03b24500fc8d189e327ca17bd04a100e55ff7f`.
This is source-equivalent ownership hardening. The structure target passes
`4/4`, the shared `cargo xc` gate is green, and the unchanged pinned
`radix-2-to-36.js` leaf passes both ordinary Wasm-AOT variants `2/2`. The
broader combined product fixture still fails before the radix assertions on
its unrelated main-lexical Symbol/Realm control; broad semantic verification
remains deferred.

The six JSON.stringify replacer producers now construct one move-only
JSON.stringify replacer invocation authority. Four distinct tagged roles make
the replacer function, exact callback receiver, property key, and mutable
value/result carrier inseparable at the sole emitter, so a producer cannot
transpose replacer, receiver, property key, and value roles through the former
eight positional locals. The recursive Rust-lexical
`json_stringify_replacer_invocation_authority_structure` guard pins the private
role domain, complete producer mappings, sole consumer, call order, active CLI
registration, and public root/object/array/abrupt fixture. This is
source-equivalent hardening and does not claim wider JSON or T20 conformance;
the authority and neighboring reviver structure targets pass `5/5` each, and
the exact public Wasm-AOT fixture passes `1/1`. Focused details are recorded in
the contract.

The iterative dynamic JSON parser's eight container-frame states now come only
from the closed `JsonParseFrameState` wire domain. A private move-only validated
local is the sole persistence and comparison authority: root and nested frame
writes validate against the complete ordered state set, every persisted load
immediately repeats that admission, and all eight comparisons borrow the
authority with a typed expected variant. Frame creation consumes the proof,
every transition uses the sole typed state writer, and the loaded proof is
consumed at final temporary-local release. An unknown internal word traps
before dispatch instead of becoming a user-facing JSON `SyntaxError`. The bounded
[frame-state contract](../docs/rust-rewrite/contracts/json-parse-frame-state.md)
and structure witness record this source-equivalent seam. The standalone new
and neighboring structure targets pass `4/4` and `5/5`, and the exact-file
formatter check is green. This slice makes no JSON grammar, product-path, broad
Test262 or complete T20 claim; the coordinated batch owns compilation and
semantic verification.

Batch AH gives that validated parse-frame local lifecycle one private
`builtins/json/parse_frame_state.rs` owner. Its move-only carrier, private raw
projections, sole validator, borrowed comparator, consuming frame push and
consuming release moved together, leaving the parent with inferred call sites
and zero carrier names, imports or re-exports. The 171-line selection is
source-equivalent after normalizing required `pub(super)` visibility; the
recursive guard pins seven child-only carrier identifiers, two projections and
the exact `5/9/4/2` lifecycle census. The frame-state and unchanged reviver
structure targets pass `4/4` and `5/5`, shared `cargo xc` passes, and the exact
dynamic-reviver CLI witness passes `1/1`. This ownership-only move needs no
Test262 cohort or semantic golden and makes no new JSON behavior claim.

## Objective

Complete ECMAScript numeric semantics, arbitrary-precision BigInt, the Math object and JSON parsing/stringification. Eliminate fixed-width BigInt approximations and folding paths that change observable conversion order or IEEE-754 edge behavior.

## Number semantics

Implement one authoritative conversion/formatting layer for:

- `ToNumber` from every primitive and object path, including Symbol/BigInt errors;
- decimal, binary, octal and hexadecimal source/string parsing, whitespace and signed Infinity;
- exact IEEE-754 handling for NaN, infinities, subnormals and signed zero;
- `Number` call/construct behavior and boxed-number branding;
- constants and predicates: `EPSILON`, safe-integer bounds, `isFinite`, `isInteger`, `isNaN`, `isSafeInteger`;
- `parseInt`/`parseFloat` static aliases and global variants;
- prototype formatting: `toString(radix)`, `toExponential`, `toFixed`, `toPrecision`, `toLocaleString`, `valueOf`;
- shortest-roundtrip and correctly rounded conversion behavior required by Test262.

Do not rely on Rust debug/display formatting where ECMAScript output differs. Preserve `-0` at every observable boundary.

## Numeric operators

Complete Number arithmetic, bitwise/shift, comparison, exponentiation, remainder and update/compound-assignment behavior with exact coercion/evaluation order. Cover all special-case tables for signed zero, infinities and NaN. Compile-time constant folding must use the same semantic helpers and must not suppress side effects or required errors.

## BigInt

Use arbitrary-precision signed integers for all BigInt values. Implement:

- literal/string parsing and `BigInt` conversion rules;
- arithmetic, exponentiation, bitwise operators, shifts, comparisons and unary operations;
- mixed Number/BigInt TypeErrors at the required point;
- division/remainder truncation, negative shifts and invalid exponent behavior;
- `BigInt.asIntN`, `BigInt.asUintN`, prototype `toString`, `toLocaleString`, `valueOf` and descriptors;
- boxed BigInts, object coercion and cross-realm branding;
- integration with BigInt typed arrays, DataView, Atomics and JSON error behavior.

Add resource limits for adversarial huge operands, but report exhaustion explicitly rather than returning truncated values.

## Math

Implement every method/constant in the pinned suite, including exact corner cases and metadata. Cover trigonometric, logarithmic, exponential, rounding, integer, clamping, `hypot`, `imul`, `clz32`, `fround`, `f16round`, random and current additions such as `sumPrecise` when present in the pin.

- Match ECMAScript coercion order for variadic methods.
- Preserve required NaN/signed-zero/infinity behavior.
- Define an injectable randomness source for deterministic tests without making production `Math.random` constant.
- Use portable Rust algorithms or well-defined host functions; add conformance tests around platform-sensitive boundaries.

## JSON

Implement a dedicated JSON parser and serializer over Lila values:

- strict JSON lexical grammar, strings/escapes, numbers and duplicate keys;
- `JSON.parse` reviver traversal/deletion/order, `__proto__` treatment and source-text context argument if present in the pin;
- `JSON.stringify` `toJSON`, replacer function/array, property-list construction, gap/space, cycle errors, property order, boxed primitives, BigInt failure and abrupt completions;
- omission/null substitution rules for objects vs arrays;
- `JSON.rawJSON`/`JSON.isRawJSON` or other current additions when present;
- deep-input handling without Rust stack overflow.

Property access and calls must route through T04/T10 so getters/proxies and mutation are visible in spec order.

## Acceptance criteria

- Full pinned Number, BigInt, Math and JSON Test262 trees are green.
- BigInt supports values far beyond 64 bits in operators, formatting and typed-data integrations.
- Numeric string conversion and formatting pass exhaustive boundary/vector tests.
- Signed zero and NaN behavior is preserved through operators, Math, JSON and typed arrays where observable.
- JSON reviver/replacer/proxy/evaluation-order tests pass without static materialization.
- Platform-dependent Math functions have documented tolerances matching Test262 and produce no target-specific regressions.
- No numeric operation silently saturates/truncates outside the specification.

## Required tests

```sh
cargo test -p lila-ir numeric_ --quiet
cargo test -p lila-aot-wasm numeric_ --quiet
cargo test -p lila-cli wasm_number --quiet
cargo test -p lila-cli wasm_bigint --quiet
cargo test -p lila-cli wasm_json --quiet
./target/debug/lila test262 run built-ins/Number --execution-backend wasm --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/BigInt --execution-backend wasm --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Math --execution-backend wasm --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/JSON --execution-backend wasm --timeout-ms 120000 --threads 4
```

Re-run numeric expression/operator tests and binary-data filters that consume Number/BigInt conversions.
