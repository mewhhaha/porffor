//! The iterator-protocol obligations of ECMA-262 7.4, and the witness a for-of
//! specialization carries to say how it discharged each of them.
//!
//! ## Why this exists
//!
//! `for (const x of arr)` is lowered to an index walk. That is not "an
//! optimization": `%Array.prototype%[@@iterator]` is a writable, configurable,
//! shadowable data property (23.1.3.x) whose `CreateArrayIterator` step closure
//! re-reads `LengthOfArrayLike` and performs a real `[[Get]]` per step
//! (23.1.5.1). Walking indices instead is sound only relative to a conjunction
//! of premises about the realm and the value.
//!
//! This module does **not** discharge those premises. It makes them named
//! values in the IR, so that
//!
//! - relying on one is a construction that a reviewer sees, not a silence;
//! - adding a fourth specialization cannot compile until its author says, for
//!   all four obligations, whether the specialization *emits* the operation or
//!   *assumes it away*, and on which premise;
//! - transposing two obligations is `E0308` rather than a plausible-looking
//!   wrong witness — including *inside this module*, because the four discharge
//!   newtypes are what [`IteratorProtocolWitness::new`] takes.
//!
//! ## What a premise is, and what it is not
//!
//! [`IntactnessPremise`] used to mix three different kinds of claim under one
//! name, which made `ByAssumption(ArrayLengthReadOnce)` read as *discharged*
//! while assuming nothing: "length is read once" is a description of what our
//! emitter does, not a condition under which doing that is correct. Every
//! variant now carries an [`IntactnessPremise::kind`]:
//!
//! - [`PremiseKind::ProgramProperty`] — a proposition about the *user's
//!   program* that only a lowering-time guard can establish. Ledger **L3**.
//! - [`PremiseKind::ImplementationFact`] — a proposition about *our emitter*,
//!   established by reading it. Recorded so the reader knows no guard is owed.
//! - [`PremiseKind::Vacuous`] — there is nothing to discharge.
//!
//! Ledger **L3** is the honest bound, restated: a *partial* intactness guard
//! already exists — `ScriptLowerer::array_prototype_mutated` — and four sibling
//! fast paths already consult it. The for-of specialization decision is the
//! outlier that does not. See ledger L3 in the contract for the exact
//! conjunct that is missing and why closing it is a separate lane (it moves
//! emitted bytes).
//!
//! ## The emitter must not read this
//!
//! No `porffor-aot-wasm` arm consults the witness, and now it cannot: every
//! reader of a witness's contents is `pub(crate)`, so a backend arm that binds
//! `protocol` and branches on `is_fully_emitted()` is `E0624` at `cargo check`
//! rather than a ten-minute rung-G diff. The type stays `pub` only because it
//! is the type of a `pub` enum variant's field.
//!
//! See `docs/rust-rewrite/contracts/Spec-operation catalog evidence and the
//! iterator-protocol obligation witness.md`.

/// The four 7.4 obligations a for-of head incurs. Closed by the spec.
///
/// ES2025 fuses `IteratorStep` and `IteratorValue` into `IteratorStepValue`;
/// the four-way decomposition is kept because it is what the catalog names,
/// what the emitted code separates, and what the corpus tests key on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IteratorObligation {
    /// 7.4.2. `GetMethod(obj, @@iterator)`, the `Call`, the object check, and
    /// the **once-only** `Get(iterator, "next")`.
    GetIterator,
    /// 7.4.8. `Call(next, iterator)`, the object check, then
    /// `ToBoolean(Get(result, "done"))`.
    IteratorStep,
    /// 7.4.9. `Get(result, "value")` — read after `"done"`, and not read at all
    /// on the exhausting step.
    IteratorValue,
    /// 7.4.11. `GetMethod(iterator, "return")` and the call, with step 4's
    /// asymmetry: an original `throw` completion wins and swallows an error
    /// raised by the close, while a `break`/`return`/`continue` completion does
    /// not.
    IteratorClose,
}

impl IteratorObligation {
    pub const fn name(self) -> &'static str {
        match self {
            Self::GetIterator => "GetIterator",
            Self::IteratorStep => "IteratorStep",
            Self::IteratorValue => "IteratorValue",
            Self::IteratorClose => "IteratorClose",
        }
    }
}

/// Declares [`EmissionSite`], its `ALL` enumeration and its `name` renderer
/// from **one** row list.
///
/// This is `spec_operations!`'s shape (`operations.rs`), applied to the site
/// domain for the same reason. The site↔witness and site↔catalog ties below
/// quantify over [`EmissionSite::ALL`]; a hand-written `ALL` would reintroduce
/// ledger **L1** exactly — a variant absent from the list is invisible to every
/// `const` expression that is supposed to constrain it, and the omission is
/// precisely what a length assertion preserves. With the enum, `ALL` and
/// `name()` being three expansions of the same `$(...)+` sequence, the omission
/// is not expressible.
///
/// A row is `[docs] Variant => "emitter function name"`. The name is the one
/// `porffor-aot-wasm/src/emission_sites.rs::emission_sites_are_backed` must
/// resolve.
macro_rules! emission_sites {
    ($(
        $( #[$meta:meta] )*
        $variant:ident => $name:literal
    ),+ $(,)?) => {
        /// Which emitter arm performs the operation.
        ///
        /// Closed, and each variant is joined to a real function by
        /// `porffor-aot-wasm/src/emission_sites.rs::emission_sites_are_backed`,
        /// an uncalled function whose exhaustive `match` names each arm's path.
        /// Renaming or deleting an emitter arm that a variant claims is
        /// therefore `E0599`, and adding a variant is a compile error until it
        /// names something real. The guarantee is name resolution, not
        /// signature — stated so nobody over-reads it.
        ///
        /// Three further ties close the triangle, so a variant cannot exist
        /// without an owner on either side: **K1** (below) rejects a site no
        /// witness constant discharges by emission, and **J10**/**J11** in
        /// `operations.rs` reject a catalog row naming an unwitnessed site and a
        /// site named by no catalog row.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub enum EmissionSite {
            $( $( #[$meta] )* $variant , )+
        }

        impl EmissionSite {
            /// Every site, in declaration order. Generated from the same rows as
            /// the enum, so it cannot be partial.
            pub const ALL: &'static [EmissionSite] = &[ $( EmissionSite::$variant , )+ ];

            pub const fn name(self) -> &'static str {
                match self { $( EmissionSite::$variant => $name , )+ }
            }
        }
    };
}

emission_sites! {
    /// `FunctionBuilder::compile_for_of_iterator` (`control_flow.rs:6874`).
    SyncForOfIterator => "compile_for_of_iterator",
    /// `FunctionBuilder::compile_async_for_of_iterator` (`control_flow.rs:5577`).
    AsyncForOfIterator => "compile_async_for_of_iterator",
    /// `FunctionBuilder::compile_array_destructure_from_value_locals`
    /// (`control_flow.rs:7656`). Array destructuring runs the same protocol:
    /// the `@@iterator` read, `finish_get_iterator_from_method`'s callability
    /// and object checks and the once-only `next` cache (`:7891`),
    /// `emit_destructuring_iterator_step` per element (`:8099`), and **both**
    /// halves of 7.4.11 step 4 under the `[[Done]]` guard — `emit_iterator_close`
    /// on the normal path (`:7710`) and
    /// `emit_iterator_close_preserving_current_throw` on the abrupt path
    /// (`:7729`). There is no array fast path here, so every array destructuring
    /// pays the real protocol.
    ArrayDestructuring => "compile_array_destructure_from_value_locals",
}

/// What kind of claim an [`IntactnessPremise`] is.
///
/// The distinction is load-bearing: only [`Self::ProgramProperty`] premises owe
/// a lowering-time guard, so the lane that closes ledger **L3** needs to know
/// which variants it must actually establish and which are already true.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PremiseKind {
    /// A proposition about the *user's program* or the realm. No type can prove
    /// it; a lowering-time guard must. Ledger **L3**.
    ProgramProperty,
    /// A proposition about *our emitter*, established by reading it. No guard
    /// is owed; the citation in the variant's doc comment is the evidence.
    ImplementationFact,
    /// There is nothing to discharge — the obligation does not arise.
    Vacuous,
}

/// The premises a specialization may rely on when it declines to emit an
/// obligation.
///
/// Closed: a lowering that needs a premise not in this list must add a variant,
/// which is a diff a reviewer sees. No [`PremiseKind::ProgramProperty`] variant
/// is checked anywhere — see ledger L3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntactnessPremise {
    /// `%Array.prototype%[@@iterator]` is still `%Array.prototype.values%`, no
    /// own `@@iterator` shadows it on the value or on an intermediate
    /// prototype, and `%ArrayIteratorPrototype%.next` is unpatched.
    /// (23.1.3.x, 23.1.5.1.)
    ///
    /// Stated for a value whose inferred kind set is exactly `{Array}`. Whether
    /// the lowering guard establishes that is ledger **L5**:
    /// `KindSet::EMPTY.is_subset_of({Array})` is `true`, so an *empty*
    /// `possible_kinds` also selects the array walk.
    ArrayIteratorIntact,
    /// The loop body does not change the array's `length`.
    ///
    /// The index walk hoists `emit_array_length` above the loop
    /// (`control_flow.rs:5285`, read above the loop header at `:5295`), whereas
    /// `CreateArrayIterator` re-reads `LengthOfArrayLike` on every step
    /// (23.1.5.1). The hoist is a fact about our emitter; *this* premise is the
    /// condition on the program under which the hoist is unobservable.
    ArrayLengthStableDuringBody,
    /// The array has no holes, and neither it nor its prototype chain carries
    /// an accessor or a proxy trap on an index key.
    ///
    /// `emit_array_read` (`control_flow.rs:5300`) reads backing storage rather
    /// than performing `[[Get]]`; again, that is a fact about our emitter, and
    /// *this* premise is the condition on the program under which skipping
    /// `[[Get]]` is unobservable (23.1.5.1).
    ArrayHasNoHolesOrIndexAccessors,
    /// `%String.prototype%[@@iterator]` is still the initial value and
    /// `%StringIteratorPrototype%.next` is unpatched. (22.1.3.x.)
    StringIteratorIntact,
    /// The walk steps by code point over the internal encoding; `CodePointAt`
    /// (11.1.5) yields an unpaired surrogate as a one-unit code point rather
    /// than skipping or replacing it.
    ///
    /// [`PremiseKind::ImplementationFact`], and a discharged one: dry run
    /// verified `utf16_units_to_runtime_string` (`lowering.rs:3162`) pairs
    /// surrogates and escapes unpaired ones, `Data::runtime_bytes_for_string`
    /// (`data.rs:3721`) re-encodes the escape through `push_wtf8_code_unit`,
    /// and `emit_decode_utf8_scalar_at_index` (`builtins/string.rs:19762`)
    /// decodes 3- and 4-byte sequences without rejecting `D800`-`DFFF`. Astral
    /// pairs and lone surrogates each yield exactly one iteration.
    StringWalkIsCodePoint,
    /// There is no iterator object, so there is nothing `IteratorClose` could
    /// call: the close obligation is *vacuous* rather than skipped.
    NoIteratorObjectExists,
    /// The head did not lower to an iteration at all — an unsupported form was
    /// reported and `StatementIr::Empty` was emitted — so no 7.4 operation is
    /// owed by the statement that was produced.
    NoIterationLowered,
}

impl IntactnessPremise {
    pub const fn name(self) -> &'static str {
        match self {
            Self::ArrayIteratorIntact => "ArrayIteratorIntact",
            Self::ArrayLengthStableDuringBody => "ArrayLengthStableDuringBody",
            Self::ArrayHasNoHolesOrIndexAccessors => "ArrayHasNoHolesOrIndexAccessors",
            Self::StringIteratorIntact => "StringIteratorIntact",
            Self::StringWalkIsCodePoint => "StringWalkIsCodePoint",
            Self::NoIteratorObjectExists => "NoIteratorObjectExists",
            Self::NoIterationLowered => "NoIterationLowered",
        }
    }

    /// Which kind of claim this premise is. Exhaustive and without a catch-all:
    /// a new premise must state whether it owes a guard.
    pub const fn kind(self) -> PremiseKind {
        match self {
            Self::ArrayIteratorIntact
            | Self::ArrayLengthStableDuringBody
            | Self::ArrayHasNoHolesOrIndexAccessors
            | Self::StringIteratorIntact => PremiseKind::ProgramProperty,
            Self::StringWalkIsCodePoint => PremiseKind::ImplementationFact,
            Self::NoIteratorObjectExists | Self::NoIterationLowered => PremiseKind::Vacuous,
        }
    }
}

/// How one obligation was accounted for.
///
/// Two cases, both carrying payload: there is no "unknown", no `Default`, and
/// no unit variant, so "I did not think about this one" has no spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObligationDischarge {
    ByEmission(EmissionSite),
    ByAssumption(IntactnessPremise),
}

impl ObligationDischarge {
    pub(crate) const fn is_emitted(self) -> bool {
        match self {
            Self::ByEmission(_) => true,
            Self::ByAssumption(_) => false,
        }
    }
}

// Four distinct newtypes, deliberately not one generic wrapper: transposing
// "what I assumed about `next`" with "what I assumed about `return`" must be
// `E0308`, not a witness that type-checks and lies. That only holds if the
// *constructors* are per-obligation too — `assumed(a, b, c, d)` over four
// same-typed `IntactnessPremise` arguments would have moved the transposition
// inside this module rather than removing it — so each newtype carries its own
// `assumed`/`emitted` pair and `IteratorProtocolWitness::new` takes the four
// newtypes. Every constructor is private to this module, so outside
// `iterator_obligations` the only inhabitants of the witness are the five named
// constants.

/// How a specialization discharged `GetIterator` (7.4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetIteratorDischarge(ObligationDischarge);

/// How a specialization discharged `IteratorStep` (7.4.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IteratorStepDischarge(ObligationDischarge);

/// How a specialization discharged `IteratorValue` (7.4.9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IteratorValueDischarge(ObligationDischarge);

/// How a specialization discharged `IteratorClose` (7.4.11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IteratorCloseDischarge(ObligationDischarge);

impl GetIteratorDischarge {
    const fn assumed(premise: IntactnessPremise) -> Self {
        Self(ObligationDischarge::ByAssumption(premise))
    }

    const fn emitted(site: EmissionSite) -> Self {
        Self(ObligationDischarge::ByEmission(site))
    }

    pub(crate) const fn get(self) -> ObligationDischarge {
        self.0
    }
}

impl IteratorStepDischarge {
    const fn assumed(premise: IntactnessPremise) -> Self {
        Self(ObligationDischarge::ByAssumption(premise))
    }

    const fn emitted(site: EmissionSite) -> Self {
        Self(ObligationDischarge::ByEmission(site))
    }

    pub(crate) const fn get(self) -> ObligationDischarge {
        self.0
    }
}

impl IteratorValueDischarge {
    const fn assumed(premise: IntactnessPremise) -> Self {
        Self(ObligationDischarge::ByAssumption(premise))
    }

    const fn emitted(site: EmissionSite) -> Self {
        Self(ObligationDischarge::ByEmission(site))
    }

    pub(crate) const fn get(self) -> ObligationDischarge {
        self.0
    }
}

impl IteratorCloseDischarge {
    const fn assumed(premise: IntactnessPremise) -> Self {
        Self(ObligationDischarge::ByAssumption(premise))
    }

    const fn emitted(site: EmissionSite) -> Self {
        Self(ObligationDischarge::ByEmission(site))
    }

    pub(crate) const fn get(self) -> ObligationDischarge {
        self.0
    }
}

/// The obligation witness a for-of specialization carries.
///
/// Non-defaultable, non-optional, all fields private, and **`new` is private to
/// this module**. Outside `iterator_obligations`, the only values of this type
/// are the five named constants below. That is what turns "these constants are
/// the only witnesses `lowering.rs` may use" from a rule in a document into a
/// property of the type: a new specialization cannot invent a witness at a
/// construction site 13,000 lines into `lowering.rs`; it must add a constant
/// here, next to the premises it is claiming.
///
/// `Copy`, and free of `String`, so putting it on a `StatementIr` variant costs
/// no allocation and preserves `PartialEq`/`Clone`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IteratorProtocolWitness {
    get_iterator: GetIteratorDischarge,
    iterator_step: IteratorStepDischarge,
    iterator_value: IteratorValueDischarge,
    iterator_close: IteratorCloseDischarge,
}

impl IteratorProtocolWitness {
    const fn new(
        get_iterator: GetIteratorDischarge,
        iterator_step: IteratorStepDischarge,
        iterator_value: IteratorValueDischarge,
        iterator_close: IteratorCloseDischarge,
    ) -> Self {
        Self {
            get_iterator,
            iterator_step,
            iterator_value,
            iterator_close,
        }
    }

    const fn emitted_by(site: EmissionSite) -> Self {
        Self::new(
            GetIteratorDischarge::emitted(site),
            IteratorStepDischarge::emitted(site),
            IteratorValueDischarge::emitted(site),
            IteratorCloseDischarge::emitted(site),
        )
    }

    /// `StatementIr::ForOfArray` — the index walk.
    ///
    /// All four obligations are discharged by assumption. `compile_for_of_array`
    /// (`control_flow.rs:5238`) is a bare `emit_array_length` +
    /// `emit_array_read` index walk with no `@@iterator` `Get` anywhere in it,
    /// which is exactly what these premises assert is safe — and what trace A1
    /// shows is observably wrong when `Array.prototype[@@iterator]` is patched.
    pub const ARRAY_INDEX_WALK: Self = Self::new(
        GetIteratorDischarge::assumed(IntactnessPremise::ArrayIteratorIntact),
        IteratorStepDischarge::assumed(IntactnessPremise::ArrayLengthStableDuringBody),
        IteratorValueDischarge::assumed(IntactnessPremise::ArrayHasNoHolesOrIndexAccessors),
        IteratorCloseDischarge::assumed(IntactnessPremise::NoIteratorObjectExists),
    );

    /// The same index walk, reached by a different desugaring: `for (x of arr)`
    /// whose body awaits, inside a plain async function, becomes a
    /// `StatementIr::GeneratorLoop` over `PropertyKeyIr::ArrayLength` and
    /// `PropertyKeyIr::ArrayIndex` (`lowering.rs`,
    /// `lower_async_for_of_array_with_body_await`).
    ///
    /// It is a *fourth* for-of specialization that is not spelled as a `ForOf*`
    /// variant, and it relies on exactly the premises of
    /// [`Self::ARRAY_INDEX_WALK`]. It is a separate constant so that the
    /// witness names the desugaring a reader has to go and check.
    pub const ARRAY_INDEX_WALK_RESUMABLE: Self = Self::ARRAY_INDEX_WALK;

    /// `StatementIr::ForOfString` — the code-point walk.
    ///
    /// `compile_for_of_string` (`control_flow.rs:5353`) steps via
    /// `emit_decode_utf8_scalar_at_index`.
    pub const STRING_CODE_POINT_WALK: Self = Self::new(
        GetIteratorDischarge::assumed(IntactnessPremise::StringIteratorIntact),
        IteratorStepDischarge::assumed(IntactnessPremise::StringWalkIsCodePoint),
        IteratorValueDischarge::assumed(IntactnessPremise::StringWalkIsCodePoint),
        IteratorCloseDischarge::assumed(IntactnessPremise::NoIteratorObjectExists),
    );

    /// `StatementIr::ForOfIterator`, sync. Every obligation is really emitted;
    /// the close predicate in `emit_iterator_close_condition_i32`
    /// (`control_flow.rs:8451`) is exactly `¬LoopContinues`.
    pub const SYNC_ITERATOR_PROTOCOL: Self = Self::emitted_by(EmissionSite::SyncForOfIterator);

    /// `StatementIr::ForOfIterator` with an async plan (`for await`).
    pub const ASYNC_ITERATOR_PROTOCOL: Self = Self::emitted_by(EmissionSite::AsyncForOfIterator);

    /// `ArrayDestructuringPatternIr` — the iterator one ArrayBindingPattern
    /// (8.6.3 IteratorBindingInitialization) or ArrayAssignmentPattern
    /// (13.15.5.5 IteratorDestructuringAssignmentEvaluation) acquires.
    ///
    /// One witness per *pattern*, not per statement: both operations perform a
    /// fresh `GetIterator` for every array pattern, nested ones included, which
    /// is why the field lives on [`crate::ArrayDestructuringPatternIr`] rather
    /// than on `ExprIr::ArrayDestructure`.
    ///
    /// Every obligation is really emitted, verified step by step at
    /// `control_flow.rs:7656-7770`: `emit_get_iterator_from_value_locals`
    /// (`:7687`), `emit_destructuring_iterator_step` (`:8099`) per element, and
    /// **both** halves of 7.4.11 step 4 — `emit_iterator_close` under the
    /// `[[Done]]` guard on the normal path (`:7710`) and
    /// `emit_iterator_close_preserving_current_throw` under the same guard on
    /// the abrupt path (`:7729`). That guard is 8.6.3 step 5's
    /// "if `iteratorRecord.[[Done]]` is false", which is what
    /// `array-elem-iter-nrml-close-skip.js` pins. There is no array fast path
    /// here, so every array destructuring pays the real protocol and the
    /// discharge is honestly `ByEmission` in all four slots.
    pub const ARRAY_DESTRUCTURING_PROTOCOL: Self =
        Self::emitted_by(EmissionSite::ArrayDestructuring);

    /// The for-of head produced no iteration: an unsupported form was reported
    /// and the statement is `StatementIr::Empty`. Every obligation is vacuous
    /// because nothing runs.
    ///
    /// This exists so that a bail-out path is still a construction a reviewer
    /// sees, rather than the one shape in which a for-of head can escape
    /// without saying anything.
    pub const NO_ITERATION: Self = Self::new(
        GetIteratorDischarge::assumed(IntactnessPremise::NoIterationLowered),
        IteratorStepDischarge::assumed(IntactnessPremise::NoIterationLowered),
        IteratorValueDischarge::assumed(IntactnessPremise::NoIterationLowered),
        IteratorCloseDischarge::assumed(IntactnessPremise::NoIterationLowered),
    );

    pub(crate) const fn get_iterator(self) -> GetIteratorDischarge {
        self.get_iterator
    }

    pub(crate) const fn iterator_step(self) -> IteratorStepDischarge {
        self.iterator_step
    }

    pub(crate) const fn iterator_value(self) -> IteratorValueDischarge {
        self.iterator_value
    }

    pub(crate) const fn iterator_close(self) -> IteratorCloseDischarge {
        self.iterator_close
    }

    /// Read one obligation's discharge, going through the typed accessor for
    /// that slot rather than the field. The four accessors return four
    /// different newtypes, so an arm that reached into the wrong slot would be
    /// `E0308` here rather than a witness that type-checks and lies.
    pub(crate) const fn discharge(self, obligation: IteratorObligation) -> ObligationDischarge {
        match obligation {
            IteratorObligation::GetIterator => self.get_iterator().get(),
            IteratorObligation::IteratorStep => self.iterator_step().get(),
            IteratorObligation::IteratorValue => self.iterator_value().get(),
            IteratorObligation::IteratorClose => self.iterator_close().get(),
        }
    }

    /// True when every obligation is discharged by real emitted code, i.e. the
    /// specialization relies on no premise about the program.
    pub(crate) const fn is_fully_emitted(self) -> bool {
        self.get_iterator.get().is_emitted()
            && self.iterator_step.get().is_emitted()
            && self.iterator_value.get().is_emitted()
            && self.iterator_close.get().is_emitted()
    }
}

// ---------------------------------------------------------------------------
// What each witness claims, checked by `cargo check`
// ---------------------------------------------------------------------------
//
// Every reader above is a `const fn`, so "this constant says what its doc
// comment says" is checkable at compile time and does not need a test run to
// discover. That matters twice here:
//
//  1. It is the campaign's standard — an invariant the compiler enforces beats
//     one a test discovers.
//  2. §13.12 narrowed every reader to `pub(crate)` so that a `porffor-aot-wasm`
//     arm binding `protocol` and branching on it is `E0624`. That narrowing left
//     the readers with no caller outside `#[cfg(test)]`, i.e. `dead_code` in the
//     product build — the "survival by `pub`" defect this area exists to delete,
//     reappearing one visibility level down. These assertions are their callers,
//     so the emitter still cannot read a witness and the readers still earn
//     their place.
//
// The mistake actually being caught: editing a witness so it *claims* emission
// it does not perform (or renaming a premise into a weaker one) is the exact lie
// that makes the index walk look discharged. That is now `E0080` at build time.

const fn assumes(
    witness: IteratorProtocolWitness,
    obligation: IteratorObligation,
    premise: IntactnessPremise,
) -> bool {
    match witness.discharge(obligation) {
        ObligationDischarge::ByAssumption(actual) => actual as u8 == premise as u8,
        ObligationDischarge::ByEmission(_) => false,
    }
}

const fn emits(
    witness: IteratorProtocolWitness,
    obligation: IteratorObligation,
    site: EmissionSite,
) -> bool {
    match witness.discharge(obligation) {
        ObligationDischarge::ByEmission(actual) => actual as u8 == site as u8,
        ObligationDischarge::ByAssumption(_) => false,
    }
}

const fn assumes_kind(
    witness: IteratorProtocolWitness,
    obligation: IteratorObligation,
    kind: PremiseKind,
) -> bool {
    match witness.discharge(obligation) {
        ObligationDischarge::ByAssumption(premise) => premise.kind() as u8 == kind as u8,
        ObligationDischarge::ByEmission(_) => false,
    }
}

const fn emits_every_obligation(witness: IteratorProtocolWitness, site: EmissionSite) -> bool {
    witness.is_fully_emitted()
        && emits(witness, IteratorObligation::GetIterator, site)
        && emits(witness, IteratorObligation::IteratorStep, site)
        && emits(witness, IteratorObligation::IteratorValue, site)
        && emits(witness, IteratorObligation::IteratorClose, site)
}

/// The array index walk emits nothing and rests on four named premises.
/// `compile_for_of_array` contains no `@@iterator` read at all, so flipping any
/// of these to `ByEmission` would be a lie about the emitter.
const _: () = assert!(
    !IteratorProtocolWitness::ARRAY_INDEX_WALK.is_fully_emitted()
        && assumes(
            IteratorProtocolWitness::ARRAY_INDEX_WALK,
            IteratorObligation::GetIterator,
            IntactnessPremise::ArrayIteratorIntact,
        )
        && assumes(
            IteratorProtocolWitness::ARRAY_INDEX_WALK,
            IteratorObligation::IteratorStep,
            IntactnessPremise::ArrayLengthStableDuringBody,
        )
        && assumes(
            IteratorProtocolWitness::ARRAY_INDEX_WALK,
            IteratorObligation::IteratorValue,
            IntactnessPremise::ArrayHasNoHolesOrIndexAccessors,
        )
        && assumes(
            IteratorProtocolWitness::ARRAY_INDEX_WALK,
            IteratorObligation::IteratorClose,
            IntactnessPremise::NoIteratorObjectExists,
        ),
    "ARRAY_INDEX_WALK must discharge all four 7.4 obligations by its named premises"
);

/// The string walk, likewise: `compile_for_of_string` steps by code point and
/// never builds a String Iterator.
const _: () = assert!(
    !IteratorProtocolWitness::STRING_CODE_POINT_WALK.is_fully_emitted()
        && assumes(
            IteratorProtocolWitness::STRING_CODE_POINT_WALK,
            IteratorObligation::GetIterator,
            IntactnessPremise::StringIteratorIntact,
        )
        && assumes(
            IteratorProtocolWitness::STRING_CODE_POINT_WALK,
            IteratorObligation::IteratorStep,
            IntactnessPremise::StringWalkIsCodePoint,
        )
        && assumes(
            IteratorProtocolWitness::STRING_CODE_POINT_WALK,
            IteratorObligation::IteratorValue,
            IntactnessPremise::StringWalkIsCodePoint,
        )
        && assumes(
            IteratorProtocolWitness::STRING_CODE_POINT_WALK,
            IteratorObligation::IteratorClose,
            IntactnessPremise::NoIteratorObjectExists,
        ),
    "STRING_CODE_POINT_WALK must discharge all four 7.4 obligations by its named premises"
);

/// The two real-protocol witnesses emit every obligation, each at its own site.
const _: () = assert!(
    emits_every_obligation(
        IteratorProtocolWitness::SYNC_ITERATOR_PROTOCOL,
        EmissionSite::SyncForOfIterator,
    ),
    "SYNC_ITERATOR_PROTOCOL must emit all four 7.4 obligations at compile_for_of_iterator"
);

const _: () = assert!(
    emits_every_obligation(
        IteratorProtocolWitness::ASYNC_ITERATOR_PROTOCOL,
        EmissionSite::AsyncForOfIterator,
    ),
    "ASYNC_ITERATOR_PROTOCOL must emit all four 7.4 obligations at compile_async_for_of_iterator"
);

/// The bail-out witness assumes nothing about the program: every obligation is
/// vacuous because no iteration was lowered.
const _: () = assert!(
    assumes_kind(
        IteratorProtocolWitness::NO_ITERATION,
        IteratorObligation::GetIterator,
        PremiseKind::Vacuous,
    ) && assumes_kind(
        IteratorProtocolWitness::NO_ITERATION,
        IteratorObligation::IteratorStep,
        PremiseKind::Vacuous,
    ) && assumes_kind(
        IteratorProtocolWitness::NO_ITERATION,
        IteratorObligation::IteratorValue,
        PremiseKind::Vacuous,
    ) && assumes_kind(
        IteratorProtocolWitness::NO_ITERATION,
        IteratorObligation::IteratorClose,
        PremiseKind::Vacuous,
    ),
    "NO_ITERATION must be vacuous in all four obligations"
);

/// Ledger **L3** keys off this split: only a [`PremiseKind::ProgramProperty`]
/// premise owes a lowering-time guard. Reclassifying `ArrayIteratorIntact` as an
/// `ImplementationFact` would make the index walk *read* as discharged while
/// still assuming the whole of 23.1.3.x — the 13.6 defect. It is now `E0080`.
const _: () = assert!(
    assumes_kind(
        IteratorProtocolWitness::ARRAY_INDEX_WALK,
        IteratorObligation::GetIterator,
        PremiseKind::ProgramProperty,
    ) && assumes_kind(
        IteratorProtocolWitness::ARRAY_INDEX_WALK,
        IteratorObligation::IteratorStep,
        PremiseKind::ProgramProperty,
    ) && assumes_kind(
        IteratorProtocolWitness::ARRAY_INDEX_WALK,
        IteratorObligation::IteratorValue,
        PremiseKind::ProgramProperty,
    ) && assumes_kind(
        IteratorProtocolWitness::STRING_CODE_POINT_WALK,
        IteratorObligation::GetIterator,
        PremiseKind::ProgramProperty,
    ),
    "the array and string walks rest on unguarded program properties: ledger L3"
);

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_OBLIGATIONS: [IteratorObligation; 4] = [
        IteratorObligation::GetIterator,
        IteratorObligation::IteratorStep,
        IteratorObligation::IteratorValue,
        IteratorObligation::IteratorClose,
    ];

    /// The array walk claims to emit nothing and to rely on four named
    /// premises. Flipping one obligation to `ByEmission` would be a lie about
    /// `compile_for_of_array`, which contains no `@@iterator` read at all.
    #[test]
    fn array_index_walk_discharges_every_obligation_by_assumption() {
        let witness = IteratorProtocolWitness::ARRAY_INDEX_WALK;
        assert!(!witness.is_fully_emitted());
        for obligation in ALL_OBLIGATIONS {
            assert!(
                matches!(
                    witness.discharge(obligation),
                    ObligationDischarge::ByAssumption(_)
                ),
                "{} must be discharged by a named premise, not by emission",
                obligation.name()
            );
        }
        assert_eq!(
            witness.discharge(IteratorObligation::GetIterator),
            ObligationDischarge::ByAssumption(IntactnessPremise::ArrayIteratorIntact)
        );
        assert_eq!(
            witness.discharge(IteratorObligation::IteratorClose),
            ObligationDischarge::ByAssumption(IntactnessPremise::NoIteratorObjectExists)
        );
    }

    #[test]
    fn string_code_point_walk_discharges_every_obligation_by_assumption() {
        let witness = IteratorProtocolWitness::STRING_CODE_POINT_WALK;
        assert!(!witness.is_fully_emitted());
        for obligation in ALL_OBLIGATIONS {
            assert!(matches!(
                witness.discharge(obligation),
                ObligationDischarge::ByAssumption(_)
            ));
        }
        assert_eq!(
            witness.discharge(IteratorObligation::IteratorValue),
            ObligationDischarge::ByAssumption(IntactnessPremise::StringWalkIsCodePoint)
        );
    }

    #[test]
    fn iterator_protocol_witnesses_emit_every_obligation() {
        for (witness, site) in [
            (
                IteratorProtocolWitness::SYNC_ITERATOR_PROTOCOL,
                EmissionSite::SyncForOfIterator,
            ),
            (
                IteratorProtocolWitness::ASYNC_ITERATOR_PROTOCOL,
                EmissionSite::AsyncForOfIterator,
            ),
        ] {
            assert!(witness.is_fully_emitted());
            for obligation in ALL_OBLIGATIONS {
                assert_eq!(
                    witness.discharge(obligation),
                    ObligationDischarge::ByEmission(site),
                    "{} must be emitted by {}",
                    obligation.name(),
                    site.name()
                );
            }
        }
    }

    /// Ledger **L3** keys off this split: only `ProgramProperty` premises owe a
    /// lowering-time guard. `ByAssumption(ArrayLengthReadOnce)` used to read as
    /// discharged while assuming nothing, because the old variant described our
    /// emitter rather than the program.
    #[test]
    fn every_assumed_premise_states_which_kind_of_claim_it_is() {
        for (witness, expected) in [
            (
                IteratorProtocolWitness::ARRAY_INDEX_WALK,
                [
                    PremiseKind::ProgramProperty,
                    PremiseKind::ProgramProperty,
                    PremiseKind::ProgramProperty,
                    PremiseKind::Vacuous,
                ],
            ),
            (
                IteratorProtocolWitness::STRING_CODE_POINT_WALK,
                [
                    PremiseKind::ProgramProperty,
                    PremiseKind::ImplementationFact,
                    PremiseKind::ImplementationFact,
                    PremiseKind::Vacuous,
                ],
            ),
            (
                IteratorProtocolWitness::NO_ITERATION,
                [
                    PremiseKind::Vacuous,
                    PremiseKind::Vacuous,
                    PremiseKind::Vacuous,
                    PremiseKind::Vacuous,
                ],
            ),
        ] {
            for (obligation, expected_kind) in ALL_OBLIGATIONS.into_iter().zip(expected) {
                let ObligationDischarge::ByAssumption(premise) = witness.discharge(obligation)
                else {
                    panic!("{} should be discharged by assumption", obligation.name());
                };
                assert_eq!(
                    premise.kind(),
                    expected_kind,
                    "{} rests on {}",
                    obligation.name(),
                    premise.name()
                );
            }
        }
    }
}
