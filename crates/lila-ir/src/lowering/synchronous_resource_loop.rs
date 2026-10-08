use super::*;

mod source_suspension;
use source_suspension::SourceSuspension;

/// A resource loop whose eagerly evaluated source cannot suspend. Its private
/// constructor is the only entry to lowering without allocating continuation
/// states for nested ordinary try clauses. The function protocol is retained.
pub(super) struct SynchronousResourceLoop<'ast> {
    source: ResourceLoopSource<'ast>,
}

enum ResourceLoopSource<'ast> {
    Classic(&'ast ForLoop),
    Iterator(&'ast ForOfLoop),
}

impl<'ast> SynchronousResourceLoop<'ast> {
    pub(super) fn classic(source: &'ast ForLoop) -> Option<Self> {
        source
            .visit_with(&mut SourceSuspension)
            .is_continue()
            .then_some(Self {
                source: ResourceLoopSource::Classic(source),
            })
    }

    pub(super) fn iterator(source: &'ast ForOfLoop) -> Option<Self> {
        SourceSuspension
            .visit_for_of_loop(source)
            .is_continue()
            .then_some(Self {
                source: ResourceLoopSource::Iterator(source),
            })
    }

    pub(super) fn lower(self, lowerer: &mut ScriptLowerer<'_>) -> (StatementIr, ValueKind) {
        let Some(owner) = lowerer.admit_sync_disposable_scope_owner() else {
            return (StatementIr::Empty, ValueKind::Undefined);
        };
        match owner {
            SyncDisposableScopeOwnerPlan::Immediate
            | SyncDisposableScopeOwnerPlan::AsyncFunction => {}
            SyncDisposableScopeOwnerPlan::PlainGenerator
            | SyncDisposableScopeOwnerPlan::AsyncGenerator => {
                lowerer.unsupported("synchronous resource loop in a generator");
                return (StatementIr::Empty, ValueKind::Undefined);
            }
        }
        // Only this proven region omits continuation allocation. Keeping the
        // analyzed owner intact preserves activation-backed nested using scopes,
        // return settlement, captured environments and the execution Realm.
        let continuation = lowerer.current_async_resume_state.take();
        let result = match self.source {
            ResourceLoopSource::Classic(source) => lowerer.lower_for_loop_region(source),
            ResourceLoopSource::Iterator(source) => lowerer.lower_for_of_loop_region(source),
        };
        assert!(
            lowerer.current_async_resume_state.is_none(),
            "a synchronous resource region must not allocate a continuation"
        );
        lowerer.current_async_resume_state = continuation;
        result
    }
}

/// Includes implicit disposal and iterator awaits in the current activation,
/// while nested function bodies retain their independent continuation owner.
pub(crate) fn source_statement_suspends(source: &Statement) -> bool {
    source.visit_with(&mut SourceSuspension).is_break()
}

pub(super) fn source_expression_suspends(source: &Expression) -> bool {
    source.visit_with(&mut SourceSuspension).is_break()
}
