use super::*;
use crate::TdzPlaceholderName;

/// Storage derived once from a checked identifier head and its environments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResumableSyncForOfBindingStorageIr {
    Activation(ForOfAssignmentIr),
    IterationEnvironment(ForOfAssignmentIr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResumableSyncForOfBindingError {
    BindingHeadEnvironmentRequired {
        mode: BindingMode,
        name: String,
    },
    VarBindingHasHeadEnvironment {
        name: String,
        tdz_placeholder_names: Vec<String>,
        iteration_storage_names: Vec<String>,
    },
    SingleBindingTdzNameCount {
        name: String,
        tdz_placeholder_names: Vec<String>,
    },
    SingleBindingTdzNameMismatch {
        source_name: String,
        expected_name: String,
        actual_name: String,
    },
    SingleBindingIterationNamesMismatch {
        name: String,
        iteration_storage_names: Vec<String>,
    },
    DuplicateTdzPlaceholderName {
        origin: &'static str,
        name: String,
    },
    InvalidEnvironmentLayout(AsyncFunctionForOfIteratorEnvironmentError),
    CapturedTdzEnvironment {
        tdz_placeholder_names: Vec<String>,
    },
}

/// The identifier-head proof shared by plain async and synchronous generators.
/// Raw head/environment pairs cannot cross into a resumable iterator plan.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValidatedResumableSyncForOfBindingIr {
    storage: ResumableSyncForOfBindingStorageIr,
    head_environment: Option<ForInOfEnvironmentIr>,
    iteration_environment: ResumableLoopIterationEnvironmentIr,
}

impl ValidatedResumableSyncForOfBindingIr {
    pub(crate) fn new(
        source_name: &str,
        binding: ForOfAssignmentIr,
        head_environment: Option<ForInOfEnvironmentIr>,
    ) -> Result<Self, ResumableSyncForOfBindingError> {
        let binding_name = binding.name.clone();
        let (storage, iteration_environment) = match binding.mode {
            BindingMode::Var => {
                if let Some(environment) = &head_environment {
                    return Err(
                        ResumableSyncForOfBindingError::VarBindingHasHeadEnvironment {
                            name: binding_name,
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
                    ResumableSyncForOfBindingStorageIr::Activation(binding),
                    ResumableLoopIterationEnvironmentIr::StorageOnly,
                )
            }
            BindingMode::Let | BindingMode::Const => {
                let environment = head_environment.as_ref().ok_or_else(|| {
                    ResumableSyncForOfBindingError::BindingHeadEnvironmentRequired {
                        mode: binding.mode,
                        name: binding_name.clone(),
                    }
                })?;
                if let Some(name) =
                    duplicate_async_function_for_of_name(&environment.tdz_binding_names)
                {
                    return Err(
                        ResumableSyncForOfBindingError::DuplicateTdzPlaceholderName {
                            origin: "single-binding head environment",
                            name,
                        },
                    );
                }
                if environment.tdz_binding_names.len() != 1 {
                    return Err(ResumableSyncForOfBindingError::SingleBindingTdzNameCount {
                        name: binding_name.clone(),
                        tdz_placeholder_names: environment.tdz_binding_names.clone(),
                    });
                }
                let expected_name = TdzPlaceholderName::for_source_name(source_name).into_string();
                if environment.tdz_binding_names[0] != expected_name {
                    return Err(
                        ResumableSyncForOfBindingError::SingleBindingTdzNameMismatch {
                            source_name: source_name.to_owned(),
                            expected_name,
                            actual_name: environment.tdz_binding_names[0].clone(),
                        },
                    );
                }
                if let Some(tdz_environment) = &environment.tdz_environment {
                    validate_async_function_for_of_environment(
                        "single-binding TDZ environment",
                        tdz_environment,
                        &environment.tdz_binding_names,
                    )
                    .map_err(ResumableSyncForOfBindingError::InvalidEnvironmentLayout)?;
                }
                if let Some(iteration_environment) = &environment.iteration_environment {
                    let expected = vec![binding_name.clone()];
                    let actual = validate_async_function_for_of_environment(
                        "single-binding iteration environment",
                        iteration_environment,
                        &expected,
                    )
                    .map_err(ResumableSyncForOfBindingError::InvalidEnvironmentLayout)?;
                    if !async_function_for_of_names_match(&expected, &actual) {
                        return Err(
                            ResumableSyncForOfBindingError::SingleBindingIterationNamesMismatch {
                                name: binding_name,
                                iteration_storage_names: actual,
                            },
                        );
                    }
                    (
                        ResumableSyncForOfBindingStorageIr::IterationEnvironment(binding),
                        ResumableLoopIterationEnvironmentIr::FreshPerIteration(
                            iteration_environment.clone(),
                        ),
                    )
                } else {
                    if environment.tdz_environment.is_some() {
                        return Err(ResumableSyncForOfBindingError::CapturedTdzEnvironment {
                            tdz_placeholder_names: environment.tdz_binding_names.clone(),
                        });
                    }
                    (
                        ResumableSyncForOfBindingStorageIr::Activation(binding),
                        ResumableLoopIterationEnvironmentIr::StorageOnly,
                    )
                }
            }
        };
        Ok(Self {
            storage,
            head_environment,
            iteration_environment,
        })
    }

    pub(crate) fn storage(&self) -> &ResumableSyncForOfBindingStorageIr {
        &self.storage
    }
    pub(crate) fn binding(&self) -> &ForOfAssignmentIr {
        match &self.storage {
            ResumableSyncForOfBindingStorageIr::Activation(binding)
            | ResumableSyncForOfBindingStorageIr::IterationEnvironment(binding) => binding,
        }
    }
    pub(crate) fn head_environment(&self) -> Option<&ForInOfEnvironmentIr> {
        self.head_environment.as_ref()
    }
    pub(crate) fn iteration_environment(&self) -> &ResumableLoopIterationEnvironmentIr {
        &self.iteration_environment
    }

    pub(super) fn into_async_parts(
        self,
    ) -> (
        AsyncFunctionForOfIteratorValueStorageIr,
        BindingMode,
        ResumableLoopIterationEnvironmentIr,
        Vec<StatementIr>,
        Option<ForInOfEnvironmentIr>,
    ) {
        let mode = self.binding().mode;
        let storage = match self.storage {
            ResumableSyncForOfBindingStorageIr::Activation(binding) => {
                AsyncFunctionForOfIteratorValueStorageIr::Activation(binding)
            }
            ResumableSyncForOfBindingStorageIr::IterationEnvironment(binding) => {
                AsyncFunctionForOfIteratorValueStorageIr::IterationEnvironment(binding)
            }
        };
        (
            storage,
            mode,
            self.iteration_environment,
            Vec::new(),
            self.head_environment,
        )
    }
}

impl From<ResumableSyncForOfBindingError> for AsyncFunctionForOfIteratorPlanError {
    fn from(error: ResumableSyncForOfBindingError) -> Self {
        match error {
            ResumableSyncForOfBindingError::BindingHeadEnvironmentRequired { mode, name } => {
                Self::BindingHeadEnvironmentRequired { mode, name }
            }
            ResumableSyncForOfBindingError::VarBindingHasHeadEnvironment {
                name,
                tdz_placeholder_names,
                iteration_storage_names,
            } => Self::VarBindingHasHeadEnvironment {
                name,
                tdz_placeholder_names,
                iteration_storage_names,
            },
            ResumableSyncForOfBindingError::SingleBindingTdzNameCount {
                name,
                tdz_placeholder_names,
            } => Self::SingleBindingTdzNameCount {
                name,
                tdz_placeholder_names,
            },
            ResumableSyncForOfBindingError::SingleBindingTdzNameMismatch {
                source_name,
                expected_name,
                actual_name,
            } => Self::SingleBindingTdzNameMismatch {
                source_name,
                expected_name,
                actual_name,
            },
            ResumableSyncForOfBindingError::SingleBindingIterationNamesMismatch {
                name,
                iteration_storage_names,
            } => Self::SingleBindingIterationNamesMismatch {
                name,
                iteration_storage_names,
            },
            ResumableSyncForOfBindingError::DuplicateTdzPlaceholderName { origin, name } => {
                Self::DuplicateTdzPlaceholderName { origin, name }
            }
            ResumableSyncForOfBindingError::InvalidEnvironmentLayout(error) => {
                Self::InvalidEnvironmentLayout(error)
            }
            ResumableSyncForOfBindingError::CapturedTdzEnvironment {
                tdz_placeholder_names,
            } => Self::CapturedTdzEnvironment {
                tdz_placeholder_names,
            },
        }
    }
}

/// Eager lexical BindingInitialization with its actual TDZ and fresh iteration
/// environments. The incoming value is a separate compiler-local sink.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValidatedResumableSyncForOfLexicalPatternIr {
    mode: BindingMode,
    value_name: String,
    head_environment: ForInOfEnvironmentIr,
    iteration_environment: ResumableLoopIterationEnvironmentIr,
    initialization: Vec<StatementIr>,
}

impl ValidatedResumableSyncForOfLexicalPatternIr {
    pub(crate) fn new(
        mode: BindingMode,
        value_name: String,
        iteration_storage_names: Vec<String>,
        tdz_placeholder_names: Vec<String>,
        initialization: Vec<StatementIr>,
        head_environment: Option<ForInOfEnvironmentIr>,
    ) -> Result<Self, AsyncFunctionForOfIteratorPlanError> {
        match mode {
            BindingMode::Let | BindingMode::Const => {}
            BindingMode::Var => {
                return Err(AsyncFunctionForOfIteratorPlanError::LexicalPatternMode { mode });
            }
        }
        if let Some(name) = duplicate_async_function_for_of_name(&iteration_storage_names) {
            return Err(
                AsyncFunctionForOfIteratorPlanError::DuplicateLexicalPatternIterationStorageName {
                    name,
                },
            );
        }
        if let Some(name) = duplicate_async_function_for_of_name(&tdz_placeholder_names) {
            return Err(
                AsyncFunctionForOfIteratorPlanError::DuplicateTdzPlaceholderName {
                    origin: "lexical-pattern input",
                    name,
                },
            );
        }
        if iteration_storage_names.len() != tdz_placeholder_names.len() {
            return Err(
                AsyncFunctionForOfIteratorPlanError::LexicalPatternNameCountMismatch {
                    iteration_storage_names,
                    tdz_placeholder_names,
                },
            );
        }
        if iteration_storage_names
            .iter()
            .any(|name| name == &value_name)
        {
            return Err(
                AsyncFunctionForOfIteratorPlanError::LexicalPatternValueNameCollision {
                    value_name,
                    iteration_storage_names,
                },
            );
        }
        let environment = head_environment.ok_or_else(|| {
            AsyncFunctionForOfIteratorPlanError::LexicalPatternHeadEnvironmentRequired {
                iteration_storage_names: iteration_storage_names.clone(),
                tdz_placeholder_names: tdz_placeholder_names.clone(),
            }
        })?;
        if let Some(name) = duplicate_async_function_for_of_name(&environment.tdz_binding_names) {
            return Err(
                AsyncFunctionForOfIteratorPlanError::DuplicateTdzPlaceholderName {
                    origin: "lexical-pattern head environment",
                    name,
                },
            );
        }
        if !async_function_for_of_names_match(
            &tdz_placeholder_names,
            &environment.tdz_binding_names,
        ) {
            return Err(
                AsyncFunctionForOfIteratorPlanError::LexicalPatternTdzNamesMismatch {
                    expected_names: tdz_placeholder_names,
                    actual_names: environment.tdz_binding_names.clone(),
                },
            );
        }
        if let Some(tdz_environment) = &environment.tdz_environment {
            validate_async_function_for_of_environment(
                "lexical-pattern TDZ environment",
                tdz_environment,
                &environment.tdz_binding_names,
            )
            .map_err(AsyncFunctionForOfIteratorPlanError::InvalidEnvironmentLayout)?;
        }

        let iteration_environment = if iteration_storage_names.is_empty() {
            if let Some(iteration_environment) = &environment.iteration_environment {
                let actual_names = validate_async_function_for_of_environment(
                    "empty lexical-pattern iteration environment",
                    iteration_environment,
                    &iteration_storage_names,
                )
                .map_err(AsyncFunctionForOfIteratorPlanError::InvalidEnvironmentLayout)?;
                return Err(
                    AsyncFunctionForOfIteratorPlanError::EmptyLexicalPatternHasIterationEnvironment {
                        actual_names,
                    },
                );
            }
            ResumableLoopIterationEnvironmentIr::StorageOnly
        } else {
            let Some(iteration_environment) = &environment.iteration_environment else {
                return Err(
                    AsyncFunctionForOfIteratorPlanError::LexicalPatternIterationNamesMismatch {
                        expected_names: iteration_storage_names,
                        actual_names: Vec::new(),
                    },
                );
            };
            let actual_names = validate_async_function_for_of_environment(
                "lexical-pattern iteration environment",
                iteration_environment,
                &iteration_storage_names,
            )
            .map_err(AsyncFunctionForOfIteratorPlanError::InvalidEnvironmentLayout)?;
            if !async_function_for_of_names_match(&iteration_storage_names, &actual_names) {
                return Err(
                    AsyncFunctionForOfIteratorPlanError::LexicalPatternIterationNamesMismatch {
                        expected_names: iteration_storage_names,
                        actual_names,
                    },
                );
            }
            ResumableLoopIterationEnvironmentIr::FreshPerIteration(iteration_environment.clone())
        };
        validate_async_function_for_of_initialization(
            mode,
            &iteration_storage_names,
            &initialization,
        )
        .map_err(AsyncFunctionForOfIteratorPlanError::InvalidLexicalPatternInitialization)?;

        if matches!(
            &iteration_environment,
            ResumableLoopIterationEnvironmentIr::StorageOnly
        ) && environment.tdz_environment.is_some()
        {
            return Err(
                AsyncFunctionForOfIteratorPlanError::CapturedTdzEnvironment {
                    tdz_placeholder_names: environment.tdz_binding_names.clone(),
                },
            );
        }
        Ok(Self {
            mode,
            value_name,
            head_environment: environment,
            iteration_environment,
            initialization,
        })
    }

    pub(crate) fn mode(&self) -> BindingMode {
        self.mode
    }
    pub(crate) fn value_name(&self) -> &str {
        &self.value_name
    }
    pub(crate) fn head_environment(&self) -> &ForInOfEnvironmentIr {
        &self.head_environment
    }
    pub(crate) fn iteration_environment(&self) -> &ResumableLoopIterationEnvironmentIr {
        &self.iteration_environment
    }
    pub(crate) fn initialization(&self) -> &[StatementIr] {
        &self.initialization
    }

    pub(super) fn into_async_parts(
        self,
    ) -> (
        AsyncFunctionForOfIteratorValueStorageIr,
        BindingMode,
        ResumableLoopIterationEnvironmentIr,
        Vec<StatementIr>,
        Option<ForInOfEnvironmentIr>,
    ) {
        (
            AsyncFunctionForOfIteratorValueStorageIr::EntryLocal {
                name: self.value_name,
            },
            self.mode,
            self.iteration_environment,
            self.initialization,
            Some(self.head_environment),
        )
    }
}
