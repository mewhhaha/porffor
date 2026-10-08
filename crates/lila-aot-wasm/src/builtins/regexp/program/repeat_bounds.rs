//! Exact decimal archive admission over the retained immutable GC byte owner.
use super::*;

impl FunctionBuilder<'_> {
    fn emit_regexp_require_canonical_bound_digits(
        &mut self,
        view: &ValidatedRegExpProgramLayoutLocals,
        start: I64Local,
        length: I64Local,
        payload: I64Local,
        state_bytes: I64Local,
        rejected: ControlTarget,
        function: &mut Function,
    ) {
        start.load(function);
        payload.load(function);
        function.instruction(&Instruction::I64Ne);
        self.emit_branch_if_to_target(rejected, function);
        length.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.emit_branch_if_to_target(rejected, function);
        payload.load(function);
        view.end().load(function);
        function.instruction(&Instruction::I64GtU);
        self.emit_branch_if_to_target(rejected, function);
        length.load(function);
        view.end().load(function);
        payload.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64GtU);
        self.emit_branch_if_to_target(rejected, function);
        let cursor = self.runtime_schema().reserve_i64_local(function);
        let digit = self.runtime_schema().reserve_i64_local(function);
        let end = self.runtime_schema().reserve_i64_local(function);
        start.load(function);
        cursor.store(function);
        start.load(function);
        length.load(function);
        function.instruction(&Instruction::I64Add);
        end.store(function);
        view.read_byte_in_range(start, digit, self, function);
        digit.load(function);
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Eq);
        length.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32And);
        self.emit_branch_if_to_target(rejected, function);
        let checked = self.open_frame(ControlFrameKind::Block, function);
        let checking = self.open_frame(ControlFrameKind::Loop, function);
        cursor.load(function);
        end.load(function);
        function.instruction(&Instruction::I64Eq);
        self.emit_branch_if_to_target(checked, function);
        view.read_byte_in_range(cursor, digit, self, function);
        digit.load(function);
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64LtU);
        digit.load(function);
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        self.emit_branch_if_to_target(rejected, function);
        cursor.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        cursor.store(function);
        self.emit_branch_to_target(checking, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        end.load(function);
        payload.store(function);
        state_bytes.load(function);
        length.load(function);
        function.instruction(&Instruction::I64Const(
            (REGEXP_REPEAT_COUNTER_DECIMAL_DIGITS - 1) as i64,
        ));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(
            REGEXP_REPEAT_COUNTER_DECIMAL_DIGITS as i64,
        ));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Const(
            REGEXP_REPEAT_COUNTER_LIMB_WIDTH as i64,
        ));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        state_bytes.store(function);
        for local in [end, digit, cursor] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    fn emit_regexp_require_ordered_bound_digits(
        &mut self,
        view: &ValidatedRegExpProgramLayoutLocals,
        minimum: (I64Local, I64Local),
        maximum: (I64Local, I64Local),
        rejected: ControlTarget,
        function: &mut Function,
    ) {
        minimum.1.load(function);
        maximum.1.load(function);
        function.instruction(&Instruction::I64GtU);
        self.emit_branch_if_to_target(rejected, function);
        minimum.1.load(function);
        maximum.1.load(function);
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        let index = self.runtime_schema().reserve_i64_local(function);
        let address = self.runtime_schema().reserve_i64_local(function);
        let left = self.runtime_schema().reserve_i64_local(function);
        let right = self.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        index.store(function);
        let ordered = self.open_frame(ControlFrameKind::Block, function);
        let comparing = self.open_frame(ControlFrameKind::Loop, function);
        index.load(function);
        minimum.1.load(function);
        function.instruction(&Instruction::I64Eq);
        self.emit_branch_if_to_target(ordered, function);
        for (base, digit) in [(minimum.0, left), (maximum.0, right)] {
            base.load(function);
            index.load(function);
            function.instruction(&Instruction::I64Add);
            address.store(function);
            view.read_byte_in_range(address, digit, self, function);
        }
        left.load(function);
        right.load(function);
        function.instruction(&Instruction::I64LtU);
        self.emit_branch_if_to_target(ordered, function);
        left.load(function);
        right.load(function);
        function.instruction(&Instruction::I64GtU);
        self.emit_branch_if_to_target(rejected, function);
        index.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        index.store(function);
        self.emit_branch_to_target(comparing, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        for local in [right, left, address, index] {
            self.runtime_schema().release_i64_local(local, function);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }

    pub(super) fn emit_validate_regexp_repeat_bounds(
        &mut self,
        view: &ValidatedRegExpProgramLayoutLocals,
        rejected: ControlTarget,
        function: &mut Function,
    ) {
        let slot = self.runtime_schema().reserve_i64_local(function);
        let row = self.runtime_schema().reserve_i64_local(function);
        let address = self.runtime_schema().reserve_i64_local(function);
        let minimum_start = self.runtime_schema().reserve_i64_local(function);
        let minimum_length = self.runtime_schema().reserve_i64_local(function);
        let maximum_kind = self.runtime_schema().reserve_i64_local(function);
        let maximum_start = self.runtime_schema().reserve_i64_local(function);
        let maximum_length = self.runtime_schema().reserve_i64_local(function);
        let state_offset = self.runtime_schema().reserve_i64_local(function);
        let state_bytes = self.runtime_schema().reserve_i64_local(function);
        let payload = self.runtime_schema().reserve_i64_local(function);
        let padded = self.runtime_schema().reserve_i64_local(function);
        let byte = self.runtime_schema().reserve_i64_local(function);
        view.sections.range_end.load(function);
        view.sections.repeat_bounds.store(function);
        view.repeat_bounds().load(function);
        view.repeat_slot_count().load(function);
        function.instruction(&Instruction::I64Const(
            REGEXP_REPEAT_BOUND_RECORD_SIZE as i64,
        ));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        payload.store(function);
        payload.load(function);
        view.end().load(function);
        function.instruction(&Instruction::I64GtU);
        self.emit_branch_if_to_target(rejected, function);
        for local in [slot, state_bytes] {
            function.instruction(&Instruction::I64Const(0));
            local.store(function);
        }
        let admitted = self.open_frame(ControlFrameKind::Block, function);
        let admitting = self.open_frame(ControlFrameKind::Loop, function);
        slot.load(function);
        view.repeat_slot_count().load(function);
        function.instruction(&Instruction::I64Eq);
        self.emit_branch_if_to_target(admitted, function);
        view.repeat_bounds().load(function);
        slot.load(function);
        function.instruction(&Instruction::I64Const(
            REGEXP_REPEAT_BOUND_RECORD_SIZE as i64,
        ));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        row.store(function);
        for (word, output) in [
            (RegExpRepeatBoundWord::MinimumDigitsOffset, minimum_start),
            (RegExpRepeatBoundWord::MinimumDigitsLength, minimum_length),
            (RegExpRepeatBoundWord::MaximumKind, maximum_kind),
            (RegExpRepeatBoundWord::MaximumDigitsOffset, maximum_start),
            (RegExpRepeatBoundWord::MaximumDigitsLength, maximum_length),
            (RegExpRepeatBoundWord::StateOffset, state_offset),
        ] {
            row.load(function);
            function.instruction(&Instruction::I64Const(word.offset() as i64));
            function.instruction(&Instruction::I64Add);
            address.store(function);
            view.read_word_in_range(address, output, self, function);
        }
        state_offset.load(function);
        state_bytes.load(function);
        function.instruction(&Instruction::I64Ne);
        self.emit_branch_if_to_target(rejected, function);
        state_bytes.load(function);
        function.instruction(&Instruction::I64Const(
            REGEXP_REPEAT_STATE_HEADER_SIZE as i64,
        ));
        function.instruction(&Instruction::I64Add);
        state_bytes.store(function);
        self.emit_regexp_require_canonical_bound_digits(
            view,
            minimum_start,
            minimum_length,
            payload,
            state_bytes,
            rejected,
            function,
        );
        maximum_kind.load(function);
        function.instruction(&Instruction::I64Const(
            RegExpRepeatMaximumKind::Unbounded.word() as i64,
        ));
        function.instruction(&Instruction::I64GtU);
        self.emit_branch_if_to_target(rejected, function);
        maximum_kind.load(function);
        function.instruction(&Instruction::I64Const(
            RegExpRepeatMaximumKind::Finite.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_regexp_require_canonical_bound_digits(
            view,
            maximum_start,
            maximum_length,
            payload,
            state_bytes,
            rejected,
            function,
        );
        self.emit_regexp_require_ordered_bound_digits(
            view,
            (minimum_start, minimum_length),
            (maximum_start, maximum_length),
            rejected,
            function,
        );
        function.instruction(&Instruction::Else);
        maximum_start.load(function);
        maximum_length.load(function);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.emit_branch_if_to_target(rejected, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        state_bytes.load(function);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(-8));
        function.instruction(&Instruction::I64And);
        state_bytes.store(function);
        slot.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        slot.store(function);
        self.emit_branch_to_target(admitting, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        state_bytes.load(function);
        view.repeat_state_byte_length().load(function);
        function.instruction(&Instruction::I64Ne);
        self.emit_branch_if_to_target(rejected, function);
        payload.load(function);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(-8));
        function.instruction(&Instruction::I64And);
        padded.store(function);
        padded.load(function);
        view.end().load(function);
        function.instruction(&Instruction::I64GtU);
        self.emit_branch_if_to_target(rejected, function);
        let padding_done = self.open_frame(ControlFrameKind::Block, function);
        let padding = self.open_frame(ControlFrameKind::Loop, function);
        payload.load(function);
        padded.load(function);
        function.instruction(&Instruction::I64Eq);
        self.emit_branch_if_to_target(padding_done, function);
        view.read_byte_in_range(payload, byte, self, function);
        byte.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.emit_branch_if_to_target(rejected, function);
        payload.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        payload.store(function);
        self.emit_branch_to_target(padding, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        padded.load(function);
        view.sections.repeat_end.store(function);
        for local in [
            byte,
            padded,
            payload,
            state_bytes,
            state_offset,
            maximum_length,
            maximum_start,
            maximum_kind,
            minimum_length,
            minimum_start,
            address,
            row,
            slot,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }
}
