use super::super::*;
use crate::gc_types::*;

/// The element types admitted by GetViewValue and SetViewValue. Clamped bytes
/// belong to TypedArray storage and cannot enter this domain.
#[derive(Clone, Copy)]
pub(super) enum DataViewElement {
    Int8,
    Uint8,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Float16,
    Float32,
    Float64,
    BigInt64,
    BigUint64,
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_data_view_access_builtin(
        &mut self,
        operation: DataViewAccess,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let out = s.reserve_completion(f);
        out.initialize(f);
        let pending = s.reserve_completion(f);
        let receiver = s.reserve_value_local(f);
        receiver.copy_from(
            self.body_entry_locals()
                .expect("DataView method")
                .this_value(),
            f,
        );
        let index_arg = s.reserve_value_local(f);
        let value = s.reserve_value_local(f);
        let endian = s.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &index_arg, f);
        match operation {
            DataViewAccess::Get(_) => self.emit_builtin_arg_to_value(1, &endian, f),
            DataViewAccess::Set(_) => {
                self.emit_builtin_arg_to_value(1, &value, f);
                self.emit_builtin_arg_to_value(2, &endian, f);
            }
        }
        let index = s.reserve_i64_local(f);
        let word = s.reserve_i64_local(f);
        let length = s.reserve_i64_local(f);
        let offset = s.reserve_i64_local(f);
        let address = s.reserve_i64_local(f);
        let shift = s.reserve_i64_local(f);
        let cursor = s.reserve_i64_local(f);
        let little = s.reserve_i32_local(f);
        let valid = s.reserve_i32_local(f);
        let byte = s.reserve_i32_local(f);
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
        if matches!(operation, DataViewAccess::Set(_)) {
            self.emit_binary_buffer_immutable_i32(&owner, f);
            self.open_frame(ControlFrameKind::If, f);
            self.emit_binary_type_error(
                RuntimeErrorMessage::DATAVIEW_BACKING_BUFFER_IS_IMMUTABLE,
                &out,
                exit,
                f,
            )?;
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        self.emit_to_index_i64_from_value_locals(
            &index_arg,
            index,
            operation.index_error(),
            &pending,
            f,
        )?;
        self.emit_binary_abrupt_exit(&pending, &out, exit, f);
        if matches!(operation, DataViewAccess::Set(_)) {
            self.emit_data_view_value_to_word(operation.element(), &value, word, &pending, f)?;
            self.emit_binary_abrupt_exit(&pending, &out, exit, f);
        }
        self.compile_truthy_tagged_i32(&endian, f)?;
        little.store(f);
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
        index.load(f);
        length.load(f);
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I64Const(operation.element().byte_width()));
        length.load(f);
        index.load(f);
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_binary_range_error(operation.positive_bound_error(), &out, exit, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        s.field(BufferViewSchema::BYTE_OFFSET)
            .read(&view, s, f)
            .store_i64(offset, f);
        if matches!(operation, DataViewAccess::Get(_)) {
            f.instruction(&Instruction::I64Const(0));
            word.store(f);
        }
        f.instruction(&Instruction::I64Const(0));
        cursor.store(f);
        let done = self.open_frame(ControlFrameKind::Block, f);
        let next = self.open_frame(ControlFrameKind::Loop, f);
        cursor.load(f);
        f.instruction(&Instruction::I64Const(operation.element().byte_width()));
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(done, f);
        offset.load(f);
        index.load(f);
        f.instruction(&Instruction::I64Add);
        cursor.load(f);
        f.instruction(&Instruction::I64Add);
        address.store(f);
        little.load(f);
        f.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        cursor.load(f);
        f.instruction(&Instruction::Else);
        f.instruction(&Instruction::I64Const(operation.element().byte_width() - 1));
        cursor.load(f);
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::I64Const(3));
        f.instruction(&Instruction::I64Shl);
        shift.store(f);
        match operation {
            DataViewAccess::Get(_) => {
                access.read_byte(address, byte, self, f)?;
                word.load(f);
                byte.load(f);
                f.instruction(&Instruction::I64ExtendI32U);
                shift.load(f);
                f.instruction(&Instruction::I64Shl);
                f.instruction(&Instruction::I64Or);
                word.store(f);
            }
            DataViewAccess::Set(_) => {
                word.load(f);
                shift.load(f);
                f.instruction(&Instruction::I64ShrU);
                f.instruction(&Instruction::I32WrapI64);
                byte.store(f);
                access.write_byte(address, byte, self, f)?;
            }
        }
        self.emit_increment_local(cursor, 1, f);
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        if matches!(operation, DataViewAccess::Get(_)) {
            self.emit_data_view_word_to_value(operation.element(), word, out.value(), f);
        }
        access.clear(s, f);
        owner.clear(f);
        view.clear(f);
        object.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&out, f);
        s.release_i32_local(byte, f);
        s.release_i32_local(valid, f);
        s.release_i32_local(little, f);
        s.release_i64_local(cursor, f);
        s.release_i64_local(shift, f);
        s.release_i64_local(address, f);
        s.release_i64_local(offset, f);
        s.release_i64_local(length, f);
        s.release_i64_local(word, f);
        s.release_i64_local(index, f);
        endian.clear(f);
        value.clear(f);
        index_arg.clear(f);
        receiver.clear(f);
        pending.clear(f);
        out.clear(f);
        Ok(())
    }

    fn emit_data_view_value_to_word(
        &mut self,
        element: DataViewElement,
        input: &ValueLocals,
        word: I64Local,
        pending: &CompletionLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        use DataViewElement::*;
        if matches!(element, BigInt64 | BigUint64) {
            return self.emit_to_bigint_u64_word_from_value_locals(input, word, pending, f);
        }
        self.emit_value_to_number_payload(input, pending, f)?;
        pending.kind().load(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        match element {
            Int8 | Uint8 | Int16 | Uint16 | Int32 | Uint32 => {
                self.emit_to_uint32_i64_from_number_payload(pending.value().scalar(), word, f)
            }
            Float32 => {
                pending.value().scalar().load(f);
                f.instruction(&Instruction::F64ReinterpretI64);
                f.instruction(&Instruction::F32DemoteF64);
                f.instruction(&Instruction::I32ReinterpretF32);
                f.instruction(&Instruction::I64ExtendI32U);
                word.store(f);
            }
            Float64 => {
                pending.value().scalar().load(f);
                word.store(f);
            }
            Float16 => {
                let s = self.runtime_schema();
                let scratch: [I64Local; 6] = std::array::from_fn(|_| s.reserve_i64_local(f));
                self.emit_f64_payload_to_half_bits_local(
                    pending.value().scalar(),
                    word,
                    scratch[0],
                    scratch[1],
                    scratch[2],
                    scratch[3],
                    scratch[4],
                    scratch[5],
                    f,
                );
                for local in scratch.into_iter().rev() {
                    s.release_i64_local(local, f);
                }
            }
            BigInt64 | BigUint64 => unreachable!("handled BigInt coercion"),
        }
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(())
    }

    pub(super) fn emit_data_view_word_to_value(
        &mut self,
        element: DataViewElement,
        word: I64Local,
        out: &ValueLocals,
        f: &mut Function,
    ) {
        use DataViewElement::*;
        let s = self.runtime_schema();
        let bits = s.reserve_i64_local(f);
        match element {
            Float64 => {
                word.load(f);
                bits.store(f);
            }
            Float32 => {
                word.load(f);
                f.instruction(&Instruction::I32WrapI64);
                f.instruction(&Instruction::F32ReinterpretI32);
                f.instruction(&Instruction::F64PromoteF32);
                f.instruction(&Instruction::I64ReinterpretF64);
                bits.store(f);
            }
            Float16 => {
                let scratch: [I64Local; 5] = std::array::from_fn(|_| s.reserve_i64_local(f));
                self.emit_half_bits_to_f64_payload(
                    word, scratch[0], scratch[1], scratch[2], scratch[3], scratch[4], f,
                );
                f.instruction(&Instruction::I64ReinterpretF64);
                bits.store(f);
                for local in scratch.into_iter().rev() {
                    s.release_i64_local(local, f);
                }
            }
            Int8 | Int16 | Int32 | Uint8 | Uint16 | Uint32 => {
                word.load(f);
                match element {
                    Int8 => {
                        f.instruction(&Instruction::I64Extend8S);
                    }
                    Int16 => {
                        f.instruction(&Instruction::I64Extend16S);
                    }
                    Int32 => {
                        f.instruction(&Instruction::I64Extend32S);
                    }
                    Uint8 | Uint16 | Uint32 => {}
                    _ => unreachable!(),
                };
                f.instruction(&if matches!(element, Int8 | Int16 | Int32) {
                    Instruction::F64ConvertI64S
                } else {
                    Instruction::F64ConvertI64U
                });
                f.instruction(&Instruction::I64ReinterpretF64);
                bits.store(f);
            }
            BigInt64 | BigUint64 => {
                let count = s.reserve_i32_local(f);
                let index = s.reserve_i32_local(f);
                let negative = s.reserve_i32_local(f);
                f.instruction(&Instruction::I32Const(1));
                count.store(f);
                f.instruction(&Instruction::I32Const(0));
                index.store(f);
                f.instruction(&Instruction::I32Const(0));
                negative.store(f);
                word.load(f);
                bits.store(f);
                if matches!(element, BigInt64) {
                    word.load(f);
                    f.instruction(&Instruction::I64Const(0));
                    f.instruction(&Instruction::I64LtS);
                    negative.store(f);
                    negative.load(f);
                    self.open_frame(ControlFrameKind::If, f);
                    f.instruction(&Instruction::I64Const(0));
                    word.load(f);
                    f.instruction(&Instruction::I64Sub);
                    bits.store(f);
                    self.pop_control(ControlFrameKind::If);
                    f.instruction(&Instruction::End);
                }
                let construction = BigIntConstruction::allocate(s, s.reserve_gc_local(f), count, f);
                construction.write(index, bits, s, f);
                let bigint = s
                    .reserve_gc_local(f)
                    .initialize(construction.publish(negative, s, f), f);
                out.set_reference(&bigint, s, f);
                bigint.clear(f);
                s.release_i32_local(negative, f);
                s.release_i32_local(index, f);
                s.release_i32_local(count, f);
            }
        }
        if !matches!(element, BigInt64 | BigUint64) {
            out.set_number(bits, f);
        }
        s.release_i64_local(bits, f);
    }
}

impl DataViewElement {
    const fn byte_width(self) -> i64 {
        match self {
            Self::Int8 | Self::Uint8 => 1,
            Self::Int16 | Self::Uint16 | Self::Float16 => 2,
            Self::Int32 | Self::Uint32 | Self::Float32 => 4,
            Self::Float64 | Self::BigInt64 | Self::BigUint64 => 8,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum DataViewAccess {
    Get(DataViewElement),
    Set(DataViewElement),
}

impl DataViewAccess {
    const fn element(self) -> DataViewElement {
        match self {
            Self::Get(element) | Self::Set(element) => element,
        }
    }

    const fn index_error(self) -> RuntimeErrorMessage {
        use DataViewElement::*;
        match self {
            Self::Get(Int8) => RuntimeErrorMessage::DATAVIEW_GETINT8_INDEX_OUT_OF_BOUNDS,
            Self::Get(Uint8) => RuntimeErrorMessage::DATAVIEW_GETUINT8_INDEX_OUT_OF_BOUNDS,
            Self::Get(Int16) => RuntimeErrorMessage::DATAVIEW_GETINT16_INDEX_OUT_OF_BOUNDS,
            Self::Get(Uint16) => RuntimeErrorMessage::DATAVIEW_GETUINT16_INDEX_OUT_OF_BOUNDS,
            Self::Get(Int32) => RuntimeErrorMessage::DATAVIEW_GETINT32_INDEX_OUT_OF_BOUNDS,
            Self::Get(Uint32) => RuntimeErrorMessage::DATAVIEW_GETUINT32_INDEX_OUT_OF_BOUNDS,
            Self::Get(Float16) => RuntimeErrorMessage::DATAVIEW_GETFLOAT16_INDEX_OUT_OF_BOUNDS,
            Self::Get(Float32) => RuntimeErrorMessage::DATAVIEW_GETFLOAT32_INDEX_OUT_OF_BOUNDS,
            Self::Get(Float64) => RuntimeErrorMessage::DATAVIEW_GETFLOAT64_INDEX_OUT_OF_BOUNDS,
            Self::Get(BigInt64) => RuntimeErrorMessage::DATAVIEW_GETBIGINT64_INDEX_OUT_OF_BOUNDS,
            Self::Get(BigUint64) => RuntimeErrorMessage::DATAVIEW_GETBIGUINT64_INDEX_OUT_OF_BOUNDS,
            Self::Set(Int8 | Uint8) => RuntimeErrorMessage::DATAVIEW_SETUINT8_INDEX_OUT_OF_BOUNDS,
            Self::Set(Int16 | Uint16) => {
                RuntimeErrorMessage::DATAVIEW_SETUINT16_INDEX_OUT_OF_BOUNDS
            }
            Self::Set(Int32 | Uint32) => {
                RuntimeErrorMessage::DATAVIEW_SETUINT32_INDEX_OUT_OF_BOUNDS
            }
            Self::Set(Float16) => RuntimeErrorMessage::DATAVIEW_SETFLOAT16_INDEX_OUT_OF_BOUNDS,
            Self::Set(Float32) => RuntimeErrorMessage::DATAVIEW_SETFLOAT32_INDEX_OUT_OF_BOUNDS,
            Self::Set(Float64) => RuntimeErrorMessage::DATAVIEW_SETFLOAT64_INDEX_OUT_OF_BOUNDS,
            Self::Set(BigInt64) => RuntimeErrorMessage::DATAVIEW_SETBIGINT64_INDEX_OUT_OF_BOUNDS,
            Self::Set(BigUint64) => RuntimeErrorMessage::DATAVIEW_SETBIGUINT64_INDEX_OUT_OF_BOUNDS,
        }
    }

    const fn positive_bound_error(self) -> RuntimeErrorMessage {
        use DataViewElement::*;
        match self {
            // Preserve the existing eight-bit positive-bound message aliases.
            Self::Get(Int8 | Uint8) | Self::Set(Int8 | Uint8) => {
                RuntimeErrorMessage::DATAVIEW_GETUINT8_INDEX_OUT_OF_BOUNDS
            }
            Self::Get(
                Int16 | Uint16 | Int32 | Uint32 | Float16 | Float32 | Float64 | BigInt64
                | BigUint64,
            )
            | Self::Set(
                Int16 | Uint16 | Int32 | Uint32 | Float16 | Float32 | Float64 | BigInt64
                | BigUint64,
            ) => self.index_error(),
        }
    }
}
