//! FlattenIntoArray retains recursive invocation state in private GC frames.
use super::*;
use crate::gc_types::{ArrayFlattenFrame, ArrayFlattenFrameSchema, GcLocal, Nullable, StoredValue};

#[derive(Clone, Copy)]
enum FlattenMethod {
    Flat,
    FlatMap,
}

impl FlattenMethod {
    fn overflow(self) -> RuntimeErrorMessage {
        match self {
            Self::Flat => {
                RuntimeErrorMessage::ARRAY_PROTOTYPE_FLAT_RESULT_EXCEEDS_THE_MAXIMUM_SAFE_LENGTH
            }
            Self::FlatMap => {
                RuntimeErrorMessage::ARRAY_PROTOTYPE_FLATMAP_RESULT_EXCEEDS_THE_MAXIMUM_SAFE_LENGTH
            }
        }
    }
}

impl FunctionBuilder<'_> {
    pub(crate) fn compile_array_prototype_flat_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_array_flatten_builtin(FlattenMethod::Flat, f)
    }
    pub(crate) fn compile_array_prototype_flat_map_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_array_flatten_builtin(FlattenMethod::FlatMap, f)
    }

    fn emit_array_flatten_builtin(
        &mut self,
        method: FlattenMethod,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let receiver = s.reserve_value_local(f);
        let target = s.reserve_value_local(f);
        let source = s.reserve_value_local(f);
        let element = s.reserve_value_local(f);
        let mapper = s.reserve_value_local(f);
        let this_arg = s.reserve_value_local(f);
        let number = s.reserve_value_local(f);
        let pending = s.reserve_completion(f);
        let length = s.reserve_i64_local(f);
        let index = s.reserve_i64_local(f);
        let target_index = s.reserve_i64_local(f);
        let zero = s.reserve_i64_local(f);
        let relative = s.reserve_i64_local(f);
        let depth = s.reserve_f64_local(f);
        let present = s.reserve_i32_local(f);
        let mapping = s.reserve_i32_local(f);
        let flatten = s.reserve_i32_local(f);
        self.compile_this_to_locals(&receiver, f)?;
        self.emit_array_like_length_snapshot(&receiver, length, &pending, f)?;
        f.instruction(&Instruction::I64Const(0));
        zero.store(f);
        f.instruction(&Instruction::I64Const(0));
        target_index.store(f);
        f.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        depth.store(f);
        match method {
            FlattenMethod::Flat => {
                self.emit_builtin_arg_to_value(0, &element, f);
                element.tag().load(f);
                f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
                f.instruction(&Instruction::I32Ne);
                self.open_frame(ControlFrameKind::If, f);
                self.emit_value_to_number_payload(&element, &pending, f)?;
                self.emit_array_native_propagate(&pending, f);
                self.emit_to_integer_or_infinity_number_payload_from_number_payload(
                    pending.value().scalar(),
                    relative,
                    f,
                );
                relative.load(f);
                f.instruction(&Instruction::F64ReinterpretI64);
                depth.store(f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
            }
            FlattenMethod::FlatMap => {
                self.emit_builtin_arg_to_value(0, &mapper, f);
                self.emit_is_callable_i32(&mapper, f)?;
                f.instruction(&Instruction::I32Eqz);
                self.open_frame(ControlFrameKind::If, f);
                self.emit_throw_current_function_realm_type_error(
                    RuntimeErrorMessage::ARRAY_PROTOTYPE_FLATMAP_MAPPER_IS_NOT_CALLABLE,
                    &pending,
                    f,
                )?;
                self.emit_array_native_propagate(&pending, f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
                self.emit_builtin_arg_to_value(1, &this_arg, f);
            }
        }
        self.emit_array_species_create(&receiver, zero, &target, f)?;
        let frame: GcLocal<ArrayFlattenFrame, Nullable> =
            s.reserve_gc_local(f).initialize_null(s, f);
        let stored = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(&receiver, f), f);
        let initial = s.reserve_gc_local(f).initialize(
            s.struct_type::<ArrayFlattenFrame>().construct(
                (
                    GcOperand::reference(&stored, s),
                    GcOperand::i64(0),
                    GcOperand::i64_local(length),
                    GcOperand::f64_local(depth),
                    GcOperand::boolean(matches!(method, FlattenMethod::FlatMap)),
                    GcOperand::null(s),
                ),
                f,
            ),
            f,
        );
        frame.replace(initial.load(s, f).nullable(), f);
        initial.clear(f);
        stored.clear(f);
        let finished = self.open_frame(ControlFrameKind::Block, f);
        let next = self.open_frame(ControlFrameKind::Loop, f);
        frame.load(s, f).is_null(f);
        self.emit_branch_if_to_target(finished, f);
        let active = s
            .reserve_gc_local(f)
            .initialize(frame.load(s, f).require_non_null(f), f);
        s.field(ArrayFlattenFrameSchema::INDEX)
            .read(&active, s, f)
            .store_i64(index, f);
        s.field(ArrayFlattenFrameSchema::LENGTH)
            .read(&active, s, f)
            .store_i64(length, f);
        index.load(f);
        length.load(f);
        f.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, f);
        frame.replace(
            s.field(ArrayFlattenFrameSchema::PARENT)
                .read(&active, s, f)
                .reference(),
            f,
        );
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let stored = s.reserve_gc_local(f).initialize(
            s.field(ArrayFlattenFrameSchema::SOURCE)
                .read(&active, s, f)
                .reference(),
            f,
        );
        s.struct_type::<StoredValue>()
            .read_into(&stored, &source, s, f);
        stored.clear(f);
        self.emit_array_native_has_index(&source, index, present, f)?;
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        relative.store(f);
        s.field(ArrayFlattenFrameSchema::INDEX).write(
            &active,
            GcOperand::i64_local(relative),
            s,
            f,
        );
        present.load(f);
        f.instruction(&Instruction::I32Eqz);
        self.emit_branch_if_to_target(next, f);
        self.emit_typed_array_or_object_index_read_from_locals(&source, index, &pending, f)?;
        self.emit_array_native_propagate(&pending, f);
        element.copy_from(pending.value(), f);
        s.field(ArrayFlattenFrameSchema::MAPPING)
            .read(&active, s, f)
            .store(mapping, f);
        mapping.load(f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_array_native_number(index, &number, f);
        let argv = self.emit_pre_evaluated_arg_vector(&[&element, &number, &receiver], f);
        self.emit_function_or_proxy_call_with_argv(&mapper, &this_arg, &argv, &pending, f)?;
        argv.clear(f);
        self.emit_array_native_propagate(&pending, f);
        element.copy_from(pending.value(), f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::I32Const(0));
        flatten.store(f);
        s.field(ArrayFlattenFrameSchema::DEPTH)
            .read(&active, s, f)
            .store_f64(depth, f);
        depth.load(f);
        f.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        f.instruction(&Instruction::F64Gt);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_is_array_i32(&element, flatten, &pending, f)?;
        self.emit_array_native_propagate(&pending, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        flatten.load(f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_array_like_length_snapshot(&element, length, &pending, f)?;
        depth.load(f);
        f.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        f.instruction(&Instruction::F64Sub);
        depth.store(f);
        let stored = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(&element, f), f);
        let child = s.reserve_gc_local(f).initialize(
            s.struct_type::<ArrayFlattenFrame>().construct(
                (
                    GcOperand::reference(&stored, s),
                    GcOperand::i64(0),
                    GcOperand::i64_local(length),
                    GcOperand::f64_local(depth),
                    GcOperand::boolean(false),
                    GcOperand::nullable_reference(&active, s),
                ),
                f,
            ),
            f,
        );
        frame.replace(child.load(s, f).nullable(), f);
        child.clear(f);
        stored.clear(f);
        f.instruction(&Instruction::Else);
        target_index.load(f);
        f.instruction(&Instruction::I64Const(9_007_199_254_740_991));
        f.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_current_function_realm_type_error(method.overflow(), &pending, f)?;
        self.emit_array_native_propagate(&pending, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let key = self.emit_array_native_index_key(target_index, f)?;
        self.emit_create_data_property_or_throw(&target, &key, &element, &pending, f)?;
        key.clear(f);
        self.emit_array_native_propagate(&pending, f);
        target_index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        target_index.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().set_normal(&target, f);
        active.clear(f);
        frame.clear(f);
        for local in [flatten, mapping, present] {
            s.release_i32_local(local, f);
        }
        s.release_f64_local(depth, f);
        for local in [relative, zero, target_index, index, length] {
            s.release_i64_local(local, f);
        }
        pending.clear(f);
        number.clear(f);
        this_arg.clear(f);
        mapper.clear(f);
        element.clear(f);
        source.clear(f);
        target.clear(f);
        receiver.clear(f);
        Ok(())
    }
}
