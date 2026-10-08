use crate::resumable_for_of_control::ResumableSyncForOfBranchOwner;
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
    ForeignBranchOwner,
    MaterializedBodyEnvironment,
}

#[derive(Clone, Copy)]
enum BodyEnvironmentPolicy {
    Retained,
    ForAwaitHeadOnly,
}

impl AsyncFunctionForOfBodyIr {
    pub(crate) fn new(
        statements: Vec<StatementIr>,
        entry_state: u32,
    ) -> Result<Self, AsyncFunctionForOfBodyError> {
        Self::new_with_environment_policy(statements, entry_state, BodyEnvironmentPolicy::Retained)
    }

    pub(crate) fn new_for_await(
        statements: Vec<StatementIr>,
        entry_state: u32,
    ) -> Result<Self, AsyncFunctionForOfBodyError> {
        Self::new_with_environment_policy(
            statements,
            entry_state,
            BodyEnvironmentPolicy::ForAwaitHeadOnly,
        )
    }

    fn new_with_environment_policy(
        statements: Vec<StatementIr>,
        entry_state: u32,
        environment_policy: BodyEnvironmentPolicy,
    ) -> Result<Self, AsyncFunctionForOfBodyError> {
        validate_branch_ownership(&statements, ResumableSyncForOfBranchOwner::CurrentLoop)?;
        let mut validation = BodyValidation {
            saw_await: false,
            environment_policy,
            eager_environment_walk: false,
        };
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

fn validate_branch_ownership(
    statements: &[StatementIr],
    owner: ResumableSyncForOfBranchOwner,
) -> Result<(), AsyncFunctionForOfBodyError> {
    statements
        .iter()
        .try_for_each(|statement| validate_statement_branch_ownership(statement, owner))
}

fn validate_statement_branch_ownership(
    statement: &StatementIr,
    owner: ResumableSyncForOfBranchOwner,
) -> Result<(), AsyncFunctionForOfBodyError> {
    match statement {
        StatementIr::EmptyStatementCompletion(item) => {
            validate_statement_branch_ownership(item.statement(), owner)
        }
        StatementIr::Block(block) | StatementIr::SyncDisposableScope { body: block, .. } => {
            validate_branch_ownership(&block.statements, owner)
        }
        StatementIr::LexicalBlock(statements) => validate_branch_ownership(statements, owner),
        // The opaque pattern constructor excludes escaping branches and
        // materialized body environments. Its iterator operations stay private
        // to that complete owner instead of entering this general body walk.
        StatementIr::AsyncFunctionArrayDestructuring(_) => Ok(()),
        StatementIr::If {
            then_branch,
            else_branch,
            ..
        }
        | StatementIr::AsyncFunctionIf {
            then_branch,
            else_branch,
            ..
        } => {
            validate_statement_branch_ownership(then_branch, owner)?;
            if let Some(branch) = else_branch {
                validate_statement_branch_ownership(branch, owner)?;
            }
            Ok(())
        }
        StatementIr::TryCatch {
            try_block,
            catch_block,
            ..
        } => {
            validate_branch_ownership(&try_block.statements, owner)?;
            validate_branch_ownership(&catch_block.statements, owner)
        }
        StatementIr::TryFinally {
            try_block,
            finally_block,
            ..
        } => {
            validate_branch_ownership(&try_block.statements, owner)?;
            validate_branch_ownership(&finally_block.statements, owner)
        }
        StatementIr::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
            ..
        } => {
            validate_branch_ownership(&try_block.statements, owner)?;
            validate_branch_ownership(&catch_block.statements, owner)?;
            validate_branch_ownership(&finally_block.statements, owner)
        }
        StatementIr::For { init, body, .. } => {
            if let Some(ForInitIr::Statements(statements)) = init {
                validate_branch_ownership(
                    statements,
                    ResumableSyncForOfBranchOwner::NestedStatement,
                )?;
            }
            validate_statement_branch_ownership(
                body,
                ResumableSyncForOfBranchOwner::NestedStatement,
            )
        }
        StatementIr::ForOfIterator { body, .. }
        | StatementIr::While { body, .. }
        | StatementIr::DoWhile { body, .. }
        | StatementIr::ForInArray { body, .. }
        | StatementIr::ForInString { body, .. }
        | StatementIr::ForInObject { body, .. }
        | StatementIr::Labelled {
            statement: body, ..
        } => validate_statement_branch_ownership(
            body,
            ResumableSyncForOfBranchOwner::NestedStatement,
        ),
        StatementIr::Switch {
            lexical_declarations,
            cases,
            ..
        } => {
            validate_branch_ownership(
                lexical_declarations,
                ResumableSyncForOfBranchOwner::NestedStatement,
            )?;
            for case in cases {
                validate_branch_ownership(
                    &case.body.statements,
                    ResumableSyncForOfBranchOwner::NestedStatement,
                )?;
            }
            Ok(())
        }
        StatementIr::ParameterInitialization { statements, .. } => {
            validate_branch_ownership(statements, ResumableSyncForOfBranchOwner::NestedStatement)
        }
        StatementIr::Break { label } | StatementIr::Continue { label } => match (owner, label) {
            (ResumableSyncForOfBranchOwner::CurrentLoop, None) => Ok(()),
            (ResumableSyncForOfBranchOwner::CurrentLoop, Some(_))
            | (ResumableSyncForOfBranchOwner::NestedStatement, _) => {
                Err(AsyncFunctionForOfBodyError::ForeignBranchOwner)
            }
        },
        StatementIr::ResumableClassDefinition(_)
        | StatementIr::ModuleUnitOnce { .. }
        | StatementIr::AsyncDisposableScope { .. }
        | StatementIr::GeneratorYield { .. }
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
        | StatementIr::OrdinaryGeneratorIf(_)
        | StatementIr::OrdinaryGeneratorSwitch(_)
        | StatementIr::OrdinaryGeneratorArrayDestructuring(_)
        | StatementIr::OrdinaryGeneratorWith(_)
        | StatementIr::AsyncFunctionWith(_)
        | StatementIr::ArrayDestructuringOperation(_)
        | StatementIr::GeneratorIf { .. }
        | StatementIr::AsyncFunctionWhile(_)
        | StatementIr::AsyncFunctionSwitch(_)
        | StatementIr::AsyncFunctionForOfIterator { .. }
        | StatementIr::GeneratorForOfIterator { .. } => {
            Err(AsyncFunctionForOfBodyError::UnsupportedContinuation)
        }
        StatementIr::AsyncAwait { .. }
        | StatementIr::Empty
        | StatementIr::ModuleImportBinding(_)
        | StatementIr::Lexical { .. }
        | StatementIr::AnnexBFunctionCopy { .. }
        | StatementIr::Var(_)
        | StatementIr::DeclarationEvaluation(_)
        | StatementIr::Expression(_)
        | StatementIr::Debugger
        | StatementIr::Throw(_)
        | StatementIr::Return(_) => Ok(()),
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
    environment_policy: BodyEnvironmentPolicy,
    eager_environment_walk: bool,
}

impl BodyValidation {
    fn environment(&self, materialized: bool) -> Result<(), AsyncFunctionForOfBodyError> {
        match (self.environment_policy, materialized) {
            (BodyEnvironmentPolicy::ForAwaitHeadOnly, true) => {
                Err(AsyncFunctionForOfBodyError::MaterializedBodyEnvironment)
            }
            (BodyEnvironmentPolicy::Retained, _)
            | (BodyEnvironmentPolicy::ForAwaitHeadOnly, false) => Ok(()),
        }
    }

    fn for_environment(
        &self,
        environment: Option<&crate::ForInOfEnvironmentIr>,
    ) -> Result<(), AsyncFunctionForOfBodyError> {
        self.environment(environment.is_some_and(|environment| {
            environment.tdz_environment.is_some() || environment.iteration_environment.is_some()
        }))
    }

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
            if matches!(self.environment_policy, BodyEnvironmentPolicy::Retained) {
                return Ok(());
            }
            // The synchronous proof excludes suspension. Reuse the same closed
            // walk to check nested environments without adopting eager try
            // clause reservations as resumable statement states.
            let previous = self.eager_environment_walk;
            self.eager_environment_walk = true;
            let result = self.statement(statement, state);
            self.eager_environment_walk = previous;
            return require_state(state, result?);
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
        self.environment(try_block.lexical_environment.is_some())?;
        if let Some(block) = catch_block {
            self.environment(block.lexical_environment.is_some())?;
        }
        if let Some(block) = finally_block {
            self.environment(block.lexical_environment.is_some())?;
        }
        if self.eager_environment_walk {
            require_state(state, self.sequence(&try_block.statements, state)?)?;
            if let Some(block) = catch_block {
                require_state(state, self.sequence(&block.statements, state)?)?;
            }
            if let Some(block) = finally_block {
                require_state(state, self.sequence(&block.statements, state)?)?;
            }
            return Ok(state);
        }
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
            StatementIr::Block(block) => {
                self.environment(block.lexical_environment.is_some())?;
                self.sequence(&block.statements, state)
            }
            StatementIr::EmptyStatementCompletion(item) => self.statement(item.statement(), state),
            StatementIr::LexicalBlock(statements) => self.sequence(statements, state),
            StatementIr::AsyncFunctionArrayDestructuring(plan) => {
                require_state(state, plan.entry_state())?;
                self.saw_await |= plan.contains_await();
                Ok(plan.exit_state())
            }
            StatementIr::AsyncFunctionWhile(_) | StatementIr::AsyncFunctionSwitch(_) => {
                Err(AsyncFunctionForOfBodyError::UnsupportedContinuation)
            }
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
                catch_parameter_environment,
                generator_plan,
                async_plan,
                ..
            } => {
                self.environment(catch_parameter_environment.is_some())?;
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
                catch_parameter_environment,
                generator_plan,
                async_plan,
                ..
            } => {
                self.environment(catch_parameter_environment.is_some())?;
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
            StatementIr::For {
                init,
                body,
                lexical_environment,
                ..
            } => {
                self.environment(lexical_environment.is_some())?;
                if matches!(
                    self.environment_policy,
                    BodyEnvironmentPolicy::ForAwaitHeadOnly
                ) {
                    if let Some(ForInitIr::Statements(statements)) = init {
                        require_state(state, self.sequence(statements, state)?)?;
                    }
                }
                if matches!(init, Some(ForInitIr::AsyncDisposable(_))) {
                    return Err(AsyncFunctionForOfBodyError::UnsupportedContinuation);
                }
                self.eager(body, state)?;
                Ok(state)
            }
            StatementIr::ForOfIterator {
                head,
                body,
                lexical_environment,
                ..
            } => {
                self.for_environment(lexical_environment.as_ref())?;
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
            StatementIr::ForInArray {
                body,
                lexical_environment,
                ..
            }
            | StatementIr::ForInString {
                body,
                lexical_environment,
                ..
            }
            | StatementIr::ForInObject {
                body,
                lexical_environment,
                ..
            } => {
                self.for_environment(lexical_environment.as_ref())?;
                self.eager(body, state)?;
                Ok(state)
            }
            StatementIr::While { body, .. }
            | StatementIr::DoWhile { body, .. }
            | StatementIr::Labelled {
                statement: body, ..
            } => {
                self.eager(body, state)?;
                Ok(state)
            }
            StatementIr::Switch {
                lexical_declarations,
                cases,
                lexical_environment,
                ..
            } => {
                self.environment(lexical_environment.is_some())?;
                require_state(state, self.sequence(lexical_declarations, state)?)?;
                for case in cases {
                    self.environment(case.body.lexical_environment.is_some())?;
                    require_state(state, self.sequence(&case.body.statements, state)?)?;
                }
                Ok(state)
            }
            StatementIr::SyncDisposableScope { body, .. } => {
                self.environment(body.lexical_environment.is_some())?;
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
            | StatementIr::OrdinaryGeneratorIf(_)
            | StatementIr::OrdinaryGeneratorSwitch(_)
            | StatementIr::OrdinaryGeneratorArrayDestructuring(_)
            | StatementIr::OrdinaryGeneratorWith(_)
            | StatementIr::AsyncFunctionWith(_)
            | StatementIr::ArrayDestructuringOperation(_)
            | StatementIr::GeneratorIf { .. }
            | StatementIr::AsyncFunctionForOfIterator { .. }
            | StatementIr::GeneratorForOfIterator { .. } => {
                Err(AsyncFunctionForOfBodyError::UnsupportedContinuation)
            }
            StatementIr::Break { .. } | StatementIr::Continue { .. } => Ok(state),
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
