use super::*;

impl FunctionBuilder<'_> {
    /// Finish actual canonical set keys into the consumed finite-atom payload.
    /// One key position becomes one ordinary matcher instruction, without
    /// expanding source-sized parser nodes or spending the instruction budget.
    pub(in crate::builtins::regexp::compiler::parser) fn emit_regexp_finish_finite_class_set(
        &mut self,
        compiler: &CompilerLocals,
        mode: &CompilerCharacterMode,
        address: I64Local,
        modifiers: &ModifierLocals,
        strings: &FiniteStringSet,
        function: &mut Function,
    ) {
        let key = self.runtime_schema().reserve_i64_local(function);
        let length = self.runtime_schema().reserve_i64_local(function);
        let count = self.runtime_schema().reserve_i64_local(function);
        let empty = self.runtime_schema().reserve_i64_local(function);
        let header = self.runtime_schema().reserve_i64_local(function);
        let rows = self.runtime_schema().reserve_i64_local(function);
        let singleton_opcode = self.runtime_schema().reserve_i64_local(function);
        let singleton_a = self.runtime_schema().reserve_i64_local(function);
        let singleton_b = self.runtime_schema().reserve_i64_local(function);
        let filled = self.runtime_schema().reserve_i64_local(function);
        let points = self.runtime_schema().reserve_i64_local(function);
        let positions = self.runtime_schema().reserve_i64_local(function);
        let position = self.runtime_schema().reserve_i64_local(function);
        let point = self.runtime_schema().reserve_i64_local(function);
        let instruction = self.runtime_schema().reserve_i64_local(function);
        let insertion = self.runtime_schema().reserve_i64_local(function);
        let shift = self.runtime_schema().reserve_i64_local(function);
        let previous = self.runtime_schema().reserve_i64_local(function);
        let source_row = self.runtime_schema().reserve_i64_local(function);
        let destination_row = self.runtime_schema().reserve_i64_local(function);
        let row_pointer = self.runtime_schema().reserve_i64_local(function);
        let row_length = self.runtime_schema().reserve_i64_local(function);
        let flags = self.runtime_schema().reserve_i64_local(function);
        let one = self.runtime_schema().reserve_i64_local(function);
        set(function, count, 0);
        set(function, empty, 0);
        copy(function, key, strings.head);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        eq(function, key, 0);
        function.instruction(&Instruction::BrIf(1));
        load(function, key, FiniteKeyWord::CodePointLength as u64, length);
        eq(function, length, 0);
        function.instruction(&Instruction::If(BlockType::Empty));
        set(function, empty, 1);
        function.instruction(&Instruction::Else);
        eq(function, length, 1);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(count, 1, function);
        function.instruction(&Instruction::End);
        load(function, key, FiniteKeyWord::Next as u64, key);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // The singleton producer owns its fold-closed bitmap and range budget.
        // Snapshot its actual instruction before Operand0 becomes a header.
        set(function, one, 0);
        self.emit_regexp_parser_bitmap_ranges(compiler, mode, address, one, modifiers, function);
        load(function, address, NodeWord::Opcode as u64, singleton_opcode);
        load(function, address, NodeWord::Operand0 as u64, singleton_a);
        load(function, address, NodeWord::Operand1 as u64, singleton_b);
        load(function, address, NodeWord::Flags as u64, flags);
        flags.load(function);
        function.instruction(&Instruction::I64Const(!(NODE_ATOM_NULLABLE as i64)));
        function.instruction(&Instruction::I64And);
        empty.load(function);
        function.instruction(&Instruction::I64Const(NODE_ATOM_NULLABLE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Or);
        flags.store(function);
        store(function, address, NodeWord::Flags as u64, flags);
        count.load(function);
        empty.load(function);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        set(function, rows, 0);
        eq(function, count, 0);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_finite_workspace_elements(
            compiler,
            rows,
            count,
            FiniteClassSetStringWord::BYTES,
            function,
        );
        function.instruction(&Instruction::End);
        set(function, filled, 0);
        copy(function, key, strings.head);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        eq(function, key, 0);
        function.instruction(&Instruction::BrIf(1));
        load(function, key, FiniteKeyWord::CodePointLength as u64, length);
        eq(function, length, 0);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        load(
            function,
            key,
            FiniteKeyWord::CodePointsPointer as u64,
            points,
        );
        self.emit_regexp_finite_workspace_elements(
            compiler,
            positions,
            length,
            REGEXP_INSTRUCTION_WIDTH as u64,
            function,
        );
        set(function, position, 0);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        position.load(function);
        length.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        points.load(function);
        position.load(function);
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg64(0)));
        point.store(function);
        mode.emit_require_character(self, compiler, point, function);
        positions.load(function);
        position.load(function);
        function.instruction(&Instruction::I64Const(REGEXP_INSTRUCTION_WIDTH as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        instruction.store(function);
        self.emit_regexp_prepare_finite_position(
            compiler,
            mode,
            modifiers,
            point,
            instruction,
            function,
        );
        self.emit_regexp_scratch_increment(position, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Parent keys are code-point lexicographic. Stable descending-length
        // insertion therefore retains the native producer's equal-length order.
        set(function, insertion, 0);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        insertion.load(function);
        filled.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        self.emit_regexp_finite_row_address(rows, insertion, source_row, function);
        load(
            function,
            source_row,
            FiniteClassSetStringWord::CodePointLength as u64,
            row_length,
        );
        row_length.load(function);
        length.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_regexp_scratch_increment(insertion, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        copy(function, shift, filled);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        shift.load(function);
        insertion.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        copy(function, previous, shift);
        self.emit_regexp_scratch_increment(previous, -1, function);
        self.emit_regexp_finite_row_address(rows, previous, source_row, function);
        self.emit_regexp_finite_row_address(rows, shift, destination_row, function);
        load(
            function,
            source_row,
            FiniteClassSetStringWord::InstructionsPointer as u64,
            row_pointer,
        );
        load(
            function,
            source_row,
            FiniteClassSetStringWord::CodePointLength as u64,
            row_length,
        );
        store(
            function,
            destination_row,
            FiniteClassSetStringWord::InstructionsPointer as u64,
            row_pointer,
        );
        store(
            function,
            destination_row,
            FiniteClassSetStringWord::CodePointLength as u64,
            row_length,
        );
        copy(function, shift, previous);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_regexp_finite_row_address(rows, insertion, destination_row, function);
        store(
            function,
            destination_row,
            FiniteClassSetStringWord::InstructionsPointer as u64,
            positions,
        );
        store(
            function,
            destination_row,
            FiniteClassSetStringWord::CodePointLength as u64,
            length,
        );
        self.emit_regexp_scratch_increment(filled, 1, function);
        function.instruction(&Instruction::End);
        load(function, key, FiniteKeyWord::Next as u64, key);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        filled.load(function);
        count.load(function);
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::End);
        set(function, one, 1);
        self.emit_regexp_finite_workspace_elements(
            compiler,
            header,
            one,
            FiniteClassSetAtomWord::BYTES,
            function,
        );
        for (word, local) in [
            (FiniteClassSetAtomWord::RowsPointer, rows),
            (FiniteClassSetAtomWord::RowCount, count),
            (FiniteClassSetAtomWord::SingletonOpcode, singleton_opcode),
            (FiniteClassSetAtomWord::SingletonOperand0, singleton_a),
            (FiniteClassSetAtomWord::SingletonOperand1, singleton_b),
            (FiniteClassSetAtomWord::ContainsEmpty, empty),
        ] {
            store(function, header, word as u64, local);
        }
        store_const(
            function,
            address,
            NodeWord::Kind as u64,
            NodeKind::FiniteClassSet as u64,
        );
        store_const(function, address, NodeWord::Opcode as u64, 0);
        store(function, address, NodeWord::Operand0 as u64, header);
        store_const(function, address, NodeWord::Operand1 as u64, 0);
        function.instruction(&Instruction::End);
        for local in [
            one,
            flags,
            row_length,
            row_pointer,
            destination_row,
            source_row,
            previous,
            shift,
            insertion,
            instruction,
            point,
            position,
            positions,
            points,
            filled,
            singleton_b,
            singleton_a,
            singleton_opcode,
            rows,
            header,
            empty,
            count,
            length,
            key,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    fn emit_regexp_finite_row_address(
        &self,
        rows: I64Local,
        index: I64Local,
        address: I64Local,
        function: &mut Function,
    ) {
        rows.load(function);
        index.load(function);
        function.instruction(&Instruction::I64Const(
            FiniteClassSetStringWord::BYTES as i64,
        ));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        address.store(function);
    }

    fn emit_regexp_prepare_finite_position(
        &mut self,
        compiler: &CompilerLocals,
        mode: &CompilerCharacterMode,
        modifiers: &ModifierLocals,
        point: I64Local,
        instruction: I64Local,
        function: &mut Function,
    ) {
        let first = self.runtime_schema().reserve_i64_local(function);
        let index = self.runtime_schema().reserve_i64_local(function);
        let row = self.runtime_schema().reserve_i64_local(function);
        let source = self.runtime_schema().reserve_i64_local(function);
        let canonical = self.runtime_schema().reserve_i64_local(function);
        let inserted = self.runtime_schema().reserve_i64_local(function);
        let packed = self.runtime_schema().reserve_i64_local(function);
        let changed = self.runtime_schema().reserve_i64_local(function);
        let all_ascii = self.runtime_schema().reserve_i64_local(function);
        let ascii_low = self.runtime_schema().reserve_i64_local(function);
        let ascii_high = self.runtime_schema().reserve_i64_local(function);
        let run_start = self.runtime_schema().reserve_i64_local(function);
        let run_end = self.runtime_schema().reserve_i64_local(function);
        store_const(function, instruction, 0, REGEXP_OPCODE_LITERAL_CODE_POINT);
        store(function, instruction, 8, point);
        store_const(function, instruction, 16, 0);
        eq(function, modifiers.ignore_case, 1);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Match static apply_modifiers: an unchanged character stays literal,
        // and a changed, wholly ASCII equivalence class uses its two-word bitmap.
        // Decide before spending range entries, including at the shared cap.
        set(function, index, 0);
        set(function, changed, 0);
        set(function, all_ascii, 1);
        set(function, ascii_low, 0);
        set(function, ascii_high, 0);
        self.emit_regexp_finite_ascii_member(point, ascii_low, ascii_high, all_ascii, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        mode.fold_count().load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        mode.fold_table().load(function);
        index.load(function);
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        row.store(function);
        row.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load32U(Self::memarg32(0)));
        source.store(function);
        row.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load32U(Self::memarg32(4)));
        canonical.store(function);
        canonical.load(function);
        point.load(function);
        function.instruction(&Instruction::I64Eq);
        source.load(function);
        point.load(function);
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        set(function, changed, 1);
        self.emit_regexp_finite_ascii_member(source, ascii_low, ascii_high, all_ascii, function);
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        eq(function, changed, 1);
        function.instruction(&Instruction::If(BlockType::Empty));
        eq(function, all_ascii, 1);
        function.instruction(&Instruction::If(BlockType::Empty));
        store_const(function, instruction, 0, REGEXP_OPCODE_POSITIVE_ASCII_CLASS);
        store(function, instruction, 8, ascii_low);
        store(function, instruction, 16, ascii_high);
        function.instruction(&Instruction::Else);
        copy(function, first, compiler.range_count);
        set(function, index, 0);
        set(function, inserted, 0);
        set(function, run_start, u64::MAX);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        mode.fold_count().load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        mode.fold_table().load(function);
        index.load(function);
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        row.store(function);
        row.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load32U(Self::memarg32(0)));
        source.store(function);
        row.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load32U(Self::memarg32(4)));
        canonical.store(function);
        canonical.load(function);
        point.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        eq(function, inserted, 0);
        point.load(function);
        source.load(function);
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_finite_fold_member(compiler, point, run_start, run_end, function);
        set(function, inserted, 1);
        function.instruction(&Instruction::End);
        source.load(function);
        point.load(function);
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_finite_fold_member(compiler, source, run_start, run_end, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        eq(function, inserted, 0);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_finite_fold_member(compiler, point, run_start, run_end, function);
        function.instruction(&Instruction::End);
        self.emit_regexp_parser_append_range(compiler, run_start, run_end, function);
        store_const(function, instruction, 0, REGEXP_OPCODE_UNICODE_PROPERTY);
        store(function, instruction, 8, first);
        compiler.range_count.load(function);
        first.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        packed.store(function);
        store(function, instruction, 16, packed);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [
            run_end, run_start, ascii_high, ascii_low, all_ascii, changed, packed, inserted,
            canonical, source, row, index, first,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    fn emit_regexp_finite_ascii_member(
        &self,
        member: I64Local,
        low: I64Local,
        high: I64Local,
        all_ascii: I64Local,
        function: &mut Function,
    ) {
        member.load(function);
        function.instruction(&Instruction::I64Const(128));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        member.load(function);
        function.instruction(&Instruction::I64Const(64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        low.load(function);
        function.instruction(&Instruction::I64Const(1));
        member.load(function);
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        low.store(function);
        function.instruction(&Instruction::Else);
        high.load(function);
        function.instruction(&Instruction::I64Const(1));
        member.load(function);
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        high.store(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        set(function, all_ascii, 0);
        function.instruction(&Instruction::End);
    }

    /// The fold table is source-sorted. Coalesce adjacent preimages before
    /// reserving the same disjoint range slice as the static modifier owner.
    fn emit_regexp_finite_fold_member(
        &mut self,
        compiler: &CompilerLocals,
        member: I64Local,
        run_start: I64Local,
        run_end: I64Local,
        function: &mut Function,
    ) {
        eq(function, run_start, u64::MAX);
        function.instruction(&Instruction::If(BlockType::Empty));
        copy(function, run_start, member);
        function.instruction(&Instruction::Else);
        member.load(function);
        run_end.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_append_range(compiler, run_start, run_end, function);
        copy(function, run_start, member);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        copy(function, run_end, member);
    }
}
