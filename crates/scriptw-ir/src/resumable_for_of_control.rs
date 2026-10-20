use crate::*;

#[derive(Clone, Copy)]
pub(crate) enum ResumableSyncForOfBranchOwner {
    CurrentLoop,
    NestedStatement,
}

pub(crate) fn resumable_sync_for_of_body_has_local_control_owners(body: &Statement) -> bool {
    ResumableSyncForOfControlVisitor {
        owner: ResumableSyncForOfBranchOwner::CurrentLoop,
    }
    .visit_statement(body)
    .is_continue()
}

/// Only the current resumable synchronous for-of consumes these unlabelled
/// branches. Child statement owners cannot borrow its targets.
struct ResumableSyncForOfControlVisitor {
    owner: ResumableSyncForOfBranchOwner,
}

impl<'ast> Visitor<'ast> for ResumableSyncForOfControlVisitor {
    type BreakTy = ();

    fn visit_statement(&mut self, statement: &'ast Statement) -> ControlFlow<Self::BreakTy> {
        let label = match statement {
            Statement::Break(statement) => statement.label(),
            Statement::Continue(statement) => statement.label(),
            Statement::DoWhileLoop(_)
            | Statement::WhileLoop(_)
            | Statement::ForLoop(_)
            | Statement::ForInLoop(_)
            | Statement::ForOfLoop(_)
            | Statement::Switch(_)
            | Statement::Labelled(_)
            | Statement::With(_) => {
                let previous = self.owner;
                self.owner = ResumableSyncForOfBranchOwner::NestedStatement;
                let result = statement.visit_with(self);
                self.owner = previous;
                return result;
            }
            Statement::Block(_)
            | Statement::Var(_)
            | Statement::Empty
            | Statement::Debugger
            | Statement::Expression(_)
            | Statement::If(_)
            | Statement::Return(_)
            | Statement::Throw(_)
            | Statement::Try(_) => return statement.visit_with(self),
        };
        match (self.owner, label) {
            (ResumableSyncForOfBranchOwner::CurrentLoop, None) => ControlFlow::Continue(()),
            (ResumableSyncForOfBranchOwner::CurrentLoop, Some(_))
            | (ResumableSyncForOfBranchOwner::NestedStatement, _) => ControlFlow::Break(()),
        }
    }

    // Branches in nested callables and class bodies have separate owners. The
    // existing source planner/lowerer validates those bodies independently.
    fn visit_function_body(&mut self, _body: &'ast FunctionBody) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }

    fn visit_class_declaration(
        &mut self,
        _class: &'ast ClassDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }

    fn visit_class_expression(
        &mut self,
        _class: &'ast ClassExpression,
    ) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }
}
