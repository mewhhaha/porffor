//! Finite compiler-owned RegExp candidate lookup and descriptor publication.

use super::*;

impl FunctionBuilder<'_> {
    /// The program half: the literal `(source, flags)` pairs the compiler
    /// already turned into validated programs. Status `SyntaxError` is a
    /// literal pair the compiler rejected; a miss is a null program with
    /// `Compiled`, which sends the runtime to its own compiler.
    pub(crate) fn compile_regexp_program_candidate_hook(&mut self) -> Result<Function, EmitError> {
        use crate::data::{
            RuntimeRegExpEntryKind, RUNTIME_REGEXP_RECORD_ENTRY_KIND_WORD,
            RUNTIME_REGEXP_RECORD_FLAGS_WORD, RUNTIME_REGEXP_RECORD_PROGRAM_PAYLOAD_WORD,
            RUNTIME_REGEXP_RECORD_SIZE, RUNTIME_REGEXP_RECORD_SOURCE_WORD,
        };
        use crate::runtime_helpers::{
            HelperParameters, RegExpCompilerStatus, RegExpProgramCandidateParameters,
        };
        let mut function = self.begin_helper_body(RuntimeHelperId::RegExpProgramCandidate);
        let parameters = self.helper_parameters::<RegExpProgramCandidateParameters>(&mut function);
        self.push_scope();
        let schema = self.runtime_schema();
        let program = schema
            .reserve_gc_local::<crate::gc_types::RegExpProgram, Nullable>(&mut function)
            .initialize_null(schema, &mut function);
        let status = schema.reserve_i32_local(&mut function);
        status.set_constant(
            RegExpCompilerStatus::Compiled.abi_word() as i32,
            &mut function,
        );
        if let Some(candidates) = self.strings.runtime_regexp_table() {
            let table = schema
                .reserve_gc_local::<crate::gc_types::PooledStringTable, NonNullable>(&mut function)
                .initialize(
                    schema.pooled_string_table_reference(&mut function),
                    &mut function,
                );
            let pointer = schema.reserve_i32_local(&mut function);
            let row = schema.reserve_i32_local(&mut function);
            let ordinal = schema.reserve_i32_local(&mut function);
            let recognized = schema.reserve_i32_local(&mut function);
            let kind_word = schema.reserve_i64_local(&mut function);
            let payload = schema.reserve_i64_local(&mut function);
            candidates.emit_start(&mut function);
            pointer.store(&mut function);
            row.set_constant(0, &mut function);
            let done = self.open_frame(ControlFrameKind::Block, &mut function);
            let next = self.open_frame(ControlFrameKind::Loop, &mut function);
            row.load(&mut function);
            function.instruction(&Instruction::I32Const(candidates.count() as i32));
            function.instruction(&Instruction::I32GeU);
            self.emit_branch_if_to_target(done, &mut function);
            candidates.load_word(pointer, RUNTIME_REGEXP_RECORD_SOURCE_WORD, &mut function);
            function.instruction(&Instruction::I32WrapI64);
            ordinal.store(&mut function);
            let source = schema
                .reserve_gc_local::<StringValue, NonNullable>(&mut function)
                .initialize(
                    schema
                        .array_type::<crate::gc_types::PooledStringTable>()
                        .read(&table, ordinal, schema, &mut function)
                        .reference()
                        .require_non_null(&mut function),
                    &mut function,
                );
            candidates.load_word(pointer, RUNTIME_REGEXP_RECORD_FLAGS_WORD, &mut function);
            function.instruction(&Instruction::I32WrapI64);
            ordinal.store(&mut function);
            let flags = schema
                .reserve_gc_local::<StringValue, NonNullable>(&mut function)
                .initialize(
                    schema
                        .array_type::<crate::gc_types::PooledStringTable>()
                        .read(&table, ordinal, schema, &mut function)
                        .reference()
                        .require_non_null(&mut function),
                    &mut function,
                );
            self.emit_string_payload_equality_i32(&parameters.source, &source, &mut function);
            self.emit_string_payload_equality_i32(&parameters.flags, &flags, &mut function);
            function.instruction(&Instruction::I32And);
            flags.clear(&mut function);
            source.clear(&mut function);
            self.open_frame(ControlFrameKind::If, &mut function);
            candidates.load_word(
                pointer,
                RUNTIME_REGEXP_RECORD_ENTRY_KIND_WORD,
                &mut function,
            );
            kind_word.store(&mut function);
            recognized.set_constant(0, &mut function);
            for kind in RuntimeRegExpEntryKind::ALL {
                kind_word.load(&mut function);
                function.instruction(&Instruction::I64Const(kind.word() as i64));
                function.instruction(&Instruction::I64Eq);
                self.open_frame(ControlFrameKind::If, &mut function);
                recognized.set_constant(1, &mut function);
                if kind.throws_syntax_error() {
                    status.set_constant(
                        RegExpCompilerStatus::SyntaxError.abi_word() as i32,
                        &mut function,
                    );
                }
                match kind {
                    RuntimeRegExpEntryKind::Program => {
                        candidates.load_word(
                            pointer,
                            RUNTIME_REGEXP_RECORD_PROGRAM_PAYLOAD_WORD,
                            &mut function,
                        );
                        payload.store(&mut function);
                        program.replace(
                            self.emit_static_regexp_program_payload(payload, &mut function)
                                .nullable(),
                            &mut function,
                        );
                        self.emit_branch_to_target(done, &mut function);
                    }
                    RuntimeRegExpEntryKind::Rejected => {
                        self.emit_branch_to_target(done, &mut function);
                    }
                    RuntimeRegExpEntryKind::Unsupported => {}
                }
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
            recognized.load(&mut function);
            function.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, &mut function);
            status.set_constant(
                RegExpCompilerStatus::CorruptProgram.abi_word() as i32,
                &mut function,
            );
            self.emit_branch_to_target(done, &mut function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            row.load(&mut function);
            function.instruction(&Instruction::I32Const(1));
            function.instruction(&Instruction::I32Add);
            row.store(&mut function);
            pointer.load(&mut function);
            function.instruction(&Instruction::I32Const(RUNTIME_REGEXP_RECORD_SIZE as i32));
            function.instruction(&Instruction::I32Add);
            pointer.store(&mut function);
            self.emit_branch_to_target(next, &mut function);
            self.pop_control(ControlFrameKind::Loop);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::Block);
            function.instruction(&Instruction::End);
            table.clear(&mut function);
            for local in [pointer, row, ordinal, recognized] {
                schema.release_i32_local(local, &mut function);
            }
            for local in [kind_word, payload] {
                schema.release_i64_local(local, &mut function);
            }
        }
        self.pop_scope();
        parameters.release(&mut function);
        // Result order: program, status, cursor, detail.
        program.load(schema, &mut function);
        status.load(&mut function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(0));
        program.clear(&mut function);
        schema.release_i32_local(status, &mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// The descriptor is a compiler-validated static span, never a guest input.
    /// Copy its exact active-memory bytes into one unpublished GC construction.
    fn emit_static_regexp_program_payload(
        &mut self,
        payload: I64Local,
        function: &mut Function,
    ) -> crate::gc_types::GcStackReference<crate::gc_types::RegExpProgram> {
        let schema = self.runtime_schema();
        let length = schema.reserve_i32_local(function);
        let pointer = schema.reserve_i32_local(function);
        let index = schema.reserve_i32_local(function);
        let byte = schema.reserve_i32_local(function);
        payload.load(function);
        function.instruction(&Instruction::I32WrapI64);
        length.store(function);
        payload.load(function);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I32WrapI64);
        pointer.store(function);
        index.set_constant(0, function);
        let construction =
            crate::gc_types::RegExpProgramConstruction::allocate(schema, length, function);
        let done = self.open_frame(ControlFrameKind::Block, function);
        let next = self.open_frame(ControlFrameKind::Loop, function);
        index.load(function);
        length.load(function);
        function.instruction(&Instruction::I32GeU);
        self.emit_branch_if_to_target(done, function);
        pointer.load(function);
        index.load(function);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        byte.store(function);
        construction.write(index, byte, schema, function);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        self.emit_branch_to_target(next, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        let program = construction.publish(schema, function);
        for local in [length, pointer, index, byte] {
            schema.release_i32_local(local, function);
        }
        program
    }
}
