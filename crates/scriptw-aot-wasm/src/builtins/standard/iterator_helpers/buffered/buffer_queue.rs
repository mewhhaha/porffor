//! Internal GC List: storage grows only when IteratorStepValue produces a value.
use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn emit_buffer_clear(&self, buffer: &GcLocal<IteratorBuffer>, f: &mut Function) {
        let s = self.runtime_schema();
        s.field(IteratorBufferSchema::HEAD)
            .write(buffer, GcOperand::null(s), s, f);
        s.field(IteratorBufferSchema::TAIL)
            .write(buffer, GcOperand::null(s), s, f);
        s.field(IteratorBufferSchema::COUNT)
            .write(buffer, GcOperand::i64(0), s, f);
    }

    pub(super) fn emit_buffer_append(
        &self,
        buffer: &GcLocal<IteratorBuffer>,
        value: &ValueLocals,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        let stored = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(value, f), f);
        let node = s.reserve_gc_local(f).initialize(
            s.struct_type::<ArgumentListNode>()
                .construct((GcOperand::reference(&stored, s), GcOperand::null(s)), f),
            f,
        );
        let tail = s.reserve_gc_local(f).initialize(
            s.field(IteratorBufferSchema::TAIL)
                .read(buffer, s, f)
                .reference(),
            f,
        );
        tail.load(s, f);
        f.instruction(&Instruction::RefIsNull);
        f.instruction(&Instruction::If(BlockType::Empty));
        s.field(IteratorBufferSchema::HEAD).write(
            buffer,
            GcOperand::nullable_reference(&node, s),
            s,
            f,
        );
        f.instruction(&Instruction::Else);
        let last = s
            .reserve_gc_local(f)
            .initialize(tail.load(s, f).require_non_null(f), f);
        s.field(ArgumentListNodeSchema::NEXT).write(
            &last,
            GcOperand::nullable_reference(&node, s),
            s,
            f,
        );
        last.clear(f);
        f.instruction(&Instruction::End);
        s.field(IteratorBufferSchema::TAIL).write(
            buffer,
            GcOperand::nullable_reference(&node, s),
            s,
            f,
        );
        let count = s.reserve_i64_local(f);
        s.field(IteratorBufferSchema::COUNT)
            .read(buffer, s, f)
            .store_i64(count, f);
        count.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        count.store(f);
        s.field(IteratorBufferSchema::COUNT)
            .write(buffer, GcOperand::i64_local(count), s, f);
        s.release_i64_local(count, f);
        tail.clear(f);
        node.clear(f);
        stored.clear(f);
    }

    pub(super) fn emit_buffer_drop_first(
        &self,
        buffer: &GcLocal<IteratorBuffer>,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        let head = s.reserve_gc_local(f).initialize(
            s.field(IteratorBufferSchema::HEAD)
                .read(buffer, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        let next = s.reserve_gc_local(f).initialize(
            s.field(ArgumentListNodeSchema::NEXT)
                .read(&head, s, f)
                .reference(),
            f,
        );
        s.field(IteratorBufferSchema::HEAD)
            .write(buffer, GcOperand::reference(&next, s), s, f);
        next.load(s, f);
        f.instruction(&Instruction::RefIsNull);
        f.instruction(&Instruction::If(BlockType::Empty));
        s.field(IteratorBufferSchema::TAIL)
            .write(buffer, GcOperand::null(s), s, f);
        f.instruction(&Instruction::End);
        let count = s.reserve_i64_local(f);
        s.field(IteratorBufferSchema::COUNT)
            .read(buffer, s, f)
            .store_i64(count, f);
        count.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        count.store(f);
        s.field(IteratorBufferSchema::COUNT)
            .write(buffer, GcOperand::i64_local(count), s, f);
        s.release_i64_local(count, f);
        next.clear(f);
        head.clear(f);
    }

    pub(super) fn emit_buffer_array(
        &mut self,
        buffer: &GcLocal<IteratorBuffer>,
        f: &mut Function,
    ) -> Result<GcLocal<ArrayObject>, EmitError> {
        let s = self.runtime_schema();
        let length = s.reserve_i32_local(f);
        let count = s.reserve_i64_local(f);
        s.field(IteratorBufferSchema::COUNT)
            .read(buffer, s, f)
            .store_i64(count, f);
        count.load(f);
        f.instruction(&Instruction::I32WrapI64);
        length.store(f);
        let initial = s.reserve_value_local(f);
        initial.set_undefined(f);
        let stored = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(&initial, f), f);
        let list = s.reserve_gc_local(f).initialize(
            s.array_type::<ValueArray>()
                .filled(GcOperand::reference(&stored, s), length, f),
            f,
        );
        let cursor = s.reserve_gc_local(f).initialize(
            s.field(IteratorBufferSchema::HEAD)
                .read(buffer, s, f)
                .reference(),
            f,
        );
        let index = s.reserve_i32_local(f);
        f.instruction(&Instruction::I32Const(0));
        index.store(f);
        let done = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        cursor.load(s, f);
        f.instruction(&Instruction::RefIsNull);
        self.emit_branch_if_to_target(done, f);
        let node = s
            .reserve_gc_local(f)
            .initialize(cursor.load(s, f).require_non_null(f), f);
        let entry = s.reserve_gc_local(f).initialize(
            s.field(ArgumentListNodeSchema::VALUE)
                .read(&node, s, f)
                .reference(),
            f,
        );
        s.array_type::<ValueArray>()
            .write(&list, index, GcOperand::reference(&entry, s), s, f);
        cursor.replace(
            s.field(ArgumentListNodeSchema::NEXT)
                .read(&node, s, f)
                .reference(),
            f,
        );
        entry.clear(f);
        node.clear(f);
        index.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        index.store(f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        let array = self.emit_array_from_argument_list(&list, f)?;
        s.release_i32_local(index, f);
        s.release_i32_local(length, f);
        s.release_i64_local(count, f);
        cursor.clear(f);
        list.clear(f);
        stored.clear(f);
        initial.clear(f);
        Ok(array)
    }
}
