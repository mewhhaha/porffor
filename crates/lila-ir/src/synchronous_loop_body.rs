use crate::{ForInitIr, ForOfIteratorHeadIr, StatementIr};

/// A borrowed loop body accepted by ordinary statement emission. The private
/// field prevents a resource-loop consumer from using a suspending body with
/// temporary-local disposal storage, even for manually constructed IR.
#[derive(Debug, Clone, Copy)]
pub struct SynchronousLoopBodyIr<'a> {
    statement: &'a StatementIr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SynchronousLoopBodyError {
    Suspension,
    ContinuationOwner,
}

impl<'a> SynchronousLoopBodyIr<'a> {
    pub fn new(statement: &'a StatementIr) -> Result<Self, SynchronousLoopBodyError> {
        if crate::ir::statement_contains_suspension(statement) {
            return Err(SynchronousLoopBodyError::Suspension);
        }
        validate(statement)?;
        Ok(Self { statement })
    }

    pub const fn statement(self) -> &'a StatementIr {
        self.statement
    }
}

fn sequence(statements: &[StatementIr]) -> Result<(), SynchronousLoopBodyError> {
    statements.iter().try_for_each(validate)
}

fn validate(statement: &StatementIr) -> Result<(), SynchronousLoopBodyError> {
    match statement {
        StatementIr::Block(block) | StatementIr::SyncDisposableScope { body: block, .. } => {
            sequence(&block.statements)
        }
        StatementIr::LexicalBlock(statements)
        | StatementIr::ParameterInitialization { statements, .. } => sequence(statements),
        StatementIr::If {
            then_branch,
            else_branch,
            ..
        } => {
            validate(then_branch)?;
            if let Some(branch) = else_branch {
                validate(branch)?;
            }
            Ok(())
        }
        StatementIr::For { init, body, .. } => {
            match init {
                Some(ForInitIr::Statements(statements)) => sequence(statements)?,
                Some(ForInitIr::AsyncDisposable(_)) => {
                    return Err(SynchronousLoopBodyError::Suspension);
                }
                Some(
                    ForInitIr::Lexical { .. }
                    | ForInitIr::LexicalBlock(_)
                    | ForInitIr::Var(_)
                    | ForInitIr::Expression(_)
                    | ForInitIr::SyncDisposable(_),
                )
                | None => {}
            }
            validate(body)
        }
        StatementIr::ForOfIterator { head, body, .. } => {
            match head {
                ForOfIteratorHeadIr::Assignment {
                    async_plan: Some(_),
                    ..
                }
                | ForOfIteratorHeadIr::AsyncDisposable(_) => {
                    return Err(SynchronousLoopBodyError::Suspension);
                }
                ForOfIteratorHeadIr::Assignment {
                    async_plan: None, ..
                }
                | ForOfIteratorHeadIr::SyncDisposable(_) => {}
            }
            validate(body)
        }
        StatementIr::While { body, .. }
        | StatementIr::DoWhile { body, .. }
        | StatementIr::ForInArray { body, .. }
        | StatementIr::ForInString { body, .. }
        | StatementIr::ForInObject { body, .. }
        | StatementIr::Labelled {
            statement: body, ..
        } => validate(body),
        StatementIr::Switch {
            lexical_declarations,
            cases,
            ..
        } => {
            sequence(lexical_declarations)?;
            cases
                .iter()
                .try_for_each(|case| sequence(&case.body.statements))
        }
        StatementIr::TryCatch {
            try_block,
            catch_block,
            generator_plan,
            async_plan,
            ..
        } => {
            if generator_plan.is_some() || async_plan.is_some() {
                return Err(SynchronousLoopBodyError::ContinuationOwner);
            }
            sequence(&try_block.statements)?;
            sequence(&catch_block.statements)
        }
        StatementIr::TryFinally {
            try_block,
            finally_block,
            generator_plan,
            async_plan,
        } => {
            if generator_plan.is_some() || async_plan.is_some() {
                return Err(SynchronousLoopBodyError::ContinuationOwner);
            }
            sequence(&try_block.statements)?;
            sequence(&finally_block.statements)
        }
        StatementIr::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
            generator_plan,
            async_plan,
            ..
        } => {
            if generator_plan.is_some() || async_plan.is_some() {
                return Err(SynchronousLoopBodyError::ContinuationOwner);
            }
            sequence(&try_block.statements)?;
            sequence(&catch_block.statements)?;
            sequence(&finally_block.statements)
        }
        StatementIr::ResumableClassDefinition(_)
        | StatementIr::ModuleUnitOnce { .. }
        | StatementIr::AsyncDisposableScope { .. }
        | StatementIr::GeneratorYield { .. }
        | StatementIr::AsyncAwait { .. }
        | StatementIr::AsyncModuleInstantiation
        | StatementIr::GeneratorLoop { .. }
        | StatementIr::GeneratorIf { .. }
        | StatementIr::AsyncFunctionIf { .. }
        | StatementIr::AsyncFunctionForOfIterator { .. } => {
            Err(SynchronousLoopBodyError::ContinuationOwner)
        }
        StatementIr::Empty
        | StatementIr::ModuleImportBinding(_)
        | StatementIr::Lexical { .. }
        | StatementIr::AnnexBFunctionCopy { .. }
        | StatementIr::Var(_)
        | StatementIr::DeclarationEvaluation(_)
        | StatementIr::Expression(_)
        | StatementIr::Debugger
        | StatementIr::Throw(_)
        | StatementIr::Return(_)
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => Ok(()),
    }
}
