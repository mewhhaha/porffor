use crate::{AsyncTryPlanIr, BlockIr, ForInitIr, ForOfIteratorHeadIr, StatementIr};

/// A complete iteration body whose continuation states are owned by the plain
/// async statement dispatcher. Materialized lexical blocks remain in the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncFunctionForOfBodyIr {
    statements: Vec<StatementIr>,
    entry_state: u32,
    exit_state: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AsyncFunctionForOfBodyError {
    AwaitRequired,
    StateOverflow { state: u32 },
    StateMismatch { expected: u32, actual: u32 },
    TryClauseLayout,
    UnsupportedContinuation,
}

impl AsyncFunctionForOfBodyIr {
    pub(crate) fn new(
        statements: Vec<StatementIr>,
        entry_state: u32,
    ) -> Result<Self, AsyncFunctionForOfBodyError> {
        let mut validation = BodyValidation { saw_await: false };
        let exit_state = validation.sequence(&statements, entry_state)?;
        if !validation.saw_await {
            return Err(AsyncFunctionForOfBodyError::AwaitRequired);
        }
        Ok(Self {
            statements,
            entry_state,
            exit_state,
        })
    }

    pub fn statements(&self) -> &[StatementIr] {
        &self.statements
    }
    pub const fn entry_state(&self) -> u32 {
        self.entry_state
    }
    pub const fn exit_state(&self) -> u32 {
        self.exit_state
    }
}

fn successor(state: u32) -> Result<u32, AsyncFunctionForOfBodyError> {
    state
        .checked_add(1)
        .ok_or(AsyncFunctionForOfBodyError::StateOverflow { state })
}

fn require_state(expected: u32, actual: u32) -> Result<(), AsyncFunctionForOfBodyError> {
    if expected == actual {
        Ok(())
    } else {
        Err(AsyncFunctionForOfBodyError::StateMismatch { expected, actual })
    }
}

struct BodyValidation {
    saw_await: bool,
}

impl BodyValidation {
    fn sequence(
        &mut self,
        statements: &[StatementIr],
        mut state: u32,
    ) -> Result<u32, AsyncFunctionForOfBodyError> {
        for statement in statements {
            state = self.statement(statement, state)?;
        }
        Ok(state)
    }

    fn eager(
        &mut self,
        statement: &StatementIr,
        state: u32,
    ) -> Result<(), AsyncFunctionForOfBodyError> {
        if crate::SynchronousLoopBodyIr::new(statement).is_ok() {
            return Ok(());
        }
        // Ordinary branch/loop dispatch does not own continuation states, even
        // when a nested eager try has reserved clause states but no await.
        require_state(state, self.statement(statement, state)?)
    }

    fn try_statement(
        &mut self,
        try_block: &BlockIr,
        catch_block: Option<&BlockIr>,
        finally_block: Option<&BlockIr>,
        plan: Option<AsyncTryPlanIr>,
        state: u32,
    ) -> Result<u32, AsyncFunctionForOfBodyError> {
        let plan = plan.ok_or(AsyncFunctionForOfBodyError::TryClauseLayout)?;
        require_state(state, plan.entry_state)?;
        let try_end = self.sequence(&try_block.statements, state)?;
        require_state(successor(try_end)?, plan.try_exit_state)?;
        let mut state = plan.try_exit_state;
        match (catch_block, plan.catch_entry_state, plan.catch_exit_state) {
            (Some(block), Some(entry), Some(exit)) => {
                require_state(state, entry)?;
                let end = self.sequence(&block.statements, entry)?;
                require_state(successor(end)?, exit)?;
                state = exit;
            }
            (None, None, None) => {}
            _ => return Err(AsyncFunctionForOfBodyError::TryClauseLayout),
        }
        match (
            finally_block,
            plan.finally_entry_state,
            plan.finally_exit_state,
        ) {
            (Some(block), Some(entry), Some(exit)) => {
                require_state(state, entry)?;
                let end = self.sequence(&block.statements, entry)?;
                require_state(successor(end)?, exit)?;
                state = exit;
            }
            (None, None, None) => {}
            _ => return Err(AsyncFunctionForOfBodyError::TryClauseLayout),
        }
        require_state(state, plan.exit_state)?;
        Ok(state)
    }

    fn statement(
        &mut self,
        statement: &StatementIr,
        state: u32,
    ) -> Result<u32, AsyncFunctionForOfBodyError> {
        match statement {
            StatementIr::AsyncAwait {
                suspend_state,
                resume_state,
                ..
            } => {
                require_state(state, *suspend_state)?;
                require_state(successor(state)?, *resume_state)?;
                self.saw_await = true;
                Ok(*resume_state)
            }
            StatementIr::Block(block) => self.sequence(&block.statements, state),
            StatementIr::LexicalBlock(statements) => self.sequence(statements, state),
            StatementIr::AsyncFunctionIf {
                then_branch,
                else_branch,
                plan,
                ..
            } => {
                require_state(state, plan.entry_state())?;
                require_state(successor(state)?, plan.then_entry_state())?;
                let then_end = self.statement(then_branch, plan.then_entry_state())?;
                require_state(successor(then_end)?, plan.else_entry_state())?;
                let else_end = if let Some(branch) = else_branch {
                    self.statement(branch, plan.else_entry_state())?
                } else {
                    plan.else_entry_state()
                };
                require_state(successor(else_end)?, plan.exit_state())?;
                Ok(plan.exit_state())
            }
            StatementIr::TryCatch {
                try_block,
                catch_block,
                generator_plan,
                async_plan,
                ..
            } => {
                if generator_plan.is_some() {
                    return Err(AsyncFunctionForOfBodyError::UnsupportedContinuation);
                }
                self.try_statement(try_block, Some(catch_block), None, *async_plan, state)
            }
            StatementIr::TryFinally {
                try_block,
                finally_block,
                generator_plan,
                async_plan,
            } => {
                if generator_plan.is_some() {
                    return Err(AsyncFunctionForOfBodyError::UnsupportedContinuation);
                }
                self.try_statement(try_block, None, Some(finally_block), *async_plan, state)
            }
            StatementIr::TryCatchFinally {
                try_block,
                catch_block,
                finally_block,
                generator_plan,
                async_plan,
                ..
            } => {
                if generator_plan.is_some() {
                    return Err(AsyncFunctionForOfBodyError::UnsupportedContinuation);
                }
                self.try_statement(
                    try_block,
                    Some(catch_block),
                    Some(finally_block),
                    *async_plan,
                    state,
                )
            }
            StatementIr::If {
                then_branch,
                else_branch,
                ..
            } => {
                self.eager(then_branch, state)?;
                if let Some(branch) = else_branch {
                    self.eager(branch, state)?;
                }
                Ok(state)
            }
            StatementIr::For { init, body, .. } => {
                if matches!(init, Some(ForInitIr::AsyncDisposable(_))) {
                    return Err(AsyncFunctionForOfBodyError::UnsupportedContinuation);
                }
                self.eager(body, state)?;
                Ok(state)
            }
            StatementIr::ForOfIterator { head, body, .. } => {
                if matches!(
                    head,
                    ForOfIteratorHeadIr::AsyncDisposable(_)
                        | ForOfIteratorHeadIr::Assignment {
                            async_plan: Some(_),
                            ..
                        }
                ) {
                    return Err(AsyncFunctionForOfBodyError::UnsupportedContinuation);
                }
                self.eager(body, state)?;
                Ok(state)
            }
            StatementIr::While { body, .. }
            | StatementIr::DoWhile { body, .. }
            | StatementIr::ForInArray { body, .. }
            | StatementIr::ForInString { body, .. }
            | StatementIr::ForInObject { body, .. }
            | StatementIr::Labelled {
                statement: body, ..
            } => {
                self.eager(body, state)?;
                Ok(state)
            }
            StatementIr::Switch {
                lexical_declarations,
                cases,
                ..
            } => {
                require_state(state, self.sequence(lexical_declarations, state)?)?;
                for case in cases {
                    require_state(state, self.sequence(&case.body.statements, state)?)?;
                }
                Ok(state)
            }
            StatementIr::SyncDisposableScope { body, .. } => {
                require_state(state, self.sequence(&body.statements, state)?)?;
                Ok(state)
            }
            StatementIr::ParameterInitialization { statements, .. } => {
                require_state(state, self.sequence(statements, state)?)?;
                Ok(state)
            }
            StatementIr::ResumableClassDefinition(_)
            | StatementIr::ModuleUnitOnce { .. }
            | StatementIr::AsyncDisposableScope { .. }
            | StatementIr::GeneratorYield { .. }
            | StatementIr::AsyncModuleInstantiation
            | StatementIr::GeneratorLoop { .. }
            | StatementIr::GeneratorIf { .. }
            | StatementIr::AsyncFunctionForOfIterator { .. }
            | StatementIr::Break { .. }
            | StatementIr::Continue { .. } => {
                Err(AsyncFunctionForOfBodyError::UnsupportedContinuation)
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
            | StatementIr::Return(_) => Ok(state),
        }
    }
}

#[cfg(test)]
mod tests;
