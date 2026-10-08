use super::regexp_protocol::NativeRegExpFlag;
use super::*;
use crate::builtins::regexp::{RegExpProgramLayoutFailure, ValidatedRegExpProgramLayoutLocals};
use crate::runtime_helpers::{
    RegExpMatcherArguments, RegExpMatcherStatus, REGEXP_MATCHER_SCRATCH_MAX_BYTES,
};

impl FunctionBuilder<'_> {
    /// The caller supplies the already acquired exec method: Get occurs exactly
    /// once, and the original whole value survives every observable call.
    pub(crate) fn emit_regexp_exec_from_values(
        &mut self,
        regexp: &ValueLocals,
        input: &GcLocal<StringValue>,
        acquired_exec: &ValueLocals,
        result: &CompletionLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        result.initialize(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        self.emit_is_callable_i32(acquired_exec, f)?;
        self.open_frame(ControlFrameKind::If, f);
        let argument = s.reserve_value_local(f);
        argument.set_reference(input, s, f);
        let arguments = self.emit_pre_evaluated_arg_vector(&[&argument], f);
        self.emit_function_or_proxy_call_with_argv(acquired_exec, regexp, &arguments, result, f)?;
        arguments.clear(f);
        argument.clear(f);
        self.emit_native_string_abrupt_exit(result, result, exit, f);
        self.emit_is_heap_object_like_tag_i32(result.value().tag(), f);
        result.value().tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Null.tag()));
        f.instruction(&Instruction::I32Eq);
        f.instruction(&Instruction::I32Or);
        f.instruction(&Instruction::I32Eqz);
        self.emit_native_string_error_if(
            RuntimeErrorMessage::REGEXP_PROTOTYPE_SYMBOL_MATCH_EXEC_RESULT_IS_NOT_OBJECT_OR_NULL,
            NativeErrorKind::TypeError,
            result,
            exit,
            f,
        )?;
        f.instruction(&Instruction::Else);
        regexp.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            s.reference_type::<RegExpObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        f.instruction(&Instruction::I32Eqz);
        self.emit_native_string_error_if(
            RuntimeErrorMessage::REGEXP_PROTOTYPE_EXEC_RECEIVER_IS_NOT_REGEXP,
            NativeErrorKind::TypeError,
            result,
            exit,
            f,
        )?;
        let object = s
            .reserve_gc_local(f)
            .initialize(regexp.cast_reference::<RegExpObject>(s, f), f);
        self.emit_native_regexp_builtin_exec(&object, regexp, input, result, f)?;
        object.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_regexp_prototype_exec_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let receiver = s.reserve_value_local(f);
        let argument = s.reserve_value_local(f);
        let pending = s.reserve_completion(f);
        let output = s.reserve_completion(f);
        self.compile_this_to_locals(&receiver, f)?;
        self.emit_builtin_arg_to_value(0, &argument, f);
        output.initialize(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        receiver.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            s.reference_type::<RegExpObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        f.instruction(&Instruction::I32Eqz);
        self.emit_native_string_error_if(
            RuntimeErrorMessage::REGEXP_PROTOTYPE_EXEC_RECEIVER_IS_NOT_REGEXP,
            NativeErrorKind::TypeError,
            &output,
            exit,
            f,
        )?;
        let object = s
            .reserve_gc_local(f)
            .initialize(receiver.cast_reference::<RegExpObject>(s, f), f);
        self.emit_value_to_string_payload(&argument, &pending, f)?;
        self.emit_native_string_abrupt_exit(&pending, &output, exit, f);
        let input = s
            .reserve_gc_local(f)
            .initialize(pending.value().cast_reference::<StringValue>(s, f), f);
        self.emit_native_regexp_builtin_exec(&object, &receiver, &input, &output, f)?;
        input.clear(f);
        object.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        output.clear(f);
        pending.clear(f);
        argument.clear(f);
        receiver.clear(f);
        Ok(())
    }

    pub(crate) fn emit_regexp_prototype_test_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let receiver = s.reserve_value_local(f);
        let argument = s.reserve_value_local(f);
        let acquired = s.reserve_value_local(f);
        let pending = s.reserve_completion(f);
        let output = s.reserve_completion(f);
        let truth = s.reserve_i32_local(f);
        self.compile_this_to_locals(&receiver, f)?;
        self.emit_builtin_arg_to_value(0, &argument, f);
        output.initialize(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        self.emit_is_heap_object_like_tag_i32(receiver.tag(), f);
        f.instruction(&Instruction::I32Eqz);
        self.emit_native_string_error_if(
            RuntimeErrorMessage::REGEXP_PROTOTYPE_TEST_RECEIVER_IS_NOT_AN_OBJECT,
            NativeErrorKind::TypeError,
            &output,
            exit,
            f,
        )?;
        self.emit_value_to_string_payload(&argument, &pending, f)?;
        self.emit_native_string_abrupt_exit(&pending, &output, exit, f);
        let input = s
            .reserve_gc_local(f)
            .initialize(pending.value().cast_reference::<StringValue>(s, f), f);
        self.emit_native_regexp_get(&receiver, "exec", &pending, f)?;
        self.emit_native_string_abrupt_exit(&pending, &output, exit, f);
        acquired.copy_from(pending.value(), f);
        self.emit_regexp_exec_from_values(&receiver, &input, &acquired, &pending, f)?;
        self.emit_native_string_abrupt_exit(&pending, &output, exit, f);
        pending.value().tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Null.tag()));
        f.instruction(&Instruction::I32Ne);
        truth.store(f);
        argument.set_boolean(truth, f);
        output.set_normal(&argument, f);
        input.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        s.release_i32_local(truth, f);
        output.clear(f);
        pending.clear(f);
        acquired.clear(f);
        argument.clear(f);
        receiver.clear(f);
        Ok(())
    }
}

#[must_use = "matcher scratch must be scrubbed and rewound before an observable exit"]
struct NativeRegExpScratch {
    base: I64Local,
    size: I64Local,
    mark: I64Local,
}
impl NativeRegExpScratch {
    fn finish(self, b: &mut FunctionBuilder<'_>, f: &mut Function) {
        self.base.load(f);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::I32Const(0));
        self.size.load(f);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::MemoryFill(0));
        self.mark.load(f);
        f.instruction(&Instruction::GlobalSet(PRIVATE_BYTE_CURSOR_GLOBAL_INDEX));
        let s = b.runtime_schema();
        s.release_i64_local(self.mark, f);
        s.release_i64_local(self.size, f);
        s.release_i64_local(self.base, f);
    }
}

impl FunctionBuilder<'_> {
    fn emit_native_regexp_scratch(
        &mut self,
        layout: &ValidatedRegExpProgramLayoutLocals,
        result: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<NativeRegExpScratch, EmitError> {
        let s = self.runtime_schema();
        let size = s.reserve_i64_local(f);
        layout.capture_count().load(f);
        f.instruction(&Instruction::I64Const(16));
        f.instruction(&Instruction::I64Mul);
        size.store(f);
        size.load(f);
        f.instruction(&Instruction::I64Const(
            REGEXP_MATCHER_SCRATCH_MAX_BYTES as i64,
        ));
        f.instruction(&Instruction::I64GeU);
        self.emit_native_string_error_if(RuntimeErrorMessage::REGEXP_MATCHER_SCRATCH_ARENA_EXCEEDS_THE_ENGINE_ADDRESSABLE_RESOURCE_LIMIT,NativeErrorKind::RangeError,result,exit,f)?;
        // The caller owns only the capture result prefix. The matcher reserves
        // its live state and grows ordered alternatives after immutable input
        // materialization, under the same total scratch ceiling.
        let mark = s.reserve_i64_local(f);
        let base = s.reserve_i64_local(f);
        f.instruction(&Instruction::GlobalGet(PRIVATE_BYTE_CURSOR_GLOBAL_INDEX));
        mark.store(f);
        self.emit_regexp_capture_output_allocation(size, base, result, exit, f)?;
        Ok(NativeRegExpScratch { base, size, mark })
    }

    fn emit_native_regexp_builtin_exec(
        &mut self,
        object: &GcLocal<RegExpObject>,
        receiver: &ValueLocals,
        input: &GcLocal<StringValue>,
        result: &CompletionLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let pending = s.reserve_completion(f);
        let last = s.reserve_value_local(f);
        let input_length = s.reserve_i64_local(f);
        let start = s.reserve_i64_local(f);
        let matched_start = s.reserve_i64_local(f);
        let matched_end = s.reserve_i64_local(f);
        let global = s.reserve_i32_local(f);
        let sticky = s.reserve_i32_local(f);
        let unicode = s.reserve_i32_local(f);
        let other = s.reserve_i32_local(f);
        let matcher_flags = s.reserve_i32_local(f);
        let status = s.reserve_i32_local(f);
        let found = s.reserve_i32_local(f);
        result.initialize(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        self.emit_native_gc_string_length(input, input_length, f);
        self.emit_native_regexp_get(receiver, "lastIndex", &pending, f)?;
        self.emit_native_string_abrupt_exit(&pending, result, exit, f);
        last.copy_from(pending.value(), f);
        self.emit_to_length_i64_from_value_locals(&last, start, &pending, f)?;
        self.emit_native_string_abrupt_exit(&pending, result, exit, f);
        // Coercion of lastIndex may have recompiled the receiver. Read all
        // semantic slots only after that coercion has completed.
        let flags = s.reserve_gc_local(f).initialize(
            s.field(RegExpObjectSchema::ORIGINAL_FLAGS)
                .read(object, s, f)
                .reference(),
            f,
        );
        self.emit_native_regexp_has_flag(&flags, NativeRegExpFlag::Global, global, f);
        self.emit_native_regexp_has_flag(&flags, NativeRegExpFlag::Sticky, sticky, f);
        global.load(f);
        sticky.load(f);
        f.instruction(&Instruction::I32Or);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::I64Const(0));
        start.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        start.load(f);
        input_length.load(f);
        f.instruction(&Instruction::I64GtU);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::I64Const(0));
        matched_end.store(f);
        self.emit_native_regexp_set_last_index(receiver, matched_end, &pending, f)?;
        self.emit_native_string_abrupt_exit(&pending, result, exit, f);
        last.set_scalar(ScalarValue::Null, f);
        result.set_normal(&last, f);
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        sticky.load(f);
        matcher_flags.store(f);
        self.emit_native_regexp_has_flag(&flags, NativeRegExpFlag::Unicode, unicode, f);
        self.emit_native_regexp_has_flag(&flags, NativeRegExpFlag::UnicodeSets, other, f);
        unicode.load(f);
        other.load(f);
        f.instruction(&Instruction::I32Or);
        unicode.store(f);
        unicode.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Shl);
        matcher_flags.load(f);
        f.instruction(&Instruction::I32Or);
        matcher_flags.store(f);
        for (flag, shift) in [
            (NativeRegExpFlag::Multiline, 2),
            (NativeRegExpFlag::DotAll, 3),
        ] {
            self.emit_native_regexp_has_flag(&flags, flag, other, f);
            other.load(f);
            f.instruction(&Instruction::I32Const(shift));
            f.instruction(&Instruction::I32Shl);
            matcher_flags.load(f);
            f.instruction(&Instruction::I32Or);
            matcher_flags.store(f);
        }
        let nullable = s.reserve_gc_local(f).initialize(
            s.field(RegExpObjectSchema::PROGRAM)
                .read(object, s, f)
                .reference(),
            f,
        );
        nullable.load(s, f);
        f.instruction(&Instruction::RefIsNull);
        self.emit_native_string_error_if(
            RuntimeErrorMessage::REGEXP_COMPILED_PROGRAM_MATCHER_FAILED,
            NativeErrorKind::Error,
            result,
            exit,
            f,
        )?;
        let program = s
            .reserve_gc_local(f)
            .initialize(nullable.load(s, f).require_non_null(f), f);
        nullable.clear(f);
        let layout_pending = self.reserve_regexp_program_layout(f);
        let layout = self.emit_validate_regexp_program_layout(
            &program,
            layout_pending,
            RegExpProgramLayoutFailure::Exec { result, exit },
            f,
        )?;
        let scratch = self.emit_native_regexp_scratch(&layout, result, exit, f)?;
        let cleanup = self.open_frame(ControlFrameKind::Block, f);
        s.call_helper(
            RegExpMatcherArguments::new(&program, input, start, matcher_flags, scratch.base),
            self.runtime_helper_base()?,
            f,
        )
        .store(found, matched_start, matched_end, status, f);
        status.load(f);
        f.instruction(&Instruction::I32Const(
            RegExpMatcherStatus::Complete.abi_word() as i32,
        ));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        found.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32GtU);
        self.emit_native_string_error_if(
            RuntimeErrorMessage::REGEXP_COMPILED_PROGRAM_MATCHER_FAILED,
            NativeErrorKind::Error,
            result,
            cleanup,
            f,
        )?;
        found.load(f);
        self.open_frame(ControlFrameKind::If, f);
        matched_start.load(f);
        matched_end.load(f);
        f.instruction(&Instruction::I64GtU);
        matched_end.load(f);
        input_length.load(f);
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        self.emit_native_string_error_if(
            RuntimeErrorMessage::REGEXP_COMPILED_PROGRAM_MATCHER_FAILED,
            NativeErrorKind::Error,
            result,
            cleanup,
            f,
        )?;
        global.load(f);
        sticky.load(f);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_native_regexp_set_last_index(receiver, matched_end, &pending, f)?;
        self.emit_native_string_abrupt_exit(&pending, result, cleanup, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_native_regexp_match_array(
            input,
            input_length,
            &flags,
            &layout,
            scratch.base,
            matched_start,
            matched_end,
            result,
            cleanup,
            f,
        )?;
        self.emit_native_string_abrupt_exit(result, result, cleanup, f);
        self.emit_regexp_legacy_match_update(
            object,
            input,
            input_length,
            &layout,
            matched_start,
            matched_end,
            result,
            f,
        );
        f.instruction(&Instruction::Else);
        global.load(f);
        sticky.load(f);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::I64Const(0));
        matched_end.store(f);
        self.emit_native_regexp_set_last_index(receiver, matched_end, &pending, f)?;
        self.emit_native_string_abrupt_exit(&pending, result, cleanup, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        last.set_scalar(ScalarValue::Null, f);
        result.set_normal(&last, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        scratch.finish(self, f);
        for outcome in RegExpMatcherStatus::ALL {
            match *outcome {
                RegExpMatcherStatus::Complete => {}
                RegExpMatcherStatus::Failed(failure) => {
                    status.load(f);
                    f.instruction(&Instruction::I32Const(failure.abi_word() as i32));
                    f.instruction(&Instruction::I32Eq);
                    let kind=match failure.route() {
                        crate::runtime_helpers::RegExpMatcherFailureRoute::GenericError=>NativeErrorKind::Error,
                        crate::runtime_helpers::RegExpMatcherFailureRoute::CurrentFunctionRealmRangeError=>NativeErrorKind::RangeError,
                    };
                    self.emit_native_string_error_if(failure.message(), kind, result, exit, f)?;
                }
            }
        }
        status.load(f);
        f.instruction(&Instruction::I32Const(
            RegExpMatcherStatus::Complete.abi_word() as i32,
        ));
        f.instruction(&Instruction::I32Ne);
        self.emit_native_string_error_if(
            RuntimeErrorMessage::REGEXP_COMPILED_PROGRAM_MATCHER_FAILED,
            NativeErrorKind::Error,
            result,
            exit,
            f,
        )?;
        self.release_regexp_program_layout(layout, f);
        program.clear(f);
        flags.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        s.release_i32_local(found, f);
        s.release_i32_local(status, f);
        s.release_i32_local(matcher_flags, f);
        s.release_i32_local(other, f);
        s.release_i32_local(unicode, f);
        s.release_i32_local(sticky, f);
        s.release_i32_local(global, f);
        s.release_i64_local(matched_end, f);
        s.release_i64_local(matched_start, f);
        s.release_i64_local(start, f);
        s.release_i64_local(input_length, f);
        last.clear(f);
        pending.clear(f);
        Ok(())
    }

    pub(super) fn emit_native_string_result_array(
        &mut self,
        length: I64Local,
        f: &mut Function,
    ) -> Result<GcLocal<ArrayObject>, EmitError> {
        let s = self.runtime_schema();
        let prototype = self.emit_load_current_function_realm_array_prototype(f);
        Ok(s.reserve_gc_local(f).initialize(
            self.emit_alloc_array_with_current_function_realm_prototype(length, prototype, f)?,
            f,
        ))
    }

    pub(super) fn emit_native_string_array_write(
        &mut self,
        array: &GcLocal<ArrayObject>,
        index: I64Local,
        value: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let stored = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(value, f), f);
        let undefined = s.reserve_value_local(f);
        undefined.set_undefined(f);
        let accessor = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(&undefined, f), f);
        let descriptor = s.reserve_gc_local(f).initialize(
            s.struct_type::<PropertyDescriptor>().construct(
                (
                    GcOperand::descriptor_word(
                        crate::heap::StoredPropertyAttributes::Data {
                            writable: true,
                            enumerable: true,
                            configurable: true,
                        }
                        .descriptor_word(),
                    ),
                    GcOperand::reference(&stored, s),
                    GcOperand::reference(&accessor, s),
                    GcOperand::reference(&accessor, s),
                ),
                f,
            ),
            f,
        );
        self.emit_array_indexed_publish_descriptor(array, index, &descriptor, f)?;
        descriptor.clear(f);
        accessor.clear(f);
        undefined.clear(f);
        stored.clear(f);
        Ok(())
    }

    pub(super) fn emit_native_string_array_read(
        &mut self,
        array: &GcLocal<ArrayObject>,
        index: I64Local,
        value: &ValueLocals,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        let descriptor = self.emit_array_indexed_descriptor(array, index, f);
        let present = s
            .reserve_gc_local(f)
            .initialize(descriptor.load(s, f).require_non_null(f), f);
        let stored = s.reserve_gc_local(f).initialize(
            s.field(PropertyDescriptorSchema::VALUE)
                .read(&present, s, f)
                .reference(),
            f,
        );
        s.struct_type::<StoredValue>()
            .read_into(&stored, value, s, f);
        stored.clear(f);
        present.clear(f);
        descriptor.clear(f);
    }

    fn emit_native_regexp_metadata(
        &mut self,
        header: &GcLocal<OrdinaryObject>,
        name: &str,
        value: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let key = self.emit_native_regexp_key(name, f);
        self.emit_object_append_data_property_with_flags(header, &key, value, true, true, true, f)?;
        key.clear(f);
        Ok(())
    }

    fn emit_native_regexp_capture_bounds(
        &mut self,
        scratch: I64Local,
        capture: I64Local,
        start: I64Local,
        end: I64Local,
        f: &mut Function,
    ) {
        capture.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64Const(16));
        f.instruction(&Instruction::I64Mul);
        scratch.load(f);
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::I64Load(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        }));
        start.store(f);
        capture.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64Const(16));
        f.instruction(&Instruction::I64Mul);
        scratch.load(f);
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::I64Load(MemArg {
            offset: 8,
            align: 3,
            memory_index: 0,
        }));
        end.store(f);
    }

    fn emit_native_regexp_match_array(
        &mut self,
        input: &GcLocal<StringValue>,
        input_length: I64Local,
        flags: &GcLocal<StringValue>,
        layout: &ValidatedRegExpProgramLayoutLocals,
        scratch: I64Local,
        matched_start: I64Local,
        matched_end: I64Local,
        result: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let length = s.reserve_i64_local(f);
        let index = s.reserve_i64_local(f);
        let start = s.reserve_i64_local(f);
        let end = s.reserve_i64_local(f);
        let zero = s.reserve_i64_local(f);
        let one = s.reserve_i64_local(f);
        let two = s.reserve_i64_local(f);
        let bits = s.reserve_i64_local(f);
        let has_indices = s.reserve_i32_local(f);
        let value = s.reserve_value_local(f);
        let pair_value = s.reserve_value_local(f);
        layout.capture_count().load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        length.store(f);
        let matches = self.emit_native_string_result_array(length, f)?;
        // Indices are an internal unpublished array until the d flag publishes
        // the edge. Pair identity is shared by numeric and named entries.
        self.emit_native_regexp_has_flag(flags, NativeRegExpFlag::HasIndices, has_indices, f);
        let indices_length = s.reserve_i64_local(f);
        f.instruction(&Instruction::I64Const(0));
        indices_length.store(f);
        has_indices.load(f);
        self.open_frame(ControlFrameKind::If, f);
        length.load(f);
        indices_length.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let indices = self.emit_native_string_result_array(indices_length, f)?;
        s.release_i64_local(indices_length, f);
        f.instruction(&Instruction::I64Const(0));
        zero.store(f);
        f.instruction(&Instruction::I64Const(1));
        one.store(f);
        f.instruction(&Instruction::I64Const(2));
        two.store(f);
        f.instruction(&Instruction::I64Const(0));
        index.store(f);
        let done = self.open_frame(ControlFrameKind::Block, f);
        let next = self.open_frame(ControlFrameKind::Loop, f);
        index.load(f);
        length.load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(done, f);
        index.load(f);
        f.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, f);
        matched_start.load(f);
        start.store(f);
        matched_end.load(f);
        end.store(f);
        f.instruction(&Instruction::Else);
        self.emit_native_regexp_capture_bounds(scratch, index, start, end, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        value.set_undefined(f);
        pair_value.set_undefined(f);
        start.load(f);
        f.instruction(&Instruction::I64Const(-1));
        f.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, f);
        end.load(f);
        f.instruction(&Instruction::I64Const(-1));
        f.instruction(&Instruction::I64Ne);
        self.emit_native_string_error_if(
            RuntimeErrorMessage::REGEXP_COMPILED_PROGRAM_MATCHER_FAILED,
            NativeErrorKind::Error,
            result,
            exit,
            f,
        )?;
        f.instruction(&Instruction::Else);
        start.load(f);
        end.load(f);
        f.instruction(&Instruction::I64GtU);
        end.load(f);
        input_length.load(f);
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        self.emit_native_string_error_if(
            RuntimeErrorMessage::REGEXP_COMPILED_PROGRAM_MATCHER_FAILED,
            NativeErrorKind::Error,
            result,
            exit,
            f,
        )?;
        let captured = s
            .reserve_gc_local(f)
            .initialize(self.emit_gc_string_slice(input, start, end, f), f);
        value.set_reference(&captured, s, f);
        captured.clear(f);
        has_indices.load(f);
        self.open_frame(ControlFrameKind::If, f);
        let pair = self.emit_native_string_result_array(two, f)?;
        start.load(f);
        f.instruction(&Instruction::F64ConvertI64U);
        f.instruction(&Instruction::I64ReinterpretF64);
        bits.store(f);
        pair_value.set_number(bits, f);
        self.emit_native_string_array_write(&pair, zero, &pair_value, f)?;
        end.load(f);
        f.instruction(&Instruction::F64ConvertI64U);
        f.instruction(&Instruction::I64ReinterpretF64);
        bits.store(f);
        pair_value.set_number(bits, f);
        self.emit_native_string_array_write(&pair, one, &pair_value, f)?;
        pair_value.set_reference(&pair, s, f);
        pair.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_native_string_array_write(&matches, index, &value, f)?;
        has_indices.load(f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_native_string_array_write(&indices, index, &pair_value, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        index.store(f);
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        let header = s.reserve_gc_local(f).initialize(
            s.field(ArrayObjectSchema::OBJECT)
                .read(&matches, s, f)
                .reference(),
            f,
        );
        matched_start.load(f);
        f.instruction(&Instruction::F64ConvertI64U);
        f.instruction(&Instruction::I64ReinterpretF64);
        bits.store(f);
        value.set_number(bits, f);
        self.emit_native_regexp_metadata(&header, "index", &value, f)?;
        value.set_reference(input, s, f);
        self.emit_native_regexp_metadata(&header, "input", &value, f)?;
        self.emit_native_regexp_named_groups(
            layout,
            scratch,
            &matches,
            &indices,
            has_indices,
            result,
            exit,
            &value,
            &pair_value,
            f,
        )?;
        self.emit_native_regexp_metadata(&header, "groups", &value, f)?;
        has_indices.load(f);
        self.open_frame(ControlFrameKind::If, f);
        let indices_header = s.reserve_gc_local(f).initialize(
            s.field(ArrayObjectSchema::OBJECT)
                .read(&indices, s, f)
                .reference(),
            f,
        );
        self.emit_native_regexp_metadata(&indices_header, "groups", &pair_value, f)?;
        indices_header.clear(f);
        value.set_reference(&indices, s, f);
        self.emit_native_regexp_metadata(&header, "indices", &value, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        value.set_reference(&matches, s, f);
        result.set_normal(&value, f);
        header.clear(f);
        indices.clear(f);
        matches.clear(f);
        pair_value.clear(f);
        value.clear(f);
        s.release_i32_local(has_indices, f);
        s.release_i64_local(bits, f);
        s.release_i64_local(two, f);
        s.release_i64_local(one, f);
        s.release_i64_local(zero, f);
        s.release_i64_local(end, f);
        s.release_i64_local(start, f);
        s.release_i64_local(index, f);
        s.release_i64_local(length, f);
        Ok(())
    }
}

impl FunctionBuilder<'_> {
    fn emit_native_regexp_named_groups(
        &mut self,
        layout: &ValidatedRegExpProgramLayoutLocals,
        scratch: I64Local,
        matches: &GcLocal<ArrayObject>,
        indices: &GcLocal<ArrayObject>,
        has_indices: I32Local,
        result: &CompletionLocals,
        exit: ControlTarget,
        groups: &ValueLocals,
        index_groups: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let failure = RegExpProgramLayoutFailure::Exec { result, exit };
        groups.set_undefined(f);
        index_groups.set_undefined(f);
        layout.named_groups().load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        let offset = s.reserve_i64_local(f);
        let magic = s.reserve_i64_local(f);
        let count = s.reserve_i64_local(f);
        let total = s.reserve_i64_local(f);
        let ordinal = s.reserve_i64_local(f);
        let row = s.reserve_i64_local(f);
        let payload = s.reserve_i64_local(f);
        let name_start = s.reserve_i64_local(f);
        let name_length = s.reserve_i64_local(f);
        let candidates = s.reserve_i64_local(f);
        let candidate_count = s.reserve_i64_local(f);
        let candidate_index = s.reserve_i64_local(f);
        let capture = s.reserve_i64_local(f);
        let selected = s.reserve_i64_local(f);
        let start = s.reserve_i64_local(f);
        let end = s.reserve_i64_local(f);
        let extent = s.reserve_i64_local(f);
        let expected_candidate = s.reserve_i64_local(f);
        let candidates_end = s.reserve_i64_local(f);
        let expected_name = s.reserve_i64_local(f);
        let records = s.reserve_i64_local(f);
        layout.named_groups().load(f);
        offset.store(f);
        layout.read_word(offset, magic, self, failure, f)?;
        magic.load(f);
        f.instruction(&Instruction::I64Const(
            lila_ir::REGEXP_NAMED_GROUP_TABLE_MAGIC_VERSION as i64,
        ));
        f.instruction(&Instruction::I64Ne);
        self.emit_native_string_error_if(
            RuntimeErrorMessage::REGEXP_COMPILED_PROGRAM_MATCHER_FAILED,
            NativeErrorKind::Error,
            result,
            exit,
            f,
        )?;
        for (delta, destination) in [(8, count), (16, total), (24, records)] {
            layout.named_groups().load(f);
            f.instruction(&Instruction::I64Const(delta));
            f.instruction(&Instruction::I64Add);
            offset.store(f);
            layout.read_word(offset, destination, self, failure, f)?;
        }
        layout.end().load(f);
        layout.named_groups().load(f);
        f.instruction(&Instruction::I64Sub);
        extent.store(f);
        records.load(f);
        f.instruction(&Instruction::I64Const(32));
        f.instruction(&Instruction::I64Ne);
        count.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Or);
        count.load(f);
        layout.capture_count().load(f);
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        total.load(f);
        layout.capture_count().load(f);
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        count.load(f);
        extent.load(f);
        f.instruction(&Instruction::I64Const(32));
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64Const(24));
        f.instruction(&Instruction::I64DivU);
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        self.emit_native_string_error_if(
            RuntimeErrorMessage::REGEXP_COMPILED_PROGRAM_MATCHER_FAILED,
            NativeErrorKind::Error,
            result,
            exit,
            f,
        )?;
        count.load(f);
        f.instruction(&Instruction::I64Const(24));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Const(32));
        f.instruction(&Instruction::I64Add);
        expected_candidate.store(f);
        total.load(f);
        extent.load(f);
        expected_candidate.load(f);
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64Const(8));
        f.instruction(&Instruction::I64DivU);
        f.instruction(&Instruction::I64GtU);
        self.emit_native_string_error_if(
            RuntimeErrorMessage::REGEXP_COMPILED_PROGRAM_MATCHER_FAILED,
            NativeErrorKind::Error,
            result,
            exit,
            f,
        )?;
        total.load(f);
        f.instruction(&Instruction::I64Const(8));
        f.instruction(&Instruction::I64Mul);
        expected_candidate.load(f);
        f.instruction(&Instruction::I64Add);
        candidates_end.store(f);
        candidates_end.load(f);
        expected_name.store(f);
        let null = s.reserve_value_local(f);
        null.set_scalar(ScalarValue::Null, f);
        let group_object = s.reserve_gc_local(f).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(&null), f)?,
            f,
        );
        let index_group_object = s.reserve_gc_local(f).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(&null), f)?,
            f,
        );
        null.clear(f);
        let value = s.reserve_value_local(f);
        let indexed = s.reserve_value_local(f);
        f.instruction(&Instruction::I64Const(0));
        ordinal.store(f);
        let done = self.open_frame(ControlFrameKind::Block, f);
        let next = self.open_frame(ControlFrameKind::Loop, f);
        ordinal.load(f);
        count.load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(done, f);
        ordinal.load(f);
        f.instruction(&Instruction::I64Const(24));
        f.instruction(&Instruction::I64Mul);
        records.load(f);
        f.instruction(&Instruction::I64Add);
        layout.named_groups().load(f);
        f.instruction(&Instruction::I64Add);
        row.store(f);
        row.load(f);
        offset.store(f);
        layout.read_word(offset, payload, self, failure, f)?;
        row.load(f);
        f.instruction(&Instruction::I64Const(8));
        f.instruction(&Instruction::I64Add);
        offset.store(f);
        layout.read_word(offset, candidates, self, failure, f)?;
        row.load(f);
        f.instruction(&Instruction::I64Const(16));
        f.instruction(&Instruction::I64Add);
        offset.store(f);
        layout.read_word(offset, candidate_count, self, failure, f)?;
        payload.load(f);
        f.instruction(&Instruction::I64Const(32));
        f.instruction(&Instruction::I64ShrU);
        name_start.store(f);
        payload.load(f);
        f.instruction(&Instruction::I64Const(u32::MAX as i64));
        f.instruction(&Instruction::I64And);
        name_length.store(f);
        candidates.load(f);
        expected_candidate.load(f);
        f.instruction(&Instruction::I64Ne);
        name_start.load(f);
        expected_name.load(f);
        f.instruction(&Instruction::I64Ne);
        f.instruction(&Instruction::I32Or);
        name_length.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Or);
        candidate_count.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Or);
        candidate_count.load(f);
        candidates_end.load(f);
        expected_candidate.load(f);
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64Const(8));
        f.instruction(&Instruction::I64DivU);
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        self.emit_native_string_error_if(
            RuntimeErrorMessage::REGEXP_COMPILED_PROGRAM_MATCHER_FAILED,
            NativeErrorKind::Error,
            result,
            exit,
            f,
        )?;
        name_start.load(f);
        extent.load(f);
        f.instruction(&Instruction::I64GtU);
        self.emit_native_string_error_if(
            RuntimeErrorMessage::REGEXP_COMPILED_PROGRAM_MATCHER_FAILED,
            NativeErrorKind::Error,
            result,
            exit,
            f,
        )?;
        name_length.load(f);
        extent.load(f);
        name_start.load(f);
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64GtU);
        self.emit_native_string_error_if(
            RuntimeErrorMessage::REGEXP_COMPILED_PROGRAM_MATCHER_FAILED,
            NativeErrorKind::Error,
            result,
            exit,
            f,
        )?;
        name_start.load(f);
        name_length.load(f);
        f.instruction(&Instruction::I64Add);
        expected_name.store(f);
        candidate_count.load(f);
        f.instruction(&Instruction::I64Const(8));
        f.instruction(&Instruction::I64Mul);
        expected_candidate.load(f);
        f.instruction(&Instruction::I64Add);
        expected_candidate.store(f);
        f.instruction(&Instruction::I64Const(0));
        candidate_index.store(f);
        f.instruction(&Instruction::I64Const(0));
        selected.store(f);
        let candidates_done = self.open_frame(ControlFrameKind::Block, f);
        let candidate_next = self.open_frame(ControlFrameKind::Loop, f);
        candidate_index.load(f);
        candidate_count.load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(candidates_done, f);
        candidate_index.load(f);
        f.instruction(&Instruction::I64Const(8));
        f.instruction(&Instruction::I64Mul);
        candidates.load(f);
        f.instruction(&Instruction::I64Add);
        layout.named_groups().load(f);
        f.instruction(&Instruction::I64Add);
        offset.store(f);
        layout.read_word(offset, capture, self, failure, f)?;
        capture.load(f);
        f.instruction(&Instruction::I64Eqz);
        capture.load(f);
        layout.capture_count().load(f);
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        self.emit_native_string_error_if(
            RuntimeErrorMessage::REGEXP_COMPILED_PROGRAM_MATCHER_FAILED,
            NativeErrorKind::Error,
            result,
            exit,
            f,
        )?;
        self.emit_native_regexp_capture_bounds(scratch, capture, start, end, f);
        start.load(f);
        f.instruction(&Instruction::I64Const(-1));
        f.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, f);
        selected.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        self.emit_native_string_error_if(
            RuntimeErrorMessage::REGEXP_COMPILED_PROGRAM_MATCHER_FAILED,
            NativeErrorKind::Error,
            result,
            exit,
            f,
        )?;
        capture.load(f);
        selected.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        candidate_index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        candidate_index.store(f);
        self.emit_branch_to_target(candidate_next, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        value.set_undefined(f);
        indexed.set_undefined(f);
        selected.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_native_string_array_read(matches, selected, &value, f);
        has_indices.load(f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_native_string_array_read(indices, selected, &indexed, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        name_start.load(f);
        layout.named_groups().load(f);
        f.instruction(&Instruction::I64Add);
        name_start.store(f);
        let name = s.reserve_gc_local(f).initialize(
            layout.read_utf8_string(name_start, name_length, self, failure, f)?,
            f,
        );
        let key = PropertyKeyLocals::from_string(s, &name, f);
        name.clear(f);
        self.emit_object_append_data_property_with_flags(
            &group_object,
            &key,
            &value,
            true,
            true,
            true,
            f,
        )?;
        has_indices.load(f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_object_append_data_property_with_flags(
            &index_group_object,
            &key,
            &indexed,
            true,
            true,
            true,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        key.clear(f);
        ordinal.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        ordinal.store(f);
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        expected_candidate.load(f);
        candidates_end.load(f);
        f.instruction(&Instruction::I64Ne);
        expected_name.load(f);
        extent.load(f);
        f.instruction(&Instruction::I64Ne);
        f.instruction(&Instruction::I32Or);
        self.emit_native_string_error_if(
            RuntimeErrorMessage::REGEXP_COMPILED_PROGRAM_MATCHER_FAILED,
            NativeErrorKind::Error,
            result,
            exit,
            f,
        )?;
        groups.set_reference(&group_object, s, f);
        has_indices.load(f);
        self.open_frame(ControlFrameKind::If, f);
        index_groups.set_reference(&index_group_object, s, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        indexed.clear(f);
        value.clear(f);
        index_group_object.clear(f);
        group_object.clear(f);
        for local in [
            records,
            expected_name,
            candidates_end,
            expected_candidate,
            extent,
            end,
            start,
            selected,
            capture,
            candidate_index,
            candidate_count,
            candidates,
            name_length,
            name_start,
            payload,
            row,
            ordinal,
            total,
            count,
            magic,
            offset,
        ] {
            s.release_i64_local(local, f);
        }
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(())
    }
}
