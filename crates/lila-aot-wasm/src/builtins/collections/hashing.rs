use super::*;
impl FunctionBuilder<'_> {
    // Equal primitive keys have equal content hashes. Object/Symbol identity
    // stays in a mutable GC field and never becomes a reference address.
    pub(super) fn emit_collection_key_hash(
        &mut self,
        key: &ValueLocals,
        hash: I64Local,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        key.scalar().load(f);
        hash.store(f);
        key.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Number.tag()));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        hash.load(f);
        f.instruction(&Instruction::I64Const(i64::MAX));
        f.instruction(&Instruction::I64And);
        f.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::I64Const(0));
        hash.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        key.scalar().load(f);
        f.instruction(&Instruction::F64ReinterpretI64);
        key.scalar().load(f);
        f.instruction(&Instruction::F64ReinterpretI64);
        f.instruction(&Instruction::F64Ne);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::I64Const(0x7ff8_0000_0000_0000));
        hash.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        key.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::String.tag()));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        let text = s
            .reserve_gc_local(f)
            .initialize(key.cast_reference::<StringValue>(s, f), f);
        let units = s.reserve_gc_local(f).initialize(
            s.struct_type::<StringValue>()
                .field(StringValueSchema::CODE_UNITS)
                .read(&text, s, f)
                .reference(),
            f,
        );
        let index = s.reserve_i32_local(f);
        let count = s.reserve_i32_local(f);
        let unit = s.reserve_i32_local(f);
        s.array_type::<CodeUnitArray>().length(&units, s, f);
        count.store(f);
        f.instruction(&Instruction::I32Const(0));
        index.store(f);
        f.instruction(&Instruction::I64Const(-3750763034362895579));
        hash.store(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        index.load(f);
        count.load(f);
        f.instruction(&Instruction::I32GeU);
        self.emit_branch_if_to_target(exit, f);
        s.array_type::<CodeUnitArray>()
            .read(&units, index, s, f)
            .store(unit, f);
        hash.load(f);
        unit.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        f.instruction(&Instruction::I64Xor);
        f.instruction(&Instruction::I64Const(1099511628211));
        f.instruction(&Instruction::I64Mul);
        hash.store(f);
        index.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        index.store(f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        s.release_i32_local(unit, f);
        s.release_i32_local(count, f);
        s.release_i32_local(index, f);
        units.clear(f);
        text.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        key.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::BigInt.tag()));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        let bigint = s
            .reserve_gc_local(f)
            .initialize(key.cast_reference::<BigIntValue>(s, f), f);
        let limbs = s.reserve_gc_local(f).initialize(
            s.struct_type::<BigIntValue>()
                .field(BigIntValueSchema::LIMBS)
                .read(&bigint, s, f)
                .reference(),
            f,
        );
        let index = s.reserve_i32_local(f);
        let count = s.reserve_i32_local(f);
        let negative = s.reserve_i32_local(f);
        let limb = s.reserve_i64_local(f);
        s.struct_type::<BigIntValue>()
            .field(BigIntValueSchema::NEGATIVE)
            .read(&bigint, s, f)
            .store(negative, f);
        negative.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        hash.store(f);
        s.array_type::<BigIntLimbArray>().length(&limbs, s, f);
        count.store(f);
        f.instruction(&Instruction::I32Const(0));
        index.store(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        index.load(f);
        count.load(f);
        f.instruction(&Instruction::I32GeU);
        self.emit_branch_if_to_target(exit, f);
        s.array_type::<BigIntLimbArray>()
            .read(&limbs, index, s, f)
            .store_i64(limb, f);
        hash.load(f);
        limb.load(f);
        f.instruction(&Instruction::I64Xor);
        f.instruction(&Instruction::I64Const(1099511628211));
        f.instruction(&Instruction::I64Mul);
        hash.store(f);
        index.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        index.store(f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        s.release_i64_local(limb, f);
        s.release_i32_local(negative, f);
        s.release_i32_local(count, f);
        s.release_i32_local(index, f);
        limbs.clear(f);
        bigint.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        key.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Symbol.tag()));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        let symbol = s
            .reserve_gc_local(f)
            .initialize(key.cast_reference::<SymbolValue>(s, f), f);
        s.struct_type::<SymbolValue>()
            .field(SymbolValueSchema::KEY_HASH_ID)
            .read(&symbol, s, f)
            .store_i64(hash, f);
        hash.load(f);
        f.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, f);
        s.emit_next_collection_key_hash_id(hash, f);
        s.struct_type::<SymbolValue>()
            .field(SymbolValueSchema::KEY_HASH_ID)
            .write(&symbol, GcOperand::i64_local(hash), s, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        symbol.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_is_heap_object_like_tag_i32(key.tag(), f);
        self.open_frame(ControlFrameKind::If, f);
        let header = s
            .reserve_gc_local(f)
            .initialize(self.emit_object_header_projection(key, f), f);
        s.struct_type::<OrdinaryObject>()
            .field(OrdinaryObjectSchema::KEY_HASH_ID)
            .read(&header, s, f)
            .store_i64(hash, f);
        hash.load(f);
        f.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, f);
        s.emit_next_collection_key_hash_id(hash, f);
        s.struct_type::<OrdinaryObject>()
            .field(OrdinaryObjectSchema::KEY_HASH_ID)
            .write(&header, GcOperand::i64_local(hash), s, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        header.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        // Final mixing spreads scalar/id sequences before bucket reduction.
        hash.load(f);
        key.tag().load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        f.instruction(&Instruction::I64Xor);
        hash.store(f);
        for (shift, multiplier) in [(33, -49064778989728563i64), (33, -4265267296055464877i64)] {
            hash.load(f);
            hash.load(f);
            f.instruction(&Instruction::I64Const(shift));
            f.instruction(&Instruction::I64ShrU);
            f.instruction(&Instruction::I64Xor);
            f.instruction(&Instruction::I64Const(multiplier));
            f.instruction(&Instruction::I64Mul);
            hash.store(f);
        }
        hash.load(f);
        hash.load(f);
        f.instruction(&Instruction::I64Const(33));
        f.instruction(&Instruction::I64ShrU);
        f.instruction(&Instruction::I64Xor);
        hash.store(f);
    }
    pub(super) fn emit_collection_hash_bucket(
        &self,
        table: &GcLocal<CollectionHashTable>,
        hash: I64Local,
        index: I32Local,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        let count = s.reserve_i32_local(f);
        s.array_type::<CollectionHashTable>().length(table, s, f);
        count.store(f);
        hash.load(f);
        count.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        f.instruction(&Instruction::I64RemU);
        f.instruction(&Instruction::I32WrapI64);
        index.store(f);
        s.release_i32_local(count, f);
    }
}
