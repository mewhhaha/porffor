//! Numeric conversion codomains (ECMA-262 5.2.5, 7.1.5, 7.1.6, 7.1.7, 7.1.9,
//! 7.1.20, 7.1.22), reified as the closed sets the spec actually names.
//!
//! See `docs/rust-rewrite/contracts/numeric-conversion-codomains.md`. The short
//! version:
//!
//! - 7.1.5 ToIntegerOrInfinity is a **total** function into **ℤ ∪ {+∞, −∞}**.
//!   That set is not `i32`, not `i64`, and not `Option<i32>` — an `Option`'s
//!   `None` means "no answer" where 7.1.5 always has one, and every integer
//!   carrier this crate reached for silently mapped `+∞` onto `0` and
//!   `truncate(1e300)` onto `i32::MAX`. [`IntegerOrInfinity`] is a three-variant
//!   enum whose finite payload is a private [`FiniteInteger`], so the enum has
//!   exactly one constructor workspace-wide.
//! - The comparisons in 21.1.3.2 step 5, 21.1.3.3 steps 4–5 and 21.1.3.5 step 5
//!   are evaluated **on the extended integers** (`+∞ > 100`, `−∞ < 0`). There is
//!   deliberately no accessor returning the raw number, no `PartialOrd<i32>` and
//!   no numeric cast: the only exits from an [`IntegerOrInfinity`] are
//!   [`IntegerOrInfinity::fraction_digits`] and [`IntegerOrInfinity::precision`].
//! - `[0, 100]` (21.1.3.2 / 21.1.3.3) and `[1, 100]` (21.1.3.5) are **different
//!   intervals belonging to different clauses**, so they are two newtypes
//!   ([`FractionDigits`], [`Precision`]) and not one `u8`. Mixing them is
//!   `E0308`. The tree's shared `0..=100` predicate was unusable at the third
//!   site and is deleted rather than moved.
//! - "Out of range" is a **RangeError**, not a declined fold. [`RangeChecked`]
//!   is therefore not an `Option`, so `?` on it does not compile, and
//!   [`NumberFormatFold`] separates "the spec requires a throw here" from "I
//!   could not fold this" — the two values the folding helpers conflated.
//! - The non-finite-receiver early return sits on **either side** of the range
//!   check depending on the clause (21.1.3.2 step 4 before step 5; 21.1.3.3
//!   steps 4–5 before step 6). There are exactly two orders, so
//!   [`NonFiniteReceiverOrder`] is a closed enum — and it is a per-clause
//!   **constant** on [`NumberFormatClause`] rather than an argument the caller
//!   names, because an argument left the wrong *choice* expressible. The clause
//!   is the type parameter of [`fold_number_format`], its `Digits` associated
//!   types are pairwise distinct, and its `RANGE_ERROR` message travels with
//!   the outcome; so neither the ordering nor the message can be transposed.
//! - 7.1.6, 7.1.7 and 7.1.9 are character-for-character the same operation
//!   through step 4 and differ only in step 5's reading of one residue.
//!   [`residue_pow2_i64`] is that shared step, stated once, as a `const fn` over
//!   integers with no floating point at all; the `const _: () = assert!(…)`
//!   tables at the end of this file check it and tie the three readings
//!   together at build time. Five emitters in `porffor-aot-wasm` re-derive this
//!   step in five different spellings and two of them are wrong; that crate
//!   boundary is ledger row **LN2** and is handled in
//!   `target/lane-notes/numeric-conversion-codomains-theory-integration.md`.
//! - 5.2.5's `x modulo y` is a **non-negative** residue when `y > 0`, not a
//!   machine remainder. Rust's `%` and Wasm's `I64RemS` take the sign of the
//!   dividend and are this operation only after a correction; applying a
//!   saturating float→int truncation *before* the modulo destroys it outright.
//!
//! `ToInt32`, `ToLength` and `ToIndex` deliberately get **no** codomain newtype:
//! this crate has no construction site for them. They get `const fn` reference
//! algorithms plus build-time tables instead. A type nobody constructs is
//! decoration; a `const` assertion that fails the build is not.

/// 7.1.5 ToIntegerOrInfinity's codomain: **ℤ ∪ {+∞, −∞}**.
///
/// Not `i32` (which cannot hold `truncate(1e300)`), not `Option<i32>` (whose
/// `None` means "no answer" where 7.1.5 always has one), and not `f64` (whose
/// domain admits `0.5` and `NaN`, neither of which 7.1.5 can return).
///
/// `PartialEq` but not `Eq`: the finite payload is a binary64, and the type
/// deliberately does not pretend to a total equality it cannot have. The
/// payload is normalised so that `-0.0` never occurs (see
/// [`IntegerOrInfinity::of_number`]), which is what makes the derived
/// `PartialEq` agree with 5.2.5 equality of mathematical values on every
/// reachable value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntegerOrInfinity {
    NegativeInfinity,
    Finite(FiniteInteger),
    PositiveInfinity,
}

/// `truncate(ℝ(number))` for a finite `number`. Exactly representable in
/// binary64, and never NaN, never infinite, never `-0.0`.
///
/// The field is private to this module, so `IntegerOrInfinity::Finite(..)`
/// cannot be *spelled* anywhere else in the workspace: an outside caller has no
/// way to obtain the payload to put inside it. That is what makes
/// [`IntegerOrInfinity::of_number`] the only constructor of the enum rather
/// than merely the recommended one.
///
/// There is deliberately no `value()`, no `as_f64()`, no `as_i32()`, no `Deref`
/// and no `From`. Adding one restores, in a single line, the defect this type
/// exists to close: a codomain that cannot hold the spec's range, escaping into
/// a comparison against a machine integer.
///
/// **And deliberately no `PartialOrd`.** It was derived once, and it was a hole
/// the size of the type's whole purpose: ordering is all an interval test needs,
/// and `of_number` is a public source of comparison constants, so any crate
/// depending on `porffor-ir` could write
/// `match (of_number(v), of_number(2.0), of_number(36.0)) { (Finite(x),
/// Finite(lo), Finite(hi)) => x >= lo && x <= hi, _ => false }` — a hand-rolled
/// `[2, 36]` radix check with a catch-all that silently answers "out of range"
/// for `±∞`. That is mistake class N3, rebuilt from outside using only the
/// public API. Nothing in this module compares two `FiniteInteger`s
/// (`fraction_digits` and `precision` compare the destructured `f64`), so the
/// derive was pure attack surface. `PartialEq` alone cannot express an interval
/// and stays.
///
/// If a future clause needs to order the extended integers, the right primitive
/// is an `Ord` on [`IntegerOrInfinity`] with `NegativeInfinity < Finite <
/// PositiveInfinity` — the *correct* order, which a `FiniteInteger`-only
/// comparison cannot express — and it arrives with its own call site.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FiniteInteger(f64);

impl IntegerOrInfinity {
    /// 7.1.5 steps 2–5, applied to a value that has already been through step 1
    /// (`? ToNumber(argument)`).
    ///
    /// **Total.** 7.1.5 has no failure mode once the argument is a Number, so
    /// this returns `Self` and not `Option<Self>`. Whether a *compiler* can
    /// discover the argument statically is a different question, asked and
    /// answered at the call site.
    pub fn of_number(number: f64) -> Self {
        // Step 2. `number == 0.0` is true for both `+0𝔽` and `-0𝔽`, and the
        // spec returns the mathematical 0 for all three of NaN, +0 and −0.
        if number.is_nan() || number == 0.0 {
            return Self::Finite(FiniteInteger(0.0));
        }
        // Step 3.
        if number == f64::INFINITY {
            return Self::PositiveInfinity;
        }
        // Step 4.
        if number == f64::NEG_INFINITY {
            return Self::NegativeInfinity;
        }
        // Step 5. `f64::trunc` is IEEE-754 `roundToIntegralTowardZero`, which
        // computes `truncate(ℝ(number))` exactly for every finite binary64.
        //
        // The `+ 0.0` is load-bearing and must not be removed or replaced with
        // `.abs()`: `(-0.5f64).trunc()` is `-0.0`, and step 2 returns the
        // mathematical 0. Without the normalisation, two spellings of one
        // mathematical value would both escape this constructor.
        Self::Finite(FiniteInteger(number.trunc() + 0.0))
    }

    /// 21.1.3.2 step 5 and 21.1.3.3 steps 4–5: `f < 0 or f > 100`.
    ///
    /// `+∞ > 100` and `−∞ < 0` hold here, which is the whole reason 7.1.5's
    /// codomain had to include them: an implementation that has already
    /// collapsed `±∞` into `0` cannot evaluate this step, it can only
    /// approximate it.
    pub fn fraction_digits(self) -> RangeChecked<FractionDigits> {
        match self {
            Self::NegativeInfinity | Self::PositiveInfinity => RangeChecked::RangeError,
            Self::Finite(FiniteInteger(value)) => {
                if (0.0..=100.0).contains(&value) {
                    // Exact: `value` is an exact integer already proven to lie
                    // in `[0, 100]`, so the saturating float→int cast never
                    // saturates.
                    RangeChecked::InBounds(FractionDigits(value as u8))
                } else {
                    RangeChecked::RangeError
                }
            }
        }
    }

    /// 21.1.3.5 step 5: `p < 1 or p > 100`.
    ///
    /// The interval is `[1, 100]`, **not** `[0, 100]`. This is why the result is
    /// a [`Precision`] and not a [`FractionDigits`].
    pub fn precision(self) -> RangeChecked<Precision> {
        match self {
            Self::NegativeInfinity | Self::PositiveInfinity => RangeChecked::RangeError,
            Self::Finite(FiniteInteger(value)) => {
                if (1.0..=100.0).contains(&value) {
                    RangeChecked::InBounds(Precision(value as u8))
                } else {
                    RangeChecked::RangeError
                }
            }
        }
    }
}

/// The outcome of testing an [`IntegerOrInfinity`] against one clause's closed
/// interval, on the **extended** integers.
///
/// Deliberately **not** `Option`. The folding helpers in `lowering.rs` return
/// `Option<String>`, so `?` on an `Option` there spells "decline the fold" —
/// which is exactly the wrong answer this tree shipped: a spec-mandated
/// RangeError silently became a runtime call. `RangeError` is not `None`, and
/// `?` on a `RangeChecked` does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub enum RangeChecked<T> {
    InBounds(T),
    RangeError,
}

impl<T> RangeChecked<T> {
    /// Retags an in-bounds payload without letting the `RangeError` arm be
    /// forgotten. Used where 21.1.3.2 step 12.a needs the `undefined` case to
    /// stay observably distinct from `f === 0`.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> RangeChecked<U> {
        match self {
            Self::InBounds(value) => RangeChecked::InBounds(f(value)),
            Self::RangeError => RangeChecked::RangeError,
        }
    }
}

/// 21.1.3.2 step 5 / 21.1.3.3 steps 4–5: a fraction-digit count already checked
/// against `[0, 100]`.
///
/// Constructible only by [`IntegerOrInfinity::fraction_digits`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FractionDigits(u8);

/// 21.1.3.5 step 5: a precision already checked against `[1, 100]`.
///
/// A **different type** from [`FractionDigits`], because `[0, 100]` and
/// `[1, 100]` are different intervals belonging to different clauses. Passing
/// one where the other is expected is `E0308`, and there is no conversion
/// between them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Precision(u8);

impl FractionDigits {
    /// 21.1.3.3 step 3's assertion: if `fractionDigits` is undefined then `f`
    /// is 0. Naming it here keeps that step from being spelled as a bare `0`
    /// at the call site.
    pub const ZERO: Self = Self(0);

    /// The two formatters in `lowering.rs` predate this contract and take
    /// `usize`; their signatures are outside this area's owned regions. This is
    /// the single narrow exit, and it cannot widen the range because the type
    /// has one constructor.
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl Precision {
    /// In `1..=100` by construction. `static_number_to_precision`'s body
    /// computes `precision as usize - 1`; the lower bound is carried by this
    /// type rather than re-tested there. Ledger **LN3**.
    pub fn get(self) -> u8 {
        self.0
    }
}

/// Where 21.1.3.x puts "if x is not finite, return `Number::toString(x, 10)`"
/// relative to the digit-count range check.
///
/// There are exactly two orders and the choice is **observable**, so it is a
/// closed enum and a required argument: omitting it is `E0061`, and a third
/// order appearing is `E0004` at every match over it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonFiniteReceiverOrder {
    /// 21.1.3.2 step 4 precedes step 5, and 21.1.3.5 step 4 precedes step 5.
    /// `Infinity.toExponential(101)` is `"Infinity"`;
    /// `Infinity.toPrecision(500)` is `"Infinity"`.
    ReceiverFirst,
    /// 21.1.3.3 steps 4–5 precede step 6.
    /// `Infinity.toFixed(101)` throws a **RangeError**.
    RangeCheckFirst,
}

/// What a static `Number.prototype` fold concluded.
///
/// Replaces `Option<String>` at the three folding helpers so that "the spec
/// requires a RangeError here" and "I could not fold this" stop being the same
/// value. Constant folding is optional, so `NotStatic` is always a legitimate
/// answer; producing a *wrong* value never is.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub enum NumberFormatFold {
    /// Emit `ExprIr::String`.
    Formatted(String),
    /// Emit `ExprIr::RuntimeThrow` with `NativeErrorKind::RangeError` and the
    /// carried message.
    ///
    /// The message travels **with** the outcome rather than being named at the
    /// dispatch site, because the dispatch site is where it went wrong: three
    /// adjacent arms each spelled a `&'static str` literal, so pasting
    /// `"Number.prototype.toFixed fraction digits out of range"` into the
    /// `toPrecision` arm compiled and shipped the wrong text. It is now
    /// `<C as NumberFormatClause>::RANGE_ERROR` and there is no string literal
    /// left at the call sites.
    RangeError(&'static str),
    /// Not statically decidable; fall through to the runtime call path.
    NotStatic,
}

/// One of the three `Number.prototype` formatting clauses, as a **type**.
///
/// This is the construct that closes mistake class N4″ — "name the wrong
/// `NonFiniteReceiverOrder`" — which a required enum argument could not: with a
/// free argument, swapping `ReceiverFirst` and `RangeCheckFirst` at any of the
/// three `*_call` helpers compiles and silently makes
/// `Infinity.toFixed(101)` answer `"Infinity"` instead of throwing.
///
/// Two properties do the work. First, the three [`Self::Digits`] types are
/// pairwise distinct — `Option<FractionDigits>` (21.1.3.2 step 12.a needs
/// `undefined` to stay observably different from `f === 0`, so the `Option` is
/// spec-mandated, not incidental), `FractionDigits` (21.1.3.3) and `Precision`
/// (21.1.3.5) — so naming the wrong clause at a call site is `E0308` on the
/// `digits` argument. Second, `ORDER` and `RANGE_ERROR` become properties *of
/// the clause* rather than values the caller supplies, so neither can be
/// transposed or copy-pasted.
///
/// This is **not** the typestate builder §2.3 refuses: it adds no phantom
/// state and no step-by-step protocol, it removes two caller-supplied values,
/// and it rejects two program classes the enum argument admitted. That is
/// AGENTS.md's test, met.
pub trait NumberFormatClause {
    /// The clause's already-range-checked digit argument. Distinct per clause;
    /// that is what makes naming the wrong clause a type error.
    type Digits;
    /// Where this clause's "if x is not finite" step sits relative to its range
    /// check.
    const ORDER: NonFiniteReceiverOrder;
    /// The message its out-of-range step throws. Matches the message the
    /// runtime path throws for the same clause.
    const RANGE_ERROR: &'static str;
}

/// 21.1.3.2 `Number.prototype.toExponential ( fractionDigits )`.
pub enum ToExponential {}

/// 21.1.3.3 `Number.prototype.toFixed ( fractionDigits )`.
pub enum ToFixed {}

/// 21.1.3.5 `Number.prototype.toPrecision ( precision )`.
pub enum ToPrecision {}

impl NumberFormatClause for ToExponential {
    /// 21.1.3.2 step 12.a: `fractionDigits === undefined` is observably
    /// different from `f === 0`.
    type Digits = Option<FractionDigits>;
    /// Step 4 (non-finite receiver) precedes step 5 (the range check), so
    /// `Infinity.toExponential(101)` is `"Infinity"`.
    const ORDER: NonFiniteReceiverOrder = NonFiniteReceiverOrder::ReceiverFirst;
    const RANGE_ERROR: &'static str = "Number.prototype.toExponential fraction digits out of range";
}

impl NumberFormatClause for ToFixed {
    /// Step 3: `fractionDigits` undefined ⇒ `f` is 0, so there is no `Option`.
    type Digits = FractionDigits;
    /// Steps 4–5 (the range check) precede step 6, so `Infinity.toFixed(101)`
    /// throws a **RangeError**. This is the one clause that differs, and the
    /// difference is observable.
    const ORDER: NonFiniteReceiverOrder = NonFiniteReceiverOrder::RangeCheckFirst;
    const RANGE_ERROR: &'static str = "Number.prototype.toFixed fraction digits out of range";
}

impl NumberFormatClause for ToPrecision {
    /// Step 2 returns `! ToString(x)` before step 3's coercion when `precision`
    /// is undefined, so `undefined` never reaches 7.1.5 and there is no
    /// `Option` here either. The interval is `[1, 100]`, not `[0, 100]`.
    type Digits = Precision;
    /// Step 4 precedes step 5, as in 21.1.3.2.
    const ORDER: NonFiniteReceiverOrder = NonFiniteReceiverOrder::ReceiverFirst;
    const RANGE_ERROR: &'static str = "Number.prototype.toPrecision precision out of range";
}

/// The single driver for 21.1.3.2, 21.1.3.3 and 21.1.3.5.
///
/// All three clauses are this function with different arguments; there is no
/// fourth spelling. Without it each call site re-derives the ordering by hand,
/// which is what the tree did — correctly, but by convention rather than by
/// construction, sixty lines apart, with the difference expressed as
/// `.is_some()` versus `.is_some_and(is_finite)`.
///
/// `digits` arrives **already checked**. The driver cannot perform the check
/// itself because the interval is clause-specific, and threading a bounds
/// selector back in would re-open the very mistake this closes. The check
/// happens where the clause is known; the driver enforces only the *ordering*,
/// which is what it is for.
///
/// The clause is a **type parameter**, not a value: `C::ORDER` and
/// `C::RANGE_ERROR` cannot be transposed or copy-pasted, and because the three
/// `C::Digits` are pairwise distinct, naming the wrong clause is `E0308` on
/// `digits`. See [`NumberFormatClause`] for why this is not the typestate
/// builder this contract refuses.
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
                RangeChecked::InBounds(digits) => finish(digits),
            }
        }
        NonFiniteReceiverOrder::RangeCheckFirst => match digits {
            RangeChecked::RangeError => NumberFormatFold::RangeError(C::RANGE_ERROR),
            RangeChecked::InBounds(digits) => {
                if !receiver.is_finite() {
                    return NumberFormatFold::Formatted(number_to_string(receiver));
                }
                finish(digits)
            }
        },
    }
}

/// 7.1.7 ToUint32's codomain: `int modulo 2^32`, read unsigned.
///
/// The field is private, so `Uint32(x)` outside this module is `E0603`: a
/// second, hand-rolled residue cannot produce the type its consumer accepts, so
/// it cannot be wired in. That is the in-crate form of the defect where two
/// implementations of one operation disagree.
///
/// No `get()`, no `From`, no `Into`, no `Deref`, no public field. The single
/// reader is [`Uint32::leading_zeros`], which is one spec step at one call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Uint32(u32);

/// 7.1.9 ToUint16's codomain: `int modulo 2^16`.
///
/// Same shape and same reasons as [`Uint32`]; the single reader is
/// [`Uint16::code_unit`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Uint16(u16);

/// The moduli 5.2.5's `x modulo 2^N` is taken at in this contract: `N = 32` for
/// 7.1.6/7.1.7 and `N = 16` for 7.1.9.
///
/// A closed two-element domain, not a bare `u32`. `residue_pow2_i64(x, 64)` and
/// `residue_pow2_i64(x, 0)` used to compile: the `debug_assert!` is gone in
/// release, `1u64 << 64` is a *masked* shift there and evaluates to `1`, so the
/// mask becomes `0` and the function silently returns `0` for every input —
/// inside the very item the LN2 backend retrofit is meant to call as normative.
/// Only `const` callers were protected, because `1u64 << 64` is a hard error in
/// const evaluation. Those two arguments are now unrepresentable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResidueWidth {
    /// 7.1.9 ToUint16.
    Bits16,
    /// 7.1.6 ToInt32 and 7.1.7 ToUint32.
    Bits32,
}

impl ResidueWidth {
    /// `2^N − 1`. The shift is on a literal from a two-element domain, so it is
    /// never 64 and never 0.
    pub const fn mask(self) -> u64 {
        match self {
            Self::Bits16 => (1u64 << 16) - 1,
            Self::Bits32 => (1u64 << 32) - 1,
        }
    }
}

/// The integer carrier one [`ResidueWidth`] reads its residue into.
///
/// This exists to tie the modulus to the destination cast, which were two
/// independent bare integers: editing `Uint32::of_number` to
/// `residue_of_number(number, 16) as u32` compiled and produced 7.1.9's residue
/// in 7.1.7's carrier, so `Math.clz32(65537)` would have answered `31` instead
/// of `15`. With this, `residue_of_number::<u32>(number)` derives its own
/// modulus from the carrier and there is no second number to get wrong.
pub trait ResidueCarrier: Copy {
    /// The modulus 5.2.5 is applied at for this carrier.
    const WIDTH: ResidueWidth;
    /// Reads the residue, which [`ResidueWidth::mask`] has already reduced into
    /// range, so this cast is exact.
    fn of_residue(residue: u64) -> Self;
}

impl ResidueCarrier for u32 {
    const WIDTH: ResidueWidth = ResidueWidth::Bits32;
    fn of_residue(residue: u64) -> Self {
        residue as u32
    }
}

impl ResidueCarrier for u16 {
    const WIDTH: ResidueWidth = ResidueWidth::Bits16;
    fn of_residue(residue: u64) -> Self {
        residue as u16
    }
}

impl Uint32 {
    /// 7.1.7 steps 2–5 on a value already through step 1. **Total.**
    pub fn of_number(number: f64) -> Self {
        Self(residue_of_number::<u32>(number))
    }

    /// 21.3.2.11 `Math.clz32` step 2: the number of leading zero bits in the
    /// 32-bit binary representation of `n`. The only reader of this type, and
    /// the only one this crate has a call site for.
    pub fn leading_zeros(self) -> u32 {
        self.0.leading_zeros()
    }
}

impl Uint16 {
    /// 7.1.9 steps 2–5 on a value already through step 1. **Total.**
    pub fn of_number(number: f64) -> Self {
        Self(residue_of_number::<u16>(number))
    }

    /// 22.1.2.1 step 2.a: the single UTF-16 code unit `String.fromCharCode`
    /// appends. The only reader of this type.
    pub fn code_unit(self) -> u16 {
        self.0
    }
}

/// 7.1.6 / 7.1.7 / 7.1.9 steps 2–4, shared. The modulus is
/// `C::WIDTH` — derived from the carrier, so there is no second argument that
/// could disagree with the destination cast.
///
/// The `|v| < 2^63` branch routes through the const-asserted integer core, so
/// [`residue_pow2_i64`] is on the product path and not merely a build-time
/// artefact. The outer branch is the one place a floating-point exactness
/// argument is load-bearing and is ledger row **LN1**: IEEE-754 specifies
/// `fmod` as exact, `rem_euclid` adds at most one exactly-representable
/// addition, and both operands and result are integers of magnitude below
/// `2^53`.
///
/// **LN1's honest statement**: no type can carry that argument *for this
/// formulation*. `rem_euclid` is not the only way to compute the branch —
/// decomposing `x.to_bits()` into `m · 2^e` gives `x mod 2^N` as `0` when
/// `e ≥ N` and `(m & ((1 << (N − e)) − 1)) << e` otherwise, which is integer
/// arithmetic, `const`-evaluable, and exact by construction rather than by an
/// appeal to IEEE-754 §5.3.1. Rewriting it that way would move the 18-case
/// boundary table from `#[cfg(test)]` into a third `const _` block and retire
/// LN1 outright.
fn residue_of_number<C: ResidueCarrier>(number: f64) -> C {
    // Step 2: non-finite or ±0 → +0𝔽. `number == 0.0` covers `-0.0`.
    if !number.is_finite() || number == 0.0 {
        return C::of_residue(0);
    }
    // Step 3. Exact for every finite binary64.
    let truncated = number.trunc();
    // 2^63, exactly representable.
    const I64_SPAN: f64 = 9_223_372_036_854_775_808.0;
    let residue = if truncated >= -I64_SPAN && truncated < I64_SPAN {
        // Step 4 inside the i64 window: two's complement *is* the residue
        // system modulo 2^64, so masking is exact.
        residue_pow2_i64(truncated as i64, C::WIDTH)
    } else {
        // Step 4 outside it. Note that this is `rem_euclid`, not `%`: 5.2.5's
        // residue takes the sign of the modulus, which is positive here, so it
        // is never negative.
        truncated.rem_euclid((C::WIDTH.mask() + 1) as f64) as u64
    };
    C::of_residue(residue)
}

/// 2^53 − 1. 7.1.20 step 3's clamp and 7.1.22 step 2's upper bound.
pub const MAX_SAFE_INTEGER_U64: u64 = 9_007_199_254_740_991;

/// 5.2.5 `x modulo 2^N` for a mathematical integer that fits an `i64`.
///
/// **This is the step all five backend emitters re-derive, in five different
/// spellings, and two get wrong.** It is a *non-negative* residue, not a machine
/// remainder: Wasm's `I64RemS` takes the sign of the dividend and needs a
/// correction afterwards, and `I64TruncSatF64S` applied *before* this step
/// destroys the residue outright — every operand of magnitude at least 2^63
/// collapses onto `i64::MAX` or `i64::MIN`, whose residues are `2^32 − 1` and
/// `0`, and neither is the answer for more than a measure-zero set of inputs.
///
/// Two's complement is the residue system modulo 2^64, so masking the low `N`
/// bits is the exact residue. No floating point, no `rem_euclid`, no libm —
/// which is what lets this be `const` on stable and lets the tables at the end
/// of this file be checked by the compiler rather than by a suite.
///
/// `width` is a [`ResidueWidth`] rather than a `u32` because this item is `pub`
/// and is the normative algorithm the LN2 backend retrofit will call: a bare
/// `bits` admitted `64` and `0`, which a release build turns into "return 0 for
/// every input" via a masked shift and a `debug_assert!` that is not there.
pub const fn residue_pow2_i64(int: i64, width: ResidueWidth) -> u64 {
    (int as u64) & width.mask()
}

/// 7.1.6 ToInt32 steps 4–5, given `truncate(ℝ(number))` reduced into an `i64`.
///
/// This has no runtime caller in this crate — there is no ToInt32 site here —
/// and that is stated rather than hidden. Its consumer is the const block at
/// the end of this file, so a wrong edit fails the build rather than the suite.
pub const fn reference_to_int32(truncated: i64) -> i32 {
    residue_pow2_i64(truncated, ResidueWidth::Bits32) as u32 as i32
}

/// 7.1.7 ToUint32 steps 4–5.
pub const fn reference_to_uint32(truncated: i64) -> u32 {
    residue_pow2_i64(truncated, ResidueWidth::Bits32) as u32
}

/// 7.1.9 ToUint16 steps 4–5.
pub const fn reference_to_uint16(truncated: i64) -> u16 {
    residue_pow2_i64(truncated, ResidueWidth::Bits16) as u16
}

/// 7.1.5's codomain restricted to the integers an `i64` holds, plus the two
/// infinities.
///
/// This is **not** a codomain type and nothing lowers through it. Its payload
/// field is public precisely because there is no invariant to protect: every
/// `i64` is a legal integer. It exists so that 7.1.20 and 7.1.22 — which
/// *consume* 7.1.5's extended codomain rather than produce it — can be written
/// as `const fn` and pinned by the tables below. Do not use it as a conversion
/// result anywhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedInteger {
    NegInfinity,
    Finite(i64),
    PosInfinity,
}

/// 7.1.20 ToLength steps 2–3.
///
/// Step 2's `≤` and step 3's `min` are comparisons on ℤ ∪ {±∞}, which is why
/// the parameter is an [`ExtendedInteger`] and not an `i64`.
pub const fn reference_to_length(len: ExtendedInteger) -> u64 {
    match len {
        ExtendedInteger::NegInfinity => 0,
        ExtendedInteger::PosInfinity => MAX_SAFE_INTEGER_U64,
        ExtendedInteger::Finite(value) => {
            if value <= 0 {
                0
            } else if (value as u64) > MAX_SAFE_INTEGER_U64 {
                MAX_SAFE_INTEGER_U64
            } else {
                value as u64
            }
        }
    }
}

/// 7.1.22 ToIndex's two outcomes.
///
/// ToIndex is the one **partial** operation in this contract and the partiality
/// is spec-mandated, so it is a two-variant enum rather than an `Option`: the
/// second variant names a throw, not an absence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub enum ToIndexOutcome {
    Index(u64),
    RangeError,
}

/// 7.1.22 ToIndex step 2.
///
/// `+∞` is not in `[0, 2^53−1]`, and an implementation that has already
/// collapsed `+∞` onto `0` cannot tell that. Hence the [`ExtendedInteger`]
/// parameter.
pub const fn reference_to_index(integer: ExtendedInteger) -> ToIndexOutcome {
    match integer {
        ExtendedInteger::NegInfinity | ExtendedInteger::PosInfinity => ToIndexOutcome::RangeError,
        ExtendedInteger::Finite(value) => {
            if value < 0 || (value as u64) > MAX_SAFE_INTEGER_U64 {
                ToIndexOutcome::RangeError
            } else {
                ToIndexOutcome::Index(value as u64)
            }
        }
    }
}

/// The residues 5.2.5 requires, checked at build time.
///
/// Every row was computed from 5.2.5 and cross-checked against an exact-integer
/// model; none was read off this tree's current behaviour. If a future edit
/// re-spells `residue_pow2_i64` as `int % (1 << bits)`, or a `reference_to_*`
/// as a truncating cast, `reference_to_uint32(-1)` and `reference_to_uint16(-1)`
/// fail here — and a `const` assertion failure is a hard compile error, not a
/// warning.
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
    // the wrong answer for `+∞`; that difference is the whole defect.
    assert!(reference_to_uint32(i64::MAX) == 4_294_967_295);
    assert!(reference_to_uint32(i64::MIN) == 0);

    // 7.1.6 — the same residue, with step 5 reading it signed.
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
    assert!(
        reference_to_length(ExtendedInteger::Finite(9_007_199_254_740_991)) == MAX_SAFE_INTEGER_U64
    );
    assert!(
        reference_to_length(ExtendedInteger::Finite(9_007_199_254_740_992)) == MAX_SAFE_INTEGER_U64
    );

    // 7.1.22.
    assert!(matches!(
        reference_to_index(ExtendedInteger::NegInfinity),
        ToIndexOutcome::RangeError
    ));
    assert!(matches!(
        reference_to_index(ExtendedInteger::PosInfinity),
        ToIndexOutcome::RangeError
    ));
    assert!(matches!(
        reference_to_index(ExtendedInteger::Finite(-1)),
        ToIndexOutcome::RangeError
    ));
    assert!(matches!(
        reference_to_index(ExtendedInteger::Finite(0)),
        ToIndexOutcome::Index(0)
    ));
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
    0,
    1,
    -1,
    255,
    65_535,
    65_536,
    2_147_483_647,
    2_147_483_648,
    -2_147_483_648,
    4_294_967_295,
    4_294_967_296,
    -4_294_967_296,
    i64::MAX,
    i64::MIN,
];

/// **Ties the tables together.** 7.1.6 and 7.1.7 differ only in the reading of
/// one residue, and 7.1.9 is the same residue at a narrower modulus. If any one
/// of the three reference algorithms is edited in isolation, this fails the
/// build rather than the suite.
const _: () = {
    let mut i = 0;
    while i < RESIDUE_TIE_INPUTS.len() {
        let value = RESIDUE_TIE_INPUTS[i];
        // 7.1.6 step 5 and 7.1.7 step 5 read the *same* `int32bit`.
        assert!(reference_to_int32(value) as u32 == reference_to_uint32(value));
        // 7.1.9's modulus divides 7.1.7's, so the narrower residue is the
        // wider one's low half.
        assert!(reference_to_uint16(value) as u32 == reference_to_uint32(value) & 0xFFFF);
        // Both readings agree on the residue's membership in [0, 2^32).
        assert!(residue_pow2_i64(value, ResidueWidth::Bits32) == reference_to_uint32(value) as u64);
        i += 1;
    }
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Ledger row **LN1**, and the one invariant in this file that no type
    /// carries.
    ///
    /// Two things are checked, and neither can be a compile error:
    ///
    /// 1. `residue_of_number`'s `|x| ≥ 2^63` branch — the only floating-point
    ///    `rem_euclid` left in this crate — agrees with the const-asserted
    ///    integer core across the boundary. The bridge is an IEEE-754 exactness
    ///    argument about `fmod`, i.e. a statement about the hardware, and
    ///    `f64::trunc` / `f64::rem_euclid` are not const-evaluable on the
    ///    pinned toolchain, so the float branch cannot be checked at build time.
    /// 2. `IntegerOrInfinity::of_number` routes each of 7.1.5's five arms to the
    ///    right variant. A type cannot prevent a wrong function *body*; a
    ///    constructor that returned `Finite(0)` for `+∞` would typecheck
    ///    perfectly. This is the only mistake class in the contract that no
    ///    construct closes, and it is checked here rather than claimed closed.
    ///
    /// This is deliberately the **only** test in this file. A second one would
    /// be evidence that an invariant a type could have carried was left to a
    /// test instead.
    #[test]
    fn ln1_residue_boundary_differential_and_integer_or_infinity_arms() {
        // 2^63 and its neighbours at the representable spacing there (ulp at
        // 2^63 is 2^11 = 2048), plus the magnitudes where the residue is
        // accidentally 0 because the ulp already exceeds the modulus.
        const TWO_63: f64 = 9_223_372_036_854_775_808.0;
        const CASES: [(f64, u32, u16); 18] = [
            (0.0, 0, 0),
            (-0.0, 0, 0),
            (f64::INFINITY, 0, 0),
            (f64::NEG_INFINITY, 0, 0),
            (f64::NAN, 0, 0),
            (-1.0, 4_294_967_295, 65_535),
            (4_294_967_295.0, 4_294_967_295, 65_535),
            (4_294_967_296.0, 0, 0),
            (65_601.0, 65_601, 65),
            // Inside the i64 window, just below the boundary.
            (TWO_63 - 2048.0, 4_294_965_248, 63_488),
            // Exactly on it, and just outside: the first value the saturating
            // backends get wrong.
            (-TWO_63, 0, 0),
            (TWO_63, 0, 0),
            (TWO_63 + 2048.0, 2048, 2048),
            (-(TWO_63 + 2048.0), 4_294_965_248, 63_488),
            // Far outside, where the ulp exceeds both moduli.
            (18_446_744_073_709_551_616.0, 0, 0),         // 2^64
            (19_342_813_113_834_066_795_298_816.0, 0, 0), // 2^84
            (1e300, 0, 0),
            (-1e300, 0, 0),
        ];

        for (input, expected_u32, expected_u16) in CASES {
            assert_eq!(
                Uint32::of_number(input),
                Uint32(expected_u32),
                "ToUint32({input:?})"
            );
            assert_eq!(
                Uint16::of_number(input),
                Uint16(expected_u16),
                "ToUint16({input:?})"
            );
            // The narrower modulus divides the wider one, on the float path as
            // well as on the const-asserted integer path.
            assert_eq!(
                u32::from(Uint16::of_number(input).code_unit()),
                expected_u32 & 0xFFFF,
                "ToUint16({input:?}) is ToUint32's low half"
            );
        }

        // The two branches of `residue_of_number` agree where they meet: every
        // finite input inside the i64 window must equal the integer core.
        for input in [
            0.5,
            -0.5,
            1.0,
            -1.0,
            2_147_483_648.0,
            -2_147_483_648.0,
            4_294_967_297.0,
            -4_294_967_297.0,
            TWO_63 - 2048.0,
            -TWO_63,
        ] {
            assert_eq!(
                Uint32::of_number(input),
                Uint32(reference_to_uint32(input.trunc() as i64)),
                "i64-window agreement for {input:?}"
            );
            assert_eq!(
                Uint16::of_number(input),
                Uint16(reference_to_uint16(input.trunc() as i64)),
                "i64-window agreement for {input:?}"
            );
        }

        // 7.1.5's five arms, each to its own variant. Steps 2, 3, 4, 5.
        assert_eq!(
            IntegerOrInfinity::of_number(f64::NAN),
            IntegerOrInfinity::of_number(0.0)
        );
        assert_eq!(
            IntegerOrInfinity::of_number(-0.0),
            IntegerOrInfinity::of_number(0.0)
        );
        assert_eq!(
            IntegerOrInfinity::of_number(f64::INFINITY),
            IntegerOrInfinity::PositiveInfinity
        );
        assert_eq!(
            IntegerOrInfinity::of_number(f64::NEG_INFINITY),
            IntegerOrInfinity::NegativeInfinity
        );
        // Step 5 truncates toward zero, and `(-0.5).trunc()` is `-0.0`, which
        // must be normalised to the mathematical 0 of step 2.
        assert_eq!(
            IntegerOrInfinity::of_number(-0.5),
            IntegerOrInfinity::of_number(0.0)
        );
        assert_eq!(
            IntegerOrInfinity::of_number(1.9),
            IntegerOrInfinity::of_number(1.0)
        );
        // The value the old `i32` carrier saturated to `i32::MAX`: it is finite,
        // enormous, and must be neither an infinity nor in `[0, 100]`.
        assert!(matches!(
            IntegerOrInfinity::of_number(1e300),
            IntegerOrInfinity::Finite(_)
        ));
        assert!(matches!(
            IntegerOrInfinity::of_number(1e300).fraction_digits(),
            RangeChecked::RangeError
        ));

        // The two intervals, at the one point where they disagree.
        assert!(matches!(
            IntegerOrInfinity::of_number(0.0).fraction_digits(),
            RangeChecked::InBounds(FractionDigits(0))
        ));
        assert!(matches!(
            IntegerOrInfinity::of_number(0.0).precision(),
            RangeChecked::RangeError
        ));
        assert!(matches!(
            IntegerOrInfinity::of_number(f64::INFINITY).precision(),
            RangeChecked::RangeError
        ));
        assert!(matches!(
            IntegerOrInfinity::of_number(f64::NEG_INFINITY).fraction_digits(),
            RangeChecked::RangeError
        ));
    }
}
