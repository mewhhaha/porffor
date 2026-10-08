use super::*;
use lila_ir::{GeneratorForOfIteratorPlanIr, GeneratorForOfIteratorValueStorageIr};

#[derive(Clone, Copy)]
pub(crate) enum ResumableSyncForOfPlan<'a> {
    Async(lila_ir::AsyncFunctionForOfSynchronousIteratorIr<'a>),
    Generator(&'a GeneratorForOfIteratorPlanIr),
}

#[derive(Clone, Copy)]
pub(super) enum ResumableSyncForOfOwner {
    Async,
    Generator,
}

#[derive(Clone, Copy)]
pub(super) enum ResumableSyncForOfValueStorage<'a> {
    Activation(&'a ForOfAssignmentIr),
    IterationEnvironment(&'a ForOfAssignmentIr),
    EntryLocal(&'a str),
}

impl<'a> ResumableSyncForOfPlan<'a> {
    pub(super) fn owner(self) -> ResumableSyncForOfOwner {
        match self {
            Self::Async(_) => ResumableSyncForOfOwner::Async,
            Self::Generator(_) => ResumableSyncForOfOwner::Generator,
        }
    }

    pub(super) fn value_storage(self) -> ResumableSyncForOfValueStorage<'a> {
        match self {
            Self::Async(plan) => match plan.plan().value_storage() {
                AsyncFunctionForOfIteratorValueStorageIr::Activation(binding) => {
                    ResumableSyncForOfValueStorage::Activation(binding)
                }
                AsyncFunctionForOfIteratorValueStorageIr::IterationEnvironment(binding) => {
                    ResumableSyncForOfValueStorage::IterationEnvironment(binding)
                }
                AsyncFunctionForOfIteratorValueStorageIr::EntryLocal { name } => {
                    ResumableSyncForOfValueStorage::EntryLocal(name)
                }
            },
            Self::Generator(plan) => match plan.value_storage() {
                GeneratorForOfIteratorValueStorageIr::Activation(binding) => {
                    ResumableSyncForOfValueStorage::Activation(binding)
                }
                GeneratorForOfIteratorValueStorageIr::IterationEnvironment(binding) => {
                    ResumableSyncForOfValueStorage::IterationEnvironment(binding)
                }
                GeneratorForOfIteratorValueStorageIr::EntryLocal { name } => {
                    ResumableSyncForOfValueStorage::EntryLocal(name)
                }
            },
        }
    }

    pub(super) fn entry_state(self) -> u32 {
        match self {
            Self::Async(plan) => plan.plan().entry_state(),
            Self::Generator(plan) => plan.entry_state(),
        }
    }

    pub(super) fn exit_state(self) -> u32 {
        match self {
            Self::Async(plan) => plan.plan().exit_state(),
            Self::Generator(plan) => plan.exit_state(),
        }
    }

    pub(super) fn body_exit_state(self) -> u32 {
        match self {
            Self::Async(plan) => plan.plan().body().exit_state(),
            Self::Generator(plan) => plan.body().exit_state(),
        }
    }

    pub(super) fn body_statements(self) -> &'a [StatementIr] {
        match self {
            Self::Async(plan) => plan.plan().body().statements(),
            Self::Generator(plan) => plan.body().statements(),
        }
    }

    pub(super) fn record(self) -> &'a lila_ir::IteratorRecordIr {
        match self {
            Self::Async(plan) => plan.plan().record(),
            Self::Generator(plan) => plan.record(),
        }
    }

    pub(super) fn head_environment(self) -> Option<&'a lila_ir::ForInOfEnvironmentIr> {
        match self {
            Self::Async(plan) => plan.plan().head_environment(),
            Self::Generator(plan) => plan.head_environment(),
        }
    }

    pub(super) fn iteration_environment(self) -> &'a ResumableLoopIterationEnvironmentIr {
        match self {
            Self::Async(plan) => plan.plan().iteration_environment(),
            Self::Generator(plan) => plan.iteration_environment(),
        }
    }

    pub(super) fn value_name(self) -> &'a str {
        match self {
            Self::Async(plan) => plan.plan().value_name(),
            Self::Generator(plan) => plan.value_name(),
        }
    }

    pub(super) fn binding_mode(self) -> Option<BindingMode> {
        match self.value_storage() {
            ResumableSyncForOfValueStorage::Activation(binding)
            | ResumableSyncForOfValueStorage::IterationEnvironment(binding) => Some(binding.mode),
            ResumableSyncForOfValueStorage::EntryLocal(_) => None,
        }
    }

    pub(super) fn head_binding_environment(
        self,
    ) -> Option<(BindingMode, &'a lila_ir::ForInOfEnvironmentIr)> {
        match self {
            // Async lexical patterns keep their actual source mode, although
            // their incoming IteratorValue uses a separate entry-local sink.
            Self::Async(plan) => plan
                .plan()
                .head_environment()
                .map(|environment| (plan.plan().value_mode(), environment)),
            Self::Generator(plan) => plan.head_binding_environment(),
        }
    }
}
