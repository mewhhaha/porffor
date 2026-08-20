# Contract: Numeric conversion codomains — pointer

The contract for the area *Numeric conversion codomains: ToIntegerOrInfinity /
ToUint32 / ToUint16 as closed types in `lila-ir`, with one const reference
algorithm for the five divergent backend hand-rolls* lives at:

`docs/rust-rewrite/contracts/Numeric conversion codomains - ToIntegerOrInfinity, ToUint32, ToUint16 as closed types in lila-ir, with one const reference algorithm for the five divergent backend hand-rolls.md`

This file exists for two reasons. First, the area brief names
`numeric-conversion-codomains.md` in its `files_owned` list while the campaign's
file-naming convention for contracts uses the full area title — the same split
that produced `reference-records.md` and `spec-operations.md`. Second, the area
title contains `/` characters, which cannot appear in a filename; the title was
rendered with `, ` in their place and is otherwise verbatim. There is one
contract; that file is it.

**Read §5 (Deviations from the area brief) before implementing anything.** Six
points in the brief would produce a defect, a decoration, or a regression if
followed verbatim:

- **§5.1** — the `toPrecision` fold cannot be fixed by calling the existing
  `static_number_fraction_digits_is_invalid` at the third site. That predicate
  hard-codes `[0, 100]`; 21.1.3.5 step 5 is `[1, 100]`. Calling it there would
  make `(1.5).toPrecision(0)` fold instead of throwing. The guard is **deleted**,
  and the two intervals become two newtypes.
- **§5.2** — "five divergent backend hand-rolls" is five *derivations of one
  shared step*, of which **two** are wrong. `array.rs:3047` (ToUint32),
  `array.rs:1977` (ToLength) and `operations.rs:4112` (ToIndex) were each traced
  in full and are **correct**. Do not "fix" them.
- **§5.3** — the three `Number.prototype` receiver guards are **correct today**
  and differ for a spec reason (21.1.3.2 step 4 before step 5, versus 21.1.3.3
  steps 4–5 before step 6). Unifying them breaks `Infinity.toFixed(101)`.
- **§5.4** — `static_string_from_char_code_value`'s `None` has two causes, and
  the lone-surrogate one must stay: five CLI fixtures depend on it.
- **§5.5** — `static_to_integer_or_zero_expr` has three defects, not one. The
  `value.trunc() as i32` saturation is the same mechanism as the backend's N1,
  inside `lila-ir`.
- **§5.6** — the "latent, not shipped" claim is confirmed, and the three
  independent accidents that make it latent are enumerated so the claim is not
  taken on trust.

Structural summary of what lands:

| Construct | Carries | Constructed at |
|---|---|---|
| `IntegerOrInfinity` (3-variant enum, private `FiniteInteger` payload) | 7.1.5's codomain ℤ ∪ {±∞} | `lowering.rs:34796` |
| `Uint32`, `Uint16` (private fields, one validating constructor each) | 7.1.7 / 7.1.9 residues | `lowering.rs:36466`, `:24658` |
| `RangeChecked<T>`, `FractionDigits`, `Precision` | 21.1.3.x range checks, on the extended integers | the three `*_call` helpers |
| `NonFiniteReceiverOrder`, `NumberFormatFold` | the ordering obligation and the RangeError branch | the folding dispatch, `lowering.rs:20693-20775` |
| `residue_pow2_i64`, `reference_to_{int32,uint32,uint16,length,index}` + `const _: () = assert!` tables | the normative algorithm the five backend emitters must match | build time, plus `residue_of_number` at run time |

`ToInt32`, `ToLength` and `ToIndex` deliberately get **no** newtype (§1.8 C4):
there is no construction site for them in `lila-ir`, and round 1 deleted two
type pairs for exactly that.

## Integrator amendment: one exact runtime residue boundary

The backend must not convert a binary64 operand to `i64` before applying the
modulo-2^32 step. `i64.trunc_sat_f64_s` destroys the residue for finite values
outside the signed-64-bit interval and maps positive infinity to `i64::MAX`.
That makes expressions such as `Infinity | 0`, `(2 ** 63) | 0`, and
`1 << Infinity` observably wrong.

One `FunctionBuilder::emit_to_uint32_i64_from_number_payload` implementation is
therefore authoritative for Array length conversion, String split limits,
`String.fromCharCode`'s low-16-bit projection, and the Number arms of unary and
binary bitwise operators. It computes

`trunc(n) - floor(trunc(n) / 2^32) * 2^32`

in binary64 before the final unsigned conversion. The intermediate is always
in `[0, 2^32)` for finite input; NaN and either infinity naturally produce NaN
at the subtraction and Wasm's saturating unsigned conversion maps that to zero,
as ECMA-262 requires. The power-of-two division is exact, and values whose
binary64 spacing is at least `2^32` are already multiples of the modulus.

No codomain wrapper is added at the Wasm-local boundary: both payloads and
locals are encoder indices, so wrapping an arbitrary `u32` at each call site
would not validate anything. The invariant is instead the single consumed
emitter plus removal of the two family-local copies and of every
`i64.trunc_sat_f64_s` conversion in the binary bitwise Number path.

Unary complement is not represented as a binary XOR with a Number `-1`:
that spelling changes `~1n` into a mixed-numeric TypeError. A dedicated
`UnaryBitwiseOp::Complement` keeps one evaluated operand across one ToNumeric
boundary, then an exhaustive Number/BigInt match selects this shared residue
emitter or the exact arbitrary-precision BigInt XOR-with-`-1n` operation.

`String.fromCharCode` stores the converted Number payload, consumes this same
modulo-2^32 emitter, and only then masks to 16 bits. Its former
`i64.trunc_sat_f64_s & 0xffff` sequence mapped `Infinity` and large finite
inputs to the signed-64-bit endpoints before the modulus and was observably
wrong.

The out-of-lane half — the two defective backend emitters, the acceptance gate,
and the three follow-ups this lane could not take — is in
`target/lane-notes/numeric-conversion-codomains-theory-integration.md`.

---

## Encoder amendments (round 2, after implementation)

The contract was implemented in full. Six points needed amending; none changes
what the types are, and none was discovered by a test.

1. **`fold_number_format` is missing from §2's `pub use` list.** Mechanical
   omission — `lowering.rs` cannot name the ordering driver without it. The
   list that landed has **18** names.
2. **`numeric_conversions.rs` does not begin with `use super::*;`** (§2). The
   module uses nothing from the crate root and an unused glob import is a
   rustc warning; `use super::*;` sits in the `#[cfg(test)]` module instead.
3. **§4.2's three inlined `ExprIr::RuntimeThrow` blocks became one builder**,
   `static_number_format_range_error(&'static str) -> TypedExpr`, in the owned
   region. §4.2's own prose already asked for this ("the `RangeError` arm
   builds the throw once"); only its sample code inlined it. Three copies of a
   five-line literal differing by one string is the same by-convention shape
   this area exists to remove.
4. **§7.10 is false as written, in our favour.**
   `"Number.prototype.toPrecision precision out of range"` is **not** a new
   string: it is already what the runtime path throws
   (`lila-aot-wasm/src/data.rs:1470`, `src/operations.rs:9236`, `:9251`,
   `:9274`). The grep returns 5 source sites. The fold and the runtime now
   throw textually identical messages, which is strictly better than the
   criterion assumed. Read §7.10 as "matches the runtime message exactly".
5. **§7.7 needs a qualifier.** `grep -rn "rem_euclid" crates/lila-ir/src/`
   returns 7 lines; exactly **one** is code (`numeric_conversions.rs`, inside
   `residue_of_number`). The rest name the operation in doc comments.
6. **The LN1 test also pins `IntegerOrInfinity::of_number`'s five arms.** §3's
   N3′ row already assigns that check to the LN1 differential, so this is
   compliance, and it stays inside the single `#[test]` so §7.8 holds.

**Ledger unchanged: five rows.** Nothing moved from the compile-error column
into it. The two classes that are *not* compile errors — N3′ (a wrong function
body, which no type can prevent) and the cross-crate half of N1 (LN2) — were
already declared as such in §3 and §2.8, with reasons, and are checked by the
LN1 differential and by the integration note's acceptance gate respectively.

**Not run:** `cargo check`. The campaign's hard constraint hands the compile
gate to the integrator; the residual blind-write risks are enumerated in
`target/lane-notes/numeric-conversion-codomains-theory-integration.md` §11, and
the encoder's self-review is §13 of the same file.

---

## Integrator amendment (round 3): `FiniteReceiver`, and the closure of F5

Applied at the compile gate, with the reasoning recorded here because it adds a
type the contract did not name.

**What F5 asked for.** The integration note's follow-up F5 asked the integrator
to delete the `value == f64::INFINITY` / `NEG_INFINITY` arms of
`static_number_to_precision`, which `NonFiniteReceiverOrder::ReceiverFirst`
had made unreachable. Re-measuring at the gate found the same dead handling in
**all three** formatters, not one: `static_number_to_exponential` carries the
same two arms, and `static_number_to_fixed` carries an `is_nan()` arm.
`fold_number_format` calls `finish` only after `receiver.is_finite()` in *both*
`NonFiniteReceiverOrder` branches, so every formatter is dead on the non-finite
path regardless of clause. F5's count was one; the measured count is three.

**Why the deletion alone was not enough.** A plain deletion leaves three private
`fn(f64, …) -> Option<String>` formatters that a new call site can hand
`f64::INFINITY` to. That is not hypothetical: this area's own history is the
`toPrecision` range check sitting *above* the `±Infinity` arms and thereby
inverting 21.1.3.5 steps 4 and 5. The dangerous direction is `ToFixed`, whose
`RangeCheckFirst` ordering means a formatter that answers `"NaN"` for
`NaN.toFixed(101)` has silently overruled a required RangeError.

**The type.** `FiniteReceiver(f64)` in `numeric_conversions.rs`: private field,
single private constructor `assume_finite`, single reader `get`. Its only
producer is the branch of `fold_number_format` that has just tested
`receiver.is_finite()`; its only consumers are the three formatters, whose
signatures now take it. The `pub use` list in `ir.rs` grows from 25 names to
**26** — the count in that doc comment is load-bearing and was updated with it.

**It meets AGENTS.md's test, and the check was performed, not assumed.** Both
perturbations were compiled at the gate and reverted:

| Perturbation | Result |
|---|---|
| `Self::static_number_to_precision(f64::INFINITY, 3)` | `E0308`: expected `FiniteReceiver`, found `f64` |
| `Self::static_number_to_precision(FiniteReceiver(f64::INFINITY), 3)` | `E0423`: cannot initialize a tuple struct which contains private fields |

**What it does not do, stated plainly.** It does not make an `== f64::INFINITY`
arm *inside* a formatter body a compile error — `get()` returns a bare `f64`,
because formatting is real floating-point arithmetic and the value has to come
back out. The newtype buys the **boundary**, not the body. Ledger row **LN6**
below records the residue.

### Ledger row added

- **LN6 — `FiniteReceiver::assume_finite` is checked at run time, not by a
  type.** Inside `numeric_conversions.rs` the constructor is in scope, so
  `fold_number_format` could in principle build one on the non-finite branch.
  The guard is a `debug_assert!(receiver.is_finite())` plus the fact that the
  producer is a single 30-line function with both call sites visible in one
  screen. Cross-module the invariant is a type; in-module it is an assertion.
  Making it a type as well would need a checked constructor returning
  `Option<Self>`, which re-opens the question `fold_number_format` exists to
  answer once — so this row is accepted, not scheduled.

**Ledger: six rows** (LN1–LN5 unchanged, LN6 new). F5 is **closed**, and closed
wider than it was written.
