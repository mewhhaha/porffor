//! A checked plain-Async array body retains one synchronous IteratorRecord.

use crate::lowering_helpers::AsyncArrayPatternSourceStates;
use crate::{
    ArrayDestructuringOperationKindIr, ArrayIteratorStorageIr, BlockIr, ExprIr, OwnedEnvBindingIr,
    StatementIr, TypedExpr,
};
use std::collections::BTreeSet;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AsyncArrayDestructuringError {
    InvalidStates,
    UnallocatedSource,
    UnallocatedPatternCell,
    AliasedPatternCell,
    ForeignIteratorOperation,
    ForeignBody,
    UnconsumedSourceAwait,
    UnconsumedSourceSuspension,
    UnconsumedSourceOperation,
}

/// Acquisition precedes this body's close scope. The complete body includes
/// both fresh and resumed execution; rejected Await reaches its same close.
#[must_use = "the complete async pattern must own its IteratorClose scope"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncFunctionArrayDestructuringIr {
    entry_state: u32,
    body_entry_state: u32,
    body_end_state: u32,
    exit_state: u32,
    raw_source: TypedExpr,
    raw_binding: OwnedEnvBindingIr,
    storage: ArrayIteratorStorageIr,
    body: BlockIr,
    contains_await: bool,
}

impl AsyncFunctionArrayDestructuringIr {
    pub(crate) fn new(
        states: AsyncArrayPatternSourceStates,
        raw_source: TypedExpr,
        storage: ArrayIteratorStorageIr,
        body: BlockIr,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<Self, AsyncArrayDestructuringError> {
        if states.entry().checked_add(1) != Some(states.body_entry())
            || states.body_end() < states.body_entry()
            || states.body_end().checked_add(1) != Some(states.exit())
            || body.lexical_environment.is_some()
        {
            return Err(AsyncArrayDestructuringError::InvalidStates);
        }
        let ExprIr::Identifier(name) = &raw_source.expr else {
            return Err(AsyncArrayDestructuringError::UnallocatedSource);
        };
        let raw_binding = inventory
            .iter()
            .find(|row| &row.name == name)
            .ok_or(AsyncArrayDestructuringError::UnallocatedSource)?;
        let mut census = PatternCells::new(inventory);
        census.insert(raw_binding)?;
        census.insert(storage.binding())?;
        let mut awaits = Vec::new();
        let mut operations = Vec::new();
        let end = validate_body(
            &body.statements,
            states.body_entry(),
            &storage,
            &mut census,
            &mut awaits,
            &mut operations,
        )?;
        if end != states.body_end() {
            return Err(AsyncArrayDestructuringError::InvalidStates);
        }
        if awaits != states.awaits() {
            return Err(AsyncArrayDestructuringError::UnconsumedSourceAwait);
        }
        if operations != states.operations() {
            return Err(AsyncArrayDestructuringError::UnconsumedSourceOperation);
        }
        Ok(Self {
            entry_state: states.entry(),
            body_entry_state: states.body_entry(),
            body_end_state: states.body_end(),
            exit_state: states.exit(),
            raw_source,
            raw_binding: raw_binding.clone(),
            storage,
            body,
            contains_await: !awaits.is_empty(),
        })
    }

    pub const fn entry_state(&self) -> u32 {
        self.entry_state
    }
    pub const fn body_entry_state(&self) -> u32 {
        self.body_entry_state
    }
    pub const fn body_end_state(&self) -> u32 {
        self.body_end_state
    }
    pub const fn exit_state(&self) -> u32 {
        self.exit_state
    }
    pub fn raw_source(&self) -> &TypedExpr {
        &self.raw_source
    }
    pub fn storage(&self) -> &ArrayIteratorStorageIr {
        &self.storage
    }
    pub fn body(&self) -> &BlockIr {
        &self.body
    }
    pub const fn contains_await(&self) -> bool {
        self.contains_await
    }
}

/// Names and slots are independent identity domains. A completed nested owner
/// still participates in the parent's complete alias census.
pub(crate) struct PatternCells<'a> {
    inventory: &'a [OwnedEnvBindingIr],
    names: BTreeSet<String>,
    slots: BTreeSet<u32>,
}
impl<'a> PatternCells<'a> {
    pub(crate) fn new(inventory: &'a [OwnedEnvBindingIr]) -> Self {
        Self {
            inventory,
            names: BTreeSet::new(),
            slots: BTreeSet::new(),
        }
    }
    pub(crate) fn insert(
        &mut self,
        binding: &OwnedEnvBindingIr,
    ) -> Result<(), AsyncArrayDestructuringError> {
        let mut rows = self
            .inventory
            .iter()
            .filter(|row| row.name == binding.name || row.slot == binding.slot);
        if rows.next() != Some(binding) || rows.next().is_some() {
            return Err(AsyncArrayDestructuringError::UnallocatedPatternCell);
        }
        if !self.names.insert(binding.name.clone()) || !self.slots.insert(binding.slot) {
            return Err(AsyncArrayDestructuringError::AliasedPatternCell);
        }
        Ok(())
    }
}

/// Array operations are admitted only with their concrete checked storage in
/// this structural scope. The general Async state reader keeps refusing bare
/// operations. This fold reads the actual body; it constructs no shadow IR.
fn validate_body(
    statements: &[StatementIr],
    mut state: u32,
    storage: &ArrayIteratorStorageIr,
    census: &mut PatternCells<'_>,
    awaits: &mut Vec<(u32, u32)>,
    operations: &mut Vec<(ArrayDestructuringOperationKindIr, u32)>,
) -> Result<u32, AsyncArrayDestructuringError> {
    for statement in statements {
        state = match statement {
            StatementIr::ArrayDestructuringOperation(operation) => {
                if operation.storage() != storage {
                    return Err(AsyncArrayDestructuringError::ForeignIteratorOperation);
                }
                operations.push((operation.kind(), state));
                if let Some(result) = operation.result_binding() {
                    census.insert(result)?;
                }
                state
            }
            StatementIr::AsyncAwait {
                suspend_state,
                resume_state,
                ..
            } => {
                if *suspend_state != state || suspend_state.checked_add(1) != Some(*resume_state) {
                    return Err(AsyncArrayDestructuringError::InvalidStates);
                }
                awaits.push((*suspend_state, *resume_state));
                *resume_state
            }
            StatementIr::AsyncFunctionArrayDestructuring(child) => {
                if child.entry_state() != state {
                    return Err(AsyncArrayDestructuringError::InvalidStates);
                }
                census.insert(&child.raw_binding)?;
                census.insert(child.storage().binding())?;
                let mut child_operations = Vec::new();
                let end = validate_body(
                    &child.body().statements,
                    child.body_entry_state(),
                    child.storage(),
                    census,
                    awaits,
                    &mut child_operations,
                )?;
                if end != child.body_end_state() {
                    return Err(AsyncArrayDestructuringError::InvalidStates);
                }
                child.exit_state()
            }
            StatementIr::AsyncFunctionIf {
                plan,
                then_branch,
                else_branch,
                ..
            } => {
                if plan.entry_state() != state
                    || state.checked_add(1) != Some(plan.then_entry_state())
                {
                    return Err(AsyncArrayDestructuringError::InvalidStates);
                }
                let then_end = validate_body(
                    std::slice::from_ref(then_branch.as_ref()),
                    plan.then_entry_state(),
                    storage,
                    census,
                    awaits,
                    operations,
                )?;
                if then_end.checked_add(1) != Some(plan.else_entry_state()) {
                    return Err(AsyncArrayDestructuringError::InvalidStates);
                }
                let else_end = match else_branch {
                    Some(branch) => validate_body(
                        std::slice::from_ref(branch.as_ref()),
                        plan.else_entry_state(),
                        storage,
                        census,
                        awaits,
                        operations,
                    )?,
                    None => plan.else_entry_state(),
                };
                if else_end.checked_add(1) != Some(plan.exit_state()) {
                    return Err(AsyncArrayDestructuringError::InvalidStates);
                }
                plan.exit_state()
            }
            StatementIr::If {
                then_branch,
                else_branch,
                ..
            } => {
                let await_count = awaits.len();
                let operation_count = operations.len();
                for branch in Some(then_branch.as_ref())
                    .into_iter()
                    .chain(else_branch.as_deref())
                {
                    let end = validate_body(
                        std::slice::from_ref(branch),
                        state,
                        storage,
                        census,
                        awaits,
                        operations,
                    )?;
                    if end != state
                        || awaits.len() != await_count
                        || operations.len() != operation_count
                    {
                        return Err(AsyncArrayDestructuringError::InvalidStates);
                    }
                }
                state
            }
            StatementIr::Block(block) => {
                if block.lexical_environment.is_some() {
                    return Err(AsyncArrayDestructuringError::ForeignBody);
                }
                validate_body(
                    &block.statements,
                    state,
                    storage,
                    census,
                    awaits,
                    operations,
                )?
            }
            StatementIr::LexicalBlock(statements) => {
                validate_body(statements, state, storage, census, awaits, operations)?
            }
            StatementIr::EmptyStatementCompletion(item) => validate_body(
                std::slice::from_ref(item.statement()),
                state,
                storage,
                census,
                awaits,
                operations,
            )?,
            StatementIr::ResumableClassDefinition(plan) => {
                for prefix in plan.prefixes() {
                    let end = validate_body(
                        prefix.statements(),
                        prefix.entry_state(),
                        storage,
                        census,
                        awaits,
                        operations,
                    )?;
                    if end != prefix.exit_state() {
                        return Err(AsyncArrayDestructuringError::InvalidStates);
                    }
                }
                crate::async_switch::sequence_exit(std::slice::from_ref(statement), state)
                    .map_err(|_| AsyncArrayDestructuringError::InvalidStates)?
            }
            StatementIr::Empty
            | StatementIr::Debugger
            | StatementIr::Lexical { .. }
            | StatementIr::DeclarationEvaluation(_)
            | StatementIr::Expression(_) => state,
            StatementIr::ModuleImportBinding(_)
            | StatementIr::AnnexBFunctionCopy { .. }
            | StatementIr::Var(_)
            | StatementIr::Throw(_)
            | StatementIr::Return(_)
            | StatementIr::ModuleUnitOnce { .. }
            | StatementIr::SyncDisposableScope { .. }
            | StatementIr::AsyncDisposableScope { .. }
            | StatementIr::AsyncModuleInstantiation
            | StatementIr::AsyncGeneratorLoop(_)
            | StatementIr::AsyncGeneratorIf(_)
            | StatementIr::AsyncGeneratorWith(_)
            | StatementIr::AsyncGeneratorSwitch(_)
            | StatementIr::AsyncGeneratorArrayDestructuring(_)
            | StatementIr::AsyncGeneratorResourceScope(_)
            | StatementIr::AsyncGeneratorResourceRegistration(_)
            | StatementIr::AsyncGeneratorForOf(_)
            | StatementIr::AsyncGeneratorForIn(_)
            | StatementIr::OrdinaryGeneratorLoop(_)
            | StatementIr::OrdinaryGeneratorIf(_)
            | StatementIr::OrdinaryGeneratorSwitch(_)
            | StatementIr::OrdinaryGeneratorWith(_)
            | StatementIr::AsyncFunctionWith(_)
            | StatementIr::OrdinaryGeneratorArrayDestructuring(_)
            | StatementIr::GeneratorYield { .. }
            | StatementIr::GeneratorIf { .. }
            | StatementIr::GeneratorLoop { .. }
            | StatementIr::AsyncFunctionWhile(_)
            | StatementIr::AsyncFunctionSwitch(_)
            | StatementIr::AsyncFunctionForOfIterator { .. }
            | StatementIr::GeneratorForOfIterator { .. }
            | StatementIr::While { .. }
            | StatementIr::DoWhile { .. }
            | StatementIr::For { .. }
            | StatementIr::ForOfIterator { .. }
            | StatementIr::ForInArray { .. }
            | StatementIr::ForInString { .. }
            | StatementIr::ForInObject { .. }
            | StatementIr::Switch { .. }
            | StatementIr::TryCatch { .. }
            | StatementIr::TryFinally { .. }
            | StatementIr::TryCatchFinally { .. }
            | StatementIr::Labelled { .. }
            | StatementIr::Break { .. }
            | StatementIr::Continue { .. }
            | StatementIr::ParameterInitialization { .. } => {
                return Err(AsyncArrayDestructuringError::ForeignBody)
            }
        };
    }
    Ok(state)
}
