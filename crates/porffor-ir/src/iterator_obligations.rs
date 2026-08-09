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
//!   wrong witness.
//!
//! Ledger **L3** is the honest bound: an [`IntactnessPremise`] is a statement
//! about the *user's program*, not about our code. No type can prove it; only a
//! lowering-time guard can, and building that guard is a separate lane.
//! Adversarial trace A1 in the contract records a program for which
//! [`IntactnessPremise::ArrayIteratorIntact`] is false today.
//!
//! ## The emitter must not read this
//!
//! No `porffor-aot-wasm` arm consults the witness. An IR field the emitter
//! reads changes emitted bytes; an IR field the emitter ignores changes nothing
//! but still forces the author to fill it in. Rung G must diff empty.
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

/// Which emitter arm performs the operation.
///
/// Closed, and each variant is joined to a real function by
/// `porffor-aot-wasm/src/emission_sites.rs::emission_sites_are_backed`, an
/// uncalled function whose exhaustive `match` names each arm's path. Renaming
/// or deleting an emitter arm that a variant claims is therefore `E0599`, and
/// adding a variant is a compile error until it names something real. The
/// guarantee is name resolution, not signature — stated so nobody over-reads
/// it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EmissionSite {
    /// `FunctionBuilder::compile_for_of_iterator` (`control_flow.rs:7422`).
    SyncForOfIterator,
    /// `FunctionBuilder::compile_async_for_of_iterator` (`control_flow.rs:6078`).
    AsyncForOfIterator,
    /// `FunctionBuilder::compile_array_destructure_from_value_locals`
    /// (`control_flow.rs:8220`). Array destructuring runs the same protocol:
    /// the `@@iterator` read, `finish_get_iterator_from_method`'s callability
    /// and object checks and the once-only `next` cache, and a guarded close.
    ArrayDestructuring,
}

impl EmissionSite {
    pub const fn name(self) -> &'static str {
        match self {
            Self::SyncForOfIterator => "compile_for_of_iterator",
            Self::AsyncForOfIterator => "compile_async_for_of_iterator",
            Self::ArrayDestructuring => "compile_array_destructure_from_value_locals",
        }
    }
}

/// The premises a specialization may rely on when it declines to emit an
/// obligation.
///
/// Closed: a lowering that needs a premise not in this list must add a variant,
/// which is a diff a reviewer sees. None of these is checked — see ledger L3.
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
    /// `length` is read once before the walk, whereas `CreateArrayIterator`
    /// re-reads `LengthOfArrayLike` on every step. (23.1.5.1.)
    ArrayLengthReadOnce,
    /// Elements are read from the backing storage rather than by `[[Get]]`, so
    /// a hole does not consult `Array.prototype` and an index accessor does not
    /// run. (23.1.5.1.)
    ArrayElementReadBypassesGet,
    /// `%String.prototype%[@@iterator]` is still the initial value and
    /// `%StringIteratorPrototype%.next` is unpatched. (22.1.3.x.)
    StringIteratorIntact,
    /// The walk steps by code point over the internal encoding; `CodePointAt`
    /// (11.1.5) yields an unpaired surrogate as a one-unit code point rather
    /// than skipping or replacing it.
    StringWalkIsCodePoint,
    /// There is no iterator object, so there is nothing `IteratorClose` could
    /// call: the close obligation is *vacuous* rather than skipped.
    NoIteratorObjectExists,
}

impl IntactnessPremise {
    pub const fn name(self) -> &'static str {
        match self {
            Self::ArrayIteratorIntact => "ArrayIteratorIntact",
            Self::ArrayLengthReadOnce => "ArrayLengthReadOnce",
            Self::ArrayElementReadBypassesGet => "ArrayElementReadBypassesGet",
            Self::StringIteratorIntact => "StringIteratorIntact",
            Self::StringWalkIsCodePoint => "StringWalkIsCodePoint",
            Self::NoIteratorObjectExists => "NoIteratorObjectExists",
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
    pub const fn is_emitted(self) -> bool {
        match self {
            Self::ByEmission(_) => true,
            Self::ByAssumption(_) => false,
        }
    }
}

// Four distinct newtypes, deliberately not one generic wrapper: transposing
// "what I assumed about `next`" with "what I assumed about `return`" at a
// construction site must be `E0308`, not a witness that type-checks and lies.
// Each constructor is private to this module — see `IteratorProtocolWitness`.

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
    const fn new(discharge: ObligationDischarge) -> Self {
        Self(discharge)
    }

    pub const fn obligation(self) -> IteratorObligation {
        IteratorObligation::GetIterator
    }

    pub const fn get(self) -> ObligationDischarge {
        self.0
    }
}

impl IteratorStepDischarge {
    const fn new(discharge: ObligationDischarge) -> Self {
        Self(discharge)
    }

    pub const fn obligation(self) -> IteratorObligation {
        IteratorObligation::IteratorStep
    }

    pub const fn get(self) -> ObligationDischarge {
        self.0
    }
}

impl IteratorValueDischarge {
    const fn new(discharge: ObligationDischarge) -> Self {
        Self(discharge)
    }

    pub const fn obligation(self) -> IteratorObligation {
        IteratorObligation::IteratorValue
    }

    pub const fn get(self) -> ObligationDischarge {
        self.0
    }
}

impl IteratorCloseDischarge {
    const fn new(discharge: ObligationDischarge) -> Self {
        Self(discharge)
    }

    pub const fn obligation(self) -> IteratorObligation {
        IteratorObligation::IteratorClose
    }

    pub const fn get(self) -> ObligationDischarge {
        self.0
    }
}

/// The obligation witness a for-of specialization carries.
///
/// Non-defaultable, non-optional, all fields private, and **`new` is private to
/// this module**. Outside `iterator_obligations`, the only values of this type
/// are the four named constants below. That is what turns "these four constants
/// are the only witnesses `lowering.rs` may use" from a rule in a document into
/// a property of the type: a fifth specialization cannot invent a witness at a
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

    const fn assumed(
        get_iterator: IntactnessPremise,
        iterator_step: IntactnessPremise,
        iterator_value: IntactnessPremise,
        iterator_close: IntactnessPremise,
    ) -> Self {
        Self::new(
            GetIteratorDischarge::new(ObligationDischarge::ByAssumption(get_iterator)),
            IteratorStepDischarge::new(ObligationDischarge::ByAssumption(iterator_step)),
            IteratorValueDischarge::new(ObligationDischarge::ByAssumption(iterator_value)),
            IteratorCloseDischarge::new(ObligationDischarge::ByAssumption(iterator_close)),
        )
    }

    const fn emitted_by(site: EmissionSite) -> Self {
        Self::new(
            GetIteratorDischarge::new(ObligationDischarge::ByEmission(site)),
            IteratorStepDischarge::new(ObligationDischarge::ByEmission(site)),
            IteratorValueDischarge::new(ObligationDischarge::ByEmission(site)),
            IteratorCloseDischarge::new(ObligationDischarge::ByEmission(site)),
        )
    }

    /// `StatementIr::ForOfArray` — the index walk.
    ///
    /// All four obligations are discharged by assumption. `compile_for_of_array`
    /// (`control_flow.rs:5732`) is a bare `emit_array_length` +
    /// `emit_array_read` index walk with no `@@iterator` `Get` anywhere in it,
    /// which is exactly what these premises assert is safe — and what trace A1
    /// shows is observably wrong when `Array.prototype[@@iterator]` is patched.
    pub const ARRAY_INDEX_WALK: Self = Self::assumed(
        IntactnessPremise::ArrayIteratorIntact,
        IntactnessPremise::ArrayLengthReadOnce,
        IntactnessPremise::ArrayElementReadBypassesGet,
        IntactnessPremise::NoIteratorObjectExists,
    );

    /// `StatementIr::ForOfString` — the code-point walk.
    ///
    /// `compile_for_of_string` (`control_flow.rs:5850`) steps via
    /// `emit_decode_utf8_scalar_at_index`.
    pub const STRING_CODE_POINT_WALK: Self = Self::assumed(
        IntactnessPremise::StringIteratorIntact,
        IntactnessPremise::StringWalkIsCodePoint,
        IntactnessPremise::StringWalkIsCodePoint,
        IntactnessPremise::NoIteratorObjectExists,
    );

    /// `StatementIr::ForOfIterator`, sync. Every obligation is really emitted;
    /// the close predicate in `emit_iterator_close_condition_i32`
    /// (`control_flow.rs:9018`) is exactly `¬LoopContinues`.
    pub const SYNC_ITERATOR_PROTOCOL: Self = Self::emitted_by(EmissionSite::SyncForOfIterator);

    /// `StatementIr::ForOfIterator` with an async plan (`for await`).
    pub const ASYNC_ITERATOR_PROTOCOL: Self = Self::emitted_by(EmissionSite::AsyncForOfIterator);

    pub const fn get_iterator(self) -> GetIteratorDischarge {
        self.get_iterator
    }

    pub const fn iterator_step(self) -> IteratorStepDischarge {
        self.iterator_step
    }

    pub const fn iterator_value(self) -> IteratorValueDischarge {
        self.iterator_value
    }

    pub const fn iterator_close(self) -> IteratorCloseDischarge {
        self.iterator_close
    }

    pub const fn discharge(self, obligation: IteratorObligation) -> ObligationDischarge {
        match obligation {
            IteratorObligation::GetIterator => self.get_iterator.get(),
            IteratorObligation::IteratorStep => self.iterator_step.get(),
            IteratorObligation::IteratorValue => self.iterator_value.get(),
            IteratorObligation::IteratorClose => self.iterator_close.get(),
        }
    }

    /// True when every obligation is discharged by real emitted code, i.e. the
    /// specialization relies on no premise about the program.
    pub const fn is_fully_emitted(self) -> bool {
        self.get_iterator.get().is_emitted()
            && self.iterator_step.get().is_emitted()
            && self.iterator_value.get().is_emitted()
            && self.iterator_close.get().is_emitted()
    }
}

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

    /// The per-obligation newtypes carry their own obligation, so a witness
    /// cannot be read back through the wrong accessor even in a test.
    #[test]
    fn discharge_newtypes_name_their_own_obligation() {
        let witness = IteratorProtocolWitness::SYNC_ITERATOR_PROTOCOL;
        assert_eq!(
            witness.get_iterator().obligation(),
            IteratorObligation::GetIterator
        );
        assert_eq!(
            witness.iterator_step().obligation(),
            IteratorObligation::IteratorStep
        );
        assert_eq!(
            witness.iterator_value().obligation(),
            IteratorObligation::IteratorValue
        );
        assert_eq!(
            witness.iterator_close().obligation(),
            IteratorObligation::IteratorClose
        );
    }
}
