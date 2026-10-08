use crate::*;

mod head;
use head::GeneratorForOfIteratorHeadIr;
pub(crate) use head::{
    GeneratorForOfAssignmentIr, GeneratorForOfIteratorHeadInputIr, GeneratorForOfLexicalPatternIr,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GeneratorForOfIteratorPlanError {
    InvalidAssignment(head::GeneratorForOfAssignmentError),
    ExitStateOverflow { body_exit_state: u32 },
}

/// IteratorValue storage borrowed from the checked generator head. EntryLocal
/// has no source binding mode and cannot select an activation or lexical cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratorForOfIteratorValueStorageIr<'a> {
    Activation(&'a ForOfAssignmentIr),
    IterationEnvironment(&'a ForOfAssignmentIr),
    EntryLocal { name: &'a str },
}

/// A synchronous generator owns every suspension in this iterator walk.
/// Construction consumes the checked head and binds its prefix to this body.
#[must_use = "a generator for-of plan must be attached to its statement"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratorForOfIteratorPlanIr {
    head: GeneratorForOfIteratorHeadIr,
    record: IteratorRecordIr,
    body: GeneratorForOfBodyIr,
    exit_state: u32,
}

impl GeneratorForOfIteratorPlanIr {
    pub(crate) fn new(
        head: GeneratorForOfIteratorHeadInputIr,
        record: IteratorRecordIr,
        body: GeneratorForOfBodyIr,
    ) -> Result<Self, GeneratorForOfIteratorPlanError> {
        let head = head
            .complete(&body)
            .map_err(GeneratorForOfIteratorPlanError::InvalidAssignment)?;
        if let GeneratorForOfIteratorValueStorageIr::EntryLocal { name } = head.storage() {
            if [
                record.iterator().as_str(),
                record.next_method().as_str(),
                record.done().as_str(),
            ]
            .contains(&name)
            {
                return Err(GeneratorForOfIteratorPlanError::InvalidAssignment(
                    head::GeneratorForOfAssignmentError::PersistentSink,
                ));
            }
        }
        let body_exit_state = body.exit_state();
        let exit_state = body_exit_state
            .checked_add(1)
            .ok_or(GeneratorForOfIteratorPlanError::ExitStateOverflow { body_exit_state })?;
        Ok(Self {
            head,
            record,
            body,
            exit_state,
        })
    }

    pub fn value_storage(&self) -> GeneratorForOfIteratorValueStorageIr<'_> {
        self.head.storage()
    }
    pub fn binding(&self) -> Option<&ForOfAssignmentIr> {
        self.head.binding()
    }
    pub fn value_name(&self) -> &str {
        match self.value_storage() {
            GeneratorForOfIteratorValueStorageIr::Activation(binding)
            | GeneratorForOfIteratorValueStorageIr::IterationEnvironment(binding) => &binding.name,
            GeneratorForOfIteratorValueStorageIr::EntryLocal { name } => name,
        }
    }
    pub fn record(&self) -> &IteratorRecordIr {
        &self.record
    }
    pub fn head_environment(&self) -> Option<&ForInOfEnvironmentIr> {
        self.head.head_environment()
    }
    pub fn head_binding_environment(&self) -> Option<(BindingMode, &ForInOfEnvironmentIr)> {
        self.head.head_binding_environment()
    }
    pub fn iteration_environment(&self) -> &ResumableLoopIterationEnvironmentIr {
        self.head.iteration_environment()
    }
    pub fn body(&self) -> &GeneratorForOfBodyIr {
        &self.body
    }
    pub fn entry_state(&self) -> u32 {
        self.body.entry_state()
    }
    pub fn exit_state(&self) -> u32 {
        self.exit_state
    }
}

#[cfg(test)]
#[path = "generator_for_of_iterator/tests.rs"]
mod tests;
