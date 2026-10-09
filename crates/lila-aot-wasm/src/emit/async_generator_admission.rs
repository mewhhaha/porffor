//! Admission follows the same checked loop and CaseBlock owners as emission.
use super::*;

#[derive(Clone, Copy)]
enum Branch {
    Break,
    Continue,
}

enum ControlScope<'a> {
    Function,
    Iteration {
        parent: &'a ControlScope<'a>,
        labels: &'a [String],
    },
    CaseBlock {
        parent: &'a ControlScope<'a>,
        labels: &'a [String],
    },
    Labelled {
        parent: &'a ControlScope<'a>,
        labels: &'a [String],
    },
}

impl ControlScope<'_> {
    fn accepts(&self, branch: Branch, label: Option<&str>) -> bool {
        match self {
            Self::Function => false,
            Self::Iteration { parent, labels } => {
                label.is_none_or(|label| labels.iter().any(|name| name == label))
                    || parent.accepts(branch, label)
            }
            Self::Labelled { parent, labels } => match branch {
                Branch::Break => {
                    label.is_some_and(|label| labels.iter().any(|name| name == label))
                        || parent.accepts(branch, label)
                }
                Branch::Continue => {
                    !label.is_some_and(|label| labels.iter().any(|name| name == label))
                        && parent.accepts(branch, label)
                }
            },
            Self::CaseBlock { parent, labels } => match branch {
                Branch::Break => {
                    label.is_none_or(|label| labels.iter().any(|name| name == label))
                        || parent.accepts(branch, label)
                }
                Branch::Continue => {
                    // A matching non-iteration label cannot name a Continue target.
                    !label.is_some_and(|label| labels.iter().any(|name| name == label))
                        && parent.accepts(branch, label)
                }
            },
        }
    }
}

pub(super) fn check(statement: &StatementIr) -> Option<&'static str> {
    unsupported(statement, &ControlScope::Function)
}

fn async_generator_scoped_resource_unsupported_feature(
    statement: &StatementIr,
    resource: &lila_ir::AsyncGeneratorScopedResourceIr,
    scope: &ControlScope<'_>,
) -> Option<&'static str> {
    match statement {
        StatementIr::AsyncGeneratorResourceRegistration(operation)
            if operation.capability_binding() == resource.capability_binding() =>
        {
            None
        }
        StatementIr::EmptyStatementCompletion(item) => {
            async_generator_scoped_resource_unsupported_feature(item.statement(), resource, scope)
        }
        StatementIr::LexicalBlock(statements) => statements.iter().find_map(|statement| {
            async_generator_scoped_resource_unsupported_feature(statement, resource, scope)
        }),
        StatementIr::Block(block) => block.statements.iter().find_map(|statement| {
            async_generator_scoped_resource_unsupported_feature(statement, resource, scope)
        }),
        _ => unsupported(statement, scope),
    }
}

fn unsupported(statement: &StatementIr, scope: &ControlScope<'_>) -> Option<&'static str> {
    let visit = |statement: &StatementIr| unsupported(statement, scope);
    match statement {
        StatementIr::ResumableClassDefinition(plan) => plan
            .prefixes()
            .flat_map(|prefix| prefix.statements())
            .find_map(visit),
        StatementIr::ModuleImportBinding(_) => Some("module import binding"),
        StatementIr::AsyncModuleInstantiation => Some("async module instantiation"),
        StatementIr::ModuleUnitOnce { .. } => Some("module unit evaluation"),
        StatementIr::AsyncFunctionWhile(_) => Some("plain async function awaited while condition"),
        StatementIr::AsyncFunctionIf { .. } => Some("plain async function branches"),
        StatementIr::AsyncFunctionSwitch(_) => Some("plain async function switch statements"),
        // The opaque mixed Array constructor validates its complete closed
        // body, including the concrete owner of every iterator operation.
        StatementIr::AsyncGeneratorArrayDestructuring(_) => None,
        StatementIr::AsyncGeneratorResourceScope(plan) => (plan.execution()!=lila_ir::ResumableRegionProtocolIr::AsyncGenerator)
            .then_some("complete resource scope belongs to its original generator or async execution protocol"),
        StatementIr::AsyncGeneratorResourceRegistration(_) => Some("resource registration requires its checked complete scope"),
        StatementIr::AsyncFunctionArrayDestructuring(_) => {
            Some("plain async function array destructuring")
        }
        StatementIr::AsyncFunctionWith(_) => Some("plain async function With statements"),
        StatementIr::Empty
        | StatementIr::Lexical { .. }
        | StatementIr::AnnexBFunctionCopy { .. }
        | StatementIr::Var(_)
        | StatementIr::DeclarationEvaluation(_)
        | StatementIr::Expression(_)
        | StatementIr::Debugger
        | StatementIr::Throw(_)
        | StatementIr::Return(_) => None,
        StatementIr::EmptyStatementCompletion(item) => {
            visit(item.statement())
        }
        StatementIr::LexicalBlock(statements)
        | StatementIr::ParameterInitialization { statements, .. } => statements
            .iter()
            .find_map(visit),
        StatementIr::Block(block) => block
            .statements
            .iter()
            .find_map(visit),
        StatementIr::GeneratorYield { .. } | StatementIr::AsyncAwait { .. } => None,
        StatementIr::AsyncGeneratorLoop(plan) => {
            let scope = &ControlScope::Iteration { parent: scope, labels: &[] };
            let visit = |statement: &StatementIr| unsupported(statement, scope);
            if plan.execution()!=lila_ir::ResumableRegionProtocolIr::AsyncGenerator {return Some("complete classic loop belongs to its original generator or async execution protocol");}
            plan.regions()
            .flat_map(|region| &region.block().statements)
            .find_map(|statement| match plan.resource() {
                Some(resource) => async_generator_scoped_resource_unsupported_feature(statement, resource, scope),
                None => visit(statement),
            })
        },
        StatementIr::AsyncGeneratorIf(plan) => plan
            .regions()
            .flat_map(|region| &region.block().statements)
            .find_map(visit),
        StatementIr::AsyncGeneratorWith(plan) => plan
            .regions()
            .flat_map(|region| &region.block().statements)
            .find_map(visit),
        StatementIr::AsyncGeneratorForIn(plan) => {
            let scope = &ControlScope::Iteration { parent: scope, labels: &[] };
            let visit = |statement: &StatementIr| unsupported(statement, scope);
            if plan.execution()!=lila_ir::ResumableRegionProtocolIr::AsyncGenerator {
                return Some("complete ForIn belongs to its original generator or async execution protocol");
            }
            [plan.head().region().block(),plan.initialization(),plan.body().block()]
                .into_iter().flat_map(|block|&block.statements)
                .find_map(visit)
        },
        StatementIr::AsyncGeneratorForOf(plan) => {
            let scope = &ControlScope::Iteration { parent: scope, labels: &[] };
            let visit = |statement: &StatementIr| unsupported(statement, scope);
            if plan.execution()!=lila_ir::ResumableRegionProtocolIr::AsyncGenerator {
                return Some("complete iterator belongs to its original generator or async execution protocol");
            }
            [Some(plan.head().region().block()),plan.resource().is_none().then_some(plan.initialization().block()),Some(plan.body().block())]
                .into_iter().flatten().flat_map(|block|&block.statements)
                .find_map(visit)
        },
        StatementIr::AsyncGeneratorSwitch(plan) => {
            let scope = &ControlScope::CaseBlock { parent: scope, labels: &[] };
            let visit = |statement: &StatementIr| unsupported(statement, scope);
            if plan.execution()!=lila_ir::ResumableRegionProtocolIr::AsyncGenerator {return Some("complete CaseBlock belongs to its original generator or async execution protocol");}
            plan.regions()
            .flat_map(|region| &region.block().statements)
            .find_map(|statement| match plan.resource() {
                Some(resource) => async_generator_scoped_resource_unsupported_feature(statement, resource, scope),
                None => visit(statement),
            })
        },
        StatementIr::OrdinaryGeneratorLoop(_)
        | StatementIr::OrdinaryGeneratorIf(_)
        | StatementIr::OrdinaryGeneratorSwitch(_)
        | StatementIr::OrdinaryGeneratorWith(_)
        | StatementIr::ArrayDestructuringOperation(_)
        | StatementIr::OrdinaryGeneratorArrayDestructuring(_) => {
            Some("ordinary generator phases require a plain generator function")
        }
        StatementIr::GeneratorLoop {
            before_suspension,
            suspension_statement,
            after_suspension,
            entry_state,
            resume_state,
            exit_state,
            ..
        } => {
            let contains_suspension = |statement: &StatementIr| {
                async_generator_contains_suspension(statement, AsyncGeneratorSuspension::Await)
                    || async_generator_contains_suspension(
                        statement,
                        AsyncGeneratorSuspension::Yield,
                    )
            };
            if before_suspension.iter().any(contains_suspension) {
                return Some("resumable loops with a suspending prelude");
            }
            let final_resume_state = match suspension_statement.as_ref() {
                StatementIr::GeneratorYield {
                    form,
                    suspend_state,
                    resume_state,
                    ..
                } => {
                    match form {
                        YieldForm::Plain => {}
                        YieldForm::Delegate(_) => {
                            return Some("resumable loops with delegated yield");
                        }
                    }
                    if suspend_state != entry_state {
                        return Some("resumable loops with non-linear suspension states");
                    }
                    if after_suspension.iter().any(contains_suspension) {
                        return Some("resumable yield loops containing multiple suspensions");
                    }
                    *resume_state
                }
                StatementIr::AsyncAwait { .. } => match direct_await_sequence_resume_state(
                    suspension_statement,
                    after_suspension,
                    *entry_state,
                ) {
                    Ok(final_resume_state) => final_resume_state,
                    Err(AwaitSequenceError::FirstAwaitRequired) => {
                        unreachable!("the loop's first suspension is an await")
                    }
                    Err(AwaitSequenceError::NestedSuspension) => {
                        return Some("resumable await loops containing nested suspensions");
                    }
                    Err(AwaitSequenceError::StateMismatch { .. }) => {
                        return Some("resumable loops with non-linear suspension states");
                    }
                },
                _ => return Some("resumable loops without a direct suspension"),
            };
            if final_resume_state != *resume_state {
                return Some("resumable loops with non-linear suspension states");
            }
            if exit_state != resume_state {
                return Some("resumable loops with an unplanned exit state");
            }
            std::iter::once(suspension_statement.as_ref())
                .chain(before_suspension)
                .chain(after_suspension)
                .find_map(visit)
        }
        StatementIr::AsyncFunctionForOfIterator { .. } => {
            Some("resumable synchronous for-of requires a plain async function")
        }
        StatementIr::GeneratorForOfIterator { .. } => {
            Some("resumable synchronous for-of requires a plain generator function")
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
            let surrounding_statements = then_before_yield
                .iter()
                .chain(then_after_yield)
                .chain(else_before_yield)
                .chain(else_after_yield);
            if surrounding_statements.clone().any(|statement| {
                async_generator_contains_suspension(statement, AsyncGeneratorSuspension::Await)
                    || async_generator_contains_suspension(
                        statement,
                        AsyncGeneratorSuspension::Yield,
                    )
            }) {
                return Some("resumable branches containing multiple suspensions");
            }
            surrounding_statements
                .chain(then_yield_statement.as_deref())
                .chain(else_yield_statement.as_deref())
                .find_map(visit)
        }
        StatementIr::ForOfIterator {
            head:
                ForOfIteratorHeadIr::Assignment {
                    binding,
                    async_plan: Some(_),
                    ..
                },
            body,
            ..
        } if async_generator_for_await_is_transparent_yield(&binding.name, body) => None,
        // Body yields occupy states inside the for-await plan's span. Captured
        // head bindings now reattach their existing per-iteration environment;
        // nested for-await and additional materialized body scopes are separate
        // dispatcher capabilities and remain explicitly rejected.
        StatementIr::ForOfIterator {
            head:
                ForOfIteratorHeadIr::Assignment {
                    async_plan: Some(_),
                    ..
                },
            body,
            ..
        } => {
            // A nested `for await` allocates its own four states inside this
            // loop's span, so this loop's per-iteration gate would enter the
            // inner loop's head instead of the inner loop entering it.
            if async_generator_contains_suspension(body, AsyncGeneratorSuspension::Await) {
                return Some("for-await iteration with a nested for-await in the loop body");
            }
            if async_generator_contains_suspension(body, AsyncGeneratorSuspension::Yield)
                && matches!(body.as_ref(), StatementIr::Block(block) if block.lexical_environment.is_some())
            {
                return Some(
                    "for-await-of with a block-scoped body environment and a body suspension",
                );
            }
            None
        }
        StatementIr::ForOfIterator {
            head: ForOfIteratorHeadIr::AsyncDisposable(_),
            ..
        } => Some("await using for-of requires a plain async function"),
        StatementIr::If {
            then_branch,
            else_branch,
            ..
        } => {
            if async_generator_contains_suspension(statement, AsyncGeneratorSuspension::Await)
                || async_generator_contains_suspension(statement, AsyncGeneratorSuspension::Yield)
            {
                return Some("branches containing suspension");
            }
            std::iter::once(then_branch.as_ref())
                .chain(else_branch.as_deref())
                .find_map(visit)
        }
        StatementIr::While { .. }
        | StatementIr::DoWhile { .. }
        | StatementIr::For { .. }
        | StatementIr::ForOfIterator { .. }
        | StatementIr::ForInArray { .. }
        | StatementIr::ForInString { .. }
        | StatementIr::ForInObject { .. } => Some("loops"),
        StatementIr::Switch { .. } => Some("switch statements"),
        StatementIr::Labelled { labels, statement, .. } => {
            let scope = match statement.as_ref() {
                StatementIr::AsyncGeneratorLoop(_)
                | StatementIr::AsyncGeneratorForIn(_)
                | StatementIr::AsyncGeneratorForOf(_) => ControlScope::Iteration { parent: scope, labels },
                StatementIr::AsyncGeneratorSwitch(_) => ControlScope::CaseBlock { parent: scope, labels },
                StatementIr::AsyncGeneratorWith(_) => ControlScope::Labelled { parent: scope, labels },
                _ => return Some("labelled statements without a checked control owner"),
            };
            unsupported(statement, &scope)
        },
        StatementIr::TryCatch {
            try_block,
            catch_block,
            async_plan: Some(_),
            ..
        } => try_block
            .statements
            .iter()
            .chain(&catch_block.statements)
            .find_map(visit),
        StatementIr::TryFinally {
            try_block,
            finally_block,
            async_plan: Some(_),
            ..
        } => try_block
            .statements
            .iter()
            .chain(&finally_block.statements)
            .find_map(visit),
        StatementIr::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
            async_plan: Some(_),
            ..
        } => try_block
            .statements
            .iter()
            .chain(&catch_block.statements)
            .chain(&finally_block.statements)
            .find_map(visit),
        StatementIr::TryCatch { .. }
        | StatementIr::TryFinally { .. }
        | StatementIr::TryCatchFinally { .. } => Some("try statements without a resume plan"),
        StatementIr::SyncDisposableScope {
            execution: SyncDisposableScopeExecutionIr::AsyncGenerator(_),
            body,
            ..
        } => body
            .statements
            .iter()
            .find_map(visit),
        StatementIr::SyncDisposableScope {
            execution:
                SyncDisposableScopeExecutionIr::Immediate
                | SyncDisposableScopeExecutionIr::PlainGenerator(_)
                | SyncDisposableScopeExecutionIr::AsyncFunction(_),
            ..
        } => Some("synchronous using scope with a mismatched execution owner"),
        StatementIr::AsyncDisposableScope {
            execution: AsyncDisposableScopeExecutionIr::AsyncGenerator(_),
            body,
            ..
        } => body
            .statements
            .iter()
            .find_map(visit),
        StatementIr::AsyncDisposableScope {
            execution: AsyncDisposableScopeExecutionIr::AsyncFunction(_),
            ..
        } => Some("await using scope with a mismatched execution owner"),
        StatementIr::Break { label } => (!scope.accepts(Branch::Break, label.as_deref()))
            .then_some("break without a checked control owner"),
        StatementIr::Continue { label } => (!scope.accepts(Branch::Continue, label.as_deref()))
            .then_some("continue without a checked iteration owner"),
    }
}
