//! A complete suspended array pattern owns IteratorClose on every entry.

use crate::generator_loop_control::{collect_suspensions, sequence_end};
use crate::lowering_helpers::GeneratorArrayPatternSourceStates;
use crate::{
    ArrayDestructuringOperationKindIr, ArrayIteratorStorageIr, ExprIr, GeneratorLoopRegionIr,
    OwnedEnvBindingIr, StatementIr, TypedExpr,
};
use std::collections::BTreeSet;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GeneratorArrayDestructuringError {
    InvalidStates,
    UnallocatedSource,
    AliasedIteratorStorage,
    ForeignIteratorOperation,
    AliasedPatternCell,
    UnallocatedPatternCell,
    ForeignBody,
    UnconsumedSourceSuspension,
    UnconsumedSourceOperation,
}

/// Acquisition occurs outside this body's close scope. Fresh and resumed body
/// entry rebuild that scope around all targets, defaults and nested patterns.
#[must_use = "the complete IteratorClose owner must be attached to its statement"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrdinaryGeneratorArrayDestructuringIr {
    entry_state: u32,
    raw_source: TypedExpr,
    raw_binding: OwnedEnvBindingIr,
    storage: ArrayIteratorStorageIr,
    body: GeneratorLoopRegionIr,
    exit_state: u32,
}

impl OrdinaryGeneratorArrayDestructuringIr {
    pub(crate) fn new(
        states: GeneratorArrayPatternSourceStates,
        raw_source: TypedExpr,
        storage: ArrayIteratorStorageIr,
        body: GeneratorLoopRegionIr,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<Self, GeneratorArrayDestructuringError> {
        let range = states.body();
        if states.entry().checked_add(1) != Some(range.entry)
            || range.end.checked_add(1) != Some(states.exit())
            || body.entry_state() != range.entry
            || body.end_state() != range.end
            || body.block().lexical_environment.is_some()
        {
            return Err(GeneratorArrayDestructuringError::InvalidStates);
        }
        let ExprIr::Identifier(source_name) = &raw_source.expr else {
            return Err(GeneratorArrayDestructuringError::UnallocatedSource);
        };
        let mut sources = inventory.iter().filter(|row| &row.name == source_name);
        let source = sources
            .next()
            .ok_or(GeneratorArrayDestructuringError::UnallocatedSource)?;
        if sources.next().is_some()
            || inventory
                .iter()
                .filter(|row| row.name == source.name || row.slot == source.slot)
                .count()
                != 1
        {
            return Err(GeneratorArrayDestructuringError::UnallocatedSource);
        }
        let iterator = storage.binding();
        if source.name == iterator.name || source.slot == iterator.slot {
            return Err(GeneratorArrayDestructuringError::AliasedIteratorStorage);
        }
        let mut iterators = inventory
            .iter()
            .filter(|row| row.name == iterator.name || row.slot == iterator.slot);
        if iterators.next() != Some(iterator) || iterators.next().is_some() {
            return Err(GeneratorArrayDestructuringError::AliasedIteratorStorage);
        }
        let mut points = Vec::new();
        collect_suspensions(&body.block().statements, &mut points);
        if points != states.suspensions() {
            return Err(GeneratorArrayDestructuringError::UnconsumedSourceSuspension);
        }
        let mut names = BTreeSet::from([source.name.as_str(), iterator.name.as_str()]);
        let mut slots = BTreeSet::from([source.slot, iterator.slot]);
        let mut operations = Vec::new();
        validate_body(
            &body.block().statements,
            range.entry,
            &storage,
            &mut names,
            &mut slots,
            &mut operations,
            inventory,
        )?;
        if operations != states.operations() {
            return Err(GeneratorArrayDestructuringError::UnconsumedSourceOperation);
        }
        Ok(Self {
            entry_state: states.entry(),
            raw_source,
            raw_binding: source.clone(),
            storage,
            body,
            exit_state: states.exit(),
        })
    }

    pub const fn entry_state(&self) -> u32 {
        self.entry_state
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
    pub fn body(&self) -> &GeneratorLoopRegionIr {
        &self.body
    }
}

/// Pattern publication is a closed statement domain. Its source compiler emits
/// iterator operation statements into acquisition cells. The expression domain
/// cannot hold an iterator operation. Nested patterns own distinct native records.
fn validate_body<'a>(
    statements: &'a [StatementIr],
    mut state: u32,
    storage: &'a ArrayIteratorStorageIr,
    names: &mut BTreeSet<&'a str>,
    slots: &mut BTreeSet<u32>,
    operations: &mut Vec<(ArrayDestructuringOperationKindIr, u32)>,
    inventory: &[OwnedEnvBindingIr],
) -> Result<(), GeneratorArrayDestructuringError> {
    for statement in statements {
        match statement {
            StatementIr::EmptyStatementCompletion(item) => {
                validate_body(
                    std::slice::from_ref(item.statement()),
                    state,
                    storage,
                    names,
                    slots,
                    operations,
                    inventory,
                )?;
            }
            StatementIr::ArrayDestructuringOperation(operation) => {
                if operation.storage() != storage {
                    return Err(GeneratorArrayDestructuringError::ForeignIteratorOperation);
                }
                operations.push((operation.kind(), state));
                if let Some(result) = operation.result_binding() {
                    require_allocated(result, inventory)?;
                    if !names.insert(&result.name) || !slots.insert(result.slot) {
                        return Err(GeneratorArrayDestructuringError::AliasedPatternCell);
                    }
                }
            }
            StatementIr::Lexical { .. }
            | StatementIr::DeclarationEvaluation(_)
            | StatementIr::Expression(_)
            | StatementIr::GeneratorYield { .. } => {}
            StatementIr::Block(block) => {
                validate_body(
                    &block.statements,
                    state,
                    storage,
                    names,
                    slots,
                    operations,
                    inventory,
                )?;
            }
            StatementIr::LexicalBlock(statements)
            | StatementIr::ParameterInitialization { statements, .. } => {
                validate_body(
                    statements, state, storage, names, slots, operations, inventory,
                )?;
            }
            StatementIr::OrdinaryGeneratorIf(plan) => {
                for region in [plan.then_branch(), plan.else_branch()] {
                    validate_body(
                        &region.block().statements,
                        region.entry_state(),
                        storage,
                        names,
                        slots,
                        operations,
                        inventory,
                    )?;
                }
            }
            StatementIr::OrdinaryGeneratorArrayDestructuring(plan) => {
                let nested = plan.storage().binding();
                require_allocated(nested, inventory)?;
                require_allocated(&plan.raw_binding, inventory)?;
                if !names.insert(&nested.name)
                    || !slots.insert(nested.slot)
                    || !names.insert(&plan.raw_binding.name)
                    || !slots.insert(plan.raw_binding.slot)
                {
                    return Err(GeneratorArrayDestructuringError::AliasedIteratorStorage);
                }
                // The immutable child has already consumed its own source tape.
                // Its cells still participate in the complete parent's alias census.
                let mut nested_operations = Vec::new();
                validate_body(
                    &plan.body().block().statements,
                    plan.body().entry_state(),
                    plan.storage(),
                    names,
                    slots,
                    &mut nested_operations,
                    inventory,
                )?;
            }
            StatementIr::ResumableClassDefinition(plan) => {
                for prefix in plan.prefixes() {
                    validate_body(
                        prefix.statements(),
                        prefix.entry_state(),
                        storage,
                        names,
                        slots,
                        operations,
                        inventory,
                    )?;
                }
            }
            StatementIr::Empty | StatementIr::Debugger => {}
            StatementIr::ModuleImportBinding(_)
            | StatementIr::AnnexBFunctionCopy { .. }
            | StatementIr::Var(_)
            | StatementIr::Throw(_)
            | StatementIr::Return(_)
            | StatementIr::ModuleUnitOnce { .. }
            | StatementIr::SyncDisposableScope { .. }
            | StatementIr::AsyncDisposableScope { .. }
            | StatementIr::AsyncAwait { .. }
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
            | StatementIr::OrdinaryGeneratorSwitch(_)
            | StatementIr::OrdinaryGeneratorWith(_)
            | StatementIr::GeneratorIf { .. }
            | StatementIr::GeneratorLoop { .. }
            | StatementIr::AsyncFunctionIf { .. }
            | StatementIr::AsyncFunctionArrayDestructuring(_)
            | StatementIr::AsyncFunctionWith(_)
            | StatementIr::AsyncFunctionWhile(_)
            | StatementIr::AsyncFunctionSwitch(_)
            | StatementIr::AsyncFunctionForOfIterator { .. }
            | StatementIr::GeneratorForOfIterator { .. }
            | StatementIr::If { .. }
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
            | StatementIr::Continue { .. } => {
                return Err(GeneratorArrayDestructuringError::ForeignBody);
            }
        }
        state = sequence_end(std::slice::from_ref(statement), state)
            .map_err(|_| GeneratorArrayDestructuringError::InvalidStates)?;
    }
    Ok(())
}

fn require_allocated(
    binding: &OwnedEnvBindingIr,
    inventory: &[OwnedEnvBindingIr],
) -> Result<(), GeneratorArrayDestructuringError> {
    let mut matches = inventory
        .iter()
        .filter(|row| row.name == binding.name || row.slot == binding.slot);
    if matches.next() != Some(binding) || matches.next().is_some() {
        return Err(GeneratorArrayDestructuringError::UnallocatedPatternCell);
    }
    Ok(())
}
