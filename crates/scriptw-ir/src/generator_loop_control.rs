use crate::{
    BlockIr, ForLexicalEnvironmentIr, GeneratorSuspensionPointIr, GeneratorTryPlanIr,
    OwnedEnvBindingIr, StatementIr, TypedExpr,
};

mod statement_states;
pub(crate) use statement_states::{
    collect_mixed_suspensions, collect_suspensions, mixed_sequence_end, sequence_end,
};
mod head_bindings;
pub(crate) use head_bindings::visit_lexical_head_statement_bindings;

/// The three classic iteration algorithms. Iterator acquisition and close have
/// a separate `GeneratorForOfIteratorPlanIr` owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratorLoopKindIr {
    For,
    While,
    DoWhile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GeneratorLoopControlError {
    StateOverflow,
    StateMismatch { expected: u32, actual: u32 },
    InvalidPhases,
    InvalidSuspension,
    ForeignContinuation,
    UnconsumedSourceSuspension,
}

pub(crate) fn checked_next_state(state: u32) -> Result<u32, GeneratorLoopControlError> {
    state
        .checked_add(1)
        .ok_or(GeneratorLoopControlError::StateOverflow)
}

fn require_state(expected: u32, actual: u32) -> Result<(), GeneratorLoopControlError> {
    if expected == actual {
        Ok(())
    } else {
        Err(GeneratorLoopControlError::StateMismatch { expected, actual })
    }
}

/// A source-owned inclusive range. Every phase has a distinct entry, including
/// an empty phase; a resumed terminal Yield is therefore still inside its owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GeneratorLoopSourceRange {
    pub(crate) entry: u32,
    pub(crate) end: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct GeneratorLoopSourceStates {
    pub(crate) kind: GeneratorLoopKindIr,
    pub(crate) initialization: Option<GeneratorLoopSourceRange>,
    pub(crate) test: GeneratorLoopSourceRange,
    pub(crate) body: GeneratorLoopSourceRange,
    pub(crate) update: Option<GeneratorLoopSourceRange>,
    pub(crate) exit: u32,
    pub(crate) suspensions: Vec<GeneratorSuspensionPointIr>,
}

/// Complete lowered statements admitted against their consumed source range.
/// Backend crates can borrow a region, but cannot fabricate one from state words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratorLoopRegionIr {
    block: BlockIr,
    entry_state: u32,
    end_state: u32,
}

impl GeneratorLoopRegionIr {
    pub(crate) fn new(
        block: BlockIr,
        range: GeneratorLoopSourceRange,
    ) -> Result<Self, GeneratorLoopControlError> {
        require_state(range.end, sequence_end(&block.statements, range.entry)?)?;
        Ok(Self {
            block,
            entry_state: range.entry,
            end_state: range.end,
        })
    }
    pub fn block(&self) -> &BlockIr {
        &self.block
    }
    pub(crate) fn into_block(self) -> BlockIr {
        self.block
    }
    pub const fn entry_state(&self) -> u32 {
        self.entry_state
    }
    pub const fn end_state(&self) -> u32 {
        self.end_state
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratorLoopExpressionIr {
    region: GeneratorLoopRegionIr,
    value: TypedExpr,
}

impl GeneratorLoopExpressionIr {
    pub(crate) fn new(region: GeneratorLoopRegionIr, value: TypedExpr) -> Self {
        Self { region, value }
    }
    pub fn region(&self) -> &GeneratorLoopRegionIr {
        &self.region
    }
    pub fn value(&self) -> &TypedExpr {
        &self.value
    }
}

/// A consumed ordinary-generator control graph. For uses Initialize -> Test ->
/// Body -> Update -> Test; While uses Test -> Body -> Test; DoWhile enters Body.
/// Its exact source suspensions and every clause/branch/loop edge are checked
/// before publication. Async execution cannot inhabit this statement variant.
#[must_use = "the checked classic loop must be attached to its statement"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrdinaryGeneratorLoopIr {
    kind: GeneratorLoopKindIr,
    initialization: Option<GeneratorLoopRegionIr>,
    test: GeneratorLoopExpressionIr,
    body: GeneratorLoopRegionIr,
    update: Option<GeneratorLoopExpressionIr>,
    lexical_environment: Option<ForLexicalEnvironmentIr>,
    value_binding: OwnedEnvBindingIr,
    exit_state: u32,
}

impl OrdinaryGeneratorLoopIr {
    pub(crate) fn new(
        source: GeneratorLoopSourceStates,
        initialization: Option<GeneratorLoopRegionIr>,
        test: GeneratorLoopExpressionIr,
        body: GeneratorLoopRegionIr,
        update: Option<GeneratorLoopExpressionIr>,
        lexical_environment: Option<ForLexicalEnvironmentIr>,
        value_binding: OwnedEnvBindingIr,
    ) -> Result<Self, GeneratorLoopControlError> {
        let plan = Self {
            kind: source.kind,
            initialization,
            test,
            body,
            update,
            lexical_environment,
            value_binding,
            exit_state: source.exit,
        };
        if plan.initialization.as_ref().map(range_of) != source.initialization
            || range_of(plan.test.region()) != source.test
            || range_of(&plan.body) != source.body
            || plan
                .update
                .as_ref()
                .map(|expression| range_of(expression.region()))
                != source.update
        {
            return Err(GeneratorLoopControlError::InvalidPhases);
        }
        match plan.kind {
            GeneratorLoopKindIr::For if plan.initialization.is_some() && plan.update.is_some() => {}
            GeneratorLoopKindIr::While | GeneratorLoopKindIr::DoWhile
                if plan.initialization.is_none()
                    && plan.update.is_none()
                    && plan.lexical_environment.is_none() => {}
            _ => return Err(GeneratorLoopControlError::InvalidPhases),
        }
        let mut previous_end = None;
        let mut actual_suspensions = Vec::new();
        for region in plan.regions() {
            if let Some(end) = previous_end {
                require_state(checked_next_state(end)?, region.entry_state())?;
            }
            previous_end = Some(region.end_state());
            collect_suspensions(&region.block.statements, &mut actual_suspensions);
        }
        require_state(
            checked_next_state(previous_end.ok_or(GeneratorLoopControlError::InvalidPhases)?)?,
            plan.exit_state,
        )?;
        if actual_suspensions != source.suspensions {
            return Err(GeneratorLoopControlError::UnconsumedSourceSuspension);
        }
        let resumed: std::collections::BTreeSet<_> = actual_suspensions
            .iter()
            .map(|point| point.resume_state)
            .collect();
        if resumed.len() != actual_suspensions.len() {
            return Err(GeneratorLoopControlError::InvalidSuspension);
        }
        for head in plan
            .initialization
            .iter()
            .chain(std::iter::once(plan.test.region()))
            .chain(plan.update.iter().map(GeneratorLoopExpressionIr::region))
        {
            if head.block.statements.iter().any(has_unowned_head_branch) {
                return Err(GeneratorLoopControlError::ForeignContinuation);
            }
        }
        if let Some(environment) = &plan.lexical_environment {
            validate_classic_loop_environment(
                &plan
                    .initialization
                    .as_ref()
                    .expect("For lexical owner has initialization")
                    .block
                    .statements,
                environment,
                &plan.value_binding,
            )?;
        }
        Ok(plan)
    }
    pub const fn kind(&self) -> GeneratorLoopKindIr {
        self.kind
    }
    pub fn initialization(&self) -> Option<&GeneratorLoopRegionIr> {
        self.initialization.as_ref()
    }
    pub fn test(&self) -> &GeneratorLoopExpressionIr {
        &self.test
    }
    pub fn body(&self) -> &GeneratorLoopRegionIr {
        &self.body
    }
    pub fn update(&self) -> Option<&GeneratorLoopExpressionIr> {
        self.update.as_ref()
    }
    pub fn lexical_environment(&self) -> Option<&ForLexicalEnvironmentIr> {
        self.lexical_environment.as_ref()
    }
    pub fn value_binding(&self) -> &OwnedEnvBindingIr {
        &self.value_binding
    }
    pub fn value_binding_name(&self) -> &str {
        &self.value_binding.name
    }
    pub fn entry_state(&self) -> u32 {
        match self.kind {
            GeneratorLoopKindIr::For => self
                .initialization
                .as_ref()
                .expect("checked For initialization")
                .entry_state(),
            GeneratorLoopKindIr::While => self.test.region.entry_state(),
            GeneratorLoopKindIr::DoWhile => self.body.entry_state(),
        }
    }
    pub const fn exit_state(&self) -> u32 {
        self.exit_state
    }
    pub fn continue_state(&self) -> u32 {
        self.update
            .as_ref()
            .map_or(self.test.region.entry_state(), |update| {
                update.region.entry_state()
            })
    }
    /// Source/evaluation order; DoWhile's body precedes its test.
    pub fn regions(&self) -> impl Iterator<Item = &GeneratorLoopRegionIr> {
        let (first, second) = match self.kind {
            GeneratorLoopKindIr::DoWhile => (&self.body, self.test.region()),
            GeneratorLoopKindIr::For | GeneratorLoopKindIr::While => {
                (self.test.region(), &self.body)
            }
        };
        self.initialization
            .iter()
            .chain([first, second])
            .chain(self.update.iter().map(GeneratorLoopExpressionIr::region))
    }
    pub fn expressions(&self) -> impl Iterator<Item = &TypedExpr> {
        std::iter::once(self.test.value())
            .chain(self.update.iter().map(GeneratorLoopExpressionIr::value))
    }
}

#[must_use = "the checked conditional must be attached to its statement"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrdinaryGeneratorIfIr {
    condition: TypedExpr,
    then_branch: GeneratorLoopRegionIr,
    else_branch: GeneratorLoopRegionIr,
    entry_state: u32,
    exit_state: u32,
}

impl OrdinaryGeneratorIfIr {
    pub(crate) fn new(
        condition: TypedExpr,
        entry_state: u32,
        then_branch: GeneratorLoopRegionIr,
        else_branch: GeneratorLoopRegionIr,
        exit_state: u32,
    ) -> Result<Self, GeneratorLoopControlError> {
        require_state(checked_next_state(entry_state)?, then_branch.entry_state())?;
        require_state(
            checked_next_state(then_branch.end_state())?,
            else_branch.entry_state(),
        )?;
        require_state(checked_next_state(else_branch.end_state())?, exit_state)?;
        Ok(Self {
            condition,
            then_branch,
            else_branch,
            entry_state,
            exit_state,
        })
    }
    pub fn condition(&self) -> &TypedExpr {
        &self.condition
    }
    pub fn then_branch(&self) -> &GeneratorLoopRegionIr {
        &self.then_branch
    }
    pub fn else_branch(&self) -> &GeneratorLoopRegionIr {
        &self.else_branch
    }
    pub const fn entry_state(&self) -> u32 {
        self.entry_state
    }
    pub const fn exit_state(&self) -> u32 {
        self.exit_state
    }
}

fn range_of(region: &GeneratorLoopRegionIr) -> GeneratorLoopSourceRange {
    GeneratorLoopSourceRange {
        entry: region.entry_state(),
        end: region.end_state(),
    }
}

pub(crate) fn has_unowned_head_branch(statement: &StatementIr) -> bool {
    head_branch(statement, false)
}

fn head_branch(statement: &StatementIr, switch_owns_break: bool) -> bool {
    let unowned = |statement: &StatementIr| head_branch(statement, switch_owns_break);
    match statement {
        StatementIr::EmptyStatementCompletion(item) => unowned(item.statement()),
        StatementIr::Break { label } => label.is_some() || !switch_owns_break,
        StatementIr::Continue { .. } => true,
        StatementIr::Block(block) => block.statements.iter().any(unowned),
        StatementIr::LexicalBlock(statements)
        | StatementIr::ParameterInitialization { statements, .. } => statements.iter().any(unowned),
        StatementIr::If {
            then_branch,
            else_branch,
            ..
        } => unowned(then_branch) || else_branch.as_deref().is_some_and(unowned),
        StatementIr::AsyncGeneratorIf(plan) => plan
            .regions()
            .flat_map(|region| &region.block().statements)
            .any(unowned),
        StatementIr::OrdinaryGeneratorIf(plan) => plan
            .then_branch
            .block
            .statements
            .iter()
            .chain(&plan.else_branch.block.statements)
            .any(unowned),
        StatementIr::GeneratorIf {
            then_before_yield,
            then_yield_statement,
            then_after_yield,
            else_before_yield,
            else_yield_statement,
            else_after_yield,
            ..
        } => then_before_yield
            .iter()
            .chain(then_yield_statement.as_deref())
            .chain(then_after_yield)
            .chain(else_before_yield)
            .chain(else_yield_statement.as_deref())
            .chain(else_after_yield)
            .any(unowned),
        StatementIr::TryCatch {
            try_block,
            catch_block,
            ..
        } => try_block
            .statements
            .iter()
            .chain(&catch_block.statements)
            .any(unowned),
        StatementIr::TryFinally {
            try_block,
            finally_block,
            ..
        } => try_block
            .statements
            .iter()
            .chain(&finally_block.statements)
            .any(unowned),
        StatementIr::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
            ..
        } => try_block
            .statements
            .iter()
            .chain(&catch_block.statements)
            .chain(&finally_block.statements)
            .any(unowned),
        StatementIr::Labelled { statement, .. } => unowned(statement),
        StatementIr::Switch {
            lexical_declarations,
            cases,
            ..
        } => {
            lexical_declarations
                .iter()
                .any(|statement| head_branch(statement, false))
                || cases.iter().any(|case| {
                    case.body
                        .statements
                        .iter()
                        .any(|statement| head_branch(statement, true))
                })
        }
        StatementIr::OrdinaryGeneratorArrayDestructuring(plan) => {
            plan.body().block().statements.iter().any(unowned)
        }
        StatementIr::AsyncGeneratorResourceScope(plan) => {
            plan.body().block().statements.iter().any(unowned)
        }
        StatementIr::AsyncGeneratorResourceRegistration(_) => false,
        StatementIr::AsyncGeneratorArrayDestructuring(plan) => {
            plan.body().block().statements.iter().any(unowned)
        }
        StatementIr::OrdinaryGeneratorWith(plan) => {
            plan.head().region().block().statements.iter().any(unowned)
                || plan.body().block().statements.iter().any(unowned)
        }
        StatementIr::AsyncGeneratorWith(plan) => {
            plan.head().region().block().statements.iter().any(unowned)
                || plan.body().block().statements.iter().any(unowned)
        }
        StatementIr::OrdinaryGeneratorSwitch(plan) => {
            plan.discriminant()
                .region()
                .block()
                .statements
                .iter()
                .any(|statement| head_branch(statement, false))
                || plan
                    .lexical_declarations()
                    .iter()
                    .any(|statement| head_branch(statement, false))
                || plan.cases().iter().any(|case| {
                    case.selector().is_some_and(|selector| {
                        selector
                            .region()
                            .block()
                            .statements
                            .iter()
                            .any(|statement| head_branch(statement, false))
                    }) || case
                        .body()
                        .block()
                        .statements
                        .iter()
                        .any(|statement| head_branch(statement, true))
                })
        }
        StatementIr::AsyncGeneratorSwitch(plan) => {
            plan.discriminant()
                .region()
                .block()
                .statements
                .iter()
                .any(|statement| head_branch(statement, false))
                || plan
                    .lexical_declarations()
                    .iter()
                    .any(|statement| head_branch(statement, false))
                || plan.cases().iter().any(|case| {
                    case.selector().is_some_and(|selector| {
                        selector
                            .region()
                            .block()
                            .statements
                            .iter()
                            .any(|statement| head_branch(statement, false))
                    }) || case
                        .body()
                        .block()
                        .statements
                        .iter()
                        .any(|statement| head_branch(statement, true))
                })
        }
        // A nested iteration owns its own branches. Function bodies are not
        // part of the enclosing expression's continuation statement tree.
        StatementIr::AsyncGeneratorLoop(_)
        | StatementIr::OrdinaryGeneratorLoop(_)
        | StatementIr::AsyncGeneratorForOf(_)
        | StatementIr::AsyncGeneratorForIn(_)
        | StatementIr::While { .. }
        | StatementIr::DoWhile { .. }
        | StatementIr::For { .. }
        | StatementIr::ForOfIterator { .. }
        | StatementIr::ForInArray { .. }
        | StatementIr::ForInString { .. }
        | StatementIr::ForInObject { .. }
        | StatementIr::Empty
        | StatementIr::ModuleImportBinding(_)
        | StatementIr::Lexical { .. }
        | StatementIr::AnnexBFunctionCopy { .. }
        | StatementIr::Var(_)
        | StatementIr::DeclarationEvaluation(_)
        | StatementIr::ArrayDestructuringOperation(_)
        | StatementIr::Expression(_)
        | StatementIr::Debugger
        | StatementIr::Throw(_)
        | StatementIr::Return(_)
        | StatementIr::GeneratorYield { .. }
        | StatementIr::ResumableClassDefinition(_)
        | StatementIr::ModuleUnitOnce { .. }
        | StatementIr::SyncDisposableScope { .. }
        | StatementIr::AsyncDisposableScope { .. }
        | StatementIr::AsyncAwait { .. }
        | StatementIr::AsyncModuleInstantiation
        | StatementIr::GeneratorLoop { .. }
        | StatementIr::AsyncFunctionIf { .. }
        | StatementIr::AsyncFunctionArrayDestructuring(_)
        | StatementIr::AsyncFunctionWith(_)
        | StatementIr::AsyncFunctionWhile(_)
        | StatementIr::AsyncFunctionSwitch(_)
        | StatementIr::AsyncFunctionForOfIterator { .. }
        | StatementIr::GeneratorForOfIterator { .. } => false,
    }
}

pub(crate) fn validate_classic_loop_environment(
    initialization: &[StatementIr],
    environment: &ForLexicalEnvironmentIr,
    value_binding: &OwnedEnvBindingIr,
) -> Result<(), GeneratorLoopControlError> {
    let slots: std::collections::BTreeSet<_> = environment
        .bindings
        .iter()
        .map(|binding| binding.slot)
        .collect();
    let per_iteration: std::collections::BTreeSet<_> =
        environment.per_iteration_slots.iter().copied().collect();
    let names: std::collections::BTreeSet<_> = environment
        .bindings
        .iter()
        .map(|binding| binding.name.as_str())
        .collect();
    let mut mutable_head_names = std::collections::BTreeSet::new();
    visit_lexical_head_statement_bindings(initialization, &mut |mode, name| {
        if mode == crate::BindingMode::Let {
            mutable_head_names.insert(name.to_string());
        }
    });
    let expected_per_iteration: std::collections::BTreeSet<_> = environment
        .bindings
        .iter()
        .filter(|binding| mutable_head_names.contains(binding.name.as_str()))
        .map(|binding| binding.slot)
        .collect();
    if slots.len() != environment.bindings.len()
        || names.len() != environment.bindings.len()
        || per_iteration.len() != environment.per_iteration_slots.len()
        || !per_iteration.is_subset(&slots)
        || per_iteration != expected_per_iteration
        || names.contains(value_binding.name.as_str())
    {
        return Err(GeneratorLoopControlError::InvalidPhases);
    }
    Ok(())
}
