use crate::{
    AsyncDisposableScopeExecutionIr, BlockIr, ExprIr, LexicalEnvironmentIr, StatementIr,
    SyncDisposableScopeExecutionIr, TypedExpr,
};

/// A selector either runs during eager selection or owns a retained selection
/// segment. The latter can be built only after its actual prefix is validated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsyncFunctionSwitchSelectorIr {
    Eager(TypedExpr),
    Resumable(AsyncFunctionSwitchSelectorContinuationIr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncFunctionSwitchSelectorContinuationIr {
    prefix: Vec<StatementIr>,
    condition: TypedExpr,
    entry_state: u32,
    ready_state: u32,
    next_state: u32,
}

impl AsyncFunctionSwitchSelectorContinuationIr {
    pub(crate) fn new(
        prefix: Vec<StatementIr>,
        condition: TypedExpr,
        entry_state: u32,
        ready_state: u32,
    ) -> Result<Self, AsyncSwitchError> {
        if sequence_exit(&prefix, entry_state)? != ready_state {
            return Err(AsyncSwitchError::StateOrder);
        }
        Ok(Self {
            prefix,
            condition,
            entry_state,
            ready_state,
            next_state: ready_state
                .checked_add(1)
                .ok_or(AsyncSwitchError::StateOverflow)?,
        })
    }

    pub fn prefix(&self) -> &[StatementIr] {
        &self.prefix
    }
    pub fn condition(&self) -> &TypedExpr {
        &self.condition
    }
    pub const fn entry_state(&self) -> u32 {
        self.entry_state
    }
    pub const fn ready_state(&self) -> u32 {
        self.ready_state
    }
    pub const fn next_state(&self) -> u32 {
        self.next_state
    }
}

/// One actual CaseBlock body coupled to its checked continuation segment.
/// Even an eager case owns a distinct segment: selection may skip it, while
/// normal fallthrough enters the next case without evaluating another selector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncFunctionSwitchCaseIr {
    selector: Option<AsyncFunctionSwitchSelectorIr>,
    body: BlockIr,
    entry_state: u32,
    next_entry_state: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncFunctionSwitchIr {
    discriminant: TypedExpr,
    lexical_environment: Option<LexicalEnvironmentIr>,
    lexical_declarations: Vec<StatementIr>,
    cases: Vec<AsyncFunctionSwitchCaseIr>,
    selection: AsyncSwitchSelection,
    entry_state: u32,
    exit_state: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AsyncSwitchSelection {
    Eager,
    Resumable { fallback_state: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AsyncSwitchError {
    StateOverflow,
    StateOrder,
    MissingChildOwner,
    UnsupportedChildOwner,
    PerCaseEnvironment,
    DuplicateDefault,
    MixedSelectorOwnership,
    MissingRetainedDiscriminant,
}

impl AsyncFunctionSwitchCaseIr {
    pub(crate) fn new(
        selector: Option<AsyncFunctionSwitchSelectorIr>,
        body: BlockIr,
        entry_state: u32,
        body_exit_state: u32,
    ) -> Result<Self, AsyncSwitchError> {
        if body.lexical_environment.is_some() {
            return Err(AsyncSwitchError::PerCaseEnvironment);
        }
        let actual_exit = sequence_exit(&body.statements, entry_state)?;
        if actual_exit != body_exit_state {
            return Err(AsyncSwitchError::StateOrder);
        }
        let next_entry_state = actual_exit
            .checked_add(1)
            .ok_or(AsyncSwitchError::StateOverflow)?;
        Ok(Self {
            selector,
            body,
            entry_state,
            next_entry_state,
        })
    }

    pub(crate) fn into_eager_case(self) -> Result<crate::SwitchCaseIr, AsyncSwitchError> {
        if self.entry_state.checked_add(1) != Some(self.next_entry_state) {
            return Err(AsyncSwitchError::MissingChildOwner);
        }
        let condition = match self.selector {
            Some(AsyncFunctionSwitchSelectorIr::Eager(condition)) => Some(condition),
            Some(AsyncFunctionSwitchSelectorIr::Resumable(_)) => {
                return Err(AsyncSwitchError::MissingChildOwner);
            }
            None => None,
        };
        Ok(crate::SwitchCaseIr {
            condition,
            body: self.body,
        })
    }

    pub fn condition(&self) -> Option<&TypedExpr> {
        self.selector.as_ref().map(|selector| match selector {
            AsyncFunctionSwitchSelectorIr::Eager(condition) => condition,
            AsyncFunctionSwitchSelectorIr::Resumable(owner) => owner.condition(),
        })
    }
    pub fn selector(&self) -> Option<&AsyncFunctionSwitchSelectorIr> {
        self.selector.as_ref()
    }
    pub fn condition_prefix(&self) -> &[StatementIr] {
        match self.selector.as_ref() {
            Some(AsyncFunctionSwitchSelectorIr::Resumable(owner)) => owner.prefix(),
            Some(AsyncFunctionSwitchSelectorIr::Eager(_)) | None => &[],
        }
    }
    pub fn body(&self) -> &BlockIr {
        &self.body
    }
    pub const fn entry_state(&self) -> u32 {
        self.entry_state
    }
    pub const fn next_entry_state(&self) -> u32 {
        self.next_entry_state
    }
}

impl AsyncFunctionSwitchIr {
    pub(crate) fn new(
        discriminant: TypedExpr,
        lexical_environment: Option<LexicalEnvironmentIr>,
        lexical_declarations: Vec<StatementIr>,
        cases: Vec<AsyncFunctionSwitchCaseIr>,
        entry_state: u32,
    ) -> Result<Self, AsyncSwitchError> {
        // A staged discriminator and its retained cell precede CaseBlock. A
        // resumable selector prefix belongs to this shared environment instead.
        if sequence_exit(&lexical_declarations, entry_state)? != entry_state {
            return Err(AsyncSwitchError::MissingChildOwner);
        }
        let selection_entry = entry_state
            .checked_add(1)
            .ok_or(AsyncSwitchError::StateOverflow)?;
        let resumable_selection = cases.iter().any(|case| {
            matches!(
                case.selector(),
                Some(AsyncFunctionSwitchSelectorIr::Resumable(_))
            )
        });
        let (selection, mut next) = if resumable_selection {
            if !matches!(discriminant.expr, ExprIr::Identifier(_)) {
                return Err(AsyncSwitchError::MissingRetainedDiscriminant);
            }
            let mut next = selection_entry;
            for case in &cases {
                match case.selector() {
                    Some(AsyncFunctionSwitchSelectorIr::Resumable(owner)) => {
                        if owner.entry_state() != next {
                            return Err(AsyncSwitchError::StateOrder);
                        }
                        next = owner.next_state();
                    }
                    Some(AsyncFunctionSwitchSelectorIr::Eager(_)) => {
                        return Err(AsyncSwitchError::MixedSelectorOwnership);
                    }
                    None => {}
                }
            }
            let first_body = next.checked_add(1).ok_or(AsyncSwitchError::StateOverflow)?;
            (
                AsyncSwitchSelection::Resumable {
                    fallback_state: next,
                },
                first_body,
            )
        } else {
            (AsyncSwitchSelection::Eager, selection_entry)
        };
        let mut saw_default = false;
        for case in &cases {
            if case.entry_state != next {
                return Err(AsyncSwitchError::StateOrder);
            }
            if case.selector.is_none() {
                if saw_default {
                    return Err(AsyncSwitchError::DuplicateDefault);
                }
                saw_default = true;
            }
            next = case.next_entry_state;
        }
        Ok(Self {
            discriminant,
            lexical_environment,
            lexical_declarations,
            cases,
            selection,
            entry_state,
            exit_state: next,
        })
    }

    pub fn discriminant(&self) -> &TypedExpr {
        &self.discriminant
    }
    pub fn lexical_environment(&self) -> Option<&LexicalEnvironmentIr> {
        self.lexical_environment.as_ref()
    }
    pub fn lexical_declarations(&self) -> &[StatementIr] {
        &self.lexical_declarations
    }
    pub fn cases(&self) -> &[AsyncFunctionSwitchCaseIr] {
        &self.cases
    }
    pub const fn selection_fallback_state(&self) -> Option<u32> {
        match self.selection {
            AsyncSwitchSelection::Eager => None,
            AsyncSwitchSelection::Resumable { fallback_state } => Some(fallback_state),
        }
    }
    pub const fn entry_state(&self) -> u32 {
        self.entry_state
    }
    pub const fn exit_state(&self) -> u32 {
        self.exit_state
    }
}

pub(crate) fn sequence_exit(
    statements: &[StatementIr],
    entry: u32,
) -> Result<u32, AsyncSwitchError> {
    let mut state = entry;
    for statement in statements {
        if let Some((start, end)) = span(statement)? {
            if start != state || end < start {
                return Err(AsyncSwitchError::StateOrder);
            }
            state = end;
        }
    }
    Ok(state)
}

fn block_span(block: &BlockIr) -> Result<Option<(u32, u32)>, AsyncSwitchError> {
    sequence_span(&block.statements)
}

fn sequence_span(statements: &[StatementIr]) -> Result<Option<(u32, u32)>, AsyncSwitchError> {
    let mut result: Option<(u32, u32)> = None;
    for statement in statements {
        if let Some((start, end)) = span(statement)? {
            if let Some((_, previous)) = result {
                if previous != start {
                    return Err(AsyncSwitchError::StateOrder);
                }
            }
            result = Some((result.map_or(start, |pair| pair.0), end));
        }
    }
    Ok(result)
}

fn branch_inside(statement: &StatementIr, entry: u32, limit: u32) -> Result<(), AsyncSwitchError> {
    if let Some((start, end)) = span(statement)? {
        if start != entry || end >= limit {
            return Err(AsyncSwitchError::StateOrder);
        }
    }
    Ok(())
}

fn try_span(
    plan: &crate::AsyncTryPlanIr,
    try_block: &BlockIr,
    catch: Option<&BlockIr>,
    finally: Option<&BlockIr>,
) -> Result<(u32, u32), AsyncSwitchError> {
    let clause_exit = |block: &BlockIr, entry| {
        sequence_exit(&block.statements, entry)?
            .checked_add(1)
            .ok_or(AsyncSwitchError::StateOverflow)
    };
    let mut next = clause_exit(try_block, plan.entry_state)?;
    if next != plan.try_exit_state {
        return Err(AsyncSwitchError::StateOrder);
    }
    match catch {
        Some(block) => {
            if plan.catch_entry_state != Some(next) {
                return Err(AsyncSwitchError::StateOrder);
            }
            next = clause_exit(block, next)?;
            if plan.catch_exit_state != Some(next) {
                return Err(AsyncSwitchError::StateOrder);
            }
        }
        None => {
            if plan.catch_entry_state.is_some() || plan.catch_exit_state.is_some() {
                return Err(AsyncSwitchError::StateOrder);
            }
        }
    }
    match finally {
        Some(block) => {
            if plan.finally_entry_state != Some(next) {
                return Err(AsyncSwitchError::StateOrder);
            }
            next = clause_exit(block, next)?;
            if plan.finally_exit_state != Some(next) {
                return Err(AsyncSwitchError::StateOrder);
            }
        }
        None => {
            if plan.finally_entry_state.is_some() || plan.finally_exit_state.is_some() {
                return Err(AsyncSwitchError::StateOrder);
            }
        }
    }
    if next != plan.exit_state {
        return Err(AsyncSwitchError::StateOrder);
    }
    Ok((plan.entry_state, next))
}

/// Closed same-activation census. Nested function definitions are expressions
/// referring to separately lowered functions, never continuations of this case.
fn span(statement: &StatementIr) -> Result<Option<(u32, u32)>, AsyncSwitchError> {
    let owned = match statement {
        StatementIr::AsyncAwait {
            suspend_state,
            resume_state,
            ..
        } => {
            if suspend_state.checked_add(1) != Some(*resume_state) {
                return Err(AsyncSwitchError::StateOrder);
            }
            Some((*suspend_state, *resume_state))
        }
        StatementIr::AsyncFunctionSwitch(plan) => Some((plan.entry_state(), plan.exit_state())),
        StatementIr::AsyncFunctionWhile(plan) => Some((plan.entry_state(), plan.exit_state())),
        StatementIr::AsyncGeneratorLoop(plan) => {
            if plan.execution() != crate::ResumableRegionProtocolIr::Async {
                return Err(AsyncSwitchError::MissingChildOwner);
            }
            Some((plan.entry_state(), plan.exit_state()))
        }
        StatementIr::AsyncGeneratorResourceScope(plan) => {
            if plan.execution() != crate::ResumableRegionProtocolIr::Async {
                return Err(AsyncSwitchError::UnsupportedChildOwner);
            }
            Some((plan.entry_state(), plan.exit_state()))
        }
        StatementIr::AsyncGeneratorSwitch(plan) => {
            if plan.execution() != crate::ResumableRegionProtocolIr::Async {
                return Err(AsyncSwitchError::UnsupportedChildOwner);
            }
            Some((plan.entry_state(), plan.exit_state()))
        }
        StatementIr::AsyncFunctionArrayDestructuring(plan) => {
            Some((plan.entry_state(), plan.exit_state()))
        }
        StatementIr::AsyncFunctionWith(plan) => Some((plan.entry_state(), plan.exit_state())),
        StatementIr::AsyncGeneratorForIn(plan) => {
            if plan.execution() != crate::ResumableRegionProtocolIr::Async {
                return Err(AsyncSwitchError::UnsupportedChildOwner);
            }
            Some((plan.entry_state(), plan.exit_state()))
        }
        StatementIr::AsyncGeneratorForOf(plan) => {
            if plan.execution() != crate::ResumableRegionProtocolIr::Async {
                return Err(AsyncSwitchError::UnsupportedChildOwner);
            }
            Some((plan.entry_state(), plan.exit_state()))
        }
        StatementIr::ResumableClassDefinition(plan) => {
            Some((plan.entry_state(), plan.exit_state()))
        }
        StatementIr::AsyncFunctionIf {
            plan,
            then_branch,
            else_branch,
            ..
        } => {
            branch_inside(
                then_branch,
                plan.then_entry_state(),
                plan.else_entry_state(),
            )?;
            if let Some(branch) = else_branch {
                branch_inside(branch, plan.else_entry_state(), plan.exit_state())?;
            }
            Some((plan.entry_state(), plan.exit_state()))
        }
        StatementIr::EmptyStatementCompletion(item) => span(item.statement())?,
        StatementIr::Block(block) => block_span(block)?,
        StatementIr::LexicalBlock(statements)
        | StatementIr::ParameterInitialization { statements, .. } => sequence_span(statements)?,
        StatementIr::Labelled {
            statement,
            async_plan,
            ..
        } => match async_plan {
            Some(plan) => {
                let Some((start, end)) = span(statement)? else {
                    return Err(AsyncSwitchError::MissingChildOwner);
                };
                if start != plan.entry_state() || end != plan.body_exit_state() {
                    return Err(AsyncSwitchError::StateOrder);
                }
                Some((plan.entry_state(), plan.exit_state()))
            }
            None => {
                if span(statement)?.is_some() {
                    return Err(AsyncSwitchError::MissingChildOwner);
                }
                None
            }
        },
        StatementIr::SyncDisposableScope {
            execution, body, ..
        } => match execution {
            SyncDisposableScopeExecutionIr::Immediate => {
                if block_span(body)?.is_some() {
                    return Err(AsyncSwitchError::MissingChildOwner);
                }
                None
            }
            SyncDisposableScopeExecutionIr::AsyncFunction(_) => block_span(body)?,
            SyncDisposableScopeExecutionIr::PlainGenerator(_)
            | SyncDisposableScopeExecutionIr::AsyncGenerator(_) => {
                return Err(AsyncSwitchError::UnsupportedChildOwner)
            }
        },
        StatementIr::AsyncDisposableScope {
            execution, body, ..
        } => match execution {
            AsyncDisposableScopeExecutionIr::AsyncFunction(capability) => {
                let finalizer = capability.finalizer();
                let body_exit = sequence_exit(&body.statements, finalizer.entry_state())?;
                if body_exit.checked_add(1) != Some(finalizer.dispose_state())
                    || finalizer.dispose_state().checked_add(1) != Some(finalizer.resume_state())
                    || finalizer.resume_state().checked_add(1) != Some(finalizer.exit_state())
                {
                    return Err(AsyncSwitchError::StateOrder);
                }
                Some((finalizer.entry_state(), finalizer.exit_state()))
            }
            AsyncDisposableScopeExecutionIr::AsyncGenerator(_) => {
                return Err(AsyncSwitchError::UnsupportedChildOwner)
            }
        },
        StatementIr::TryCatch {
            try_block,
            catch_block,
            generator_plan,
            async_plan,
            ..
        } => {
            if generator_plan.is_some() {
                return Err(AsyncSwitchError::UnsupportedChildOwner);
            }
            match async_plan {
                Some(plan) => Some(try_span(plan, try_block, Some(catch_block), None)?),
                None => {
                    if block_span(try_block)?.is_some() || block_span(catch_block)?.is_some() {
                        return Err(AsyncSwitchError::MissingChildOwner);
                    }
                    None
                }
            }
        }
        StatementIr::TryFinally {
            try_block,
            finally_block,
            generator_plan,
            async_plan,
        } => {
            if generator_plan.is_some() {
                return Err(AsyncSwitchError::UnsupportedChildOwner);
            }
            match async_plan {
                Some(plan) => Some(try_span(plan, try_block, None, Some(finally_block))?),
                None => {
                    if block_span(try_block)?.is_some() || block_span(finally_block)?.is_some() {
                        return Err(AsyncSwitchError::MissingChildOwner);
                    }
                    None
                }
            }
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
                return Err(AsyncSwitchError::UnsupportedChildOwner);
            }
            match async_plan {
                Some(plan) => Some(try_span(
                    plan,
                    try_block,
                    Some(catch_block),
                    Some(finally_block),
                )?),
                None => {
                    if block_span(try_block)?.is_some()
                        || block_span(catch_block)?.is_some()
                        || block_span(finally_block)?.is_some()
                    {
                        return Err(AsyncSwitchError::MissingChildOwner);
                    }
                    None
                }
            }
        }
        StatementIr::If {
            then_branch,
            else_branch,
            ..
        } => {
            if span(then_branch)?.is_some()
                || else_branch
                    .as_deref()
                    .map(span)
                    .transpose()?
                    .flatten()
                    .is_some()
            {
                return Err(AsyncSwitchError::MissingChildOwner);
            }
            None
        }
        StatementIr::For { init, body, .. } => {
            match init {
                Some(crate::ForInitIr::Statements(statements)) => {
                    if sequence_span(statements)?.is_some() {
                        return Err(AsyncSwitchError::UnsupportedChildOwner);
                    }
                }
                Some(crate::ForInitIr::AsyncDisposable(_)) => {
                    return Err(AsyncSwitchError::UnsupportedChildOwner)
                }
                Some(
                    crate::ForInitIr::Lexical { .. }
                    | crate::ForInitIr::LexicalBlock(_)
                    | crate::ForInitIr::Var(_)
                    | crate::ForInitIr::Expression(_)
                    | crate::ForInitIr::SyncDisposable(_),
                )
                | None => {}
            }
            if span(body)?.is_some() {
                return Err(AsyncSwitchError::UnsupportedChildOwner);
            }
            None
        }
        StatementIr::ForOfIterator { head, body, .. } => {
            match head {
                crate::ForOfIteratorHeadIr::Assignment {
                    async_plan: Some(_),
                    ..
                }
                | crate::ForOfIteratorHeadIr::AsyncDisposable(_) => {
                    return Err(AsyncSwitchError::UnsupportedChildOwner)
                }
                crate::ForOfIteratorHeadIr::Assignment {
                    async_plan: None, ..
                }
                | crate::ForOfIteratorHeadIr::SyncDisposable(_) => {}
            }
            if span(body)?.is_some() {
                return Err(AsyncSwitchError::UnsupportedChildOwner);
            }
            None
        }
        StatementIr::While { body, .. }
        | StatementIr::DoWhile { body, .. }
        | StatementIr::ForInArray { body, .. }
        | StatementIr::ForInString { body, .. }
        | StatementIr::ForInObject { body, .. } => {
            if span(body)?.is_some() {
                return Err(AsyncSwitchError::UnsupportedChildOwner);
            }
            None
        }
        StatementIr::Switch {
            lexical_declarations,
            cases,
            ..
        } => {
            if sequence_span(lexical_declarations)?.is_some() {
                return Err(AsyncSwitchError::MissingChildOwner);
            }
            for case in cases {
                if block_span(&case.body)?.is_some() {
                    return Err(AsyncSwitchError::MissingChildOwner);
                }
            }
            None
        }
        StatementIr::GeneratorYield { .. }
        | StatementIr::GeneratorLoop { .. }
        | StatementIr::AsyncGeneratorIf(_)
        | StatementIr::AsyncGeneratorWith(_)
        | StatementIr::AsyncGeneratorArrayDestructuring(_)
        | StatementIr::AsyncGeneratorResourceRegistration(_)
        | StatementIr::OrdinaryGeneratorLoop(_)
        | StatementIr::OrdinaryGeneratorIf(_)
        | StatementIr::OrdinaryGeneratorSwitch(_)
        | StatementIr::OrdinaryGeneratorArrayDestructuring(_)
        | StatementIr::OrdinaryGeneratorWith(_)
        | StatementIr::ArrayDestructuringOperation(_)
        | StatementIr::GeneratorIf { .. }
        | StatementIr::GeneratorForOfIterator { .. }
        | StatementIr::AsyncFunctionForOfIterator { .. }
        | StatementIr::AsyncModuleInstantiation
        | StatementIr::ModuleUnitOnce { .. } => {
            return Err(AsyncSwitchError::UnsupportedChildOwner)
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
        | StatementIr::Continue { .. } => None,
    };
    Ok(owned)
}

#[cfg(test)]
mod tests;
