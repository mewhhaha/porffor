//! Shared kernels retain the actual iterator record and caller authority.

use super::*;
use crate::runtime_helpers::{
    ArrayIteratorCloseArguments, ArrayIteratorCloseParameters, ArrayIteratorElisionArguments,
    ArrayIteratorElisionParameters, ArrayIteratorStepValueArguments,
    ArrayIteratorStepValueParameters, HelperParameters,
};

impl FunctionBuilder<'_> {
    pub(crate) fn emit_sync_iterator_close(
        &mut self,
        iterator: &OwnedSyncIterator,
        pending: &CompletionLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if matches!(iterator.consumer, SyncIteratorConsumer::ArrayDestructuring) {
            return self.emit_array_iterator_close_call(iterator, pending, result, function);
        }
        self.emit_sync_iterator_close_inline(iterator, pending, result, function)
    }

    pub(super) fn emit_sync_iterator_step_into(
        &mut self,
        iterator: &OwnedSyncIterator,
        done: I32Local,
        value: Option<&ValueLocals>,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if matches!(iterator.consumer, SyncIteratorConsumer::ArrayDestructuring) {
            return self.emit_array_iterator_step_call(iterator, done, value, result, function);
        }
        self.emit_sync_iterator_step_inline_into(iterator, done, value, result, function)
    }

    pub(super) fn emit_array_iterator_step_call(
        &mut self,
        iterator: &OwnedSyncIterator,
        done: I32Local,
        value: Option<&ValueLocals>,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        // Preserve the inline kernel's error-Realm resolution before the call.
        let realm = self.emit_execution_realm(function);
        let base = self.runtime_helper_base()?;
        match value {
            Some(_) => schema.call_helper(
                ArrayIteratorStepValueArguments::new(
                    iterator.record(),
                    &realm,
                    self.current_environment(),
                    self.current_program(),
                ),
                base,
                function,
            ),
            None => schema.call_helper(
                ArrayIteratorElisionArguments::new(
                    iterator.record(),
                    &realm,
                    self.current_environment(),
                    self.current_program(),
                ),
                base,
                function,
            ),
        }
        .store(result, function);
        realm.clear(function);
        schema
            .struct_type::<IteratorRecord>()
            .field(IteratorRecordSchema::DONE)
            .read(iterator.record(), schema, function)
            .store(done, function);
        if let Some(value) = value {
            // A normal exhausted step can return its original `done` property
            // value. Keep the pattern's initialized undefined in that case.
            done.load(function);
            function.instruction(&Instruction::I32Eqz);
            result.kind().load(function);
            function.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
            function.instruction(&Instruction::I32Eq);
            function.instruction(&Instruction::I32And);
            self.open_frame(ControlFrameKind::If, function);
            value.copy_from(result.value(), function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        // `_into` callers finalize retained state before propagating Throw.
        Ok(())
    }

    pub(super) fn emit_array_iterator_close_call(
        &mut self,
        iterator: &OwnedSyncIterator,
        pending: &CompletionLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let realm = self.emit_execution_realm(function);
        schema
            .call_helper(
                ArrayIteratorCloseArguments::new(
                    iterator.record(),
                    pending,
                    &realm,
                    self.current_environment(),
                    self.current_program(),
                ),
                self.runtime_helper_base()?,
                function,
            )
            .store(result, function);
        realm.clear(function);
        Ok(())
    }

    fn own_array_iterator_parameter(
        &self,
        record: &GcLocal<IteratorRecord>,
        function: &mut Function,
    ) -> OwnedSyncIterator {
        let schema = self.runtime_schema();
        OwnedSyncIterator {
            record: schema
                .reserve_gc_local(function)
                .initialize(record.load(schema, function), function),
            consumer: SyncIteratorConsumer::ArrayDestructuring,
        }
    }

    pub(crate) fn compile_array_iterator_step_value_helper(
        &mut self,
    ) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::ArrayIteratorStepValue);
        let parameters = self.helper_parameters::<ArrayIteratorStepValueParameters>(&mut function);
        let schema = self.runtime_schema();
        let iterator = self.own_array_iterator_parameter(&parameters.iterator, &mut function);
        let done = schema.reserve_i32_local(&mut function);
        let value = schema.reserve_value_local(&mut function);
        value.set_undefined(&mut function);
        let result = schema.reserve_completion(&mut function);
        self.emit_sync_iterator_step_inline_into(
            &iterator,
            done,
            Some(&value),
            &result,
            &mut function,
        )?;
        result.emit(&mut function);
        result.clear(&mut function);
        value.clear(&mut function);
        schema.release_i32_local(done, &mut function);
        iterator.clear(&mut function);
        self.clear_helper_execution_realm(&mut function);
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    pub(crate) fn compile_array_iterator_elision_helper(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::ArrayIteratorElision);
        let parameters = self.helper_parameters::<ArrayIteratorElisionParameters>(&mut function);
        let schema = self.runtime_schema();
        let iterator = self.own_array_iterator_parameter(&parameters.iterator, &mut function);
        let done = schema.reserve_i32_local(&mut function);
        let result = schema.reserve_completion(&mut function);
        self.emit_sync_iterator_step_inline_into(&iterator, done, None, &result, &mut function)?;
        result.emit(&mut function);
        result.clear(&mut function);
        schema.release_i32_local(done, &mut function);
        iterator.clear(&mut function);
        self.clear_helper_execution_realm(&mut function);
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    pub(crate) fn compile_array_iterator_close_helper(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::ArrayIteratorClose);
        let parameters = self.helper_parameters::<ArrayIteratorCloseParameters>(&mut function);
        let schema = self.runtime_schema();
        let iterator = self.own_array_iterator_parameter(&parameters.iterator, &mut function);
        let result = schema.reserve_completion(&mut function);
        self.emit_sync_iterator_close_inline(
            &iterator,
            &parameters.pending,
            &result,
            &mut function,
        )?;
        result.emit(&mut function);
        result.clear(&mut function);
        iterator.clear(&mut function);
        self.clear_helper_execution_realm(&mut function);
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }
}
