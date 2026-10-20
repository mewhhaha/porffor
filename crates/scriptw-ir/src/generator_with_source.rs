//! One complete source walk owns the outer head and original With body.

use super::*;
use crate::generator_loop_control::{checked_next_state, GeneratorLoopSourceRange};
use crate::generator_loop_source::append_classic_generator_statement;
use boa_ast::statement::With;

pub(crate) struct GeneratorWithSource<'ast> {
    source: &'ast With,
}

/// The head finishes before the Object Environment Record exists. Even eager
/// With statements have a distinct body entry and cleanup exit.
pub(crate) struct GeneratorWithSourceStates {
    head: GeneratorLoopSourceRange,
    body: GeneratorLoopSourceRange,
    exit: u32,
    suspensions: Vec<GeneratorSuspensionPointIr>,
}

impl GeneratorWithSourceStates {
    pub(crate) fn entry(&self) -> u32 {
        self.head.entry
    }

    pub(crate) fn head(&self) -> GeneratorLoopSourceRange {
        self.head
    }

    pub(crate) fn body(&self) -> GeneratorLoopSourceRange {
        self.body
    }

    pub(crate) fn exit(&self) -> u32 {
        self.exit
    }

    pub(crate) fn suspensions(&self) -> &[GeneratorSuspensionPointIr] {
        &self.suspensions
    }
}

impl<'ast> GeneratorWithSource<'ast> {
    pub(crate) fn new(source: &'ast With) -> Option<Self> {
        let checked = Self { source };
        checked.states(0)?;
        Some(checked)
    }

    pub(crate) fn source(&self) -> &'ast With {
        self.source
    }

    pub(crate) fn states(&self, entry: u32) -> Option<GeneratorWithSourceStates> {
        if contains(self.source, ContainsSymbol::AwaitExpression) {
            return None;
        }
        let mut cursor = entry;
        let mut suspensions = Vec::new();
        GeneratorExpressionSourcePlan::new(
            self.source.expression(),
            GeneratorValueBranchAdmission::OrdinaryOutsideLoops,
        )?
        .append(&mut cursor, &mut suspensions)?;
        let head = GeneratorLoopSourceRange { entry, end: cursor };
        cursor = checked_next_state(cursor).ok()?;
        let body_entry = cursor;
        append_classic_generator_statement(self.source.statement(), &mut cursor, &mut suspensions)?;
        let body = GeneratorLoopSourceRange {
            entry: body_entry,
            end: cursor,
        };
        let exit = checked_next_state(cursor).ok()?;
        // The containing function must still have a normal terminal state.
        checked_next_state(exit).ok()?;
        Some(GeneratorWithSourceStates {
            head,
            body,
            exit,
            suspensions,
        })
    }

    pub(crate) fn append(
        &self,
        cursor: &mut u32,
        points: &mut Vec<GeneratorSuspensionPointIr>,
    ) -> Option<()> {
        let states = self.states(*cursor)?;
        *cursor = states.exit;
        points.extend(states.suspensions);
        Some(())
    }
}
