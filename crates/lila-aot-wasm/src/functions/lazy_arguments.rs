use super::*;
use crate::arguments_protocol::MappedArgumentsPlan;
use crate::gc_types::legacy_arguments::{self as frames, Field};

impl FunctionBuilder<'_> {
    pub(crate) fn emit_defer_legacy_arguments(
        &mut self,
        plan: &MappedArgumentsPlan,
        function: &mut Function,
    ) {
        let frame = self.reserve_legacy_arguments_frame();
        let mapping = self.reserve_arg_vector_local();
        let length = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let flags = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        let callee = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.params.len() as i64));
        function.instruction(&Instruction::LocalSet(length));
        self.emit_arg_vector_new_len(length, mapping, function);
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag));
        for entry in plan.entries().iter().copied() {
            function.instruction(&Instruction::I64Const(entry.argument_index_i64()));
            function.instruction(&Instruction::LocalSet(index));
            function.instruction(&Instruction::I64Const(
                StoredPropertyAttributes::Data {
                    writable: false,
                    enumerable: false,
                    configurable: false,
                }
                .descriptor_word()
                .with_flags(DescriptorFlags {
                    array_own_property: false,
                    mapped: Some(entry.mapped_slot()),
                })
                .as_i64(),
            ));
            function.instruction(&Instruction::LocalSet(flags));
            self.emit_arg_vector_write(mapping, index, flags, tag, function);
        }
        self.load_i64_to_local_from_offset(
            self.class_function_context_local,
            HEAP_CLASS_FUNCTION_CONTEXT_ACTIVE_FUNCTION_OFFSET,
            callee,
            function,
        );
        function.instruction(&Instruction::GlobalGet(frames::GLOBAL_INDEX));
        function.instruction(&Instruction::LocalGet(
            self.arg_vector_param_local().index(),
        ));
        function.instruction(&Instruction::LocalGet(mapping.index()));
        function.instruction(&Instruction::LocalGet(callee));
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalGet(self.class_function_context_local));
        function.instruction(&Instruction::I64Const(0));
        frames::emit_new(function);
        function.instruction(&Instruction::LocalTee(frame.index()));
        function.instruction(&Instruction::GlobalSet(frames::GLOBAL_INDEX));
        self.deferred_arguments_frame = Some(frame);
        self.release_temp_local(callee);
        self.release_temp_local(tag);
        self.release_temp_local(flags);
        self.release_temp_local(index);
        self.release_temp_local(length);
        self.release_arg_vector_local(mapping);
    }

    /// Run at the actual own-property observation, including each prototype
    /// visited by Get and each direct target inspected by a Proxy invariant.
    /// The key is already a canonical property key, so this invokes no source
    /// conversion and cannot reorder user effects.
    pub(crate) fn emit_observe_legacy_arguments(
        &mut self,
        object: u32,
        object_tag: u32,
        key: u32,
        function: &mut Function,
    ) {
        let Some(base) = self.heap_alloc_function_index else {
            return;
        };
        let literal = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(object_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("arguments")));
        function.instruction(&Instruction::LocalSet(literal));
        self.emit_property_key_payload_equality_i32(key, literal, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(object));
        for _ in 0..5 {
            function.instruction(&Instruction::I64Const(0));
        }
        crate::gc_types::arg_vector::emit_null(function);
        function.instruction(&Instruction::Call(
            RuntimeHelperId::MaterializeLegacyArguments.index(base),
        ));
        for _ in 0..4 {
            function.instruction(&Instruction::Drop);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(literal);
    }

    /// Materialization is a bounded helper: it allocates the canonical mapped
    /// object and writes its existing non-configurable data property directly.
    /// It calls no JavaScript, getter, or Proxy trap.
    pub(crate) fn compile_materialize_legacy_arguments_helper(
        &mut self,
    ) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::MaterializeLegacyArguments);
        let frame = self.reserve_legacy_arguments_frame();
        let mapping = self.reserve_arg_vector_local();
        let callee = self.reserve_temp_local();
        let materialized = self.reserve_temp_local();
        let buffer = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let entry = self.reserve_temp_local();
        let key = self.reserve_temp_local();
        let literal = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(frames::GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(frame.index()));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(frame.index()));
        function.instruction(&Instruction::RefIsNull);
        function.instruction(&Instruction::BrIf(1));
        frames::emit_get(&mut function, frame, Field::Callee);
        function.instruction(&Instruction::LocalTee(callee));
        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        frames::emit_get(&mut function, frame, Field::Materialized);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        frames::emit_get(&mut function, frame, Field::Arguments);
        function.instruction(&Instruction::LocalSet(
            self.arg_vector_param_local().index(),
        ));
        frames::emit_get(&mut function, frame, Field::Mapping);
        function.instruction(&Instruction::LocalSet(mapping.index()));
        frames::emit_get(&mut function, frame, Field::Environment);
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        frames::emit_get(&mut function, frame, Field::Context);
        function.instruction(&Instruction::LocalSet(self.class_function_context_local));
        self.emit_arguments_object_payload(
            &PresentArgumentsObjectProtocol::MappedFromFrame(mapping),
            &mut function,
        )?;
        function.instruction(&Instruction::LocalSet(materialized));
        frames::emit_materialized_set(&mut function, frame, materialized);

        // Named storage may have grown while this invocation was suspended.
        // Find the entry afresh; never retain a linear entry address in a frame.
        self.load_i64_to_local_from_offset(callee, HEAP_PTR_OFFSET, buffer, &mut function);
        self.load_i64_to_local_from_offset(callee, HEAP_LEN_OFFSET, length, &mut function);
        function.instruction(&Instruction::I64Const(self.strings.payload("arguments")));
        function.instruction(&Instruction::LocalSet(literal));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(buffer));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry));
        self.load_i64_to_local_from_offset(entry, HEAP_OBJECT_KEY_OFFSET, key, &mut function);
        self.emit_property_key_payload_equality_i32(key, literal, &mut function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            entry,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            materialized,
            &mut function,
        );
        self.store_i64_const_at_offset(
            entry,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            ValueKind::Arguments.tag() as u64,
            &mut function,
        );
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        frames::emit_get(&mut function, frame, Field::Parent);
        function.instruction(&Instruction::LocalSet(frame.index()));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for _ in 0..4 {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }
}
