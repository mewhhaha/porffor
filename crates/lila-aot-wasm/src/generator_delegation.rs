use super::*;
use crate::gc_types::*;
use crate::operations::PropertyKeyLocals;

pub(crate) enum AsyncGeneratorDelegationKind {
    YieldStar,
    ForAwaitYield,
}

enum GeneratorDelegateProperty {
    AsyncIterator,
    Iterator,
    Next,
    Return,
    Throw,
    Done,
    Value,
}

impl FunctionBuilder<'_> {
    fn emit_generator_delegate_property_read(
        &mut self,
        target: &ValueLocals,
        property: GeneratorDelegateProperty,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let key = match property {
            GeneratorDelegateProperty::AsyncIterator | GeneratorDelegateProperty::Iterator => {
                let symbol = match property {
                    GeneratorDelegateProperty::AsyncIterator => {
                        lila_ir::WellKnownSymbol::AsyncIterator
                    }
                    GeneratorDelegateProperty::Iterator => lila_ir::WellKnownSymbol::Iterator,
                    GeneratorDelegateProperty::Next
                    | GeneratorDelegateProperty::Return
                    | GeneratorDelegateProperty::Throw
                    | GeneratorDelegateProperty::Done
                    | GeneratorDelegateProperty::Value => unreachable!("selected symbol property"),
                };
                let root = schema.reserve_gc_local(function).initialize(
                    self.emit_well_known_symbol_reference(symbol, function)?,
                    function,
                );
                let key = PropertyKeyLocals::from_symbol(schema, &root, function);
                root.clear(function);
                key
            }
            GeneratorDelegateProperty::Next
            | GeneratorDelegateProperty::Return
            | GeneratorDelegateProperty::Throw
            | GeneratorDelegateProperty::Done
            | GeneratorDelegateProperty::Value => {
                let name = match property {
                    GeneratorDelegateProperty::Next => "next",
                    GeneratorDelegateProperty::Return => "return",
                    GeneratorDelegateProperty::Throw => "throw",
                    GeneratorDelegateProperty::Done => "done",
                    GeneratorDelegateProperty::Value => "value",
                    GeneratorDelegateProperty::AsyncIterator
                    | GeneratorDelegateProperty::Iterator => {
                        unreachable!("selected string property")
                    }
                };
                let root = schema.reserve_gc_local(function).initialize(
                    self.emit_interned_string_reference(name, function)?,
                    function,
                );
                let key = PropertyKeyLocals::from_string(schema, &root, function);
                root.clear(function);
                key
            }
        };
        // GetV boxes the lookup target and retains the original primitive
        // receiver. Iterator-result/iterator targets are already Objects.
        let boxed = schema.reserve_completion(function);
        self.emit_value_to_object_locals(target, &boxed, function)?;
        boxed.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        result.copy_from(&boxed, function);
        function.instruction(&Instruction::Else);
        self.emit_object_read(boxed.value(), target, &key, result, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        boxed.clear(function);
        key.clear(function);
        Ok(())
    }

    fn emit_generator_delegate_propagate(
        &mut self,
        result: &CompletionLocals,
        function: &mut Function,
    ) {
        self.completion().copy_from(result, function);
        self.emit_propagate_current_throw_if_needed(function);
    }

    fn emit_generator_delegate_error(
        &mut self,
        message: RuntimeErrorMessage,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let error = self.runtime_schema().reserve_completion(function);
        self.emit_throw_runtime_error(
            lila_ir::NativeErrorKind::TypeError,
            message,
            &error,
            function,
        )?;
        self.completion().copy_from(&error, function);
        error.clear(function);
        self.emit_propagate_current_throw(function);
        Ok(())
    }

    fn emit_require_generator_delegate_object(
        &mut self,
        value: &ValueLocals,
        message: RuntimeErrorMessage,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_is_heap_object_like_tag_i32(value.tag(), function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_generator_delegate_error(message, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_generator_delegate_call(
        &mut self,
        method: &ValueLocals,
        receiver: &ValueLocals,
        arguments: &[&ValueLocals],
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argv = self.emit_pre_evaluated_arg_vector(arguments, function);
        self.emit_function_or_proxy_call_with_argv(method, receiver, &argv, result, function)?;
        argv.clear(function);
        Ok(())
    }

    fn emit_acquire_generator_delegate(
        &mut self,
        source: &ValueLocals,
        asynchronous: bool,
        function: &mut Function,
    ) -> Result<GcLocal<GeneratorDelegate>, EmitError> {
        let schema = self.runtime_schema();
        let method = schema.reserve_completion(function);
        let iterator = schema.reserve_completion(function);
        let next = schema.reserve_completion(function);
        let is_async = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I32Const(i32::from(asynchronous)));
        is_async.store(function);
        self.compile_nullish_tagged_i32(source.tag(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.emit_generator_delegate_error(
            RuntimeErrorMessage::YIELD_TARGET_IS_NOT_ITERABLE,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        if asynchronous {
            self.emit_generator_delegate_property_read(
                source,
                GeneratorDelegateProperty::AsyncIterator,
                &method,
                function,
            )?;
            self.emit_generator_delegate_propagate(&method, function);
            self.compile_nullish_tagged_i32(method.value().tag(), function)?;
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I32Const(0));
            is_async.store(function);
            self.emit_generator_delegate_property_read(
                source,
                GeneratorDelegateProperty::Iterator,
                &method,
                function,
            )?;
            self.emit_generator_delegate_propagate(&method, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        } else {
            self.emit_generator_delegate_property_read(
                source,
                GeneratorDelegateProperty::Iterator,
                &method,
                function,
            )?;
            self.emit_generator_delegate_propagate(&method, function);
        }
        self.emit_generator_delegate_call(method.value(), source, &[], &iterator, function)?;
        self.emit_generator_delegate_propagate(&iterator, function);
        self.emit_require_generator_delegate_object(
            iterator.value(),
            RuntimeErrorMessage::YIELD_ITERATOR_METHOD_MUST_RETURN_OBJECT,
            function,
        )?;
        self.emit_generator_delegate_property_read(
            iterator.value(),
            GeneratorDelegateProperty::Next,
            &next,
            function,
        )?;
        self.emit_generator_delegate_propagate(&next, function);
        let stored_iterator = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(iterator.value(), function),
            function,
        );
        let stored_next = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(next.value(), function),
            function,
        );
        let record = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<IteratorRecord>().construct(
                (
                    GcOperand::reference(&stored_iterator, schema),
                    GcOperand::reference(&stored_next, schema),
                    GcOperand::boolean(false),
                ),
                function,
            ),
            function,
        );
        let empty = schema.reserve_value_local(function);
        empty.set_undefined(function);
        let stored_empty = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&empty, function),
            function,
        );
        let delegate = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<GeneratorDelegate>().construct(
                (
                    GcOperand::reference(&record, schema),
                    GcOperand::constant(GeneratorDelegatePending::Normal),
                    GcOperand::reference(&stored_empty, schema),
                    GcOperand::boolean_local(is_async),
                ),
                function,
            ),
            function,
        );
        stored_empty.clear(function);
        empty.clear(function);
        record.clear(function);
        stored_next.clear(function);
        stored_iterator.clear(function);
        schema.release_i32_local(is_async, function);
        next.clear(function);
        iterator.clear(function);
        method.clear(function);
        Ok(delegate)
    }

    fn emit_load_generator_delegate_record(
        &self,
        delegate: &GcLocal<GeneratorDelegate>,
        iterator: &ValueLocals,
        next: &ValueLocals,
        function: &mut Function,
    ) -> GcLocal<IteratorRecord> {
        let schema = self.runtime_schema();
        let record = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<GeneratorDelegate>()
                .field(GeneratorDelegateSchema::ITERATOR)
                .read(delegate, schema, function)
                .reference(),
            function,
        );
        let stored_iterator = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<IteratorRecord>()
                .field(IteratorRecordSchema::ITERATOR)
                .read(&record, schema, function)
                .reference(),
            function,
        );
        let stored_next = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<IteratorRecord>()
                .field(IteratorRecordSchema::NEXT_METHOD)
                .read(&record, schema, function)
                .reference(),
            function,
        );
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored_iterator, iterator, schema, function);
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored_next, next, schema, function);
        stored_next.clear(function);
        stored_iterator.clear(function);
        record
    }

    fn emit_finish_generator_delegation_value(
        &mut self,
        value: &ValueLocals,
        resume_mode: &GeneratorResumeModeIr,
        asynchronous: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.completion().set_normal(value, function);
        match resume_mode {
            GeneratorResumeModeIr::Ignore => {
                if !self.has_generator_statement_list_value() {
                    self.emit_statement_result(function);
                }
            }
            GeneratorResumeModeIr::Return => {
                self.set_completion_kind(CompletionKind::Return, function);
                if asynchronous {
                    self.emit_dispatch_async_generator_completion(function);
                } else {
                    self.emit_dispatch_current_completion(function)?;
                }
            }
            GeneratorResumeModeIr::AssignIdentifier(name) => {
                self.emit_resumed_binding_assignment(name, value, function)?;
                self.emit_statement_result(function);
            }
            GeneratorResumeModeIr::AssignGlobal { name, strictness } => {
                self.emit_resumed_global_assignment(name, value, *strictness, function)?;
                self.emit_statement_result(function);
            }
            GeneratorResumeModeIr::AssignProperty(reference) => {
                self.write_suspended_property_reference(reference, value, function)?;
                self.emit_statement_result(function);
            }
        }
        Ok(())
    }

    pub(crate) fn compile_generator_delegation(
        &mut self,
        expression: &TypedExpr,
        suspend_state: u32,
        resume_state: u32,
        resume_mode: &GeneratorResumeModeIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let activation = self.control_flow_generator_activation(function)?;
        let row = schema.struct_type::<GeneratorActivation>();
        let point = self.emit_resumable_resume_point(function)?;
        point.load(function);
        function.instruction(&Instruction::I32Const(suspend_state as i32));
        function.instruction(&Instruction::I32Eq);
        point.load(function);
        function.instruction(&Instruction::I32Const(resume_state as i32));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        let terminal = self.open_frame(ControlFrameKind::Block, function);
        self.throw_handler_stack.push(terminal);
        self.finally_stack.push(terminal);
        let incoming = schema.reserve_value_local(function);
        let iterator = schema.reserve_value_local(function);
        let next = schema.reserve_value_local(function);
        let result_value = schema.reserve_value_local(function);
        let value = schema.reserve_value_local(function);
        let method = schema.reserve_completion(function);
        let called = schema.reserve_completion(function);
        let closed = schema.reserve_completion(function);
        let close_pending = schema.reserve_completion(function);
        let kind = schema.reserve_i32_local(function);
        let done = schema.reserve_i32_local(function);
        let root = schema
            .reserve_gc_local::<GeneratorDelegate, Nullable>(function)
            .initialize_null(schema, function);
        point.load(function);
        function.instruction(&Instruction::I32Const(suspend_state as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        if let GeneratorResumeModeIr::AssignProperty(reference) = resume_mode {
            self.prepare_suspended_property_reference(reference, function)?;
        }
        self.compile_expr_to_value(expression, &value, function)?;
        self.emit_propagate_current_throw_if_needed(function);
        let delegate = self.emit_acquire_generator_delegate(&value, false, function)?;
        row.field(GeneratorActivationSchema::DELEGATE).write(
            &activation,
            GcOperand::nullable_reference(&delegate, schema),
            schema,
            function,
        );
        root.replace(delegate.load(schema, function).nullable(), function);
        delegate.clear(function);
        incoming.set_undefined(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            GeneratorResumeKind::Normal,
        )));
        kind.store(function);
        function.instruction(&Instruction::Else);
        root.replace(
            row.field(GeneratorActivationSchema::DELEGATE)
                .read(&activation, schema, function)
                .reference(),
            function,
        );
        row.field(GeneratorActivationSchema::RESUME_KIND)
            .read(&activation, schema, function)
            .store(kind, function);
        let stored = schema.reserve_gc_local(function).initialize(
            row.field(GeneratorActivationSchema::RESUME_VALUE)
                .read(&activation, schema, function)
                .reference(),
            function,
        );
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored, &incoming, schema, function);
        stored.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let delegate = schema.reserve_gc_local(function).initialize(
            root.load(schema, function).require_non_null(function),
            function,
        );
        root.clear(function);
        let record =
            self.emit_load_generator_delegate_record(&delegate, &iterator, &next, function);
        kind.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            GeneratorResumeKind::Throw,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_generator_delegate_property_read(
            &iterator,
            GeneratorDelegateProperty::Throw,
            &method,
            function,
        )?;
        self.emit_generator_delegate_propagate(&method, function);
        self.compile_nullish_tagged_i32(method.value().tag(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        close_pending.initialize(function);
        self.emit_iterator_close_with_completion(&iterator, &close_pending, &closed, function)?;
        self.emit_generator_delegate_propagate(&closed, function);
        self.emit_generator_delegate_error(
            RuntimeErrorMessage::YIELD_ITERATOR_HAS_NO_THROW_METHOD,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_generator_delegate_call(
            method.value(),
            &iterator,
            &[&incoming],
            &called,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        kind.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            GeneratorResumeKind::Return,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_generator_delegate_property_read(
            &iterator,
            GeneratorDelegateProperty::Return,
            &method,
            function,
        )?;
        self.emit_generator_delegate_propagate(&method, function);
        self.compile_nullish_tagged_i32(method.value().tag(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.completion().set_normal(&incoming, function);
        self.set_completion_kind(CompletionKind::Return, function);
        self.emit_dispatch_current_completion(function)?;
        function.instruction(&Instruction::Else);
        self.emit_generator_delegate_call(
            method.value(),
            &iterator,
            &[&incoming],
            &called,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_generator_delegate_call(&next, &iterator, &[&incoming], &called, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_generator_delegate_propagate(&called, function);
        result_value.copy_from(called.value(), function);
        self.emit_require_generator_delegate_object(
            &result_value,
            RuntimeErrorMessage::YIELD_ITERATOR_RESULT_MUST_BE_OBJECT,
            function,
        )?;
        self.emit_generator_delegate_property_read(
            &result_value,
            GeneratorDelegateProperty::Done,
            &method,
            function,
        )?;
        self.emit_generator_delegate_propagate(&method, function);
        self.compile_truthy_tagged_i32(method.value(), function)?;
        done.store(function);
        done.load(function);
        self.open_frame(ControlFrameKind::If, function);
        schema
            .struct_type::<IteratorRecord>()
            .field(IteratorRecordSchema::DONE)
            .write(&record, GcOperand::boolean(true), schema, function);
        self.emit_generator_delegate_property_read(
            &result_value,
            GeneratorDelegateProperty::Value,
            &method,
            function,
        )?;
        self.emit_generator_delegate_propagate(&method, function);
        value.copy_from(method.value(), function);
        row.field(GeneratorActivationSchema::DELEGATE).write(
            &activation,
            GcOperand::null(schema),
            schema,
            function,
        );
        self.emit_set_resumable_resume_point(resume_state, function)?;
        kind.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            GeneratorResumeKind::Return,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.completion().set_normal(&value, function);
        self.set_completion_kind(CompletionKind::Return, function);
        self.emit_dispatch_current_completion(function)?;
        function.instruction(&Instruction::Else);
        self.emit_finish_generator_delegation_value(&value, resume_mode, false, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        // GeneratorYield publishes this same IteratorResult. DELEGATE retains
        // its original identity; STATUS alone identifies suspension.
        self.emit_set_resumable_resume_point(resume_state, function)?;
        self.emit_save_resumable_environment(function)?;
        row.field(GeneratorActivationSchema::STATUS).write(
            &activation,
            GcOperand::constant(GeneratorState::SuspendedYield),
            schema,
            function,
        );
        self.completion().set_normal(&result_value, function);
        self.emit_return_current_completion(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.finally_stack.pop();
        self.throw_handler_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        row.field(GeneratorActivationSchema::DELEGATE).write(
            &activation,
            GcOperand::null(schema),
            schema,
            function,
        );
        if let GeneratorResumeModeIr::AssignProperty(_) = resume_mode {
            self.completion().kind().load(function);
            function.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
            function.instruction(&Instruction::I32Ne);
            self.open_frame(ControlFrameKind::If, function);
            self.clear_suspended_property_reference(function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.emit_dispatch_current_completion(function)?;
        record.clear(function);
        delegate.clear(function);
        schema.release_i32_local(done, function);
        schema.release_i32_local(kind, function);
        close_pending.clear(function);
        closed.clear(function);
        called.clear(function);
        method.clear(function);
        value.clear(function);
        result_value.clear(function);
        next.clear(function);
        iterator.clear(function);
        incoming.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i32_local(point, function);
        activation.clear(function);
        Ok(())
    }

    fn emit_store_async_generator_delegate_pending(
        &self,
        delegate: &GcLocal<GeneratorDelegate>,
        kind: &GcI32DomainLocal<GeneratorDelegatePending>,
        value: &ValueLocals,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(value, function),
            function,
        );
        let row = schema.struct_type::<GeneratorDelegate>();
        row.field(GeneratorDelegateSchema::PENDING_KIND).write(
            delegate,
            kind.operand(),
            schema,
            function,
        );
        row.field(GeneratorDelegateSchema::PENDING_VALUE).write(
            delegate,
            GcOperand::reference(&stored, schema),
            schema,
            function,
        );
        stored.clear(function);
    }

    fn emit_async_generator_delegate_await(
        &mut self,
        activation: &GcLocal<AsyncGeneratorActivation>,
        value: &ValueLocals,
        resume_state: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        self.emit_set_resumable_resume_point(resume_state, function)?;
        self.emit_save_resumable_environment(function)?;
        self.emit_async_generator_await_reactions(activation, value, function)?;
        self.emit_propagate_current_throw_if_needed(function);
        let row = schema.struct_type::<AsyncGeneratorActivation>();
        row.field(AsyncGeneratorActivationSchema::BODY_STATUS)
            .write(
                activation,
                GcOperand::constant(AsyncGeneratorBodyStatus::Await),
                schema,
                function,
            );
        row.field(AsyncGeneratorActivationSchema::EXECUTION_STATE)
            .write(
                activation,
                GcOperand::constant(AsyncGeneratorExecutionState::Executing),
                schema,
                function,
            );
        self.emit_statement_result(function);
        self.emit_return_current_completion(function);
        Ok(())
    }

    fn emit_async_generator_delegate_yield(
        &mut self,
        activation: &GcLocal<AsyncGeneratorActivation>,
        value: &ValueLocals,
        resume_state: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        self.emit_set_resumable_resume_point(resume_state, function)?;
        self.emit_save_resumable_environment(function)?;
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(value, function),
            function,
        );
        let row = schema.struct_type::<AsyncGeneratorActivation>();
        row.field(AsyncGeneratorActivationSchema::BODY_RESULT)
            .write(
                activation,
                GcOperand::reference(&stored, schema),
                schema,
                function,
            );
        stored.clear(function);
        row.field(AsyncGeneratorActivationSchema::BODY_STATUS)
            .write(
                activation,
                GcOperand::constant(AsyncGeneratorBodyStatus::Yield),
                schema,
                function,
            );
        row.field(AsyncGeneratorActivationSchema::EXECUTION_STATE)
            .write(
                activation,
                GcOperand::constant(AsyncGeneratorExecutionState::SuspendedYield),
                schema,
                function,
            );
        self.completion().set_normal(value, function);
        self.emit_return_current_completion(function);
        Ok(())
    }

    fn emit_async_for_await_delegate_completion(
        &mut self,
        pending: &GcI32DomainLocal<GeneratorDelegatePending>,
        value: &ValueLocals,
        activation: &GcLocal<AsyncGeneratorActivation>,
        function: &mut Function,
    ) {
        self.completion().set_normal(value, function);
        pending.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            GeneratorDelegatePending::Throw,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.set_completion_kind(CompletionKind::Throw, function);
        function.instruction(&Instruction::Else);
        self.set_completion_kind(CompletionKind::Return, function);
        self.emit_mark_async_generator_return_awaited(activation, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_dispatch_async_generator_completion(function);
    }

    fn emit_async_for_await_delegate_close(
        &mut self,
        activation: &GcLocal<AsyncGeneratorActivation>,
        record: &GcLocal<IteratorRecord>,
        iterator: &ValueLocals,
        pending: &GcI32DomainLocal<GeneratorDelegatePending>,
        original: &ValueLocals,
        is_async: I32Local,
        resume_state: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let method = schema.reserve_completion(function);
        let called = schema.reserve_completion(function);
        let awaited = schema.reserve_value_local(function);
        schema
            .struct_type::<IteratorRecord>()
            .field(IteratorRecordSchema::DONE)
            .write(record, GcOperand::boolean(true), schema, function);
        is_async.load(function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_generator_delegate_property_read(
            iterator,
            GeneratorDelegateProperty::Return,
            &method,
            function,
        )?;
        method.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        pending.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            GeneratorDelegatePending::Throw,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_async_for_await_delegate_completion(pending, original, activation, function);
        function.instruction(&Instruction::Else);
        self.emit_generator_delegate_propagate(&method, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.compile_nullish_tagged_i32(method.value().tag(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.emit_async_for_await_delegate_completion(pending, original, activation, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_generator_delegate_call(method.value(), iterator, &[], &called, function)?;
        called.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        pending.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            GeneratorDelegatePending::Throw,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_async_for_await_delegate_completion(pending, original, activation, function);
        function.instruction(&Instruction::Else);
        self.emit_generator_delegate_propagate(&called, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        awaited.copy_from(called.value(), function);
        function.instruction(&Instruction::Else);
        let realm = self.load_current_realm(function);
        let promise = self.emit_async_from_sync_iterator_method(
            record,
            AsyncFromSyncIteratorMethod::Return,
            None,
            &realm,
            function,
        )?;
        awaited.set_reference(&promise, schema, function);
        promise.clear(function);
        realm.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let await_failure = self.open_frame(ControlFrameKind::Block, function);
        self.throw_handler_stack.push(await_failure);
        self.emit_async_generator_delegate_await(activation, &awaited, resume_state, function)?;
        self.throw_handler_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        // This edge is reached only if PromiseResolve of the returned value
        // throws before an Await reaction can be registered.
        pending.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            GeneratorDelegatePending::Throw,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_async_for_await_delegate_completion(pending, original, activation, function);
        function.instruction(&Instruction::Else);
        self.emit_dispatch_async_generator_completion(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        awaited.clear(function);
        called.clear(function);
        method.clear(function);
        Ok(())
    }

    pub(crate) fn compile_async_generator_delegation(
        &mut self,
        expression: &TypedExpr,
        suspend_state: u32,
        resume_state: u32,
        resume_mode: &GeneratorResumeModeIr,
        delegation_kind: AsyncGeneratorDelegationKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let activation = self.control_flow_async_generator_activation(function)?;
        let row = schema.struct_type::<AsyncGeneratorActivation>();
        let point = self.emit_resumable_resume_point(function)?;
        point.load(function);
        function.instruction(&Instruction::I32Const(suspend_state as i32));
        function.instruction(&Instruction::I32Eq);
        point.load(function);
        function.instruction(&Instruction::I32Const(resume_state as i32));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        let terminal = self.open_frame(ControlFrameKind::Block, function);
        self.throw_handler_stack.push(terminal);
        self.finally_stack.push(terminal);
        let incoming = schema.reserve_value_local(function);
        let original = schema.reserve_value_local(function);
        let iterator = schema.reserve_value_local(function);
        let next = schema.reserve_value_local(function);
        let value = schema.reserve_value_local(function);
        let awaited = schema.reserve_value_local(function);
        let method = schema.reserve_completion(function);
        let called = schema.reserve_completion(function);
        let incoming_kind = schema.reserve_i32_local(function);
        let is_async = schema.reserve_i32_local(function);
        let done = schema.reserve_i32_local(function);
        let pending = GcI32DomainLocal::new(schema, GeneratorDelegatePending::Normal, function);
        let root = schema
            .reserve_gc_local::<GeneratorDelegate, Nullable>(function)
            .initialize_null(schema, function);
        point.load(function);
        function.instruction(&Instruction::I32Const(suspend_state as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        if let GeneratorResumeModeIr::AssignProperty(reference) = resume_mode {
            self.prepare_suspended_property_reference(reference, function)?;
        }
        self.compile_expr_to_value(expression, &value, function)?;
        self.emit_propagate_current_throw_if_needed(function);
        let delegate = self.emit_acquire_generator_delegate(&value, true, function)?;
        row.field(AsyncGeneratorActivationSchema::DELEGATE).write(
            &activation,
            GcOperand::nullable_reference(&delegate, schema),
            schema,
            function,
        );
        root.replace(delegate.load(schema, function).nullable(), function);
        delegate.clear(function);
        incoming.set_undefined(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            AsyncGeneratorResumeKind::Normal,
        )));
        incoming_kind.store(function);
        function.instruction(&Instruction::Else);
        root.replace(
            row.field(AsyncGeneratorActivationSchema::DELEGATE)
                .read(&activation, schema, function)
                .reference(),
            function,
        );
        row.field(AsyncGeneratorActivationSchema::RESUME_KIND)
            .read(&activation, schema, function)
            .store(incoming_kind, function);
        let stored = schema.reserve_gc_local(function).initialize(
            row.field(AsyncGeneratorActivationSchema::RESUME_VALUE)
                .read(&activation, schema, function)
                .reference(),
            function,
        );
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored, &incoming, schema, function);
        stored.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let delegate = schema.reserve_gc_local(function).initialize(
            root.load(schema, function).require_non_null(function),
            function,
        );
        root.clear(function);
        let record =
            self.emit_load_generator_delegate_record(&delegate, &iterator, &next, function);
        let delegate_row = schema.struct_type::<GeneratorDelegate>();
        delegate_row
            .field(GeneratorDelegateSchema::ASYNC_ITERATOR)
            .read(&delegate, schema, function)
            .store(is_async, function);
        delegate_row
            .field(GeneratorDelegateSchema::PENDING_KIND)
            .read(&delegate, schema, function)
            .store_domain(&pending, function);
        let stored = schema.reserve_gc_local(function).initialize(
            delegate_row
                .field(GeneratorDelegateSchema::PENDING_VALUE)
                .read(&delegate, schema, function)
                .reference(),
            function,
        );
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored, &original, schema, function);
        stored.clear(function);
        incoming_kind.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            AsyncGeneratorResumeKind::Fulfill,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        pending.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            GeneratorDelegatePending::ReturnValue,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.completion().set_normal(&incoming, function);
        self.set_completion_kind(CompletionKind::Return, function);
        self.emit_mark_async_generator_return_awaited(&activation, function);
        self.emit_dispatch_async_generator_completion(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        if matches!(delegation_kind, AsyncGeneratorDelegationKind::ForAwaitYield) {
            pending.load(function);
            function.instruction(&Instruction::I32Const(GcI32Constant::encode(
                GeneratorDelegatePending::YieldValue,
            )));
            function.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_async_generator_delegate_yield(
                &activation,
                &incoming,
                resume_state,
                function,
            )?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            pending.load(function);
            function.instruction(&Instruction::I32Const(GcI32Constant::encode(
                GeneratorDelegatePending::Throw,
            )));
            function.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_async_for_await_delegate_completion(
                &pending,
                &original,
                &activation,
                function,
            );
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            pending.load(function);
            function.instruction(&Instruction::I32Const(GcI32Constant::encode(
                GeneratorDelegatePending::Return,
            )));
            function.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_require_generator_delegate_object(
                &incoming,
                RuntimeErrorMessage::YIELD_ITERATOR_RESULT_MUST_BE_OBJECT,
                function,
            )?;
            self.emit_async_for_await_delegate_completion(
                &pending,
                &original,
                &activation,
                function,
            );
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.emit_require_generator_delegate_object(
            &incoming,
            RuntimeErrorMessage::YIELD_ITERATOR_RESULT_MUST_BE_OBJECT,
            function,
        )?;
        pending.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            GeneratorDelegatePending::MissingThrowClose,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_generator_delegate_error(
            RuntimeErrorMessage::YIELD_ITERATOR_HAS_NO_THROW_METHOD,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_generator_delegate_property_read(
            &incoming,
            GeneratorDelegateProperty::Done,
            &method,
            function,
        )?;
        self.emit_generator_delegate_propagate(&method, function);
        self.compile_truthy_tagged_i32(method.value(), function)?;
        done.store(function);
        done.load(function);
        self.open_frame(ControlFrameKind::If, function);
        schema
            .struct_type::<IteratorRecord>()
            .field(IteratorRecordSchema::DONE)
            .write(&record, GcOperand::boolean(true), schema, function);
        row.field(AsyncGeneratorActivationSchema::DELEGATE).write(
            &activation,
            GcOperand::null(schema),
            schema,
            function,
        );
        match delegation_kind {
            AsyncGeneratorDelegationKind::YieldStar => {
                self.emit_generator_delegate_property_read(
                    &incoming,
                    GeneratorDelegateProperty::Value,
                    &method,
                    function,
                )?;
                self.emit_generator_delegate_propagate(&method, function);
                value.copy_from(method.value(), function);
                pending.load(function);
                function.instruction(&Instruction::I32Const(GcI32Constant::encode(
                    GeneratorDelegatePending::Return,
                )));
                function.instruction(&Instruction::I32Eq);
                self.open_frame(ControlFrameKind::If, function);
                self.completion().set_normal(&value, function);
                self.set_completion_kind(CompletionKind::Return, function);
                self.emit_dispatch_async_generator_completion(function);
                function.instruction(&Instruction::Else);
                self.emit_finish_generator_delegation_value(&value, resume_mode, true, function)?;
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
            AsyncGeneratorDelegationKind::ForAwaitYield => self.emit_statement_result(function),
        }
        self.emit_set_resumable_resume_point(resume_state, function)?;
        self.emit_branch_to_target(terminal, function);
        function.instruction(&Instruction::Else);
        self.emit_generator_delegate_property_read(
            &incoming,
            GeneratorDelegateProperty::Value,
            &method,
            function,
        )?;
        self.emit_generator_delegate_propagate(&method, function);
        value.copy_from(method.value(), function);
        match delegation_kind {
            AsyncGeneratorDelegationKind::YieldStar => self.emit_async_generator_delegate_yield(
                &activation,
                &value,
                resume_state,
                function,
            )?,
            AsyncGeneratorDelegationKind::ForAwaitYield => {
                // The source body is plain Yield(value), which owns Await(value)
                // even when the iterator itself is a genuine async iterator.
                pending.set_constant(GeneratorDelegatePending::YieldValue, function);
                self.emit_store_async_generator_delegate_pending(
                    &delegate, &pending, &value, function,
                );
                self.emit_async_generator_delegate_await(
                    &activation,
                    &value,
                    resume_state,
                    function,
                )?;
            }
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // A wrapper rejection owns any sync IteratorClose. A plain-Yield Await
        // rejection belongs to the for-await body and must close its iterator.
        incoming_kind.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            AsyncGeneratorResumeKind::Reject,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        if matches!(delegation_kind, AsyncGeneratorDelegationKind::ForAwaitYield) {
            pending.load(function);
            function.instruction(&Instruction::I32Const(GcI32Constant::encode(
                GeneratorDelegatePending::YieldValue,
            )));
            function.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I32Const(GcI32Constant::encode(
                AsyncGeneratorResumeKind::Throw,
            )));
            incoming_kind.store(function);
            function.instruction(&Instruction::Else);
            pending.load(function);
            function.instruction(&Instruction::I32Const(GcI32Constant::encode(
                GeneratorDelegatePending::Throw,
            )));
            function.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_async_for_await_delegate_completion(
                &pending,
                &original,
                &activation,
                function,
            );
            function.instruction(&Instruction::Else);
            self.completion().set_throw(&incoming, function);
            self.emit_dispatch_async_generator_completion(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        } else {
            self.completion().set_throw(&incoming, function);
            self.emit_dispatch_async_generator_completion(function);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        pending.set_constant(GeneratorDelegatePending::Normal, function);
        incoming_kind.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            AsyncGeneratorResumeKind::Return,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        pending.set_constant(GeneratorDelegatePending::Return, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        incoming_kind.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            AsyncGeneratorResumeKind::Throw,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        pending.set_constant(GeneratorDelegatePending::Throw, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_store_async_generator_delegate_pending(&delegate, &pending, &incoming, function);
        if matches!(delegation_kind, AsyncGeneratorDelegationKind::ForAwaitYield) {
            pending.load(function);
            function.instruction(&Instruction::I32Const(GcI32Constant::encode(
                GeneratorDelegatePending::Normal,
            )));
            function.instruction(&Instruction::I32Ne);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_async_for_await_delegate_close(
                &activation,
                &record,
                &iterator,
                &pending,
                &incoming,
                is_async,
                resume_state,
                function,
            )?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            incoming.set_undefined(function);
            is_async.load(function);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_generator_delegate_call(&next, &iterator, &[], &called, function)?;
            self.emit_generator_delegate_propagate(&called, function);
            awaited.copy_from(called.value(), function);
            function.instruction(&Instruction::Else);
            let realm = self.load_current_realm(function);
            let promise = self.emit_async_from_sync_iterator_method(
                &record,
                AsyncFromSyncIteratorMethod::Next,
                None,
                &realm,
                function,
            )?;
            awaited.set_reference(&promise, schema, function);
            promise.clear(function);
            realm.clear(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        } else {
            is_async.load(function);
            self.open_frame(ControlFrameKind::If, function);
            pending.load(function);
            function.instruction(&Instruction::I32Const(GcI32Constant::encode(
                GeneratorDelegatePending::Throw,
            )));
            function.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_generator_delegate_property_read(
                &iterator,
                GeneratorDelegateProperty::Throw,
                &method,
                function,
            )?;
            self.emit_generator_delegate_propagate(&method, function);
            self.compile_nullish_tagged_i32(method.value().tag(), function)?;
            self.open_frame(ControlFrameKind::If, function);
            self.emit_generator_delegate_property_read(
                &iterator,
                GeneratorDelegateProperty::Return,
                &method,
                function,
            )?;
            self.emit_generator_delegate_propagate(&method, function);
            self.compile_nullish_tagged_i32(method.value().tag(), function)?;
            self.open_frame(ControlFrameKind::If, function);
            self.emit_generator_delegate_error(
                RuntimeErrorMessage::YIELD_ITERATOR_HAS_NO_THROW_METHOD,
                function,
            )?;
            function.instruction(&Instruction::Else);
            self.emit_generator_delegate_call(method.value(), &iterator, &[], &called, function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            pending.set_constant(GeneratorDelegatePending::MissingThrowClose, function);
            function.instruction(&Instruction::Else);
            self.emit_generator_delegate_call(
                method.value(),
                &iterator,
                &[&incoming],
                &called,
                function,
            )?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::Else);
            pending.load(function);
            function.instruction(&Instruction::I32Const(GcI32Constant::encode(
                GeneratorDelegatePending::Return,
            )));
            function.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_generator_delegate_property_read(
                &iterator,
                GeneratorDelegateProperty::Return,
                &method,
                function,
            )?;
            self.emit_generator_delegate_propagate(&method, function);
            self.compile_nullish_tagged_i32(method.value().tag(), function)?;
            self.open_frame(ControlFrameKind::If, function);
            pending.set_constant(GeneratorDelegatePending::ReturnValue, function);
            self.emit_store_async_generator_delegate_pending(
                &delegate, &pending, &incoming, function,
            );
            self.emit_async_generator_delegate_await(
                &activation,
                &incoming,
                resume_state,
                function,
            )?;
            function.instruction(&Instruction::Else);
            self.emit_generator_delegate_call(
                method.value(),
                &iterator,
                &[&incoming],
                &called,
                function,
            )?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::Else);
            self.emit_generator_delegate_call(&next, &iterator, &[&incoming], &called, function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.emit_generator_delegate_propagate(&called, function);
            awaited.copy_from(called.value(), function);
            function.instruction(&Instruction::Else);
            let realm = self.load_current_realm(function);
            pending.load(function);
            function.instruction(&Instruction::I32Const(GcI32Constant::encode(
                GeneratorDelegatePending::Throw,
            )));
            function.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, function);
            let promise = self.emit_async_from_sync_iterator_method(
                &record,
                AsyncFromSyncIteratorMethod::Throw,
                Some(&incoming),
                &realm,
                function,
            )?;
            awaited.set_reference(&promise, schema, function);
            promise.clear(function);
            function.instruction(&Instruction::Else);
            pending.load(function);
            function.instruction(&Instruction::I32Const(GcI32Constant::encode(
                GeneratorDelegatePending::Return,
            )));
            function.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, function);
            let promise = self.emit_async_from_sync_iterator_method(
                &record,
                AsyncFromSyncIteratorMethod::Return,
                Some(&incoming),
                &realm,
                function,
            )?;
            awaited.set_reference(&promise, schema, function);
            promise.clear(function);
            function.instruction(&Instruction::Else);
            let promise = self.emit_async_from_sync_iterator_method(
                &record,
                AsyncFromSyncIteratorMethod::Next,
                Some(&incoming),
                &realm,
                function,
            )?;
            awaited.set_reference(&promise, schema, function);
            promise.clear(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            realm.clear(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.emit_store_async_generator_delegate_pending(&delegate, &pending, &incoming, function);
        self.emit_async_generator_delegate_await(&activation, &awaited, resume_state, function)?;
        self.finally_stack.pop();
        self.throw_handler_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        if matches!(delegation_kind, AsyncGeneratorDelegationKind::ForAwaitYield) {
            // Await's PromiseResolve may itself throw synchronously. The parked
            // closed phase distinguishes a body failure from an iterator fault.
            let cleanup_exit = self.open_frame(ControlFrameKind::Block, function);
            self.throw_handler_stack.push(cleanup_exit);
            self.finally_stack.push(cleanup_exit);
            let current = schema.reserve_gc_local(function).initialize(
                row.field(AsyncGeneratorActivationSchema::DELEGATE)
                    .read(&activation, schema, function)
                    .reference(),
                function,
            );
            current.load(schema, function).is_null(function);
            function.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, function);
            let retained = schema.reserve_gc_local(function).initialize(
                current.load(schema, function).require_non_null(function),
                function,
            );
            delegate_row
                .field(GeneratorDelegateSchema::PENDING_KIND)
                .read(&retained, schema, function)
                .store_domain(&pending, function);
            let stored = schema.reserve_gc_local(function).initialize(
                delegate_row
                    .field(GeneratorDelegateSchema::PENDING_VALUE)
                    .read(&retained, schema, function)
                    .reference(),
                function,
            );
            schema
                .struct_type::<StoredValue>()
                .read_into(&stored, &original, schema, function);
            stored.clear(function);
            self.completion().kind().load(function);
            function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
            function.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, function);
            pending.load(function);
            function.instruction(&Instruction::I32Const(GcI32Constant::encode(
                GeneratorDelegatePending::Throw,
            )));
            function.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.completion().set_throw(&original, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            pending.load(function);
            function.instruction(&Instruction::I32Const(GcI32Constant::encode(
                GeneratorDelegatePending::YieldValue,
            )));
            function.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, function);
            incoming.copy_from(self.completion().value(), function);
            pending.set_constant(GeneratorDelegatePending::Throw, function);
            self.emit_store_async_generator_delegate_pending(
                &retained, &pending, &incoming, function,
            );
            let retained_record =
                self.emit_load_generator_delegate_record(&retained, &iterator, &next, function);
            delegate_row
                .field(GeneratorDelegateSchema::ASYNC_ITERATOR)
                .read(&retained, schema, function)
                .store(is_async, function);
            self.emit_async_for_await_delegate_close(
                &activation,
                &retained_record,
                &iterator,
                &pending,
                &incoming,
                is_async,
                resume_state,
                function,
            )?;
            retained_record.clear(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            retained.clear(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            current.clear(function);
            self.finally_stack.pop();
            self.throw_handler_stack.pop();
            self.pop_control(ControlFrameKind::Block);
            function.instruction(&Instruction::End);
        }
        row.field(AsyncGeneratorActivationSchema::DELEGATE).write(
            &activation,
            GcOperand::null(schema),
            schema,
            function,
        );
        if let GeneratorResumeModeIr::AssignProperty(_) = resume_mode {
            self.completion().kind().load(function);
            function.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
            function.instruction(&Instruction::I32Ne);
            self.open_frame(ControlFrameKind::If, function);
            self.clear_suspended_property_reference(function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.emit_dispatch_async_generator_completion(function);
        record.clear(function);
        delegate.clear(function);
        pending.clear(schema, function);
        schema.release_i32_local(done, function);
        schema.release_i32_local(is_async, function);
        schema.release_i32_local(incoming_kind, function);
        called.clear(function);
        method.clear(function);
        awaited.clear(function);
        value.clear(function);
        next.clear(function);
        iterator.clear(function);
        original.clear(function);
        incoming.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i32_local(point, function);
        activation.clear(function);
        Ok(())
    }
}
