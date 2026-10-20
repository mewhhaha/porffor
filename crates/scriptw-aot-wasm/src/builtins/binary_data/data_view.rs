use super::*;

impl FunctionBuilder<'_> {
    pub(in crate::builtins) fn emit_data_view_constructor_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let out = s.reserve_completion(f);
        out.initialize(f);
        let pending = s.reserve_completion(f);
        let source = s.reserve_value_local(f);
        let offset_arg = s.reserve_value_local(f);
        let length_arg = s.reserve_value_local(f);
        let new_target = s.reserve_value_local(f);
        new_target.copy_from(
            self.body_entry_locals()
                .expect("DataView constructor")
                .new_target(),
            f,
        );
        let offset = s.reserve_i64_local(f);
        let length = s.reserve_i64_local(f);
        let tracking = s.reserve_i32_local(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        self.emit_binary_require_new_target(
            &new_target,
            RuntimeErrorMessage::ARRAYBUFFER_CONSTRUCTOR_REQUIRES_NEW,
            &out,
            exit,
            f,
        )?;
        self.emit_builtin_arg_to_value(0, &source, f);
        self.emit_builtin_arg_to_value(1, &offset_arg, f);
        self.emit_builtin_arg_to_value(2, &length_arg, f);
        let owner = self.emit_binary_buffer_owner(&source, &out, exit, f)?;
        self.emit_to_index_i64_from_value_locals(
            &offset_arg,
            offset,
            RuntimeErrorMessage::DATAVIEW_BYTELENGTH_OUT_OF_BOUNDS,
            &pending,
            f,
        )?;
        self.emit_binary_abrupt_exit(&pending, &out, exit, f);
        let access = self.emit_binary_buffer_access(&owner, f);
        access.valid.load(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_binary_type_error(
            RuntimeErrorMessage::DATAVIEW_BACKING_BUFFER_IS_DETACHED,
            &out,
            exit,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        offset.load(f);
        access.length.load(f);
        f.instruction(&Instruction::I64GtU);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_binary_range_error(
            RuntimeErrorMessage::DATAVIEW_BYTELENGTH_OUT_OF_BOUNDS,
            &out,
            exit,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::I32Const(0));
        tracking.store(f);
        length_arg.tag().load(f);
        f.instruction(&Instruction::I32Const(
            WasmRuntimeValueTag::Undefined as i32,
        ));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        access.length.load(f);
        offset.load(f);
        f.instruction(&Instruction::I64Sub);
        length.store(f);
        self.emit_binary_buffer_resizable_i32(&owner, f);
        tracking.store(f);
        f.instruction(&Instruction::Else);
        self.emit_to_index_i64_from_value_locals(
            &length_arg,
            length,
            RuntimeErrorMessage::DATAVIEW_BYTELENGTH_OUT_OF_BOUNDS,
            &pending,
            f,
        )?;
        self.emit_binary_abrupt_exit(&pending, &out, exit, f);
        // The first explicit-length check uses the witness acquired before its
        // ToIndex hook. Late detachment/bounds are checked after prototype Get.
        length.load(f);
        access.length.load(f);
        offset.load(f);
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64GtU);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_binary_range_error(
            RuntimeErrorMessage::DATAVIEW_BYTELENGTH_OUT_OF_BOUNDS,
            &out,
            exit,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        access.clear(s, f);
        self.emit_get_prototype_from_constructor(
            &new_target,
            OrdinaryDefaultPrototype::DataView,
            &pending,
            f,
        )?;
        self.emit_binary_abrupt_exit(&pending, &out, exit, f);
        // Prototype lookup can detach or resize. The final length check is
        // required only when the original byteLength argument was supplied.
        let current = self.emit_binary_buffer_access(&owner, f);
        current.valid.load(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_binary_type_error(
            RuntimeErrorMessage::DATAVIEW_BACKING_BUFFER_IS_DETACHED,
            &out,
            exit,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        offset.load(f);
        current.length.load(f);
        f.instruction(&Instruction::I64GtU);
        length_arg.tag().load(f);
        f.instruction(&Instruction::I32Const(
            WasmRuntimeValueTag::Undefined as i32,
        ));
        f.instruction(&Instruction::I32Ne);
        length.load(f);
        current.length.load(f);
        offset.load(f);
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_binary_range_error(
            RuntimeErrorMessage::DATAVIEW_BYTELENGTH_OUT_OF_BOUNDS,
            &out,
            exit,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        current.clear(s, f);
        let view = s.reserve_gc_local(f).initialize(
            s.struct_type::<BufferView>().construct(
                (
                    GcOperand::reference(&owner, s),
                    GcOperand::i64_local(offset),
                    GcOperand::i64_local(length),
                    GcOperand::boolean_local(tracking),
                ),
                f,
            ),
            f,
        );
        let header = s.reserve_gc_local(f).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(pending.value()), f)?,
            f,
        );
        let result = s.reserve_gc_local(f).initialize(
            s.struct_type::<DataViewObject>().construct(
                (
                    GcOperand::reference(&header, s),
                    GcOperand::reference(&view, s),
                ),
                f,
            ),
            f,
        );
        out.value().set_reference(&result, s, f);
        result.clear(f);
        header.clear(f);
        view.clear(f);
        owner.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&out, f);
        s.release_i32_local(tracking, f);
        s.release_i64_local(length, f);
        s.release_i64_local(offset, f);
        new_target.clear(f);
        length_arg.clear(f);
        offset_arg.clear(f);
        source.clear(f);
        pending.clear(f);
        out.clear(f);
        Ok(())
    }

    pub(in crate::builtins) fn emit_data_view_accessor_builtin(
        &mut self,
        kind: DataViewAccessor,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let out = s.reserve_completion(f);
        out.initialize(f);
        let receiver = s.reserve_value_local(f);
        receiver.copy_from(
            self.body_entry_locals()
                .expect("DataView accessor")
                .this_value(),
            f,
        );
        let length = s.reserve_i64_local(f);
        let valid = s.reserve_i32_local(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let object = self.emit_binary_require_ref::<DataViewObject>(
            &receiver,
            RuntimeErrorMessage::DATAVIEW_ACCESSOR_REQUIRES_DATAVIEW,
            &out,
            exit,
            f,
        )?;
        let view = s.reserve_gc_local(f).initialize(
            s.field(DataViewObjectSchema::VIEW)
                .read(&object, s, f)
                .reference(),
            f,
        );
        let owner = s.reserve_gc_local(f).initialize(
            s.field(BufferViewSchema::BUFFER)
                .read(&view, s, f)
                .reference(),
            f,
        );
        match kind {
            DataViewAccessor::Buffer => self.emit_binary_buffer_value(&owner, out.value(), f),
            DataViewAccessor::ByteLength | DataViewAccessor::ByteOffset => {
                let access = self.emit_binary_buffer_access(&owner, f);
                self.emit_binary_view_length(&view, &access, length, valid, f);
                valid.load(f);
                f.instruction(&Instruction::I32Eqz);
                self.open_frame(ControlFrameKind::If, f);
                self.emit_binary_type_error(
                    RuntimeErrorMessage::DATAVIEW_BYTELENGTH_OUT_OF_BOUNDS,
                    &out,
                    exit,
                    f,
                )?;
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
                if matches!(kind, DataViewAccessor::ByteOffset) {
                    s.field(BufferViewSchema::BYTE_OFFSET)
                        .read(&view, s, f)
                        .store_i64(length, f);
                }
                length.load(f);
                f.instruction(&Instruction::F64ConvertI64U);
                f.instruction(&Instruction::I64ReinterpretF64);
                length.store(f);
                out.value().set_number(length, f);
                access.clear(s, f);
            }
        }
        owner.clear(f);
        view.clear(f);
        object.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&out, f);
        s.release_i32_local(valid, f);
        s.release_i64_local(length, f);
        receiver.clear(f);
        out.clear(f);
        Ok(())
    }
}
