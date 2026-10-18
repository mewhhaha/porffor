//! Bounded code for repeated live identifier reads in ordinary source bodies.
use super::*;
use std::num::NonZeroU32;

const MIN_READS: usize = 64;

/// One complete, non-suspending ExpressionStatement is evaluated on every
/// iteration. Matching the entire typed expression retains its resolution
/// start, strictness, reference operation and value knowledge.
pub(super) struct IdentifierReadRun<'a> {
    statement: &'a StatementIr,
    count: NonZeroU32,
}

impl<'a> IdentifierReadRun<'a> {
    pub(super) fn of_prefix(statements: &'a [StatementIr]) -> Option<Self> {
        let first = statements.first()?;
        let StatementIr::Expression(expression) = first else {
            return None;
        };
        let read = match &expression.expr {
            ExprIr::Identifier(_)
            | ExprIr::GlobalIdentifierRead { .. }
            | ExprIr::GlobalPropertyRead { .. } => true,
            ExprIr::EnvironmentIdentifier(identifier) => matches!(
                identifier.operation,
                lila_ir::EnvironmentIdentifierOperationIr::Read
            ),
            _ => false,
        };
        if !read {
            return None;
        }
        let count = statements.iter().take_while(|next| *next == first).count();
        if count < MIN_READS {
            return None;
        }
        Some(Self {
            statement: first,
            count: NonZeroU32::new(u32::try_from(count).ok()?)?,
        })
    }

    pub(super) fn len(&self) -> usize {
        self.count.get() as usize
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn compile_identifier_read_run(
        &mut self,
        run: &IdentifierReadRun<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let remaining = schema.reserve_i32_local(function);
        remaining.set_constant(run.count.get() as i32, function);
        let next = self.open_frame(ControlFrameKind::Loop, function);
        // ResolveBinding/GetValue, completion publication and abrupt dispatch
        // remain the ordinary statement path. Getters and with environments
        // may change the binding, return value or throw on any iteration.
        self.compile_statement(run.statement, function)?;
        remaining.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Sub);
        remaining.store(function);
        remaining.load(function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_branch_to_target(next, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        schema.release_i32_local(remaining, function);
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/repeated_identifier_reads.rs"]
mod tests;
