# Contract: Numeric conversion codomains — `ToIntegerOrInfinity` / `ToUint32` / `ToUint16` as closed types in `porffor-ir`, with one const reference algorithm for the five divergent backend hand-rolls

Area owner: FORMALIZER, theory-first campaign, round 2.
Implements: ECMA-262 5.2.5 (`modulo`), 7.1.5, 7.1.6, 7.1.7, 7.1.9, 7.1.20, 7.1.22,
21.1.3.2, 21.1.3.3, 21.1.3.5, 21.3.2.11, 22.1.2.1, 22.1.3.23.
Short-name pointer: `docs/rust-rewrite/contracts/numeric-conversion-codomains.md`.

---

## 0. How to read this, and what is measured

This document is the encoder's specification and the dry-runner's oracle. Every
line number is from `git rev-parse --short HEAD` = `091487732` on branch
`claude/test-driven-rust-opus-pp6giw`, and every one was opened and read, not
inferred from a grep count.

Three conventions, following the house style of
`Reference Records: one record, a carried [[Strict]], and a write that consumes it.md`:

- **Invariants** are numbered `I1..I12`. §2 assigns each one either a Rust
  construct or a ledger row. There is no third option.
- **Ledger rows** are numbered `LN1..LN5`. A ledger row is a place where a test
  or a run-time check remains load-bearing, *with the reason a type cannot carry
  it*. A row without a reason is a defect in this document.
- **Measured** means counted from the tree at `091487732`. **Derived** means
  computed from IEEE-754 or from the spec text. **Estimated** appears nowhere in
  this document.

§5 lists five places where this contract **departs from the area brief**, each
with the evidence. Read §5 before implementing anything: three of the five would
otherwise produce either a wrong retrofit or a decoration type.

### 0.1 What was measured

| Fact | Value | How |
|---|---|---|
| `pub fn spec_*` constructors in `ir.rs` | **31** | `grep -c "^    pub fn spec_" crates/porffor-ir/src/ir.rs` |
| References to `spec_to_integer_or_infinity` outside its definition | **2**, both under `#[cfg(test)]` | `ir.rs:3856` (module gate at `ir.rs:3451`), `porffor-aot-wasm/src/lib.rs:1468` (module gate at `lib.rs:109`) |
| References to `spec_to_length` outside its definition | **2**, both under `#[cfg(test)]` | `ir.rs:3878`, `porffor-aot-wasm/src/lib.rs:1486` |
| References to `spec_to_index` outside its definition | **2**, both under `#[cfg(test)]` | `ir.rs:3900`, `porffor-aot-wasm/src/lib.rs:1503` |
| Modular-residue sites in `porffor-ir` | **2** | `rem_euclid` at `lowering.rs:24662` and `lowering.rs:36471`; there is no third |
| Float→integer truncating casts on a conversion path in `porffor-ir` | **1** | `value.trunc() as i32` at `lowering.rs:34801`; `grep -n "trunc() as i32"` returns exactly this line |
| Distinct hand-rolls of the 7.1.5-step-2 / 7.1.6-step-2 case split in `porffor-aot-wasm` | **5**, in **5 different spellings** | §1.4 |
| CLI fixtures mentioning `toPrecision`, `toFixed`, `toExponential`, `clz32` | **0, 0, 0, 0** of 532 | `grep -rl` in `crates/porffor-cli/tests/fixtures` |
| CLI fixtures mentioning `fromCharCode` | **5** of 532, none reaching the static-generator fold | ibid. |
| test262 corpus files in §6 that exist at the current pin | **9 of 9** | checked by path |

### 0.2 The one-sentence claim

7.1.5, 7.1.6, 7.1.7, 7.1.9, 7.1.20 and 7.1.22 are **total functions into named
closed codomains**, and every defect this area covers is a codomain error: a
result stored in a type that cannot hold the spec's range (`Option<i32>` for
ℤ ∪ {±∞}), or a result computed by an operation that is not the spec's
(`I64TruncSatF64S` for `truncate`, machine remainder for 5.2.5 `modulo`).

---

## 1. Spec basis

### 1.1 `modulo` is 5.2.5's modulo, not a machine remainder

5.2.5 (Mathematical Operations) defines the notation used by 7.1.6 step 4,
7.1.7 step 4 and 7.1.9 step 4:

> The notation "x modulo y" (y must be finite and non-zero) computes a value k
> of the same sign as y (or zero) such that abs(k) < abs(y) and x - k = q × y
> for some integer q.

Three consequences, all load-bearing:

1. **The modulus is always positive here** (2^32, 2^16), so the residue is
   always in `[0, modulus)`. It is *never* negative. Rust's `%` and Wasm's
   `I64RemS` both produce a result with the sign of the *dividend*, so they are
   not this operation; they are this operation only after a sign correction.
2. **`x` ranges over all mathematical integers**, including those far outside
   `i64`. `truncate(ℝ(1e300))` is an integer with 997 binary digits. Any
   implementation that funnels `x` through a 64-bit integer before the modulo
   has already lost, unless it can prove the funnel is exact.
3. **`modulo` is exact.** It is defined on ℝ, not on a float format. An
   implementation is correct only if it produces the exact residue, not a
   nearby one.

The spec's `truncate` (5.2.5) is likewise exact: `truncate(x)` is the integer
part of `x`, toward zero. For an IEEE-754 binary64 argument, `truncate` is
representable in binary64 and `f64::trunc` computes it exactly
(IEEE-754 §5.9 `roundToIntegralTowardZero`).

### 1.2 The six operations, as total functions with named codomains

Each row states the operation's codomain as a *set*, which is what a Rust type
must be able to hold. "After ToNumber" means the row describes steps 2 onward;
step 1 (`? ToNumber(argument)`) is a separate operation with its own abrupt
completion and is not this area's business.

| § | Operation | Domain (after ToNumber) | Codomain | Total? |
|---|---|---|---|---|
| 7.1.5 | ToIntegerOrInfinity | binary64 | **ℤ ∪ {+∞, −∞}** | yes |
| 7.1.6 | ToInt32 | binary64 | `[−2^31, 2^31)` ∩ ℤ | yes |
| 7.1.7 | ToUint32 | binary64 | `[0, 2^32)` ∩ ℤ | yes |
| 7.1.9 | ToUint16 | binary64 | `[0, 2^16)` ∩ ℤ | yes |
| 7.1.20 | ToLength | ℤ ∪ {±∞} | `[0, 2^53−1]` ∩ ℤ | yes |
| 7.1.22 | ToIndex | ℤ ∪ {±∞} | `[0, 2^53−1]` ∩ ℤ **or** RangeError | no — partial, and the partiality is the point |

The codomain of 7.1.5 is the one that matters most here, and it is the one the
tree gets wrong. It is **not** `i32`. It is **not** `i64`. It is **not** `f64`
either as a *set*, though every reachable value is f64-representable — see
§1.3. It is the extended integers.

#### 7.1.5 ToIntegerOrInfinity ( argument )

```
1. Let number be ? ToNumber(argument).
2. If number is one of NaN, +0𝔽, or -0𝔽, return 0.
3. If number is +∞𝔽, return +∞.
4. If number is -∞𝔽, return -∞.
5. Return truncate(ℝ(number)).
```

The case analysis is exhaustive over binary64 and the five arms are disjoint:
`{NaN, +0, −0}`, `{+∞}`, `{−∞}`, and the finite non-zero remainder. Note that
step 2 returns the *mathematical* 0, not `−0𝔽`: a `-0.0` leaking out of an
implementation of 7.1.5 is a codomain violation even though it compares equal
to `0.0`.

#### 7.1.6 ToInt32 / 7.1.7 ToUint32 / 7.1.9 ToUint16

```
1. Let number be ? ToNumber(argument).
2. If number is not finite or number is either +0𝔽 or -0𝔽, return +0𝔽.
3. Let int be truncate(ℝ(number)).
4. Let intNbit be int modulo 2^N.            (N = 32 for 7.1.6/7.1.7, 16 for 7.1.9)
5. — 7.1.6: If int32bit ≥ 2^31, return 𝔽(int32bit - 2^32); otherwise return 𝔽(int32bit).
   — 7.1.7: Return 𝔽(int32bit).
   — 7.1.9: Return 𝔽(int16bit).
```

**7.1.6 and 7.1.7 differ only in step 5.** Steps 2–4 are character-for-character
the same operation. This is the fact the contract exports as
`residue_pow2_i64`, and it is the fact that makes two disagreeing in-tree
ToUint32 implementations (§1.4) a structural problem rather than a coincidence.

Note that step 2 folds `±0` in with the non-finite cases even though step 3–4
would give the same answer for `±0`. The redundancy is harmless in the spec and
becomes actively misleading in an implementation: a `value == 0.0` guard reads
as load-bearing when only the `!is_finite()` half of the disjunction is
(verified for `static_clz32`, §4.3).

#### 7.1.20 ToLength ( argument )

```
1. Let len be ? ToIntegerOrInfinity(argument).
2. If len ≤ 0, return +0𝔽.
3. Return 𝔽(min(len, 2^53 - 1)).
```

Step 2's `≤` and step 3's `min` are comparisons on **ℤ ∪ {±∞}**, so an
implementation must be able to compare `−∞ ≤ 0` and `min(+∞, 2^53−1)`. This is
the first place the extended codomain is consumed rather than merely produced.

#### 7.1.22 ToIndex ( value )

```
1. Let integer be ? ToIntegerOrInfinity(value).
2. If integer is not in the inclusive interval from 0 to 2^53 - 1,
   throw a RangeError exception.
3. Return integer.
```

(The pre-ES2023 spelling routed through ToLength and SameValueZero. It is
equivalent and this contract uses the current text.) Again the interval test is
on the extended integers: `+∞` is not in `[0, 2^53−1]`, and an implementation
that has already collapsed `+∞` to `0` cannot tell that.

### 1.3 Why binary64 is an exact carrier for the finite part of 7.1.5

**Theorem A.** For every finite binary64 `x`, `truncate(ℝ(x))` is exactly
representable in binary64, and `f64::trunc(x)` computes it.

*Proof.* If `|x| ≥ 2^52` then `x` is already an integer (its ulp is ≥ 1) and
`truncate(ℝ(x)) = ℝ(x)`. If `|x| < 2^52` then `|truncate(ℝ(x))| < 2^52 < 2^53`,
and every integer of magnitude below 2^53 is representable. `f64::trunc` is
IEEE-754 `roundToIntegralTowardZero`, which is exact by definition. ∎

**Theorem B (the exactness of the `rem_euclid` residue).** For every finite
binary64 `n` that is an integer, and for `M = 2^k` with `k ≤ 52`,
`n.rem_euclid(M as f64)` is exactly `n modulo 2^k` in the sense of 5.2.5, and
the result is exactly representable.

*Proof.* (i) Rust's `f64::%` is IEEE-754 `fmod`, defined as `n − q·M` for the
integer `q = truncate(n/M)`, and IEEE-754 §5.3.1 specifies it as **exact**: the
result is computed without rounding error for all finite operands with `M ≠ 0`.
Call it `r`; then `|r| < M` and `r ≡ n (mod M)`. (ii) `rem_euclid` returns `r`
if `r ≥ 0`, else `r + M`. Both `r` and `r + M` are integers of magnitude
`< M = 2^k ≤ 2^52 < 2^53`, hence exactly representable, and the single addition
of two exactly-representable values whose exact sum is representable is itself
exact. (iii) The result lies in `[0, M)` and is congruent to `n`, which is
5.2.5's `n modulo M` because `M > 0`. ∎

**Corollary.** `x.trunc().rem_euclid(4294967296.0) as u32` is exactly 7.1.7
steps 3–4 for every finite `x`, and `x.trunc().rem_euclid(65536.0) as u16` is
exactly 7.1.9 steps 3–4. The `as` casts are exact because the operand is
already an exact integer inside the destination range, so Rust's saturating
float→int cast never saturates.

This is why `lowering.rs:36471` and `lowering.rs:24662` are **correct today**
and why the backend hand-rolls that reach for `I64TruncSatF64S` are not. The
theorem replaces an empirical test; §7 records that no test is owed for it.

**Theorem C (the i64 window).** For every binary64 integer `n` with
`−2^63 ≤ n < 2^63`, `n as i64` is exact, and
`((n as i64) as u64) & (2^k − 1)` equals `n modulo 2^k` for `k < 64`.

*Proof.* Exactness of the cast is the range hypothesis. Two's complement is by
definition the residue system modulo 2^64, so `(n as i64) as u64 ≡ n (mod 2^64)`;
masking to the low `k` bits reduces modulo `2^k`, and `2^k | 2^64`. The masked
value is in `[0, 2^k)`, which is 5.2.5's residue. ∎

Theorem C is what makes the reference algorithm expressible as a `const fn` over
integers with no floating-point operations at all, which is why the const tables
in §2.7 can be evaluated at build time on stable Rust without depending on the
const-ness of `f64::trunc` or `f64::rem_euclid` (see ledger **LN1**).

### 1.4 The shared case split, and its five spellings in this tree

Every one of the five backend emitters begins by re-deriving 7.1.5 step 2 /
7.1.6 step 2 inline. All five were read. **No two spell it the same way:**

| # | Function | File:line | Spelling of the case split | Verdict |
|---|---|---|---|---|
| 1 | `compile_bitwise_number_payload` | `porffor-aot-wasm/src/expressions.rs:1684` | **none** — `F64ReinterpretI64` → `I64TruncSatF64S` → `I32WrapI64`, no guard at all | **defective**, §1.5 |
| 2 | `emit_to_uint32_i64_from_number_payload` | `porffor-aot-wasm/src/builtins/string.rs:15052` | four `F64` compares: `x ≠ x`, `x = 0`, `x = +∞`, `x = −∞` | **defective**, §1.6 |
| 3 | `emit_array_to_uint32_i64_from_number_payload` | `porffor-aot-wasm/src/builtins/array.rs:3047` | three: `x ≠ x`, `x = +∞`, `x = −∞` (±0 correctly falls through) | **correct** — the reference |
| 4 | `emit_to_length_i64_from_number_payload_local` | `porffor-aot-wasm/src/builtins/array.rs:1977` | two: `x ≠ x`, `x ≤ 0` (folds 7.1.20 step 2's clamp and −∞ into one test) | **correct** |
| 5 | `emit_to_index_from_number_payload` | `porffor-aot-wasm/src/operations.rs:4112` | one: `x ≠ x`; ±∞ routed to the RangeError branch instead | **correct** |

The area title's word is **divergent**, and that is exactly what was measured:
five independent derivations of one shared step, of which two are wrong. It is
*not* five defects. §5.2 states this as a correction to the brief.

### 1.5 The N1 defect, stated exactly

`compile_bitwise_number_payload` (`expressions.rs:1701-1713`) computes, for each
operand:

```
LocalGet(int_local); F64ReinterpretI64; I64TruncSatF64S; LocalSet(int_local)
...
LocalGet(int_local); I32WrapI64
```

`I64TruncSatF64S` maps NaN → 0, `+∞` → `i64::MAX`, `−∞` → `i64::MIN`, and
saturates finite operands outside `[−2^63, 2^63)` to the same two bounds.
`I32WrapI64` then takes the low 32 bits. Composing:

- `I32WrapI64(i64::MAX)` = `0xFFFF_FFFF` = **−1** as `i32`.
- `I32WrapI64(i64::MIN)` = `0x0000_0000` = **0**.

**Claim N1a: every binary64 `x ≥ 2^63`, and `+∞`, produce a wrong ToInt32.**
For `+∞`, 7.1.6 step 2 requires `+0` and the code yields `−1`. For finite
`x ≥ 2^63`, the code yields `−1` unconditionally, so it is correct only if
`truncate(x) ≡ 2^32 − 1 (mod 2^32)`. But `x ≥ 2^63` implies
`ulp(x) ≥ 2^(63−52) = 2^11`, so `truncate(x)` is a multiple of 2048, while
`2^32 − 1` is odd. The congruence is therefore unsatisfiable. **Derived, and
confirmed by sampling 200 000 binary64 values drawn uniformly over exponents
63–120: 200 000 wrong, 0 right.**

**Claim N1b: `x ≤ −2^63` is wrong whenever `truncate(x)` is not a multiple of
2^32.** The code yields `0`; the spec yields `truncate(x) mod 2^32` read signed.
For `|x| ≥ 2^84` the ulp is ≥ 2^32 and the residue is 0, so the code is
accidentally right; in the band `(−2^84, −2^63]` it is usually wrong.
**Measured by sampling: 33.61 % wrong over exponents 63–120 overall, 92.83 %
wrong when restricted to the band `(−2^84, −2^63]`.**

**Claim N1c: `|x| < 2^63` is entirely correct.** `I64TruncSatF64S` is exact
there (Theorem C), `I32WrapI64` is the residue, and the signed reading is
7.1.6 step 5. **Sampled 200 000 values over exponents −20…62 of both signs:
0 wrong.** The defect window is precisely `|x| ≥ 2^63` plus `+∞`.

Worked counterexamples (all computed exactly, not estimated):

| JS | 7.1.6 requires | `expressions.rs` yields |
|---|---|---|
| `Infinity \| 0` | `0` | `-1` |
| `Infinity << 0` | `0` | `-1` |
| `(2**63) \| 0` | `0` | `-1` |
| `(1e300) \| 0` | `0` | `-1` |
| `(2**84) \| 0` | `0` | `-1` |
| `(2**63 + 2048) \| 0` | `2048` | `-1` |
| `(-(2**63 + 2048)) \| 0` | `-2048` | `0` |
| `NaN \| 0` | `0` | `0` — **accidentally** correct |
| `-Infinity \| 0` | `0` | `0` — **accidentally** correct |
| `(-1e300) \| 0` | `0` | `0` — **accidentally** correct |

The three accidental passes are the reason this defect survived: the obvious
probes (`NaN|0`, `-Infinity|0`) are exactly the ones that pass.

### 1.6 The N2 defect, stated exactly

`emit_to_uint32_i64_from_number_payload` (`string.rs:15052`) guards NaN, `±0`
and `±∞` correctly, then computes `F64Trunc` → `I64TruncSatF64S` → `I64RemS 2^32`
→ sign-correct-by-adding-2^32.

The sign correction is right (it converts Wasm's dividend-signed remainder into
5.2.5's non-negative residue). The **saturation before the modulo** is not: for
`|x| ≥ 2^63` the `I64TruncSatF64S` clamps to `i64::MAX` / `i64::MIN`, and
`i64::MAX mod 2^32 = 2^32 − 1`.

| JS | 7.1.7 requires | `string.rs` yields | `array.rs` yields |
|---|---|---|---|
| `ToUint32(2**64)` | `0` | `4294967295` | `0` |
| `ToUint32(2**63)` | `0` | `4294967295` | `0` |
| `ToUint32(1e300)` | `0` | `4294967295` | `0` |
| `ToUint32(2**63 + 2048)` | `2048` | `4294967295` | `2048` |
| `ToUint32(-(2**63 + 2048))` | `4294965248` | `0` | `4294965248` |
| `ToUint32(-1)` | `4294967295` | `4294967295` | `4294967295` |
| `ToUint32(2**32)` | `0` | `0` | `0` |
| `ToUint32(2**32 - 1)` | `4294967295` | `4294967295` | `4294967295` |
| `ToUint32(±Infinity)`, `ToUint32(NaN)`, `ToUint32(±0)` | `0` | `0` | `0` |

Over the 18-case probe used to build this table: `expressions.rs` disagrees with
7.1.6 in **7** cases, `string.rs` disagrees with 7.1.7 in **6**, `array.rs`
disagrees in **0**.

`emit_to_uint32_i64_from_number_payload` is reached from
`emit_split_limit_to_uint32_local` (`string.rs:15030`, called at `string.rs:2913`,
`:14333`, `:14356`), i.e. `String.prototype.split` and `Symbol.split`. 22.1.3.23
step 6 is `Let lim be ℝ(? ToUint32(limit))`, so:

- `"a,b,c".split(",", 2**64)` must be `[]` (lim = 0); today lim = 4294967295 and
  all three elements are returned.
- `"a,b,c".split(",", -1)` must be all three (lim = 2^32 − 1); correct today.
- `"a,b,c".split(",", 2**32)` must be `[]`; correct today.

The three-input probe separates the two implementations while pinning the two
boundary behaviours the retrofit must not regress.

### 1.7 The three `Number.prototype` orderings

This is where the in-crate defect lives, and it is an **ordering** obligation,
not a value obligation. The three clauses interleave the same three steps —
coerce the argument, return early for a non-finite receiver, range-check the
argument — in three different orders.

**21.1.3.2 `Number.prototype.toExponential ( fractionDigits )`**

```
1. Let x be ? ThisNumberValue(this value).
2. Let f be ? ToIntegerOrInfinity(fractionDigits).
3. Assert: If fractionDigits is undefined, then f is 0.
4. If x is not finite, return Number::toString(x, 10).
5. If f < 0 or f > 100, throw a RangeError exception.
...
12.a. If fractionDigits is undefined, [choose the shortest representation].
```

Non-finite receiver **wins**: `Infinity.toExponential(101)` is `"Infinity"`, not
a RangeError. Note step 12.a: `fractionDigits === undefined` is observably
distinct from `f === 0`, so an `Option` is spec-shaped here, not a convenience.

**21.1.3.3 `Number.prototype.toFixed ( fractionDigits )`**

```
1. Let x be ? ThisNumberValue(this value).
2. Let f be ? ToIntegerOrInfinity(fractionDigits).
3. Assert: If fractionDigits is undefined, then f is 0.
4. If f is not finite, throw a RangeError exception.
5. If f < 0 or f > 100, throw a RangeError exception.
6. If x is not finite, return Number::toString(x, 10).
...
```

Range check **wins**: `Infinity.toFixed(101)` is a **RangeError**. Step 4 is
mathematically subsumed by step 5 once `f` is compared on the extended integers,
and is present only because the spec spells the infinity case out.

**21.1.3.5 `Number.prototype.toPrecision ( precision )`**

```
1. Let x be ? ThisNumberValue(this value).
2. If precision is undefined, return ! ToString(x).
3. Let p be ? ToIntegerOrInfinity(precision).
4. If x is not finite, return Number::toString(x, 10).
5. If p < 1 or p > 100, throw a RangeError exception.
...
```

Non-finite receiver **wins**, *and the interval is `[1, 100]`, not `[0, 100]`*.
Step 2 short-circuits before any coercion, so `undefined` never reaches 7.1.5.

Three facts follow, and each is a mistake this contract must close:

- **F1.** The "is the receiver non-finite" early return sits on **either side**
  of the range check depending on the clause. There are exactly two orders.
- **F2.** The accepted interval is `[0, 100]` for two clauses and `[1, 100]` for
  the third. A shared `is_invalid(args)` predicate hard-coding `0..=100` is
  therefore **not reusable** at the third site even if someone remembers to
  call it.
- **F3.** All three comparisons (`f < 0`, `f > 100`, `p < 1`) are evaluated on
  **ℤ ∪ {±∞}**. `+∞ > 100` and `−∞ < 0` must hold. A type that cannot represent
  `±∞` cannot evaluate these steps; it can only approximate them.

### 1.8 Latitude, and the choices this contract makes

The spec leaves four things open. Each choice is recorded here so the encoder
does not re-decide it and the dry-runner can check it.

| # | Latitude | Choice | Why |
|---|---|---|---|
| C1 | 7.1.5's codomain is a mathematical set; the carrier is unspecified. | `enum IntegerOrInfinity { NegativeInfinity, Finite(FiniteInteger), PositiveInfinity }`, `FiniteInteger` wrapping an `f64` constrained to be finite, integral and not `−0.0`. | Theorem A: every reachable finite value is binary64-exact. An `i64` or `i128` carrier would be *lossy*, not merely awkward — `truncate(1e300)` does not fit. |
| C2 | Constant folding is optional; a compiler may always defer to the runtime. | A fold that would produce a **wrong value** must be removed; a fold that **declines** may stay. `NumberFormatFold::NotStatic` is a legitimate answer everywhere. | Spec correctness before speed (AGENTS.md). A decline costs a runtime call; a wrong fold costs conformance. |
| C3 | Whether `String.fromCharCode` folding accepts a non-finite loop induction variable. | The conversion is **total** (`Uint16::of_number(Infinity)` = `Uint16(0)`, per 7.1.9 step 2), and the *fold* declines separately, at the caller, on a `!is_finite()` test that belongs to the static generator's own domain. | §5.4. The decline is a property of enumerating an arithmetic progression, not of ToUint16. Splitting them keeps the codomain honest **and** keeps rung G byte-identical. |
| C4 | Whether `ToInt32`, `ToLength`, `ToIndex` get codomain newtypes. | **No.** They get `const fn` reference algorithms plus `const _: () = assert!(...)` tables. | There is no construction site for them in `porffor-ir`. Round 1 deleted `PropertyDescriptorIr` and `IntegerIndexedConversionIr` for exactly this (mistake class N7). A newtype nobody constructs is decoration; a const table that fails the build is not. |

---

## 2. Type mapping

New file: **`crates/porffor-ir/src/numeric_conversions.rs`**, declared inside
`crates/porffor-ir/src/ir.rs` immediately after the existing `reference` module
(`ir.rs:22-23`) and following its shape exactly:

```rust
/// Numeric conversion codomains (7.1.5, 7.1.6, 7.1.7, 7.1.9, 7.1.20, 7.1.22).
/// See `docs/rust-rewrite/contracts/numeric-conversion-codomains.md`.
///
/// Declared here rather than in `lib.rs` for the same reason as `reference`:
/// `lib.rs` is a single-lane hub owned by another area this round, and the
/// `#[path]` keeps the module a sibling file on disk.
#[path = "numeric_conversions.rs"]
pub mod numeric_conversions;

pub use numeric_conversions::{
    reference_to_index, reference_to_int32, reference_to_length, reference_to_uint16,
    reference_to_uint32, residue_pow2_i64, ExtendedInteger, FiniteInteger, FractionDigits,
    IntegerOrInfinity, NonFiniteReceiverOrder, NumberFormatFold, Precision, RangeChecked,
    ToIndexOutcome, Uint16, Uint32, MAX_SAFE_INTEGER_U64,
};
```

`lib.rs:79` is `pub use ir::*;` (verified), so both the module and the
re-exports reach `porffor_ir::` with no edit to `lib.rs`. `numeric_conversions.rs`
begins with `use super::*;`, matching `reference.rs:24`.

### 2.0 Invariant index

| # | Invariant | Carried by | §|
|---|---|---|---|
| I1 | 7.1.5's codomain includes `±∞` and unbounded integers | `IntegerOrInfinity` (3-variant enum) | 2.1 |
| I2 | A `FiniteInteger` is finite, integral, and never `−0.0` | private tuple field + single validating constructor | 2.1 |
| I3 | 7.1.5 is **total**: there is no "failed to convert" | `of_number(f64) -> Self`, not `-> Option<Self>` | 2.1 |
| I4 | Comparisons in 21.1.3.x are on the **extended** integers | `fraction_digits()` / `precision()` are the only readers; no `PartialOrd<i32>` exists | 2.2 |
| I5 | "Out of range" is a **RangeError**, never a fold decline | `RangeChecked<T>` — deliberately not `Option` | 2.2 |
| I6 | `[0,100]` and `[1,100]` are different intervals belonging to different clauses | two distinct newtypes `FractionDigits` / `Precision` | 2.2 |
| I7 | The non-finite-receiver early return has exactly two positions, **and each clause gets the right one** | `NonFiniteReceiverOrder` (2-variant enum) as a per-clause `const` on the `NumberFormatClause` trait, selected by a type parameter whose `Digits` associated types are pairwise distinct | 2.3 |
| I7b | Each clause throws **its own** RangeError message | `NumberFormatClause::RANGE_ERROR`, carried out in `NumberFormatFold::RangeError(&'static str)`; no string literal at the three dispatch arms | 2.3 |
| I8 | A fold has three outcomes, and `RangeError` must be handled | `#[must_use] enum NumberFormatFold`, matched exhaustively with no `_` arm | 2.3 |
| I9 | ToUint32's residue is 5.2.5 `modulo`, produced in one place | `Uint32`, private field, single constructor `of_number` | 2.4 |
| I10 | ToUint16 likewise, modulus 2^16 | `Uint16`, ditto | 2.4 |
| I10b | The **modulus** is tied to the **carrier**, and is never 0 or 64 | `ResidueWidth` (2-variant enum) + `ResidueCarrier` (`const WIDTH`, `of_residue`), so `residue_of_number::<u32>` derives its own modulus | 2.4, 2.5 |
| I11 | 7.1.6/7.1.7/7.1.9 share steps 2–4 and differ only in step 5 | `residue_pow2_i64` + cross-table `const _: () = assert!` | 2.5, 2.7 |
| I12 | `NormalResult` rows agree with the codomains | `const _: () = assert!(matches!(...))` in `operations.rs` | 2.6 |

### 2.1 `IntegerOrInfinity` and `FiniteInteger` — I1, I2, I3

```rust
/// 7.1.5 ToIntegerOrInfinity's codomain: **ℤ ∪ {+∞, −∞}**.
///
/// Not `i32` (which cannot hold `truncate(1e300)`), not `Option<i32>` (whose
/// `None` means "no answer" where 7.1.5 always has one), and not `f64` (whose
/// domain admits `0.5` and `NaN`, neither of which 7.1.5 can return).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntegerOrInfinity {
    NegativeInfinity,
    Finite(FiniteInteger),
    PositiveInfinity,
}

/// `truncate(ℝ(number))` for a finite `number`. Exactly representable in
/// binary64 (Theorem A). Never NaN, never infinite, never `-0.0`.
///
/// The field is private to this module, so `IntegerOrInfinity::Finite(..)`
/// cannot be spelled anywhere else in the workspace: an outside caller has no
/// way to obtain the payload. That is what makes `of_number` the *only*
/// constructor of the enum, not merely the recommended one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FiniteInteger(f64);

impl IntegerOrInfinity {
    /// 7.1.5 steps 2–5, applied to a value that has already been through
    /// step 1 (`? ToNumber`). **Total** — 7.1.5 has no failure mode once the
    /// argument is a Number, so this returns `Self`, not `Option<Self>`.
    pub fn of_number(number: f64) -> Self {
        if number.is_nan() || number == 0.0 {
            // Step 2. `number == 0.0` is true for both `+0.0` and `-0.0`.
            return Self::Finite(FiniteInteger(0.0));
        }
        if number == f64::INFINITY {
            return Self::PositiveInfinity;   // step 3
        }
        if number == f64::NEG_INFINITY {
            return Self::NegativeInfinity;   // step 4
        }
        // Step 5. `trunc` is exact (Theorem A). `+ 0.0` normalises a `-0.0`
        // produced by e.g. `(-0.5).trunc()` into the mathematical 0 step 2
        // would have returned; without it two spellings of the same
        // mathematical value would escape.
        Self::Finite(FiniteInteger(number.trunc() + 0.0))
    }
}
```

> **Encoder note.** `(-0.5f64).trunc()` is `-0.0`, and `-0.0 + 0.0` is `+0.0`
> under round-to-nearest. Do not replace the `+ 0.0` with `.abs()` or with a
> comparison; it is the shortest exact normalisation. Do not remove it: it is
> what makes `PartialEq` on `FiniteInteger` agree with 5.2.5 equality of
> mathematical values.

**No accessor returning the raw number is provided, and no `PartialOrd`.** There
is deliberately no `FiniteInteger::value()`, no `as_f64()`, no `as_i32()`, no
`Deref`. Adding a raw accessor would restore mistake class N3 in one line.

The first landing derived `PartialOrd` on `FiniteInteger`, and that was the same
hole by another route — this section's claim that "the only ways out of an
`IntegerOrInfinity` are `fraction_digits()` and `precision()`" was false while it
was there. Ordering is all an interval test needs, `Finite` is a public variant,
and `of_number` is a public source of comparison constants, so any crate
depending on `porffor-ir` could write

```rust
match (
    IntegerOrInfinity::of_number(v),
    IntegerOrInfinity::of_number(2.0),
    IntegerOrInfinity::of_number(36.0),
) {
    (
        IntegerOrInfinity::Finite(x),
        IntegerOrInfinity::Finite(lo),
        IntegerOrInfinity::Finite(hi),
    ) => x >= lo && x <= hi,
    _ => false,
}
```

— a hand-rolled `[2, 36]` radix check (21.1.3.4) on the extended integers, with a
catch-all that silently answers "out of range" for `±∞`. That is N3 rebuilt from
outside using only the public API. The derive is **deleted**; nothing in the
module compares two `FiniteInteger`s (`fraction_digits` and `precision` compare
the destructured `f64` via `(0.0..=100.0).contains(&value)`), so it was pure
attack surface. `PartialEq` alone cannot express an interval and stays.

If a future clause needs to order the extended integers, the correct primitive is
an ordering on `IntegerOrInfinity` itself, `NegativeInfinity < Finite <
PositiveInfinity` — which is the order 21.1.3.x actually evaluates in, and the
one a `FiniteInteger`-only comparison cannot express. It arrives with its own
call site and its own amendment, not as a derive.

### 2.2 `RangeChecked`, `FractionDigits`, `Precision` — I4, I5, I6

```rust
/// The outcome of testing an `IntegerOrInfinity` against one clause's closed
/// interval, on the **extended** integers.
///
/// Deliberately **not** `Option`. The three folding helpers in `lowering.rs`
/// return `Option<String>`, so `?` on an `Option` there spells "decline the
/// fold" — which is exactly the wrong answer the tree ships today
/// (`lowering.rs:34724`, `:34747`, `:34789`). `RangeError` is not `None`, and
/// `?` on a `RangeChecked` does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub enum RangeChecked<T> {
    InBounds(T),
    RangeError,
}

impl<T> RangeChecked<T> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> RangeChecked<U> { ... }
}

/// 21.1.3.2 step 5 / 21.1.3.3 steps 4–5: a fraction-digit count already checked
/// against `[0, 100]`. Constructible only by `IntegerOrInfinity::fraction_digits`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FractionDigits(u8);

/// 21.1.3.5 step 5: a precision already checked against `[1, 100]`.
/// A **different type** from `FractionDigits`, because `[0,100]` and `[1,100]`
/// are different intervals belonging to different clauses; passing one where
/// the other is expected is `E0308`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Precision(u8);

impl FractionDigits {
    /// 21.1.3.3 step 3's assertion: `fractionDigits` undefined ⇒ f is 0.
    pub const ZERO: Self = Self(0);
    /// The formatter interface in `lowering.rs` predates this contract and
    /// takes `usize`; those signatures are outside this area's owned regions.
    pub fn as_usize(self) -> usize { self.0 as usize }
}

impl Precision {
    /// In `1..=100` by construction. `static_number_to_precision`'s body
    /// computes `precision as usize - 1`, which is why the lower bound is
    /// carried by the type rather than re-tested there (ledger **LN3**).
    pub fn get(self) -> u8 { self.0 }
}

impl IntegerOrInfinity {
    /// 21.1.3.2 step 5 and 21.1.3.3 steps 4–5: `f < 0 or f > 100`.
    /// `+∞ > 100` and `−∞ < 0` hold, which is the whole reason 7.1.5's
    /// codomain had to include them.
    pub fn fraction_digits(self) -> RangeChecked<FractionDigits> {
        match self {
            Self::NegativeInfinity | Self::PositiveInfinity => RangeChecked::RangeError,
            Self::Finite(FiniteInteger(v)) => {
                if (0.0..=100.0).contains(&v) {
                    RangeChecked::InBounds(FractionDigits(v as u8))
                } else {
                    RangeChecked::RangeError
                }
            }
        }
    }

    /// 21.1.3.5 step 5: `p < 1 or p > 100`.
    pub fn precision(self) -> RangeChecked<Precision> {
        match self {
            Self::NegativeInfinity | Self::PositiveInfinity => RangeChecked::RangeError,
            Self::Finite(FiniteInteger(v)) => {
                if (1.0..=100.0).contains(&v) {
                    RangeChecked::InBounds(Precision(v as u8))
                } else {
                    RangeChecked::RangeError
                }
            }
        }
    }
}
```

Both matches are exhaustive over the three variants with **no `_` arm**; adding
a fourth variant to `IntegerOrInfinity` is `E0004` here.

The `v as u8` casts are exact: `v` is an exact integer already proven to lie in
`[0, 100]` or `[1, 100]`, so Rust's saturating float→int cast never saturates.

### 2.3 `NonFiniteReceiverOrder` and `NumberFormatFold` — I7, I8

```rust
/// Where 21.1.3.x puts "if x is not finite, return Number::toString(x, 10)"
/// relative to the digit-count range check. There are exactly two orders and
/// the choice is observable, so it is a closed enum and a **required**
/// argument — omitting it is `E0061`, adding a third order is `E0004` at every
/// match over it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonFiniteReceiverOrder {
    /// 21.1.3.2 step 4 precedes step 5; 21.1.3.5 step 4 precedes step 5.
    /// `Infinity.toExponential(101) === "Infinity"`.
    /// `Infinity.toPrecision(0) === "Infinity"`.
    ReceiverFirst,
    /// 21.1.3.3 steps 4–5 precede step 6.
    /// `Infinity.toFixed(101)` throws a **RangeError**.
    RangeCheckFirst,
}

/// What a static `Number.prototype` fold concluded. Replaces `Option<String>`
/// at the three folding helpers so that "the spec requires a RangeError here"
/// and "I could not fold this" stop being the same value.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub enum NumberFormatFold {
    /// Emit `ExprIr::String`.
    Formatted(String),
    /// Emit `ExprIr::RuntimeThrow { name: NativeErrorKind::RangeError, .. }`
    /// with the carried, clause-owned message.
    RangeError(&'static str),
    /// Not statically decidable; fall through to the runtime call path.
    NotStatic,
}

/// One of the three clauses, as a **type**. This is what closes N4″.
pub trait NumberFormatClause {
    /// The clause's already-range-checked digit argument. Pairwise distinct
    /// across the three impls, which is what makes naming the wrong clause a
    /// type error rather than a silent reordering.
    type Digits;
    const ORDER: NonFiniteReceiverOrder;
    const RANGE_ERROR: &'static str;
}

pub enum ToExponential {}   // 21.1.3.2
pub enum ToFixed {}         // 21.1.3.3
pub enum ToPrecision {}     // 21.1.3.5

impl NumberFormatClause for ToExponential {
    /// Step 12.a needs `undefined` observably distinct from `f === 0`.
    type Digits = Option<FractionDigits>;
    const ORDER: NonFiniteReceiverOrder = NonFiniteReceiverOrder::ReceiverFirst;
    const RANGE_ERROR: &'static str = "Number.prototype.toExponential fraction digits out of range";
}
impl NumberFormatClause for ToFixed {
    type Digits = FractionDigits;
    const ORDER: NonFiniteReceiverOrder = NonFiniteReceiverOrder::RangeCheckFirst;
    const RANGE_ERROR: &'static str = "Number.prototype.toFixed fraction digits out of range";
}
impl NumberFormatClause for ToPrecision {
    type Digits = Precision;
    const ORDER: NonFiniteReceiverOrder = NonFiniteReceiverOrder::ReceiverFirst;
    const RANGE_ERROR: &'static str = "Number.prototype.toPrecision precision out of range";
}

/// The single driver for 21.1.3.2 / 21.1.3.3 / 21.1.3.5. All three clauses are
/// this function at different type arguments; there is no fourth spelling.
pub fn fold_number_format<C: NumberFormatClause>(
    receiver: f64,
    digits: RangeChecked<C::Digits>,
    number_to_string: impl FnOnce(f64) -> String,
    format: impl FnOnce(f64, C::Digits) -> Option<String>,
) -> NumberFormatFold {
    let finish = |digits: C::Digits| match format(receiver, digits) {
        Some(text) => NumberFormatFold::Formatted(text),
        None => NumberFormatFold::NotStatic,
    };
    match C::ORDER {
        NonFiniteReceiverOrder::ReceiverFirst => {
            if !receiver.is_finite() {
                return NumberFormatFold::Formatted(number_to_string(receiver));
            }
            match digits {
                RangeChecked::RangeError => NumberFormatFold::RangeError(C::RANGE_ERROR),
                RangeChecked::InBounds(d) => finish(d),
            }
        }
        NonFiniteReceiverOrder::RangeCheckFirst => match digits {
            RangeChecked::RangeError => NumberFormatFold::RangeError(C::RANGE_ERROR),
            RangeChecked::InBounds(d) => {
                if !receiver.is_finite() {
                    return NumberFormatFold::Formatted(number_to_string(receiver));
                }
                finish(d)
            }
        },
    }
}
```

Note that `digits: RangeChecked<T>` is passed **already checked**. The driver
cannot perform the check itself because the interval is clause-specific
(fact F2), and threading a bounds selector back in would re-open the mistake it
closes. The check happens where the clause is known — at the call site, by
calling `fraction_digits()` or `precision()` — and the driver only enforces the
*ordering*, which is what it is for.

`fold_number_format` earns its place under the AGENTS.md test: without it each
of the three call sites re-derives the order by hand, which is what they do
today at `lowering.rs:20696-20698`, `:20738-20740` and `:20754-20756`. With it,
the order is a required argument from a closed domain.

**N4″ is closeable after all, and the clause trait closes it.** This section
originally stopped at "the required enum argument makes *omitting* the decision
`E0061` and a third order `E0004`; naming the **wrong** one is still one
identifier". That was true and it was not good enough: swapping `ReceiverFirst`
and `RangeCheckFirst` at any of the three `*_call` helpers compiles and makes
`Infinity.toFixed(101)` answer `"Infinity"` instead of throwing. The same shape
applied to the message: three adjacent dispatch arms each spelled a `&'static
str` literal, so pasting `toFixed`'s text into the `toPrecision` arm compiled.

Making the clause a **type parameter** closes both, and it costs one trait and
three uninhabited marker types:

- `C::Digits` is `Option<FractionDigits>` / `FractionDigits` / `Precision`,
  pairwise distinct, so naming the wrong clause is **`E0308`** on the `digits`
  argument. The `Option` is not incidental: 21.1.3.2 step 12.a is why
  `toExponential` alone carries it, so the discrimination is spec-derived.
- `C::ORDER` and `C::RANGE_ERROR` are properties *of the clause*, not values the
  caller supplies, so there is nothing left at the call site to transpose or
  paste. `fold_number_format` takes **four** arguments, not five, and the three
  dispatch arms in `lowering.rs` contain no string literal.
- Inference cannot pick `C` from `RangeChecked<C::Digits>` (associated types are
  not injective), so the turbofish is mandatory — naming the clause is a
  required, single, spelled decision.

**Still refused: a typestate builder.** The area brief lists "typestate builders
that will not emit until ordering obligations are met" as a favoured form. It is
the wrong form here. A typestate encodes an ordering the *caller* performs step
by step; here the ordering is a per-clause constant. A `Builder<ReceiverChecked>`
phantom would add three types and two methods and would reject no program the
clause trait does not already reject. The clause trait passes AGENTS.md's test —
it turns two plausible mistakes into compile errors — where the builder does
not, and that difference is the whole reason one is taken and the other is not.

### 2.4 `Uint32` and `Uint16` — I9, I10

```rust
/// 7.1.7 ToUint32's codomain: `int modulo 2^32`, read unsigned.
///
/// The field is private, so `Uint32(x)` outside this module is `E0603`: a
/// second, hand-rolled residue cannot produce this type. That is the in-crate
/// form of mistake class N2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Uint32(u32);

/// 7.1.9 ToUint16's codomain: `int modulo 2^16`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Uint16(u16);

impl Uint32 {
    /// 7.1.7 steps 2–5 on a value already through step 1. **Total.**
    pub fn of_number(number: f64) -> Self {
        Self(residue_of_number(number, 32) as u32)
    }
    /// 21.3.2.11 `Math.clz32` step 2 — the only reader, and the only one this
    /// crate has a call site for.
    pub fn leading_zeros(self) -> u32 { self.0.leading_zeros() }
}

impl Uint16 {
    /// 7.1.9 steps 2–5 on a value already through step 1. **Total.**
    pub fn of_number(number: f64) -> Self {
        Self(residue_of_number(number, 16) as u16)
    }
    /// 22.1.2.1 step 2.a — the single UTF-16 code unit `String.fromCharCode`
    /// appends. The only reader.
    pub fn code_unit(self) -> u16 { self.0 }
}

/// 7.1.6/7.1.7/7.1.9 steps 2–4, shared. `bits` is 32 or 16.
///
/// The `|v| < 2^63` branch routes through the const-asserted integer core
/// (Theorem C), so `residue_pow2_i64` is on the product path and not merely a
/// build-time artefact. The outer branch is exact by Theorem B and is the one
/// ledger row this file owes (**LN1**).
fn residue_of_number(number: f64, bits: u32) -> u64 {
    // Step 2: non-finite or ±0 → +0. `number == 0.0` covers `-0.0`.
    if !number.is_finite() || number == 0.0 {
        return 0;
    }
    let truncated = number.trunc();               // step 3, exact (Theorem A)
    const I64_SPAN: f64 = 9_223_372_036_854_775_808.0; // 2^63, exactly representable
    if truncated >= -I64_SPAN && truncated < I64_SPAN {
        residue_pow2_i64(truncated as i64, bits)  // step 4, exact (Theorem C)
    } else {
        // step 4 for |x| ≥ 2^63, exact by Theorem B.
        truncated.rem_euclid((1u64 << bits) as f64) as u64
    }
}
```

Neither type has `get()`, `From`, `Into`, `Deref`, or a public field. `Uint32`
exposes exactly `leading_zeros`; `Uint16` exposes exactly `code_unit`. Each
matches a single spec step at a single call site. If a later area needs another
reader it adds it with its call site, never speculatively — this is the
enforcement of mistake class N7.

### 2.5 The const reference algorithms — I11, and C4

These carry **no** newtype. They are the normative statement of 7.1.6, 7.1.7,
7.1.9, 7.1.20 and 7.1.22 that the five backend emitters must match, and their
build-time consumer is §2.7's const tables.

```rust
/// 2^53 − 1. 7.1.20 step 3's clamp and 7.1.22 step 2's upper bound.
pub const MAX_SAFE_INTEGER_U64: u64 = 9_007_199_254_740_991;

/// 5.2.5 `x modulo 2^bits` for a mathematical integer that fits an `i64`.
///
/// **This is the step all five backend hand-rolls re-derive and two get
/// wrong.** It is a *non-negative* residue, not a machine remainder: Wasm's
/// `I64RemS` takes the sign of the dividend and needs a correction, and
/// `I64TruncSatF64S` applied before this step destroys the residue outright
/// (§1.5, §1.6).
///
/// Two's complement is the residue system modulo 2^64, so masking the low `N`
/// bits is the exact residue (Theorem C). No floating point, no `rem_euclid`,
/// no libm — which is what lets it be `const` on stable and lets §2.7's tables
/// be checked by the compiler.
///
/// `width` is a closed domain, not a `u32`. This item is `pub` and is the
/// normative algorithm the LN2 backend retrofit will call; with a bare `bits`,
/// `residue_pow2_i64(x, 64)` and `(x, 0)` compiled, and in **release** — where
/// the `debug_assert!` is gone — `1u64 << 64` is a masked shift evaluating to
/// `1`, the mask becomes `0`, and the function silently returns `0` for every
/// input. Only `const` callers were protected, because `1u64 << 64` is a hard
/// error in const evaluation.
pub enum ResidueWidth { Bits16, Bits32 }

impl ResidueWidth {
    pub const fn mask(self) -> u64 {
        match self {
            Self::Bits16 => (1u64 << 16) - 1,
            Self::Bits32 => (1u64 << 32) - 1,
        }
    }
}

pub const fn residue_pow2_i64(int: i64, width: ResidueWidth) -> u64 {
    (int as u64) & width.mask()
}

/// Ties the modulus to the destination carrier, which were two independent bare
/// integers at the two `of_number` sites. Editing `Uint32::of_number` to
/// `residue_of_number(number, 16) as u32` compiled and produced 7.1.9's residue
/// in 7.1.7's carrier — `Math.clz32(65537)` would have answered `31` instead of
/// `15`. `residue_of_number::<u32>(number)` now derives its own modulus and
/// there is no second number to get wrong.
pub trait ResidueCarrier: Copy {
    const WIDTH: ResidueWidth;
    fn of_residue(residue: u64) -> Self;
}

/// 7.1.6 steps 4–5, given `truncate(ℝ(number))` reduced into an `i64`.
pub const fn reference_to_int32(truncated: i64) -> i32 {
    residue_pow2_i64(truncated, 32) as u32 as i32
}

/// 7.1.7 steps 4–5.
pub const fn reference_to_uint32(truncated: i64) -> u32 {
    residue_pow2_i64(truncated, 32) as u32
}

/// 7.1.9 steps 4–5.
pub const fn reference_to_uint16(truncated: i64) -> u16 {
    residue_pow2_i64(truncated, 16) as u16
}

/// 7.1.5's codomain restricted to the integers an `i64` holds, plus the two
/// infinities.
///
/// This is **not** a codomain type and nothing lowers through it. Its payload
/// field is public precisely because there is no invariant to protect: every
/// `i64` is a legal integer. It exists so 7.1.20 and 7.1.22 — which consume
/// 7.1.5's *extended* codomain — can be written as `const fn` and pinned by
/// §2.7's tables. Do not use it as a conversion result anywhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedInteger {
    NegInfinity,
    Finite(i64),
    PosInfinity,
}

/// 7.1.20 ToLength steps 2–3.
pub const fn reference_to_length(len: ExtendedInteger) -> u64 {
    match len {
        ExtendedInteger::NegInfinity => 0,
        ExtendedInteger::PosInfinity => MAX_SAFE_INTEGER_U64,
        ExtendedInteger::Finite(v) => {
            if v <= 0 {
                0
            } else if (v as u64) > MAX_SAFE_INTEGER_U64 {
                MAX_SAFE_INTEGER_U64
            } else {
                v as u64
            }
        }
    }
}

/// 7.1.22 ToIndex's two outcomes. `ToIndex` is the one **partial** operation in
/// this contract, and the partiality is spec-mandated, so it is a two-variant
/// enum rather than an `Option`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub enum ToIndexOutcome {
    Index(u64),
    RangeError,
}

/// 7.1.22 ToIndex step 2.
pub const fn reference_to_index(integer: ExtendedInteger) -> ToIndexOutcome {
    match integer {
        ExtendedInteger::NegInfinity | ExtendedInteger::PosInfinity => ToIndexOutcome::RangeError,
        ExtendedInteger::Finite(v) => {
            if v < 0 || (v as u64) > MAX_SAFE_INTEGER_U64 {
                ToIndexOutcome::RangeError
            } else {
                ToIndexOutcome::Index(v as u64)
            }
        }
    }
}
```

> **Encoder note on constness.** Everything above uses only `as`, `<<`, `-`,
> `&`, `<=`, `>`, `if`/`else` and `match` — all const-stable well before the
> pinned toolchain (`rustc 1.94.1`, edition 2021). Do **not** rewrite
> `residue_pow2_i64` in terms of `i64::rem_euclid` or `f64::rem_euclid`: the
> point of the mask formulation is that it needs no const-stability gamble and
> no floating point.
>
> **Correction.** An earlier revision of this note said `debug_assert!` in a
> `const fn` "is a no-op in const evaluation". It is not: with
> `debug_assertions` on it *is* evaluated during const evaluation. It happened
> to pass for 16 and 32, so nothing broke, but the claim was wrong and the
> integration note's §11.3 mitigation rested on the same misreading. The
> `debug_assert!` is gone anyway — `ResidueWidth` makes the states it guarded
> unrepresentable, which is the form AGENTS.md asks for.

`reference_to_int32` has **no runtime caller** in `porffor-ir` — there is no
ToInt32 site in this crate — and that is stated rather than hidden. Its
consumer is the const block in §2.7, so a wrong edit to it fails the build.
`reference_to_uint32` and `reference_to_uint16` do have runtime callers via
`residue_of_number` (§2.4).

### 2.6 The `NormalResult` rows — I12

`crates/porffor-ir/src/operations.rs:774-779` currently reads:

```rust
Self::ToNumber | Self::ToIntegerOrInfinity => NormalResult::Number,
...
Self::ToLength | Self::ToIndex => NormalResult::Integer,
```

`NormalResult` (`operations.rs:590-611`) is a **descriptive tag** for the
catalog's rendered "returns" column. It is not a codomain and this contract does
not promote it to one — nothing constructs a value of it. What the contract does
is state what each row claims and pin the claim:

- `ToIntegerOrInfinity => Number` is **correct and deliberate**: 7.1.5's
  codomain includes `±∞`, which are not integers, and `NormalResult::Integer`
  would be a false claim. The spec's own return spelling is "an integer or
  +∞ or −∞".
- `ToLength => Integer` and `ToIndex => Integer` are correct: 7.1.20 step 3 and
  7.1.22 step 3 both land in `[0, 2^53−1] ∩ ℤ`.

Add, at the end of `operations.rs`:

```rust
/// Ties the catalog's descriptive `NormalResult` rows to the codomain types in
/// `numeric_conversions`, so the two cannot drift.
///
/// `ToIntegerOrInfinity` is `Number`, not `Integer`, because 7.1.5's codomain
/// is ℤ ∪ {±∞} and `±∞` are not integers — the same fact that forces
/// `IntegerOrInfinity` to have three variants. If someone "tidies" this row to
/// `Integer` on the grounds that the operation has "Integer" in its name, this
/// assertion fails the build.
const _: () = {
    assert!(matches!(
        SpecOperationIr::ToIntegerOrInfinity.normal_result(),
        NormalResult::Number
    ));
    assert!(matches!(
        SpecOperationIr::ToLength.normal_result(),
        NormalResult::Integer
    ));
    assert!(matches!(
        SpecOperationIr::ToIndex.normal_result(),
        NormalResult::Integer
    ));
};
```

`normal_result` is `pub const fn` (verified at `operations.rs:756`), and
`matches!` expands to a `match`, so no non-const `PartialEq` is invoked.

Also annotate the three catalog rows at `operations.rs:692-694` with a one-line
comment pointing at this contract. Do not change the rows.

### 2.7 The const tables — the build-time evidence

In `numeric_conversions.rs`. Every row below was computed by hand from 5.2.5 and
cross-checked against an exact-integer model; none is copied from the tree's
current behaviour.

```rust
/// The residues 5.2.5 requires, checked at build time.
const _: () = {
    // 7.1.7 — the values `Math.clz32` and `String.prototype.split` depend on.
    assert!(reference_to_uint32(0) == 0);
    assert!(reference_to_uint32(-1) == 4_294_967_295);
    assert!(reference_to_uint32(4_294_967_295) == 4_294_967_295);
    assert!(reference_to_uint32(4_294_967_296) == 0);
    assert!(reference_to_uint32(4_294_967_297) == 1);
    assert!(reference_to_uint32(-4_294_967_295) == 1);
    assert!(reference_to_uint32(-4_294_967_296) == 0);
    assert!(reference_to_uint32(-4_294_967_297) == 4_294_967_295);
    // `i64::MAX`'s residue is the value the saturating backends return for
    // *every* out-of-window input. It is the right answer for this integer and
    // the wrong answer for `+∞`; the difference is the whole of defect N1.
    assert!(reference_to_uint32(i64::MAX) == 4_294_967_295);
    assert!(reference_to_uint32(i64::MIN) == 0);

    // 7.1.6 — same residue, step 5 reads it signed.
    assert!(reference_to_int32(0) == 0);
    assert!(reference_to_int32(-1) == -1);
    assert!(reference_to_int32(2_147_483_647) == 2_147_483_647);
    assert!(reference_to_int32(2_147_483_648) == -2_147_483_648);
    assert!(reference_to_int32(4_294_967_295) == -1);
    assert!(reference_to_int32(4_294_967_296) == 0);

    // 7.1.9 — test262 `String/fromCharCode/S9.7_A2.1.js` CHECK#3/#6/#7.
    assert!(reference_to_uint16(0) == 0);
    assert!(reference_to_uint16(1) == 1);
    assert!(reference_to_uint16(-1) == 65_535);
    assert!(reference_to_uint16(65_535) == 65_535);
    assert!(reference_to_uint16(65_536) == 0);
    assert!(reference_to_uint16(65_536 + 65) == 65);
    assert!(reference_to_uint16(4_294_967_295) == 65_535);

    // 7.1.20.
    assert!(reference_to_length(ExtendedInteger::NegInfinity) == 0);
    assert!(reference_to_length(ExtendedInteger::PosInfinity) == MAX_SAFE_INTEGER_U64);
    assert!(reference_to_length(ExtendedInteger::Finite(-1)) == 0);
    assert!(reference_to_length(ExtendedInteger::Finite(0)) == 0);
    assert!(reference_to_length(ExtendedInteger::Finite(5)) == 5);
    assert!(reference_to_length(ExtendedInteger::Finite(9_007_199_254_740_991)) == MAX_SAFE_INTEGER_U64);
    assert!(reference_to_length(ExtendedInteger::Finite(9_007_199_254_740_992)) == MAX_SAFE_INTEGER_U64);

    // 7.1.22.
    assert!(matches!(reference_to_index(ExtendedInteger::NegInfinity), ToIndexOutcome::RangeError));
    assert!(matches!(reference_to_index(ExtendedInteger::PosInfinity), ToIndexOutcome::RangeError));
    assert!(matches!(reference_to_index(ExtendedInteger::Finite(-1)), ToIndexOutcome::RangeError));
    assert!(matches!(reference_to_index(ExtendedInteger::Finite(0)), ToIndexOutcome::Index(0)));
    assert!(matches!(
        reference_to_index(ExtendedInteger::Finite(9_007_199_254_740_991)),
        ToIndexOutcome::Index(9_007_199_254_740_991)
    ));
    assert!(matches!(
        reference_to_index(ExtendedInteger::Finite(9_007_199_254_740_992)),
        ToIndexOutcome::RangeError
    ));
};

/// The inputs the three residue readings are cross-checked on.
const RESIDUE_TIE_INPUTS: [i64; 14] = [
    0, 1, -1, 255, 65_535, 65_536, 2_147_483_647, 2_147_483_648, -2_147_483_648,
    4_294_967_295, 4_294_967_296, -4_294_967_296, i64::MAX, i64::MIN,
];

/// **Ties the tables together.** 7.1.6 and 7.1.7 differ only in the reading of
/// one residue (§1.2), and 7.1.9 is the same residue at a narrower modulus.
/// If any of the three reference algorithms is edited in isolation, this fails
/// the build rather than the suite.
const _: () = {
    let mut i = 0;
    while i < RESIDUE_TIE_INPUTS.len() {
        let v = RESIDUE_TIE_INPUTS[i];
        // 7.1.6 step 5 and 7.1.7 step 5 read the *same* `int32bit`.
        assert!(reference_to_int32(v) as u32 == reference_to_uint32(v));
        // 7.1.9's modulus divides 7.1.7's, so the narrower residue is the
        // wider one's low half.
        assert!(reference_to_uint16(v) as u32 == reference_to_uint32(v) & 0xFFFF);
        // Both readings agree on the residue's membership in [0, 2^32).
        assert!(residue_pow2_i64(v, 32) == reference_to_uint32(v) as u64);
        i += 1;
    }
};
```

`while` loops, indexing and `matches!` are all const-stable on the pinned
toolchain.

### 2.8 The runtime-checked ledger

These are the only places where a test or a run-time check remains load-bearing
in this area. Each row carries the reason a type cannot carry the invariant.

| # | Invariant | Why no type | Where it is checked instead |
|---|---|---|---|
| **LN1** | `residue_of_number`'s `\|x\| ≥ 2^63` branch (`f64::rem_euclid`) agrees with the const-asserted integer core. | The bridge is Theorem B — a statement about IEEE-754 `fmod` exactness, i.e. about the hardware — and no type can carry it **for this formulation**. That is the honest wording; the row previously said "no type can carry this", which is stronger than the evidence. The branch does not need `rem_euclid`: for `\|x\| ≥ 2^63`, decomposing `x.to_bits()` as `m · 2^e` with `m` a 53-bit integer and `e ≥ 11` gives `x mod 2^N` as `0` when `e ≥ N` and `(m & ((1 << (N − e)) − 1)) << e` otherwise — pure `u64` integer arithmetic, `const`-evaluable on stable, exact by construction rather than by appeal to IEEE-754 §5.3.1. Rewriting it that way would move the 18-case boundary table into a third `const _` block and retire this row. Not taken this round; recorded so the choice is visible. | One `#[cfg(test)]` differential in `numeric_conversions.rs` over the boundary: `2^63 − 2048`, `2^63`, `2^63 + 2048`, `−2^63`, `−(2^63 + 2048)`, `2^84`, `1e300`, `−1e300`, `2^32`, `2^32 − 1`, `−1`, `±0`, `±∞`, `NaN`. Expected values are in §1.6's table. |
| **LN2** | The five `porffor-aot-wasm` emitters compute the same functions as §2.5's reference algorithms. | `porffor-ir` cannot see `porffor-aot-wasm`; this is the same crate-boundary shape as the existing ledger **L2** in the spec-operation contract. | `target/lane-notes/numeric-conversion-codomains-theory-integration.md` carries the retrofit and the acceptance gate. Out of this lane. |
| **LN3** | `Precision::get() ≥ 1`, relied on by `static_number_to_precision`'s body for `precision as usize - 1` (`lowering.rs:34965`). | The body is outside this area's owned region and takes a primitive; a dependent range type would require editing unowned lines. | `Precision` has one constructor, which tests `(1.0..=100.0)`. A `debug_assert!((1..=100).contains(&precision))` replaces the deleted range check at `lowering.rs:34952`. |
| **LN4** | `spec_to_integer_or_infinity`, `spec_to_length` and `spec_to_index` are reachable only from `#[cfg(test)]`. | Deleting them requires editing `crates/porffor-aot-wasm/src/lib.rs`'s test module (`:1468`, `:1486`, `:1503`), which is outside this lane and inside batch 2's crate. | **Measured and recorded only** (§0.1, §4.5). Handed to the integrator. |
| **LN5** | ~~`static_number_expr` resolves the identifiers `Infinity` and `NaN` by spelling alone.~~ **Closed.** The row was both understated and wrong about impact. Four resolvers tested the spelling with no scope lookup, not one: `static_number_expr`, `static_to_number_expr`, `static_number_to_string_receiver_value` and `is_static_undefined_expr`. And "not worsened by this contract" was **false at the `toPrecision` site**: `function f(){ let Infinity = 5; return (1.5).toPrecision(Infinity); }` used to reach the deleted `!(1..=100)` guard, decline, and give the correct runtime answer `"1.5000"`; with the new 21.1.3.5 step 5 arm it folded to `PositiveInfinity` → `RangeChecked::RangeError` → an emitted `RangeError` throw. A latent defect became a shipped wrong answer. | Nothing — this was never an invariant a type could not carry. The guard already existed in the same file (`expression_is_builtin_symbol_intrinsic`, `identifier_is_builtin_native_error`), with a doc comment recording that the class had already shipped two silent wrong answers; it simply was not called. | **Now a type-adjacent single helper.** The four clauses (9.1.1: no active `with`, no Environment Record binding, global proven present, source still `Builtin`) are one predicate, `ScriptLowerer::identifier_resolves_to_builtin_global`; the two pre-existing guards were rewritten to call it, and `static_global_number_identifier` wraps it for the three number-valued spellings. All four resolvers route through it. A fifth resolver that maps a spelling to a value is still a review item, which is what remains of this row. |

Five rows is the whole ledger. Everything else in §2.0 is a compile error.

---

## 3. The mistake-class table

| Class | The mistake, concretely | What it becomes |
|---|---|---|
| **N1** — saturate before the modulo | Writing `I64TruncSatF64S` before the residue, in-crate. | In `porffor-ir`, unrepresentable: `Uint32`'s field is private, so a hand-rolled residue cannot be spelled as a `Uint32` — **`E0603`** at the tuple-struct constructor. Across the crate boundary this is **ledger LN2**, not a compile error, and this contract does not claim otherwise. What it provides instead is `residue_pow2_i64` as the single normative algorithm plus §2.7's build-checked table. |
| **N1′** — "the modulo is a machine remainder" | Editing `residue_pow2_i64` to `int % (1 << bits)`, or `reference_to_uint32` to a truncating cast. | **Build failure** at the §2.7 const block: `reference_to_uint32(-1) == 4_294_967_295` and `reference_to_uint16(-1) == 65_535` both fail immediately. A `const` assertion failure is a hard compile error, not a warning. |
| **N2** — two implementations of one operation that disagree | Adding a second in-crate ToUint32. | **`E0603`**: `Uint32(x)` is not constructible outside `numeric_conversions`; there is no `From<u32>`, no `new`, no public field. The second implementation cannot produce the type the consumer accepts, so it cannot be wired in. |
| **N2′** — the two readings of one residue drift apart | Editing `reference_to_int32` without `reference_to_uint32`. | **Build failure** at the cross-table `const _` in §2.7: `reference_to_int32(v) as u32 == reference_to_uint32(v)` fails on the first tie input that distinguishes them. |
| **N3** — a codomain that cannot hold the spec's range | Keeping `-> Option<i32>` and mapping `±∞` to `Some(0)`. | The helper's return type becomes `Option<IntegerOrInfinity>`. `IntegerOrInfinity` has no `PartialOrd<i32>`, so `!(0..=100).contains(&fraction_digits)` is **`E0277`**; it has no numeric conversion, so `fraction_digits as usize` is **`E0605`**; and it has no `value()`, so there is no escape hatch. The only exits are `fraction_digits()` and `precision()`. |
| **N3′** — collapsing `+∞` into `+0` | `IntegerOrInfinity::of_number` returning `Finite(FiniteInteger(0.0))` for `f64::INFINITY`. | Not a compile error — it is a wrong function body, and the type cannot prevent a wrong body. It **is** caught by the §6.4 and §6.7 dry-run traces and by the LN1 differential, which include `±∞` explicitly. Stated here so the encoder does not believe the type does this work. |
| **N4** — a validity guard that is optional at the call site | Folding `toPrecision` without the range check, as `lowering.rs:20738-20748` does today. | `static_number_to_precision_call` returns `NumberFormatFold`, which is `#[must_use]` and has three variants. The dispatch matches it exhaustively with **no `_` arm**, so failing to handle `NumberFormatFold::RangeError` is **`E0004`**. Separately, `static_number_fraction_digits_is_invalid` is **deleted**, so there is no longer a guard to forget to call. |
| **N4′** — reusing the wrong interval | Calling `fraction_digits()` where 21.1.3.5 requires `precision()`. | **`E0308`**: `RangeChecked<FractionDigits>` is not `RangeChecked<Precision>`, and `Precision` is not constructible from a `FractionDigits`. This is why they are two types rather than one `u8` (fact F2). |
| **N4″** — putting the non-finite-receiver return on the wrong side | "Unifying" `toFixed`'s receiver guard with `toExponential`'s, which is a plausible tidy-up given they sit 60 lines apart and differ only by `.is_some()` vs `.is_some_and(is_finite)`. | **Closed.** The guard is deleted from both call sites and the position is `<C as NumberFormatClause>::ORDER`, a per-clause constant rather than an argument. Because `C::Digits` is `Option<FractionDigits>` / `FractionDigits` / `Precision` — pairwise distinct — naming the wrong clause at a `*_call` helper is **`E0308`** on the `digits` argument, and inference cannot supply `C` (associated types are not injective), so the clause must be spelled. A third order is still **`E0004`** at the match over `NonFiniteReceiverOrder`. This row previously conceded that the wrong *choice* was expressible; it no longer is. |
| **N4‴** — giving a clause another clause's RangeError message | Copy-pasting `"Number.prototype.toFixed fraction digits out of range"` into the `toPrecision` dispatch arm, three adjacent arms that each spelled a `&'static str` literal. | **Closed by the same construct.** The message is `C::RANGE_ERROR`, carried out of the fold as `NumberFormatFold::RangeError(&'static str)`; the three dispatch arms are `RangeError(message) => …` and contain no string literal, so there is nothing at the call site to paste. |
| **N5** — bare `u32` Wasm-local-index parameters | `emit_to_length_i64_from_number_payload_local(payload_local, dest_local)`, both `u32`. | **Out of lane.** No compile error is claimed. Carried by the integration note; identical in class to round 1's `IteratorSlot`/`NextMethodSlot`/`DoneSlot` finding. |
| **N6** — a catalog row claiming an implementation the product path never reaches | The three numeric `spec_*` constructors, whose only callers are in `#[cfg(test)]`. | **Measured and recorded** (ledger LN4). No compile error this round: the deletion requires editing an aot-wasm test module. §4.5 specifies the annotation that goes on them. |
| **N7** — landing a type nobody constructs | Adding `Int32`, `Length` or `Index` newtypes to match the symmetry of `Uint32`/`Uint16`. | **Refused structurally.** §2.5 ships them as `const fn` + `const _: () = assert!(...)` instead. Every type this contract *does* define has a named construction site with a line number: `IntegerOrInfinity` at `lowering.rs:34796`, `Uint32` at `lowering.rs:36466`, `Uint16` at `lowering.rs:24658`. §7's acceptance criteria require each to be reachable from the product path. |
| **N7′** — a speculative accessor | Adding `FiniteInteger::value()`, `Uint32::get()`, or `Uint16::get()` "for completeness". | Prohibited by §2.1 and §2.4. Not a compile error; a review criterion, listed in §7. Each such accessor re-opens N3 in one line. |

---

## 4. The retrofit map

Order matters: each step compiles against the previous one, and step 5 is the
only one that changes emitted behaviour.

### 4.1 Order

| Step | What | Files | Emitted bytes change? |
|---|---|---|---|
| 1 | Create `numeric_conversions.rs` with §2.1–2.7 in full. | new file | no (nothing calls it yet) |
| 2 | Declare and re-export the module. | `ir.rs:22-23` region | no |
| 3 | Retrofit `static_clz32`. | `lowering.rs:36466-36473` | **no** — proven identical, §4.3 |
| 4 | Retrofit `static_string_from_char_code_value` and its caller. | `lowering.rs:24658-24664`, `:24478` | **no** — proven identical, §4.4 |
| 5 | Retrofit `static_to_integer_or_zero_expr`, the three `*_call` helpers, the dispatch, and `static_number_to_precision`'s prologue. | `lowering.rs:34713-34801`, `:20693-20775`, `:34951-34953` | **yes, by design** — §4.2 |
| 6 | Annotate the three `spec_*` constructors; add the `NormalResult` const block and the catalog-row comments. | `ir.rs:1099-1129`, `operations.rs:692-694` and tail | no |

Steps 1–4 and 6 are a pure refactor and must produce an **empty rung-G diff**.
Step 5 is a behaviour fix and must not be claimed as a refactor.

### 4.2 Step 5 — the `Number.prototype` folding path

**`static_to_integer_or_zero_expr` (`lowering.rs:34796-34802`) is renamed and
retyped.** Today:

```rust
fn static_to_integer_or_zero_expr(&self, expr: &Expression) -> Option<i32> {
    let value = self.static_to_number_like_expr(expr)?;
    if !value.is_finite() || value == 0.0 || value.is_nan() {
        return Some(0);
    }
    Some(value.trunc() as i32)
}
```

Three defects in six lines, all now unrepresentable: the codomain cannot hold
`±∞`; `!value.is_finite()` is tested *before* the infinity cases are
distinguished, collapsing them; and `value.trunc() as i32` saturates
`truncate(1e300)` to `i32::MAX`. (The third `is_nan()` disjunct is dead — it is
subsumed by `!is_finite()`.) After:

```rust
/// 7.1.5 ToIntegerOrInfinity over a statically-known argument. The `Option`
/// means "not statically decidable", which is this compiler's business;
/// 7.1.5 itself is total, so it is `IntegerOrInfinity`, not
/// `Option<IntegerOrInfinity>`, on the inside.
fn static_to_integer_or_infinity_expr(&self, expr: &Expression) -> Option<IntegerOrInfinity> {
    Some(IntegerOrInfinity::of_number(
        self.static_to_number_like_expr(expr)?,
    ))
}
```

**`static_number_fraction_digits_is_invalid` (`lowering.rs:34757-34776`) is
deleted.** It is the optional-guard mechanism of N4 and it has no remaining
caller once the three sites route through `fold_number_format`. Its `0.0..=100.0`
interval was in any case unusable at the third site (fact F2).

**The three `*_call` helpers return `NumberFormatFold`.**

```rust
// lowering.rs:34713 — 21.1.3.2
fn static_number_to_exponential_call(&self, receiver: &Expression, args: &[Expression])
    -> NumberFormatFold
{
    if args.len() > 1 {
        return NumberFormatFold::NotStatic;
    }
    let Some(value) = self.static_number_to_string_receiver_value(receiver) else {
        return NumberFormatFold::NotStatic;
    };
    // 21.1.3.2 step 12.a distinguishes `fractionDigits === undefined` from
    // `f === 0`, so the `Option` here is spec-shaped, not a convenience.
    let digits = match args.first() {
        Some(arg) if !self.is_static_undefined_expr(Self::unwrap_parenthesized_expr(arg)) => {
            let Some(f) = self.static_to_integer_or_infinity_expr(arg) else {
                return NumberFormatFold::NotStatic;
            };
            f.fraction_digits().map(Some)
        }
        _ => RangeChecked::InBounds(None),
    };
    fold_number_format(
        value,
        digits,
        NonFiniteReceiverOrder::ReceiverFirst,   // step 4 before step 5
        Self::js_number_to_string,
        |value, digits: Option<FractionDigits>| {
            Self::static_number_to_exponential(value, digits.map(FractionDigits::as_usize))
        },
    )
}

// lowering.rs:34736 — 21.1.3.3
fn static_number_to_fixed_call(&self, receiver: &Expression, args: &[Expression])
    -> NumberFormatFold
{
    if args.len() > 1 {
        return NumberFormatFold::NotStatic;
    }
    let Some(value) = self.static_number_to_string_receiver_value(receiver) else {
        return NumberFormatFold::NotStatic;
    };
    let digits = match args.first() {
        Some(arg) if !self.is_static_undefined_expr(Self::unwrap_parenthesized_expr(arg)) => {
            let Some(f) = self.static_to_integer_or_infinity_expr(arg) else {
                return NumberFormatFold::NotStatic;
            };
            f.fraction_digits()
        }
        // step 3: fractionDigits undefined ⇒ f is 0.
        _ => RangeChecked::InBounds(FractionDigits::ZERO),
    };
    fold_number_format(
        value,
        digits,
        NonFiniteReceiverOrder::RangeCheckFirst,  // steps 4-5 before step 6
        Self::js_number_to_string,
        |value, digits: FractionDigits| Self::static_number_to_fixed(value, digits.as_usize()),
    )
}

// lowering.rs:34778 — 21.1.3.5
fn static_number_to_precision_call(&self, receiver: &Expression, args: &[Expression])
    -> NumberFormatFold
{
    if args.len() > 1 {
        return NumberFormatFold::NotStatic;
    }
    let Some(value) = self.static_number_to_string_receiver_value(receiver) else {
        return NumberFormatFold::NotStatic;
    };
    let precision = match args.first() {
        Some(arg) if !self.is_static_undefined_expr(Self::unwrap_parenthesized_expr(arg)) => {
            let Some(p) = self.static_to_integer_or_infinity_expr(arg) else {
                return NumberFormatFold::NotStatic;
            };
            p.precision()
        }
        // step 2: `precision` undefined returns ToString(x) *before* step 3's
        // coercion. Already correct in the current code; preserved verbatim.
        _ => return NumberFormatFold::Formatted(Self::js_number_to_string(value)),
    };
    fold_number_format(
        value,
        precision,
        NonFiniteReceiverOrder::ReceiverFirst,   // step 4 before step 5
        Self::js_number_to_string,
        |value, precision: Precision| Self::static_number_to_precision(value, precision.get()),
    )
}
```

> **Encoder note.** `js_number_to_string` is `fn(value: f64) -> String` — an
> associated function with no `&self` (verified at `lowering.rs:35126`), so
> `Self::js_number_to_string` passes directly as a function item and satisfies
> `impl FnOnce(f64) -> String`. No closure wrapper is needed.

**The dispatch (`lowering.rs:20693-20775`) collapses to three exhaustive
matches.** The two hand-rolled `RuntimeThrow` blocks (`:20699-20709` and
`:20757-20767`) and the receiver-finiteness guards (`:20695-20698`,
`:20753-20756`) are deleted; the `RangeError` arm builds the throw once. The
per-clause message strings are preserved exactly:

- `"Number.prototype.toExponential fraction digits out of range"`
- `"Number.prototype.toFixed fraction digits out of range"`
- new, for `toPrecision`: `"Number.prototype.toPrecision precision out of range"`

Shape at each of the three sites (`toExponential`, `toFixed`, `toPrecision`):

```rust
if field_name == "toExponential" {
    match self.static_number_to_exponential_call(access.target(), args) {
        NumberFormatFold::Formatted(value) => {
            for arg in args {
                self.lower_expression(arg);
            }
            return TypedExpr::from_info(ValueInfo::new(ValueKind::String), ExprIr::String(value));
        }
        NumberFormatFold::RangeError => {
            for arg in args {
                self.lower_expression(arg);
            }
            return TypedExpr::from_info(
                ValueInfo::undefined(),
                ExprIr::RuntimeThrow {
                    name: NativeErrorKind::RangeError,
                    message: "Number.prototype.toExponential fraction digits out of range",
                },
            );
        }
        NumberFormatFold::NotStatic => {}
    }
}
```

**No `_` arm.** The `NotStatic => {}` arm must be spelled out; a catch-all here
would silently absorb a future fourth outcome, which is mistake class N4 again.

**Argument evaluation order is preserved.** Both existing arms lower every
argument before returning, in source order; the retrofit keeps that in both the
`Formatted` and `RangeError` arms. This matters: the spec evaluates the
arguments before ToIntegerOrInfinity can throw, and the fold must not drop an
argument's side effects. This is a real obligation of the retrofit and the
dry-runner should check it at all three sites.

**`static_number_to_precision`'s prologue (`lowering.rs:34951-34953`, owned).**

```rust
-    fn static_number_to_precision(value: f64, precision: i32) -> Option<String> {
-        if !(1..=100).contains(&precision) {
-            return None;
-        }
+    fn static_number_to_precision(value: f64, precision: u8) -> Option<String> {
+        // 21.1.3.5 step 5 is discharged by `Precision`'s constructor before
+        // this is reached, and step 4's non-finite receiver return is
+        // discharged by `NonFiniteReceiverOrder::ReceiverFirst`. Ledger LN3.
+        debug_assert!((1..=100).contains(&precision));
```

Three notes for the encoder, all verified against the body at
`lowering.rs:34954-35046`:

1. The rest of the body needs **no textual edit**. `precision as usize`,
   `precision <= 1`, `precision as usize - 1`, `precision as usize == digits.len()`
   and `match precision { 1 => ..., 2 => ..., 3 => ..., _ => ... }` all typecheck
   at `u8`.
2. The lookup table at `:34991-35040` has element type `(f64, _, &str)`, with the
   middle component inferred from `precision == case_precision` at `:35043`. Its
   literals are `1..=21`, all of which fit `u8`, so the inference simply changes
   and no textual edit is needed. Do not add a type annotation.
3. The `value == f64::INFINITY` / `NEG_INFINITY` arms at `:34955-34960` become
   unreachable, because `ReceiverFirst` returns before calling the formatter.
   They are **outside the owned region**; leave them and record the follow-up in
   the integration note. Do not delete them from this lane.

**The ordering inversion this fixes.** In the current body the `1..=100` range
check precedes the `±Infinity` returns, which inverts 21.1.3.5 steps 4 and 5:
`Infinity.toPrecision(500)` should be `"Infinity"` and the current code declines
the fold instead. The decline is safe, so this is latent, but the order is
wrong. The retrofit removes the inversion by construction rather than by
reordering two `if`s.

### 4.3 Step 3 — `static_clz32`

21.3.2.11 `Math.clz32 ( x )`: `1. Let n be ℝ(? ToUint32(x)). 2. Let p be the
number of leading zero bits in the 32-bit binary representation of n. 3. Return 𝔽(p).`

```rust
-    fn static_clz32(value: f64) -> f64 {
-        if !value.is_finite() || value == 0.0 {
-            return 32.0;
-        }
-        let truncated = value.trunc();
-        let modulo = truncated.rem_euclid(4294967296.0) as u32;
-        modulo.leading_zeros() as f64
-    }
+    /// 21.3.2.11 steps 1-3. The early `!is_finite() || == 0.0` guard is gone:
+    /// 7.1.7 step 2 already sends NaN, ±0 and ±∞ to +0, and
+    /// `0u32.leading_zeros()` is 32, so the guard was a restatement of the
+    /// codomain rather than a substitute for it.
+    fn static_clz32(value: f64) -> f64 {
+        f64::from(Uint32::of_number(value).leading_zeros())
+    }
```

**Byte-identity argument (rung G):** for every `f64`, the old and new bodies
agree. Non-finite and `±0` take the old guard's `32.0`; the new body computes
`Uint32::of_number(x) = Uint32(0)` by 7.1.7 step 2 and
`0u32.leading_zeros() = 32`. Finite non-zero values take the old body's
`trunc().rem_euclid(2^32) as u32`, which is exactly what `residue_of_number`
computes (`|x| < 2^63` via Theorem C, otherwise via the identical `rem_euclid`).
Therefore no fold result changes, and no fixture reaches this path anyway (0 of
532 mention `clz32`). **Rung G must be empty for this step.**

The call site at `lowering.rs:20824` is unchanged.

### 4.4 Step 4 — `static_string_from_char_code_value`

22.1.2.1 `String.fromCharCode ( ...codeUnits )` step 2.a:
`Let nextCU be the code unit whose numeric value is ℝ(? ToUint16(next)).`

```rust
-    fn static_string_from_char_code_value(value: f64) -> Option<String> {
-        if !value.is_finite() {
-            return None;
-        }
-        let unit = value.trunc().rem_euclid(65536.0) as u16;
-        String::from_utf16(&[unit]).ok()
-    }
+    /// 22.1.2.1 step 2.a. The remaining `None` has exactly one cause: a lone
+    /// surrogate, which `String::from_utf16` refuses and Rust's `String`
+    /// cannot hold. That is a limitation of the *fold's* output type, not of
+    /// ToUint16, which is total. The old `!is_finite()` early return moves to
+    /// the caller (see below) because it belongs to the static generator's
+    /// domain, not to 7.1.9. See contract §1.8 C3.
+    fn static_string_from_char_code_value(value: f64) -> Option<String> {
+        String::from_utf16(&[Uint16::of_number(value).code_unit()]).ok()
+    }
```

and at the single call site (`lowering.rs:24478`), inside the
`static_generator_...` enumeration loop, immediately before the call:

```rust
+            // The static generator enumerates a finite arithmetic progression.
+            // A non-finite induction variable means the progression is not one,
+            // so decline the fold here rather than folding `ToUint16(±∞) = 0`
+            // into a `"\0"` element. Hoisted out of
+            // `static_string_from_char_code_value` so that helper can state
+            // 7.1.9's totality. Contract §1.8 C3.
+            if !current.is_finite() {
+                return None;
+            }
             let value = Self::static_string_from_char_code_value(current)?;
```

**Byte-identity argument (rung G):** the decline for non-finite `current` is
preserved verbatim, one call frame higher. For finite `current`, the old and new
bodies compute the same `u16` — the old body's `trunc().rem_euclid(65536.0) as u16`
is exactly `residue_of_number(current, 16)` (Theorems B and C), and neither
`±0` nor NaN reaches the finite path. **Rung G must be empty for this step.**

The hoist is not cosmetic: without it, a `for (let i = Infinity; i > 0; i--)`
generator would enter the `MAX_STATIC_ARRAY_SHAPE_INDEX` loop and iterate
1 000 000 times before declining at `lowering.rs:24474`, turning an immediate
decline into a compile-time stall.

### 4.5 Step 6 — annotations, no deletions

At `ir.rs:1099`, `:1109`, `:1119`, above each of the three constructors:

```rust
    /// **Reachable only from tests as of `091487732`.** Measured: this
    /// constructor has exactly two references workspace-wide besides its own
    /// definition, and both sit inside `#[cfg(test)]` modules — `ir.rs:3856`
    /// (gate at `ir.rs:3451`) and `crates/porffor-aot-wasm/src/lib.rs:1468`
    /// (gate at `lib.rs:109`). AGENTS.md wants unreachable-from-product code to
    /// fail to build; deleting this requires editing an aot-wasm test module,
    /// which is outside this area's lane. Ledger **LN4** in
    /// `docs/rust-rewrite/contracts/numeric-conversion-codomains.md`.
    /// Of the 31 `pub fn spec_*` constructors in this file, 21 have exactly two
    /// references; these three were the ones verified line by line.
```

(with `:3878` / `lib.rs:1486` for `spec_to_length` and `:3900` / `lib.rs:1503`
for `spec_to_index`).

At `operations.rs:692-694`, one comment above the three rows:

```rust
    // 7.1.5's codomain is ℤ ∪ {+∞, −∞}; 7.1.20 and 7.1.22 land in
    // [0, 2^53−1] ∩ ℤ. `normal_result` below reflects exactly that, and a
    // `const _` at the end of this file asserts it. See
    // `docs/rust-rewrite/contracts/numeric-conversion-codomains.md` §2.6.
    ToIntegerOrInfinity => "ToIntegerOrInfinity", SpecOperationIr::ToIntegerOrInfinity;
```

### 4.6 What stays untouched, and why it is said out loud

| Not edited | Reason |
|---|---|
| `crates/porffor-aot-wasm/src/expressions.rs`, `operations.rs`, `builtins/string.rs`, `builtins/array.rs`, `lib.rs` | Out of lane; batch 2 is live in this crate. The five emitters are the integration note's subject. |
| `crates/porffor-ir/src/lib.rs` | Single-lane hub owned entirely by the TDZ area this round. `pub use ir::*;` at `:79` already exposes the new module. |
| `static_number_to_exponential` (`lowering.rs:34824`), `static_number_to_fixed` (`lowering.rs:34921`) | Outside the owned regions. Their `Option<usize>` / `usize` parameters are why `FractionDigits::as_usize` exists. |
| `static_number_to_precision`'s body below `:34953` | Outside the owned region. The `u8` change is an inference change there, not a textual edit (§4.2 note 2). |
| `static_number_expr` (`lowering.rs:35774`), `static_to_number_like_expr` (`:34804`), `static_to_number_expr` (`:34467`) | Outside the owned regions. `static_to_number_like_expr`'s *header line* is at `:34804`, three lines past the owned block's end at `:34801`; do not drift into it. Ledger LN5 records the identifier-shadowing hazard in `static_number_expr`. |
| The `Math.clz32` and `String.fromCharCode` **call sites'** surrounding logic | Only the two lines named in §4.3/§4.4 change. |
| `NormalResult`'s variants (`operations.rs:590-611`) | It is a descriptive tag, not a codomain. Promoting it would be N7. |

### 4.7 Region disjointness

This area's owned regions in `lowering.rs`, in file order — verified against the
TDZ area's regions, whose nearest approach is 146 lines:

```
20693 - 20775   toExponential / toPrecision / toFixed folding dispatch
20824           Math.clz32 fold call site
24478           String.fromCharCode static-generator call site
24658 - 24664   static_string_from_char_code_value
34713 - 34801   static_number_to_*_call, static_number_fraction_digits_is_invalid,
                static_to_integer_or_zero_expr
34951 - 34953   static_number_to_precision prologue
36466 - 36473   static_clz32
```

Two edits fall marginally outside a listed region and are called out so the
integrator can confirm them rather than discover them:

- §4.4 inserts two lines **immediately before** `lowering.rs:24478`. That is
  inside the same statement's enclosing loop and inside no other area's region.
- §4.2 deletes the `RuntimeThrow` blocks inside `20693-20775`; the deletion
  shifts every later line number in the file. **Apply the `lowering.rs` edits in
  descending line order** (36466 → 34951 → 34713 → 24658 → 24478 → 20824 →
  20693) so earlier line numbers stay valid throughout.

---

## 5. Deviations from the area brief, with evidence

### 5.1 `toPrecision`'s missing guard could not have been fixed by calling the existing one

The brief says the guard "is present at two of three sites by convention, which
is the mistake class the codomain closes", implying the fix is to call it at the
third. It is not. `static_number_fraction_digits_is_invalid`
(`lowering.rs:34757-34776`) hard-codes `!(0.0..=100.0).contains(&integer)`, but
21.1.3.5 step 5 is `p < 1 or p > 100`. Calling it at the `toPrecision` site
would have made `(1.5).toPrecision(0)` fold instead of throwing, i.e. it would
have introduced a *new* wrong answer. The interval is clause-specific, which is
why §2.2 defines two newtypes rather than one, and why the guard is deleted
rather than moved.

### 5.2 "Five divergent backend hand-rolls" is five *derivations*, of which two are wrong

All five were read in full. `emit_array_to_uint32_i64_from_number_payload`
(`array.rs:3047`), `emit_to_length_i64_from_number_payload_local`
(`array.rs:1977`) and `emit_to_index_from_number_payload`
(`operations.rs:4112`) were each traced against 7.1.7, 7.1.20 and 7.1.22 and are
**correct** — including the edge cases `ToLength(+∞) = 2^53−1`,
`ToLength(−0) = 0`, `ToIndex(−0.5) = 0`, `ToIndex(−1)` → RangeError and
`ToIndex(2^53)` → RangeError. `emit_to_length_...` uses the trapping
`I64TruncF64U` and `emit_to_index_...` the trapping `I64TruncF64S`, and in both
cases the preceding clamp proves the operand is in range, so neither can trap.
The two defects are `compile_bitwise_number_payload` and
`emit_to_uint32_i64_from_number_payload`. The divergence is in the **five
different spellings of one shared step** (§1.4), which is what makes a single
reference algorithm the right remedy.

### 5.3 The three `Number.prototype` receiver guards are currently **correct**, and the retrofit must not "unify" them

The brief describes the receiver guards as an inconsistency. They are not:
`lowering.rs:20696` requires `value.is_finite()` for `toExponential` and
`:20754` requires only `.is_some()` for `toFixed`, and that difference is
exactly 21.1.3.2 step 4 sitting *before* step 5 while 21.1.3.3 steps 4–5 sit
*before* step 6 (§1.7). Both are right today, **by convention rather than by
construction** — which is why the retrofit replaces them with an explicit
`NonFiniteReceiverOrder` argument instead of deleting one of them. An encoder
who read the guards as a bug and unified them would break `Infinity.toFixed(101)`.

### 5.4 `static_string_from_char_code_value`'s `None` has two causes, and only one is legitimate

The brief asks whether the non-finite decline "is the codomain's business or the
caller's". Answer: the caller's — but the decision hinges on a fact the brief
does not mention. The helper's `None` today has **two** causes:
`!value.is_finite()`, and `String::from_utf16` refusing a lone surrogate. The
second is real and must stay: `String.fromCharCode(0xD800)` is a valid JS string
that Rust's `String` cannot hold, and five CLI fixtures depend on that decline
(`wasm_uri_codecs_core.js`, `wasm_regexp_exec_word_escape_program.js`,
`wasm_string_code_point_at_surrogates.js`, `wasm_regexp_escape_surrogates.js`,
`wasm_bigint_number_relational.js`). §4.4 keeps the surrogate decline in the
helper, moves the finiteness decline to the caller, and makes `Uint16::of_number`
total. A retrofit that removed *both* would change five fixtures.

### 5.5 `IntegerOrInfinity`'s defect is worse than "returns `Some(0)` for non-finite"

The brief names `Some(0)` for non-finite input. The body has two further
problems, both visible at `lowering.rs:34798-34801`: the `is_nan()` disjunct is
dead (subsumed by `!is_finite()`), and `value.trunc() as i32` **saturates**, so
`truncate(1e300)` becomes `i32::MAX`. The saturation is latent for the same
reason the rest is — `i32::MAX` is outside `0..=100` and `1..=100` — but it is
the same mechanism as backend defect N1, inside `porffor-ir`, and it is the
reason the codomain must not be `i32` rather than merely must include `±∞`.

### 5.6 The brief's "latent, not shipped" framing is confirmed, with one addition

Every wrong value `static_to_integer_or_zero_expr` can produce is filtered by a
downstream range check before it reaches an emitted program: `0..=100` at
`lowering.rs:34728` and `:34749`, and `1..=100` at `:34952`. Verified by
enumerating the reachable wrong values: `Some(0)` for `±∞`, and `i32::MAX` /
`i32::MIN` for `|truncate(x)| ≥ 2^31`. None is in `0..=100` or `1..=100` except
`Some(0)`, which is in `0..=100` — so `(1.5).toFixed(Infinity)` *would* fold to
`"2"` were it not for `static_number_fraction_digits_is_invalid` at
`lowering.rs:20755`, and `(1.5).toExponential(Infinity)` likewise via `:20697`.
**The `toFixed` and `toExponential` paths are protected by the guard; the
`toPrecision` path is protected only by `1..=100` excluding 0.** Three
independent accidents, none of them a type. That is the precise sense in which
this is latent, and it is what §7 requires the dry-runner to re-derive.

**Two corrections the dry run forced, both of which this enumeration missed.**

1. The enumeration is over the values `static_to_integer_or_zero_expr` could
   produce for a *correctly resolved* argument. It does not consider an argument
   whose **identifier is shadowed**, and that is where the latency broke:
   `function f(){ let Infinity = 5; return (1.5).toPrecision(Infinity); }` used
   to produce `Some(0)` → the old `!(1..=100)` guard → decline → the correct
   runtime answer `"1.5000"`. With the new step 5 arm it folded to
   `PositiveInfinity` → `RangeError`. Ledger **LN5**, now closed by
   `identifier_resolves_to_builtin_global`.
2. It does not consider a **receiver that should have thrown**.
   `static_number_to_string_receiver_value` maps `Number.prototype` to
   `Some(0.0)` (`lowering.rs:35071`), an ES5 fossil: since ES2015
   `Number.prototype` is an ordinary object with no `[[NumberData]]`, so
   21.1.3.5 step 1's `ThisNumberValue` throws a **TypeError** before step 3 is
   reached. The mapping is pre-existing and already mis-folds
   `Number.prototype.toFixed(101)` to a RangeError and
   `Number.prototype.toFixed(1)` to `"0.0"`; this lane adds a third site for it,
   `Number.prototype.toPrecision(0)` → RangeError where the spec says TypeError.
   The line is outside this area's owned regions, so it is handed on as a
   follow-up in the integration note beside F5/F7 rather than deleted here. The
   fix is to delete the `"prototype" => Some(0.0)` arm or gate it behind a
   `ThisNumberValue`-shaped predicate.

---

---

## 5b. DISCREPANCY-FIXER RECORD

Written blind (no `cargo`/`rustc`; the integrator owns the compile gate). Files
touched: `crates/porffor-ir/src/numeric_conversions.rs`,
`crates/porffor-ir/src/lowering.rs`, `crates/porffor-ir/src/ir.rs`.

### Fixed in code

| Severity | What | How |
|---|---|---|
| **blocker** | The new 21.1.3.5 RangeError arm turned ledger **LN5**'s latent identifier-shadowing hazard into a shipped wrong answer: `function f(){ let Infinity = 5; return (1.5).toPrecision(Infinity); }` emitted a RangeError where the correct answer is `"1.5000"`. Four resolvers tested spelling with no scope lookup, not the one LN5 named. | New `ScriptLowerer::identifier_resolves_to_builtin_global` (9.1.1's four clauses, lifted from the two pre-existing intrinsic guards, which now call it) and `static_global_number_identifier` on top of it. `static_to_number_expr`, `static_number_to_string_receiver_value`, `static_number_expr` and `is_static_undefined_expr` all route through it. |
| **bug** | **N4″** conceded uncloseable; and the RangeError message was an open `&'static str` domain at three dispatch sites (**N4‴**). | `trait NumberFormatClause { type Digits; const ORDER; const RANGE_ERROR; }` with `ToExponential` / `ToFixed` / `ToPrecision` marker types; `fold_number_format::<C>` loses its `order` parameter; `NumberFormatFold::RangeError` carries `C::RANGE_ERROR`. Wrong clause is `E0308`; wrong message is unspellable. |
| **bug** | `#[derive(PartialOrd)]` on `FiniteInteger` gave a full ordered-comparison escape hatch to every downstream crate, contradicting §2.1's core claim. | Derive deleted. |
| **bug** | `residue_pow2_i64`'s `bits: u32` admitted `0` and `64` (silently returning 0 in release), and the modulus was decoupled from the carrier at both `of_number` sites. | `ResidueWidth` (2 variants, `const fn mask`) + `ResidueCarrier` (`const WIDTH`, `of_residue`), so `residue_of_number::<u32>` / `::<u16>` derive their own modulus. `debug_assert!` deleted along with the states it guarded. |

`ir.rs`'s `pub use` list is now **25** names, counted.

### Corrected in this document, not in the code

- §2.1's "the only ways out are `fraction_digits()` and `precision()`" (was
  false while `PartialOrd` was derived), §2.3's N4″ concession, §2.5's
  `debug_assert!`-is-a-const-no-op claim, §3's N4″ row, ledger **LN1**'s
  "no type can carry this" and ledger **LN5** in full.
- §5.6's "three independent accidents" enumeration missed two cases: a shadowed
  identifier, and a receiver that should have thrown.
- §6.1's "currently failing" was derived, not measured; re-stated with the
  snapshot measurement.

### Handed to the integration note, not fixed here

- `static_number_to_string_receiver_value`'s `"prototype" => Some(0.0)`
  (`lowering.rs:35071`) — an ES5 fossil that makes `Number.prototype.toPrecision(0)`
  a RangeError where 21.1.3.5 step 1 requires a TypeError. Outside this area's
  owned regions; pre-existing at two other sites; this lane adds a third.
- `MAX_SAFE_INTEGER_U64` (`numeric_conversions.rs`) versus
  `porffor-aot-wasm/src/heap.rs:10`'s `MAX_SAFE_INTEGER` — two workspace
  spellings of 2^53−1, this area's own duplication class one crate over. Both
  are correct today, so this is a tidy-up, not a defect. Measured consumers that
  one `pub(crate) use` in `heap.rs` would tie: `array.rs:1995`, `:1999`,
  `:8446`, `:8613`, `:18967`, `:18978`; `standard.rs:21916`, `:21929`,
  `:22384`, `:22395`; `collections.rs:1664`; `array_from_async.rs:1232`; plus
  four inline literals at `array.rs:20963`, `standard.rs:42213` and
  `operations.rs:4098`/`:4105`.
- **LN1 is reducible.** The `|x| ≥ 2^63` branch can be computed by integer
  bit-extraction from `x.to_bits()` instead of `f64::rem_euclid`, which would
  make `residue_of_number` fully `const fn` and retire the row. Not taken;
  recorded so the choice is visible rather than implied.

## 6. Dry-run corpus, with the traces the dry-runner must reproduce

Each entry names the file, the step being traced, and the expected verdict
**before** and **after** the retrofit. All nine test262 paths were confirmed to
exist at the current pin.

### 6.1 `test/language/expressions/left-shift/S9.5_A1_T1.js` — N1, headline

Asserts `(Number.POSITIVE_INFINITY << 0) === +0`. Trace
`expressions.rs:1701-1713` symbolically: `F64ReinterpretI64` → `+∞`;
`I64TruncSatF64S` → `i64::MAX`; `I32WrapI64` → `0xFFFFFFFF`; shift count
`0 & 0x1f = 0`; `I32Shl` → `0xFFFFFFFF`; `I64ExtendI32S`, `F64ConvertI64S` →
`-1.0`. 7.1.6 step 2 requires `+0`. **Derived from the emitted instruction
sequence; no pinned wasm-aot artifact covers this path.** Out of lane; fixed by
the integration note's retrofit. The file also asserts `NaN << 0` and `-0 << 0`,
both of which pass today by accident (§1.5).

> The earlier wording, "currently failing", was a derived verdict presented as a
> measurement. Measured instead (`test262/snapshots/*.json`, excluding the
> per-case files): **7** snapshots list
> `language/expressions/left-shift/S9.5_A1_T1.js` and/or
> `.../unsigned-right-shift/S9.6_A1.js` in `completed_paths` — four 45-case leaf
> snapshots, two 11038-case runs and one 23643-case run — **and every one of
> them carries `"execution_backend": "spec-exec"`**; neither path appears in any
> `failures` list. The only `wasm-aot` snapshots in the tree are eight
> `matrix-cache-wasm-aot-*.json` with empty `completed_paths`. `porffor-spec-exec`
> contains no numeric conversion at all — `execute_script` (`lib.rs:358`) calls
> `boa_engine`'s `Context::eval` — so those green counts say nothing about
> `compile_bitwise_number_payload`. The N1 verdict stands on the symbolic trace,
> which is sound and needs no run; but the F1 retrofit's acceptance gate must
> not be mistaken for a regression check that already exists, because it does
> not.

### 6.2 `.../left-shift/S9.5_A2.1_T1.js` — N1 magnitude class

"ToInt32 returns values between −2^31 and 2^31−1 ... numbers in and outside of
Int32 scope." Confirm the defect window is exactly `|x| ≥ 2^63` (§1.5 claims
N1a/N1b/N1c) rather than "outside Int32 scope": values in
`[2^31, 2^63)` are handled correctly today.

### 6.3 `.../left-shift/S9.5_A3.2_T1.js` — N1/N2, the exactness requirement

"Operator uses floor, abs". This is the requirement that separates
`array.rs:3047` from the other two. Trace `array.rs:3047`'s
`n − floor(n / 2^32) · 2^32` against Theorem B and confirm the two compute the
same function on finite inputs.

### 6.4 `test/language/expressions/unsigned-right-shift/S9.6_A1.js` — N1 under 7.1.7

Same residue, unsigned reading. Confirms the defect is in the shared truncation
(steps 2–4) and not in step 5's signed/unsigned split — which is precisely what
the `reference_to_int32 as u32 == reference_to_uint32` const assertion (§2.7)
encodes.

### 6.5 `test/built-ins/Math/clz32/int32bit.js` — the reference implementation

Twenty assertions. Trace the **new** `static_clz32` through
`Uint32::of_number` for `4294967295`, `4294967296`, `4294967297`, `-4294967295`,
`-4294967296`, `-4294967297` and confirm `0, 32, 31, 31, 32, 0`. All six are in
the `|x| < 2^63` window, so they route through `residue_pow2_i64` and are
covered by the const table. **Passing today; must still pass. This is the
byte-identity check for §4.3.**

Note the file's `info` block cites "7.1.6 ToUint32" — that is the ES2016
numbering. The current section is 7.1.7. The step text quoted is identical.

### 6.6 `test/built-ins/Math/clz32/infinity.js` — N3, guard vs codomain

`Math.clz32(±Infinity) === 32`. **The trace's job is to establish that the old
early guard and 7.1.7 step 2 agree by construction, not by coincidence**, so
deleting the guard is safe. Old: guard returns `32.0` directly. New:
`Uint32::of_number(±∞)` = `Uint32(0)` by step 2, `0u32.leading_zeros()` = 32.
Same value, and now for the spec's reason. Confirm the fold fires at all:
`static_number_expr` (`lowering.rs:35785-35786`) resolves the identifier
`Infinity` to `f64::INFINITY`, so it does (subject to ledger LN5).

### 6.7 `test/built-ins/Number/prototype/toPrecision/tointeger-precision.js` — N3/N4

Drives the one folding path with no guard. Five assertions, all of which fold to
in-range precisions today and must continue to:
`(123.456).toPrecision(1.1)` → `"1e+2"` (7.1.5 step 5 truncates 1.1 to 1),
`(1.9)` → `"1e+2"`, `(true)` → `"1e+2"`, `("2")` → `"1.2e+2"`,
`([2])` → `"1.2e+2"`. Trace each through
`static_to_number_like_expr` → `IntegerOrInfinity::of_number` →
`precision()` → `Precision(1)` or `Precision(2)` → `static_number_to_precision`,
and confirm the values still come out of the `(123.456, 1, "1e+2")` and
`(123.456, 2, "1.2e+2")` rows of the lookup table at `lowering.rs:35008-35009`.
**This is the regression risk of §4.2 and the reason the `u8` inference note
exists.**

### 6.8 `test/built-ins/String/fromCharCode/S9.7_A2.1.js` — N3 for ToUint16

Seven checks, of which CHECK#3 (`-1` → `65535`), CHECK#6 (`65536` → `0`) and
CHECK#7 (`4294967295` → `65535`) are the residue cases. All three are rows of
the §2.7 const table. Note that this file exercises `String.fromCharCode`
directly, not the static-generator fold at `lowering.rs:24478`, so it runs on
the backend path; the trace's purpose is to fix the expected residues, and the
in-crate check is that `Uint16::of_number` reproduces them.

### 6.9 `.../split/separator-number-limit-math-pow-2-32-1-instance-is-number.js` — N2 boundary

`limit = 2^32 − 1`, where the correct and the incorrect ToUint32 still agree
(both yield `4294967295`, §1.6 table). The boundary case that must not regress
when `string.rs:15052` adopts `array.rs:3047`'s algorithm. Expected result is
9 elements.

### 6.10 Adversarial — N1, the four that are wrong and the one that is lucky

| Input | 7.1.6 | Today | Note |
|---|---|---|---|
| `Infinity\|0` | `0` | `-1` | wrong |
| `(2**63)\|0` | `0` | `-1` | wrong |
| `(1e300)\|0` | `0` | `-1` | wrong |
| `(-1e300)\|0` | `0` | `0` | right by accident (`ulp ≥ 2^32`) |
| `NaN\|0` | `0` | `0` | right by accident (`I64TruncSatF64S(NaN) = 0`) |

Establishes that the current pass on `NaN` is luck, not coverage.

### 6.11 Adversarial — N2, the three-input split probe

`"a,b,c".split(",", 2**64)` must be `[]`; `"a,b,c".split(",", -1)` must be
`["a","b","c"]`; `"a,b,c".split(",", 2**32)` must be `[]`. Today the first
returns three elements. **This is the acceptance gate for the `string.rs`
retrofit** and is restated in the integration note.

### 6.12 Adversarial — N3/N4 against all three in-crate folds at once

| Input | Spec | Today | After |
|---|---|---|---|
| `(1.5).toPrecision(Infinity)` | RangeError (`p = +∞ > 100`) | fold declines → runtime | `NumberFormatFold::RangeError` → folded throw |
| `(1.5).toPrecision(NaN)` | RangeError (`p = +0 < 1`) | fold declines → runtime | `NumberFormatFold::RangeError` → folded throw |
| `(1.5).toPrecision(0)` | RangeError | fold declines → runtime | `RangeError` |
| `Infinity.toPrecision(500)` | `"Infinity"` (step 4 wins) | fold declines → runtime | `Formatted("Infinity")` |
| `Infinity.toFixed(101)` | **RangeError** (steps 4–5 win) | folded throw, correct | `RangeError`, unchanged |
| `Infinity.toExponential(101)` | `"Infinity"` (step 4 wins) | fold declines (receiver guard) → runtime | `Formatted("Infinity")` |
| `(1.5).toFixed(Infinity)` | RangeError | folded throw, correct | `RangeError`, unchanged |
| `Math.clz32(2**32)` | `32` | `32` | `32`, unchanged |
| `Math.clz32(2**32 + 1)` | `31` | `31` | `31`, unchanged |
| `String.fromCharCode(65536 + 65)` | `"A"` | `"A"` | `"A"`, unchanged |

The first two rows are the pair `static_to_integer_or_zero_expr` cannot tell
apart today (both become `Some(0)`); after the retrofit they are
`PositiveInfinity` and `Finite(0)`, reaching `RangeChecked::RangeError` by two
different arms of `precision()`. Rows 4 and 6 are the ordering fix: the fold now
produces the spec's answer where it previously declined.

---

## 7. Acceptance criteria

The encoder is done when all of the following hold. The dry-runner checks each
against the code as written, not against this document.

1. `cargo check -p porffor-ir` is clean. A failure inside a `const _: () = { ... }`
   block is a **content** error in §2.7's table, not a syntax problem — fix the
   algorithm, never the table.
2. Every type defined in `numeric_conversions.rs` has a **named construction
   site on the product path**:
   - `IntegerOrInfinity` ← `static_to_integer_or_infinity_expr`, `lowering.rs:34796`,
     live from `:20712`, `:20740` and `:20768`.
   - `Uint32` ← `static_clz32`, `lowering.rs:36466`, live from `:20824`.
   - `Uint16` ← `static_string_from_char_code_value`, `lowering.rs:24658`, live
     from `:24478`.
   - `FractionDigits`, `Precision`, `RangeChecked`, `NumberFormatFold`,
     `NonFiniteReceiverOrder` ← the three `*_call` helpers.
   - `ExtendedInteger`, `ToIndexOutcome`, `reference_to_*`, `residue_pow2_i64`
     ← the const blocks in §2.7 (build-time), and `residue_pow2_i64`,
     `reference_to_uint32`, `reference_to_uint16` additionally from
     `residue_of_number` at run time.
   A type with neither is deleted, not documented. This is mistake class N7.
3. `numeric_conversions.rs` contains **no** `FiniteInteger::value`,
   `Uint32::get`, `Uint16::get`, `From`/`Into`/`Deref`/`AsRef` impl, or public
   tuple field on `IntegerOrInfinity`, `FiniteInteger`, `Uint32`, `Uint16`,
   `FractionDigits` or `Precision`. `ExtendedInteger::Finite(i64)` is the one
   deliberately public payload and §2.5 says why.
4. Every `match` over `IntegerOrInfinity`, `RangeChecked`, `NumberFormatFold`,
   `NonFiniteReceiverOrder`, `ExtendedInteger` and `ToIndexOutcome` in the
   workspace is exhaustive with **no `_` arm**. `grep -n "_ =>"` over the new
   file returns nothing.
5. `static_number_fraction_digits_is_invalid` no longer exists, and
   `grep -rn "static_number_fraction_digits_is_invalid" crates/` is empty.
6. `grep -rn "trunc() as i32" crates/porffor-ir/src/` is empty.
7. `grep -rn "rem_euclid" crates/porffor-ir/src/` returns exactly **one** line,
   inside `residue_of_number`. The two sites at `lowering.rs:24662` and
   `:36471` are gone, replaced by calls.
8. The ledger has exactly the five rows in §2.8, and `numeric_conversions.rs`
   contains exactly one `#[cfg(test)]` test — the **LN1** boundary differential.
   Any further test in this file is a sign an invariant was left to a test that
   a type could have carried.
9. Rung G is **empty** after steps 1–4 and 6. It is also expected empty after
   step 5, but for a different reason — 0 of 532 fixtures exercise
   `toPrecision`, `toFixed` or `toExponential` — so step 5 must **not** be
   reported as byte-identity-verified. It is a behaviour change with no fixture
   coverage, and the integration note says so.
10. The `toPrecision` RangeError message string is new and does not collide:
    `grep -rn "toPrecision precision out of range" crates/` returns exactly the
    one site — which is now `ToPrecision::RANGE_ERROR` in
    `numeric_conversions.rs`, not a literal at the dispatch arm. The three
    dispatch arms in `lowering.rs` contain **no** string literal at all; that is
    the check for N4‴.
12. `grep -rn "fold_number_format" crates/` shows every call carrying an
    explicit clause turbofish (`::<ToExponential>`, `::<ToFixed>`,
    `::<ToPrecision>`) and **no** `NonFiniteReceiverOrder` argument. An
    order argument reappearing is N4″ reopening.
13. `grep -rn "residue_of_number(" crates/porffor-ir/src/` shows only
    `::<u32>` and `::<u16>` calls, and `residue_pow2_i64` takes a
    `ResidueWidth`. An integer `bits` parameter reappearing is I10b reopening.
14. `grep -rn "PartialOrd" crates/porffor-ir/src/numeric_conversions.rs` is
    empty. Re-deriving it on `FiniteInteger` reopens N3 from outside the crate
    (§2.1).
15. Every identifier-spelling resolver in `lowering.rs` that maps `Infinity`,
    `NaN` or `undefined` to a value calls
    `identifier_resolves_to_builtin_global`. Measured today: four of them, plus
    the two pre-existing intrinsic guards that were rewritten onto the same
    predicate. A fifth added without it is ledger **LN5**.
11. `target/lane-notes/numeric-conversion-codomains-theory-integration.md`
    exists and names: the seven owned `lowering.rs` regions, the five backend
    functions with their verdicts, `emit_array_to_uint32_i64_from_number_payload`
    as the reference, §6.11 as the retrofit's acceptance gate, and the three
    follow-ups this lane could not take (LN2, LN4, and the dead `±Infinity`
    arms at `lowering.rs:34955-34960`).
