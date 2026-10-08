//! A retained native IteratorRecord is owned by its real activation cell.

use crate::{BindingMode, ExprIr, OwnedEnvBindingIr, StatementIr, TypedExpr, ValueInfo, ValueKind};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArrayIteratorStorageError {
    MissingOwnedBinding,
    AliasedResultBinding,
}

/// The backend stores a native IteratorRecord in a private field of this
/// existing BindingCell. Its iterator, next method and Done are never JS values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayIteratorStorageIr {
    binding: OwnedEnvBindingIr,
}

impl ArrayIteratorStorageIr {
    pub(crate) fn new(
        binding: OwnedEnvBindingIr,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<Self, ArrayIteratorStorageError> {
        let mut matches = inventory
            .iter()
            .filter(|row| row.name == binding.name || row.slot == binding.slot);
        if matches.next() != Some(&binding) || matches.next().is_some() {
            return Err(ArrayIteratorStorageError::MissingOwnedBinding);
        }
        Ok(Self { binding })
    }

    pub fn binding(&self) -> &OwnedEnvBindingIr {
        &self.binding
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ArrayDestructuringOperation {
    StepValue {
        storage: ArrayIteratorStorageIr,
        result: OwnedEnvBindingIr,
    },
    Elision(ArrayIteratorStorageIr),
    RestArray {
        storage: ArrayIteratorStorageIr,
        result: OwnedEnvBindingIr,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrayDestructuringOperationKindIr {
    StepValue,
    Elision,
    RestArray,
}

enum ArrayValueOperation {
    StepValue,
    RestArray,
}

/// Read-only projections of actual IteratorStepValue, elision and rest owners.
pub enum ArrayDestructuringOperationView<'a> {
    StepValue(&'a ArrayIteratorStorageIr),
    Elision(&'a ArrayIteratorStorageIr),
    RestArray(&'a ArrayIteratorStorageIr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayDestructuringOperationIr(ArrayDestructuringOperation);

impl ArrayDestructuringOperationIr {
    pub(crate) fn step_value(
        storage: &ArrayIteratorStorageIr,
        result: OwnedEnvBindingIr,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<(Vec<StatementIr>, TypedExpr), ArrayIteratorStorageError> {
        Self::validate_result(storage, &result, inventory)?;
        Ok(Self::publish_result(
            ArrayValueOperation::StepValue,
            storage.clone(),
            result,
        ))
    }

    pub(crate) fn elision(storage: &ArrayIteratorStorageIr) -> StatementIr {
        StatementIr::ArrayDestructuringOperation(Box::new(Self(
            ArrayDestructuringOperation::Elision(storage.clone()),
        )))
    }

    pub(crate) fn rest_array(
        storage: &ArrayIteratorStorageIr,
        result: OwnedEnvBindingIr,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<(Vec<StatementIr>, TypedExpr), ArrayIteratorStorageError> {
        Self::validate_result(storage, &result, inventory)?;
        Ok(Self::publish_result(
            ArrayValueOperation::RestArray,
            storage.clone(),
            result,
        ))
    }

    fn validate_result(
        storage: &ArrayIteratorStorageIr,
        result: &OwnedEnvBindingIr,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<(), ArrayIteratorStorageError> {
        ArrayIteratorStorageIr::new(result.clone(), inventory)?;
        if result.name == storage.binding.name || result.slot == storage.binding.slot {
            return Err(ArrayIteratorStorageError::AliasedResultBinding);
        }
        Ok(())
    }

    pub fn use_view(&self) -> ArrayDestructuringOperationView<'_> {
        match &self.0 {
            ArrayDestructuringOperation::StepValue { storage, .. } => {
                ArrayDestructuringOperationView::StepValue(storage)
            }
            ArrayDestructuringOperation::Elision(storage) => {
                ArrayDestructuringOperationView::Elision(storage)
            }
            ArrayDestructuringOperation::RestArray { storage, .. } => {
                ArrayDestructuringOperationView::RestArray(storage)
            }
        }
    }

    pub fn storage(&self) -> &ArrayIteratorStorageIr {
        match &self.0 {
            ArrayDestructuringOperation::StepValue { storage, .. }
            | ArrayDestructuringOperation::Elision(storage)
            | ArrayDestructuringOperation::RestArray { storage, .. } => storage,
        }
    }

    pub fn kind(&self) -> ArrayDestructuringOperationKindIr {
        match self.use_view() {
            ArrayDestructuringOperationView::StepValue(_) => {
                ArrayDestructuringOperationKindIr::StepValue
            }
            ArrayDestructuringOperationView::Elision(_) => {
                ArrayDestructuringOperationKindIr::Elision
            }
            ArrayDestructuringOperationView::RestArray(_) => {
                ArrayDestructuringOperationKindIr::RestArray
            }
        }
    }

    pub fn result_binding(&self) -> Option<&OwnedEnvBindingIr> {
        match &self.0 {
            ArrayDestructuringOperation::StepValue { result, .. }
            | ArrayDestructuringOperation::RestArray { result, .. } => Some(result),
            ArrayDestructuringOperation::Elision(_) => None,
        }
    }

    fn publish_result(
        operation: ArrayValueOperation,
        storage: ArrayIteratorStorageIr,
        result: OwnedEnvBindingIr,
    ) -> (Vec<StatementIr>, TypedExpr) {
        let name = result.name.clone();
        let (operation, kind) = match operation {
            ArrayValueOperation::StepValue => (
                ArrayDestructuringOperation::StepValue { storage, result },
                ValueKind::Dynamic,
            ),
            ArrayValueOperation::RestArray => (
                ArrayDestructuringOperation::RestArray { storage, result },
                ValueKind::Array,
            ),
        };
        let read = TypedExpr::from_info(ValueInfo::new(kind), ExprIr::Identifier(name.clone()));
        (
            vec![
                StatementIr::Lexical {
                    mode: BindingMode::Let,
                    name,
                    init: TypedExpr::undefined(),
                },
                StatementIr::ArrayDestructuringOperation(Box::new(Self(operation))),
            ],
            read,
        )
    }
}
