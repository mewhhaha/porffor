//! IteratorClose observes return while preserving the original completion.

use super::*;
use crate::gc_types::{CompletionLocals, ValueLocals};

impl FunctionBuilder<'_> {
    pub(crate) fn emit_iterator_close_with_completion(
        &mut self,
        iterator: &ValueLocals,
        pending: &CompletionLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let original = schema.reserve_completion(function);
        original.copy_from(pending, function);
        result.copy_from(&original, function);
        let inner = schema.reserve_completion(function);
        let method = schema.reserve_value_local(function);
        let key = self.emit_function_string_key("return", function)?;
        let arguments = self.emit_pre_evaluated_arg_vector(&[], function);
        let finish = self.open_frame(ControlFrameKind::Block, function);
        let inner_finished = self.open_frame(ControlFrameKind::Block, function);
        self.emit_dynamic_property_read_with_key_locals(
            iterator, iterator, &key, &inner, function,
        )?;
        inner.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_branch_to_target(inner_finished, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.compile_nullish_tagged_i32(inner.value().tag(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        // GetMethod treats both undefined and null as an absent method.
        self.emit_branch_to_target(finish, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        method.copy_from(inner.value(), function);
        self.emit_is_callable_i32(&method, function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            NativeErrorKind::TypeError,
            RuntimeErrorMessage::ITERATORCLOSE_RETURN_METHOD_MUST_BE_CALLABLE,
            &inner,
            function,
        )?;
        self.emit_branch_to_target(inner_finished, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_function_or_proxy_call_with_argv(
            &method, iterator, &arguments, &inner, function,
        )?;
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        // All Get/Call effects occur even when their error will lose to an
        // incoming Throw. The original value, kind and target are retained.
        original.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_branch_to_target(finish, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        inner.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        result.copy_from(&inner, function);
        self.emit_branch_to_target(finish, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_object_value_i32(inner.value(), function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            NativeErrorKind::TypeError,
            RuntimeErrorMessage::ITERATORCLOSE_RETURN_RESULT_MUST_BE_OBJECT,
            result,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        arguments.clear(function);
        key.clear(function);
        method.clear(function);
        inner.clear(function);
        original.clear(function);
        Ok(())
    }
}
