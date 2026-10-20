use super::*;
use crate::{
    AsyncTryPlanIr, ResumableResumeEnvironmentIr, ResumableSuspensionKindIr,
    ResumableSuspensionPointIr,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum StatementProtocol {
    Generator,
    AsyncGenerator,
}

pub(crate) fn sequence_end(
    statements: &[StatementIr],
    state: u32,
) -> Result<u32, GeneratorLoopControlError> {
    sequence_end_for(statements, state, StatementProtocol::Generator)
}
pub(crate) fn mixed_sequence_end(
    statements: &[StatementIr],
    state: u32,
) -> Result<u32, GeneratorLoopControlError> {
    sequence_end_for(statements, state, StatementProtocol::AsyncGenerator)
}

enum SuspensionCensus<'a> {
    Generator(&'a mut Vec<GeneratorSuspensionPointIr>),
    Mixed(&'a mut Vec<ResumableSuspensionPointIr>),
}
impl SuspensionCensus<'_> {
    fn push_yield(&mut self, suspend_state: u32, resume_state: u32) {
        match self {
            Self::Generator(points) => points.push(GeneratorSuspensionPointIr {
                suspend_state,
                resume_state,
            }),
            Self::Mixed(points) => points.push(ResumableSuspensionPointIr {
                kind: ResumableSuspensionKindIr::Yield,
                suspend_state,
                resume_state,
                resume_environment: ResumableResumeEnvironmentIr::InvocationOuter,
            }),
        }
    }
    fn push_await(&mut self, suspend_state: u32, resume_state: u32) {
        match self {
            Self::Generator(_) => {}
            Self::Mixed(points) => points.push(ResumableSuspensionPointIr {
                kind: ResumableSuspensionKindIr::Await,
                suspend_state,
                resume_state,
                resume_environment: ResumableResumeEnvironmentIr::InvocationOuter,
            }),
        }
    }
    fn extend_mixed(&mut self, tape: &[ResumableSuspensionPointIr]) {
        match self {
            Self::Generator(_) => {}
            Self::Mixed(points) => points.extend_from_slice(tape),
        }
    }
    fn extend_checked_protocol(
        &mut self,
        protocol: crate::ResumableRegionProtocolIr,
        tape: &[ResumableSuspensionPointIr],
    ) {
        match self {
            Self::Generator(points) => match protocol {
                crate::ResumableRegionProtocolIr::Generator => {
                    // The opaque owner's source/region constructors validate
                    // its protocol before this projection. An Await cannot be
                    // erased into an ordinary-generator suspension census.
                    points.extend(tape.iter().map(|point| match point.kind {
                        ResumableSuspensionKindIr::Yield => GeneratorSuspensionPointIr {
                            suspend_state: point.suspend_state,
                            resume_state: point.resume_state,
                        },
                        ResumableSuspensionKindIr::Await
                        | ResumableSuspensionKindIr::ForAwaitNext
                        | ResumableSuspensionKindIr::ForAwaitClose => {
                            unreachable!("checked Generator owner cannot contain Await points")
                        }
                    }));
                }
                crate::ResumableRegionProtocolIr::Async
                | crate::ResumableRegionProtocolIr::AsyncGenerator => {}
            },
            Self::Mixed(points) => points.extend_from_slice(tape),
        }
    }
}
pub(crate) fn collect_suspensions(
    statements: &[StatementIr],
    points: &mut Vec<GeneratorSuspensionPointIr>,
) {
    collect_suspensions_for(statements, &mut SuspensionCensus::Generator(points));
}
pub(crate) fn collect_mixed_suspensions(
    statements: &[StatementIr],
    points: &mut Vec<ResumableSuspensionPointIr>,
) {
    collect_suspensions_for(statements, &mut SuspensionCensus::Mixed(points));
}

fn sequence_end_for(
    statements: &[StatementIr],
    mut state: u32,
    protocol: StatementProtocol,
) -> Result<u32, GeneratorLoopControlError> {
    for statement in statements {
        state = statement_end_for(statement, state, protocol)?;
    }
    Ok(state)
}

fn eager_for(
    statement: &StatementIr,
    state: u32,
    protocol: StatementProtocol,
) -> Result<(), GeneratorLoopControlError> {
    require_state(state, statement_end_for(statement, state, protocol)?)
}

fn try_end_for(
    try_block: &BlockIr,
    catch_block: Option<&BlockIr>,
    finally_block: Option<&BlockIr>,
    plan: Option<GeneratorTryPlanIr>,
    async_plan: Option<AsyncTryPlanIr>,
    state: u32,
    protocol: StatementProtocol,
) -> Result<u32, GeneratorLoopControlError> {
    let plan = plan.ok_or(GeneratorLoopControlError::ForeignContinuation)?;
    match protocol {
        StatementProtocol::Generator => {
            if async_plan.is_some() {
                return Err(GeneratorLoopControlError::ForeignContinuation);
            }
        }
        StatementProtocol::AsyncGenerator => {
            let expected = AsyncTryPlanIr {
                entry_state: plan.entry_state,
                try_exit_state: plan.try_exit_state,
                catch_entry_state: plan.catch_entry_state,
                catch_exit_state: plan.catch_exit_state,
                finally_entry_state: plan.finally_entry_state,
                finally_exit_state: plan.finally_exit_state,
                exit_state: plan.exit_state,
            };
            if async_plan != Some(expected) {
                return Err(GeneratorLoopControlError::ForeignContinuation);
            }
        }
    }
    require_state(state, plan.entry_state)?;
    require_state(
        checked_next_state(sequence_end_for(&try_block.statements, state, protocol)?)?,
        plan.try_exit_state,
    )?;
    let mut state = plan.try_exit_state;
    for (block, entry, exit) in [
        (catch_block, plan.catch_entry_state, plan.catch_exit_state),
        (
            finally_block,
            plan.finally_entry_state,
            plan.finally_exit_state,
        ),
    ] {
        match (block, entry, exit) {
            (Some(block), Some(entry), Some(exit)) => {
                require_state(state, entry)?;
                require_state(
                    checked_next_state(sequence_end_for(&block.statements, entry, protocol)?)?,
                    exit,
                )?;
                state = exit;
            }
            (None, None, None) => {}
            _ => return Err(GeneratorLoopControlError::ForeignContinuation),
        }
    }
    require_state(state, plan.exit_state)?;
    Ok(state)
}

fn statement_end_for(
    statement: &StatementIr,
    state: u32,
    protocol: StatementProtocol,
) -> Result<u32, GeneratorLoopControlError> {
    match statement {
        StatementIr::EmptyStatementCompletion(item) => {
            statement_end_for(item.statement(), state, protocol)
        }
        StatementIr::GeneratorYield {
            suspend_state,
            resume_state,
            ..
        } => {
            require_state(state, *suspend_state)?;
            require_state(checked_next_state(state)?, *resume_state)?;
            Ok(*resume_state)
        }
        StatementIr::AsyncAwait {
            suspend_state,
            resume_state,
            ..
        } => {
            if protocol != StatementProtocol::AsyncGenerator {
                return Err(GeneratorLoopControlError::ForeignContinuation);
            }
            require_state(state, *suspend_state)?;
            require_state(checked_next_state(state)?, *resume_state)?;
            Ok(*resume_state)
        }
        StatementIr::AsyncGeneratorLoop(plan) => {
            let expected = match protocol {
                StatementProtocol::Generator => crate::ResumableRegionProtocolIr::Generator,
                StatementProtocol::AsyncGenerator => {
                    crate::ResumableRegionProtocolIr::AsyncGenerator
                }
            };
            if plan.execution() != expected {
                return Err(GeneratorLoopControlError::ForeignContinuation);
            }
            require_state(state, plan.entry_state())?;
            Ok(plan.exit_state())
        }
        StatementIr::AsyncGeneratorIf(plan) => {
            if protocol != StatementProtocol::AsyncGenerator {
                return Err(GeneratorLoopControlError::ForeignContinuation);
            }
            require_state(state, plan.entry_state())?;
            Ok(plan.exit_state())
        }
        StatementIr::AsyncGeneratorWith(plan) => {
            if protocol != StatementProtocol::AsyncGenerator {
                return Err(GeneratorLoopControlError::ForeignContinuation);
            }
            require_state(state, plan.entry_state())?;
            Ok(plan.exit_state())
        }
        StatementIr::AsyncGeneratorSwitch(plan) => {
            let expected = match protocol {
                StatementProtocol::Generator => crate::ResumableRegionProtocolIr::Generator,
                StatementProtocol::AsyncGenerator => {
                    crate::ResumableRegionProtocolIr::AsyncGenerator
                }
            };
            if plan.execution() != expected {
                return Err(GeneratorLoopControlError::ForeignContinuation);
            }
            require_state(state, plan.entry_state())?;
            Ok(plan.exit_state())
        }
        StatementIr::AsyncGeneratorForIn(plan) => {
            let expected = match protocol {
                StatementProtocol::Generator => crate::ResumableRegionProtocolIr::Generator,
                StatementProtocol::AsyncGenerator => {
                    crate::ResumableRegionProtocolIr::AsyncGenerator
                }
            };
            if plan.execution() != expected {
                return Err(GeneratorLoopControlError::ForeignContinuation);
            }
            require_state(state, plan.entry_state())?;
            Ok(plan.exit_state())
        }
        StatementIr::AsyncGeneratorForOf(plan) => {
            let expected = match protocol {
                StatementProtocol::Generator => crate::ResumableRegionProtocolIr::Generator,
                StatementProtocol::AsyncGenerator => {
                    crate::ResumableRegionProtocolIr::AsyncGenerator
                }
            };
            if plan.execution() != expected {
                return Err(GeneratorLoopControlError::ForeignContinuation);
            }
            require_state(state, plan.entry_state())?;
            Ok(plan.exit_state())
        }
        StatementIr::AsyncGeneratorResourceScope(plan) => {
            let expected = match protocol {
                StatementProtocol::Generator => crate::ResumableRegionProtocolIr::Generator,
                StatementProtocol::AsyncGenerator => {
                    crate::ResumableRegionProtocolIr::AsyncGenerator
                }
            };
            if plan.execution() != expected {
                return Err(GeneratorLoopControlError::ForeignContinuation);
            }
            require_state(state, plan.entry_state())?;
            Ok(plan.exit_state())
        }
        StatementIr::AsyncGeneratorResourceRegistration(_) => {
            Err(GeneratorLoopControlError::ForeignContinuation)
        }
        StatementIr::AsyncGeneratorArrayDestructuring(plan) => {
            if protocol != StatementProtocol::AsyncGenerator {
                return Err(GeneratorLoopControlError::ForeignContinuation);
            }
            require_state(state, plan.entry_state())?;
            Ok(plan.exit_state())
        }
        StatementIr::OrdinaryGeneratorLoop(plan) => {
            if protocol != StatementProtocol::Generator {
                return Err(GeneratorLoopControlError::ForeignContinuation);
            }
            require_state(state, plan.entry_state())?;
            Ok(plan.exit_state())
        }
        StatementIr::OrdinaryGeneratorIf(plan) => {
            if protocol != StatementProtocol::Generator {
                return Err(GeneratorLoopControlError::ForeignContinuation);
            }
            require_state(state, plan.entry_state())?;
            Ok(plan.exit_state())
        }
        StatementIr::OrdinaryGeneratorSwitch(plan) => {
            if protocol != StatementProtocol::Generator {
                return Err(GeneratorLoopControlError::ForeignContinuation);
            }
            require_state(state, plan.entry_state())?;
            Ok(plan.exit_state())
        }
        StatementIr::OrdinaryGeneratorArrayDestructuring(plan) => {
            if protocol != StatementProtocol::Generator {
                return Err(GeneratorLoopControlError::ForeignContinuation);
            }
            require_state(state, plan.entry_state())?;
            Ok(plan.exit_state())
        }
        StatementIr::OrdinaryGeneratorWith(plan) => {
            if protocol != StatementProtocol::Generator {
                return Err(GeneratorLoopControlError::ForeignContinuation);
            }
            require_state(state, plan.entry_state())?;
            Ok(plan.exit_state())
        }
        StatementIr::ResumableClassDefinition(plan) => {
            require_state(state, plan.entry_state())?;
            for prefix in plan.prefixes() {
                require_state(
                    prefix.exit_state(),
                    sequence_end_for(prefix.statements(), prefix.entry_state(), protocol)?,
                )?;
            }
            Ok(plan.exit_state())
        }
        StatementIr::Block(block) => sequence_end_for(&block.statements, state, protocol),
        StatementIr::LexicalBlock(statements) => sequence_end_for(statements, state, protocol),
        StatementIr::GeneratorIf {
            entry_state,
            exit_state,
            then_before_yield,
            then_yield_statement,
            then_after_yield,
            else_before_yield,
            else_yield_statement,
            else_after_yield,
            then_resume_state,
            else_resume_state,
            ..
        } => {
            if protocol != StatementProtocol::Generator {
                return Err(GeneratorLoopControlError::ForeignContinuation);
            }
            require_state(state, *entry_state)?;
            if then_resume_state.is_none() && else_resume_state.is_none() {
                return Err(GeneratorLoopControlError::InvalidSuspension);
            }
            let mut last = state;
            for (before, yielded, after, resumed) in [
                (
                    then_before_yield,
                    then_yield_statement.as_deref(),
                    then_after_yield,
                    then_resume_state,
                ),
                (
                    else_before_yield,
                    else_yield_statement.as_deref(),
                    else_after_yield,
                    else_resume_state,
                ),
            ] {
                require_state(state, sequence_end_for(before, state, protocol)?)?;
                match (yielded, resumed) {
                    (
                        Some(StatementIr::GeneratorYield {
                            suspend_state,
                            resume_state,
                            ..
                        }),
                        Some(expected),
                    ) if *suspend_state == state
                        && resume_state == expected
                        && *resume_state > last =>
                    {
                        last = *resume_state;
                    }
                    (None, None) => {}
                    _ => return Err(GeneratorLoopControlError::InvalidSuspension),
                }
                for statement in after {
                    eager_for(statement, last, protocol)?;
                }
            }
            require_state(checked_next_state(last)?, *exit_state)?;
            Ok(*exit_state)
        }
        StatementIr::TryCatch {
            try_block,
            catch_block,
            generator_plan,
            async_plan,
            ..
        } => try_end_for(
            try_block,
            Some(catch_block),
            None,
            *generator_plan,
            *async_plan,
            state,
            protocol,
        ),
        StatementIr::TryFinally {
            try_block,
            finally_block,
            generator_plan,
            async_plan,
        } => try_end_for(
            try_block,
            None,
            Some(finally_block),
            *generator_plan,
            *async_plan,
            state,
            protocol,
        ),
        StatementIr::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
            generator_plan,
            async_plan,
            ..
        } => try_end_for(
            try_block,
            Some(catch_block),
            Some(finally_block),
            *generator_plan,
            *async_plan,
            state,
            protocol,
        ),
        StatementIr::If {
            then_branch,
            else_branch,
            ..
        } => {
            eager_for(then_branch, state, protocol)?;
            if let Some(branch) = else_branch {
                eager_for(branch, state, protocol)?;
            }
            Ok(state)
        }
        StatementIr::Labelled {
            statement,
            async_plan: None,
            ..
        } => match statement.as_ref() {
            StatementIr::OrdinaryGeneratorLoop(_)
            | StatementIr::AsyncGeneratorLoop(_)
            | StatementIr::AsyncGeneratorIf(_)
            | StatementIr::AsyncGeneratorWith(_)
            | StatementIr::AsyncGeneratorSwitch(_)
            | StatementIr::AsyncGeneratorForOf(_)
            | StatementIr::AsyncGeneratorForIn(_)
            | StatementIr::OrdinaryGeneratorSwitch(_)
            | StatementIr::OrdinaryGeneratorWith(_)
            | StatementIr::OrdinaryGeneratorIf(_)
            | StatementIr::Block(_)
            | StatementIr::LexicalBlock(_)
            | StatementIr::Labelled { .. } => statement_end_for(statement, state, protocol),
            _ => {
                eager_for(statement, state, protocol)?;
                Ok(state)
            }
        },
        StatementIr::For { init, body, .. } => {
            match init {
                Some(crate::ForInitIr::Statements(statements)) => {
                    require_state(state, sequence_end_for(statements, state, protocol)?)?;
                }
                Some(
                    crate::ForInitIr::SyncDisposable(_) | crate::ForInitIr::AsyncDisposable(_),
                ) => return Err(GeneratorLoopControlError::ForeignContinuation),
                Some(
                    crate::ForInitIr::Lexical { .. }
                    | crate::ForInitIr::LexicalBlock(_)
                    | crate::ForInitIr::Var(_)
                    | crate::ForInitIr::Expression(_),
                )
                | None => {}
            }
            eager_for(body, state, protocol)?;
            Ok(state)
        }
        StatementIr::ForOfIterator { head, body, .. } => {
            if matches!(
                head,
                crate::ForOfIteratorHeadIr::AsyncDisposable(_)
                    | crate::ForOfIteratorHeadIr::Assignment {
                        async_plan: Some(_),
                        ..
                    }
            ) {
                return Err(GeneratorLoopControlError::ForeignContinuation);
            }
            eager_for(body, state, protocol)?;
            Ok(state)
        }
        StatementIr::While { body, .. }
        | StatementIr::DoWhile { body, .. }
        | StatementIr::ForInArray { body, .. }
        | StatementIr::ForInString { body, .. }
        | StatementIr::ForInObject { body, .. } => {
            eager_for(body, state, protocol)?;
            Ok(state)
        }
        StatementIr::Switch {
            lexical_declarations,
            cases,
            ..
        } => {
            require_state(
                state,
                sequence_end_for(lexical_declarations, state, protocol)?,
            )?;
            for case in cases {
                require_state(
                    state,
                    sequence_end_for(&case.body.statements, state, protocol)?,
                )?;
            }
            Ok(state)
        }
        StatementIr::ParameterInitialization { statements, .. } => {
            require_state(state, sequence_end_for(statements, state, protocol)?)?;
            Ok(state)
        }
        StatementIr::SyncDisposableScope { execution, .. } => {
            use crate::SyncDisposableScopeExecutionIr;
            match execution {
                SyncDisposableScopeExecutionIr::Immediate => {}
                SyncDisposableScopeExecutionIr::PlainGenerator(_)
                    if protocol == StatementProtocol::Generator => {}
                SyncDisposableScopeExecutionIr::AsyncGenerator(_)
                    if protocol == StatementProtocol::AsyncGenerator => {}
                SyncDisposableScopeExecutionIr::PlainGenerator(_)
                | SyncDisposableScopeExecutionIr::AsyncFunction(_)
                | SyncDisposableScopeExecutionIr::AsyncGenerator(_) => {
                    return Err(GeneratorLoopControlError::ForeignContinuation);
                }
            }
            crate::SynchronousLoopBodyIr::new(statement)
                .map_err(|_| GeneratorLoopControlError::ForeignContinuation)?;
            Ok(state)
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
        | StatementIr::Continue { .. } => Ok(state),
        StatementIr::ArrayDestructuringOperation(_) => match protocol {
            StatementProtocol::Generator => Ok(state),
            StatementProtocol::AsyncGenerator => {
                Err(GeneratorLoopControlError::ForeignContinuation)
            }
        },
        StatementIr::ModuleUnitOnce { .. }
        | StatementIr::AsyncDisposableScope { .. }
        | StatementIr::AsyncModuleInstantiation
        | StatementIr::GeneratorLoop { .. }
        | StatementIr::AsyncFunctionIf { .. }
        | StatementIr::AsyncFunctionArrayDestructuring(_)
        | StatementIr::AsyncFunctionWith(_)
        | StatementIr::AsyncFunctionWhile(_)
        | StatementIr::AsyncFunctionSwitch(_)
        | StatementIr::AsyncFunctionForOfIterator { .. }
        | StatementIr::GeneratorForOfIterator { .. }
        | StatementIr::Labelled {
            async_plan: Some(_),
            ..
        } => Err(GeneratorLoopControlError::ForeignContinuation),
    }
}

fn collect_suspensions_for(statements: &[StatementIr], points: &mut SuspensionCensus<'_>) {
    for statement in statements {
        match statement {
            StatementIr::EmptyStatementCompletion(item) => {
                collect_suspensions_for(std::slice::from_ref(item.statement()), points);
            }
            StatementIr::GeneratorYield {
                suspend_state,
                resume_state,
                ..
            } => points.push_yield(*suspend_state, *resume_state),
            StatementIr::AsyncAwait {
                suspend_state,
                resume_state,
                ..
            } => points.push_await(*suspend_state, *resume_state),
            StatementIr::AsyncGeneratorLoop(plan) => {
                points.extend_checked_protocol(plan.execution(), plan.suspensions())
            }
            StatementIr::AsyncGeneratorIf(plan) => points.extend_mixed(plan.suspensions()),
            StatementIr::AsyncGeneratorWith(plan) => points.extend_mixed(plan.suspensions()),
            StatementIr::AsyncGeneratorSwitch(plan) => {
                points.extend_checked_protocol(plan.execution(), plan.suspensions())
            }
            StatementIr::AsyncGeneratorForIn(plan) => {
                points.extend_checked_protocol(plan.execution(), plan.suspensions())
            }
            StatementIr::AsyncGeneratorForOf(plan) => {
                points.extend_checked_protocol(plan.execution(), plan.suspensions())
            }
            StatementIr::AsyncGeneratorArrayDestructuring(plan) => {
                points.extend_mixed(plan.suspensions())
            }
            StatementIr::AsyncGeneratorResourceScope(plan) => {
                points.extend_checked_protocol(plan.execution(), plan.suspensions())
            }
            StatementIr::AsyncGeneratorResourceRegistration(_) => {}
            StatementIr::Block(block) => collect_suspensions_for(&block.statements, points),
            StatementIr::LexicalBlock(statements) => collect_suspensions_for(statements, points),
            StatementIr::OrdinaryGeneratorLoop(plan) => {
                for region in plan.regions() {
                    collect_suspensions_for(&region.block.statements, points);
                }
            }
            StatementIr::OrdinaryGeneratorIf(plan) => {
                collect_suspensions_for(&plan.then_branch.block.statements, points);
                collect_suspensions_for(&plan.else_branch.block.statements, points);
            }
            StatementIr::OrdinaryGeneratorSwitch(plan) => {
                for region in plan.regions() {
                    collect_suspensions_for(&region.block().statements, points);
                }
            }
            StatementIr::OrdinaryGeneratorArrayDestructuring(plan) => {
                collect_suspensions_for(&plan.body().block().statements, points);
            }
            StatementIr::OrdinaryGeneratorWith(plan) => {
                collect_suspensions_for(&plan.head().region().block().statements, points);
                collect_suspensions_for(&plan.body().block().statements, points);
            }
            StatementIr::ResumableClassDefinition(plan) => {
                for prefix in plan.prefixes() {
                    collect_suspensions_for(prefix.statements(), points);
                }
            }
            StatementIr::Switch {
                lexical_declarations,
                cases,
                ..
            } => {
                collect_suspensions_for(lexical_declarations, points);
                for case in cases {
                    collect_suspensions_for(&case.body.statements, points);
                }
            }
            StatementIr::GeneratorIf {
                then_before_yield,
                then_yield_statement,
                then_after_yield,
                else_before_yield,
                else_yield_statement,
                else_after_yield,
                ..
            } => {
                for statement in then_before_yield
                    .iter()
                    .chain(then_yield_statement.as_deref())
                    .chain(then_after_yield)
                    .chain(else_before_yield)
                    .chain(else_yield_statement.as_deref())
                    .chain(else_after_yield)
                {
                    collect_suspensions_for(std::slice::from_ref(statement), points);
                }
            }
            StatementIr::TryCatch {
                try_block,
                catch_block,
                ..
            } => {
                collect_suspensions_for(&try_block.statements, points);
                collect_suspensions_for(&catch_block.statements, points);
            }
            StatementIr::TryFinally {
                try_block,
                finally_block,
                ..
            } => {
                collect_suspensions_for(&try_block.statements, points);
                collect_suspensions_for(&finally_block.statements, points);
            }
            StatementIr::TryCatchFinally {
                try_block,
                catch_block,
                finally_block,
                ..
            } => {
                collect_suspensions_for(&try_block.statements, points);
                collect_suspensions_for(&catch_block.statements, points);
                collect_suspensions_for(&finally_block.statements, points);
            }
            StatementIr::Labelled { statement, .. } => {
                collect_suspensions_for(std::slice::from_ref(statement.as_ref()), points)
            }
            StatementIr::Empty
            | StatementIr::ModuleImportBinding(_)
            | StatementIr::Lexical { .. }
            | StatementIr::AnnexBFunctionCopy { .. }
            | StatementIr::Var(_)
            | StatementIr::DeclarationEvaluation(_)
            | StatementIr::Expression(_)
            | StatementIr::ArrayDestructuringOperation(_)
            | StatementIr::Debugger
            | StatementIr::Throw(_)
            | StatementIr::Return(_)
            | StatementIr::Break { .. }
            | StatementIr::Continue { .. }
            | StatementIr::If { .. }
            | StatementIr::ParameterInitialization { .. }
            | StatementIr::While { .. }
            | StatementIr::DoWhile { .. }
            | StatementIr::For { .. }
            | StatementIr::ForOfIterator { .. }
            | StatementIr::ForInArray { .. }
            | StatementIr::ForInString { .. }
            | StatementIr::ForInObject { .. }
            | StatementIr::ModuleUnitOnce { .. }
            | StatementIr::SyncDisposableScope { .. }
            | StatementIr::AsyncDisposableScope { .. }
            | StatementIr::AsyncModuleInstantiation
            | StatementIr::GeneratorLoop { .. }
            | StatementIr::AsyncFunctionIf { .. }
            | StatementIr::AsyncFunctionArrayDestructuring(_)
            | StatementIr::AsyncFunctionWith(_)
            | StatementIr::AsyncFunctionWhile(_)
            | StatementIr::AsyncFunctionSwitch(_)
            | StatementIr::AsyncFunctionForOfIterator { .. }
            | StatementIr::GeneratorForOfIterator { .. } => {}
        }
    }
}
