//! Agent text crosses a bounded byte wire; shared buffers cross as native resources.
use super::*;
use crate::builtins::intl_provider_wire::{IntlByteArrayBuilder, IntlByteArrayReader};
use crate::functions::NonArrayRealmIntrinsicSlot;
use crate::runtime_helpers::TransientByteAllocArguments;
use lila_runtime::AgentHostOperation;

impl FunctionBuilder<'_> {
    fn emit_agent_scalar_call(
        &self,
        operation: AgentHostOperation,
        first: I64Local,
        second: I64Local,
        output: I64Local,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let import = self
            .functions
            .agent_call_import_function_index()
            .ok_or_else(|| EmitError::unsupported("missing Test262 agent scalar host import"))?;
        f.instruction(&Instruction::I64Const(operation.wire()));
        first.load(f);
        second.load(f);
        f.instruction(&Instruction::Call(import));
        output.store(f);
        Ok(())
    }

    fn emit_agent_wire_extent(
        &mut self,
        length: I64Local,
        f: &mut Function,
    ) -> Result<I64Local, EmitError> {
        let schema = self.runtime_schema();
        length.load(f);
        f.instruction(&Instruction::I64Const(i32::MAX as i64));
        f.instruction(&Instruction::I64GtU);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let pointer = schema.reserve_i64_local(f);
        schema
            .call_helper(
                TransientByteAllocArguments::new(length),
                self.runtime_helper_base()?,
                f,
            )
            .store(pointer, f);
        pointer.load(f);
        length.load(f);
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I64Const(i32::MAX as i64));
        f.instruction(&Instruction::I64GtU);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(pointer)
    }

    fn emit_agent_text_call(
        &mut self,
        operation: AgentHostOperation,
        string: &GcLocal<StringValue>,
        status: I64Local,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let builder = IntlByteArrayBuilder::new(schema, f);
        builder.append_remaining_utf8(string, schema, f);
        let bytes = builder.finish(schema, f);
        let length = schema.reserve_i64_local(f);
        schema.array_type::<ByteArray>().length(&bytes, schema, f);
        f.instruction(&Instruction::I64ExtendI32U);
        length.store(f);
        let pointer = self.emit_agent_wire_extent(length, f)?;
        let index = schema.reserve_i32_local(f);
        let byte = schema.reserve_i32_local(f);
        f.instruction(&Instruction::I32Const(0));
        index.store(f);
        let done = self.open_frame(ControlFrameKind::Block, f);
        let next = self.open_frame(ControlFrameKind::Loop, f);
        index.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        length.load(f);
        f.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_branch_to_target(done, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        schema
            .array_type::<ByteArray>()
            .read(&bytes, index, schema, f)
            .store(byte, f);
        pointer.load(f);
        f.instruction(&Instruction::I32WrapI64);
        index.load(f);
        f.instruction(&Instruction::I32Add);
        byte.load(f);
        f.instruction(&Instruction::I32Store8(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));
        index.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        index.store(f);
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.emit_agent_scalar_call(operation, pointer, length, status, f)?;
        schema.release_i32_local(byte, f);
        schema.release_i32_local(index, f);
        schema.release_i64_local(pointer, f);
        schema.release_i64_local(length, f);
        bytes.clear(f);
        Ok(())
    }

    pub(crate) fn compile_host_agent_start_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_agent_string_builtin(AgentHostOperation::Start, f)
    }
    pub(crate) fn compile_host_agent_report_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_agent_string_builtin(AgentHostOperation::Report, f)
    }
    fn emit_agent_string_builtin(
        &mut self,
        operation: AgentHostOperation,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        let output = schema.reserve_completion(f);
        let pending = schema.reserve_completion(f);
        let status = schema.reserve_i64_local(f);
        self.emit_builtin_arg_to_value(0, &value, f);
        output.initialize(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        self.emit_value_to_string_payload(&value, &pending, f)?;
        self.emit_host_abrupt_exit(&pending, &output, exit, f);
        let string = schema
            .reserve_gc_local(f)
            .initialize(pending.value().cast_reference::<StringValue>(schema, f), f);
        self.emit_agent_text_call(operation, &string, status, f)?;
        string.clear(f);
        if operation == AgentHostOperation::Start {
            status.load(f);
            f.instruction(&Instruction::I64Const(0));
            f.instruction(&Instruction::I64LtS);
            self.open_frame(ControlFrameKind::If, f);
            self.emit_throw_current_function_realm_type_error(
                RuntimeErrorMessage::FAILED_TO_START_TEST262_AGENT,
                &output,
                f,
            )?;
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        schema.release_i64_local(status, f);
        pending.clear(f);
        output.clear(f);
        value.clear(f);
        Ok(())
    }

    pub(crate) fn compile_host_agent_broadcast_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        let id_value = schema.reserve_value_local(f);
        let output = schema.reserve_completion(f);
        let pending = schema.reserve_completion(f);
        let id = schema.reserve_i64_local(f);
        let status = schema.reserve_i64_local(f);
        self.emit_builtin_arg_to_value(0, &value, f);
        self.emit_builtin_arg_to_value(1, &id_value, f);
        output.initialize(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        value.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<SharedArrayBuffer>(GcNullability::NonNullable)
                .heap_type,
        ));
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::AGENT_BROADCAST_REQUIRES_SHAREDARRAYBUFFER,
            &output,
            f,
        )?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let buffer = schema
            .reserve_gc_local(f)
            .initialize(value.cast_reference::<SharedArrayBuffer>(schema, f), f);
        let resource = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<SharedArrayBuffer>()
                .field(SharedArrayBufferSchema::BACKING_RESOURCE)
                .read(&buffer, schema, f)
                .reference(),
            f,
        );
        self.emit_value_to_number_payload(&id_value, &pending, f)?;
        pending.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        output.copy_from(&pending, f);
        f.instruction(&Instruction::Else);
        self.emit_to_uint32_i64_from_number_payload(pending.value().scalar(), id, f);
        id.load(f);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::I64ExtendI32S);
        id.store(f);
        self.functions
            .gc_host_imports()
            .get(GcHostImport::AgentBroadcastResource)
            .ok_or_else(|| EmitError::unsupported("missing agent resource broadcast import"))?
            .broadcast_shared_buffer(&resource, id, status, schema, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        resource.clear(f);
        buffer.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        schema.release_i64_local(status, f);
        schema.release_i64_local(id, f);
        pending.clear(f);
        output.clear(f);
        id_value.clear(f);
        value.clear(f);
        Ok(())
    }

    pub(crate) fn compile_host_agent_receive_broadcast_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let output = schema.reserve_completion(f);
        let id = schema.reserve_i64_local(f);
        output.initialize(f);
        let native = self
            .functions
            .gc_host_imports()
            .get(GcHostImport::AgentReceiveResource)
            .ok_or_else(|| EmitError::unsupported("missing agent resource receive import"))?
            .receive_shared_buffer(id, f)?;
        native.is_null(f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::TEST262_AGENT_STOPPED_BEFORE_RECEIVING_A_BROADCAST,
            &output,
            f,
        )?;
        f.instruction(&Instruction::Else);
        let resource = schema
            .reserve_gc_local(f)
            .initialize(native.into_resource(schema, f), f);
        let realm = schema
            .reserve_gc_local(f)
            .initialize(self.emit_current_function_realm(f), f);
        let prototype = schema.reserve_value_local(f);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::SharedArrayBufferPrototype,
            &prototype,
            f,
        );
        let header = schema.reserve_gc_local(f).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(&prototype), f)?,
            f,
        );
        let buffer = schema.reserve_gc_local(f).initialize(
            schema.struct_type::<SharedArrayBuffer>().construct(
                (
                    GcOperand::reference(&header, schema),
                    GcOperand::reference(&resource, schema),
                ),
                f,
            ),
            f,
        );
        let buffer_value = schema.reserve_value_local(f);
        let id_value = schema.reserve_value_local(f);
        buffer_value.set_reference(&buffer, schema, f);
        id.load(f);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::I64ExtendI32S);
        f.instruction(&Instruction::F64ConvertI64S);
        f.instruction(&Instruction::I64ReinterpretF64);
        id.store(f);
        id_value.set_number(id, f);
        let message = self.emit_pre_evaluated_arg_vector(&[&buffer_value, &id_value], f);
        let pair = self.emit_array_from_argument_list(&message, f)?;
        output.value().set_reference(&pair, schema, f);
        pair.clear(f);
        message.clear(f);
        id_value.clear(f);
        buffer_value.clear(f);
        buffer.clear(f);
        header.clear(f);
        prototype.clear(f);
        realm.clear(f);
        resource.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        schema.release_i64_local(id, f);
        output.clear(f);
        Ok(())
    }

    pub(crate) fn compile_host_agent_get_report_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let output = schema.reserve_completion(f);
        let length = schema.reserve_i64_local(f);
        let zero = schema.reserve_i64_local(f);
        let status = schema.reserve_i64_local(f);
        f.instruction(&Instruction::I64Const(0));
        zero.store(f);
        output.initialize(f);
        self.emit_agent_scalar_call(AgentHostOperation::ReportLength, zero, zero, length, f)?;
        length.load(f);
        f.instruction(&Instruction::I64Const(0));
        f.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, f);
        output.value().set_scalar(ScalarValue::Null, f);
        f.instruction(&Instruction::Else);
        let pointer = self.emit_agent_wire_extent(length, f)?;
        self.emit_agent_scalar_call(AgentHostOperation::ReportCopy, pointer, length, status, f)?;
        status.load(f);
        length.load(f);
        f.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let count = schema.reserve_i32_local(f);
        let index = schema.reserve_i32_local(f);
        let byte = schema.reserve_i32_local(f);
        length.load(f);
        f.instruction(&Instruction::I32WrapI64);
        count.store(f);
        let bytes = schema.reserve_gc_local(f).initialize(
            schema
                .array_type::<ByteArray>()
                .filled(GcOperand::i32(0), count, f),
            f,
        );
        f.instruction(&Instruction::I32Const(0));
        index.store(f);
        let done = self.open_frame(ControlFrameKind::Block, f);
        let next = self.open_frame(ControlFrameKind::Loop, f);
        index.load(f);
        count.load(f);
        f.instruction(&Instruction::I32GeU);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_branch_to_target(done, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        pointer.load(f);
        f.instruction(&Instruction::I32WrapI64);
        index.load(f);
        f.instruction(&Instruction::I32Add);
        f.instruction(&Instruction::I32Load8U(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));
        byte.store(f);
        schema.array_type::<ByteArray>().write(
            &bytes,
            index,
            GcOperand::i32_local(byte),
            schema,
            f,
        );
        index.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        index.store(f);
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        let reader = IntlByteArrayReader::new(&bytes, schema, f);
        let string = reader.consume_remaining_utf8(schema, f);
        reader.finish(schema, f);
        output.value().set_reference(&string, schema, f);
        string.clear(f);
        bytes.clear(f);
        schema.release_i32_local(byte, f);
        schema.release_i32_local(index, f);
        schema.release_i32_local(count, f);
        schema.release_i64_local(pointer, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        schema.release_i64_local(status, f);
        schema.release_i64_local(zero, f);
        schema.release_i64_local(length, f);
        output.clear(f);
        Ok(())
    }

    pub(crate) fn compile_host_agent_sleep_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        let output = schema.reserve_completion(f);
        let pending = schema.reserve_completion(f);
        let zero = schema.reserve_i64_local(f);
        let status = schema.reserve_i64_local(f);
        f.instruction(&Instruction::I64Const(0));
        zero.store(f);
        self.emit_builtin_arg_to_value(0, &value, f);
        output.initialize(f);
        self.emit_value_to_number_payload(&value, &pending, f)?;
        pending.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        output.copy_from(&pending, f);
        f.instruction(&Instruction::Else);
        self.emit_agent_scalar_call(
            AgentHostOperation::Sleep,
            pending.value().scalar(),
            zero,
            status,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        schema.release_i64_local(status, f);
        schema.release_i64_local(zero, f);
        pending.clear(f);
        output.clear(f);
        value.clear(f);
        Ok(())
    }
    pub(crate) fn compile_host_agent_monotonic_now_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_agent_no_argument_builtin(AgentHostOperation::MonotonicNow, f)
    }
    pub(crate) fn compile_host_agent_leaving_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_agent_no_argument_builtin(AgentHostOperation::Leaving, f)
    }
    fn emit_agent_no_argument_builtin(
        &mut self,
        operation: AgentHostOperation,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let zero = schema.reserve_i64_local(f);
        let result = schema.reserve_i64_local(f);
        f.instruction(&Instruction::I64Const(0));
        zero.store(f);
        self.emit_agent_scalar_call(operation, zero, zero, result, f)?;
        self.completion().initialize(f);
        if operation == AgentHostOperation::MonotonicNow {
            self.completion().value().set_number(result, f);
        }
        schema.release_i64_local(result, f);
        schema.release_i64_local(zero, f);
        Ok(())
    }
}
