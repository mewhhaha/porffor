//! Iterator.from caches a genuine direct record before identity or wrapping.
use super::*;
use crate::gc_types::{
    IteratorFromWrapper, IteratorFromWrapperSchema, IteratorRecord, IteratorRecordSchema,
};

/// Only Object acquisition followed by the observable next Get mints this
/// completed record. Publication consumes it once for identity or wrapping.
#[must_use = "a completed Iterator.from record must be published"]
struct CompletedIteratorFromRecord {
    record: GcLocal<IteratorRecord>,
}

impl FunctionBuilder<'_> {
    pub(in crate::builtins) fn emit_iterator_from_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let result = schema.reserve_completion(function);
        result.initialize(function);
        let exit = self.open_frame(ControlFrameKind::Block, function);
        let record = self.emit_iterator_from_acquire(&result, exit, function)?;
        self.emit_iterator_from_publish(record, &result, exit, function)?;
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&result, function);
        result.clear(function);
        Ok(())
    }

    fn emit_iterator_from_acquire(
        &mut self,
        result: &CompletionLocals,
        exit: ControlTarget,
        function: &mut Function,
    ) -> Result<CompletedIteratorFromRecord, EmitError> {
        let schema = self.runtime_schema();
        let source = schema.reserve_value_local(function);
        let object = schema.reserve_value_local(function);
        let iterator = schema.reserve_value_local(function);
        let next = schema.reserve_value_local(function);
        let method = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        self.emit_builtin_arg_to_value(0, &source, function);
        self.emit_is_heap_object_like_tag_i32(source.tag(), function);
        source.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::String as i32));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        self.emit_native_iterator_error_if(
            RuntimeErrorMessage::ITERATOR_FROM_CALLED_ON_NULL_OR_UNDEFINED,
            result,
            exit,
            function,
        )?;

        // GetV boxes only the lookup base. GetMethod and Call retain the
        // original primitive String or Object receiver, with zero arguments.
        self.emit_value_to_current_function_realm_object_locals(&source, &pending, function)?;
        self.emit_native_iterator_abrupt_exit(&pending, result, exit, function);
        object.copy_from(pending.value(), function);
        let symbol = schema.reserve_gc_local(function).initialize(
            self.emit_well_known_symbol_reference(lila_ir::WellKnownSymbol::Iterator, function)?,
            function,
        );
        let key = PropertyKeyLocals::from_symbol(schema, &symbol, function);
        symbol.clear(function);
        self.emit_object_read(&object, &source, &key, &pending, function)?;
        key.clear(function);
        self.emit_native_iterator_abrupt_exit(&pending, result, exit, function);
        method.copy_from(pending.value(), function);
        iterator.copy_from(&source, function);
        self.compile_nullish_tagged_i32(method.tag(), function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_iterator_from_call(
            &method,
            &source,
            &pending,
            RuntimeErrorMessage::ITERATOR_FROM_ITERATOR_METHOD_MUST_BE_CALLABLE,
            function,
        )?;
        self.emit_native_iterator_abrupt_exit(&pending, result, exit, function);
        iterator.copy_from(pending.value(), function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // Both the nullish fallback and the method route check Object before
        // reading next. Callability of this cached next remains deferred.
        self.emit_is_heap_object_like_tag_i32(iterator.tag(), function);
        function.instruction(&Instruction::I32Eqz);
        self.emit_native_iterator_error_if(
            RuntimeErrorMessage::ITERATOR_FROM_ITERATOR_METHOD_MUST_RETURN_OBJECT,
            result,
            exit,
            function,
        )?;
        let key = self.emit_iterator_named_key("next", function)?;
        self.emit_object_read(&iterator, &iterator, &key, &pending, function)?;
        key.clear(function);
        self.emit_native_iterator_abrupt_exit(&pending, result, exit, function);
        next.copy_from(pending.value(), function);
        let iterator_value = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&iterator, function),
            function,
        );
        let next_value = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&next, function),
            function,
        );
        let record = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<IteratorRecord>().construct(
                (
                    GcOperand::reference(&iterator_value, schema),
                    GcOperand::reference(&next_value, schema),
                    GcOperand::boolean(false),
                ),
                function,
            ),
            function,
        );
        next_value.clear(function);
        iterator_value.clear(function);
        pending.clear(function);
        method.clear(function);
        next.clear(function);
        iterator.clear(function);
        object.clear(function);
        source.clear(function);
        Ok(CompletedIteratorFromRecord { record })
    }

    fn emit_iterator_from_publish(
        &mut self,
        completed: CompletedIteratorFromRecord,
        result: &CompletionLocals,
        exit: ControlTarget,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let CompletedIteratorFromRecord { record } = completed;
        let iterator = schema.reserve_value_local(function);
        let base = schema.reserve_value_local(function);
        let prototype = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        let should_wrap = schema.reserve_i32_local(function);
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .field(IteratorRecordSchema::ITERATOR)
                .read(&record, schema, function)
                .reference(),
            function,
        );
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored, &iterator, schema, function);
        stored.clear(function);
        let realm = schema
            .reserve_gc_local(function)
            .initialize(self.emit_current_function_realm(function), function);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::IteratorPrototype,
            &prototype,
            function,
        );
        base.copy_from(&iterator, function);
        function.instruction(&Instruction::I32Const(1));
        should_wrap.store(function);
        let finished = self.open_frame(ControlFrameKind::Block, function);
        let again = self.open_frame(ControlFrameKind::Loop, function);
        self.emit_object_get_prototype_of(&base, &pending, function)?;
        self.emit_native_iterator_abrupt_exit(&pending, result, exit, function);
        pending.value().tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Null as i32));
        function.instruction(&Instruction::I32Eq);
        self.emit_branch_if_to_target(finished, function);
        pending.value().reference().load(function);
        prototype.reference().load(function);
        function.instruction(&Instruction::RefEq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I32Const(0));
        should_wrap.store(function);
        self.emit_branch_to_target(finished, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        base.copy_from(pending.value(), function);
        self.emit_branch_to_target(again, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        result.set_normal(&iterator, function);
        should_wrap.load(function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::IteratorFromWrapperPrototype,
            &prototype,
            function,
        );
        let header = schema.reserve_gc_local(function).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(&prototype), function)?,
            function,
        );
        let wrapper = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<IteratorFromWrapper>().construct(
                (
                    GcOperand::reference(&header, schema),
                    GcOperand::reference(&record, schema),
                ),
                function,
            ),
            function,
        );
        base.set_reference(&wrapper, schema, function);
        result.set_normal(&base, function);
        wrapper.clear(function);
        header.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        realm.clear(function);
        schema.release_i32_local(should_wrap, function);
        pending.clear(function);
        prototype.clear(function);
        base.clear(function);
        iterator.clear(function);
        record.clear(function);
        Ok(())
    }

    /// The wrapper forwards Call directly, including primitive results.
    fn emit_iterator_from_call(
        &mut self,
        method: &ValueLocals,
        receiver: &ValueLocals,
        result: &CompletionLocals,
        message: RuntimeErrorMessage,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let exit = self.open_frame(ControlFrameKind::Block, function);
        self.emit_is_callable_i32(method, function)?;
        function.instruction(&Instruction::I32Eqz);
        self.emit_native_iterator_error_if(message, result, exit, function)?;
        let arguments = self.emit_pre_evaluated_arg_vector(&[], function);
        self.emit_function_or_proxy_call_with_argv(method, receiver, &arguments, result, function)?;
        arguments.clear(function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(in crate::builtins) fn emit_iterator_from_wrapper_return_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_iterator_from_wrapper_builtin(true, function)
    }

    pub(in crate::builtins) fn emit_iterator_from_wrapper_next_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_iterator_from_wrapper_builtin(false, function)
    }

    fn emit_iterator_from_wrapper_builtin(
        &mut self,
        returning: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        let iterator = schema.reserve_value_local(function);
        let method = schema.reserve_value_local(function);
        let result = schema.reserve_completion(function);
        result.initialize(function);
        receiver.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| {
                    EmitError::unsupported("Iterator.from wrapper requires its builtin entry")
                })?
                .this_value(),
            function,
        );
        let exit = self.open_frame(ControlFrameKind::Block, function);
        receiver.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<IteratorFromWrapper>(GcNullability::NonNullable)
                .heap_type,
        ));
        function.instruction(&Instruction::I32Eqz);
        self.emit_native_iterator_error_if(
            if returning {
                RuntimeErrorMessage::ITERATOR_FROM_WRAPPER_RETURN_CALLED_ON_INCOMPATIBLE_RECEIVER
            } else {
                RuntimeErrorMessage::ITERATOR_FROM_WRAPPER_NEXT_CALLED_ON_INCOMPATIBLE_RECEIVER
            },
            &result,
            exit,
            function,
        )?;
        let wrapper = schema.reserve_gc_local(function).initialize(
            receiver.cast_reference::<IteratorFromWrapper>(schema, function),
            function,
        );
        let record = schema.reserve_gc_local(function).initialize(
            schema
                .field(IteratorFromWrapperSchema::RECORD)
                .read(&wrapper, schema, function)
                .reference(),
            function,
        );
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .field(IteratorRecordSchema::ITERATOR)
                .read(&record, schema, function)
                .reference(),
            function,
        );
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored, &iterator, schema, function);
        stored.clear(function);
        if returning {
            // The return method is nullable and freshly acquired for each call.
            let key = self.emit_iterator_named_key("return", function)?;
            self.emit_object_read(&iterator, &iterator, &key, &result, function)?;
            key.clear(function);
            self.emit_native_iterator_abrupt_exit(&result, &result, exit, function);
            method.copy_from(result.value(), function);
            self.compile_nullish_tagged_i32(method.tag(), function)?;
            self.open_frame(ControlFrameKind::If, function);
            method.set_undefined(function);
            self.emit_iterator_result_object_from_locals(&method, true, &result, function)?;
            function.instruction(&Instruction::Else);
            self.emit_iterator_from_call(
                &method,
                &iterator,
                &result,
                RuntimeErrorMessage::ITERATOR_FROM_WRAPPER_RETURN_METHOD_MUST_BE_CALLABLE,
                function,
            )?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        } else {
            let stored = schema.reserve_gc_local(function).initialize(
                schema
                    .field(IteratorRecordSchema::NEXT_METHOD)
                    .read(&record, schema, function)
                    .reference(),
                function,
            );
            schema
                .struct_type::<StoredValue>()
                .read_into(&stored, &method, schema, function);
            stored.clear(function);
            self.emit_iterator_from_call(
                &method,
                &iterator,
                &result,
                RuntimeErrorMessage::ITERATOR_FROM_WRAPPER_NEXT_METHOD_MUST_BE_CALLABLE,
                function,
            )?;
        }
        record.clear(function);
        wrapper.clear(function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&result, function);
        result.clear(function);
        method.clear(function);
        iterator.clear(function);
        receiver.clear(function);
        Ok(())
    }
}
