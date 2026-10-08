use super::*;
// Every table index is an offset in a GC array. Tombstones retain history order.
impl FunctionBuilder<'_> {
    pub(super) fn emit_collection_alloc_map(
        &mut self,
        header: &GcLocal<OrdinaryObject>,
        f: &mut Function,
    ) -> GcLocal<MapObject> {
        let s = self.runtime_schema();
        let zero = s.reserve_i32_local(f);
        f.instruction(&Instruction::I32Const(0));
        zero.store(f);
        let entries = s.reserve_gc_local(f).initialize(
            s.array_type::<MapEntryTable>()
                .filled(GcOperand::null(s), zero, f),
            f,
        );
        f.instruction(&Instruction::I32Const(4));
        zero.store(f);
        let buckets = s.reserve_gc_local(f).initialize(
            s.array_type::<CollectionHashTable>()
                .filled(GcOperand::i64(0), zero, f),
            f,
        );
        let record = s.reserve_gc_local(f).initialize(
            s.struct_type::<MapObject>().construct(
                (
                    GcOperand::reference(header, s),
                    GcOperand::reference(&entries, s),
                    GcOperand::i64(0),
                    GcOperand::i64(0),
                    GcOperand::reference(&buckets, s),
                ),
                f,
            ),
            f,
        );
        buckets.clear(f);
        entries.clear(f);
        s.release_i32_local(zero, f);
        record
    }
    pub(super) fn emit_collection_find_map(
        &mut self,
        record: &GcLocal<MapObject>,
        key: &ValueLocals,
        index: I64Local,
        found: I32Local,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let cursor = s.reserve_i32_local(f);
        let bucket = s.reserve_i32_local(f);
        let chain = s.reserve_i64_local(f);
        let hash = s.reserve_i64_local(f);
        let retained = s.reserve_value_local(f);
        let table = s.reserve_gc_local(f).initialize(
            s.struct_type::<MapObject>()
                .field(MapObjectSchema::ENTRIES)
                .read(record, s, f)
                .reference(),
            f,
        );
        let buckets = s.reserve_gc_local(f).initialize(
            s.struct_type::<MapObject>()
                .field(MapObjectSchema::HASH_INDEX)
                .read(record, s, f)
                .reference(),
            f,
        );
        self.emit_collection_key_hash(key, hash, f);
        self.emit_collection_hash_bucket(&buckets, hash, bucket, f);
        s.array_type::<CollectionHashTable>()
            .read(&buckets, bucket, s, f)
            .store_i64(chain, f);
        f.instruction(&Instruction::I32Const(0));
        found.store(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        chain.load(f);
        f.instruction(&Instruction::I64Eqz);
        self.emit_branch_if_to_target(exit, f);
        chain.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        index.store(f);
        index.load(f);
        f.instruction(&Instruction::I32WrapI64);
        cursor.store(f);
        let entry = s.reserve_gc_local(f).initialize(
            s.array_type::<MapEntryTable>()
                .read(&table, cursor, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        let stored = s.reserve_gc_local(f).initialize(
            s.struct_type::<MapEntry>()
                .field(MapEntrySchema::KEY)
                .read(&entry, s, f)
                .reference(),
            f,
        );
        self.emit_stored_value_to_locals(&stored, &retained, f);
        self.emit_tagged_payload_same_value_zero_i32(&retained, key, f)?;
        found.store(f);
        s.struct_type::<MapEntry>()
            .field(MapEntrySchema::HASH_NEXT)
            .read(&entry, s, f)
            .store_i64(chain, f);
        stored.clear(f);
        entry.clear(f);
        found.load(f);
        self.emit_branch_if_to_target(exit, f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        found.load(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        s.struct_type::<MapObject>()
            .field(MapObjectSchema::HISTORY_LENGTH)
            .read(record, s, f)
            .store_i64(index, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        retained.clear(f);
        buckets.clear(f);
        table.clear(f);
        s.release_i64_local(hash, f);
        s.release_i64_local(chain, f);
        s.release_i32_local(bucket, f);
        s.release_i32_local(cursor, f);
        Ok(())
    }

    fn emit_collection_grow_map(
        &mut self,
        record: &GcLocal<MapObject>,
        count: I64Local,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        let capacity = s.reserve_i32_local(f);
        let cursor = s.reserve_i32_local(f);
        let next_capacity = s.reserve_i64_local(f);
        let table = s.reserve_gc_local(f).initialize(
            s.struct_type::<MapObject>()
                .field(MapObjectSchema::ENTRIES)
                .read(record, s, f)
                .reference(),
            f,
        );
        s.array_type::<MapEntryTable>().length(&table, s, f);
        capacity.store(f);
        count.load(f);
        f.instruction(&Instruction::I64Const(u32::MAX as i64));
        f.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        count.load(f);
        capacity.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        f.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, f);
        capacity.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        f.instruction(&Instruction::I64Const(2));
        f.instruction(&Instruction::I64Mul);
        next_capacity.store(f);
        next_capacity.load(f);
        f.instruction(&Instruction::I64Const(4));
        f.instruction(&Instruction::I64LtU);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::I64Const(4));
        next_capacity.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        next_capacity.load(f);
        f.instruction(&Instruction::I64Const(u32::MAX as i64));
        f.instruction(&Instruction::I64GtU);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::I64Const(u32::MAX as i64));
        next_capacity.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        next_capacity.load(f);
        f.instruction(&Instruction::I32WrapI64);
        capacity.store(f);
        let grown = s.reserve_gc_local(f).initialize(
            s.array_type::<MapEntryTable>()
                .filled(GcOperand::null(s), capacity, f),
            f,
        );
        f.instruction(&Instruction::I32Const(0));
        cursor.store(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        cursor.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        count.load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(exit, f);
        let entry = s.reserve_gc_local(f).initialize(
            s.array_type::<MapEntryTable>()
                .read(&table, cursor, s, f)
                .reference(),
            f,
        );
        s.array_type::<MapEntryTable>().write(
            &grown,
            cursor,
            GcOperand::reference(&entry, s),
            s,
            f,
        );
        entry.clear(f);
        cursor.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        cursor.store(f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        s.struct_type::<MapObject>()
            .field(MapObjectSchema::ENTRIES)
            .write(record, GcOperand::reference(&grown, s), s, f);
        grown.clear(f);
        self.emit_collection_rehash_map(record, capacity, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        table.clear(f);
        s.release_i64_local(next_capacity, f);
        s.release_i32_local(cursor, f);
        s.release_i32_local(capacity, f);
    }
    pub(super) fn emit_collection_clear_map(
        &mut self,
        record: &GcLocal<MapObject>,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        let cursor = s.reserve_i32_local(f);
        let count = s.reserve_i64_local(f);
        let table = s.reserve_gc_local(f).initialize(
            s.struct_type::<MapObject>()
                .field(MapObjectSchema::ENTRIES)
                .read(record, s, f)
                .reference(),
            f,
        );
        s.struct_type::<MapObject>()
            .field(MapObjectSchema::HISTORY_LENGTH)
            .read(record, s, f)
            .store_i64(count, f);
        f.instruction(&Instruction::I32Const(0));
        cursor.store(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        cursor.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        count.load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(exit, f);
        s.array_type::<MapEntryTable>()
            .write(&table, cursor, GcOperand::null(s), s, f);
        cursor.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        cursor.store(f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        s.struct_type::<MapObject>()
            .field(MapObjectSchema::LIVE_COUNT)
            .write(record, GcOperand::i64(0), s, f);
        let buckets = s.reserve_gc_local(f).initialize(
            s.struct_type::<MapObject>()
                .field(MapObjectSchema::HASH_INDEX)
                .read(record, s, f)
                .reference(),
            f,
        );
        let capacity = s.reserve_i32_local(f);
        s.array_type::<CollectionHashTable>().length(&buckets, s, f);
        capacity.store(f);
        f.instruction(&Instruction::I32Const(0));
        cursor.store(f);
        let cleared = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        cursor.load(f);
        capacity.load(f);
        f.instruction(&Instruction::I32GeU);
        self.emit_branch_if_to_target(cleared, f);
        s.array_type::<CollectionHashTable>()
            .write(&buckets, cursor, GcOperand::i64(0), s, f);
        cursor.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        cursor.store(f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        s.release_i32_local(capacity, f);
        buckets.clear(f);
        // HISTORY_LENGTH intentionally survives Clear; active cursors are live.
        table.clear(f);
        s.release_i64_local(count, f);
        s.release_i32_local(cursor, f);
    }
    pub(super) fn emit_collection_delete_map(
        &mut self,
        record: &GcLocal<MapObject>,
        key: &ValueLocals,
        removed: I32Local,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let index = s.reserve_i64_local(f);
        let cursor = s.reserve_i32_local(f);
        let live = s.reserve_i64_local(f);
        self.emit_collection_find_map(record, key, index, removed, f)?;
        removed.load(f);
        self.open_frame(ControlFrameKind::If, f);
        let table = s.reserve_gc_local(f).initialize(
            s.struct_type::<MapObject>()
                .field(MapObjectSchema::ENTRIES)
                .read(record, s, f)
                .reference(),
            f,
        );
        self.emit_collection_unlink_map(record, key, index, f);
        index.load(f);
        f.instruction(&Instruction::I32WrapI64);
        cursor.store(f);
        s.array_type::<MapEntryTable>()
            .write(&table, cursor, GcOperand::null(s), s, f);
        s.struct_type::<MapObject>()
            .field(MapObjectSchema::LIVE_COUNT)
            .read(record, s, f)
            .store_i64(live, f);
        self.emit_increment_local(live, -1, f);
        s.struct_type::<MapObject>()
            .field(MapObjectSchema::LIVE_COUNT)
            .write(record, GcOperand::i64_local(live), s, f);
        table.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        s.release_i64_local(live, f);
        s.release_i32_local(cursor, f);
        s.release_i64_local(index, f);
        Ok(())
    }
    pub(super) fn emit_collection_put_map(
        &mut self,
        record: &GcLocal<MapObject>,
        key: &ValueLocals,
        value: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let index = s.reserve_i64_local(f);
        let found = s.reserve_i32_local(f);
        let cursor = s.reserve_i32_local(f);
        let live = s.reserve_i64_local(f);
        self.emit_collection_find_map(record, key, index, found, f)?;
        found.load(f);
        self.open_frame(ControlFrameKind::If, f);
        let table = s.reserve_gc_local(f).initialize(
            s.struct_type::<MapObject>()
                .field(MapObjectSchema::ENTRIES)
                .read(record, s, f)
                .reference(),
            f,
        );
        index.load(f);
        f.instruction(&Instruction::I32WrapI64);
        cursor.store(f);
        let entry = s.reserve_gc_local(f).initialize(
            s.array_type::<MapEntryTable>()
                .read(&table, cursor, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        let stored = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(value, f), f);
        s.struct_type::<MapEntry>()
            .field(MapEntrySchema::VALUE)
            .write(&entry, GcOperand::reference(&stored, s), s, f);
        stored.clear(f);
        entry.clear(f);
        table.clear(f);
        f.instruction(&Instruction::Else);
        // Append never reuses a tombstone: deletion/reinsertion moves to the end.
        self.emit_collection_grow_map(record, index, f);
        let table = s.reserve_gc_local(f).initialize(
            s.struct_type::<MapObject>()
                .field(MapObjectSchema::ENTRIES)
                .read(record, s, f)
                .reference(),
            f,
        );
        let hash = s.reserve_i64_local(f);
        let head = s.reserve_i64_local(f);
        let bucket = s.reserve_i32_local(f);
        let buckets = s.reserve_gc_local(f).initialize(
            s.struct_type::<MapObject>()
                .field(MapObjectSchema::HASH_INDEX)
                .read(record, s, f)
                .reference(),
            f,
        );
        self.emit_collection_key_hash(key, hash, f);
        self.emit_collection_hash_bucket(&buckets, hash, bucket, f);
        s.array_type::<CollectionHashTable>()
            .read(&buckets, bucket, s, f)
            .store_i64(head, f);
        let stored_value = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(value, f), f);
        let stored_key = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(key, f), f);
        let entry = s.reserve_gc_local(f).initialize(
            s.struct_type::<MapEntry>().construct(
                (
                    GcOperand::reference(&stored_key, s),
                    GcOperand::reference(&stored_value, s),
                    GcOperand::i64_local(head),
                ),
                f,
            ),
            f,
        );
        stored_key.clear(f);
        index.load(f);
        f.instruction(&Instruction::I32WrapI64);
        cursor.store(f);
        s.array_type::<MapEntryTable>().write(
            &table,
            cursor,
            GcOperand::nullable_reference(&entry, s),
            s,
            f,
        );
        self.emit_increment_local(index, 1, f);
        s.array_type::<CollectionHashTable>().write(
            &buckets,
            bucket,
            GcOperand::i64_local(index),
            s,
            f,
        );
        s.struct_type::<MapObject>()
            .field(MapObjectSchema::HISTORY_LENGTH)
            .write(record, GcOperand::i64_local(index), s, f);
        s.struct_type::<MapObject>()
            .field(MapObjectSchema::LIVE_COUNT)
            .read(record, s, f)
            .store_i64(live, f);
        self.emit_increment_local(live, 1, f);
        s.struct_type::<MapObject>()
            .field(MapObjectSchema::LIVE_COUNT)
            .write(record, GcOperand::i64_local(live), s, f);
        buckets.clear(f);
        s.release_i32_local(bucket, f);
        s.release_i64_local(head, f);
        s.release_i64_local(hash, f);
        entry.clear(f);
        stored_value.clear(f);
        table.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        s.release_i64_local(live, f);
        s.release_i32_local(cursor, f);
        s.release_i32_local(found, f);
        s.release_i64_local(index, f);
        Ok(())
    }
    pub(super) fn emit_collection_read_map(
        &mut self,
        record: &GcLocal<MapObject>,
        index: I64Local,
        key: &ValueLocals,
        value: &ValueLocals,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        let cursor = s.reserve_i32_local(f);
        index.load(f);
        f.instruction(&Instruction::I32WrapI64);
        cursor.store(f);
        let table = s.reserve_gc_local(f).initialize(
            s.struct_type::<MapObject>()
                .field(MapObjectSchema::ENTRIES)
                .read(record, s, f)
                .reference(),
            f,
        );
        let entry = s.reserve_gc_local(f).initialize(
            s.array_type::<MapEntryTable>()
                .read(&table, cursor, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        let stored = s.reserve_gc_local(f).initialize(
            s.struct_type::<MapEntry>()
                .field(MapEntrySchema::KEY)
                .read(&entry, s, f)
                .reference(),
            f,
        );
        self.emit_stored_value_to_locals(&stored, key, f);
        stored.clear(f);
        let stored = s.reserve_gc_local(f).initialize(
            s.struct_type::<MapEntry>()
                .field(MapEntrySchema::VALUE)
                .read(&entry, s, f)
                .reference(),
            f,
        );
        self.emit_stored_value_to_locals(&stored, value, f);
        stored.clear(f);
        entry.clear(f);
        table.clear(f);
        s.release_i32_local(cursor, f);
    }
    // Advances through a fresh table and length on every step, including after callbacks.
    pub(super) fn emit_collection_next_map(
        &mut self,
        record: &GcLocal<MapObject>,
        index: I64Local,
        found: I32Local,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        let cursor = s.reserve_i32_local(f);
        let count = s.reserve_i64_local(f);
        let table = s.reserve_gc_local(f).initialize(
            s.struct_type::<MapObject>()
                .field(MapObjectSchema::ENTRIES)
                .read(record, s, f)
                .reference(),
            f,
        );
        s.struct_type::<MapObject>()
            .field(MapObjectSchema::HISTORY_LENGTH)
            .read(record, s, f)
            .store_i64(count, f);
        f.instruction(&Instruction::I32Const(0));
        found.store(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        index.load(f);
        count.load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(exit, f);
        index.load(f);
        f.instruction(&Instruction::I32WrapI64);
        cursor.store(f);
        s.array_type::<MapEntryTable>()
            .read(&table, cursor, s, f)
            .reference()
            .is_null(f);
        f.instruction(&Instruction::I32Eqz);
        found.store(f);
        found.load(f);
        self.emit_branch_if_to_target(exit, f);
        self.emit_increment_local(index, 1, f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        table.clear(f);
        s.release_i64_local(count, f);
        s.release_i32_local(cursor, f);
    }
    pub(super) fn emit_collection_alloc_set(
        &mut self,
        header: &GcLocal<OrdinaryObject>,
        f: &mut Function,
    ) -> GcLocal<SetObject> {
        let s = self.runtime_schema();
        let zero = s.reserve_i32_local(f);
        f.instruction(&Instruction::I32Const(0));
        zero.store(f);
        let entries = s.reserve_gc_local(f).initialize(
            s.array_type::<SetEntryTable>()
                .filled(GcOperand::null(s), zero, f),
            f,
        );
        f.instruction(&Instruction::I32Const(4));
        zero.store(f);
        let buckets = s.reserve_gc_local(f).initialize(
            s.array_type::<CollectionHashTable>()
                .filled(GcOperand::i64(0), zero, f),
            f,
        );
        let record = s.reserve_gc_local(f).initialize(
            s.struct_type::<SetObject>().construct(
                (
                    GcOperand::reference(header, s),
                    GcOperand::reference(&entries, s),
                    GcOperand::i64(0),
                    GcOperand::i64(0),
                    GcOperand::reference(&buckets, s),
                ),
                f,
            ),
            f,
        );
        buckets.clear(f);
        entries.clear(f);
        s.release_i32_local(zero, f);
        record
    }
    pub(super) fn emit_collection_find_set(
        &mut self,
        record: &GcLocal<SetObject>,
        key: &ValueLocals,
        index: I64Local,
        found: I32Local,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let cursor = s.reserve_i32_local(f);
        let bucket = s.reserve_i32_local(f);
        let chain = s.reserve_i64_local(f);
        let hash = s.reserve_i64_local(f);
        let retained = s.reserve_value_local(f);
        let table = s.reserve_gc_local(f).initialize(
            s.struct_type::<SetObject>()
                .field(SetObjectSchema::ENTRIES)
                .read(record, s, f)
                .reference(),
            f,
        );
        let buckets = s.reserve_gc_local(f).initialize(
            s.struct_type::<SetObject>()
                .field(SetObjectSchema::HASH_INDEX)
                .read(record, s, f)
                .reference(),
            f,
        );
        self.emit_collection_key_hash(key, hash, f);
        self.emit_collection_hash_bucket(&buckets, hash, bucket, f);
        s.array_type::<CollectionHashTable>()
            .read(&buckets, bucket, s, f)
            .store_i64(chain, f);
        f.instruction(&Instruction::I32Const(0));
        found.store(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        chain.load(f);
        f.instruction(&Instruction::I64Eqz);
        self.emit_branch_if_to_target(exit, f);
        chain.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        index.store(f);
        index.load(f);
        f.instruction(&Instruction::I32WrapI64);
        cursor.store(f);
        let entry = s.reserve_gc_local(f).initialize(
            s.array_type::<SetEntryTable>()
                .read(&table, cursor, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        let stored = s.reserve_gc_local(f).initialize(
            s.struct_type::<SetEntry>()
                .field(SetEntrySchema::VALUE)
                .read(&entry, s, f)
                .reference(),
            f,
        );
        self.emit_stored_value_to_locals(&stored, &retained, f);
        self.emit_tagged_payload_same_value_zero_i32(&retained, key, f)?;
        found.store(f);
        s.struct_type::<SetEntry>()
            .field(SetEntrySchema::HASH_NEXT)
            .read(&entry, s, f)
            .store_i64(chain, f);
        stored.clear(f);
        entry.clear(f);
        found.load(f);
        self.emit_branch_if_to_target(exit, f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        found.load(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        s.struct_type::<SetObject>()
            .field(SetObjectSchema::HISTORY_LENGTH)
            .read(record, s, f)
            .store_i64(index, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        retained.clear(f);
        buckets.clear(f);
        table.clear(f);
        s.release_i64_local(hash, f);
        s.release_i64_local(chain, f);
        s.release_i32_local(bucket, f);
        s.release_i32_local(cursor, f);
        Ok(())
    }

    fn emit_collection_grow_set(
        &mut self,
        record: &GcLocal<SetObject>,
        count: I64Local,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        let capacity = s.reserve_i32_local(f);
        let cursor = s.reserve_i32_local(f);
        let next_capacity = s.reserve_i64_local(f);
        let table = s.reserve_gc_local(f).initialize(
            s.struct_type::<SetObject>()
                .field(SetObjectSchema::ENTRIES)
                .read(record, s, f)
                .reference(),
            f,
        );
        s.array_type::<SetEntryTable>().length(&table, s, f);
        capacity.store(f);
        count.load(f);
        f.instruction(&Instruction::I64Const(u32::MAX as i64));
        f.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        count.load(f);
        capacity.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        f.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, f);
        capacity.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        f.instruction(&Instruction::I64Const(2));
        f.instruction(&Instruction::I64Mul);
        next_capacity.store(f);
        next_capacity.load(f);
        f.instruction(&Instruction::I64Const(4));
        f.instruction(&Instruction::I64LtU);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::I64Const(4));
        next_capacity.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        next_capacity.load(f);
        f.instruction(&Instruction::I64Const(u32::MAX as i64));
        f.instruction(&Instruction::I64GtU);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::I64Const(u32::MAX as i64));
        next_capacity.store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        next_capacity.load(f);
        f.instruction(&Instruction::I32WrapI64);
        capacity.store(f);
        let grown = s.reserve_gc_local(f).initialize(
            s.array_type::<SetEntryTable>()
                .filled(GcOperand::null(s), capacity, f),
            f,
        );
        f.instruction(&Instruction::I32Const(0));
        cursor.store(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        cursor.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        count.load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(exit, f);
        let entry = s.reserve_gc_local(f).initialize(
            s.array_type::<SetEntryTable>()
                .read(&table, cursor, s, f)
                .reference(),
            f,
        );
        s.array_type::<SetEntryTable>().write(
            &grown,
            cursor,
            GcOperand::reference(&entry, s),
            s,
            f,
        );
        entry.clear(f);
        cursor.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        cursor.store(f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        s.struct_type::<SetObject>()
            .field(SetObjectSchema::ENTRIES)
            .write(record, GcOperand::reference(&grown, s), s, f);
        grown.clear(f);
        self.emit_collection_rehash_set(record, capacity, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        table.clear(f);
        s.release_i64_local(next_capacity, f);
        s.release_i32_local(cursor, f);
        s.release_i32_local(capacity, f);
    }
    pub(super) fn emit_collection_clear_set(
        &mut self,
        record: &GcLocal<SetObject>,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        let cursor = s.reserve_i32_local(f);
        let count = s.reserve_i64_local(f);
        let table = s.reserve_gc_local(f).initialize(
            s.struct_type::<SetObject>()
                .field(SetObjectSchema::ENTRIES)
                .read(record, s, f)
                .reference(),
            f,
        );
        s.struct_type::<SetObject>()
            .field(SetObjectSchema::HISTORY_LENGTH)
            .read(record, s, f)
            .store_i64(count, f);
        f.instruction(&Instruction::I32Const(0));
        cursor.store(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        cursor.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        count.load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(exit, f);
        s.array_type::<SetEntryTable>()
            .write(&table, cursor, GcOperand::null(s), s, f);
        cursor.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        cursor.store(f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        s.struct_type::<SetObject>()
            .field(SetObjectSchema::LIVE_COUNT)
            .write(record, GcOperand::i64(0), s, f);
        let buckets = s.reserve_gc_local(f).initialize(
            s.struct_type::<SetObject>()
                .field(SetObjectSchema::HASH_INDEX)
                .read(record, s, f)
                .reference(),
            f,
        );
        let capacity = s.reserve_i32_local(f);
        s.array_type::<CollectionHashTable>().length(&buckets, s, f);
        capacity.store(f);
        f.instruction(&Instruction::I32Const(0));
        cursor.store(f);
        let cleared = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        cursor.load(f);
        capacity.load(f);
        f.instruction(&Instruction::I32GeU);
        self.emit_branch_if_to_target(cleared, f);
        s.array_type::<CollectionHashTable>()
            .write(&buckets, cursor, GcOperand::i64(0), s, f);
        cursor.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        cursor.store(f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        s.release_i32_local(capacity, f);
        buckets.clear(f);
        // HISTORY_LENGTH intentionally survives Clear; active cursors are live.
        table.clear(f);
        s.release_i64_local(count, f);
        s.release_i32_local(cursor, f);
    }
    pub(super) fn emit_collection_delete_set(
        &mut self,
        record: &GcLocal<SetObject>,
        key: &ValueLocals,
        removed: I32Local,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let index = s.reserve_i64_local(f);
        let cursor = s.reserve_i32_local(f);
        let live = s.reserve_i64_local(f);
        self.emit_collection_find_set(record, key, index, removed, f)?;
        removed.load(f);
        self.open_frame(ControlFrameKind::If, f);
        let table = s.reserve_gc_local(f).initialize(
            s.struct_type::<SetObject>()
                .field(SetObjectSchema::ENTRIES)
                .read(record, s, f)
                .reference(),
            f,
        );
        self.emit_collection_unlink_set(record, key, index, f);
        index.load(f);
        f.instruction(&Instruction::I32WrapI64);
        cursor.store(f);
        s.array_type::<SetEntryTable>()
            .write(&table, cursor, GcOperand::null(s), s, f);
        s.struct_type::<SetObject>()
            .field(SetObjectSchema::LIVE_COUNT)
            .read(record, s, f)
            .store_i64(live, f);
        self.emit_increment_local(live, -1, f);
        s.struct_type::<SetObject>()
            .field(SetObjectSchema::LIVE_COUNT)
            .write(record, GcOperand::i64_local(live), s, f);
        table.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        s.release_i64_local(live, f);
        s.release_i32_local(cursor, f);
        s.release_i64_local(index, f);
        Ok(())
    }
    pub(super) fn emit_collection_put_set(
        &mut self,
        record: &GcLocal<SetObject>,
        value: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let index = s.reserve_i64_local(f);
        let found = s.reserve_i32_local(f);
        let cursor = s.reserve_i32_local(f);
        let live = s.reserve_i64_local(f);
        self.emit_collection_find_set(record, value, index, found, f)?;
        found.load(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        // Append never reuses a tombstone: deletion/reinsertion moves to the end.
        self.emit_collection_grow_set(record, index, f);
        let table = s.reserve_gc_local(f).initialize(
            s.struct_type::<SetObject>()
                .field(SetObjectSchema::ENTRIES)
                .read(record, s, f)
                .reference(),
            f,
        );
        let hash = s.reserve_i64_local(f);
        let head = s.reserve_i64_local(f);
        let bucket = s.reserve_i32_local(f);
        let buckets = s.reserve_gc_local(f).initialize(
            s.struct_type::<SetObject>()
                .field(SetObjectSchema::HASH_INDEX)
                .read(record, s, f)
                .reference(),
            f,
        );
        self.emit_collection_key_hash(value, hash, f);
        self.emit_collection_hash_bucket(&buckets, hash, bucket, f);
        s.array_type::<CollectionHashTable>()
            .read(&buckets, bucket, s, f)
            .store_i64(head, f);
        let stored_value = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(value, f), f);
        let entry = s.reserve_gc_local(f).initialize(
            s.struct_type::<SetEntry>().construct(
                (
                    GcOperand::reference(&stored_value, s),
                    GcOperand::i64_local(head),
                ),
                f,
            ),
            f,
        );
        index.load(f);
        f.instruction(&Instruction::I32WrapI64);
        cursor.store(f);
        s.array_type::<SetEntryTable>().write(
            &table,
            cursor,
            GcOperand::nullable_reference(&entry, s),
            s,
            f,
        );
        self.emit_increment_local(index, 1, f);
        s.array_type::<CollectionHashTable>().write(
            &buckets,
            bucket,
            GcOperand::i64_local(index),
            s,
            f,
        );
        s.struct_type::<SetObject>()
            .field(SetObjectSchema::HISTORY_LENGTH)
            .write(record, GcOperand::i64_local(index), s, f);
        s.struct_type::<SetObject>()
            .field(SetObjectSchema::LIVE_COUNT)
            .read(record, s, f)
            .store_i64(live, f);
        self.emit_increment_local(live, 1, f);
        s.struct_type::<SetObject>()
            .field(SetObjectSchema::LIVE_COUNT)
            .write(record, GcOperand::i64_local(live), s, f);
        buckets.clear(f);
        s.release_i32_local(bucket, f);
        s.release_i64_local(head, f);
        s.release_i64_local(hash, f);
        entry.clear(f);
        stored_value.clear(f);
        table.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        s.release_i64_local(live, f);
        s.release_i32_local(cursor, f);
        s.release_i32_local(found, f);
        s.release_i64_local(index, f);
        Ok(())
    }
    pub(super) fn emit_collection_read_set(
        &mut self,
        record: &GcLocal<SetObject>,
        index: I64Local,
        value: &ValueLocals,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        let cursor = s.reserve_i32_local(f);
        index.load(f);
        f.instruction(&Instruction::I32WrapI64);
        cursor.store(f);
        let table = s.reserve_gc_local(f).initialize(
            s.struct_type::<SetObject>()
                .field(SetObjectSchema::ENTRIES)
                .read(record, s, f)
                .reference(),
            f,
        );
        let entry = s.reserve_gc_local(f).initialize(
            s.array_type::<SetEntryTable>()
                .read(&table, cursor, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        let stored = s.reserve_gc_local(f).initialize(
            s.struct_type::<SetEntry>()
                .field(SetEntrySchema::VALUE)
                .read(&entry, s, f)
                .reference(),
            f,
        );
        self.emit_stored_value_to_locals(&stored, value, f);
        stored.clear(f);
        entry.clear(f);
        table.clear(f);
        s.release_i32_local(cursor, f);
    }
    // Advances through a fresh table and length on every step, including after callbacks.
    pub(super) fn emit_collection_next_set(
        &mut self,
        record: &GcLocal<SetObject>,
        index: I64Local,
        found: I32Local,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        let cursor = s.reserve_i32_local(f);
        let count = s.reserve_i64_local(f);
        let table = s.reserve_gc_local(f).initialize(
            s.struct_type::<SetObject>()
                .field(SetObjectSchema::ENTRIES)
                .read(record, s, f)
                .reference(),
            f,
        );
        s.struct_type::<SetObject>()
            .field(SetObjectSchema::HISTORY_LENGTH)
            .read(record, s, f)
            .store_i64(count, f);
        f.instruction(&Instruction::I32Const(0));
        found.store(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        index.load(f);
        count.load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(exit, f);
        index.load(f);
        f.instruction(&Instruction::I32WrapI64);
        cursor.store(f);
        s.array_type::<SetEntryTable>()
            .read(&table, cursor, s, f)
            .reference()
            .is_null(f);
        f.instruction(&Instruction::I32Eqz);
        found.store(f);
        found.load(f);
        self.emit_branch_if_to_target(exit, f);
        self.emit_increment_local(index, 1, f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        table.clear(f);
        s.release_i64_local(count, f);
        s.release_i32_local(cursor, f);
    }
    fn emit_collection_rehash_map(
        &mut self,
        record: &GcLocal<MapObject>,
        capacity: I32Local,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        let count = s.reserve_i64_local(f);
        let index = s.reserve_i64_local(f);
        let cursor = s.reserve_i32_local(f);
        let bucket = s.reserve_i32_local(f);
        let hash = s.reserve_i64_local(f);
        let head = s.reserve_i64_local(f);
        let key = s.reserve_value_local(f);
        let table = s.reserve_gc_local(f).initialize(
            s.struct_type::<MapObject>()
                .field(MapObjectSchema::ENTRIES)
                .read(record, s, f)
                .reference(),
            f,
        );
        let buckets = s.reserve_gc_local(f).initialize(
            s.array_type::<CollectionHashTable>()
                .filled(GcOperand::i64(0), capacity, f),
            f,
        );
        s.struct_type::<MapObject>()
            .field(MapObjectSchema::HISTORY_LENGTH)
            .read(record, s, f)
            .store_i64(count, f);
        f.instruction(&Instruction::I64Const(0));
        index.store(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        index.load(f);
        count.load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(exit, f);
        index.load(f);
        f.instruction(&Instruction::I32WrapI64);
        cursor.store(f);
        let maybe = s.reserve_gc_local(f).initialize(
            s.array_type::<MapEntryTable>()
                .read(&table, cursor, s, f)
                .reference(),
            f,
        );
        maybe.load(s, f).is_null(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        let entry = s
            .reserve_gc_local(f)
            .initialize(maybe.load(s, f).require_non_null(f), f);
        let stored = s.reserve_gc_local(f).initialize(
            s.struct_type::<MapEntry>()
                .field(MapEntrySchema::KEY)
                .read(&entry, s, f)
                .reference(),
            f,
        );
        self.emit_stored_value_to_locals(&stored, &key, f);
        self.emit_collection_key_hash(&key, hash, f);
        self.emit_collection_hash_bucket(&buckets, hash, bucket, f);
        s.array_type::<CollectionHashTable>()
            .read(&buckets, bucket, s, f)
            .store_i64(head, f);
        s.struct_type::<MapEntry>()
            .field(MapEntrySchema::HASH_NEXT)
            .write(&entry, GcOperand::i64_local(head), s, f);
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        head.store(f);
        s.array_type::<CollectionHashTable>().write(
            &buckets,
            bucket,
            GcOperand::i64_local(head),
            s,
            f,
        );
        stored.clear(f);
        entry.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        maybe.clear(f);
        self.emit_increment_local(index, 1, f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        s.struct_type::<MapObject>()
            .field(MapObjectSchema::HASH_INDEX)
            .write(record, GcOperand::reference(&buckets, s), s, f);
        key.clear(f);
        buckets.clear(f);
        table.clear(f);
        s.release_i64_local(head, f);
        s.release_i64_local(hash, f);
        s.release_i32_local(bucket, f);
        s.release_i32_local(cursor, f);
        s.release_i64_local(index, f);
        s.release_i64_local(count, f);
    }
    fn emit_collection_unlink_map(
        &mut self,
        record: &GcLocal<MapObject>,
        key: &ValueLocals,
        index: I64Local,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        let cursor = s.reserve_i32_local(f);
        let bucket = s.reserve_i32_local(f);
        let hash = s.reserve_i64_local(f);
        let chain = s.reserve_i64_local(f);
        let previous = s.reserve_i64_local(f);
        let next = s.reserve_i64_local(f);
        let wanted = s.reserve_i64_local(f);
        let table = s.reserve_gc_local(f).initialize(
            s.struct_type::<MapObject>()
                .field(MapObjectSchema::ENTRIES)
                .read(record, s, f)
                .reference(),
            f,
        );
        let buckets = s.reserve_gc_local(f).initialize(
            s.struct_type::<MapObject>()
                .field(MapObjectSchema::HASH_INDEX)
                .read(record, s, f)
                .reference(),
            f,
        );
        self.emit_collection_key_hash(key, hash, f);
        self.emit_collection_hash_bucket(&buckets, hash, bucket, f);
        s.array_type::<CollectionHashTable>()
            .read(&buckets, bucket, s, f)
            .store_i64(chain, f);
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        wanted.store(f);
        f.instruction(&Instruction::I64Const(0));
        previous.store(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        chain.load(f);
        f.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        chain.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I32WrapI64);
        cursor.store(f);
        let entry = s.reserve_gc_local(f).initialize(
            s.array_type::<MapEntryTable>()
                .read(&table, cursor, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        s.struct_type::<MapEntry>()
            .field(MapEntrySchema::HASH_NEXT)
            .read(&entry, s, f)
            .store_i64(next, f);
        entry.clear(f);
        chain.load(f);
        wanted.load(f);
        f.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, f);
        previous.load(f);
        f.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, f);
        s.array_type::<CollectionHashTable>().write(
            &buckets,
            bucket,
            GcOperand::i64_local(next),
            s,
            f,
        );
        f.instruction(&Instruction::Else);
        previous.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I32WrapI64);
        cursor.store(f);
        let parent = s.reserve_gc_local(f).initialize(
            s.array_type::<MapEntryTable>()
                .read(&table, cursor, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        s.struct_type::<MapEntry>()
            .field(MapEntrySchema::HASH_NEXT)
            .write(&parent, GcOperand::i64_local(next), s, f);
        parent.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        chain.load(f);
        previous.store(f);
        next.load(f);
        chain.store(f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        buckets.clear(f);
        table.clear(f);
        s.release_i64_local(wanted, f);
        s.release_i64_local(next, f);
        s.release_i64_local(previous, f);
        s.release_i64_local(chain, f);
        s.release_i64_local(hash, f);
        s.release_i32_local(bucket, f);
        s.release_i32_local(cursor, f);
    }

    fn emit_collection_rehash_set(
        &mut self,
        record: &GcLocal<SetObject>,
        capacity: I32Local,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        let count = s.reserve_i64_local(f);
        let index = s.reserve_i64_local(f);
        let cursor = s.reserve_i32_local(f);
        let bucket = s.reserve_i32_local(f);
        let hash = s.reserve_i64_local(f);
        let head = s.reserve_i64_local(f);
        let key = s.reserve_value_local(f);
        let table = s.reserve_gc_local(f).initialize(
            s.struct_type::<SetObject>()
                .field(SetObjectSchema::ENTRIES)
                .read(record, s, f)
                .reference(),
            f,
        );
        let buckets = s.reserve_gc_local(f).initialize(
            s.array_type::<CollectionHashTable>()
                .filled(GcOperand::i64(0), capacity, f),
            f,
        );
        s.struct_type::<SetObject>()
            .field(SetObjectSchema::HISTORY_LENGTH)
            .read(record, s, f)
            .store_i64(count, f);
        f.instruction(&Instruction::I64Const(0));
        index.store(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        index.load(f);
        count.load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(exit, f);
        index.load(f);
        f.instruction(&Instruction::I32WrapI64);
        cursor.store(f);
        let maybe = s.reserve_gc_local(f).initialize(
            s.array_type::<SetEntryTable>()
                .read(&table, cursor, s, f)
                .reference(),
            f,
        );
        maybe.load(s, f).is_null(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        let entry = s
            .reserve_gc_local(f)
            .initialize(maybe.load(s, f).require_non_null(f), f);
        let stored = s.reserve_gc_local(f).initialize(
            s.struct_type::<SetEntry>()
                .field(SetEntrySchema::VALUE)
                .read(&entry, s, f)
                .reference(),
            f,
        );
        self.emit_stored_value_to_locals(&stored, &key, f);
        self.emit_collection_key_hash(&key, hash, f);
        self.emit_collection_hash_bucket(&buckets, hash, bucket, f);
        s.array_type::<CollectionHashTable>()
            .read(&buckets, bucket, s, f)
            .store_i64(head, f);
        s.struct_type::<SetEntry>()
            .field(SetEntrySchema::HASH_NEXT)
            .write(&entry, GcOperand::i64_local(head), s, f);
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        head.store(f);
        s.array_type::<CollectionHashTable>().write(
            &buckets,
            bucket,
            GcOperand::i64_local(head),
            s,
            f,
        );
        stored.clear(f);
        entry.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        maybe.clear(f);
        self.emit_increment_local(index, 1, f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        s.struct_type::<SetObject>()
            .field(SetObjectSchema::HASH_INDEX)
            .write(record, GcOperand::reference(&buckets, s), s, f);
        key.clear(f);
        buckets.clear(f);
        table.clear(f);
        s.release_i64_local(head, f);
        s.release_i64_local(hash, f);
        s.release_i32_local(bucket, f);
        s.release_i32_local(cursor, f);
        s.release_i64_local(index, f);
        s.release_i64_local(count, f);
    }
    fn emit_collection_unlink_set(
        &mut self,
        record: &GcLocal<SetObject>,
        key: &ValueLocals,
        index: I64Local,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        let cursor = s.reserve_i32_local(f);
        let bucket = s.reserve_i32_local(f);
        let hash = s.reserve_i64_local(f);
        let chain = s.reserve_i64_local(f);
        let previous = s.reserve_i64_local(f);
        let next = s.reserve_i64_local(f);
        let wanted = s.reserve_i64_local(f);
        let table = s.reserve_gc_local(f).initialize(
            s.struct_type::<SetObject>()
                .field(SetObjectSchema::ENTRIES)
                .read(record, s, f)
                .reference(),
            f,
        );
        let buckets = s.reserve_gc_local(f).initialize(
            s.struct_type::<SetObject>()
                .field(SetObjectSchema::HASH_INDEX)
                .read(record, s, f)
                .reference(),
            f,
        );
        self.emit_collection_key_hash(key, hash, f);
        self.emit_collection_hash_bucket(&buckets, hash, bucket, f);
        s.array_type::<CollectionHashTable>()
            .read(&buckets, bucket, s, f)
            .store_i64(chain, f);
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        wanted.store(f);
        f.instruction(&Instruction::I64Const(0));
        previous.store(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        chain.load(f);
        f.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        chain.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I32WrapI64);
        cursor.store(f);
        let entry = s.reserve_gc_local(f).initialize(
            s.array_type::<SetEntryTable>()
                .read(&table, cursor, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        s.struct_type::<SetEntry>()
            .field(SetEntrySchema::HASH_NEXT)
            .read(&entry, s, f)
            .store_i64(next, f);
        entry.clear(f);
        chain.load(f);
        wanted.load(f);
        f.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, f);
        previous.load(f);
        f.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, f);
        s.array_type::<CollectionHashTable>().write(
            &buckets,
            bucket,
            GcOperand::i64_local(next),
            s,
            f,
        );
        f.instruction(&Instruction::Else);
        previous.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I32WrapI64);
        cursor.store(f);
        let parent = s.reserve_gc_local(f).initialize(
            s.array_type::<SetEntryTable>()
                .read(&table, cursor, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        s.struct_type::<SetEntry>()
            .field(SetEntrySchema::HASH_NEXT)
            .write(&parent, GcOperand::i64_local(next), s, f);
        parent.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        chain.load(f);
        previous.store(f);
        next.load(f);
        chain.store(f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        buckets.clear(f);
        table.clear(f);
        s.release_i64_local(wanted, f);
        s.release_i64_local(next, f);
        s.release_i64_local(previous, f);
        s.release_i64_local(chain, f);
        s.release_i64_local(hash, f);
        s.release_i32_local(bucket, f);
        s.release_i32_local(cursor, f);
    }
}
