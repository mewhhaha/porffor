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
conversion/builtin integrations are still open. The BigInt prototype result
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
import, injected provider, and BigInt bitwise CLI fixture. The full Number,
BigInt, Math and JSON current-pin trees have not met this task's shortcut-free
acceptance gate. The JSON reviver frame protocol now has a theory source of
truth at `docs/rust-rewrite/contracts/json-reviver-frame.md`. Its dynamic frame
stores closed typed states and an explicit nested-versus-root property role;
exhaustive emission gives every valid wire word a semantic arm and traps an
invalid word as an internal invariant violation. The static-specialized and
dynamic parser paths share the post-call deletion/replacement emitter, so an
ordinary empty-string property cannot be mistaken for the synthetic root. A
registered dynamic-input CLI fixture pins postorder traversal, array-length
and object-key snapshots, forward mutation, `context.source` SameValue
eligibility, nested empty-string replacement, root replacement/`undefined`,
and abrupt completion ordering. This batch has only run static gates for that
seam while the repository-wide conformance matrix owns the verifier; its
focused Wasm fixture and broader JSON/Test262 gates remain deferred, and this
is not a JSON closure or full-tree conformance claim.

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
