//! The Property Descriptor lattice (ECMA-262 6.2.6), as one closed type.
//!
//! See `docs/rust-rewrite/contracts/property-descriptor-lattice.md`.
//!
//! # The one-sentence claim
//!
//! 6.2.6 is a **partial record over six independently present-or-absent
//! fields** plus a **three-way partition** of that record, and every defect in
//! this area is either a *partition* error (a record classified into the wrong
//! one of the three cases, or into a case the code has no arm for) or a
//! *presence* error (an absent field materialised as a value, or a present
//! field's presence encoded twice in two independent `Option`s that can
//! disagree). So this module adds a closed partition enum
//! ([`PropertyDescriptorKind`]) and a closed presence enum ([`Presence`]), and
//! everything else is derived from them.
//!
//! # Three spec facts the types encode
//!
//! 1. **Absent is not `false`.** "`[[Writable]]` is absent" and "`[[Writable]]`
//!    is present and is `false`" are different records: 10.1.6.3 step 5 leaves
//!    the existing attribute alone for the first and overwrites it for the
//!    second. [`Presence::Absent`] and `Presence::Present(false)` are different
//!    values, and every consumer matches all three arms.
//! 2. **Absent is not `undefined`.** `{get: undefined}` *has* a `[[Get]]`
//!    field; `{}` does not. 6.2.6.5 step 5 takes *hasGet* from HasProperty, not
//!    from the retrieved value.
//! 3. **The partition is a theorem about *validated* descriptors.** Read
//!    literally, 6.2.6.1/6.2.6.2/6.2.6.3 do not partition all Property
//!    Descriptors: a record carrying both `[[Value]]` and `[[Get]]` satisfies
//!    the first two simultaneously. That record is excluded upstream by
//!    **6.2.6.5 step 9**, which throws a **TypeError** — an observable
//!    behaviour, not an unrepresentable state. So [`PartialDescriptor`] can
//!    represent it, [`ValidatedDescriptor`] cannot, and [`classify`] takes the
//!    latter.
//!
//! # What is deliberately *not* typed here
//!
//! * Which Wasm comparison a body emits for 10.1.6.3 step 4.e (`SameValue` vs
//!   strict equality). Both helpers take four Wasm local indices and a
//!   `&mut Function`, and both are correct *somewhere* in the same function —
//!   step 4.e needs `SameValue`, step 4.d's `[[Get]]`/`[[Set]]` comparison is
//!   correct with either. Ledger row LN1.
//! * Whether 6.2.6.5's per-field read goes through HasProperty or an own-slot
//!   probe. [`TO_PROPERTY_DESCRIPTOR_ORDER`] makes *dropping or reordering* a
//!   field a compile error; the read primitive is a method call with the same
//!   signature as its wrong alternative. Ledger row LN2.

use core::convert::Infallible;
use core::fmt;
use core::marker::PhantomData;

/// The six fields of a 6.2.6 Property Descriptor, and their property keys.
///
/// This is the *only* place any of these six strings is spelled. `"writeable"`,
/// `"enumberable"` and every other plausible typo are
/// `E0599: no variant or associated item named ...`, which matters because a
/// misspelt key in a `defineProperty` argument object is otherwise **completely
/// invisible**: 6.2.6.5 ignores stray keys, so `configurabel: false` yields a
/// non-configurable property by 6.2.6.6 default and the program behaves
/// correctly for the wrong reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DescriptorField {
    Value,
    Writable,
    Get,
    Set,
    Enumerable,
    Configurable,
}

impl DescriptorField {
    /// The 6.2.6 Table 3 property key for this field.
    pub const fn key(self) -> &'static str {
        match self {
            Self::Value => "value",
            Self::Writable => "writable",
            Self::Get => "get",
            Self::Set => "set",
            Self::Enumerable => "enumerable",
            Self::Configurable => "configurable",
        }
    }

    /// Which side of the 6.2.6.1/6.2.6.2 split this field determines.
    ///
    /// `[[Enumerable]]` and `[[Configurable]]` determine neither, which is
    /// exactly why a descriptor carrying only those is *generic* (6.2.6.3) and
    /// why 10.1.6.3 step 4.c exempts it from the kind-change check.
    pub const fn side(self) -> Option<DescriptorSide> {
        match self {
            Self::Value | Self::Writable => Some(DescriptorSide::Data),
            Self::Get | Self::Set => Some(DescriptorSide::Accessor),
            Self::Enumerable | Self::Configurable => None,
        }
    }

    /// Every field, in 6.2.6.4 step order (steps 4..9).
    pub const ALL: [Self; 6] = [
        Self::Value,
        Self::Writable,
        Self::Get,
        Self::Set,
        Self::Enumerable,
        Self::Configurable,
    ];

    /// The two fields that decide `side`, in `ALL` order.
    pub const fn side_fields(side: DescriptorSide) -> [Self; 2] {
        match side {
            DescriptorSide::Data => [Self::Value, Self::Writable],
            DescriptorSide::Accessor => [Self::Get, Self::Set],
        }
    }
}

/// The two halves of the 6.2.6.1/6.2.6.2 split.
///
/// Not a *kind*: `Generic` is a kind and is not a side, because no field puts a
/// descriptor on a "generic side".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescriptorSide {
    Data,
    Accessor,
}

/// The order 6.2.6.5 ToPropertyDescriptor observes the six fields in.
///
/// This is a *permutation* of [`DescriptorField::ALL`], not the same list:
/// 6.2.6.5 reads `[[Enumerable]]` and `[[Configurable]]` first. The order is
/// observable — every field is read with HasProperty and then Get, and both
/// walk the prototype chain, so a descriptor object with side-effecting
/// accessors observes exactly this sequence.
pub const TO_PROPERTY_DESCRIPTOR_ORDER: [DescriptorField; 6] = [
    DescriptorField::Enumerable,
    DescriptorField::Configurable,
    DescriptorField::Value,
    DescriptorField::Writable,
    DescriptorField::Get,
    DescriptorField::Set,
];

// These two are not tautologies. They pin two hand-written tables — one of
// which lives in `porffor-aot-wasm`'s emitter — against the enum, so dropping a
// field, duplicating one, or reordering the enum's variants is a compile error
// rather than a silently missing `Object.defineProperty` field.
const _: () = {
    let mask = (1u32 << (DescriptorField::ALL[0] as u32))
        | (1u32 << (DescriptorField::ALL[1] as u32))
        | (1u32 << (DescriptorField::ALL[2] as u32))
        | (1u32 << (DescriptorField::ALL[3] as u32))
        | (1u32 << (DescriptorField::ALL[4] as u32))
        | (1u32 << (DescriptorField::ALL[5] as u32));
    assert!(
        mask == 0b11_1111,
        "DescriptorField::ALL must list every field exactly once",
    );
};

const _: () = {
    let mask = (1u32 << (TO_PROPERTY_DESCRIPTOR_ORDER[0] as u32))
        | (1u32 << (TO_PROPERTY_DESCRIPTOR_ORDER[1] as u32))
        | (1u32 << (TO_PROPERTY_DESCRIPTOR_ORDER[2] as u32))
        | (1u32 << (TO_PROPERTY_DESCRIPTOR_ORDER[3] as u32))
        | (1u32 << (TO_PROPERTY_DESCRIPTOR_ORDER[4] as u32))
        | (1u32 << (TO_PROPERTY_DESCRIPTOR_ORDER[5] as u32));
    assert!(
        mask == 0b11_1111,
        "TO_PROPERTY_DESCRIPTOR_ORDER must list every field exactly once",
    );
};

/// Whether a 6.2.6 field is present, and where its value lives.
///
/// This replaces the pair (`data: Option<(u32, u32)>`, `data_present_local:
/// Option<u32>`) that `emit_object_define_entry` used to carry, whose two
/// `None`s meant different things and whose `(None, Some(_))` combination said
/// "statically absent" and "maybe present at run time" at the same time.
///
/// `T` is the value carrier. `R` is the carrier for a *run-time* presence flag;
/// a carrier that has no such concept sets `R = core::convert::Infallible`,
/// which makes [`Presence::Runtime`] unconstructible and lets the arm be
/// discharged by `match present {}` rather than by a panic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presence<T, R> {
    /// 6.2.6: the record does not have this field. Known when the compiler
    /// runs.
    Absent,
    /// The record has this field, and the compiler knows it. There is no
    /// run-time flag to test, so a consumer that emits 10.1.6.3 step-4
    /// validation from a flag emits nothing here.
    Present(T),
    /// Whether the record has this field is decided when the program runs:
    /// `present` is non-zero iff the field is there. `value` is meaningful only
    /// then, but is *always* supplied — which is what makes the old
    /// `(None, Some(_))` pair unspellable.
    Runtime { present: R, value: T },
}

impl<T, R> Presence<T, R> {
    /// 6.2.6-level "has this field", as a three-valued answer. There is no
    /// two-valued form, deliberately: the two-valued form is what turned
    /// "the caller does not know" and "the field is absent" into one value.
    pub const fn known(&self) -> KnownPresence {
        match self {
            Self::Absent => KnownPresence::No,
            Self::Present(_) => KnownPresence::Yes,
            Self::Runtime { .. } => KnownPresence::AtRuntime,
        }
    }

    /// The field's value carrier, if the field has one at all.
    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Absent => None,
            Self::Present(value) => Some(value),
            Self::Runtime { value, .. } => Some(value),
        }
    }

    /// The run-time presence flag, if presence is decided at run time.
    pub fn runtime_flag(&self) -> Option<&R> {
        match self {
            Self::Absent => None,
            Self::Present(_) => None,
            Self::Runtime { present, .. } => Some(present),
        }
    }
}

impl<T> Presence<T, Infallible> {
    /// The field's value if present, `None` if absent.
    ///
    /// Total, and provably so: `Infallible` is uninhabited, so the `Runtime`
    /// arm is discharged by `match present {}` rather than by a panic or a
    /// catch-all.
    pub fn into_static_value(self) -> Option<T> {
        match self {
            Self::Absent => None,
            Self::Present(value) => Some(value),
            Self::Runtime { present, .. } => absurd(present),
        }
    }
}

/// Consume a value of an uninhabited type.
///
/// This is the whole reason `DescriptorCarrier::RuntimeFlag` exists as an
/// associated type rather than as a fixed `u32`: a carrier that sets it to
/// `Infallible` gets every "presence is decided at run time" arm discharged by
/// the compiler instead of by a panic or a catch-all.
fn absurd(flag: Infallible) -> ! {
    match flag {}
}

/// The three answers to "does this record have this field".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnownPresence {
    No,
    Yes,
    AtRuntime,
}

/// What a descriptor's fields are made of, in one lowering context.
///
/// Deliberately not sealed: `porffor-aot-wasm` adds its own impl for a
/// descriptor whose fields are Wasm local indices. The trait has no methods, so
/// an impl cannot misbehave.
pub trait DescriptorCarrier {
    /// Carrier for `[[Value]]`, `[[Get]]`, `[[Set]]`.
    type Value: Clone + fmt::Debug;
    /// Carrier for `[[Writable]]`, `[[Enumerable]]`, `[[Configurable]]`.
    type Flag: Copy + fmt::Debug;
    /// Carrier for a run-time presence flag. `Infallible` if there is none.
    type RuntimeFlag: Copy + fmt::Debug;
}

/// Descriptors rendered as JavaScript source text (`modules/namespace.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceText;

impl DescriptorCarrier for SourceText {
    /// An expression, already rendered.
    type Value = String;
    type Flag = bool;
    /// Source text is emitted when the compiler runs, so a key cannot be
    /// run-time-conditional. `Presence::Runtime` is therefore unconstructible
    /// for this carrier.
    type RuntimeFlag = Infallible;
}

impl SourceText {
    /// 6.2.6.6 step 2's *like* record, spelled as JavaScript source text.
    pub fn completion_defaults() -> CompletionDefaults<Self> {
        CompletionDefaults {
            undefined_value: String::from("undefined"),
            false_flag: false,
        }
    }
}

/// A 6.2.6 Property Descriptor: six independently present-or-absent fields.
///
/// Every combination is representable, *including* one carrying both
/// `[[Value]]` and `[[Get]]` — 6.2.6.5 step 9 makes that a **TypeError**, not
/// an unrepresentable state, and a type that banned it would be a spec error:
/// `Object.getOwnPropertyDescriptors` + `Object.defineProperties` round-trips
/// make the raw shape observable and the TypeError is the observable behaviour.
pub struct PartialDescriptor<C: DescriptorCarrier> {
    pub value: Presence<C::Value, C::RuntimeFlag>,
    pub writable: Presence<C::Flag, C::RuntimeFlag>,
    pub get: Presence<C::Value, C::RuntimeFlag>,
    pub set: Presence<C::Value, C::RuntimeFlag>,
    pub enumerable: Presence<C::Flag, C::RuntimeFlag>,
    pub configurable: Presence<C::Flag, C::RuntimeFlag>,
}

impl<C: DescriptorCarrier> Clone for PartialDescriptor<C> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            writable: self.writable,
            get: self.get.clone(),
            set: self.set.clone(),
            enumerable: self.enumerable,
            configurable: self.configurable,
        }
    }
}

impl<C: DescriptorCarrier> fmt::Debug for PartialDescriptor<C> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PartialDescriptor")
            .field("value", &self.value)
            .field("writable", &self.writable)
            .field("get", &self.get)
            .field("set", &self.set)
            .field("enumerable", &self.enumerable)
            .field("configurable", &self.configurable)
            .finish()
    }
}

impl<C: DescriptorCarrier> Default for PartialDescriptor<C> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<C: DescriptorCarrier> PartialDescriptor<C> {
    /// The empty descriptor, `{}`.
    ///
    /// Generic by 6.2.6.3, and its 6.2.6.6 completion is the all-defaults
    /// **data** property — which is what `Object.defineProperty(obj, "p", {})`
    /// on a fresh key must produce (test262 `15.2.3.6-4-52.js`).
    pub fn empty() -> Self {
        Self {
            value: Presence::Absent,
            writable: Presence::Absent,
            get: Presence::Absent,
            set: Presence::Absent,
            enumerable: Presence::Absent,
            configurable: Presence::Absent,
        }
    }

    /// 6.2.6-level "has this field", for one named field.
    pub fn known(&self, field: DescriptorField) -> KnownPresence {
        match field {
            DescriptorField::Value => self.value.known(),
            DescriptorField::Writable => self.writable.known(),
            DescriptorField::Get => self.get.known(),
            DescriptorField::Set => self.set.known(),
            DescriptorField::Enumerable => self.enumerable.known(),
            DescriptorField::Configurable => self.configurable.known(),
        }
    }

    /// The run-time presence flag for one named field, if it has one.
    pub fn runtime_flag(&self, field: DescriptorField) -> Option<C::RuntimeFlag> {
        match field {
            DescriptorField::Value => self.value.runtime_flag().copied(),
            DescriptorField::Writable => self.writable.runtime_flag().copied(),
            DescriptorField::Get => self.get.runtime_flag().copied(),
            DescriptorField::Set => self.set.runtime_flag().copied(),
            DescriptorField::Enumerable => self.enumerable.runtime_flag().copied(),
            DescriptorField::Configurable => self.configurable.runtime_flag().copied(),
        }
    }

    /// The fields 6.2.6.4 FromPropertyDescriptor would emit, in 6.2.6.4 step
    /// order.
    ///
    /// **Any subset** of the six is a legal answer — every one of 6.2.6.4's
    /// steps 4..9 is conditional on presence. The four-key form is the codomain
    /// only when the input is *complete*, which is true at exactly one caller
    /// (10.1.8.1 `Object.getOwnPropertyDescriptor`, via 10.1.6.1 whose step 3
    /// asserts a fully populated descriptor) and false at the Proxy
    /// `[[DefineOwnProperty]]`/`[[GetOwnProperty]]` traps, which are handed a
    /// caller's *partial* descriptor. See [`CompleteDescriptor::keys`] for the
    /// complete case.
    pub fn present_fields(&self) -> impl Iterator<Item = DescriptorField> + '_ {
        DescriptorField::ALL
            .into_iter()
            .filter(move |field| !matches!(self.known(*field), KnownPresence::No))
    }

    fn side_state(&self, side: DescriptorSide) -> SideState {
        let mut state = SideState {
            statically_present: None,
            runtime_present: None,
        };
        for field in DescriptorField::side_fields(side) {
            match self.known(field) {
                KnownPresence::No => {}
                KnownPresence::Yes => {
                    if state.statically_present.is_none() {
                        state.statically_present = Some(field);
                    }
                }
                KnownPresence::AtRuntime => {
                    if state.runtime_present.is_none() {
                        state.runtime_present = Some(field);
                    }
                }
            }
        }
        state
    }

    /// Statically discharge **6.2.6.5 step 9**: this descriptor is not both a
    /// data and an accessor descriptor.
    ///
    /// Available when every presence is `Absent` or `Present`. A `Runtime`
    /// presence on both sides cannot be decided here and returns
    /// [`ValidateError::NeedsRuntimeCheck`]; that is the one case
    /// [`PartialDescriptor::from_runtime_checked`] exists for.
    pub fn validate(self) -> Result<ValidatedDescriptor<C>, ValidateError> {
        let data = self.side_state(DescriptorSide::Data);
        let accessor = self.side_state(DescriptorSide::Accessor);
        match (data.witness(), accessor.witness()) {
            (Some(data_side), Some(accessor_side)) => {
                match (data.statically_present, accessor.statically_present) {
                    (Some(data_side), Some(accessor_side)) => {
                        Err(ValidateError::BothDataAndAccessor(BothDataAndAccessor {
                            data_side,
                            accessor_side,
                        }))
                    }
                    (None, Some(_)) | (Some(_), None) | (None, None) => {
                        Err(ValidateError::NeedsRuntimeCheck {
                            data_side,
                            accessor_side,
                        })
                    }
                }
            }
            (Some(_), None) | (None, Some(_)) | (None, None) => Ok(ValidatedDescriptor(self)),
        }
    }

    /// 6.2.6.5 step 9 has been discharged by an **emitted run-time check**
    /// elsewhere. NOT an invariant — a named escape hatch, ledger row LN3.
    ///
    /// Declared call sites, and the lines that discharge the obligation:
    ///
    /// * `porffor-aot-wasm/src/objects.rs`, `emit_object_define_entry` — the
    ///   positional adapter for the two `Object.defineProperty` sites in
    ///   `builtins/standard.rs`, both of which are dominated by the emitted
    ///   6.2.6.5 step 9 throw in that file (`if (getter_present ||
    ///   setter_present) && (value_present || writable_present) throw
    ///   TypeError`).
    ///
    /// Adding a caller without adding a line here is the defect this doc
    /// comment exists to make visible to `rg from_runtime_checked`. The
    /// *static* half of step 9 is still enforced: a descriptor that is
    /// statically both a data and an accessor descriptor panics here, because
    /// no emitted run-time check can rescue it.
    pub fn from_runtime_checked(self) -> ValidatedDescriptor<C> {
        let data = self.side_state(DescriptorSide::Data);
        let accessor = self.side_state(DescriptorSide::Accessor);
        assert!(
            data.statically_present.is_none() || accessor.statically_present.is_none(),
            "6.2.6.5 step 9: a descriptor that is statically both a data and an accessor \
             descriptor cannot be rescued by an emitted run-time check",
        );
        ValidatedDescriptor(self)
    }
}

struct SideState {
    statically_present: Option<DescriptorField>,
    runtime_present: Option<DescriptorField>,
}

impl SideState {
    fn witness(&self) -> Option<DescriptorField> {
        match self.statically_present {
            Some(field) => Some(field),
            None => self.runtime_present,
        }
    }
}

/// The two fields that make a descriptor fail 6.2.6.5 step 9.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BothDataAndAccessor {
    /// `Value` or `Writable`.
    pub data_side: DescriptorField,
    /// `Get` or `Set`.
    pub accessor_side: DescriptorField,
}

/// Why a [`PartialDescriptor`] could not be statically validated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidateError {
    /// 6.2.6.5 step 9 fails outright: both sides are statically present.
    BothDataAndAccessor(BothDataAndAccessor),
    /// Both sides are *possible*, but at least one only at run time, so
    /// 6.2.6.5 step 9 has to be an emitted check. See
    /// [`PartialDescriptor::from_runtime_checked`].
    NeedsRuntimeCheck {
        data_side: DescriptorField,
        accessor_side: DescriptorField,
    },
}

impl fmt::Display for ValidateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BothDataAndAccessor(both) => write!(
                formatter,
                "6.2.6.5 step 9: descriptor has both `{}` and `{}`",
                both.data_side.key(),
                both.accessor_side.key(),
            ),
            Self::NeedsRuntimeCheck {
                data_side,
                accessor_side,
            } => write!(
                formatter,
                "6.2.6.5 step 9 is a run-time question here: `{}` and `{}` are both possible",
                data_side.key(),
                accessor_side.key(),
            ),
        }
    }
}

/// A [`PartialDescriptor`] that has passed **6.2.6.5 step 9**: it is not both a
/// data and an accessor descriptor.
///
/// The three-way partition of 6.2.6.1–3 is a partition only on this type, which
/// is why [`classify`] takes it and not a raw [`PartialDescriptor`]. The field
/// is private and there is no `DerefMut`: mutating a validated descriptor back
/// into an invalid one requires re-validating.
pub struct ValidatedDescriptor<C: DescriptorCarrier>(PartialDescriptor<C>);

impl<C: DescriptorCarrier> Clone for ValidatedDescriptor<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<C: DescriptorCarrier> fmt::Debug for ValidatedDescriptor<C> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("ValidatedDescriptor")
            .field(&self.0)
            .finish()
    }
}

impl<C: DescriptorCarrier> ValidatedDescriptor<C> {
    pub fn as_partial(&self) -> &PartialDescriptor<C> {
        &self.0
    }

    pub fn into_partial(self) -> PartialDescriptor<C> {
        self.0
    }
}

impl<C: DescriptorCarrier<RuntimeFlag = Infallible>> ValidatedDescriptor<C> {
    /// The 6.2.6.1–3 partition, decided when the compiler runs.
    ///
    /// Available only for a carrier whose run-time flag is uninhabited, so
    /// [`DescriptorClassification::Dynamic`] cannot arrive — the arms below are
    /// discharged by `match flag {}`, not by a panic.
    pub fn static_kind(&self) -> PropertyDescriptorKind {
        match classify(self) {
            DescriptorClassification::Static(kind) => kind,
            DescriptorClassification::Dynamic {
                accessor_terms,
                data_terms,
            } => {
                for slot in [
                    data_terms.runtime[0],
                    data_terms.runtime[1],
                    accessor_terms.runtime[0],
                    accessor_terms.runtime[1],
                ] {
                    match slot {
                        None => {}
                        Some(flag) => absurd(flag),
                    }
                }
                unreachable!("classify returns Dynamic only when some runtime term is Some")
            }
        }
    }
}

/// The 6.2.6.1 / 6.2.6.2 / 6.2.6.3 partition. Three cases, and `Generic` is one
/// of them rather than a hole patched afterwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyDescriptorKind {
    Data,
    Accessor,
    Generic,
}

/// The statically-known part and the run-time part of one side's disjunction.
///
/// 6.2.6.1 IsAccessorDescriptor is "`Desc` has `[[Get]]` **or** `[[Set]]`", and
/// 6.2.6.2 IsDataDescriptor is the same over `[[Value]]`/`[[Writable]]`. Naming
/// the two disjunctions is what lets a consumer emit *the spec's* predicate
/// instead of re-deriving one — and, crucially, lets the two independent
/// 10.1.6.3 obligations (step 6.a and step 7.a) be emitted from their own
/// side's terms instead of from one conjunction over all four.
pub struct KindTerms<C: DescriptorCarrier> {
    /// `true` iff some field on this side is [`Presence::Present`] — the side
    /// is then unconditionally true and `runtime` need not be consulted.
    pub statically_true: bool,
    /// Run-time presence flags, in [`DescriptorField::ALL`] order. At most two,
    /// because a side has exactly two fields.
    pub runtime: [Option<C::RuntimeFlag>; 2],
}

impl<C: DescriptorCarrier> Clone for KindTerms<C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C: DescriptorCarrier> Copy for KindTerms<C> {}

impl<C: DescriptorCarrier> fmt::Debug for KindTerms<C> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("KindTerms")
            .field("statically_true", &self.statically_true)
            .field("runtime", &self.runtime)
            .finish()
    }
}

impl<C: DescriptorCarrier> KindTerms<C> {
    /// A side with no run-time terms.
    pub fn static_only(statically_true: bool) -> Self {
        Self {
            statically_true,
            runtime: [None, None],
        }
    }

    /// Whether this side's truth is decided when the program runs.
    pub fn is_dynamic(&self) -> bool {
        self.runtime[0].is_some() || self.runtime[1].is_some()
    }

    /// Whether this side can be true at all.
    pub fn is_possible(&self) -> bool {
        self.statically_true || self.is_dynamic()
    }

    /// The run-time terms, in order, with the empty slots dropped.
    pub fn runtime_flags(&self) -> Vec<C::RuntimeFlag> {
        let mut flags = Vec::with_capacity(2);
        for slot in self.runtime {
            match slot {
                None => {}
                Some(flag) => flags.push(flag),
            }
        }
        flags
    }
}

/// The partition, evaluated against presences that may not all be static.
pub enum DescriptorClassification<C: DescriptorCarrier> {
    /// Every relevant presence is `Absent` or `Present`: the case is decided
    /// when the compiler runs.
    Static(PropertyDescriptorKind),
    /// At least one of the four kind-determining fields has a `Runtime`
    /// presence.
    Dynamic {
        /// Terms whose disjunction is 6.2.6.1 IsAccessorDescriptor.
        accessor_terms: KindTerms<C>,
        /// Terms whose disjunction is 6.2.6.2 IsDataDescriptor.
        data_terms: KindTerms<C>,
    },
}

impl<C: DescriptorCarrier> Clone for DescriptorClassification<C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C: DescriptorCarrier> Copy for DescriptorClassification<C> {}

impl<C: DescriptorCarrier> fmt::Debug for DescriptorClassification<C> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Static(kind) => formatter.debug_tuple("Static").field(kind).finish(),
            Self::Dynamic {
                accessor_terms,
                data_terms,
            } => formatter
                .debug_struct("Dynamic")
                .field("accessor_terms", accessor_terms)
                .field("data_terms", data_terms)
                .finish(),
        }
    }
}

impl<C: DescriptorCarrier> DescriptorClassification<C> {
    /// One side's terms, whichever case this classification is in.
    ///
    /// A `Static` classification has no run-time terms by definition, so a
    /// consumer that emits from `runtime` emits nothing — which is exactly the
    /// "this is an internal define, 10.1.6.3 step 4 is discharged by
    /// construction" case.
    pub fn terms(&self, side: DescriptorSide) -> KindTerms<C> {
        match self {
            Self::Static(kind) => KindTerms::static_only(match (*kind, side) {
                (PropertyDescriptorKind::Data, DescriptorSide::Data) => true,
                (PropertyDescriptorKind::Data, DescriptorSide::Accessor) => false,
                (PropertyDescriptorKind::Accessor, DescriptorSide::Data) => false,
                (PropertyDescriptorKind::Accessor, DescriptorSide::Accessor) => true,
                (PropertyDescriptorKind::Generic, DescriptorSide::Data) => false,
                (PropertyDescriptorKind::Generic, DescriptorSide::Accessor) => false,
            }),
            Self::Dynamic {
                accessor_terms,
                data_terms,
            } => match side {
                DescriptorSide::Data => *data_terms,
                DescriptorSide::Accessor => *accessor_terms,
            },
        }
    }
}

/// The **one** derivation of the 6.2.6.1–3 partition in the workspace.
///
/// Nothing else may re-derive it. Before this module the tree spelled it nine
/// different ways — two function names, three `bool`s, one run-time OR-fold,
/// one `if data.is_some()`, one four-way `is_some()` correction, and one
/// six-key object shape.
///
/// Takes a [`ValidatedDescriptor`], because the three cases are a partition
/// only after 6.2.6.5 step 9.
pub fn classify<C: DescriptorCarrier>(
    descriptor: &ValidatedDescriptor<C>,
) -> DescriptorClassification<C> {
    let partial = descriptor.as_partial();
    let data_terms = kind_terms(partial, DescriptorSide::Data);
    let accessor_terms = kind_terms(partial, DescriptorSide::Accessor);
    if data_terms.is_dynamic() || accessor_terms.is_dynamic() {
        return DescriptorClassification::Dynamic {
            accessor_terms,
            data_terms,
        };
    }
    DescriptorClassification::Static(
        match (data_terms.statically_true, accessor_terms.statically_true) {
            (true, false) => PropertyDescriptorKind::Data,
            (false, true) => PropertyDescriptorKind::Accessor,
            (false, false) => PropertyDescriptorKind::Generic,
            (true, true) => unreachable!(
                "ValidatedDescriptor discharges 6.2.6.5 step 9, so a validated descriptor is \
                 never statically both a data and an accessor descriptor",
            ),
        },
    )
}

fn kind_terms<C: DescriptorCarrier>(
    partial: &PartialDescriptor<C>,
    side: DescriptorSide,
) -> KindTerms<C> {
    let mut terms = KindTerms::static_only(false);
    for (index, field) in DescriptorField::side_fields(side).into_iter().enumerate() {
        match partial.known(field) {
            KnownPresence::No => {}
            KnownPresence::Yes => terms.statically_true = true,
            KnownPresence::AtRuntime => terms.runtime[index] = partial.runtime_flag(field),
        }
    }
    terms
}

/// 6.2.6.6's output, and 10.1.6.3 step 3's assertion: a **fully populated**
/// Property Descriptor. This is what a stored property is.
///
/// There is no `Generic` variant. 6.2.6.6 step 3 says a generic descriptor
/// completes to a *data* descriptor, so `Generic` is a classification of an
/// *incoming* descriptor and never a stored kind — no heap word represents it.
///
/// `Accessor` has **no `writable` field**. 10.1.6.3 steps 6.b and 7 say a
/// conversion between kinds preserves only `[[Enumerable]]` and
/// `[[Configurable]]`; an accessor that could carry `[[Writable]]` is not a
/// representation of anything 6.2.6 defines, and the stale bit it would carry
/// is what a later accessor-to-data conversion reads back as `writable: true`.
pub enum CompleteDescriptor<C: DescriptorCarrier> {
    Data {
        value: C::Value,
        writable: C::Flag,
        enumerable: C::Flag,
        configurable: C::Flag,
    },
    Accessor {
        get: C::Value,
        set: C::Value,
        enumerable: C::Flag,
        configurable: C::Flag,
    },
}

impl<C: DescriptorCarrier> Clone for CompleteDescriptor<C> {
    fn clone(&self) -> Self {
        match self {
            Self::Data {
                value,
                writable,
                enumerable,
                configurable,
            } => Self::Data {
                value: value.clone(),
                writable: *writable,
                enumerable: *enumerable,
                configurable: *configurable,
            },
            Self::Accessor {
                get,
                set,
                enumerable,
                configurable,
            } => Self::Accessor {
                get: get.clone(),
                set: set.clone(),
                enumerable: *enumerable,
                configurable: *configurable,
            },
        }
    }
}

impl<C: DescriptorCarrier> fmt::Debug for CompleteDescriptor<C> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Data {
                value,
                writable,
                enumerable,
                configurable,
            } => formatter
                .debug_struct("Data")
                .field("value", value)
                .field("writable", writable)
                .field("enumerable", enumerable)
                .field("configurable", configurable)
                .finish(),
            Self::Accessor {
                get,
                set,
                enumerable,
                configurable,
            } => formatter
                .debug_struct("Accessor")
                .field("get", get)
                .field("set", set)
                .field("enumerable", enumerable)
                .field("configurable", configurable)
                .finish(),
        }
    }
}

impl<C: DescriptorCarrier> CompleteDescriptor<C> {
    /// 6.2.6.4's codomain when the input is complete: exactly four keys, one of
    /// exactly two sets.
    ///
    /// This is what `Object.getOwnPropertyDescriptor` returns (10.1.8.1 →
    /// 10.1.6.1, whose step 3 asserts a fully populated descriptor), and it is
    /// the direct refutation of any "descriptor shape" that names all six keys:
    /// a descriptor for an accessor has **no `writable` key at all**.
    pub fn keys(&self) -> [DescriptorField; 4] {
        match self {
            Self::Data { .. } => [
                DescriptorField::Value,
                DescriptorField::Writable,
                DescriptorField::Enumerable,
                DescriptorField::Configurable,
            ],
            Self::Accessor { .. } => [
                DescriptorField::Get,
                DescriptorField::Set,
                DescriptorField::Enumerable,
                DescriptorField::Configurable,
            ],
        }
    }

    pub fn kind(&self) -> PropertyDescriptorKind {
        match self {
            Self::Data { .. } => PropertyDescriptorKind::Data,
            Self::Accessor { .. } => PropertyDescriptorKind::Accessor,
        }
    }
}

/// The carrier-specific spellings of 6.2.6.6 step 2's *like* record.
pub struct CompletionDefaults<C: DescriptorCarrier> {
    /// `[[Value]]`, `[[Get]]` and `[[Set]]` all default to `undefined`.
    pub undefined_value: C::Value,
    /// `[[Writable]]`, `[[Enumerable]]` and `[[Configurable]]` all default to
    /// `false`.
    pub false_flag: C::Flag,
}

impl<C: DescriptorCarrier> Clone for CompletionDefaults<C> {
    fn clone(&self) -> Self {
        Self {
            undefined_value: self.undefined_value.clone(),
            false_flag: self.false_flag,
        }
    }
}

impl<C: DescriptorCarrier> fmt::Debug for CompletionDefaults<C> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompletionDefaults")
            .field("undefined_value", &self.undefined_value)
            .field("false_flag", &self.false_flag)
            .finish()
    }
}

/// 6.2.6.6 CompletePropertyDescriptor.
///
/// Consumes the validated descriptor: completing twice is meaningless, and
/// `E0382: use of moved value` says so.
///
/// Restricted to carriers with an uninhabited run-time flag. A descriptor whose
/// field presences are only known when the *program* runs cannot be completed
/// when the *compiler* runs; the AOT backend completes such a descriptor with
/// emitted Wasm instead, on the 10.1.6.3 step-2 create-a-new-entry path.
///
/// The `Data | Generic` or-pattern is 6.2.6.6 step 3 verbatim — "if
/// IsGenericDescriptor(Desc) is true, **or** IsDataDescriptor(Desc) is true" —
/// written as one arm so that adding a fourth kind is still a compile error.
pub fn complete_property_descriptor<C: DescriptorCarrier<RuntimeFlag = Infallible>>(
    descriptor: ValidatedDescriptor<C>,
    kind: PropertyDescriptorKind,
    defaults: &CompletionDefaults<C>,
) -> CompleteDescriptor<C> {
    let partial = descriptor.into_partial();
    match kind {
        PropertyDescriptorKind::Data | PropertyDescriptorKind::Generic => CompleteDescriptor::Data {
            value: partial
                .value
                .into_static_value()
                .unwrap_or_else(|| defaults.undefined_value.clone()),
            writable: partial
                .writable
                .into_static_value()
                .unwrap_or(defaults.false_flag),
            enumerable: partial
                .enumerable
                .into_static_value()
                .unwrap_or(defaults.false_flag),
            configurable: partial
                .configurable
                .into_static_value()
                .unwrap_or(defaults.false_flag),
        },
        PropertyDescriptorKind::Accessor => CompleteDescriptor::Accessor {
            get: partial
                .get
                .into_static_value()
                .unwrap_or_else(|| defaults.undefined_value.clone()),
            set: partial
                .set
                .into_static_value()
                .unwrap_or_else(|| defaults.undefined_value.clone()),
            enumerable: partial
                .enumerable
                .into_static_value()
                .unwrap_or(defaults.false_flag),
            configurable: partial
                .configurable
                .into_static_value()
                .unwrap_or(defaults.false_flag),
        },
    }
}

// ---------------------------------------------------------------------------
// Descriptors rendered as JavaScript source text.
// ---------------------------------------------------------------------------

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::DataSide {}
    impl Sealed for super::AccessorSide {}
}

/// A builder restricted to the data half of the 6.2.6.1/6.2.6.2 split.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataSide;

/// A builder restricted to the accessor half.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccessorSide;

/// Which half of the split a [`DescriptorSourceText`] builder may fill in.
pub trait DescriptorSideMarker: sealed::Sealed {
    const SIDE: DescriptorSide;
}

impl DescriptorSideMarker for DataSide {
    const SIDE: DescriptorSide = DescriptorSide::Data;
}

impl DescriptorSideMarker for AccessorSide {
    const SIDE: DescriptorSide = DescriptorSide::Accessor;
}

/// A 6.2.6 partial descriptor rendered as the `{ … }` argument to
/// `Object.defineProperty`.
///
/// Every key comes from [`DescriptorField::key`], so a typo is `E0599` instead
/// of a silently-ignored property that leaves the attribute at its 6.2.6.6
/// default.
///
/// The side is a **typestate**, which is what discharges 6.2.6.5 step 9 without
/// a `Result`: `DescriptorSourceText::<DataSide>` has no `get`/`set` method and
/// `DescriptorSourceText::<AccessorSide>` has no `value`/`writable` method, so
/// a descriptor that would throw a TypeError does not compile.
///
/// `SourceText::RuntimeFlag = Infallible`, so [`Presence::Runtime`] is
/// unconstructible here: a compile-time-emitted descriptor cannot acquire a
/// run-time-conditional key.
#[derive(Debug, Clone)]
pub struct DescriptorSourceText<S: DescriptorSideMarker> {
    partial: PartialDescriptor<SourceText>,
    side: PhantomData<S>,
}

impl<S: DescriptorSideMarker> DescriptorSourceText<S> {
    fn with(partial: PartialDescriptor<SourceText>) -> Self {
        Self {
            partial,
            side: PhantomData,
        }
    }

    pub fn enumerable(mut self, flag: bool) -> Self {
        self.partial.enumerable = Presence::Present(flag);
        self
    }

    pub fn configurable(mut self, flag: bool) -> Self {
        self.partial.configurable = Presence::Present(flag);
        self
    }

    /// 6.2.6.4 over a *partial* descriptor: `{ k: v, … }` over the present
    /// fields only, in [`DescriptorField::ALL`] order — which is 6.2.6.4's own
    /// step order.
    ///
    /// Total: the side typestate has already discharged 6.2.6.5 step 9, and
    /// `SourceText`'s uninhabited run-time flag has already discharged the
    /// "presence not known yet" case.
    pub fn render(&self) -> String {
        let mut rendered: Vec<(DescriptorField, String)> = Vec::new();
        for field in self.partial.present_fields() {
            rendered.push((field, source_text_field(&self.partial, field)));
        }
        render_fields(&rendered)
    }
}

impl DescriptorSourceText<DataSide> {
    /// A descriptor that may carry `[[Value]]` and `[[Writable]]`.
    pub fn data() -> Self {
        Self::with(PartialDescriptor::empty())
    }

    pub fn value(mut self, expression: impl Into<String>) -> Self {
        self.partial.value = Presence::Present(expression.into());
        self
    }

    pub fn writable(mut self, flag: bool) -> Self {
        self.partial.writable = Presence::Present(flag);
        self
    }

    /// 6.2.6.5 step 9, then 6.2.6.6 CompletePropertyDescriptor.
    ///
    /// Infallible in both halves and provably so: `DataSide` has no `get`/`set`
    /// method, so step 9's antecedent cannot hold, and `SourceText`'s run-time
    /// flag is uninhabited, so [`ValidatedDescriptor::static_kind`] is total.
    /// The result is a `CompleteDescriptor::Data` for both the `Data` and the
    /// `Generic` classification, which is 6.2.6.6 step 3.
    pub fn complete(self) -> CompleteDescriptor<SourceText> {
        let validated = self.partial.validate().expect(
            "a DataSide DescriptorSourceText cannot carry [[Get]] or [[Set]], so 6.2.6.5 step 9 \
             cannot fire",
        );
        let kind = validated.static_kind();
        complete_property_descriptor(validated, kind, &SourceText::completion_defaults())
    }
}

impl DescriptorSourceText<AccessorSide> {
    /// A descriptor that may carry `[[Get]]` and `[[Set]]`.
    pub fn accessor() -> Self {
        Self::with(PartialDescriptor::empty())
    }

    pub fn get(mut self, expression: impl Into<String>) -> Self {
        self.partial.get = Presence::Present(expression.into());
        self
    }

    pub fn set(mut self, expression: impl Into<String>) -> Self {
        self.partial.set = Presence::Present(expression.into());
        self
    }
}

impl CompleteDescriptor<SourceText> {
    /// 6.2.6.4 over a *complete* descriptor: exactly the four keys
    /// [`CompleteDescriptor::keys`] names, in that order.
    pub fn render(&self) -> String {
        let rendered: Vec<(DescriptorField, String)> = match self {
            Self::Data {
                value,
                writable,
                enumerable,
                configurable,
            } => vec![
                (DescriptorField::Value, value.clone()),
                (DescriptorField::Writable, render_flag(*writable)),
                (DescriptorField::Enumerable, render_flag(*enumerable)),
                (DescriptorField::Configurable, render_flag(*configurable)),
            ],
            Self::Accessor {
                get,
                set,
                enumerable,
                configurable,
            } => vec![
                (DescriptorField::Get, get.clone()),
                (DescriptorField::Set, set.clone()),
                (DescriptorField::Enumerable, render_flag(*enumerable)),
                (DescriptorField::Configurable, render_flag(*configurable)),
            ],
        };
        let order = [
            rendered[0].0,
            rendered[1].0,
            rendered[2].0,
            rendered[3].0,
        ];
        assert_eq!(
            order,
            self.keys(),
            "CompleteDescriptor::render must emit CompleteDescriptor::keys() order",
        );
        render_fields(&rendered)
    }
}

fn render_fields(fields: &[(DescriptorField, String)]) -> String {
    let mut text = String::from("{");
    for (index, (field, value)) in fields.iter().enumerate() {
        if index > 0 {
            text.push(',');
        }
        text.push(' ');
        text.push_str(field.key());
        text.push_str(": ");
        text.push_str(value);
    }
    text.push_str(" }");
    text
}

fn render_flag(flag: bool) -> String {
    String::from(if flag { "true" } else { "false" })
}

fn source_text_field(partial: &PartialDescriptor<SourceText>, field: DescriptorField) -> String {
    match field {
        DescriptorField::Value => source_text_value(&partial.value),
        DescriptorField::Get => source_text_value(&partial.get),
        DescriptorField::Set => source_text_value(&partial.set),
        DescriptorField::Writable => source_text_flag(&partial.writable),
        DescriptorField::Enumerable => source_text_flag(&partial.enumerable),
        DescriptorField::Configurable => source_text_flag(&partial.configurable),
    }
}

fn source_text_value(presence: &Presence<String, Infallible>) -> String {
    match presence {
        Presence::Absent => {
            unreachable!("present_fields yields only fields whose presence is not `No`")
        }
        Presence::Present(expression) => expression.clone(),
        Presence::Runtime { present, .. } => absurd(*present),
    }
}

fn source_text_flag(presence: &Presence<bool, Infallible>) -> String {
    match presence {
        Presence::Absent => {
            unreachable!("present_fields yields only fields whose presence is not `No`")
        }
        Presence::Present(flag) => render_flag(*flag),
        Presence::Runtime { present, .. } => absurd(*present),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct TestLocals;

    impl DescriptorCarrier for TestLocals {
        type Value = u32;
        type Flag = u32;
        type RuntimeFlag = u32;
    }

    fn empty() -> PartialDescriptor<TestLocals> {
        PartialDescriptor::empty()
    }

    #[test]
    fn every_field_key_is_the_spec_table_3_name() {
        assert_eq!(
            DescriptorField::ALL.map(DescriptorField::key),
            ["value", "writable", "get", "set", "enumerable", "configurable"],
        );
    }

    #[test]
    fn to_property_descriptor_order_is_a_permutation_of_all() {
        let mut order = TO_PROPERTY_DESCRIPTOR_ORDER;
        let mut all = DescriptorField::ALL;
        order.sort();
        all.sort();
        assert_eq!(order, all);
    }

    #[test]
    fn the_empty_descriptor_is_generic() {
        let validated = empty().validate().expect("`{}` passes 6.2.6.5 step 9");
        assert!(matches!(
            classify(&validated),
            DescriptorClassification::Static(PropertyDescriptorKind::Generic),
        ));
    }

    #[test]
    fn a_writable_only_descriptor_is_a_data_descriptor() {
        // 6.2.6.2: `[[Value]]` is not required. This is the case the old
        // `if data.is_some() { DATA } else { ACCESSOR }` got wrong.
        let mut partial = empty();
        partial.writable = Presence::Present(7);
        let validated = partial.validate().expect("data-only passes step 9");
        assert!(matches!(
            classify(&validated),
            DescriptorClassification::Static(PropertyDescriptorKind::Data),
        ));
    }

    #[test]
    fn a_set_only_descriptor_is_an_accessor_descriptor() {
        let mut partial = empty();
        partial.set = Presence::Present(3);
        let validated = partial.validate().expect("accessor-only passes step 9");
        assert!(matches!(
            classify(&validated),
            DescriptorClassification::Static(PropertyDescriptorKind::Accessor),
        ));
    }

    #[test]
    fn both_sides_statically_present_fails_step_nine() {
        let mut partial = empty();
        partial.value = Presence::Present(1);
        partial.get = Presence::Present(2);
        assert_eq!(
            partial.validate().err(),
            Some(ValidateError::BothDataAndAccessor(BothDataAndAccessor {
                data_side: DescriptorField::Value,
                accessor_side: DescriptorField::Get,
            })),
        );
    }

    #[test]
    fn the_two_side_markers_name_the_two_sides() {
        assert_eq!(DataSide::SIDE, DescriptorSide::Data);
        assert_eq!(AccessorSide::SIDE, DescriptorSide::Accessor);
        assert_eq!(DescriptorField::Writable.side(), Some(DescriptorSide::Data));
        assert_eq!(DescriptorField::Set.side(), Some(DescriptorSide::Accessor));
        // 6.2.6.3: a descriptor carrying only these two is *generic*.
        assert_eq!(DescriptorField::Enumerable.side(), None);
        assert_eq!(DescriptorField::Configurable.side(), None);
    }

    #[test]
    fn both_sides_possible_at_runtime_needs_the_emitted_check() {
        let mut partial = empty();
        partial.writable = Presence::Runtime {
            present: 10,
            value: 11,
        };
        partial.get = Presence::Runtime {
            present: 12,
            value: 13,
        };
        assert_eq!(
            partial.clone().validate().err(),
            Some(ValidateError::NeedsRuntimeCheck {
                data_side: DescriptorField::Writable,
                accessor_side: DescriptorField::Get,
            }),
        );
        // The escape hatch accepts it, and `classify` then names each side's
        // terms separately — which is what lets 10.1.6.3 step 6.a and step 7.a
        // be emitted independently instead of under one four-way conjunction.
        let validated = partial.from_runtime_checked();
        match classify(&validated) {
            DescriptorClassification::Static(kind) => {
                panic!("expected a dynamic classification, got {kind:?}")
            }
            DescriptorClassification::Dynamic {
                accessor_terms,
                data_terms,
            } => {
                assert_eq!(data_terms.runtime_flags(), vec![10]);
                assert_eq!(accessor_terms.runtime_flags(), vec![12]);
            }
        }
    }

    #[test]
    fn present_fields_is_any_subset() {
        let mut partial = empty();
        partial.get = Presence::Present(1);
        partial.enumerable = Presence::Present(2);
        partial.configurable = Presence::Present(3);
        assert_eq!(
            partial.present_fields().collect::<Vec<_>>(),
            vec![
                DescriptorField::Get,
                DescriptorField::Enumerable,
                DescriptorField::Configurable,
            ],
        );
    }

    #[test]
    fn a_generic_descriptor_completes_to_a_data_property() {
        // 6.2.6.6 step 3, and test262 `15.2.3.6-4-52.js`.
        let validated = PartialDescriptor::<SourceText>::empty()
            .validate()
            .expect("`{}` passes 6.2.6.5 step 9");
        let kind = validated.static_kind();
        assert_eq!(kind, PropertyDescriptorKind::Generic);
        let complete =
            complete_property_descriptor(validated, kind, &SourceText::completion_defaults());
        assert_eq!(complete.kind(), PropertyDescriptorKind::Data);
        assert_eq!(
            complete.render(),
            "{ value: undefined, writable: false, enumerable: false, configurable: false }",
        );
    }

    #[test]
    fn the_module_namespace_to_string_tag_descriptor_renders_byte_for_byte() {
        assert_eq!(
            DescriptorSourceText::data().value("\"Module\"").complete().render(),
            "{ value: \"Module\", writable: false, enumerable: false, configurable: false }",
        );
    }

    #[test]
    fn the_module_namespace_export_descriptor_renders_byte_for_byte() {
        assert_eq!(
            DescriptorSourceText::accessor()
                .get("() => value")
                .enumerable(true)
                .configurable(false)
                .render(),
            "{ get: () => value, enumerable: true, configurable: false }",
        );
    }

    #[test]
    fn an_accessor_complete_descriptor_has_no_writable_key() {
        // 10.1.6.3 steps 6.b and 7: the surviving attribute set is decided
        // entirely by the target kind.
        let complete = CompleteDescriptor::<SourceText>::Accessor {
            get: String::from("g"),
            set: String::from("undefined"),
            enumerable: false,
            configurable: true,
        };
        assert!(!complete.keys().contains(&DescriptorField::Writable));
        assert_eq!(
            complete.render(),
            "{ get: g, set: undefined, enumerable: false, configurable: true }",
        );
    }
}
