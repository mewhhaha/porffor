//! ECMA-262 §6.1.5.1 Table 1, *Well-Known Symbols*, plus the two rows added by
//! Explicit Resource Management. Thirteen plus two, exactly fifteen.
//!
//! > Well-known symbols are built-in Symbol values that are explicitly
//! > referenced by algorithms of this specification. They are typically used as
//! > the keys of properties whose values serve as extension points of a
//! > specification algorithm. — §6.1.5.1
//!
//! Table 1 is exhaustive by construction: §6.1.5.1's normative content *is* the
//! table. The registry symbols of `Symbol.for` / `Symbol.keyFor`
//! (§19.4.2.2/§19.4.2.6) are a different, **open** domain and are deliberately
//! not modelled here; `Symbol.prototype` is a property of the constructor, not a
//! symbol. That is why `test262/…/built-ins/Symbol/` has 18 subdirectories and
//! not 15.
//!
//! # Three strings per symbol
//!
//! Each row denotes three distinct strings in this compiler, and confusing them
//! is a defect class in its own right. The enum names all three separately and
//! implements no `Display`, so no call site can stringify without choosing:
//!
//! | String | Example | Accessor | Consumed by |
//! |---|---|---|---|
//! | member name | `"iterator"` | [`WellKnownSymbol::member_name`] | §19.4.2's key on the `Symbol` intrinsic; source text |
//! | description | `"Symbol.iterator"` | [`WellKnownSymbol::description`] | Table 1's `[[Description]]`, **and this compiler's runtime string encoding of the symbol value** |
//! | shape namespace key | `"@@Symbol.iterator"` | [`shape_namespace_key`] | user object-literal shape maps, which must be unreachable from string-keyed reads |
//!
//! [`WellKnownSymbol::description`] is generated as
//! `concat!("Symbol.", member_name)`, so the S2 coherence invariant
//! (`description == "Symbol." ++ member_name`) is not asserted — it is
//! *definitional*, and a row cannot express a violation of it.
//!
//! # Why an enum and not the fifteen literals
//!
//! Before this module there were **two** byte-identical hand-kept fifteen-element
//! `matches!` whitelists in `lowering.rs`, 5,700 lines apart, each preceded by a
//! byte-identical four-clause "is this the real builtin `Symbol`?" guard. That is
//! the same defect shape already recorded at `names.rs:190-200`
//! (`intl402/DateTimeFormat/prop-desc.js`), alive for a second closed set. Both
//! whitelists are gone; [`WellKnownSymbol::from_member_name`] is the only parse.
//! See `docs/rust-rewrite/contracts/closed-name-domains.md`.
//!
//! Deliberately absent, and deliberately never to be added: `Display`,
//! `AsRef<str>`, `Deref<Target = str>`, `FromStr`, `Default`, and
//! `From<WellKnownSymbol> for String`. With three strings per variant an
//! implicit stringification is not merely loose, it is *ambiguous*.

use crate::ir::SYMBOL_SHAPE_PROPERTY_PREFIX;

/// Byte-wise `&str` equality usable in a `const` initializer. Private for the
/// same reason as `native_error.rs`'s copy: a `pub` one would be crate surface
/// with no product call site.
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

/// The `[[Description]]` prefix shared by every row of §6.1.5.1 Table 1.
///
/// This is *not* the same string as [`SYMBOL_SHAPE_PROPERTY_PREFIX`], which
/// marks a shape-map entry as symbol-keyed. See the module docs.
const SYMBOL_DESCRIPTION_PREFIX: &str = "Symbol.";

/// A member name of the `Symbol` intrinsic (§19.4.2) — `"iterator"`.
///
/// This and [`SymbolDescription`] exist only at the parse boundary, and only
/// because `from_member_name` and `from_description` otherwise share the
/// signature `fn(&str) -> Option<Self>`. Handing one the other's domain
/// compiles, returns `None`, and takes the conservative branch — no compile
/// error, no test signal, and the specialization silently stops firing. That is
/// the M1 failure mode the enum was built to kill, relocated to the parse
/// boundary, so it is closed here.
///
/// Unlike the three *output* newtypes the contract rejected under M14, these are
/// not unwrapped at any map boundary: the shape maps stay string-keyed, there
/// are exactly two constructors, and no `Deref` or `From<&str>` is provided —
/// which is the whole point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolMemberName<'a>(&'a str);

impl<'a> SymbolMemberName<'a> {
    #[must_use]
    pub const fn new(name: &'a str) -> Self {
        Self(name)
    }

    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

/// Table 1's `[[Description]]` for a symbol — `"Symbol.iterator"` — which is
/// also this compiler's runtime string encoding of the symbol value. See
/// [`SymbolMemberName`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolDescription<'a>(&'a str);

impl<'a> SymbolDescription<'a> {
    #[must_use]
    pub const fn new(description: &'a str) -> Self {
        Self(description)
    }

    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

macro_rules! well_known_symbols {
    (
        table_1 {
            $(
                $(#[$t1_meta:meta])*
                $t1_variant:ident => $t1_member:literal;
            )+
        }
        explicit_resource_management {
            $(
                $(#[$erm_meta:meta])*
                $erm_variant:ident => $erm_member:literal;
            )+
        }
    ) => {
        /// One of the fifteen well-known symbols. See the module docs.
        ///
        /// A closed domain: prefer `match` without a catch-all wherever the code
        /// genuinely covers all fifteen, so that a sixteenth row is
        /// `error[E0004]` rather than a silently disabled extension point.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(u8)]
        pub enum WellKnownSymbol {
            $(
                $(#[$t1_meta])*
                $t1_variant,
            )+
            $(
                $(#[$erm_meta])*
                $erm_variant,
            )+
        }

        impl WellKnownSymbol {
            /// All fifteen, Table 1 order first, then the two Explicit Resource
            /// Management additions.
            ///
            /// The length is written into the type: adding a row without
            /// updating it is `error[E0308]`.
            pub const ALL: [WellKnownSymbol; 15] = [
                $(WellKnownSymbol::$t1_variant,)+
                $(WellKnownSymbol::$erm_variant,)+
            ];

            /// §6.1.5.1 Table 1 as of ES2024, in table order.
            pub const TABLE_1: [WellKnownSymbol; 13] = [$(WellKnownSymbol::$t1_variant,)+];

            /// Explicit Resource Management's two additions to the same table.
            ///
            /// Named separately rather than buried in `ALL` so that "why is this
            /// set fifteen and not thirteen?" has an answer in the type.
            pub const EXPLICIT_RESOURCE_MANAGEMENT: [WellKnownSymbol; 2] =
                [$(WellKnownSymbol::$erm_variant,)+];

            /// The key of the corresponding property of the `Symbol` intrinsic
            /// (§19.4.2) — e.g. `"iterator"` for `Symbol.iterator`. This is the
            /// spelling that appears in source text.
            #[must_use]
            pub const fn member_name(self) -> &'static str {
                match self {
                    $(WellKnownSymbol::$t1_variant => $t1_member,)+
                    $(WellKnownSymbol::$erm_variant => $erm_member,)+
                }
            }

            /// Table 1's `[[Description]]` column — e.g. `"Symbol.iterator"`.
            ///
            /// This is also this compiler's runtime string encoding of the symbol
            /// *value*: a well-known symbol is an `ExprIr::String(description)`
            /// carrying `ValueKind::Symbol`. The `ValueKind` is what
            /// distinguishes it from an ordinary string.
            #[must_use]
            pub const fn description(self) -> &'static str {
                match self {
                    $(WellKnownSymbol::$t1_variant => concat!("Symbol.", $t1_member),)+
                    $(WellKnownSymbol::$erm_variant => concat!("Symbol.", $erm_member),)+
                }
            }

            /// The only parse from source text. Replaced both hand-kept
            /// `matches!` whitelists in `lowering.rs`.
            ///
            /// Takes a [`SymbolMemberName`] rather than a `&str` so that passing
            /// a description here — which compiled, returned `None`, and
            /// silently disabled the specialization — is `E0308`.
            #[must_use]
            pub const fn from_member_name(name: SymbolMemberName<'_>) -> Option<Self> {
                let name = name.as_str();
                $(
                    if str_eq(name, $t1_member) {
                        return Some(WellKnownSymbol::$t1_variant);
                    }
                )+
                $(
                    if str_eq(name, $erm_member) {
                        return Some(WellKnownSymbol::$erm_variant);
                    }
                )+
                None
            }

            /// Recovers the symbol from its runtime string encoding.
            ///
            /// `None` for a `"Symbol."`-prefixed description outside the fifteen:
            /// the description namespace is open (a program may compute one), the
            /// well-known set is not.
            ///
            /// Takes a [`SymbolDescription`] for the same reason
            /// [`WellKnownSymbol::from_member_name`] takes a
            /// [`SymbolMemberName`].
            #[must_use]
            pub const fn from_description(description: SymbolDescription<'_>) -> Option<Self> {
                let description = description.as_str();
                $(
                    if str_eq(description, concat!("Symbol.", $t1_member)) {
                        return Some(WellKnownSymbol::$t1_variant);
                    }
                )+
                $(
                    if str_eq(description, concat!("Symbol.", $erm_member)) {
                        return Some(WellKnownSymbol::$erm_variant);
                    }
                )+
                None
            }
        }
    };
}

well_known_symbols! {
    table_1 {
        /// §7.4.3 GetIterator, async form — tried *before* `@@iterator`.
        AsyncIterator => "asyncIterator";
        /// §13.10.2 InstanceofOperator step 2.
        HasInstance => "hasInstance";
        /// §23.1.3.1.1 IsConcatSpreadable.
        IsConcatSpreadable => "isConcatSpreadable";
        /// §7.4.3 GetIterator, sync form.
        Iterator => "iterator";
        /// §22.1.3.x `String.prototype.match`.
        Match => "match";
        /// §22.1.3.x `String.prototype.matchAll`.
        MatchAll => "matchAll";
        /// §22.1.3.x `String.prototype.replace`/`replaceAll`.
        Replace => "replace";
        /// §22.1.3.x `String.prototype.search`.
        Search => "search";
        /// §7.3 SpeciesConstructor.
        Species => "species";
        /// §22.1.3.x `String.prototype.split`.
        Split => "split";
        /// §7.1.1 ToPrimitive step 2.
        ToPrimitive => "toPrimitive";
        /// §20.1.3.6 `Object.prototype.toString` step 15.
        ToStringTag => "toStringTag";
        /// §9.1.1.2.1 object Environment Record `HasBinding`, i.e. `with`.
        Unscopables => "unscopables";
    }
    explicit_resource_management {
        /// Explicit Resource Management: `using`.
        Dispose => "dispose";
        /// Explicit Resource Management: `await using`.
        AsyncDispose => "asyncDispose";
    }
}

/// The shape-map key for a symbol-keyed entry: `"@@" ++ description()`.
///
/// A separate name from [`WellKnownSymbol::description`] on purpose, and it must
/// stay one. `ObjectShape::properties` is a string-key map consulted by
/// string-keyed reads, so a symbol-keyed entry has to live under a name no
/// string-keyed read resolves — `read_heap_shape_property` filters on the `@@`
/// prefix. Handing it a bare description instead would let
/// `obj["Symbol.iterator"]` reach a symbol-keyed entry, which §6.1.5 forbids.
#[must_use]
pub fn shape_namespace_key(symbol: WellKnownSymbol) -> String {
    format!("{SYMBOL_SHAPE_PROPERTY_PREFIX}{}", symbol.description())
}

/// Whether `name` lies in this compiler's symbol-value string namespace.
///
/// Read the name as `is_in_symbol_value_namespace`: it is a prefix test, so it
/// answers `true` for `"Symbol."`, `"Symbol.for"`, `"Symbol.keyFor"` and
/// `"Symbol.prototype"`, none of which is a symbol value and the first three of
/// which exist as literals in `builtins.rs`. That is the intended, honest
/// behaviour of a namespace test — it is only misleading if the name is read as
/// a well-known-symbol test.
///
/// An **open** predicate over an **open** domain, and not derivable from
/// [`WellKnownSymbol`]: a program can produce a `ValueKind::Symbol` string that
/// is not one of the fifteen. Callers that need a well-known symbol must use
/// [`WellKnownSymbol::from_description`]; this answers only "is this in the
/// namespace at all". It is contract ledger entry R2.
#[must_use]
pub const fn is_symbol_description(name: &str) -> bool {
    let prefix = SYMBOL_DESCRIPTION_PREFIX.as_bytes();
    let bytes = name.as_bytes();
    if bytes.len() < prefix.len() {
        return false;
    }
    let mut i = 0;
    while i < prefix.len() {
        if bytes[i] != prefix[i] {
            return false;
        }
        i += 1;
    }
    true
}

const fn symbol_eq(a: WellKnownSymbol, b: WellKnownSymbol) -> bool {
    a as u8 == b as u8
}

const fn found_is(found: Option<WellKnownSymbol>, expected: WellKnownSymbol) -> bool {
    match found {
        Some(symbol) => symbol_eq(symbol, expected),
        None => false,
    }
}

/// Contract assertions W3, W4, W5 and W8, in one walk of `WellKnownSymbol::ALL`.
///
/// - W3: `ALL` is in discriminant order and complete.
/// - W4: `from_member_name(k.member_name()) == Some(k)`.
/// - W5: `from_description(k.description()) == Some(k)`.
/// - W8: every generated description lies in the namespace
///   `is_symbol_description` recognises, so the predicate and the encoding
///   cannot disagree.
const fn all_is_ordered_and_round_trips() -> bool {
    let all = WellKnownSymbol::ALL;
    let mut i = 0;
    while i < all.len() {
        if all[i] as u8 != i as u8 {
            return false;
        }
        if !found_is(
            WellKnownSymbol::from_member_name(SymbolMemberName::new(all[i].member_name())),
            all[i],
        ) {
            return false;
        }
        if !found_is(
            WellKnownSymbol::from_description(SymbolDescription::new(all[i].description())),
            all[i],
        ) {
            return false;
        }
        if !is_symbol_description(all[i].description()) {
            return false;
        }
        i += 1;
    }
    true
}

/// Contract assertion W7: no two rows share a member name or a description.
///
/// A shared spelling would make the second row unreachable through
/// `from_member_name`/`from_description` — a variant that parses as another.
const fn spellings_are_distinct() -> bool {
    let all = WellKnownSymbol::ALL;
    let mut i = 0;
    while i < all.len() {
        let mut j = i + 1;
        while j < all.len() {
            if str_eq(all[i].member_name(), all[j].member_name()) {
                return false;
            }
            if str_eq(all[i].description(), all[j].description()) {
                return false;
            }
            j += 1;
        }
        i += 1;
    }
    true
}

/// Contract assertion W2: `TABLE_1` and `EXPLICIT_RESOURCE_MANAGEMENT` partition
/// `ALL` exactly, in order.
///
/// The lengths alone are carried by the array types; what this adds is that the
/// two named groups really *are* the fifteen, so a row added to `ALL` cannot
/// escape being classified as either ES2024 Table 1 or an ERM addition.
///
/// Stated plainly, because the contract used to claim the opposite: under the
/// current macro this **cannot fail**. `ALL`, `TABLE_1` and
/// `EXPLICIT_RESOURCE_MANAGEMENT` are three expansions of the same two `$(...)+`
/// sequences in the same order, so the concatenation identity is definitional.
/// It is kept as executable documentation of the partition, and it is the check
/// that would start being able to fail the moment either group stops being a
/// direct expansion. The assertions that *can* fail today are W7 (a duplicate
/// spelling is writable in a row list) and W8 (a change to
/// `SYMBOL_DESCRIPTION_PREFIX` is a real edit); the discriminant-order halves of
/// W3 are definitional for the same reason as this one.
const fn partition_covers_all() -> bool {
    let all = WellKnownSymbol::ALL;
    let table_1 = WellKnownSymbol::TABLE_1;
    let erm = WellKnownSymbol::EXPLICIT_RESOURCE_MANAGEMENT;
    if table_1.len() + erm.len() != all.len() {
        return false;
    }
    let mut i = 0;
    while i < table_1.len() {
        if !symbol_eq(table_1[i], all[i]) {
            return false;
        }
        i += 1;
    }
    let mut j = 0;
    while j < erm.len() {
        if !symbol_eq(erm[j], all[table_1.len() + j]) {
            return false;
        }
        j += 1;
    }
    true
}

const _: () = assert!(
    all_is_ordered_and_round_trips(),
    "WellKnownSymbol::ALL must be in declaration order, and member_name/description must round-trip and stay inside SYMBOL_DESCRIPTION_PREFIX"
);
const _: () = assert!(
    spellings_are_distinct(),
    "two WellKnownSymbol variants share a member_name() or description()"
);
const _: () = assert!(
    partition_covers_all(),
    "WellKnownSymbol::TABLE_1 ++ EXPLICIT_RESOURCE_MANAGEMENT must equal ALL, in order"
);
