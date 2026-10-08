//! InternalizeJSONProperty keeps live values separate from private source records.
use super::*;
impl FunctionBuilder<'_> {
    fn emit_json_reviver_frame(
        &self,
        holder: &ValueLocals,
        key: &GcLocal<StringValue>,
        record: &GcLocal<JsonParseRecord, Nullable>,
        role: JsonReviverPropertyRole,
        parent: &GcLocal<JsonReviverFrame, Nullable>,
        f: &mut Function,
    ) -> GcLocal<JsonReviverFrame> {
        let s = self.runtime_schema();
        let holder = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(holder, f), f);
        let undefined = s.reserve_value_local(f);
        undefined.set_undefined(f);
        let value = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(&undefined, f), f);
        let frame = s.reserve_gc_local(f).initialize(
            s.struct_type::<JsonReviverFrame>().construct(
                (
                    GcOperand::reference(&holder, s),
                    GcOperand::reference(key, s),
                    GcOperand::reference(record, s),
                    GcOperand::reference(&value, s),
                    GcOperand::constant(JsonReviverFrameState::Enter),
                    GcOperand::i64(0),
                    GcOperand::i64(0),
                    GcOperand::null(s),
                    GcOperand::reference(parent, s),
                    GcOperand::constant(role),
                ),
                f,
            ),
            f,
        );
        value.clear(f);
        undefined.clear(f);
        holder.clear(f);
        frame
    }
    fn emit_json_reviver_state(
        &self,
        frame: &GcLocal<JsonReviverFrame>,
        state: JsonReviverFrameState,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        s.struct_type::<JsonReviverFrame>()
            .field(JsonReviverFrameSchema::STATE)
            .write(frame, GcOperand::constant(state), s, f);
    }
    fn emit_json_metadata_child(
        &mut self,
        record: &GcLocal<JsonParseRecord, Nullable>,
        key: &GcLocal<StringValue>,
        f: &mut Function,
    ) -> GcLocal<JsonParseRecord, Nullable> {
        let s = self.runtime_schema();
        let output = s
            .reserve_gc_local::<JsonParseRecord, Nullable>(f)
            .initialize_null(s, f);
        record.load(s, f).is_null(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        let parent = s
            .reserve_gc_local(f)
            .initialize(record.load(s, f).require_non_null(f), f);
        let cursor = s.reserve_gc_local(f).initialize(
            s.struct_type::<JsonParseRecord>()
                .field(JsonParseRecordSchema::CHILDREN)
                .read(&parent, s, f)
                .reference(),
            f,
        );
        let done = self.open_frame(ControlFrameKind::Block, f);
        let next = self.open_frame(ControlFrameKind::Loop, f);
        cursor.load(s, f).is_null(f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_branch_to_target(done, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let child = s
            .reserve_gc_local(f)
            .initialize(cursor.load(s, f).require_non_null(f), f);
        let name = s.reserve_gc_local(f).initialize(
            s.struct_type::<JsonParseChild>()
                .field(JsonParseChildSchema::KEY)
                .read(&child, s, f)
                .reference(),
            f,
        );
        self.emit_string_payload_equality_i32(&name, key, f);
        self.open_frame(ControlFrameKind::If, f);
        output.replace(
            s.struct_type::<JsonParseChild>()
                .field(JsonParseChildSchema::RECORD)
                .read(&child, s, f)
                .reference()
                .nullable(),
            f,
        );
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        cursor.replace(
            s.struct_type::<JsonParseChild>()
                .field(JsonParseChildSchema::NEXT)
                .read(&child, s, f)
                .reference(),
            f,
        );
        name.clear(f);
        child.clear(f);
        output.load(s, f).is_null(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_branch_to_target(done, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        cursor.clear(f);
        parent.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        output
    }
    pub(super) fn emit_json_revive(
        &mut self,
        record: &GcLocal<JsonParseRecord>,
        reviver: &ValueLocals,
        realm: &GcLocal<RealmRecord>,
        output: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let holder = s.reserve_value_local(f);
        let value = s.reserve_value_local(f);
        let original = s.reserve_value_local(f);
        let returned = s.reserve_value_local(f);
        let key_value = s.reserve_value_local(f);
        let context_value = s.reserve_value_local(f);
        let pending = s.reserve_completion(f);
        let root = self.emit_json_plain_object(realm, f)?;
        holder.set_reference(&root, s, f);
        let empty = self.emit_json_string("", f)?;
        self.emit_json_define(&holder, &empty, output, false, f)?;
        let parent = s
            .reserve_gc_local::<JsonReviverFrame, Nullable>(f)
            .initialize_null(s, f);
        let metadata = s
            .reserve_gc_local(f)
            .initialize(record.load(s, f).nullable(), f);
        let first = self.emit_json_reviver_frame(
            &holder,
            &empty,
            &metadata,
            JsonReviverPropertyRole::Root,
            &parent,
            f,
        );
        let current = s
            .reserve_gc_local(f)
            .initialize(first.load(s, f).nullable(), f);
        first.clear(f);
        metadata.clear(f);
        parent.clear(f);
        empty.clear(f);
        root.clear(f);
        let state = s.reserve_i32_local(f);
        let role = s.reserve_i32_local(f);
        let is_array = s.reserve_i32_local(f);
        let cursor = s.reserve_i64_local(f);
        let limit = s.reserve_i64_local(f);
        let ordinal = s.reserve_i32_local(f);
        let done = self.open_frame(ControlFrameKind::Block, f);
        let next = self.open_frame(ControlFrameKind::Loop, f);
        current.load(s, f).is_null(f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_branch_to_target(done, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let frame = s
            .reserve_gc_local(f)
            .initialize(current.load(s, f).require_non_null(f), f);
        let holder_record = s.reserve_gc_local(f).initialize(
            s.struct_type::<JsonReviverFrame>()
                .field(JsonReviverFrameSchema::HOLDER)
                .read(&frame, s, f)
                .reference(),
            f,
        );
        s.struct_type::<StoredValue>()
            .read_into(&holder_record, &holder, s, f);
        let key = s.reserve_gc_local(f).initialize(
            s.struct_type::<JsonReviverFrame>()
                .field(JsonReviverFrameSchema::KEY)
                .read(&frame, s, f)
                .reference(),
            f,
        );
        let metadata = s.reserve_gc_local(f).initialize(
            s.struct_type::<JsonReviverFrame>()
                .field(JsonReviverFrameSchema::RECORD)
                .read(&frame, s, f)
                .reference(),
            f,
        );
        s.struct_type::<JsonReviverFrame>()
            .field(JsonReviverFrameSchema::STATE)
            .read(&frame, s, f)
            .store(state, f);
        for selected in JsonReviverFrameState::ALL {
            state.load(f);
            f.instruction(&Instruction::I32Const(selected.wire_code()));
            f.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, f);
            match selected {
                JsonReviverFrameState::Enter => {
                    self.emit_json_get(&holder, &key, &value, f)?;
                    let stored = s
                        .reserve_gc_local(f)
                        .initialize(s.struct_type::<StoredValue>().from_value(&value, f), f);
                    s.struct_type::<JsonReviverFrame>()
                        .field(JsonReviverFrameSchema::VALUE)
                        .write(&frame, GcOperand::reference(&stored, s), s, f);
                    stored.clear(f);
                    metadata.load(s, f).is_null(f);
                    f.instruction(&Instruction::I32Eqz);
                    self.open_frame(ControlFrameKind::If, f);
                    let node = s
                        .reserve_gc_local(f)
                        .initialize(metadata.load(s, f).require_non_null(f), f);
                    let initial = s.reserve_gc_local(f).initialize(
                        s.struct_type::<JsonParseRecord>()
                            .field(JsonParseRecordSchema::VALUE)
                            .read(&node, s, f)
                            .reference(),
                        f,
                    );
                    s.struct_type::<StoredValue>()
                        .read_into(&initial, &original, s, f);
                    self.emit_tagged_payload_same_value_i32(&value, &original, f)?;
                    f.instruction(&Instruction::I32Eqz);
                    self.open_frame(ControlFrameKind::If, f);
                    metadata.set_null(s, f);
                    s.struct_type::<JsonReviverFrame>()
                        .field(JsonReviverFrameSchema::RECORD)
                        .write(&frame, GcOperand::null(s), s, f);
                    self.pop_control(ControlFrameKind::If);
                    f.instruction(&Instruction::End);
                    initial.clear(f);
                    node.clear(f);
                    self.pop_control(ControlFrameKind::If);
                    f.instruction(&Instruction::End);
                    self.emit_is_heap_object_like_tag_i32(value.tag(), f);
                    self.open_frame(ControlFrameKind::If, f);
                    self.emit_is_array_i32(&value, is_array, &pending, f)?;
                    self.completion().copy_from(&pending, f);
                    self.emit_propagate_current_throw_if_needed(f);
                    is_array.load(f);
                    self.open_frame(ControlFrameKind::If, f);
                    let length_key = self.emit_json_string("length", f)?;
                    self.emit_json_get(&value, &length_key, &original, f)?;
                    self.emit_to_length_i64_from_value_locals(&original, limit, &pending, f)?;
                    self.completion().copy_from(&pending, f);
                    self.emit_propagate_current_throw_if_needed(f);
                    length_key.clear(f);
                    s.struct_type::<JsonReviverFrame>()
                        .field(JsonReviverFrameSchema::LIMIT)
                        .write(&frame, GcOperand::i64_local(limit), s, f);
                    self.emit_json_reviver_state(&frame, JsonReviverFrameState::ArrayChildren, f);
                    f.instruction(&Instruction::Else);
                    let keys = self.emit_json_enumerable_keys(&value, f)?;
                    s.array_type::<ValueArray>().length(&keys, s, f);
                    f.instruction(&Instruction::I64ExtendI32U);
                    limit.store(f);
                    s.struct_type::<JsonReviverFrame>()
                        .field(JsonReviverFrameSchema::KEYS)
                        .write(&frame, GcOperand::nullable_reference(&keys, s), s, f);
                    s.struct_type::<JsonReviverFrame>()
                        .field(JsonReviverFrameSchema::LIMIT)
                        .write(&frame, GcOperand::i64_local(limit), s, f);
                    keys.clear(f);
                    self.emit_json_reviver_state(&frame, JsonReviverFrameState::ObjectChildren, f);
                    self.pop_control(ControlFrameKind::If);
                    f.instruction(&Instruction::End);
                    f.instruction(&Instruction::Else);
                    self.emit_json_reviver_state(&frame, JsonReviverFrameState::Apply, f);
                    self.pop_control(ControlFrameKind::If);
                    f.instruction(&Instruction::End);
                }
                JsonReviverFrameState::ArrayChildren | JsonReviverFrameState::ObjectChildren => {
                    s.struct_type::<JsonReviverFrame>()
                        .field(JsonReviverFrameSchema::CURSOR)
                        .read(&frame, s, f)
                        .store_i64(cursor, f);
                    s.struct_type::<JsonReviverFrame>()
                        .field(JsonReviverFrameSchema::LIMIT)
                        .read(&frame, s, f)
                        .store_i64(limit, f);
                    cursor.load(f);
                    limit.load(f);
                    f.instruction(&Instruction::I64LtU);
                    self.open_frame(ControlFrameKind::If, f);
                    let child_key = s
                        .reserve_gc_local::<StringValue, Nullable>(f)
                        .initialize_null(s, f);
                    match selected {
                        JsonReviverFrameState::ArrayChildren => {
                            let name = self.emit_json_index_key(cursor, f)?;
                            child_key.replace(name.load(s, f).nullable(), f);
                            name.clear(f);
                        }
                        JsonReviverFrameState::ObjectChildren => {
                            let keys = s.reserve_gc_local(f).initialize(
                                s.struct_type::<JsonReviverFrame>()
                                    .field(JsonReviverFrameSchema::KEYS)
                                    .read(&frame, s, f)
                                    .reference()
                                    .require_non_null(f),
                                f,
                            );
                            cursor.load(f);
                            f.instruction(&Instruction::I32WrapI64);
                            ordinal.store(f);
                            self.emit_argument_vector_entry_to_value(&keys, ordinal, &key_value, f);
                            child_key.replace(
                                key_value.cast_reference::<StringValue>(s, f).nullable(),
                                f,
                            );
                            keys.clear(f);
                        }
                        JsonReviverFrameState::Enter | JsonReviverFrameState::Apply => {
                            unreachable!()
                        }
                    }
                    let name = s
                        .reserve_gc_local(f)
                        .initialize(child_key.load(s, f).require_non_null(f), f);
                    let child_metadata = self.emit_json_metadata_child(&metadata, &name, f);
                    let stored = s.reserve_gc_local(f).initialize(
                        s.struct_type::<JsonReviverFrame>()
                            .field(JsonReviverFrameSchema::VALUE)
                            .read(&frame, s, f)
                            .reference(),
                        f,
                    );
                    s.struct_type::<StoredValue>()
                        .read_into(&stored, &value, s, f);
                    json_increment(cursor, f);
                    s.struct_type::<JsonReviverFrame>()
                        .field(JsonReviverFrameSchema::CURSOR)
                        .write(&frame, GcOperand::i64_local(cursor), s, f);
                    let child = self.emit_json_reviver_frame(
                        &value,
                        &name,
                        &child_metadata,
                        JsonReviverPropertyRole::Nested,
                        &current,
                        f,
                    );
                    current.replace(child.load(s, f).nullable(), f);
                    child.clear(f);
                    stored.clear(f);
                    child_metadata.clear(f);
                    name.clear(f);
                    child_key.clear(f);
                    f.instruction(&Instruction::Else);
                    self.emit_json_reviver_state(&frame, JsonReviverFrameState::Apply, f);
                    self.pop_control(ControlFrameKind::If);
                    f.instruction(&Instruction::End);
                }
                JsonReviverFrameState::Apply => {
                    let stored = s.reserve_gc_local(f).initialize(
                        s.struct_type::<JsonReviverFrame>()
                            .field(JsonReviverFrameSchema::VALUE)
                            .read(&frame, s, f)
                            .reference(),
                        f,
                    );
                    s.struct_type::<StoredValue>()
                        .read_into(&stored, &value, s, f);
                    stored.clear(f);
                    let context = self.emit_json_plain_object(realm, f)?;
                    context_value.set_reference(&context, s, f);
                    metadata.load(s, f).is_null(f);
                    f.instruction(&Instruction::I32Eqz);
                    self.open_frame(ControlFrameKind::If, f);
                    let record = s
                        .reserve_gc_local(f)
                        .initialize(metadata.load(s, f).require_non_null(f), f);
                    let source = s.reserve_gc_local(f).initialize(
                        s.struct_type::<JsonParseRecord>()
                            .field(JsonParseRecordSchema::SOURCE)
                            .read(&record, s, f)
                            .reference(),
                        f,
                    );
                    source.load(s, f).is_null(f);
                    f.instruction(&Instruction::I32Eqz);
                    self.open_frame(ControlFrameKind::If, f);
                    let text = s
                        .reserve_gc_local(f)
                        .initialize(source.load(s, f).require_non_null(f), f);
                    original.set_reference(&text, s, f);
                    let source_key = self.emit_json_string("source", f)?;
                    self.emit_json_define(&context_value, &source_key, &original, false, f)?;
                    source_key.clear(f);
                    text.clear(f);
                    self.pop_control(ControlFrameKind::If);
                    f.instruction(&Instruction::End);
                    source.clear(f);
                    record.clear(f);
                    self.pop_control(ControlFrameKind::If);
                    f.instruction(&Instruction::End);
                    key_value.set_reference(&key, s, f);
                    self.emit_json_call(
                        reviver,
                        &holder,
                        &[&key_value, &value, &context_value],
                        &returned,
                        f,
                    )?;
                    context.clear(f);
                    s.struct_type::<JsonReviverFrame>()
                        .field(JsonReviverFrameSchema::ROLE)
                        .read(&frame, s, f)
                        .store(role, f);
                    role.load(f);
                    f.instruction(&Instruction::I32Const(
                        JsonReviverPropertyRole::Nested.wire_code(),
                    ));
                    f.instruction(&Instruction::I32Eq);
                    self.open_frame(ControlFrameKind::If, f);
                    self.emit_json_tag_test(&returned, WasmRuntimeValueTag::Undefined, f);
                    self.open_frame(ControlFrameKind::If, f);
                    let property = self.emit_value_to_property_key_locals(&key_value, f)?;
                    self.emit_object_delete(&holder, &property, &pending, f)?;
                    property.clear(f);
                    self.completion().copy_from(&pending, f);
                    self.emit_propagate_current_throw_if_needed(f);
                    f.instruction(&Instruction::Else);
                    self.emit_json_define(&holder, &key, &returned, true, f)?;
                    self.pop_control(ControlFrameKind::If);
                    f.instruction(&Instruction::End);
                    f.instruction(&Instruction::Else);
                    output.copy_from(&returned, f);
                    self.pop_control(ControlFrameKind::If);
                    f.instruction(&Instruction::End);
                    current.replace(
                        s.struct_type::<JsonReviverFrame>()
                            .field(JsonReviverFrameSchema::PARENT)
                            .read(&frame, s, f)
                            .reference(),
                        f,
                    );
                }
            }
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        metadata.clear(f);
        key.clear(f);
        holder_record.clear(f);
        frame.clear(f);
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        s.release_i32_local(ordinal, f);
        s.release_i64_local(limit, f);
        s.release_i64_local(cursor, f);
        s.release_i32_local(is_array, f);
        s.release_i32_local(role, f);
        s.release_i32_local(state, f);
        current.clear(f);
        pending.clear(f);
        context_value.clear(f);
        key_value.clear(f);
        returned.clear(f);
        original.clear(f);
        value.clear(f);
        holder.clear(f);
        Ok(())
    }
}
