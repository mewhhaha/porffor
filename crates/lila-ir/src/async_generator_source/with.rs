//! The original With head completes before its separately owned object record.

use super::*;
use boa_ast::statement::With;

pub(crate) struct AsyncGeneratorWithSourceStates {
    head: AsyncGeneratorSourceRange,
    body: AsyncGeneratorSourceRange,
    exit: u32,
    suspensions: Vec<ResumableSuspensionPointIr>,
}

impl AsyncGeneratorWithSourceStates {
    pub(crate) fn head(&self) -> AsyncGeneratorSourceRange {
        self.head
    }
    pub(crate) fn body(&self) -> AsyncGeneratorSourceRange {
        self.body
    }
    pub(crate) fn exit(&self) -> u32 {
        self.exit
    }
    pub(crate) fn suspensions(&self) -> &[ResumableSuspensionPointIr] {
        &self.suspensions
    }
}

#[derive(Clone, Copy)]
pub(crate) struct AsyncGeneratorWithSource<'ast> {
    source: &'ast With,
}

impl<'ast> AsyncGeneratorWithSource<'ast> {
    pub(crate) fn new(source: &'ast With) -> Option<Self> {
        // This shape proof follows the actual mixed region's foreign-owner
        // boundary. The complete state/tape walk remains the shared allocator.
        region::admits(source.statement()).then_some(())?;
        let owner = Self { source };
        owner.states(0)?;
        Some(owner)
    }
    pub(crate) fn source(self) -> &'ast With {
        self.source
    }
    pub(crate) fn states(self, entry: u32) -> Option<AsyncGeneratorWithSourceStates> {
        let mut states = ResumableStateAllocator::at(entry);
        let (head, body) = append_phases(self.source, &mut states).ok()?;
        let exit = states.current();
        let (suspensions, _) = states.into_tape();
        Some(AsyncGeneratorWithSourceStates {
            head,
            body,
            exit,
            suspensions,
        })
    }
}

pub(super) fn append(
    source: &With,
    states: &mut ResumableStateAllocator,
) -> Result<(), AsyncGeneratorSourceError> {
    if !region::admits(source.statement()) {
        return Err(AsyncGeneratorSourceError::ForeignContinuation);
    }
    append_phases(source, states).map(|_| ())
}

fn append_phases(
    source: &With,
    states: &mut ResumableStateAllocator,
) -> Result<(AsyncGeneratorSourceRange, AsyncGeneratorSourceRange), AsyncGeneratorSourceError> {
    let head = phase(states, |states| {
        expression::append(source.expression(), states)
    })?;
    let body = phase(states, |states| {
        states.with_enclosing_scope(|states| {
            statement::append(
                source.statement(),
                states,
                AsyncGeneratorSourceDomain::FunctionBody,
            )
        })
    })?;
    Ok((head, body))
}
