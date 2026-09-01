use super::*;

/// What lowering a `for`-`of` head produced: the statement, the kind its body
/// evaluates to, and the witness saying how that statement discharged the four
/// 7.4 obligations.
///
/// The obligation is attached to the lowering of the head. Every path out of
/// `ScriptLowerer::lower_for_of_head` returns one of these, and there is no
/// `Default`. The dedicated resumable-sync constructor hardcodes its protocol
/// witness, so that statement cannot be paired with another emitter's credit.
///
/// Before the statement crosses back to dispatch,
/// [`ForOfLoweringIr::into_statement_and_kind`] consumes the carrier, reads the
/// witness and checks the two locally decidable conditions. Spread, `yield*`
/// and array destructuring reach the protocol by other routes and are named as
/// `EmissionSite`s instead — see ledger L6.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ForOfLoweringIr {
    statement: StatementIr,
    result_kind: ValueKind,
    protocol: IteratorProtocolWitness,
}

impl ForOfLoweringIr {
    pub(super) fn new(
        statement: StatementIr,
        result_kind: ValueKind,
        protocol: IteratorProtocolWitness,
    ) -> Self {
        Self {
            statement,
            result_kind,
            protocol,
        }
    }

    /// The head did not lower to an iteration: an unsupported form was reported
    /// and the statement is `StatementIr::Empty`.
    pub(super) fn no_iteration() -> Self {
        Self::new(
            StatementIr::Empty,
            ValueKind::Undefined,
            IteratorProtocolWitness::NO_ITERATION,
        )
    }

    pub(super) fn async_function_iterator(
        iterable: TypedExpr,
        plan: AsyncFunctionForOfIteratorPlanIr,
        result_kind: ValueKind,
    ) -> Self {
        Self::new(
            StatementIr::AsyncFunctionForOfIterator { iterable, plan },
            result_kind,
            IteratorProtocolWitness::RESUMABLE_SYNC_ITERATOR_PROTOCOL,
        )
    }

    /// The statement and the kind its body evaluates to. The witness is dropped
    /// here — its work is done by the time the head has lowered — but it is
    /// *read* on the way out rather than silently discarded.
    ///
    /// The `protocol()` accessor this replaces had **zero callers anywhere in
    /// the workspace** and was `pub`, so no `dead_code` warning fired: the
    /// "survival by `pub`" shape ledger row I7 exists to delete, as recorded in
    /// the iterator-protocol contract. The two conditions below are its
    /// replacement, and each names a real mistake:
    ///
    /// * A head that lowered to *nothing* must carry the bail-out witness.
    ///   Returning `StatementIr::Empty` with, say,
    ///   `SYNC_ITERATOR_PROTOCOL` would credit `compile_for_of_iterator` with
    ///   emitting four obligations for a statement that never runs, which is
    ///   exactly the attribution K1 and J10 exist to keep honest.
    /// * A head that lowered to a real for-of statement must carry that
    ///   statement's witness. The resumable-sync form has one constructor that
    ///   selects its one emitter site.
    pub(super) fn into_statement_and_kind(self) -> (StatementIr, ValueKind) {
        debug_assert!(
            !matches!(self.statement, StatementIr::Empty)
                || self.protocol == IteratorProtocolWitness::NO_ITERATION,
            "a for-of head that lowered to no statement must carry the NO_ITERATION witness",
        );
        debug_assert!(
            !matches!(self.statement, StatementIr::ForOfIterator { .. })
                || self.protocol != IteratorProtocolWitness::NO_ITERATION,
            "a for-of head that lowered to a real specialization must not claim that no \
             iteration was lowered",
        );
        debug_assert!(
            !matches!(
                self.statement,
                StatementIr::AsyncFunctionForOfIterator { .. }
            ) || self.protocol == IteratorProtocolWitness::RESUMABLE_SYNC_ITERATOR_PROTOCOL,
            "a resumable synchronous for-of must carry its dedicated protocol witness",
        );
        (self.statement, self.result_kind)
    }
}
