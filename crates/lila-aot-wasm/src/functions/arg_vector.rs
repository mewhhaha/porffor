use super::*;
use crate::gc_types::arg_vector;

impl FunctionBuilder<'_> {
    /// Capture already evaluated arguments before their scalar locals are reused.
    pub(crate) fn emit_arg_vector_new_fixed(
        &mut self,
        args: &[TaggedLocals],
        destination: ArgVectorLocal,
        function: &mut Function,
    ) {
        if args.is_empty() {
            arg_vector::emit_null(function);
        } else {
            arg_vector::emit_new_fixed(function, args);
        }
        function.instruction(&Instruction::LocalSet(destination.index()));
    }

    pub(crate) fn emit_arg_vector_new_len(
        &mut self,
        argc_local: u32,
        destination: ArgVectorLocal,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::I64Const((u32::MAX / 2) as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(argc_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        arg_vector::emit_new_default(function);
        function.instruction(&Instruction::LocalSet(destination.index()));
    }

    pub(crate) fn emit_arg_vector_read(
        &mut self,
        source: ArgVectorLocal,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(source.index()));
        function.instruction(&Instruction::RefIsNull);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(source.index()));
        arg_vector::emit_len(function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source.index()));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        arg_vector::emit_get(function);
        function.instruction(&Instruction::LocalSet(tag_local));

        function.instruction(&Instruction::LocalGet(source.index()));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        arg_vector::emit_get(function);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    /// The caller must use an index below the destination's allocated
    /// argument count. Only internal copy and concatenation loops call this.
    pub(crate) fn emit_arg_vector_write(
        &mut self,
        destination: ArgVectorLocal,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(destination.index()));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(tag_local));
        arg_vector::emit_set(function);

        function.instruction(&Instruction::LocalGet(destination.index()));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(payload_local));
        arg_vector::emit_set(function);
    }

    pub(crate) fn emit_arg_vector_len_to_local(
        &mut self,
        source: ArgVectorLocal,
        length_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(source.index()));
        function.instruction(&Instruction::RefIsNull);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(source.index()));
        arg_vector::emit_len(function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(length_local));
    }

    /// Copy a private, dense canonical Array into the unobservable call vector.
    pub(crate) fn emit_arg_vector_from_array(
        &mut self,
        source_array_local: u32,
        destination: ArgVectorLocal,
        function: &mut Function,
    ) {
        let len_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            source_array_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            source_array_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.emit_arg_vector_new_len(len_local, destination, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, tag_local, function);
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.emit_arg_vector_write(destination, index_local, payload_local, tag_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(len_local);
    }

    /// Append to a private argument staging Array without invoking JS property
    /// assignment or consulting the Array prototype. The caller increments
    /// `index_local` after this own-data write.
    pub(crate) fn emit_dense_call_staging_append(
        &mut self,
        array_local: u32,
        index_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, cap_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_grow_buffer(
            array_local,
            buffer_local,
            len_local,
            cap_local,
            index_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, tag_local, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_NORMAL_DATA,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.store_i64_local_at_offset(array_local, HEAP_LEN_OFFSET, len_local, function);

        self.release_temp_local(entry_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    /// Materialize a JS Array only where call arguments become observable or
    /// must outlive this Wasm invocation in a linear-memory activation record.
    pub(crate) fn emit_arg_vector_snapshot_array(
        &mut self,
        source: ArgVectorLocal,
        destination_array_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let len_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        self.emit_arg_vector_len_to_local(source, len_local, function);
        self.emit_alloc_array_with_len_local(
            len_local,
            destination_array_local,
            buffer_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_arg_vector_read(source, index_local, payload_local, tag_local, function);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, tag_local, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_NORMAL_DATA,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(len_local);
        Ok(())
    }
}
