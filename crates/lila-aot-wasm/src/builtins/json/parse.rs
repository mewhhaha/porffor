//! An iterative UTF-16 parser publishes only completed private records.
use super::*;
impl FunctionBuilder<'_> {
    fn emit_json_parse_text(
        &mut self,
        text: &GcLocal<StringValue>,
        realm: &GcLocal<RealmRecord>,
        f: &mut Function,
    ) -> Result<GcLocal<JsonParseRecord>, EmitError> {
        let s = self.runtime_schema();
        let length = s.reserve_i64_local(f);
        let index = s.reserve_i64_local(f);
        let start = s.reserve_i64_local(f);
        let ordinal = s.reserve_i64_local(f);
        let state = s.reserve_i32_local(f);
        let unit = s.reserve_i32_local(f);
        let expect_value = s.reserve_i32_local(f);
        self.emit_native_gc_string_length(text, length, f);
        json_i64(index, 0, f);
        json_i32(expect_value, 1, f);
        let current = s
            .reserve_gc_local::<JsonParseFrame, Nullable>(f)
            .initialize_null(s, f);
        let delivered = s
            .reserve_gc_local::<JsonParseRecord, Nullable>(f)
            .initialize_null(s, f);
        let source = s
            .reserve_gc_local::<StringValue, Nullable>(f)
            .initialize_null(s, f);
        let no_children = s
            .reserve_gc_local::<JsonParseChild, Nullable>(f)
            .initialize_null(s, f);
        let value = s.reserve_value_local(f);
        let container = s.reserve_value_local(f);
        let done = self.open_frame(ControlFrameKind::Block, f);
        let next = self.open_frame(ControlFrameKind::Loop, f);
        self.emit_json_skip_whitespace(text, index, length, f);
        expect_value.load(f);
        self.open_frame(ControlFrameKind::If, f);
        index.load(f);
        start.store(f);
        self.emit_json_peek(text, index, length, unit, f);
        for (code, array) in [(91, true), (123, false)] {
            unit.load(f);
            f.instruction(&Instruction::I32Const(code));
            f.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, f);
            json_increment(index, f);
            if array {
                let prototype = s.reserve_value_local(f);
                let array_proto = s
                    .reserve_gc_local(f)
                    .initialize(self.emit_load_realm_array_prototype(realm, f), f);
                prototype.set_reference(&array_proto, s, f);
                json_i64(ordinal, 0, f);
                let object = s.reserve_gc_local(f).initialize(
                    self.emit_alloc_array_payload_with_length_and_prototype(
                        ordinal, &prototype, f,
                    )?,
                    f,
                );
                value.set_reference(&object, s, f);
                object.clear(f);
                array_proto.clear(f);
                prototype.clear(f);
            } else {
                let object = self.emit_json_plain_object(realm, f)?;
                value.set_reference(&object, s, f);
                object.clear(f);
            }
            let frame = self.emit_json_parse_frame(
                &value,
                if array {
                    JsonParseFrameState::ArrayFirstOrEnd
                } else {
                    JsonParseFrameState::ObjectFirstKeyOrEnd
                },
                &current,
                f,
            );
            current.replace(frame.load(s, f).nullable(), f);
            frame.clear(f);
            json_i32(expect_value, 0, f);
            self.emit_branch_to_target(next, f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        self.emit_json_parse_primitive(
            text,
            index,
            length,
            realm,
            RuntimeErrorMessage::INVALID_JSON_PARSE_TEXT,
            &value,
            f,
        )?;
        let primitive_source = s
            .reserve_gc_local(f)
            .initialize(self.emit_gc_string_slice(text, start, index, f), f);
        source.replace(primitive_source.load(s, f).nullable(), f);
        primitive_source.clear(f);
        let record = self.emit_json_parse_record(&value, &source, &no_children, f);
        delivered.replace(record.load(s, f).nullable(), f);
        record.clear(f);
        source.set_null(s, f);
        json_i32(expect_value, 0, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        delivered.load(s, f).is_null(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        current.load(s, f).is_null(f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_branch_to_target(done, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let frame = s
            .reserve_gc_local(f)
            .initialize(current.load(s, f).require_non_null(f), f);
        let record = s
            .reserve_gc_local(f)
            .initialize(delivered.load(s, f).require_non_null(f), f);
        let stored = s.reserve_gc_local(f).initialize(
            s.struct_type::<JsonParseRecord>()
                .field(JsonParseRecordSchema::VALUE)
                .read(&record, s, f)
                .reference(),
            f,
        );
        s.struct_type::<StoredValue>()
            .read_into(&stored, &value, s, f);
        let holder = s.reserve_gc_local(f).initialize(
            s.struct_type::<JsonParseFrame>()
                .field(JsonParseFrameSchema::CONTAINER)
                .read(&frame, s, f)
                .reference(),
            f,
        );
        s.struct_type::<StoredValue>()
            .read_into(&holder, &container, s, f);
        s.struct_type::<JsonParseFrame>()
            .field(JsonParseFrameSchema::STATE)
            .read(&frame, s, f)
            .store(state, f);
        state.load(f);
        f.instruction(&Instruction::I32Const(
            JsonParseFrameState::ObjectValue.wire_code(),
        ));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        let key = s.reserve_gc_local(f).initialize(
            s.struct_type::<JsonParseFrame>()
                .field(JsonParseFrameSchema::KEY)
                .read(&frame, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        self.emit_json_define(&container, &key, &value, false, f)?;
        self.emit_json_parse_child(&frame, &key, &record, f)?;
        key.clear(f);
        self.emit_json_parse_state(&frame, JsonParseFrameState::ObjectCommaOrEnd, f);
        f.instruction(&Instruction::Else);
        s.struct_type::<JsonParseFrame>()
            .field(JsonParseFrameSchema::INDEX)
            .read(&frame, s, f)
            .store_i64(ordinal, f);
        let key = self.emit_json_index_key(ordinal, f)?;
        self.emit_json_define(&container, &key, &value, false, f)?;
        self.emit_json_parse_child(&frame, &key, &record, f)?;
        key.clear(f);
        json_increment(ordinal, f);
        s.struct_type::<JsonParseFrame>()
            .field(JsonParseFrameSchema::INDEX)
            .write(&frame, GcOperand::i64_local(ordinal), s, f);
        self.emit_json_parse_state(&frame, JsonParseFrameState::ArrayCommaOrEnd, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        holder.clear(f);
        stored.clear(f);
        record.clear(f);
        frame.clear(f);
        delivered.set_null(s, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let frame = s
            .reserve_gc_local(f)
            .initialize(current.load(s, f).require_non_null(f), f);
        s.struct_type::<JsonParseFrame>()
            .field(JsonParseFrameSchema::STATE)
            .read(&frame, s, f)
            .store(state, f);
        self.emit_json_skip_whitespace(text, index, length, f);
        self.emit_json_peek(text, index, length, unit, f);
        for selected in JsonParseFrameState::ALL {
            state.load(f);
            f.instruction(&Instruction::I32Const(selected.wire_code()));
            f.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, f);
            match selected {
                JsonParseFrameState::ArrayFirstOrEnd | JsonParseFrameState::ObjectFirstKeyOrEnd => {
                    let closing = if matches!(selected, JsonParseFrameState::ArrayFirstOrEnd) {
                        93
                    } else {
                        125
                    };
                    unit.load(f);
                    f.instruction(&Instruction::I32Const(closing));
                    f.instruction(&Instruction::I32Eq);
                    self.open_frame(ControlFrameKind::If, f);
                    self.emit_json_complete_container(&frame, &current, &delivered, index, f)?;
                    f.instruction(&Instruction::Else);
                    self.emit_json_parse_state(
                        &frame,
                        if closing == 93 {
                            JsonParseFrameState::ArrayValue
                        } else {
                            JsonParseFrameState::ObjectKey
                        },
                        f,
                    );
                    self.pop_control(ControlFrameKind::If);
                    f.instruction(&Instruction::End);
                }
                JsonParseFrameState::ArrayValue | JsonParseFrameState::ObjectValue => {
                    json_i32(expect_value, 1, f)
                }
                JsonParseFrameState::ObjectKey => {
                    let key = self.emit_json_parse_string(
                        text,
                        index,
                        length,
                        realm,
                        RuntimeErrorMessage::INVALID_JSON_PARSE_TEXT,
                        f,
                    )?;
                    s.struct_type::<JsonParseFrame>()
                        .field(JsonParseFrameSchema::KEY)
                        .write(&frame, GcOperand::nullable_reference(&key, s), s, f);
                    key.clear(f);
                    self.emit_json_parse_state(&frame, JsonParseFrameState::ObjectColon, f);
                }
                JsonParseFrameState::ObjectColon => {
                    self.emit_json_expect_unit(
                        text,
                        index,
                        length,
                        58,
                        realm,
                        RuntimeErrorMessage::INVALID_JSON_PARSE_TEXT,
                        f,
                    )?;
                    self.emit_json_parse_state(&frame, JsonParseFrameState::ObjectValue, f);
                    json_i32(expect_value, 1, f);
                }
                JsonParseFrameState::ArrayCommaOrEnd | JsonParseFrameState::ObjectCommaOrEnd => {
                    let array = matches!(selected, JsonParseFrameState::ArrayCommaOrEnd);
                    unit.load(f);
                    f.instruction(&Instruction::I32Const(44));
                    f.instruction(&Instruction::I32Eq);
                    self.open_frame(ControlFrameKind::If, f);
                    json_increment(index, f);
                    self.emit_json_parse_state(
                        &frame,
                        if array {
                            JsonParseFrameState::ArrayValue
                        } else {
                            JsonParseFrameState::ObjectKey
                        },
                        f,
                    );
                    f.instruction(&Instruction::Else);
                    unit.load(f);
                    f.instruction(&Instruction::I32Const(if array { 93 } else { 125 }));
                    f.instruction(&Instruction::I32Ne);
                    self.open_frame(ControlFrameKind::If, f);
                    self.emit_json_bad_text(
                        realm,
                        RuntimeErrorMessage::INVALID_JSON_PARSE_TEXT,
                        f,
                    )?;
                    self.pop_control(ControlFrameKind::If);
                    f.instruction(&Instruction::End);
                    self.emit_json_complete_container(&frame, &current, &delivered, index, f)?;
                    self.pop_control(ControlFrameKind::If);
                    f.instruction(&Instruction::End);
                }
            }
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        frame.clear(f);
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.emit_json_skip_whitespace(text, index, length, f);
        index.load(f);
        length.load(f);
        f.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_json_bad_text(realm, RuntimeErrorMessage::INVALID_JSON_PARSE_TEXT, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let result = s
            .reserve_gc_local(f)
            .initialize(delivered.load(s, f).require_non_null(f), f);
        container.clear(f);
        value.clear(f);
        no_children.clear(f);
        source.clear(f);
        delivered.clear(f);
        current.clear(f);
        s.release_i32_local(expect_value, f);
        s.release_i32_local(unit, f);
        s.release_i32_local(state, f);
        s.release_i64_local(ordinal, f);
        s.release_i64_local(start, f);
        s.release_i64_local(index, f);
        s.release_i64_local(length, f);
        Ok(result)
    }
    fn emit_json_complete_container(
        &mut self,
        frame: &GcLocal<JsonParseFrame>,
        current: &GcLocal<JsonParseFrame, Nullable>,
        delivered: &GcLocal<JsonParseRecord, Nullable>,
        index: I64Local,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        json_increment(index, f);
        let stored = s.reserve_gc_local(f).initialize(
            s.struct_type::<JsonParseFrame>()
                .field(JsonParseFrameSchema::CONTAINER)
                .read(frame, s, f)
                .reference(),
            f,
        );
        let value = s.reserve_value_local(f);
        s.struct_type::<StoredValue>()
            .read_into(&stored, &value, s, f);
        let source = s
            .reserve_gc_local::<StringValue, Nullable>(f)
            .initialize_null(s, f);
        let children = s.reserve_gc_local(f).initialize(
            s.struct_type::<JsonParseFrame>()
                .field(JsonParseFrameSchema::CHILDREN)
                .read(frame, s, f)
                .reference(),
            f,
        );
        let record = self.emit_json_parse_record(&value, &source, &children, f);
        delivered.replace(record.load(s, f).nullable(), f);
        current.replace(
            s.struct_type::<JsonParseFrame>()
                .field(JsonParseFrameSchema::PARENT)
                .read(frame, s, f)
                .reference(),
            f,
        );
        record.clear(f);
        children.clear(f);
        source.clear(f);
        value.clear(f);
        stored.clear(f);
        Ok(())
    }
    pub(super) fn emit_json_parse_entry(&mut self, f: &mut Function) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let text = s.reserve_value_local(f);
        let reviver = s.reserve_value_local(f);
        let pending = s.reserve_completion(f);
        let output = s.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &text, f);
        self.emit_builtin_arg_to_value(1, &reviver, f);
        self.emit_value_to_string_payload(&text, &pending, f)?;
        self.completion().copy_from(&pending, f);
        self.emit_propagate_current_throw_if_needed(f);
        let string = s
            .reserve_gc_local(f)
            .initialize(pending.value().cast_reference::<StringValue>(s, f), f);
        let realm = self.emit_execution_realm(f);
        let record = self.emit_json_parse_text(&string, &realm, f)?;
        let stored = s.reserve_gc_local(f).initialize(
            s.struct_type::<JsonParseRecord>()
                .field(JsonParseRecordSchema::VALUE)
                .read(&record, s, f)
                .reference(),
            f,
        );
        s.struct_type::<StoredValue>()
            .read_into(&stored, &output, s, f);
        self.emit_is_callable_i32(&reviver, f)?;
        self.open_frame(ControlFrameKind::If, f);
        self.emit_json_revive(&record, &reviver, &realm, &output, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.completion().set_normal(&output, f);
        stored.clear(f);
        record.clear(f);
        realm.clear(f);
        string.clear(f);
        output.clear(f);
        pending.clear(f);
        reviver.clear(f);
        text.clear(f);
        Ok(())
    }
    pub(super) fn emit_json_raw_entry(&mut self, f: &mut Function) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let input = s.reserve_value_local(f);
        let pending = s.reserve_completion(f);
        let value = s.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &input, f);
        self.emit_value_to_string_payload(&input, &pending, f)?;
        self.completion().copy_from(&pending, f);
        self.emit_propagate_current_throw_if_needed(f);
        let text = s
            .reserve_gc_local(f)
            .initialize(pending.value().cast_reference::<StringValue>(s, f), f);
        let realm = self.emit_execution_realm(f);
        let length = s.reserve_i64_local(f);
        let index = s.reserve_i64_local(f);
        self.emit_native_gc_string_length(&text, length, f);
        json_i64(index, 0, f);
        // No whitespace/container is accepted: the primitive grammar must consume all units.
        self.emit_json_parse_primitive(
            &text,
            index,
            length,
            &realm,
            RuntimeErrorMessage::INVALID_JSON_RAWJSON_TEXT,
            &value,
            f,
        )?;
        index.load(f);
        length.load(f);
        f.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_json_error(
            &realm,
            NativeErrorKind::SyntaxError,
            RuntimeErrorMessage::INVALID_JSON_RAWJSON_TEXT,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let prototype = s.reserve_value_local(f);
        prototype.set_scalar(ScalarValue::Null, f);
        let header = s.reserve_gc_local(f).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(&prototype), f)?,
            f,
        );
        let key_string = self.emit_json_string("rawJSON", f)?;
        let key_value = s.reserve_value_local(f);
        key_value.set_reference(&key_string, s, f);
        let key = self.emit_value_to_property_key_locals(&key_value, f)?;
        value.set_reference(&text, s, f);
        self.emit_object_append_data_property_with_flags(
            &header, &key, &value, false, true, false, f,
        )?;
        s.struct_type::<OrdinaryObject>()
            .field(OrdinaryObjectSchema::EXTENSIBLE)
            .write(&header, GcOperand::boolean(false), s, f);
        let raw = s.reserve_gc_local(f).initialize(
            s.struct_type::<RawJsonObject>().construct(
                (
                    GcOperand::reference(&header, s),
                    GcOperand::reference(&text, s),
                ),
                f,
            ),
            f,
        );
        value.set_reference(&raw, s, f);
        self.completion().set_normal(&value, f);
        raw.clear(f);
        key.clear(f);
        key_value.clear(f);
        key_string.clear(f);
        header.clear(f);
        prototype.clear(f);
        s.release_i64_local(index, f);
        s.release_i64_local(length, f);
        realm.clear(f);
        text.clear(f);
        value.clear(f);
        pending.clear(f);
        input.clear(f);
        Ok(())
    }
}
