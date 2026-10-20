use super::*;

impl<'a> ScriptLowerer<'a> {
    /// Builds the activation-backed synchronous Iterator Record walk for an
    /// ordinary `for-of` whose body suspends in a plain async function.
    pub(super) fn lower_async_function_for_of_iterator_with_body_await(
        &mut self,
        head: AsyncFunctionForOfIteratorHeadIr,
        iterable: TypedExpr,
        body: StatementIr,
        body_kind: ValueKind,
        entry_state: Option<u32>,
        head_environment: Option<ForInOfEnvironmentIr>,
    ) -> ForOfLoweringIr {
        let Some(entry_state) = entry_state else {
            self.unsupported(
                "async for-of with a body await requires a plain async function body with a \
                 resumable entry state, and this body has none",
            );
            return ForOfLoweringIr::no_iteration();
        };
        let statements = match body {
            StatementIr::Block(block) if block.lexical_environment.is_none() => block.statements,
            StatementIr::LexicalBlock(statements) => statements,
            statement => vec![statement],
        };
        let statements = flatten_suspending_lexical_blocks(statements);
        let record = IteratorRecordIr::new(
            self.alloc_iterator_slot(),
            self.alloc_next_method_slot(),
            self.alloc_done_slot(),
        );
        let plan = AsyncFunctionForOfIteratorPlanIr::new(
            head,
            record,
            head_environment,
            statements,
            entry_state,
        );
        self.publish_async_function_for_of_iterator(iterable, plan, body_kind, None)
    }

    pub(super) fn lower_plain_async_for_await_with_body_await(
        &mut self,
        head: AsyncFunctionForOfIteratorHeadIr,
        iterable: TypedExpr,
        body: StatementIr,
        body_kind: ValueKind,
        entry_state: u32,
        head_environment: Option<ForInOfEnvironmentIr>,
    ) -> ForOfLoweringIr {
        let Some(lowered_body_exit) = self.current_async_resume_state else {
            self.unsupported(
                "plain async for-await-of body requires its actual lowered exit state",
            );
            return ForOfLoweringIr::no_iteration();
        };
        let statements = match body {
            StatementIr::Block(block) if block.lexical_environment.is_none() => block.statements,
            StatementIr::LexicalBlock(statements) => statements,
            statement => vec![statement],
        };
        let statements = flatten_suspending_lexical_blocks(statements);
        let record = IteratorRecordIr::new(
            self.alloc_iterator_slot(),
            self.alloc_next_method_slot(),
            self.alloc_done_slot(),
        );
        let async_iterator_binding = self.alloc_suspension_owned_binding(
            "async.forof.async_iterator.",
            ValueInfo::new(ValueKind::Boolean),
        );
        let close_on_rejection_binding = self.alloc_suspension_owned_binding(
            "async.forof.close_on_rejection.",
            ValueInfo::new(ValueKind::Boolean),
        );
        let plan = AsyncFunctionForOfIteratorPlanIr::new_for_await(
            head,
            record,
            head_environment,
            statements,
            entry_state,
            async_iterator_binding,
            close_on_rejection_binding,
        );
        self.publish_async_function_for_of_iterator(
            iterable,
            plan,
            body_kind,
            Some(lowered_body_exit),
        )
    }

    fn publish_async_function_for_of_iterator(
        &mut self,
        iterable: TypedExpr,
        plan: Result<AsyncFunctionForOfIteratorPlanIr, AsyncFunctionForOfIteratorPlanError>,
        body_kind: ValueKind,
        lowered_body_exit: Option<u32>,
    ) -> ForOfLoweringIr {
        let plan = match plan {
            Ok(plan) => plan,
            Err(AsyncFunctionForOfIteratorPlanError::InvalidBody(error)) => {
                self.unsupported(&format!("invalid async for-of body continuation: {error:?}"));
                return ForOfLoweringIr::no_iteration();
            }
            Err(AsyncFunctionForOfIteratorPlanError::ExitStateOverflow { body_exit_state }) => {
                self.unsupported(&format!(
                    "async for-of exit state overflows after body completion {body_exit_state}"
                ));
                return ForOfLoweringIr::no_iteration();
            }
            Err(AsyncFunctionForOfIteratorPlanError::CapturedTdzEnvironment {
                tdz_placeholder_names,
            }) => {
                self.unsupported(&format!(
                    "async for-of with a body await cannot materialize the head's captured TDZ \
                     environment for {tdz_placeholder_names:?}"
                ));
                return ForOfLoweringIr::no_iteration();
            }
            Err(
                error @ (AsyncFunctionForOfIteratorPlanError::AwaitedBindingHeadRequired
                | AsyncFunctionForOfIteratorPlanError::AwaitedEntryStateOverflow { .. }
                | AsyncFunctionForOfIteratorPlanError::AwaitedProtocolStorageAlias { .. }
                | AsyncFunctionForOfIteratorPlanError::BindingHeadEnvironmentRequired {
                    ..
                }
                | AsyncFunctionForOfIteratorPlanError::VarBindingHasHeadEnvironment { .. }
                | AsyncFunctionForOfIteratorPlanError::SingleBindingTdzNameCount { .. }
                | AsyncFunctionForOfIteratorPlanError::SingleBindingTdzNameMismatch { .. }
                | AsyncFunctionForOfIteratorPlanError::SingleBindingIterationNamesMismatch {
                    ..
                }
                | AsyncFunctionForOfIteratorPlanError::PreparedAssignmentHasHeadEnvironment {
                    ..
                }
                | AsyncFunctionForOfIteratorPlanError::LexicalPatternMode { .. }
                | AsyncFunctionForOfIteratorPlanError::LexicalPatternHeadEnvironmentRequired {
                    ..
                }
                | AsyncFunctionForOfIteratorPlanError::LexicalPatternNameCountMismatch { .. }
                | AsyncFunctionForOfIteratorPlanError::DuplicateTdzPlaceholderName { .. }
                | AsyncFunctionForOfIteratorPlanError::DuplicateLexicalPatternIterationStorageName {
                    ..
                }
                | AsyncFunctionForOfIteratorPlanError::LexicalPatternTdzNamesMismatch { .. }
                | AsyncFunctionForOfIteratorPlanError::LexicalPatternIterationNamesMismatch {
                    ..
                }
                | AsyncFunctionForOfIteratorPlanError::EmptyLexicalPatternHasIterationEnvironment {
                    ..
                }
                | AsyncFunctionForOfIteratorPlanError::LexicalPatternValueNameCollision { .. }
                | AsyncFunctionForOfIteratorPlanError::InvalidEnvironmentLayout(_)
                | AsyncFunctionForOfIteratorPlanError::InvalidLexicalPatternInitialization(_)),
            ) => {
                self.unsupported(&format!(
                    "invalid resumable async for-of head invariant: {error:?}"
                ));
                return ForOfLoweringIr::no_iteration();
            }
        };
        let body_exit_matches = match (plan.execution(), lowered_body_exit) {
            (AsyncFunctionForOfIteratorExecutionIr::Synchronous(_), None) => true,
            (AsyncFunctionForOfIteratorExecutionIr::Awaited(_), Some(actual)) => {
                actual == plan.body().exit_state()
            }
            (AsyncFunctionForOfIteratorExecutionIr::Synchronous(_), Some(_))
            | (AsyncFunctionForOfIteratorExecutionIr::Awaited(_), None) => false,
        };
        if !body_exit_matches {
            self.unsupported(
                "for-await-of body continuation disagrees with its checked state span",
            );
            return ForOfLoweringIr::no_iteration();
        }
        match plan.value_storage() {
            AsyncFunctionForOfIteratorValueStorageIr::Activation(binding) => {
                self.add_suspension_owned_binding(binding.name.clone(), binding.mode);
            }
            AsyncFunctionForOfIteratorValueStorageIr::IterationEnvironment(_)
            | AsyncFunctionForOfIteratorValueStorageIr::EntryLocal { .. } => {}
        }
        self.current_async_resume_state = Some(plan.exit_state());
        ForOfLoweringIr::async_function_iterator(iterable, plan, body_kind)
    }
}
