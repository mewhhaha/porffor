//! Closed frame fields and consuming container completion replace linear stacks.
use super::*;
impl FunctionBuilder<'_> {
    pub(super) fn emit_json_parse_frame(
        &mut self,
        value: &ValueLocals,
        state: JsonParseFrameState,
        parent: &GcLocal<JsonParseFrame, Nullable>,
        f: &mut Function,
    ) -> GcLocal<JsonParseFrame> {
        let s = self.runtime_schema();
        let stored = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(value, f), f);
        let frame = s.reserve_gc_local(f).initialize(
            s.struct_type::<JsonParseFrame>().construct(
                (
                    GcOperand::reference(&stored, s),
                    GcOperand::constant(state),
                    GcOperand::null(s),
                    GcOperand::i64(0),
                    GcOperand::null(s),
                    GcOperand::reference(parent, s),
                ),
                f,
            ),
            f,
        );
        stored.clear(f);
        frame
    }
    pub(super) fn emit_json_parse_state(
        &self,
        frame: &GcLocal<JsonParseFrame>,
        state: JsonParseFrameState,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        s.struct_type::<JsonParseFrame>()
            .field(JsonParseFrameSchema::STATE)
            .write(frame, GcOperand::constant(state), s, f);
    }
    pub(super) fn emit_json_parse_record(
        &self,
        value: &ValueLocals,
        source: &GcLocal<StringValue, Nullable>,
        children: &GcLocal<JsonParseChild, Nullable>,
        f: &mut Function,
    ) -> GcLocal<JsonParseRecord> {
        let s = self.runtime_schema();
        let stored = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(value, f), f);
        let record = s.reserve_gc_local(f).initialize(
            s.struct_type::<JsonParseRecord>().construct(
                (
                    GcOperand::reference(&stored, s),
                    GcOperand::reference(source, s),
                    GcOperand::reference(children, s),
                ),
                f,
            ),
            f,
        );
        stored.clear(f);
        record
    }
    /// Updating the original key's record makes duplicate-key source metadata
    /// agree with CreateDataProperty's last value without exposing metadata to JS.
    pub(super) fn emit_json_parse_child(
        &mut self,
        frame: &GcLocal<JsonParseFrame>,
        key: &GcLocal<StringValue>,
        record: &GcLocal<JsonParseRecord>,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let found = s.reserve_i32_local(f);
        json_i32(found, 0, f);
        let head = s.reserve_gc_local(f).initialize(
            s.struct_type::<JsonParseFrame>()
                .field(JsonParseFrameSchema::CHILDREN)
                .read(frame, s, f)
                .reference(),
            f,
        );
        let cursor = s.reserve_gc_local(f).initialize(head.load(s, f), f);
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
        let child_key = s.reserve_gc_local(f).initialize(
            s.struct_type::<JsonParseChild>()
                .field(JsonParseChildSchema::KEY)
                .read(&child, s, f)
                .reference(),
            f,
        );
        self.emit_string_payload_equality_i32(key, &child_key, f);
        self.open_frame(ControlFrameKind::If, f);
        s.struct_type::<JsonParseChild>()
            .field(JsonParseChildSchema::RECORD)
            .write(&child, GcOperand::reference(record, s), s, f);
        json_i32(found, 1, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        cursor.replace(
            s.struct_type::<JsonParseChild>()
                .field(JsonParseChildSchema::NEXT)
                .read(&child, s, f)
                .reference(),
            f,
        );
        child_key.clear(f);
        child.clear(f);
        found.load(f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_branch_to_target(done, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_branch_to_target(next, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        found.load(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        let child = s.reserve_gc_local(f).initialize(
            s.struct_type::<JsonParseChild>().construct(
                (
                    GcOperand::reference(key, s),
                    GcOperand::reference(record, s),
                    GcOperand::reference(&head, s),
                ),
                f,
            ),
            f,
        );
        s.struct_type::<JsonParseFrame>()
            .field(JsonParseFrameSchema::CHILDREN)
            .write(frame, GcOperand::nullable_reference(&child, s), s, f);
        child.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        cursor.clear(f);
        head.clear(f);
        s.release_i32_local(found, f);
        Ok(())
    }
}
