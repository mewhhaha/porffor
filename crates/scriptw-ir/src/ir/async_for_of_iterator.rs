use super::*;

/// One plain-async Iterator Record walk with its complete checked body.
/// The closed execution component selects synchronous stepping or awaited
/// Next/Close; both retain the same head, record and body ownership.
#[must_use = "a plain-async for-of plan must be attached to its statement"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncFunctionForOfIteratorPlanIr {
    value_storage: AsyncFunctionForOfIteratorValueStorageIr,
    value_mode: BindingMode,
    record: IteratorRecordIr,
    head_environment: Option<ForInOfEnvironmentIr>,
    iteration_environment: ResumableLoopIterationEnvironmentIr,
    body: AsyncFunctionForOfBodyIr,
    exit_state: u32,
    execution: AsyncFunctionForOfIteratorProtocolIr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AsyncFunctionForOfIteratorProtocolIr {
    Synchronous,
    Awaited {
        entry_state: u32,
        value_resume_state: u32,
        close_resume_state: u32,
        async_iterator_binding: String,
        close_on_rejection_binding: String,
    },
}

/// Constructor input cannot contain a Close state: it is known only after the
/// checked body and both final successors have completed validation.
enum AsyncFunctionForOfIteratorInputIr {
    Synchronous {
        entry_state: u32,
    },
    Awaited {
        entry_state: u32,
        value_resume_state: u32,
        async_iterator_binding: String,
        close_on_rejection_binding: String,
    },
}

/// The private constructors bind each borrowed execution view to its actual
/// checked owner. An awaited plan cannot enter the synchronous emitter.
#[derive(Debug, Clone, Copy)]
pub enum AsyncFunctionForOfIteratorExecutionIr<'a> {
    Synchronous(AsyncFunctionForOfSynchronousIteratorIr<'a>),
    Awaited(AsyncFunctionForAwaitOfIteratorIr<'a>),
}

#[derive(Debug, Clone, Copy)]
pub struct AsyncFunctionForOfSynchronousIteratorIr<'a> {
    plan: &'a AsyncFunctionForOfIteratorPlanIr,
}

impl<'a> AsyncFunctionForOfSynchronousIteratorIr<'a> {
    pub fn plan(self) -> &'a AsyncFunctionForOfIteratorPlanIr {
        self.plan
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AsyncFunctionForAwaitOfIteratorIr<'a> {
    plan: &'a AsyncFunctionForOfIteratorPlanIr,
    value_resume_state: u32,
    close_resume_state: u32,
    async_iterator_binding: &'a str,
    close_on_rejection_binding: &'a str,
}

impl<'a> AsyncFunctionForAwaitOfIteratorIr<'a> {
    pub fn plan(self) -> &'a AsyncFunctionForOfIteratorPlanIr {
        self.plan
    }
    pub const fn value_resume_state(self) -> u32 {
        self.value_resume_state
    }
    pub const fn close_resume_state(self) -> u32 {
        self.close_resume_state
    }
    pub const fn async_iterator_binding(self) -> &'a str {
        self.async_iterator_binding
    }
    pub const fn close_on_rejection_binding(self) -> &'a str {
        self.close_on_rejection_binding
    }
}

impl AsyncFunctionForOfIteratorPlanIr {
    pub(crate) fn new(
        head: AsyncFunctionForOfIteratorHeadIr,
        record: IteratorRecordIr,
        head_environment: Option<ForInOfEnvironmentIr>,
        statements: Vec<StatementIr>,
        entry_state: u32,
    ) -> Result<Self, AsyncFunctionForOfIteratorPlanError> {
        Self::new_with_protocol(
            head,
            record,
            head_environment,
            statements,
            AsyncFunctionForOfIteratorInputIr::Synchronous { entry_state },
        )
    }

    pub(crate) fn new_for_await(
        head: AsyncFunctionForOfIteratorHeadIr,
        record: IteratorRecordIr,
        head_environment: Option<ForInOfEnvironmentIr>,
        statements: Vec<StatementIr>,
        entry_state: u32,
        async_iterator_binding: String,
        close_on_rejection_binding: String,
    ) -> Result<Self, AsyncFunctionForOfIteratorPlanError> {
        if !matches!(&head, AsyncFunctionForOfIteratorHeadIr::Binding { .. }) {
            return Err(AsyncFunctionForOfIteratorPlanError::AwaitedBindingHeadRequired);
        }
        let value_resume_state = entry_state.checked_add(1).ok_or(
            AsyncFunctionForOfIteratorPlanError::AwaitedEntryStateOverflow { entry_state },
        )?;
        Self::new_with_protocol(
            head,
            record,
            head_environment,
            statements,
            AsyncFunctionForOfIteratorInputIr::Awaited {
                entry_state,
                value_resume_state,
                async_iterator_binding,
                close_on_rejection_binding,
            },
        )
    }

    fn new_with_protocol(
        head: AsyncFunctionForOfIteratorHeadIr,
        record: IteratorRecordIr,
        head_environment: Option<ForInOfEnvironmentIr>,
        mut statements: Vec<StatementIr>,
        input: AsyncFunctionForOfIteratorInputIr,
    ) -> Result<Self, AsyncFunctionForOfIteratorPlanError> {
        let (
            value_storage,
            value_mode,
            iteration_environment,
            mut initialization,
            head_environment,
        ) = match head {
            AsyncFunctionForOfIteratorHeadIr::Binding {
                source_name,
                binding,
            } => ValidatedResumableSyncForOfBindingIr::new(&source_name, binding, head_environment)
                .map_err(AsyncFunctionForOfIteratorPlanError::from)?
                .into_async_parts(),
            AsyncFunctionForOfIteratorHeadIr::PreparedAssignment { value_name } => {
                if let Some(environment) = &head_environment {
                    return Err(
                        AsyncFunctionForOfIteratorPlanError::PreparedAssignmentHasHeadEnvironment {
                            value_name,
                            tdz_placeholder_names: environment.tdz_binding_names.clone(),
                            iteration_storage_names: environment
                                .iteration_environment
                                .as_ref()
                                .map(async_function_for_of_environment_names)
                                .unwrap_or_default(),
                        },
                    );
                }
                (
                    AsyncFunctionForOfIteratorValueStorageIr::EntryLocal { name: value_name },
                    BindingMode::Let,
                    ResumableLoopIterationEnvironmentIr::StorageOnly,
                    Vec::new(),
                    None,
                )
            }
            AsyncFunctionForOfIteratorHeadIr::LexicalPattern {
                mode,
                value_name,
                iteration_storage_names,
                tdz_placeholder_names,
                initialization,
            } => ValidatedResumableSyncForOfLexicalPatternIr::new(
                mode,
                value_name,
                iteration_storage_names,
                tdz_placeholder_names,
                initialization,
                head_environment,
            )?
            .into_async_parts(),
        };

        if matches!(
            &iteration_environment,
            ResumableLoopIterationEnvironmentIr::StorageOnly
        ) {
            if let Some(environment) = &head_environment {
                if environment.tdz_environment.is_some() {
                    return Err(
                        AsyncFunctionForOfIteratorPlanError::CapturedTdzEnvironment {
                            tdz_placeholder_names: environment.tdz_binding_names.clone(),
                        },
                    );
                }
            }
        }
        initialization.append(&mut statements);
        let body = match &input {
            AsyncFunctionForOfIteratorInputIr::Synchronous { entry_state } => {
                AsyncFunctionForOfBodyIr::new(initialization, *entry_state)
            }
            AsyncFunctionForOfIteratorInputIr::Awaited {
                value_resume_state, ..
            } => AsyncFunctionForOfBodyIr::new_for_await(initialization, *value_resume_state),
        }
        .map_err(AsyncFunctionForOfIteratorPlanError::InvalidBody)?;
        let body_exit_state = body.exit_state();
        let successor = body_exit_state
            .checked_add(1)
            .ok_or(AsyncFunctionForOfIteratorPlanError::ExitStateOverflow { body_exit_state })?;
        let (execution, exit_state) = match input {
            AsyncFunctionForOfIteratorInputIr::Synchronous { .. } => {
                (AsyncFunctionForOfIteratorProtocolIr::Synchronous, successor)
            }
            AsyncFunctionForOfIteratorInputIr::Awaited {
                entry_state,
                value_resume_state,
                async_iterator_binding,
                close_on_rejection_binding,
            } => {
                let name = match &value_storage {
                    AsyncFunctionForOfIteratorValueStorageIr::Activation(binding)
                    | AsyncFunctionForOfIteratorValueStorageIr::IterationEnvironment(binding) => {
                        &binding.name
                    }
                    AsyncFunctionForOfIteratorValueStorageIr::EntryLocal { name } => name,
                };
                let flags_alias =
                    async_iterator_binding.as_str() == close_on_rejection_binding.as_str();
                for protocol_name in [
                    async_iterator_binding.as_str(),
                    close_on_rejection_binding.as_str(),
                ] {
                    if protocol_name.is_empty()
                        || protocol_name == name.as_str()
                        || protocol_name == record.iterator().as_str()
                        || protocol_name == record.next_method().as_str()
                        || protocol_name == record.done().as_str()
                        || flags_alias
                        || head_environment.as_ref().is_some_and(|environment| {
                            environment
                                .tdz_binding_names
                                .iter()
                                .any(|name| name.as_str() == protocol_name)
                        })
                    {
                        return Err(
                            AsyncFunctionForOfIteratorPlanError::AwaitedProtocolStorageAlias {
                                name: protocol_name.to_string(),
                            },
                        );
                    }
                }
                let close_resume_state = successor;
                let exit_state = close_resume_state.checked_add(1).ok_or(
                    AsyncFunctionForOfIteratorPlanError::ExitStateOverflow { body_exit_state },
                )?;
                (
                    AsyncFunctionForOfIteratorProtocolIr::Awaited {
                        entry_state,
                        value_resume_state,
                        close_resume_state,
                        async_iterator_binding,
                        close_on_rejection_binding,
                    },
                    exit_state,
                )
            }
        };

        Ok(Self {
            value_storage,
            value_mode,
            record,
            head_environment,
            iteration_environment,
            body,
            exit_state,
            execution,
        })
    }

    pub fn execution(&self) -> AsyncFunctionForOfIteratorExecutionIr<'_> {
        match &self.execution {
            AsyncFunctionForOfIteratorProtocolIr::Synchronous => {
                AsyncFunctionForOfIteratorExecutionIr::Synchronous(
                    AsyncFunctionForOfSynchronousIteratorIr { plan: self },
                )
            }
            AsyncFunctionForOfIteratorProtocolIr::Awaited {
                value_resume_state,
                close_resume_state,
                async_iterator_binding,
                close_on_rejection_binding,
                ..
            } => {
                AsyncFunctionForOfIteratorExecutionIr::Awaited(AsyncFunctionForAwaitOfIteratorIr {
                    plan: self,
                    value_resume_state: *value_resume_state,
                    close_resume_state: *close_resume_state,
                    async_iterator_binding,
                    close_on_rejection_binding,
                })
            }
        }
    }

    pub fn value_storage(&self) -> &AsyncFunctionForOfIteratorValueStorageIr {
        &self.value_storage
    }

    pub fn value_name(&self) -> &str {
        match &self.value_storage {
            AsyncFunctionForOfIteratorValueStorageIr::Activation(binding)
            | AsyncFunctionForOfIteratorValueStorageIr::IterationEnvironment(binding) => {
                &binding.name
            }
            AsyncFunctionForOfIteratorValueStorageIr::EntryLocal { name } => name,
        }
    }

    pub fn value_mode(&self) -> BindingMode {
        self.value_mode
    }

    pub fn record(&self) -> &IteratorRecordIr {
        &self.record
    }

    pub fn head_environment(&self) -> Option<&ForInOfEnvironmentIr> {
        self.head_environment.as_ref()
    }

    pub fn iteration_environment(&self) -> &ResumableLoopIterationEnvironmentIr {
        &self.iteration_environment
    }

    pub fn body(&self) -> &AsyncFunctionForOfBodyIr {
        &self.body
    }

    pub fn entry_state(&self) -> u32 {
        match &self.execution {
            AsyncFunctionForOfIteratorProtocolIr::Synchronous => self.body.entry_state(),
            AsyncFunctionForOfIteratorProtocolIr::Awaited { entry_state, .. } => *entry_state,
        }
    }

    pub fn exit_state(&self) -> u32 {
        self.exit_state
    }
}
