//! A mixed pattern owns its synchronous IteratorRecord across Await and Yield.

use crate::async_array_destructuring::{AsyncArrayDestructuringError, PatternCells};
use crate::async_generator_loop_control::validate_tape;
use crate::async_generator_source::{
    AsyncGeneratorArrayPatternSourceStates, AsyncGeneratorSourceRange,
};
use crate::generator_loop_control::{collect_mixed_suspensions, mixed_sequence_end};
use crate::{
    ArrayDestructuringOperationKindIr, ArrayIteratorStorageIr, AsyncGeneratorLoopRegionIr, BlockIr,
    ExprIr, OwnedEnvBindingIr, ResumableSuspensionPointIr, StatementIr, TypedExpr,
};

type Error = AsyncArrayDestructuringError;

/// Only the concrete Array constructor can produce this body. General mixed
/// regions cannot admit bare iterator operations.
pub(crate) struct CheckedArrayBody {
    block: BlockIr,
    range: AsyncGeneratorSourceRange,
}

impl CheckedArrayBody {
    pub(crate) fn into_parts(self) -> (BlockIr, AsyncGeneratorSourceRange) {
        (self.block, self.range)
    }
}

#[must_use = "the complete pattern must own its IteratorClose scope"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncGeneratorArrayDestructuringIr {
    entry_state: u32,
    exit_state: u32,
    raw_source: TypedExpr,
    raw_binding: OwnedEnvBindingIr,
    storage: ArrayIteratorStorageIr,
    body: AsyncGeneratorLoopRegionIr,
    suspensions: Vec<ResumableSuspensionPointIr>,
}

impl AsyncGeneratorArrayDestructuringIr {
    pub(crate) fn new(
        states: AsyncGeneratorArrayPatternSourceStates,
        raw_source: TypedExpr,
        storage: ArrayIteratorStorageIr,
        body: BlockIr,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<Self, Error> {
        let range = states.body();
        if states.entry().checked_add(1) != Some(range.entry())
            || range.end() < range.entry()
            || range.end().checked_add(1) != Some(states.exit())
            || states.exit().checked_add(1).is_none()
            || body.lexical_environment.is_some()
        {
            return Err(Error::InvalidStates);
        }
        let ExprIr::Identifier(name) = &raw_source.expr else {
            return Err(Error::UnallocatedSource);
        };
        let raw_binding = inventory
            .iter()
            .find(|row| &row.name == name)
            .ok_or(Error::UnallocatedSource)?;
        let mut cells = PatternCells::new(inventory);
        cells.insert(raw_binding)?;
        cells.insert(storage.binding())?;
        let mut operations = Vec::new();
        let end = validate_body(
            &body.statements,
            range.entry(),
            &storage,
            &mut cells,
            &mut operations,
        )?;
        if end != range.end() {
            return Err(Error::InvalidStates);
        }
        if operations != states.operations() {
            return Err(Error::UnconsumedSourceOperation);
        }
        let mut suspensions = Vec::new();
        collect_mixed_suspensions(&body.statements, &mut suspensions);
        validate_tape(states.suspensions(), &suspensions)
            .map_err(|_| Error::UnconsumedSourceSuspension)?;
        Ok(Self {
            entry_state: states.entry(),
            exit_state: states.exit(),
            raw_source,
            raw_binding: raw_binding.clone(),
            storage,
            body: AsyncGeneratorLoopRegionIr::from_array_pattern(CheckedArrayBody {
                block: body,
                range,
            }),
            suspensions,
        })
    }

    pub const fn entry_state(&self) -> u32 {
        self.entry_state
    }
    pub const fn body_entry_state(&self) -> u32 {
        self.body.entry_state()
    }
    pub const fn body_end_state(&self) -> u32 {
        self.body.end_state()
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
    pub fn body(&self) -> &AsyncGeneratorLoopRegionIr {
        &self.body
    }
    pub(crate) fn suspensions(&self) -> &[ResumableSuspensionPointIr] {
        &self.suspensions
    }
}

/// Read the actual body while carrying its concrete storage owner. Child
/// patterns contribute all their cells but keep their own operation tape.
fn validate_body(
    statements: &[StatementIr],
    mut state: u32,
    storage: &ArrayIteratorStorageIr,
    cells: &mut PatternCells<'_>,
    operations: &mut Vec<(ArrayDestructuringOperationKindIr, u32)>,
) -> Result<u32, Error> {
    for statement in statements {
        state = match statement {
            StatementIr::ArrayDestructuringOperation(operation) => {
                if operation.storage() != storage {
                    return Err(Error::ForeignIteratorOperation);
                }
                operations.push((operation.kind(), state));
                if let Some(binding) = operation.result_binding() {
                    cells.insert(binding)?;
                }
                state
            }
            StatementIr::AsyncGeneratorArrayDestructuring(child) => {
                if child.entry_state() != state {
                    return Err(Error::InvalidStates);
                }
                cells.insert(&child.raw_binding)?;
                cells.insert(child.storage().binding())?;
                let end = validate_body(
                    &child.body().block().statements,
                    child.body_entry_state(),
                    child.storage(),
                    cells,
                    &mut Vec::new(),
                )?;
                if end != child.body_end_state() {
                    return Err(Error::InvalidStates);
                }
                child.exit_state()
            }
            StatementIr::EmptyStatementCompletion(item) => validate_body(
                std::slice::from_ref(item.statement()),
                state,
                storage,
                cells,
                operations,
            )?,
            StatementIr::Block(block) => {
                if block.lexical_environment.is_some() {
                    return Err(Error::ForeignBody);
                }
                validate_body(&block.statements, state, storage, cells, operations)?
            }
            StatementIr::LexicalBlock(statements) => {
                validate_body(statements, state, storage, cells, operations)?
            }
            StatementIr::AsyncGeneratorIf(plan) => {
                for region in plan.regions() {
                    let end = validate_body(
                        &region.block().statements,
                        region.entry_state(),
                        storage,
                        cells,
                        operations,
                    )?;
                    if end != region.end_state() {
                        return Err(Error::InvalidStates);
                    }
                }
                mixed_end(statement, state)?
            }
            StatementIr::ResumableClassDefinition(plan) => {
                for prefix in plan.prefixes() {
                    let end = validate_body(
                        prefix.statements(),
                        prefix.entry_state(),
                        storage,
                        cells,
                        operations,
                    )?;
                    if end != prefix.exit_state() {
                        return Err(Error::InvalidStates);
                    }
                }
                mixed_end(statement, state)?
            }
            StatementIr::If {
                then_branch,
                else_branch,
                ..
            } => {
                let count = operations.len();
                for branch in Some(then_branch.as_ref())
                    .into_iter()
                    .chain(else_branch.as_deref())
                {
                    if validate_body(
                        std::slice::from_ref(branch),
                        state,
                        storage,
                        cells,
                        operations,
                    )? != state
                        || operations.len() != count
                    {
                        return Err(Error::InvalidStates);
                    }
                }
                mixed_end(statement, state)?
            }
            StatementIr::Empty
            | StatementIr::Debugger
            | StatementIr::Lexical { .. }
            | StatementIr::DeclarationEvaluation(_)
            | StatementIr::Expression(_)
            | StatementIr::GeneratorYield { .. }
            | StatementIr::AsyncAwait { .. } => mixed_end(statement, state)?,
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
            | StatementIr::AsyncGeneratorWith(_)
            | StatementIr::AsyncGeneratorSwitch(_)
            | StatementIr::AsyncGeneratorResourceScope(_)
            | StatementIr::AsyncGeneratorResourceRegistration(_)
            | StatementIr::AsyncGeneratorForOf(_)
            | StatementIr::AsyncGeneratorForIn(_)
            | StatementIr::OrdinaryGeneratorLoop(_)
            | StatementIr::OrdinaryGeneratorIf(_)
            | StatementIr::OrdinaryGeneratorSwitch(_)
            | StatementIr::OrdinaryGeneratorArrayDestructuring(_)
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
            | StatementIr::ParameterInitialization { .. } => return Err(Error::ForeignBody),
        };
    }
    Ok(state)
}

fn mixed_end(statement: &StatementIr, state: u32) -> Result<u32, Error> {
    mixed_sequence_end(std::slice::from_ref(statement), state).map_err(|_| Error::InvalidStates)
}

#[cfg(test)]
mod tests;
