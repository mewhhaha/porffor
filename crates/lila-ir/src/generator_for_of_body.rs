use crate::resumable_for_of_control::ResumableSyncForOfBranchOwner;
use crate::{BlockIr, ForInitIr, ForOfIteratorHeadIr, GeneratorTryPlanIr, StatementIr};

/// A complete iteration body whose continuation states are owned by the synchronous
/// generator statement dispatcher. Materialized lexical blocks remain in the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratorForOfBodyIr {
    statements: Vec<StatementIr>,
    entry_state: u32,
    exit_state: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GeneratorForOfBodyError {
    YieldRequired,
    StateOverflow { state: u32 },
    StateMismatch { expected: u32, actual: u32 },
    TryClauseLayout,
    UnsupportedContinuation,
    ForeignBranchOwner,
}

impl GeneratorForOfBodyIr {
    pub(crate) fn new(
        statements: Vec<StatementIr>,
        entry_state: u32,
    ) -> Result<Self, GeneratorForOfBodyError> {
        let mut validation = BodyValidation {
            saw_yield: false,
            branch_owner: ResumableSyncForOfBranchOwner::CurrentLoop,
        };
        let exit_state = validation.sequence(&statements, entry_state)?;
        if !validation.saw_yield {
            return Err(GeneratorForOfBodyError::YieldRequired);
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

fn successor(state: u32) -> Result<u32, GeneratorForOfBodyError> {
    state
        .checked_add(1)
        .ok_or(GeneratorForOfBodyError::StateOverflow { state })
}

fn require_state(expected: u32, actual: u32) -> Result<(), GeneratorForOfBodyError> {
    if expected == actual {
        Ok(())
    } else {
        Err(GeneratorForOfBodyError::StateMismatch { expected, actual })
    }
}

struct BodyValidation {
    saw_yield: bool,
    branch_owner: ResumableSyncForOfBranchOwner,
}

impl BodyValidation {
    fn sequence(
        &mut self,
        statements: &[StatementIr],
        mut state: u32,
    ) -> Result<u32, GeneratorForOfBodyError> {
        for statement in statements {
            state = self.statement(statement, state)?;
        }
        Ok(state)
    }

    fn eager(
        &mut self,
        statement: &StatementIr,
        state: u32,
    ) -> Result<(), GeneratorForOfBodyError> {
        // Ordinary branch/loop dispatch does not own continuation states, even
        // when a nested eager try has reserved clause states but no yield.
        require_state(state, self.statement(statement, state)?)
    }

    fn eager_child_owner(
        &mut self,
        statement: &StatementIr,
        state: u32,
    ) -> Result<(), GeneratorForOfBodyError> {
        let previous = self.branch_owner;
        self.branch_owner = ResumableSyncForOfBranchOwner::NestedStatement;
        let result = self.eager(statement, state);
        self.branch_owner = previous;
        result
    }

    fn try_statement(
        &mut self,
        try_block: &BlockIr,
        catch_block: Option<&BlockIr>,
        finally_block: Option<&BlockIr>,
        plan: Option<GeneratorTryPlanIr>,
        state: u32,
    ) -> Result<u32, GeneratorForOfBodyError> {
        let plan = plan.ok_or(GeneratorForOfBodyError::TryClauseLayout)?;
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
            _ => return Err(GeneratorForOfBodyError::TryClauseLayout),
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
            _ => return Err(GeneratorForOfBodyError::TryClauseLayout),
        }
        require_state(state, plan.exit_state)?;
        Ok(state)
    }

    fn statement(
        &mut self,
        statement: &StatementIr,
        state: u32,
    ) -> Result<u32, GeneratorForOfBodyError> {
        match statement {
            StatementIr::OrdinaryGeneratorArrayDestructuring(plan) => {
                require_state(state, plan.entry_state())?;
                let mut points = Vec::new();
                crate::generator_loop_control::collect_suspensions(
                    &plan.body().block().statements,
                    &mut points,
                );
                self.saw_yield |= !points.is_empty();
                Ok(plan.exit_state())
            }
            StatementIr::OrdinaryGeneratorIf(plan) => {
                require_state(state, plan.entry_state())?;
                let then_end = self.sequence(
                    &plan.then_branch().block().statements,
                    plan.then_branch().entry_state(),
                )?;
                require_state(plan.then_branch().end_state(), then_end)?;
                let else_end = self.sequence(
                    &plan.else_branch().block().statements,
                    plan.else_branch().entry_state(),
                )?;
                require_state(plan.else_branch().end_state(), else_end)?;
                Ok(plan.exit_state())
            }
            StatementIr::GeneratorYield {
                suspend_state,
                resume_state,
                ..
            } => {
                require_state(state, *suspend_state)?;
                require_state(successor(state)?, *resume_state)?;
                self.saw_yield = true;
                Ok(*resume_state)
            }
            StatementIr::EmptyStatementCompletion(item) => self.statement(item.statement(), state),
            StatementIr::Block(block) => self.sequence(&block.statements, state),
            StatementIr::LexicalBlock(statements) => self.sequence(statements, state),
            StatementIr::TryCatch {
                try_block,
                catch_block,
                generator_plan,
                async_plan,
                ..
            } => {
                if async_plan.is_some() {
                    return Err(GeneratorForOfBodyError::UnsupportedContinuation);
                }
                self.try_statement(try_block, Some(catch_block), None, *generator_plan, state)
            }
            StatementIr::TryFinally {
                try_block,
                finally_block,
                generator_plan,
                async_plan,
            } => {
                if async_plan.is_some() {
                    return Err(GeneratorForOfBodyError::UnsupportedContinuation);
                }
                self.try_statement(try_block, None, Some(finally_block), *generator_plan, state)
            }
            StatementIr::TryCatchFinally {
                try_block,
                catch_block,
                finally_block,
                generator_plan,
                async_plan,
                ..
            } => {
                if async_plan.is_some() {
                    return Err(GeneratorForOfBodyError::UnsupportedContinuation);
                }
                self.try_statement(
                    try_block,
                    Some(catch_block),
                    Some(finally_block),
                    *generator_plan,
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
                    return Err(GeneratorForOfBodyError::UnsupportedContinuation);
                }
                if let Some(ForInitIr::Statements(statements)) = init {
                    for statement in statements {
                        self.eager_child_owner(statement, state)?;
                    }
                }
                self.eager_child_owner(body, state)?;
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
                    return Err(GeneratorForOfBodyError::UnsupportedContinuation);
                }
                self.eager_child_owner(body, state)?;
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
                self.eager_child_owner(body, state)?;
                Ok(state)
            }
            StatementIr::Switch {
                lexical_declarations,
                cases,
                ..
            } => {
                let previous = self.branch_owner;
                self.branch_owner = ResumableSyncForOfBranchOwner::NestedStatement;
                let result = (|| {
                    require_state(state, self.sequence(lexical_declarations, state)?)?;
                    for case in cases {
                        require_state(state, self.sequence(&case.body.statements, state)?)?;
                    }
                    Ok(state)
                })();
                self.branch_owner = previous;
                result
            }
            StatementIr::Break { label } | StatementIr::Continue { label } => {
                match (self.branch_owner, label) {
                    (ResumableSyncForOfBranchOwner::CurrentLoop, None) => Ok(state),
                    (ResumableSyncForOfBranchOwner::CurrentLoop, Some(_))
                    | (ResumableSyncForOfBranchOwner::NestedStatement, _) => {
                        Err(GeneratorForOfBodyError::ForeignBranchOwner)
                    }
                }
            }
            StatementIr::ParameterInitialization { statements, .. } => {
                let previous = self.branch_owner;
                self.branch_owner = ResumableSyncForOfBranchOwner::NestedStatement;
                let result = self.sequence(statements, state);
                self.branch_owner = previous;
                require_state(state, result?)?;
                Ok(state)
            }
            StatementIr::ResumableClassDefinition(_)
            | StatementIr::ModuleUnitOnce { .. }
            | StatementIr::AsyncDisposableScope { .. }
            | StatementIr::SyncDisposableScope { .. }
            | StatementIr::AsyncAwait { .. }
            | StatementIr::AsyncModuleInstantiation
            | StatementIr::GeneratorLoop { .. }
            | StatementIr::AsyncGeneratorLoop(_)
            | StatementIr::AsyncGeneratorIf(_)
            | StatementIr::AsyncGeneratorWith(_)
            | StatementIr::AsyncGeneratorSwitch(_)
            | StatementIr::AsyncGeneratorArrayDestructuring(_)
            | StatementIr::AsyncGeneratorResourceScope(_)
            | StatementIr::AsyncGeneratorResourceRegistration(_)
            | StatementIr::AsyncGeneratorForOf(_)
            | StatementIr::AsyncGeneratorForIn(_)
            | StatementIr::OrdinaryGeneratorLoop(_)
            | StatementIr::OrdinaryGeneratorSwitch(_)
            | StatementIr::AsyncFunctionArrayDestructuring(_)
            | StatementIr::AsyncFunctionWith(_)
            | StatementIr::OrdinaryGeneratorWith(_)
            | StatementIr::ArrayDestructuringOperation(_)
            | StatementIr::GeneratorIf { .. }
            | StatementIr::AsyncFunctionForOfIterator { .. }
            | StatementIr::AsyncFunctionIf { .. }
            | StatementIr::AsyncFunctionWhile(_)
            | StatementIr::AsyncFunctionSwitch(_)
            | StatementIr::GeneratorForOfIterator { .. } => {
                Err(GeneratorForOfBodyError::UnsupportedContinuation)
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
