//! The closed domain of ECMA-262 error intrinsic names.
//!
//! ECMA-262 §20.5.5 *Native Error Types Used in This Standard* enumerates
//! exactly six NativeErrors, and §20.5.6 defines all six by a single parametric
//! template. `Error` (§20.5.1), `AggregateError` (§20.5.7) and `SuppressedError`
//! (Explicit Resource Management) are each defined by their own clause and are
//! **not** NativeErrors under §20.5.5.
//!
//! [`NativeErrorKind`] names the nine-element union of those four clauses.
//! ECMA-262 has no name for that union because it has no need of one, so the
//! name is a mild abuse. Which six are the spec-exact §20.5.5 NativeErrors is
//! recorded in the per-variant doc comments of the row list below, not in a
//! predicate: `is_native_error`, `NATIVE_ERRORS` and the two const assertions
//! that tied them together had **no call site anywhere in the workspace**, and a
//! `pub` item with no call site is precisely the "survival by `pub`" AGENTS.md
//! names. Worse, `for k in ALL { if k.is_native_error() { … } }` reads as "for
//! every error kind" while silently skipping `Error`, `AggregateError` and
//! `SuppressedError` — a wrong answer the predicate invited rather than
//! prevented. Reintroduce it, spelled `is_20_5_5_native_error` so no call site
//! can read it as a general error test, in the same patch that gives it a
//! consumer.
//!
//! The nine are, exhaustively:
//!
//! ```text
//! Error, EvalError, RangeError, ReferenceError, SyntaxError, TypeError,
//! URIError, AggregateError, SuppressedError
//! ```
//!
//! Everything below — the spellings, the constructor mapping, the §20.5.5
//! subset predicate — is generated from **one** row list by the
//! `native_error_kinds!` macro. Adding a tenth error intrinsic means editing
//! that list, which regenerates every derived table at once; there is no second
//! list to forget. That is the property this module exists to have. See
//! `docs/rust-rewrite/contracts/closed-name-domains.md`.
//!
//! Deliberately absent, and deliberately never to be added: `Display`,
//! `AsRef<str>`, `Deref<Target = str>`, `FromStr`, `Default`, and
//! `From<NativeErrorKind> for String`. A stringification must name
//! [`NativeErrorKind::as_str`] at the call site, so that `format!("{kind}")`
//! cannot quietly reintroduce the `&str` domain this type replaces.

use crate::StandardBuiltinId;

/// Byte-wise `&str` equality usable in a `const` initializer.
///
/// Private on purpose: a `pub const fn str_eq` would be workspace surface with
/// no product call site, which is the shape AGENTS.md names as having shipped
/// here before. `well_known.rs` keeps its own copy for the same reason.
const fn str_eq(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

macro_rules! native_error_kinds {
    (
        $(
            $(#[$variant_meta:meta])*
            $variant:ident => $spelling:literal, $constructor:ident;
        )+
    ) => {
        /// One of the nine ECMA-262 error intrinsic names. See the module docs.
        ///
        /// This is a closed domain: `match` over it without a catch-all, so that
        /// a tenth kind is `error[E0004]` at every consumer rather than a silent
        /// fall-through to `Error`.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(u8)]
        pub enum NativeErrorKind {
            $(
                $(#[$variant_meta])*
                $variant,
            )+
        }

        impl NativeErrorKind {
            /// Every error intrinsic name, in declaration order.
            ///
            /// The length is written into the type: adding a row without
            /// updating it is `error[E0308]`, and the tie between this order and
            /// the `#[repr(u8)]` discriminants is checked by the
            /// `all_is_ordered_and_round_trips` const assertion below.
            pub const ALL: [NativeErrorKind; 9] = [$(NativeErrorKind::$variant,)+];

            /// The single spelling authority for these names in this workspace
            /// (contract invariant E2).
            ///
            /// `names.rs`'s nine `*_ERROR_NAME` consts are defined *from* this
            /// function, so there is exactly one literal per name in the crate.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(NativeErrorKind::$variant => $spelling,)+
                }
            }

            /// The only parse. Total on the domain, `None` off it.
            #[must_use]
            pub const fn from_str(name: &str) -> Option<Self> {
                $(
                    if str_eq(name, $spelling) {
                        return Some(NativeErrorKind::$variant);
                    }
                )+
                None
            }

            /// The intrinsic constructor this name denotes (invariant E3,
            /// forward direction).
            ///
            /// Total by construction, which is why `infer_expr_throw_info` calls
            /// this instead of writing its own nine-arm match: a tenth kind
            /// cannot be omitted at a call site that does not enumerate.
            #[must_use]
            pub const fn constructor(self) -> StandardBuiltinId {
                match self {
                    $(NativeErrorKind::$variant => StandardBuiltinId::$constructor,)+
                }
            }

            /// Invariant E3, reverse direction. Partial: `StandardBuiltinId` has
            /// hundreds of variants and only nine are error constructors.
            ///
            /// Its consumer is the `all_is_ordered_and_round_trips` const
            /// assertion, which uses the round trip to prove
            /// [`NativeErrorKind::constructor`] is injective — i.e. that no two
            /// error kinds share a prototype.
            #[must_use]
            pub const fn from_constructor(id: StandardBuiltinId) -> Option<Self> {
                match id {
                    $(StandardBuiltinId::$constructor => Some(NativeErrorKind::$variant),)+
                    _ => None,
                }
            }
        }
    };
}

native_error_kinds! {
    /// §20.5.1. The shared superclass, **not** a §20.5.5 NativeError.
    Error => "Error", ErrorConstructor;
    /// §20.5.5 NativeError.
    EvalError => "EvalError", EvalErrorConstructor;
    /// §20.5.5 NativeError.
    RangeError => "RangeError", RangeErrorConstructor;
    /// §20.5.5 NativeError.
    ReferenceError => "ReferenceError", ReferenceErrorConstructor;
    /// §20.5.5 NativeError.
    SyntaxError => "SyntaxError", SyntaxErrorConstructor;
    /// §20.5.5 NativeError.
    TypeError => "TypeError", TypeErrorConstructor;
    /// §20.5.5 NativeError.
    URIError => "URIError", URIErrorConstructor;
    /// §20.5.7. Its own clause, with an extra `errors` argument, so the §20.5.6
    /// template does not produce it: **not** a §20.5.5 NativeError.
    AggregateError => "AggregateError", AggregateErrorConstructor;
    /// Explicit Resource Management. Its own clause, with `error` and
    /// `suppressed` arguments: **not** a §20.5.5 NativeError.
    SuppressedError => "SuppressedError", SuppressedErrorConstructor;
}

const fn kind_eq(a: NativeErrorKind, b: NativeErrorKind) -> bool {
    a as u8 == b as u8
}

const fn found_is(found: Option<NativeErrorKind>, expected: NativeErrorKind) -> bool {
    match found {
        Some(kind) => kind_eq(kind, expected),
        None => false,
    }
}

/// Contract assertions N3, N4 and N6, in one walk of `NativeErrorKind::ALL`.
///
/// - N3: `ALL` is in discriminant order and complete, so `ALL[i]` and the
///   `#[repr(u8)]` tag agree.
/// - N4: `from_str(k.as_str()) == Some(k)` — parse and print cannot diverge.
/// - N6: `from_constructor(k.constructor()) == Some(k)` — the round trip proves
///   `constructor` is injective on the domain, i.e. no two error kinds map to
///   one intrinsic constructor and therefore share a prototype.
const fn all_is_ordered_and_round_trips() -> bool {
    let all = NativeErrorKind::ALL;
    let mut i = 0;
    while i < all.len() {
        if all[i] as u8 != i as u8 {
            return false;
        }
        if !found_is(NativeErrorKind::from_str(all[i].as_str()), all[i]) {
            return false;
        }
        if !found_is(
            NativeErrorKind::from_constructor(all[i].constructor()),
            all[i],
        ) {
            return false;
        }
        i += 1;
    }
    true
}

/// Contract assertion N5: no two kinds share a spelling.
///
/// Two kinds with the same `as_str()` would make the second unreachable through
/// `from_str`, so one of them would silently never parse.
const fn spellings_are_distinct() -> bool {
    let all = NativeErrorKind::ALL;
    let mut i = 0;
    while i < all.len() {
        let mut j = i + 1;
        while j < all.len() {
            if str_eq(all[i].as_str(), all[j].as_str()) {
                return false;
            }
            j += 1;
        }
        i += 1;
    }
    true
}

const _: () = assert!(
    all_is_ordered_and_round_trips(),
    "NativeErrorKind::ALL must be in declaration order, and as_str/constructor must round-trip through from_str/from_constructor"
);
const _: () = assert!(
    spellings_are_distinct(),
    "two NativeErrorKind variants share an as_str() spelling"
);
